//! 无头协议会话 — BrowserShell 数据模型 + renderer IPC 管线。
//!
//! 发布构建只持有 renderer IPC（进程隔离）；进程内 WebView 仅用于单元测试。

use serde_json::Value;
use std::sync::{Arc, Mutex};
use zero_browser_shell::BrowserShell;

use zero_net::cookie::CookieStore;
#[cfg(not(test))]
use zero_net::{HttpClient, HttpMethod, HttpRequest};
#[cfg(not(test))]
use zero_protocol::message::{
    AutomationOperation, AutomationRequest, AutomationResult, FetchParams, FramePublishMode, LoadHtmlParams,
};
use zero_protocol::message::{
    AutomationValue, ImeEventParams, ImeEventType, IndexedDbResponseParams, IpcColorScheme, IpcMediaType, IpcMessage,
    IpcMessageKind, KeyboardEventParams, KeyboardEventType, MouseEventParams, MouseEventType,
    NavigationCommittedParams, PreDocumentScriptsParams, ScrollEventParams, SetColorSchemeParams, SetMediaTypeParams,
    SetViewportParams,
};
use zero_protocol::paint_snapshot::IpcImagePayload;
#[cfg(not(test))]
use zero_protocol::process::RendererHandle;
#[cfg(test)]
use zero_webview::{WebView, WebViewConfig};

// ── 会话 ──

/// 已完成的代理 fetch（worker 线程 → 会话线程）。cookie 应用、Network 事件与
/// 响应回发全部由会话线程在排空时执行——会话状态保持单线程变更。
pub(super) struct CompletedFetch {
    pub(super) request_id: u64,
    pub(super) net_request_id: String,
    pub(super) network_enabled: bool,
    pub(super) origin_url: String,
    pub(super) outcome: Result<zero_net::HttpResponse, zero_net::NetError>,
}

/// Page.addScriptToEvaluateOnNewDocument 登记的注入脚本。
///（ZeroWeb 单引擎无 world 隔离：一律在主 world 执行，见矩阵 createIsolatedWorld 注记。）
pub(super) struct InjectedScript {
    pub(super) identifier: String,
    pub(super) source: String,
    /// 注册时的 worldName（如 Playwright 的 utility world）；None = 主 world。
    pub(super) world_name: Option<String>,
}

/// 在途导航登记——renderer `NavigationStarted`/`NavigationCommitted` 校验与 commit
/// 回声判定的会话侧依据（对齐 GUI process_backend 的 pending_indexed_db_navigations）。
pub(super) struct PendingNavigation {
    /// 登记导航的目标 URL。
    pub(super) url: String,
    /// 登记导航世代（会话命令分配，或 renderer 侧导航自增后采纳）。
    pub(super) epoch: u64,
    /// true = 会话命令发起（Page.navigate/LoadHtml——cmd_navigate 已先行写 shell
    /// URL，其 commit 与 shell 同 URL 时为回声）；false = renderer 侧发起
    ///（anchor/表单/location.href，FIXME(tab-ipc)：同 tab 提交面）。
    pub(super) client_initiated: bool,
    /// 该世代已有 URL 对齐的 `NavigationStarted` 到达。commit 凭此甄别「世代数值
    /// 碰撞的被取代导航 commit」（未确认 → 丢弃）与重定向最终 URL commit（放行）。
    pub(super) started_confirmed: bool,
}

/// 浏览器会话。发布构建只持有 renderer IPC；进程内 WebView 仅用于单元测试。
pub(super) struct HeadlessSession {
    /// 浏览器 shell（数据模型）。
    pub(super) shell: BrowserShell,
    /// 进程内页面渲染（仅单元测试）。
    #[cfg(test)]
    pub(super) webview: WebView,
    /// 独立页面渲染进程（发布构建）。
    #[cfg(not(test))]
    pub(super) renderer: RendererHandle,
    /// Browser 进程拥有网络能力，renderer 仅经 IPC 请求资源。
    #[cfg(not(test))]
    pub(super) http: HttpClient,
    /// 最新 renderer 绘制快照，供截图与 DOM 统计使用。
    #[cfg(not(test))]
    pub(super) snapshot: crate::tab_snapshot::TabSnapshot,
    /// 当前导航世代：会话命令发起时自增；renderer 侧导航自增后经
    /// [`Self::note_navigation_started`] 采纳。commit 校验与 wait_for_load 武装
    /// 门控的世代基准（测试构建无 renderer 消息面，字段仍随 commit 校验纯函数
    /// 参与单测）。
    pub(super) navigation_epoch: u64,
    /// 在途导航登记（命令时点建立、NavigationStarted 确认、commit 消费），见
    /// [`PendingNavigation`]。
    pub(super) pending_navigation: Option<PendingNavigation>,
    #[cfg(not(test))]
    pub(super) next_request_id: u64,
    /// addScriptToEvaluateOnNewDocument 注册的脚本（新文档加载后重放）。
    pub(super) injected_scripts: Vec<InjectedScript>,
    /// Cookie jar（Storage 域 + proxy_fetch 双向接线；与 WS [`zero_net::shared_cookie_store`] 同进程共享）。
    pub(super) cookie_store: Arc<Mutex<CookieStore>>,
    /// Emulation.setUserAgentOverride（None = 默认 UA）。
    pub(super) user_agent_override: Option<String>,
    /// Network.enable 门控（Network 域事件源开关）。
    pub(super) network_enabled: bool,
    /// Network 域事件队列（proxy_fetch 生命周期观测，transport 逐命令排空盖章）。
    /// 边界（t4 返修 D5）：条目不带归属 target 标签——排空时盖章为当前命令会话
    ///（空闲 drain 盖首个附接会话），多 target 并发下事件可能盖章到非归属会话；
    /// 条目级归属需扩队列/IPC 契约，挂账遗留（trial/t4-repair 遗留清单）。
    pub(super) pending_network_events: Vec<(String, serde_json::Value)>,
    /// Console 事件队列（S11：renderer `ConsoleLog` → `Runtime.consoleAPICalled`，
    /// transport 逐命令排空盖章；`(level, text, args_json)`）。
    pub(super) pending_console_events: Vec<(String, String, String)>,
    /// DCL 先返语义：navigate 已在 DomContentLoaded 返回、renderer `LoadComplete`
    /// 尚未到达（全量 load 未落定）。true 时 load 生命周期族由 LoadComplete 到达
    /// 时经 pending_network_events 延迟补发；false = load 已随导航事件族同步发出。
    pub(super) load_event_pending: bool,
    /// 未捕获脚本错误队列（R-baidu2/P3：renderer `ScriptError` →
    /// `Runtime.exceptionThrown` 事件源）。
    pub(super) pending_script_errors: Vec<zero_protocol::message::ScriptErrorParams>,
    /// 子帧元数据记录（frameAttached 已宣告、未 detach 的 child frame id），
    /// 按主帧 id（=targetId）分组——单 session 多 target，记录不得跨页串扰。
    /// ZeroWeb 无子帧文档加载——frame 为纯元数据面（url 停留 about:blank）。
    pub(super) active_child_frames: std::collections::HashMap<String, Vec<String>>,
    /// 子帧 frameId 序号（`zeroweb-frame-<n>`）。
    pub(super) next_frame_seq: u64,
    /// R3282（#4）：可选 GPU 截图渲染器（`ZW_HEADLESS_GPU_SCREENSHOT=1` 启用；
    /// 默认 CPU——oracle 像素对比基线稳定）。
    pub(super) gpu_renderer: Option<zero_render_foundation::gpu::renderer::GpuRenderer>,
    /// Surface-local 字体注册表（系统基表 + 下载字体）：截图光栅化用，与
    /// compositor 同一资源模型（renderer 数字 ID ≠ 全局资源 ID）。
    pub(super) paint_fonts: zero_paint_convert::fonts::PaintFonts,
    /// R-baidu2/P7：代理 fetch 完成队列——HTTP 在 worker 线程执行，会话线程
    /// 在各等待点排空（会话不再被单次 fetch 阻塞，CDP 保持响应）。
    #[cfg(not(test))]
    pub(super) fetch_completion_tx: std::sync::mpsc::Sender<CompletedFetch>,
    #[cfg(not(test))]
    pub(super) fetch_completions_rx: std::sync::mpsc::Receiver<CompletedFetch>,
    /// 代理 fetch 共享调度器（GET 走 6/origin、24 total 限流，对齐 renderer 路径
    /// `resource_policy`）：无上限 thread-per-send 实测把重资源站点打成百级并行
    /// 连接（bilibili 首页 116 条）并拖垮后续导航。
    #[cfg(not(test))]
    pub(super) fetch_scheduler: std::sync::Arc<std::sync::Mutex<zero_net::PerOriginFetchScheduler>>,
}

/// iframe 探测回执 → 子帧数（`return String(document.querySelectorAll('iframe').length)`
/// 的 ExecuteScript 结果；解析失败按 0——探测为元数据面，不得因形状偏差阻塞事件族）。
#[cfg(not(test))]
fn iframe_count_from_result(response: &zero_protocol::AutomationResponse) -> u32 {
    match &response.result {
        Ok(AutomationResult::Value(AutomationValue::String(text))) => text.parse::<u32>().unwrap_or(0),
        _ => 0,
    }
}

/// 按帧更新下载字体注册表并重写 surface-local 数字 ID（compositor 主路径同序：
/// update → remap → to_render_primitives）。校验失败保留上一帧可用 registry
///（信任边界：无效资源不得替换当前可用集合）。
pub(super) fn apply_frame_fonts(
    paint_fonts: &mut zero_paint_convert::fonts::PaintFonts,
    font_payloads: &[zero_protocol::IpcFontPayload],
    glyphs: &mut [zero_protocol::IpcGlyph],
) {
    if let Err(error) = paint_fonts.update(font_payloads) {
        tracing::warn!(%error, "headless: rejected page font resources");
    }
    paint_fonts.remap_glyphs(glyphs);
}

impl HeadlessSession {
    /// S12：非阻塞取一条 renderer 消息（CDP 空闲期 transport 轮询 drain 用；
    /// 测试进程内无 renderer → 恒 None）。
    #[cfg(not(test))]
    pub(super) fn try_recv_renderer(&mut self) -> Option<IpcMessage> {
        self.renderer.try_recv().ok().flatten()
    }

    #[cfg(test)]
    pub(super) fn try_recv_renderer(&mut self) -> Option<zero_protocol::message::IpcMessage> {
        None
    }

    /// S12：测试进程内无 renderer 消息面（空实现保持 drain 调用面编译完整）。
    #[cfg(test)]
    pub(super) fn handle_renderer_message(
        &mut self,
        _message: zero_protocol::message::IpcMessage,
    ) -> Result<Option<Result<(), String>>, String> {
        Ok(None)
    }

    /// S78 诊断：renderer stderr 临终输出 tail（进程死亡前的 panic/退出原因）。
    ///
    /// tail 此前只进内部环形缓冲、headless 全程无人消费——renderer 死亡时命令面
    /// 只见「Channel error: Broken pipe」，死因不可见（S78 排障实测缺口）。
    /// 测试构建为进程内 WebView、无 renderer，恒返回空。
    #[cfg(test)]
    pub(super) fn renderer_stderr_tail(&self) -> String {
        String::new()
    }

    #[cfg(not(test))]
    pub(super) fn renderer_stderr_tail(&self) -> String {
        self.renderer.stderr_tail()
    }

    #[cfg(test)]
    pub(super) fn new(viewport_width: f32, viewport_height: f32) -> Self {
        // BrowserShell::new 已带一个空白活跃标签；再 new_tab 会多出一个永不导航的
        // 幽灵 target（/json/list、Target.getTargets 与 PW pages 各多一个
        // about:blank，e1-zero B 阶段「新页 url()=about:blank」的一半根因）。
        let shell = BrowserShell::new();
        let config = WebViewConfig {
            width: viewport_width as u32,
            height: viewport_height as u32,
            ..Default::default()
        };
        let webview = WebView::new(config);
        Self {
            shell,
            webview,
            injected_scripts: Vec::new(),
            cookie_store: zero_net::shared_cookie_store(),
            user_agent_override: None,
            network_enabled: false,
            pending_network_events: Vec::new(),
            pending_console_events: Vec::new(),
            load_event_pending: false,
            pending_script_errors: Vec::new(),
            active_child_frames: std::collections::HashMap::new(),
            next_frame_seq: 1,
            navigation_epoch: 0,
            pending_navigation: None,
            gpu_renderer: None,
            paint_fonts: new_session_paint_fonts(),
        }
    }

    #[cfg(not(test))]
    pub(super) fn new(viewport_width: f32, viewport_height: f32) -> Self {
        let renderer_path = crate::process_backend::resolve_renderer_binary().unwrap_or_else(|| {
            std::path::PathBuf::from(if cfg!(windows) {
                "zero-renderer.exe"
            } else {
                "zero-renderer"
            })
        });
        let mut renderer = RendererHandle::spawn(renderer_path.to_string_lossy().as_ref())
            .unwrap_or_else(|error| panic!("failed to start zero-renderer for headless mode: {error}"));
        renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::SetFramePublishMode(FramePublishMode::Compositor),
            })
            .unwrap_or_else(|error| panic!("failed to configure headless renderer: {error}"));
        renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::SetViewport(SetViewportParams {
                    width: viewport_width as u32,
                    height: viewport_height as u32,
                    device_scale_factor: 1.0,
                }),
            })
            .unwrap_or_else(|error| panic!("failed to set headless viewport: {error}"));
        // 同 cfg(test) 路径：BrowserShell::new 已带一个空白活跃标签，不再 new_tab
        //（幽灵 target 去重，见上）。
        let shell = BrowserShell::new();
        let (fetch_completion_tx, fetch_completions_rx) = std::sync::mpsc::channel();
        Self {
            shell,
            renderer,
            http: HttpClient::new(),
            fetch_scheduler: zero_net::PerOriginFetchScheduler::new_shared(),
            snapshot: crate::tab_snapshot::TabSnapshot::default(),
            navigation_epoch: 0,
            pending_navigation: None,
            next_request_id: 1,
            injected_scripts: Vec::new(),
            cookie_store: zero_net::shared_cookie_store(),
            user_agent_override: None,
            network_enabled: false,
            pending_network_events: Vec::new(),
            pending_console_events: Vec::new(),
            load_event_pending: false,
            pending_script_errors: Vec::new(),
            active_child_frames: std::collections::HashMap::new(),
            next_frame_seq: 1,
            gpu_renderer: None,
            paint_fonts: new_session_paint_fonts(),
            fetch_completion_tx,
            fetch_completions_rx,
        }
    }
}

/// headless 会话对 `IndexedDbRequest` 的统一错误应答串。wire 契约：
/// `"<DOMException name>: <message>"`——shim `_zwIDBHostCall` 按首个 `:` 解析
/// DOMException name 并抛出，factory.open 转为 request 的 error 事件。
pub(super) fn headless_indexed_db_unavailable_error() -> String {
    "UnknownError: IndexedDB is unavailable in headless mode".to_string()
}

/// headless 对 `IndexedDbRequest` 的错误应答消息：id 原样回带（renderer router
/// 按 id 匹配 pending 等待者），`response: None + error: Some` 组合经
/// `response_result` 统一映射为宿主错误。纯函数以便单测 pin 应答形状。
pub(super) fn headless_indexed_db_response_for(request_id: u64) -> IpcMessage {
    IpcMessage {
        id: request_id,
        kind: IpcMessageKind::IndexedDbResponse(IndexedDbResponseParams {
            response: None,
            error: Some(headless_indexed_db_unavailable_error()),
        }),
    }
}

/// 进程级共享系统字体表作基表（与 BrowserApp/renderer 枚举同源 → 数字 ID 对齐）；
/// 首次解析 ~0.5s 后进程内缓存，会话构建均摊免费。
fn new_session_paint_fonts() -> zero_paint_convert::fonts::PaintFonts {
    let (system_fonts, _) = crate::app::shared_system_fonts();
    zero_paint_convert::fonts::PaintFonts::new(std::sync::Arc::new(system_fonts))
}

/// P100：拆批像素按 key 注入 headless 缓存（幂等）；epoch 过期批随帧失效
/// （同 GUI 路径 apply_inbound_message 规则）。独立纯函数以便 cfg(test) 覆盖
/// （`handle_renderer_message` 所在 impl 为 cfg(not(test))：会话构造依赖
/// renderer 子进程）。
fn inject_image_batch(
    snapshot: &mut crate::tab_snapshot::TabSnapshot,
    navigation_epoch: u64,
    payloads: Vec<IpcImagePayload>,
) {
    if navigation_epoch != snapshot.navigation_epoch {
        return;
    }
    for payload in payloads {
        if let Ok(data) =
            zero_render_foundation::image_cache::ImageData::from_rgba(payload.rgba, payload.width, payload.height)
        {
            snapshot.image_cache.insert_with_key(
                zero_render_foundation::image_cache::ImageKey::new(payload.image_key),
                data,
            );
        }
    }
}

#[cfg(not(test))]
impl HeadlessSession {
    pub(super) fn handle_renderer_message(
        &mut self,
        message: IpcMessage,
    ) -> Result<Option<Result<(), String>>, String> {
        match message.kind {
            IpcMessageKind::FetchRequest(params) => {
                self.proxy_fetch(params)?;
                Ok(None)
            }
            IpcMessageKind::CompositorFrame { paint, .. } | IpcMessageKind::ViewPainted(paint) => {
                let mut paint = *paint;
                apply_frame_fonts(&mut self.paint_fonts, &paint.font_payloads, &mut paint.glyphs);
                crate::paint_ipc::apply_paint_snapshot(&mut self.snapshot, paint);
                Ok(None)
            }
            IpcMessageKind::ImagePayloadBatch {
                // headless 无 compositor 进程（snapshot 即渲染源），批携带的
                // surface_id 在此无用，按 key 注入本会话缓存即可。
                surface_id: _,
                navigation_epoch,
                payloads,
            } => {
                // P100：帧内图片超批上限时，像素经 regular FIFO 批消息先行到达，
                // 帧本体只带 key 引用；此处按 key 注入缓存（幂等）。epoch 过期批
                // 随帧失效（同 GUI 路径 apply_inbound_message 规则）。
                inject_image_batch(&mut self.snapshot, navigation_epoch, payloads);
                Ok(None)
            }
            // DCL 先返语义：navigate 在 DomContentLoaded 即返回（文档可交互，图片/字体
            // 可续加载）；全量 load 生命周期族延迟到 LoadComplete 到达时经
            // pending_network_events 补发（同 S16 document.write 通道）。
            // https://html.spec.whatwg.org/multipage/#the-end
            IpcMessageKind::DomContentLoaded => {
                self.load_event_pending = true;
                Ok(Some(Ok(())))
            }
            IpcMessageKind::LoadComplete => {
                if self.load_event_pending {
                    // 导航已返回（DCL），LoadComplete 只补发延迟的 load 族，不再终结
                    // 任何等待者（此时无导航在途）。
                    self.load_event_pending = false;
                    let frame_id = self.active_frame_id().unwrap_or_default();
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    self.pending_network_events.push((
                        "Page.lifecycleEvent".into(),
                        serde_json::json!({ "frameId": frame_id, "name": "load", "timestamp": ts }),
                    ));
                    self.pending_network_events
                        .push(("Page.loadEventFired".into(), serde_json::json!({ "timestamp": ts })));
                    self.pending_network_events.push((
                        "Page.frameStoppedLoading".into(),
                        serde_json::json!({ "frameId": frame_id }),
                    ));
                    Ok(None)
                } else {
                    Ok(Some(Ok(())))
                }
            }
            IpcMessageKind::LoadFailed(message) | IpcMessageKind::CrashNotification(message) => Ok(Some(Err(message))),
            // renderer 侧导航提交（target=_blank 点击降级同 traversable、location.href 赋值、
            // 表单导航——FIXME(tab-ipc)：新建 tab 面落地前均为同 tab 提交）→ 回写 shell URL
            // 并补发导航事件族（见 apply_navigation_commit）。CDP target 元数据
            //（/json/list、Target.getTargets、targetInfo）与客户端 page.url() 全部取自
            // shell Tab/事件流，不回写则新文档加载后 target url 停留旧值（e1-zero B/C
            // 阶段实测：页内 location.href 已是 news.qq.com，target url() 仍 about:blank）。
            // spec：target 的 url 为当前文档 URL，Chrome 在 commit 时更新 targetInfo。
            // https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate
            // 世代校验/回声去重/重定向替换见 apply_renderer_navigation_commit。
            IpcMessageKind::NavigationCommitted(params) => {
                self.apply_renderer_navigation_commit(&params);
                Ok(None)
            }
            // 导航开始登记（commit 校验、回声判定与 wait_for_load 武装门控的依据）：
            // 会话命令导航确认既有登记，renderer 侧导航采纳其自增世代，其余丢弃。
            // 见 note_navigation_started。
            IpcMessageKind::NavigationStarted(params) => {
                self.note_navigation_started(&params.url, params.navigation_epoch);
                Ok(None)
            }
            // S11：page console 输出 → 会话事件队列（transport 逐命令排空盖章为
            // `Runtime.consoleAPICalled`； PW 消费面 = msg.type()/text()）。
            IpcMessageKind::ScriptError(params) => {
                self.pending_script_errors.push(params);
                Ok(None)
            }
            IpcMessageKind::ConsoleLog(params) => {
                self.pending_console_events
                    .push((params.level, params.text, params.args_json));
                Ok(None)
            }
            // S16：document.write 写周期落定 → load 生命周期重发（`Page.lifecycleEvent`
            // + `Page.loadEventFired`；PW page.setContent 在 console tag 清 lifecycle 后
            // 等待新 load——spec document.close() 解析结束触发 load 的软导航语义）。
            // 不发 frameNavigated/contextsCleared：文档对象与 JS context 未换代。
            IpcMessageKind::DocumentWriteSettled(_) => {
                // 写周期落定已重发完整 DCL+load 族——同一文档的 LoadComplete 到达时
                // 不再补发延迟 load 族。
                self.load_event_pending = false;
                let frame_id = self.active_frame_id().unwrap_or_default();
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                self.pending_network_events.push((
                    "Page.lifecycleEvent".into(),
                    serde_json::json!({ "frameId": frame_id, "name": "DOMContentLoaded", "timestamp": ts }),
                ));
                self.pending_network_events.push((
                    "Page.domContentEventFired".into(),
                    serde_json::json!({ "timestamp": ts }),
                ));
                self.pending_network_events.push((
                    "Page.lifecycleEvent".into(),
                    serde_json::json!({ "frameId": frame_id, "name": "load", "timestamp": ts }),
                ));
                self.pending_network_events
                    .push(("Page.loadEventFired".into(), serde_json::json!({ "timestamp": ts })));
                Ok(None)
            }
            // S14：page fetch 观测 → Network 事件队列（Network.enable 门控）。
            // phase 0=requestWillBeSent / 1=responseReceived / 2=loadingFinished|loadingFailed；
            // seq 为三阶段关联 id（headless 作 requestId）。
            IpcMessageKind::FetchObserved(params) => {
                if self.network_enabled {
                    let frame_id = self.active_frame_id().unwrap_or_default();
                    let request_id = format!("zw-net-{}", params.seq);
                    let document_url = self
                        .shell
                        .active_tab()
                        .and_then(|tab| tab.url().map(str::to_string))
                        .unwrap_or_default();
                    // CDP timestamp 契约 = 秒（浮点，单调基任意）；毫秒值会被
                    // frontend 瀑布图/时序计算读歪（M2-N4 事件形状对齐）。
                    let now = || {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs_f64()
                    };
                    match params.phase {
                        0 => self.pending_network_events.push((
                            "Network.requestWillBeSent".into(),
                            serde_json::json!({
                                "requestId": request_id,
                                "frameId": frame_id,
                                "documentURL": document_url,
                                "request": {
                                    "url": params.url,
                                    "method": params.method,
                                    "headers": { "Accept": "*/*" },
                                    "mixedContentType": "none",
                                    "initialPriority": "High",
                                    "referrerPolicy": "strict-origin-when-cross-origin",
                                },
                                "timestamp": now(),
                                "wallTime": now(),
                                "initiator": { "type": "other" },
                                "type": "Fetch",
                            }),
                        )),
                        1 => self.pending_network_events.push((
                            "Network.responseReceived".into(),
                            serde_json::json!({
                                "requestId": request_id,
                                "frameId": frame_id,
                                "type": "Fetch",
                                "response": {
                                    "url": params.url,
                                    "status": params.status,
                                    "statusText": if params.status == 200 { "OK" } else { "" },
                                    "headers": {},
                                    "mimeType": "application/json",
                                    "charset": "",
                                    "connectionReused": false,
                                    "connectionId": 0,
                                    "remoteIPAddress": "",
                                    "remotePort": 0,
                                    "fromDiskCache": false,
                                    "fromServiceWorker": false,
                                    "fromPrefetchCache": false,
                                    "encodedDataLength": 0,
                                    "protocol": "http/1.1",
                                },
                                "timestamp": now(),
                            }),
                        )),
                        // S17：loadingFinished 前发 dataReceived（body 一次性到达语义——
                        // dataLength=encodedDataLength=观测 body 字节数；分块流式随 net
                        // 观测点流式化。失败路径 data_length=0 → 只发 dataLength 0 事件，
                        // PW/devtools 按 finished 空载消费）。
                        _ => {
                            self.pending_network_events.push((
                                "Network.dataReceived".into(),
                                serde_json::json!({
                                    "requestId": request_id,
                                    "timestamp": now(),
                                    "dataLength": params.data_length,
                                    "encodedDataLength": params.data_length,
                                }),
                            ));
                            self.pending_network_events.push((
                                "Network.loadingFinished".into(),
                                serde_json::json!({
                                    "requestId": request_id,
                                    "timestamp": now(),
                                    "encodedDataLength": params.data_length,
                                }),
                            ));
                        }
                    }
                }
                Ok(None)
            }
            // S13：page fetch 观测 → Network 事件队列（Network.enable 门控）。
            // phase 0=requestWillBeSent / 1=responseReceived / 2=loadingFinished|loadingFailed。
            // S13：headless 无 Service Worker 支持，但必须**应答** renderer 的 SW 请求——
            // shim 的 fetch settle 路径（`__zwServiceWorkerFetchSettled` → ensureDocument →
            // `__zw_sw_controller`）会同步阻塞等响应（SW IPC client 20s 超时），不应答则
            // JS worker 挂死、PW click 10s 超时（network.events 实测根因）。
            // 语义：无注册/无 controller（headless 正确状态）。
            IpcMessageKind::ServiceWorkerRequest(params) => {
                use zero_protocol::message::{
                    ServiceWorkerError, ServiceWorkerErrorCode, ServiceWorkerOperation, ServiceWorkerResponseParams,
                    ServiceWorkerResult,
                };
                let result = match &params.operation {
                    ServiceWorkerOperation::Controller => Ok(ServiceWorkerResult::OptionalSnapshot(None)),
                    ServiceWorkerOperation::GetRegistrations => Ok(ServiceWorkerResult::Snapshots(Vec::new())),
                    ServiceWorkerOperation::StateChanges { .. } => Ok(ServiceWorkerResult::StateChanges(
                        zero_protocol::message::ServiceWorkerStateChanges {
                            latest_sequence: 0,
                            states: Vec::new(),
                            claim_clients: false,
                        },
                    )),
                    // 写类操作在无 SW 支持的 headless 中一律 NotFound（spec：无注册）。
                    _ => Err(ServiceWorkerError {
                        code: ServiceWorkerErrorCode::NotFound,
                        message: "service workers are not supported in headless mode".into(),
                    }),
                };
                self.renderer
                    .send(IpcMessage {
                        id: message.id,
                        kind: IpcMessageKind::ServiceWorkerResponse(ServiceWorkerResponseParams { result }),
                    })
                    .map_err(|error| error.to_string())?;
                Ok(None)
            }
            // IndexedDbRequest 必须应答：renderer 的 `__zw_idb` 宿主桥在
            // `IndexedDbRequest` 发出后同步阻塞等待 `IndexedDbResponse`（recv_timeout
            // 20s）。若落入 catch-all 静默丢弃，renderer worker 线程挂起至 execute
            // 看门狗（TAB_JS_EXEC_TIMEOUT_MS）终止整个脚本，微任务队列随之丢失——
            // 页面所有 pending 回调（IDB onsuccess/onerror 及排队其后的任务）永不触发。
            // headless 无存储后端，按 Indexed DB 规范的错误路径快速失败：错误串遵循
            // `"<DOMException name>: <message>"` wire 契约（shim `_zwIDBHostCall` 按首个
            // `:` 解析 name 抛 DOMException，factory.open 转为 request 的 error 事件，
            // spec: https://www.w3.org/TR/IndexedDB/#open-a-database-request）。
            IpcMessageKind::IndexedDbRequest(_) => {
                let response = headless_indexed_db_response_for(message.id);
                self.renderer.send(response).map_err(|error| error.to_string())?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn proxy_fetch(&mut self, params: FetchParams) -> Result<(), String> {
        let method = match params.method.to_ascii_uppercase().as_str() {
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            _ => HttpMethod::Get,
        };
        let parsed_url = zero_net::parse_url(&params.url).ok();
        // Cookie 注入（Storage jar → 请求头）+ UA override
        let mut headers = params.headers;
        if let Some(parsed) = &parsed_url {
            let cookie_header = self
                .cookie_store
                .lock()
                .expect("headless cookie store lock")
                .cookie_header(parsed);
            if !cookie_header.is_empty() {
                headers.retain(|(k, _)| !k.eq_ignore_ascii_case("cookie"));
                headers.push(("Cookie".to_string(), cookie_header));
            }
        }
        if let Some(ua) = &self.user_agent_override {
            headers.retain(|(k, _)| !k.eq_ignore_ascii_case("user-agent"));
            headers.push(("User-Agent".to_string(), ua.clone()));
        }

        // Network 事件源（Network.enable 门控）：requestWillBeSent
        let net_request_id = format!("zw-net-{}", self.next_request_id);
        let frame_id = self.active_frame_id();
        if self.network_enabled {
            let request_headers: serde_json::Map<String, Value> = headers
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            // 事件形状对齐 Chrome 实捕获（M2-N4）：documentURL/wallTime/type 必带
            let document_url = self
                .shell
                .active_tab()
                .and_then(|tab| tab.url().map(str::to_string))
                .unwrap_or_default();
            let wall_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64();
            self.pending_network_events.push((
                "Network.requestWillBeSent".to_string(),
                serde_json::json!({
                    "requestId": net_request_id,
                    "frameId": frame_id,
                    "documentURL": document_url,
                    "request": {
                        "url": params.url,
                        "method": params.method,
                        "headers": request_headers,
                        "mixedContentType": "none",
                        "initialPriority": "High",
                        "referrerPolicy": "strict-origin-when-cross-origin",
                    },
                    "timestamp": wall_time,
                    "wallTime": wall_time,
                    "initiator": { "type": "other" },
                    "type": "Document",
                }),
            ));
        }

        // R-baidu2/P7：HTTP 移交 worker 线程执行——会话线程若在此阻塞（最长
        // http 超时 30s × 重定向链），CDP 命令与 renderer 消息全部饿死（SERP
        // 图片加载实测冻结自动化分钟级）。结果经完成队列由会话线程排空。
        let request_id = params.request_id;
        let net_request_id_clone = net_request_id.clone();
        let network_enabled = self.network_enabled;
        let job_url = params.url.clone();
        let job_method = method.clone();
        let job_headers = headers.clone();
        let job_body = params.body.clone();
        let outcome_tx = self.fetch_completion_tx.clone();
        let http = self.http.clone();
        let scheduler = self.fetch_scheduler.clone();
        // GET 且无 body 走 per-origin 调度（6/origin、24 total，对齐 renderer 路径
        // resource_policy）：无上限并发发送实测对重资源站点形成百级并行连接并拖垮
        // 后续导航；非 GET / 带 body（XHR 写操作）保留直发。
        let via_scheduler = matches!(job_method, HttpMethod::Get) && job_body.is_none();
        let spawn = std::thread::Builder::new()
            .name("headless-fetch".into())
            .spawn(move || {
                let outcome = if via_scheduler {
                    let (priority, _) = zero_net::FetchPriority::from_fetch_headers(&job_headers, &job_url);
                    let (rx, _telemetry, _owns) =
                        zero_net::PerOriginFetchScheduler::submit_shared_with_key_headers_and_telemetry(
                            &scheduler,
                            job_url.clone(),
                            job_url.clone(),
                            priority,
                            job_headers,
                            30,
                        );
                    rx.recv()
                        .map_err(|error| zero_net::NetError::Network(format!("fetch scheduler channel: {error}")))
                        .and_then(|result| result.map_err(zero_net::NetError::Network))
                } else {
                    http.send(HttpRequest {
                        method: job_method,
                        url: job_url.clone(),
                        headers: job_headers,
                        body: job_body,
                    })
                };
                let _ = outcome_tx.send(CompletedFetch {
                    request_id,
                    net_request_id: net_request_id_clone,
                    network_enabled,
                    origin_url: job_url,
                    outcome,
                });
            });
        if let Err(error) = spawn {
            // 线程创建失败（极端场景）：同步兜底，正确性优先于响应性。
            tracing::warn!("headless fetch worker spawn failed, running inline: {error}");
            let outcome = self.http.send(HttpRequest {
                method,
                url: params.url.clone(),
                headers,
                body: params.body,
            });
            self.apply_fetch_completion(CompletedFetch {
                request_id,
                net_request_id,
                network_enabled,
                origin_url: params.url,
                outcome,
            })?;
        }
        Ok(())
    }

    /// 排空已完成的代理 fetch：cookie 应用 → Network 事件 → 响应回发。
    /// 必须在所有等待 renderer 消息的循环中周期调用，否则 renderer 侧
    /// `ipc_fetch` 永久阻塞（learning #24 同族）。
    pub(super) fn drain_fetch_completions(&mut self) {
        let mut completions: Vec<CompletedFetch> = Vec::new();
        while let Ok(completion) = self.fetch_completions_rx.try_recv() {
            completions.push(completion);
        }
        for completion in completions {
            if let Err(e) = self.apply_fetch_completion(completion) {
                tracing::warn!("fetch completion apply failed: {e}");
            }
        }
    }

    fn apply_fetch_completion(&mut self, completion: CompletedFetch) -> Result<(), String> {
        let CompletedFetch {
            request_id,
            net_request_id,
            network_enabled,
            origin_url,
            outcome,
        } = completion;
        let parsed_url = zero_net::parse_url(&origin_url).ok();
        let frame_id = self.active_frame_id();
        match outcome {
            Ok(response) => {
                // Set-Cookie 捕获（响应 → Storage jar）
                if let Some(parsed) = &parsed_url {
                    let set_cookies: Vec<String> = response
                        .headers
                        .iter()
                        .filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie"))
                        .map(|(_, v)| v.clone())
                        .collect();
                    for header_value in set_cookies {
                        if let Ok(cookie) = CookieStore::parse_set_cookie(&header_value) {
                            self.cookie_store
                                .lock()
                                .expect("headless cookie store lock")
                                .add_from_url(cookie, parsed);
                        }
                    }
                }
                // Network 事件：responseReceived + loadingFinished
                if network_enabled {
                    let response_headers: serde_json::Map<String, Value> = response
                        .headers
                        .iter()
                        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                        .collect();
                    let mime_type = response
                        .headers
                        .iter()
                        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default();
                    self.pending_network_events.push((
                        "Network.responseReceived".to_string(),
                        serde_json::json!({
                            "requestId": net_request_id,
                            "frameId": frame_id,
                            "response": {
                                "url": response.url,
                                "status": response.status_code,
                                "statusText": "",
                                "headers": response_headers,
                                "mimeType": mime_type,
                            },
                        }),
                    ));
                    // S17：dataReceived（body 一次性到达语义——proxy_fetch 同步取回完整
                    // body，dataLength=encodedDataLength=body 字节数；分块流式随 net
                    // 观测点流式化）。
                    self.pending_network_events.push((
                        "Network.dataReceived".to_string(),
                        serde_json::json!({
                            "requestId": net_request_id,
                            "timestamp": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs_f64(),
                            "dataLength": response.body.len(),
                            "encodedDataLength": response.body.len(),
                        }),
                    ));
                    self.pending_network_events.push((
                        "Network.loadingFinished".to_string(),
                        serde_json::json!({ "requestId": net_request_id }),
                    ));
                }
                self.renderer
                    .send_fetch_response(request_id, response.status_code, response.headers, response.body)
            }
            Err(error) => {
                if network_enabled {
                    self.pending_network_events.push((
                        "Network.loadingFailed".to_string(),
                        serde_json::json!({
                            "requestId": net_request_id,
                            "errorText": error.to_string(),
                            "canceled": false,
                        }),
                    ));
                }
                self.renderer
                    .send_fetch_response(request_id, 0, Vec::new(), error.to_string().into_bytes())
            }
        }
        .map_err(|error| error.to_string())
    }

    fn wait_for_load(&mut self) -> Result<(), String> {
        // 武装门控：renderer 发出 NavigationStarted（携带本次导航 epoch）之前，接收
        // 缓冲中的终结信号（DCL/LoadComplete/LoadFailed）一律属上一文档——吞掉，不得
        // 冒充本次导航的完成依据。不排空缓冲：旧文档持续产出消息（图片 fetch 等）
        // 时排空永不收敛，会饿死本次导航命令的发送。
        let epoch = self.navigation_epoch;
        let mut armed = false;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if std::time::Instant::now() >= deadline {
                return Err("navigation timed out".into());
            }
            self.drain_fetch_completions();
            match self.renderer.try_recv().map_err(|error| error.to_string())? {
                Some(message) => {
                    if !armed {
                        match message.kind {
                            IpcMessageKind::NavigationStarted(ref params) if params.navigation_epoch == epoch => {
                                // 世代数值可能碰撞（renderer 侧导航自增 vs 会话命令自增同值）：
                                // URL 须对齐在途登记才武装——否则被取代导航的 started 误武装后，
                                // 旧文档终结信号会冒充本次导航的完成依据（t4 返修 D1 同族）。
                                // 登记缺失时保守武装（原语义）。
                                if self
                                    .pending_navigation
                                    .as_ref()
                                    .is_none_or(|pending| pending.url == params.url && pending.epoch == epoch)
                                {
                                    armed = true;
                                }
                            }
                            // 陈旧终结信号：丢弃（其唯一起作用的就是终结等待者）。
                            // NavigationCommitted 同列（t4 返修 D1：迟到的上一文档 commit
                            // 不得冒充本次导航——模型面兜底在 apply_renderer_navigation_commit
                            // 的世代校验，此处仅免其无效分派）。
                            IpcMessageKind::DomContentLoaded
                            | IpcMessageKind::LoadComplete
                            | IpcMessageKind::LoadFailed(_)
                            | IpcMessageKind::CrashNotification(_)
                            | IpcMessageKind::NavigationCommitted(_) => continue,
                            _ => {}
                        }
                    }
                    if let Some(result) = self.handle_renderer_message(message)? {
                        return result;
                    }
                }
                None if !self.renderer.is_alive() => return Err("renderer exited during navigation".into()),
                None => std::thread::sleep(std::time::Duration::from_millis(2)),
            }
        }
    }

    pub(super) fn navigate_renderer(&mut self, url: &str) -> Result<(), String> {
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1).max(1);
        // 在途导航登记（commit 校验/回声判定依据）：会话命令导航，started_confirmed
        // 待 renderer NavigationStarted 确认。
        self.pending_navigation = Some(PendingNavigation {
            url: url.to_string(),
            epoch: self.navigation_epoch,
            client_initiated: true,
            started_confirmed: false,
        });
        // 上一文档未交付的延迟 load 族作废（文档换代，LoadComplete 不再属于新导航）。
        self.load_event_pending = false;
        // 导航终止旧文档的 fetch group（https://fetch.spec.whatwg.org/#fetch-groups）：
        // 换新调度器实例——旧页挂起子资源不得占用并发槽饿死新文档 fetch；旧实例由
        // 在途任务持有排空后消亡，队列内未启动任务随 reply 通道关闭而终止。
        self.fetch_scheduler = zero_net::PerOriginFetchScheduler::new_shared();
        self.snapshot.begin_navigation(url.to_string());
        self.renderer
            .navigate(url, None, self.navigation_epoch)
            .map_err(|error| error.to_string())?;
        self.wait_for_load()
    }

    pub(super) fn load_html_renderer(&mut self, html: &str, css: Option<&str>) -> Result<(), String> {
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1).max(1);
        // 同 navigate_renderer：会话命令导航的在途登记。
        self.pending_navigation = Some(PendingNavigation {
            url: "about:blank".to_string(),
            epoch: self.navigation_epoch,
            client_initiated: true,
            started_confirmed: false,
        });
        self.load_event_pending = false;
        self.snapshot.begin_navigation("about:blank".to_string());
        self.renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::LoadHtml(LoadHtmlParams {
                    html: html.to_string(),
                    css: css.map(str::to_string),
                    url: Some("about:blank".to_string()),
                    navigation_epoch: self.navigation_epoch,
                }),
            })
            .map_err(|error| error.to_string())?;
        self.wait_for_load()
    }

    pub(super) fn execute_script_renderer(&mut self, script: &str) -> Result<String, String> {
        match self.execute_script_typed_renderer(script)? {
            AutomationValue::String(value) => Ok(value),
            value => serde_json::to_string(&value).map_err(|error| error.to_string()),
        }
    }

    /// 执行脚本并返回类型化结果（CDP Runtime 域 remoteObject 需要值类型；
    /// renderer 的 ExecuteScript 以 JSON envelope 返回类型化 AutomationValue）。
    pub(super) fn execute_script_typed_renderer(&mut self, script: &str) -> Result<AutomationValue, String> {
        let result = self.automation_request(AutomationOperation::ExecuteScript {
            script: script.to_string(),
            arguments: Vec::new(),
        })?;
        Ok(match result {
            AutomationResult::Value(value) => value,
            _ => AutomationValue::Null,
        })
    }

    /// 自动化操作统一入口：发 `AutomationRequest`、等响应、提取结果
    ///（Runtime objectId 桥四操作与既有 ExecuteScript 共用同一 IPC 往返）。
    pub(super) fn automation_request(&mut self, operation: AutomationOperation) -> Result<AutomationResult, String> {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
        self.renderer
            .send(IpcMessage {
                id: request_id,
                kind: IpcMessageKind::AutomationRequest(AutomationRequest { operation }),
            })
            .map_err(|error| error.to_string())?;
        // t2-pb1 fix#15 协同：renderer 侧 evaluate 可挂起续答（worker 长臂时 2s 放行主循环），
        // 迟到的回复最迟在 worker 单臂上限之后到达。这里等待窗须覆盖同一上限（js_worker
        // TAB_JS_EXEC_TIMEOUT_MS=30s + 余量），否则挂起补答被 headless 先行超时报错丢弃
        // （bilibili 30s timer 臂期间 ev19 实测 evaluate 间歇性 "automation request timeout"）。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(35);
        let response = loop {
            if std::time::Instant::now() >= deadline {
                return Err("automation request timeout".into());
            }
            self.drain_fetch_completions();
            match self.renderer.try_recv().map_err(|error| error.to_string())? {
                Some(IpcMessage {
                    id,
                    kind: IpcMessageKind::AutomationResponse(response),
                }) if id == request_id => break response,
                Some(message) => {
                    self.handle_renderer_message(message)?;
                }
                None if !self.renderer.is_alive() => return Err("renderer exited during script execution".into()),
                None => std::thread::sleep(std::time::Duration::from_millis(2)),
            }
        };
        response.result.map_err(|error| error.message)
    }

    /// Page.navigate 事件族的 iframe 探测（slice21 eval-wedge 根因修复）。
    ///
    /// 探测求值不得阻塞导航命令：renderer 单 JS worker 正忙于页面脚本臂时，该求值
    /// 排在其后，命令处理会阻塞整个单线程复用循环（其他连接的命令/事件全部停摆，
    /// 导航后跨连接 evaluate 全数饿死）。策略：renderer 空闲时毫秒级往返，行为与
    /// 旧同步探测一致；`bound` 内未应答则放弃本轮探测（按 0 个子帧处理）——迟到的
    /// 求值与导航管线自身的 worker 快照安装存在竞态（执行时点文档可能仍是旧文档），
    /// 迟发 frameAttached 不可靠，宁缺勿错；脚本臂阻塞页由此失去 frame 元数据
    ///（修复前该形态直接楔死整个复用循环，严格更优）。
    ///
    /// 返回本轮可见子帧数（探测失败按 0，与旧路径 `.ok().unwrap_or(0)` 同语义）。
    #[cfg(not(test))]
    pub(super) fn probe_iframes_deferred(&mut self, script: &str, bound: std::time::Duration) -> u32 {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
        let sent = self.renderer.send(IpcMessage {
            id: request_id,
            kind: IpcMessageKind::AutomationRequest(AutomationRequest {
                operation: AutomationOperation::ExecuteScript {
                    script: script.to_string(),
                    arguments: Vec::new(),
                },
            }),
        });
        if sent.is_err() {
            return 0;
        }
        let deadline = std::time::Instant::now() + bound;
        loop {
            if std::time::Instant::now() >= deadline {
                tracing::debug!(
                    timeout_ms = bound.as_millis() as u64,
                    "导航事件族 iframe 探测超界放弃（renderer 忙）"
                );
                return 0;
            }
            self.drain_fetch_completions();
            match self.renderer.try_recv() {
                Ok(Some(IpcMessage {
                    id,
                    kind: IpcMessageKind::AutomationResponse(response),
                })) if id == request_id => return iframe_count_from_result(&response),
                Ok(Some(message)) => {
                    // 非本探测的消息照常分派（load 族/console/网络观测不丢）。
                    if self.handle_renderer_message(message).is_err() {
                        return 0;
                    }
                }
                Ok(None) if !self.renderer.is_alive() => return 0,
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(2)),
                Err(_) => return 0,
            }
        }
    }

    /// CDP Input 域 → renderer IPC 发送辅助（见各 send_input_*）。
    pub(super) fn send_input_mouse(
        &mut self,
        x: f32,
        y: f32,
        button: u8,
        event_type: MouseEventType,
    ) -> Result<(), String> {
        self.renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::MouseEvent(MouseEventParams {
                    x,
                    y,
                    button,
                    event_type,
                }),
            })
            .map_err(|error| error.to_string())
    }

    pub(super) fn send_input_scroll(
        &mut self,
        delta_x: f32,
        delta_y: f32,
        cursor_x: f32,
        cursor_y: f32,
    ) -> Result<(), String> {
        self.renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
                    delta_x,
                    delta_y,
                    cursor_x,
                    cursor_y,
                }),
            })
            .map_err(|error| error.to_string())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn send_input_key(
        &mut self,
        key: String,
        code: String,
        ctrl: bool,
        shift: bool,
        alt: bool,
        meta: bool,
        text: Option<String>,
        event_type: KeyboardEventType,
    ) -> Result<(), String> {
        self.renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                    key,
                    code,
                    text,
                    ctrl,
                    shift,
                    alt,
                    meta,
                    event_type,
                }),
            })
            .map_err(|error| error.to_string())
    }

    pub(super) fn send_input_ime_commit(&mut self, text: String) -> Result<(), String> {
        self.renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::ImeEvent(ImeEventParams {
                    event_type: ImeEventType::Commit,
                    text,
                    cursor_start: None,
                    cursor_end: None,
                }),
            })
            .map_err(|error| error.to_string())
    }

    /// Emulation.setDeviceMetricsOverride → renderer SetViewport。
    pub(super) fn send_set_viewport(
        &mut self,
        width: f32,
        height: f32,
        device_scale_factor: f32,
    ) -> Result<(), String> {
        self.renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::SetViewport(SetViewportParams {
                    width: width as u32,
                    height: height as u32,
                    device_scale_factor,
                }),
            })
            .map_err(|error| error.to_string())
    }

    /// Emulation.setEmulatedMedia（prefers-color-scheme）→ renderer SetColorScheme。
    pub(super) fn send_set_color_scheme(&mut self, dark: bool) -> Result<(), String> {
        self.renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::SetColorScheme(SetColorSchemeParams {
                    scheme: if dark {
                        IpcColorScheme::Dark
                    } else {
                        IpcColorScheme::Light
                    },
                }),
            })
            .map_err(|error| error.to_string())
    }

    /// Emulation.setEmulatedMedia（media type）→ renderer SetMediaType。
    pub(super) fn send_set_media_type(&mut self, print: bool) -> Result<(), String> {
        self.renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::SetMediaType(SetMediaTypeParams {
                    media_type: if print {
                        IpcMediaType::Print
                    } else {
                        IpcMediaType::Screen
                    },
                }),
            })
            .map_err(|error| error.to_string())
    }

    /// CDP addScriptToEvaluateOnNewDocument → renderer PreDocumentScripts（全量列表，
    /// renderer 整体替换；此后每个新文档在页面脚本执行前执行）。
    pub(super) fn send_pre_document_scripts(&mut self) -> Result<(), String> {
        let sources: Vec<String> = self
            .injected_scripts
            .iter()
            .map(|script| script.source.clone())
            .collect();
        self.renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::PreDocumentScripts(PreDocumentScriptsParams { sources }),
            })
            .map_err(|error| error.to_string())
    }
}

impl HeadlessSession {
    /// 活跃标签页的主 frame id（proxy_fetch 等 session 侧无 CDP 会话上下文的场景）。
    pub(super) fn active_frame_id(&self) -> Option<String> {
        self.shell.active_tab().map(|tab| format!("zeroweb-tab-{}", tab.id().0))
    }

    /// renderer `NavigationStarted` → 在途导航登记。
    ///
    /// 世代语义（对齐 GUI process_backend handle_navigation_started 的 ==/==+1 门控）：
    /// 会话命令导航（Page.navigate/LoadHtml）世代由会话分配、登记已在命令时点建立
    ///（navigate_renderer/load_html_renderer），URL 对齐的 started 仅做确认——确认前
    /// 其 commit 不得放行（甄别世代碰撞）；renderer 侧导航（anchor/表单/location.href，
    /// FIXME(tab-ipc) 同 tab 提交面）世代由 renderer 自增，恰为当前世代 +1 时采纳并
    /// 重登记（client_initiated=false：其同 URL commit 不得按回声跳过，t4 返修 D2）；
    /// 其余为陈旧信号或世代数值碰撞——丢弃，不得覆盖在途登记或回退世代。
    pub(super) fn note_navigation_started(&mut self, url: &str, epoch: u64) {
        if let Some(pending) = self.pending_navigation.as_mut()
            && pending.epoch == epoch
            && pending.url == url
        {
            pending.started_confirmed = true;
            return;
        }
        if epoch == self.navigation_epoch.wrapping_add(1) {
            self.navigation_epoch = epoch;
            self.pending_navigation = Some(PendingNavigation {
                url: url.to_string(),
                epoch,
                client_initiated: false,
                started_confirmed: true,
            });
            return;
        }
        tracing::warn!(
            epoch,
            current_epoch = self.navigation_epoch,
            url,
            "headless: dropped stale navigation start"
        );
    }

    /// renderer `NavigationCommitted` → 世代校验 + 回声去重 + 重定向替换后应用
    ///（世代/回声判定通过后走 [`Self::apply_navigation_commit`]）。
    ///
    /// 1. 世代校验（t4 返修 D1，对齐 GUI process_backend handle_navigation_committed）：
    ///    commit 世代 ≠ 当前导航世代 → 陈旧提交（渲染器缓冲中上一文档的迟到 commit，
    ///    或已被新导航取代）→ 丢弃：不回写 shell、不入队事件族（防 shell URL 暂时
    ///    回退、历史污染 [A,B,A,B] 与向客户端发假 frameNavigated 族）。
    /// 2. 在途登记：世代对齐且 started 已确认 → 消费放行（登记 URL 不强校验——
    ///    重定向 commit 携带最终 URL）；世代对齐但未确认 → 同世代碰撞的被取代导航
    ///    commit → 丢弃（登记保留，后续真 commit 仍可校验，GUI 同语义）；登记缺失
    ///    （同世代重复 commit 等异常时序；epoch 恒在场无需缺省分支）→ 保守放行：
    ///    世代已对齐当前导航，回声判定退化为「shell 同 URL 即跳过」。
    /// 3. 回声（t4 返修 D2 收窄）：仅会话命令导航（或登记缺失退化）且 shell 已提交
    ///    同一 URL 时跳过；renderer 侧同 URL 真导航（锚点/表单）照常发展事件族。
    ///
    /// 返回是否对 shell 模型产生了变化（单测钉形状）。
    pub(super) fn apply_renderer_navigation_commit(&mut self, params: &NavigationCommittedParams) -> bool {
        if params.navigation_epoch != self.navigation_epoch {
            tracing::warn!(
                commit_epoch = params.navigation_epoch,
                current_epoch = self.navigation_epoch,
                url = %params.url,
                "headless: dropped stale navigation commit (epoch mismatch)"
            );
            return false;
        }
        let client_initiated = match self.pending_navigation.take() {
            Some(pending) if pending.epoch == params.navigation_epoch && pending.started_confirmed => {
                Some(pending.client_initiated)
            }
            Some(pending) => {
                tracing::warn!(
                    commit_epoch = params.navigation_epoch,
                    url = %params.url,
                    "headless: dropped navigation commit without confirmed start"
                );
                self.pending_navigation = Some(pending);
                return false;
            }
            None => None,
        };
        let same_url = self
            .shell
            .active_tab()
            .is_some_and(|tab| tab.url() == Some(params.url.as_str()));
        // 回声：commit 世代属会话命令导航（cmd_navigate 已先行 shell.navigate 写入
        // 同一 URL）——跳过防重复历史条目与多余事件族；renderer 侧发起的同 URL 真
        // 导航（锚点/表单，client_initiated=false）不属回声，照常发展事件族（D2）。
        if same_url && client_initiated.is_none_or(|client| client) {
            return false;
        }
        // 重定向替换（t4 返修 D3）：会话命令导航的 commit URL ≠ 命令时写入的 URL
        //（HTTP 重定向最终 URL）——取代初始历史条目而非追加（Chrome 客户端重定向
        // 单条目语义）；renderer 侧导航的登记 URL 从未写入 shell，常规追加。
        self.apply_navigation_commit(&params.url, client_initiated == Some(true));
        true
    }

    /// 导航提交应用：shell 模型回写 + 导航事件族入队（世代校验/回声判定见
    /// [`Self::apply_renderer_navigation_commit`]）。
    ///
    /// `replace_current_entry`：重定向提交以最终 URL 取代当前历史条目（不追加）；
    /// false = 常规 `shell.navigate` 追加。事件族对齐 `emit_navigation_event_family`
    /// 的上下文换代子集（frameStartedLoading → frameNavigated →
    /// executionContextsCleared → 新 main world context）：缺 context 重置时客户端
    ///（PW）对新文档的 evaluate 全数失败（"object handle is unknown or has been
    /// released"，t4 minrepro 实测）。
    /// 子集边界（t4 返修 D4）：不含子框架分离宣告——ZeroWeb 无子帧文档加载，frame
    /// 为纯元数据面；不含 out-of-world 重宣告——单引擎无 world 隔离，main world 恒
    /// id=1（S4 契约）。DCL/load 生命周期与 iframe 探测由 renderer 自身消息管线补发。
    /// on_page_loaded 复位 loading 并以 URL 作标题回退（headless 不消费
    /// TitleChanged，与 cmd_navigate 的标题回退一致）。
    pub(super) fn apply_navigation_commit(&mut self, url: &str, replace_current_entry: bool) {
        if replace_current_entry {
            self.shell.replace_current_entry_url(url);
        } else {
            self.shell.navigate(url);
        }
        self.shell.on_page_loaded(url);
        let frame_id = self.active_frame_id().unwrap_or_default();
        let mut push = |method: &str, params: serde_json::Value| {
            self.pending_network_events.push((method.into(), params));
        };
        push("Page.frameStartedLoading", serde_json::json!({ "frameId": frame_id }));
        push(
            "Page.frameNavigated",
            serde_json::json!({
                "frame": {
                    "id": frame_id,
                    "loaderId": "",
                    "url": url,
                    "mimeType": "text/html",
                }
            }),
        );
        // 文档换代：清旧上下文并重发 main world context（id=1，S4 契约）——
        // 客户端据新 context 事件重建 evaluate 面。
        push("Runtime.executionContextsCleared", serde_json::json!({}));
        push(
            "Runtime.executionContextCreated",
            serde_json::json!({
                "context": {
                    "id": 1,
                    "origin": "://",
                    "name": "main",
                    "auxData": {
                        "frameId": frame_id,
                        "isDefault": true,
                    }
                }
            }),
        );
    }

    /// CDP Runtime 域脚本执行入口：类型化结果（测试进程内路径为扁平字符串语义）。
    pub(super) fn execute_script_typed(&mut self, script: &str) -> Result<AutomationValue, String> {
        #[cfg(test)]
        {
            self.webview
                .execute_script(script)
                .map(AutomationValue::String)
                .map_err(|error| error.to_string())
        }
        #[cfg(not(test))]
        {
            self.execute_script_typed_renderer(script)
        }
    }
}

#[cfg(test)]
impl HeadlessSession {
    /// [`Self::probe_iframes_deferred`] 的进程内测试形态：无 renderer，直接在内存
    /// WebView 上同步求值（有界等待/放弃语义由真实进程集成测试覆盖）。
    pub(super) fn probe_iframes_deferred(&mut self, script: &str, _bound: std::time::Duration) -> u32 {
        match self.webview.execute_script(script) {
            Ok(text) => text.parse::<u32>().unwrap_or(0),
            Err(_) => 0,
        }
    }

    /// 测试进程内无 renderer：Input 域 IPC 发送为 no-op（形状断言在域层单测覆盖）。
    pub(super) fn send_input_mouse(
        &mut self,
        _x: f32,
        _y: f32,
        _button: u8,
        _event_type: MouseEventType,
    ) -> Result<(), String> {
        Ok(())
    }

    pub(super) fn send_input_scroll(
        &mut self,
        _delta_x: f32,
        _delta_y: f32,
        _cursor_x: f32,
        _cursor_y: f32,
    ) -> Result<(), String> {
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn send_input_key(
        &mut self,
        _key: String,
        _code: String,
        _ctrl: bool,
        _shift: bool,
        _alt: bool,
        _meta: bool,
        _text: Option<String>,
        _event_type: KeyboardEventType,
    ) -> Result<(), String> {
        Ok(())
    }

    pub(super) fn send_input_ime_commit(&mut self, _text: String) -> Result<(), String> {
        Ok(())
    }

    pub(super) fn send_set_viewport(
        &mut self,
        _width: f32,
        _height: f32,
        _device_scale_factor: f32,
    ) -> Result<(), String> {
        Ok(())
    }

    pub(super) fn send_set_color_scheme(&mut self, _dark: bool) -> Result<(), String> {
        Ok(())
    }

    pub(super) fn send_set_media_type(&mut self, _print: bool) -> Result<(), String> {
        Ok(())
    }

    pub(super) fn send_pre_document_scripts(&mut self) -> Result<(), String> {
        // T-S3 发送面守卫钉：测试进程内无 renderer——记录本应发出的登记列表
        // （全量替换语义），由 headless/tests.rs 断言；删除 page.rs 发送调用点即红。
        let sources = self
            .injected_scripts
            .iter()
            .map(|script| script.source.clone())
            .collect();
        LAST_PREDOC_SEND.with(|slot| *slot.borrow_mut() = Some(sources));
        Ok(())
    }

    /// 测试进程内无 renderer：句柄桥操作不在此层执行（renderer 单测 + cdp-e2e
    /// 覆盖语义；此处仅保证编译面完整，dispatch 层形状断言用 -32000 传回）。
    pub(super) fn automation_request(
        &mut self,
        _operation: zero_protocol::message::AutomationOperation,
    ) -> Result<zero_protocol::message::AutomationResult, String> {
        Err("renderer unavailable in unit tests".into())
    }
}

#[cfg(test)]
thread_local! {
    /// T-S3 发送面钉记录器：最近一次 PreDocumentScripts 发送的登记 source 列表
    /// （None = 复位后未发送）。headless/tests.rs 的发送面守卫钉消费并复位。
    pub(super) static LAST_PREDOC_SEND: std::cell::RefCell<Option<Vec<String>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn batch_payload(key: u64) -> IpcImagePayload {
        IpcImagePayload {
            image_key: key,
            width: 1,
            height: 1,
            rgba: vec![0, 0, 0, 255],
        }
    }

    /// P100 返修（TV-1）：headless 消费路径——同 epoch 批按 key 注入且幂等。
    #[test]
    fn image_batch_injects_by_key_idempotently() {
        let mut snapshot = crate::tab_snapshot::TabSnapshot::default();
        snapshot.begin_navigation("https://example.com/".into());
        let epoch = snapshot.navigation_epoch;
        inject_image_batch(&mut snapshot, epoch, vec![batch_payload(42), batch_payload(43)]);
        let key = |k: u64| zero_render_foundation::image_cache::ImageKey::new(k);
        assert!(snapshot.image_cache.get(&key(42)).is_some());
        assert!(snapshot.image_cache.get(&key(43)).is_some());
        // 重复批幂等（批先于/后于帧重复到达均无害）。
        inject_image_batch(&mut snapshot, epoch, vec![batch_payload(42)]);
        assert!(snapshot.image_cache.get(&key(42)).is_some());
    }

    /// P100 返修（TV-1）：过期 epoch 批随帧失效，不注入。
    #[test]
    fn stale_image_batch_is_dropped() {
        let mut snapshot = crate::tab_snapshot::TabSnapshot::default();
        snapshot.begin_navigation("https://example.com/".into());
        let epoch = snapshot.navigation_epoch;
        inject_image_batch(&mut snapshot, epoch.wrapping_sub(1), vec![batch_payload(42)]);
        let key = zero_render_foundation::image_cache::ImageKey::new(42);
        assert!(snapshot.image_cache.get(&key).is_none());
    }
}

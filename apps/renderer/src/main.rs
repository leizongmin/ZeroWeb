//! ZeroWeb 渲染进程入口 — 独立进程处理页面渲染，经 IPC 向浏览器传递绘制快照。
//!
// Windows：GUI 子系统。renderer 由 browser 通过 stdin/stdout 管道 spawn，
// 不需要控制台；不加此项 Windows 会为子进程分配一个控制台窗口。
#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

mod error_page;
mod ipc_fetch;
mod js_worker;
#[cfg(target_os = "macos")]
mod macos_app;
mod page_scripts;
mod paint_export;
mod script_prefetch;
mod text_metrics;

use zero_webview::AsyncPageLoad;

use crate::ipc_fetch::{InflightIpcFetches, IpcAsyncFetchHost, StubAsyncFetchHost};

use crate::js_worker::RendererJsWorker;
use crate::page_scripts::{DomDispatchResult, PageScriptContext, dispatch_dom_event, run_page_scripts};
use crate::script_prefetch::PendingScriptPrefetch;

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use std::io;
use zero_engine::{DomEventDetail, MediaType, PrefersColorSchemeValue, selector_from_element_hit, set_char_measure_fn};
use zero_protocol::IpcChannel;
use zero_protocol::message::{
    DispatchDomEventParams, DispatchDomEventResultParams, FetchParams, FetchResponseParams, HitTestElementResultParams,
    HitTestLinkParams, HitTestLinkResultParams, IpcColorScheme, IpcMediaType, IpcMessage, IpcMessageKind,
    KeyboardEventParams, LoadHtmlParams, MouseEventParams, NavigateParams, ScrollEventParams, SetColorSchemeParams,
    SetMediaTypeParams, SetViewportParams, StorageOpParams,
};
use zero_protocol::transport::PipeTransport;
use zero_protocol::{ProcessRole, is_disconnected_channel_message};

/// 渲染进程 → 浏览器 IPC 发送端（stdout）。
type IpcOutbound = PipeTransport<io::Empty, Box<dyn io::Write + Send>>;

fn browser_ipc_disconnected(err: &str) -> bool {
    is_disconnected_channel_message(err)
}

/// P1a form input：判定 key 是否为单字符可打印键（用于向 input/textarea 注入字符）。
/// 多字符 key 名（"Enter"/"Backspace"/"ArrowLeft"/"Shift"/"Tab"…）与控制字符排除。
fn is_printable_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if !c.is_control() => chars.next().is_none(),
        _ => false,
    }
}

/// P1a change-on-blur：读表单元素的「当前值」用于失焦 change 比对。textarea 的 value 是其
/// **文本内容**（R2702 value↔内容映射，非 value 属性）；input 取 value 属性。select 不走此路径
/// （change 在 click 派发）。host 侧 blur_focused/focus_if_text_input/focus_via_tab 共用。
fn read_input_value_for_change(html: &str, selector: &str) -> String {
    if zero_engine::query_tag_from_html(html, selector).eq_ignore_ascii_case("textarea") {
        zero_engine::query_text_from_html(html, selector)
    } else {
        zero_engine::query_attr_from_html(html, selector, "value")
    }
}

fn spawn_browser_ipc_inbound() -> (Receiver<IpcMessage>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let join = thread::Builder::new()
        .name("renderer-ipc-in".into())
        .spawn(move || {
            let mut transport = PipeTransport::new(io::stdin(), io::empty());
            loop {
                match transport.recv() {
                    Ok(msg) => {
                        if tx.send(msg).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Renderer stdin IPC reader stopped: {e}");
                        break;
                    }
                }
            }
        })
        .expect("spawn renderer ipc inbound reader");
    (rx, join)
}
use zero_render_foundation::font::loader::FontLoader;

/// 单帧渲染预算（ms）——与 tab_worker 对齐，每 tick 推进一小步后归还 IPC 循环。
const RENDER_FRAME_BUDGET_MS: f64 = 16.0;
/// 分阶段加载最长 wall-clock 时间（复杂页面如 qq.com 布局可能需数十秒）。
const PAGE_LOAD_DEADLINE: Duration = Duration::from_secs(120);
/// 无 IPC 消息时推进 pending load 的轮询间隔。
const LOAD_TICK_INTERVAL: Duration = Duration::from_millis(16);

/// 进行中的分阶段页面加载（异步 tick，不阻塞 IPC 消息循环）。
struct PendingLoad {
    load: AsyncPageLoad,
    page_url: String,
    deadline: Instant,
    run_scripts_after: bool,
    emit_load_complete: bool,
}

/// 渲染进程运行时状态。
struct RendererRuntime {
    /// 向浏览器写入 IPC（stdout）。
    outbound: IpcOutbound,
    /// 浏览器 → 渲染进程消息（stdin 读线程填充）。
    inbound_rx: Receiver<IpcMessage>,
    /// 持有 IPC 读线程 JoinHandle（仅保活，不被读；drop 即分离线程）。
    #[allow(dead_code)]
    inbound_thread: Option<JoinHandle<()>>,
    /// 当前视口（CSS 逻辑像素），随 SetViewport 更新；publish 用。
    viewport: (u32, u32),
    /// 页面运行时（B3：渲染/字体/脚本/hit-test 全经 WebView，与 tabworker 同一页面运行时）。
    webview: Option<zero_webview::WebView>,
    /// 字体加载器：为 paint 阶段提供真实字符 advance。
    font_loader: FontLoader,
    /// 当前主字体 id。
    font_id: Option<u32>,
    /// 当前 URL。
    current_url: Option<String>,
    /// 消息 ID 计数器。
    next_msg_id: u64,
    /// Fetch 请求 ID 计数器。
    next_fetch_id: u64,
    /// 渲染进程 ID。
    renderer_id: u64,
    /// 导航历史栈。
    history: Vec<String>,
    /// 当前历史索引。
    history_index: usize,
    /// 等待处理的浏览器侧消息（fetch 阻塞 recv 时暂存）。
    deferred_inbound: VecDeque<IpcMessage>,
    /// 当前页面 HTML（脚本执行后同步更新）。
    cached_html: String,
    /// 当前页面附加 CSS。
    cached_css: String,
    /// JS 执行 worker。
    js_worker: RendererJsWorker,
    /// 是否允许执行 JavaScript。
    javascript_enabled: bool,
    /// 最近一次交互目标选择器（键盘事件）。
    event_target: String,
    /// 与浏览器 TabSnapshot 对齐的导航世代。
    navigation_epoch: u64,
    /// 已发送图片 key（S8）：browser 端 ImageCache 已存则不再重传像素；navigation 重置。
    sent_image_keys: std::collections::HashSet<u64>,
    /// 异步分阶段加载（与 tab_worker 相同 tick 模型）。
    pending_load: Option<PendingLoad>,
    /// 页面 HTML/CSS/图片加载完成后的非阻塞脚本预取。
    pending_script_prefetch: Option<PendingScriptPrefetch>,
    /// 进行中的非阻塞 IPC fetch（request_id → Receiver 完成端）。
    inflight_fetches: InflightIpcFetches,
    /// in-process 测试无 browser 进程时，避免阻塞 IPC / 子资源永久 pending。
    stub_network: bool,
    /// P1a Slice 2b：observer host-tick 重入守卫——`publish_webview` 末尾触发 tick，tick 回调
    /// 若改 DOM → rerender → 再次 `publish_webview`；depth>0 时跳过 tick，防 tick→rerender→tick
    /// 链（observer 仅在 cross/size-change 时派发，本身收敛；此守卫为兜底，单次外部触发最多 2 次 publish）。
    observer_tick_depth: u32,
    /// P1a change-on-blur：当前焦点文本输入的 stable selector（失焦时据此派发 blur+change）。
    focus_target: Option<String>,
    /// P1a change-on-blur：焦点元素获焦时的 value（失焦时与当前 value 比，变化才派发 change）。
    focus_value: Option<String>,
    /// R2942 mirror：子资源 fetch/decode 失败 `(kind, url)`（stylesheet/image）。load 完成时从
    /// `AsyncPageLoad.take_failed_resources` drain 并 stash，脚本阶段经 `finish_page_load` 派 window 'error'。
    pending_resource_errors: Vec<(String, String)>,
    /// R2943 mirror：img 元素级 load/error `(绝对 URL, "load"/"error")`。load 完成时 drain 并 stash，
    /// 脚本阶段经 `finish_page_load` 派 `__zw_dispatch_img_event`。
    pending_img_events: Vec<(String, &'static str)>,
    /// R2944 mirror：stylesheet 元素级 load/error `(绝对 URL, "load"/"error")`。load 完成时 drain 并 stash，
    /// 脚本阶段经 `finish_page_load` 派 `__zw_dispatch_link_event`。
    pending_link_events: Vec<(String, &'static str)>,
    /// R2947：@font-face 加载结果 `(family, "loaded"/"error")`。load 完成时从
    /// `AsyncPageLoad.take_font_events` drain 并 stash，脚本阶段经 `finish_page_load` 派 FontFaceSet
    /// 'loadingdone'/'loadingerror' + 解析 `document.fonts.ready`。
    pending_font_events: Vec<(String, &'static str)>,
}

impl RendererRuntime {
    /// 创建新的渲染进程运行时。
    fn new(renderer_id: u64) -> Self {
        let (inbound_rx, inbound_thread) = spawn_browser_ipc_inbound();
        let mut rt = Self::with_io(renderer_id, Box::new(io::stdout()), inbound_rx);
        rt.inbound_thread = Some(inbound_thread);
        rt
    }

    /// 用指定出站 writer + 入站通道构造（`new()` 走 stdin/stdout；本方法供 in-process 测试，
    /// 是 B3 cutover 回归门的基础——renderer 是 bin，否则 wiring 无法单测）。
    fn with_io(renderer_id: u64, outbound: Box<dyn io::Write + Send>, inbound_rx: Receiver<IpcMessage>) -> Self {
        let outbound = PipeTransport::new(io::empty(), outbound);
        let (font_loader, font_id, font_resolver) = load_system_fonts();
        set_char_measure_fn(text_metrics::measure_char);
        let js_worker = RendererJsWorker::spawn(renderer_id);
        // P1b S3 / R2923（镜像 browser tab_worker）：注入生产 fetch handler（经 zero_net::HttpClient::send
        // 真实 HTTP，支持全方法/头/体）。js_worker 早于 WebView 创建，但 HttpClient::new() 自建 reqwest
        // client，无需 WebView 句柄，故可在 spawn 后立即注入。test 构建不注入（renderer runtime 单测用合成 handler）。
        #[cfg(not(test))]
        js_worker.set_fetch_handler(crate::js_worker::default_fetch_handler());
        // B3：renderer 内部持有 WebView，渲染/字体/脚本全经 WebView（与 tabworker 同一页面运行时）。
        // external_script 委派 js_worker（避免双 V8）；font_resolver 设到 WebView（paint 字体面）。
        let mut webview = zero_webview::WebView::new(zero_webview::WebViewConfig {
            width: 1280,
            height: 800,
            external_script: Some(js_worker.executor()),
            ..Default::default()
        });
        webview.set_font_resolver(font_resolver);
        // R2202 U1b-wiring 生产接通：注入 per-family 行度量（env-gated ZW_PERFONT_LINEHEIGHT=1，
        // 默认 dormant 零回归；激活后 line-height:normal 走真实 ascent−descent+line_gap）。
        webview.set_font_metric_map(font_loader.build_line_metric_map());
        Self {
            outbound,
            inbound_rx,
            inbound_thread: None,
            viewport: (1280, 800),
            webview: Some(webview),
            font_loader,
            font_id,
            current_url: None,
            next_msg_id: 1,
            next_fetch_id: 1,
            renderer_id,
            history: Vec::new(),
            history_index: 0,
            deferred_inbound: VecDeque::new(),
            cached_html: String::new(),
            cached_css: String::new(),
            js_worker,
            javascript_enabled: true,
            event_target: "body".to_string(),
            navigation_epoch: 0,
            // 性能门禁优化 S8（2026-08-08）：已发送图片 key——browser 端 ImageCache
            // 已存则不再重传像素（DOM 变更 publish 的 ViewPainted 体积大头）。navigation 重置。
            sent_image_keys: std::collections::HashSet::new(),
            pending_load: None,
            pending_script_prefetch: None,
            inflight_fetches: InflightIpcFetches::new(),
            stub_network: false,
            observer_tick_depth: 0,
            focus_target: None,
            focus_value: None,
            pending_resource_errors: Vec::new(),
            pending_img_events: Vec::new(),
            pending_link_events: Vec::new(),
            pending_font_events: Vec::new(),
        }
    }

    fn alloc_msg_id(&mut self) -> u64 {
        let id = self.next_msg_id;
        self.next_msg_id += 1;
        id
    }

    fn send(&mut self, kind: IpcMessageKind) -> Result<(), String> {
        let msg = IpcMessage {
            id: self.alloc_msg_id(),
            kind,
        };
        self.outbound.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))
    }

    fn send_with_id(&mut self, id: u64, kind: IpcMessageKind) -> Result<(), String> {
        let msg = IpcMessage { id, kind };
        self.outbound.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))
    }

    fn after_page_html_loaded_with_cache(&mut self, fetch_cache: HashMap<String, String>) -> Result<(), String> {
        let js_enabled = self.javascript_enabled;
        let current_url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let skip = page_scripts::should_skip_scripts(&current_url);
        let changed = {
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &current_url,
                js_worker: &self.js_worker,
            };
            let fetch_from_cache = |url: &str| {
                fetch_cache
                    .get(url)
                    .cloned()
                    .ok_or_else(|| format!("script fetch failed: {url}"))
            };
            run_page_scripts(&mut ctx, js_enabled, fetch_from_cache)
        };
        if changed {
            self.rerender_publish_webview()?;
        }
        // R2940–R2944 mirror：脚本阶段收尾——派发页面生命周期（DOMContentLoaded + load）+ 子资源 fetch 失败
        // window 'error' + img/link 元素级 load/error。与 browser tab_scripts::PageScriptRunner::finish 对齐，
        // 使默认多进程路径具备事件 API parity（此前仅 --single-process 路径派发）。JS 关 / view-source 跳过。
        // 无脚本但 JS 启用的页面（仅 `<body onload>`）也派发——finish_page_load 内 lifecycle 无条件执行。
        if js_enabled && !skip {
            let resource_errors = std::mem::take(&mut self.pending_resource_errors);
            let img_events = std::mem::take(&mut self.pending_img_events);
            let link_events = std::mem::take(&mut self.pending_link_events);
            let font_events = std::mem::take(&mut self.pending_font_events);
            page_scripts::finish_page_load(&self.js_worker, resource_errors, img_events, link_events, font_events);
        }
        Ok(())
    }

    /// 非阻塞推进脚本预取；完成后执行页面脚本。
    fn tick_script_prefetch(&mut self) -> Result<(), String> {
        self.drain_inflight_fetch_responses();
        let Some(mut prefetch) = self.pending_script_prefetch.take() else {
            return Ok(());
        };

        const SCRIPT_PREFETCH_PARALLEL: usize = 4;
        let _changed = if self.stub_network {
            let mut host = StubAsyncFetchHost;
            prefetch.tick(&mut host, SCRIPT_PREFETCH_PARALLEL)
        } else {
            let outbound = &mut self.outbound;
            let next_fetch_id = &mut self.next_fetch_id;
            let inflight = &mut self.inflight_fetches;
            let mut host = IpcAsyncFetchHost::new(outbound, next_fetch_id, inflight);
            prefetch.tick(&mut host, SCRIPT_PREFETCH_PARALLEL)
        };

        if prefetch.is_active() {
            self.pending_script_prefetch = Some(prefetch);
            return Ok(());
        }

        let cache = prefetch.finish();
        self.after_page_html_loaded_with_cache(cache)?;
        self.try_publish_progress(true)
    }

    /// 从 WebView 当前渲染产出发布 IPC frame（ViewPainted + 可选 Title）。B3：发布源切到 WebView。
    fn publish_webview(&mut self, title: Option<String>, allow_network_fetch: bool) -> Result<(), String> {
        let html = self.cached_html.clone();
        let url = self.current_url.clone().unwrap_or_else(|| "about:blank".into());
        let (vw, vh, document_height, primitives, hit_test, mut image_cache) = {
            let wv = self.webview.as_ref().expect("webview");
            let render = wv.last_render().ok_or_else(|| "WebView 无渲染结果".to_string())?;
            (
                self.viewport.0,
                self.viewport.1,
                wv.document_height().unwrap_or(self.viewport.1 as f32),
                render.primitives.clone(),
                wv.build_hit_test_cache(),
                wv.snapshot_image_cache(),
            )
        };
        // P1a gBCR：render 后用最新 layout 填 rect snapshot——js_worker 的 RectBridge handler
        // 经 identity(selector)→NodeId 查此 snapshot 返真实 DOMRect（未填/未命中→零 rect，零回归）。
        if let Some(cache) = hit_test.as_ref() {
            cache.fill_layout_rect_snapshot(&self.js_worker.rect_snapshot());
            // P1a elementFromPoint：render 后 swap 最新 `Arc<HitTestCache>` 进共享槽（无数据 clone，
            // 仅引用计数）→ js_worker 的 `__zw_elementFromPoint` 读它求 `(x,y)` 命中元素。
            *self.js_worker.element_from_point_cache().lock().unwrap() = Some(std::sync::Arc::new(cache.clone()));
        }
        // S8：已发送图片 key（browser 端 ImageCache 已存）不重传像素。
        // fetch 闭包按字段捕获（2021 edition 最小捕获）——sent_image_keys 独立借用
        let payloads = if allow_network_fetch {
            let mut fetch = |u: &str| {
                if self.stub_network {
                    None
                } else {
                    ipc_fetch_get(
                        &mut self.outbound,
                        &self.inbound_rx,
                        &mut self.next_fetch_id,
                        &mut self.deferred_inbound,
                        u,
                    )
                    .ok()
                }
            };
            paint_export::fetch_image_payloads_with_cache(
                &html,
                &url,
                &mut image_cache,
                &mut fetch,
                &mut self.sent_image_keys,
            )
        } else {
            let mut no_fetch = |_u: &str| None;
            paint_export::fetch_image_payloads_with_cache(
                &html,
                &url,
                &mut image_cache,
                &mut no_fetch,
                &mut self.sent_image_keys,
            )
        };
        let frame = zero_page_runtime::FrameModel {
            viewport: (vw, vh),
            document_height,
            primitives,
            hit_test,
        };
        publish_render_with_layout(
            &mut self.outbound,
            &mut self.next_msg_id,
            &frame,
            title,
            payloads,
            self.navigation_epoch,
        )?;
        // P1a Slice 2b：render 填完 rect snapshot 后触发 observer 重算——IO `_crossed`（threshold
        // 越界）/ RO size-diff 仅在变化时派发，故 observe() 之后的真实 layout 变化能触发 observer 回调
        // （observe 仅派发 initial）。depth 守卫防 tick→rerender→publish→tick 反馈环（observer 本身
        // 收敛，此为兜底）；kill-switch：gBCR 关（ZW_REAL_RECT=0，rect 恒零）或 JS 关时跳过。
        if self.observer_tick_depth == 0 && self.javascript_enabled && crate::js_worker::real_rect_enabled() {
            self.observer_tick_depth += 1;
            let tick_res = self.tick_observers_inner();
            self.observer_tick_depth -= 1;
            tick_res?;
        }
        Ok(())
    }

    /// P1a Slice 2b：执行 observer tick（`__zw_observers_tick()`）并 apply 回调产生的 DOM mutation。
    /// 回调改了 DOM → 单次 rerender（rerender 再入 publish_webview，但 depth>0 跳过 tick，有界）。
    fn tick_observers_inner(&mut self) -> Result<(), String> {
        let url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let changed = {
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &url,
                js_worker: &self.js_worker,
            };
            page_scripts::tick_observers(&mut ctx)
        };
        if changed {
            self.rerender_publish_webview()?;
        }
        Ok(())
    }

    /// P1a form input：向 selector 指向的焦点 input/textarea 注入一个字符（更新 value + 派发
    /// 'input' 事件）；回调改了 DOM 则单次 rerender。调用方须先判定 key 为可打印单字符。
    fn apply_text_input_at(&mut self, selector: &str, key: &str) -> Result<(), String> {
        if !self.javascript_enabled {
            return Ok(());
        }
        let url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let changed = {
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &url,
                js_worker: &self.js_worker,
            };
            page_scripts::apply_text_input(&mut ctx, selector, key)
        };
        if changed {
            self.rerender_publish_webview()?;
        }
        Ok(())
    }

    /// P1a form input：Backspace 删焦点 input/textarea 末字符（value + input 事件）；改 DOM 则单次 rerender。
    fn apply_text_delete_at(&mut self, selector: &str) -> Result<(), String> {
        if !self.javascript_enabled {
            return Ok(());
        }
        let url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let changed = {
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &url,
                js_worker: &self.js_worker,
            };
            page_scripts::apply_text_delete(&mut ctx, selector)
        };
        if changed {
            self.rerender_publish_webview()?;
        }
        Ok(())
    }

    /// P1a form submit：Enter 在单行 input → 解析 enclosing `<form>` 派发 'submit' 事件。
    /// R3054：未 preventDefault 且 method=GET → 导航到 action?query（闭合 click 默认动作族）。
    fn submit_form_on_enter_at(&mut self, selector: &str) -> Result<(), String> {
        if !self.javascript_enabled {
            return Ok(());
        }
        let url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let outcome = {
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &url,
                js_worker: &self.js_worker,
            };
            page_scripts::apply_submit_on_enter(&mut ctx, selector)
        };
        if outcome.html_changed {
            self.rerender_publish_webview()?;
        }
        // R3054：implicit submit（submitter=None）未取消 → GET 导航。
        if outcome.default_allowed {
            self.navigate_form_get(selector, None)?;
        }
        Ok(())
    }

    /// P1a form submit：click 命中 submit button → 解析 enclosing `<form>` 派发 'submit' 事件。
    /// R3054：未 preventDefault 且 method=GET → 导航到 action?query（submitter name=value 入 query）。
    fn submit_form_on_click_at(&mut self, selector: &str) -> Result<(), String> {
        if !self.javascript_enabled {
            return Ok(());
        }
        let url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let outcome = {
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &url,
                js_worker: &self.js_worker,
            };
            page_scripts::apply_submit_on_click(&mut ctx, selector)
        };
        if outcome.html_changed {
            self.rerender_publish_webview()?;
        }
        // R3054：click submit（submitter=该按钮）未取消 → GET 导航。
        if outcome.default_allowed {
            self.navigate_form_get(selector, Some(selector))?;
        }
        Ok(())
    }

    /// P1a 导航（R3054）：form GET 提交导航——解析 enclosing form → [`form_get_submission_url`]
    /// 算 GET 目标 URL（method=GET 且 action 可解析）→ handle_navigate。POST/form 不匹配 → no-op。
    /// 在 submit 事件派发 + apply 之后读 `cached_html`（含 listener 变更，如 JS 注入隐藏字段）。
    fn navigate_form_get(&mut self, selector: &str, submitter: Option<&str>) -> Result<(), String> {
        let base = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let html = self.cached_html.clone();
        let Some(form_sel) = zero_engine::enclosing_form_selector(&html, selector) else {
            return Ok(());
        };
        if let Some(nav_url) = zero_engine::form_get_submission_url(&html, &form_sel, submitter, &base) {
            self.handle_navigate(zero_protocol::message::NavigateParams {
                url: nav_url,
                referrer: self.current_url.clone(),
                navigation_epoch: self.navigation_epoch.wrapping_add(1),
            })?;
        }
        Ok(())
    }

    /// P1a form reset（R3050）：click 命中 reset button → 解析 enclosing `<form>` 调 `form.reset()`
    ///（dispatch 'reset' + revert 控件，复用 R3048）。改 DOM 则 rerender。
    fn reset_form_on_click_at(&mut self, selector: &str) -> Result<(), String> {
        if !self.javascript_enabled {
            return Ok(());
        }
        let url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let changed = {
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &url,
                js_worker: &self.js_worker,
            };
            page_scripts::apply_reset_on_click(&mut ctx, selector)
        };
        if changed {
            self.rerender_publish_webview()?;
        }
        Ok(())
    }

    /// P1a 导航（R3053，闭合 R3052 限制③）：click 命中 hash 链接（`<a href="#sec">`）→
    /// `location.hash = hash`（R3006：更新 hash + history entry + 派 hashchange）。SPA hash 路由核心交互。
    /// 改 DOM（SPA router listener 切视图）则 rerender。headless 无 viewport → 不滚锚。
    fn set_hash_on_click_at(&mut self, selector: &str) -> Result<(), String> {
        if !self.javascript_enabled {
            return Ok(());
        }
        let url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let changed = {
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &url,
                js_worker: &self.js_worker,
            };
            page_scripts::apply_set_hash_on_click(&mut ctx, selector)
        };
        if changed {
            self.rerender_publish_webview()?;
        }
        Ok(())
    }

    /// P1a checkbox：click 命中 checkbox → 翻转 checked + 派发 'change' 事件；改 DOM 则 rerender。
    fn toggle_checkbox_at(&mut self, selector: &str) -> Result<(), String> {
        if !self.javascript_enabled {
            return Ok(());
        }
        let url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let changed = {
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &url,
                js_worker: &self.js_worker,
            };
            page_scripts::apply_toggle_checkbox(&mut ctx, selector)
        };
        if changed {
            self.rerender_publish_webview()?;
        }
        Ok(())
    }

    /// P1a radio：click 命中 radio → set checked + 同 name 组兄弟 unset + 派发 'change'。
    fn toggle_radio_at(&mut self, selector: &str) -> Result<(), String> {
        if !self.javascript_enabled {
            return Ok(());
        }
        let url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let changed = {
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &url,
                js_worker: &self.js_worker,
            };
            page_scripts::apply_toggle_radio(&mut ctx, selector)
        };
        if changed {
            self.rerender_publish_webview()?;
        }
        Ok(())
    }

    /// P1a change-on-blur：失焦——若 `focus_target` 是文本输入，派发 'blur'；若 value 自获焦以来
    /// 变化，再派发 'change'。清空 focus 状态。回调改 DOM 则单次 rerender。
    fn blur_focused(&mut self) -> Result<(), String> {
        let Some(old) = self.focus_target.clone() else {
            return Ok(());
        };
        let old_val = self.focus_value.clone().unwrap_or_default();
        self.focus_target = None;
        self.focus_value = None;
        if !self.javascript_enabled {
            return Ok(());
        }
        let cur_val = read_input_value_for_change(&self.cached_html, &old);
        let url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let mut changed = false;
        {
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &url,
                js_worker: &self.js_worker,
            };
            changed |=
                page_scripts::dispatch_dom_event(&mut ctx, self.javascript_enabled, &old, "blur", None).html_changed;
            if cur_val != old_val {
                changed |= page_scripts::dispatch_dom_event(&mut ctx, self.javascript_enabled, &old, "change", None)
                    .html_changed;
            }
        }
        if changed {
            self.rerender_publish_webview()?;
        }
        Ok(())
    }

    /// P1a change-on-blur：获焦——若 `selector` 是文本输入，记 focus_target/value + 派发 'focus'。
    fn focus_if_text_input(&mut self, selector: &str) -> Result<(), String> {
        if !self.javascript_enabled || !zero_engine::is_text_input(&self.cached_html, selector) {
            return Ok(());
        }
        let val = read_input_value_for_change(&self.cached_html, selector);
        self.focus_target = Some(selector.to_string());
        self.focus_value = Some(val);
        let url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let changed = {
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &url,
                js_worker: &self.js_worker,
            };
            page_scripts::dispatch_dom_event(&mut ctx, self.javascript_enabled, selector, "focus", None).html_changed
        };
        if changed {
            self.rerender_publish_webview()?;
        }
        Ok(())
    }

    /// P1a Tab 焦点导航：设 event_target 到 `selector`，派发 'focus'；若为文本输入则记 focus 跟踪
    /// （供后续 change-on-blur），否则不记（blur_focused 已清旧焦点跟踪）。
    fn focus_via_tab(&mut self, selector: &str) -> Result<(), String> {
        self.event_target = selector.to_string();
        if !self.javascript_enabled {
            return Ok(());
        }
        let url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let changed = {
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &url,
                js_worker: &self.js_worker,
            };
            page_scripts::dispatch_dom_event(&mut ctx, self.javascript_enabled, selector, "focus", None).html_changed
        };
        if zero_engine::is_text_input(&self.cached_html, selector) {
            self.focus_target = Some(selector.to_string());
            self.focus_value = Some(read_input_value_for_change(&self.cached_html, selector));
        }
        if changed {
            self.rerender_publish_webview()?;
        }
        Ok(())
    }

    fn sync_cached_html_from_webview(&mut self) {
        if let Some(wv) = self.webview.as_ref() {
            let html = wv.html_content().to_string();
            if !html.is_empty() {
                self.cached_html = html;
            }
        }
    }

    fn try_publish_progress(&mut self, allow_network_fetch: bool) -> Result<(), String> {
        let title = self.webview.as_ref().and_then(|w| w.title().map(str::to_string));
        self.publish_webview(title, allow_network_fetch)
    }

    /// 非阻塞消化 inbound 中的 `FetchResponse`，避免 load tick 阻塞时 async 子资源无法完成。
    fn drain_inflight_fetch_responses(&mut self) {
        loop {
            let msg = if let Some(m) = self.deferred_inbound.pop_front() {
                m
            } else {
                match self.inbound_rx.try_recv() {
                    Ok(m) => m,
                    Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
                }
            };
            if self.inflight_fetches.try_complete(&msg) {
                continue;
            }
            self.deferred_inbound.push_back(msg);
            break;
        }
    }

    /// 推进 pending load 一步；加载完成时发布帧、LoadComplete 与可选脚本阶段。
    fn tick_pending_load(&mut self) -> Result<(), String> {
        self.drain_inflight_fetch_responses();
        self.tick_pending_load_with_budget(RENDER_FRAME_BUDGET_MS)
    }

    /// R2949 FontFace.load()：drain JS 投递的字体加载请求 → fetch_get 字节 → load_font +
    /// register_family_alias（复用既有 @font-face 加载逻辑）→ 刷新 resolver + 请求重绘 →
    /// async_resolver.resolve 解析 Promise。失败（fetch/load）resolve "err" 使 shim reject。
    /// 复用既有字体加载代码路径（与 drain_loaded_fonts 一致），仅触发条件不同（@font-face 由
    /// async_load poll_fonts 收集；FontFace.load() 由 JS __zw_load_font 投递）。
    fn tick_font_face_loads(&mut self) {
        if !zero_webview::live_fontface_enabled() {
            // 与 @font-face live 加载同 kill-switch；关闭时仍 resolve 各请求为 err（font 不会加载）。
            let pending: Vec<zero_engine::FontLoadRequest> = self
                .js_worker
                .pending_font_loads()
                .lock()
                .map(|mut q| std::mem::take(&mut *q))
                .unwrap_or_default();
            for req in pending {
                self.js_worker
                    .async_resolver()
                    .resolve(&req.resolve_id, "err:live-fontface-disabled");
            }
            return;
        }
        let pending: Vec<zero_engine::FontLoadRequest> = self
            .js_worker
            .pending_font_loads()
            .lock()
            .map(|mut q| std::mem::take(&mut *q))
            .unwrap_or_default();
        if pending.is_empty() {
            return;
        }
        let resolver = self.js_worker.async_resolver();
        let mut updated = false;
        for req in pending {
            // fetch_get 阻塞 IPC 取字节（与 image payload fetch 同机制；stub_network 时 Err）。
            let bytes = match self.fetch_get(&req.src) {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!(family = %req.family, src = %req.src, err = %e, "FontFace.load fetch failed");
                    resolver.resolve(&req.resolve_id, "err:fetch");
                    continue;
                }
            };
            match self.register_loaded_font(&req.family, req.weight, req.is_italic, &bytes) {
                true => {
                    updated = true;
                    resolver.resolve(&req.resolve_id, "ok");
                }
                false => {
                    resolver.resolve(&req.resolve_id, "err:load");
                }
            }
        }
        if updated {
            let font_resolver = self.font_loader.build_font_resolver();
            if let Some(wv) = self.webview.as_mut() {
                wv.set_font_resolver(font_resolver);
            }
            // 请求重绘使新字体生效——经 pending_load（若有）的 request_rerender，否则直接 try_publish。
            if let Some(pending) = self.pending_load.as_mut() {
                pending.load.request_rerender();
            } else {
                let _ = self.try_publish_progress(false);
            }
        }
    }

    /// 加载字体字节并按 (weight, style) 注册 alias（R2417/R2493 键规则）。返 true=注册成功（需刷新 resolver）。
    /// 抽自 drain_loaded_fonts 字体加载块，供 @font-face（async_load）与 FontFace.load()（JS 投递）共用。
    fn register_loaded_font(&mut self, family: &str, weight: Option<u16>, is_italic: bool, bytes: &[u8]) -> bool {
        let Ok(id) = self.font_loader.load_font(bytes) else {
            tracing::warn!(family = %family, "font load_font failed");
            return false;
        };
        let want_bold = weight.is_some_and(|w| w >= 600);
        let key = match (want_bold, is_italic) {
            (true, true) => format!("{family}:700:italic"),
            (true, false) => format!("{family}:700"),
            (false, true) => format!("{family}:italic"),
            (false, false) => family.to_string(),
        };
        self.font_loader.register_family_alias(&key, id);
        true
    }

    fn tick_pending_load_with_budget(&mut self, budget_ms: f64) -> Result<(), String> {
        let Some(mut pending) = self.pending_load.take() else {
            return Ok(());
        };

        if Instant::now() >= pending.deadline {
            let url = pending.page_url;
            return self.show_error_page(&url, "页面加载超时（分阶段加载未完成）");
        }

        let publish_after = {
            let webview = self.webview.as_mut().expect("webview");
            let font_loader = &self.font_loader;
            let font_id = self.font_id;
            text_metrics::with_measure_ctx_opt(font_loader, font_id, || {
                if self.stub_network {
                    let mut host = StubAsyncFetchHost;
                    let changed = pending.load.tick(webview, &mut host, budget_ms);
                    return changed
                        && webview.last_render().is_some()
                        && !matches!(pending.load.stage(), zero_webview::PageLoadStage::FetchingDocument);
                }
                let outbound = &mut self.outbound;
                let next_fetch_id = &mut self.next_fetch_id;
                let inflight = &mut self.inflight_fetches;
                let mut host = IpcAsyncFetchHost::new(outbound, next_fetch_id, inflight);
                let changed = pending.load.tick(webview, &mut host, budget_ms);
                changed
                    && webview.last_render().is_some()
                    && !matches!(pending.load.stage(), zero_webview::PageLoadStage::FetchingDocument)
            })
        };

        // R2408+ slice 2：drain 已 fetch 的 @font-face 字节 → load_font + register_family_alias
        // → 刷新 webview font_resolver → 请求重绘。须在 with_measure_ctx_opt 闭包外（闭包内
        // font_loader 被不可变借做文本度量，此处可 &mut）。env `ZW_LIVE_FONTFACE` kill-switch
        // 默认开（=0/`false` 关闭，退回 R2406 前丢弃行为）。
        if zero_webview::live_fontface_enabled() {
            let loaded = pending.load.drain_loaded_fonts();
            if !loaded.is_empty() {
                let mut updated = false;
                for (family, weight, is_italic, bytes) in loaded {
                    // R2417/R2493（weight, style）注册键规则抽入 register_loaded_font，
                    // 与 FontFace.load()（tick_font_face_loads）共用——bold/italic face 不注册到 plain family
                    //（否则 build_font_resolver 的「second face=bold」启发式顺序依赖错配，R2417）。
                    if self.register_loaded_font(&family, weight, is_italic, &bytes) {
                        updated = true;
                    } else {
                        tracing::warn!(family = %family, "live @font-face load failed");
                    }
                }
                if updated {
                    let resolver = self.font_loader.build_font_resolver();
                    if let Some(wv) = self.webview.as_mut() {
                        wv.set_font_resolver(resolver);
                    }
                    pending.load.request_rerender();
                }
            }
        }

        let stage = pending.load.stage();
        if publish_after {
            self.sync_cached_html_from_webview();
            tracing::info!(url = %pending.page_url, stage = ?stage, "progressive paint publish");
            // 加载过程中仅用已解码 cache，避免同步 IPC fetch 阻塞 async 子资源。
            self.try_publish_progress(false)?;
        }

        if pending.load.is_active() {
            self.pending_load = Some(pending);
            return Ok(());
        }

        if let Some(err) = pending.load.take_error() {
            let url = pending.page_url;
            return self.show_error_page(&url, &err);
        }

        let page_url = pending.page_url;
        let run_scripts = pending.run_scripts_after;
        let emit_complete = pending.emit_load_complete;

        // R2942/R2943/R2944 mirror：load 完成（!is_active 且无 error）时 drain 子资源加载结果，
        // stash 到 self 供后续脚本阶段 `finish_page_load` 派发（脚本阶段在 prefetch 完成后才跑，
        // 晚于本点，故须暂存）。drain 在 take_error 之后、error 分支已 return，确保仅成功 load 才 drain。
        self.pending_resource_errors = pending
            .load
            .take_failed_resources()
            .into_iter()
            .map(|r| (r.kind.to_string(), r.url))
            .collect();
        self.pending_img_events = pending.load.take_img_element_events();
        self.pending_link_events = pending.load.take_link_element_events();
        self.pending_font_events = pending.load.take_font_events();

        self.sync_cached_html_from_webview();
        self.try_publish_progress(true)?;
        if emit_complete {
            self.send(IpcMessageKind::LoadComplete)?;
            tracing::info!("页面渲染完成: {page_url}");
        }
        if run_scripts && !page_scripts::should_skip_scripts(&page_url) {
            self.pending_script_prefetch = Some(PendingScriptPrefetch::from_html(&page_url, &self.cached_html));
        }
        Ok(())
    }

    fn start_pending_load(&mut self, pending: PendingLoad) -> Result<(), String> {
        self.pending_load = Some(pending);
        self.tick_pending_load()
    }

    fn recv_next_or_timeout(&mut self, timeout: Duration) -> Result<Option<IpcMessage>, String> {
        if let Some(msg) = self.deferred_inbound.pop_front() {
            return Ok(Some(msg));
        }
        match self.inbound_rx.recv_timeout(timeout) {
            Ok(msg) => Ok(Some(msg)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => Err("IPC 通道已关闭".into()),
        }
    }

    /// 用当前 cached_html/css 经 WebView 重绘并发布（脚本改 DOM 后的重渲染路径）。
    fn rerender_publish_webview(&mut self) -> Result<(), String> {
        let html = self.cached_html.clone();
        let font_loader = &self.font_loader;
        let font_id = self.font_id;
        let wv = self.webview.as_mut().expect("webview");
        text_metrics::with_measure_ctx_opt(font_loader, font_id, || {
            // DOM 变更不能丢弃异步页面加载器已经加载的外链样式表。
            // https://html.spec.whatwg.org/multipage/semantics.html#the-link-element
            wv.reload_html_after_script(&html);
        });
        self.publish_webview(None, true)
    }

    fn dispatch_dom_at(
        &mut self,
        selector: Option<String>,
        x: f32,
        y: f32,
        event_type: &str,
        detail: Option<DomEventDetail>,
    ) -> DomDispatchResult {
        let selector = selector.or_else(|| {
            self.webview
                .as_ref()
                .and_then(|wv| wv.hit_test_element(x, y))
                .map(|hit| selector_from_element_hit(&hit))
        });
        if let Some(sel) = selector {
            self.event_target = sel.clone();
            let js_enabled = self.javascript_enabled;
            let current_url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
            let mut ctx = PageScriptContext {
                html: &mut self.cached_html,
                url: &current_url,
                js_worker: &self.js_worker,
            };
            let result = dispatch_dom_event(&mut ctx, js_enabled, &sel, event_type, detail.as_ref());
            if result.html_changed {
                let _ = self.rerender_publish_webview();
            }
            result
        } else {
            DomDispatchResult {
                default_allowed: true,
                html_changed: false,
            }
        }
    }

    /// 经浏览器 IPC 代理 GET 请求。
    fn fetch_get(&mut self, url: &str) -> Result<Vec<u8>, String> {
        if self.stub_network {
            return Err(format!("stub network (no browser process): {url}"));
        }
        ipc_fetch_get(
            &mut self.outbound,
            &self.inbound_rx,
            &mut self.next_fetch_id,
            &mut self.deferred_inbound,
            url,
        )
    }

    fn push_history(&mut self, url: &str) {
        if self.history_index + 1 < self.history.len() {
            self.history.truncate(self.history_index + 1);
        }
        if self.history.last().map(String::as_str) != Some(url) {
            self.history.push(url.to_string());
            self.history_index = self.history.len().saturating_sub(1);
        }
    }

    fn history_url(&self, index: usize) -> Option<&str> {
        self.history.get(index).map(String::as_str)
    }

    fn run_staged_load(
        &mut self,
        page_url: String,
        html: String,
        push_history: bool,
        send_complete: bool,
    ) -> Result<(), String> {
        if push_history {
            self.push_history(&page_url);
        }
        self.send(IpcMessageKind::UrlChanged(page_url.clone()))?;
        self.current_url = Some(page_url.clone());
        self.cached_html = html.clone();
        self.cached_css = String::new();

        self.webview
            .as_mut()
            .expect("webview")
            .prepare_document_state(&page_url);
        self.start_pending_load(PendingLoad {
            load: AsyncPageLoad::from_html(page_url.clone(), html),
            page_url,
            deadline: Instant::now() + PAGE_LOAD_DEADLINE,
            run_scripts_after: true,
            emit_load_complete: send_complete,
        })
    }

    fn show_error_page(&mut self, page_url: &str, error: &str) -> Result<(), String> {
        tracing::error!("页面加载失败 ({page_url}): {error}");
        self.pending_load = None;
        self.send(IpcMessageKind::LoadFailed(error.to_string()))?;
        let html = error_page::generate_error_page(page_url, error);
        let error_url = format!("error://{page_url}");
        self.send(IpcMessageKind::UrlChanged(error_url.clone()))?;
        self.current_url = Some(error_url.clone());
        self.cached_html = html.clone();
        self.cached_css.clear();
        self.webview
            .as_mut()
            .expect("webview")
            .prepare_document_state(&error_url);
        self.start_pending_load(PendingLoad {
            load: AsyncPageLoad::from_html(error_url.clone(), html),
            page_url: error_url,
            deadline: Instant::now() + PAGE_LOAD_DEADLINE,
            run_scripts_after: false,
            emit_load_complete: false,
        })?;
        self.send(IpcMessageKind::TitleChanged("加载失败".to_string()))
    }

    fn handle_navigate(&mut self, params: NavigateParams) -> Result<(), String> {
        tracing::info!("导航到: {}", params.url);
        self.pending_load = None;
        self.pending_script_prefetch = None;
        self.inflight_fetches.clear();
        self.push_history(&params.url);
        self.send(IpcMessageKind::UrlChanged(params.url.clone()))?;
        self.current_url = Some(params.url.clone());
        self.cached_html.clear();
        self.cached_css.clear();
        // S8：新页面图片 key 空间不同——清空已发送记录，确保新页图片像素被传输
        self.sent_image_keys.clear();
        // P1a change-on-blur：导航清焦点状态（新页面无焦点）。
        self.focus_target = None;
        self.focus_value = None;

        self.navigation_epoch = params.navigation_epoch;
        let page_url = params.url.clone();
        self.webview
            .as_mut()
            .expect("webview")
            .prepare_document_state(&page_url);
        self.start_pending_load(PendingLoad {
            load: AsyncPageLoad::start(page_url.clone()),
            page_url,
            deadline: Instant::now() + PAGE_LOAD_DEADLINE,
            run_scripts_after: true,
            emit_load_complete: true,
        })
    }

    fn handle_load_html(&mut self, params: LoadHtmlParams) -> Result<(), String> {
        self.navigation_epoch = params.navigation_epoch;
        // P1a change-on-blur：加载新 HTML 清焦点状态。
        self.focus_target = None;
        self.focus_value = None;
        let page_url = params.url.clone().unwrap_or_else(|| "about:blank".to_string());
        tracing::info!("加载内联 HTML: {page_url}");
        self.cached_css = params.css.clone().unwrap_or_default();
        let css = self.cached_css.as_str();
        let mut html = params.html;
        if !css.is_empty() {
            html.push_str("\n<style>\n");
            html.push_str(css);
            html.push_str("\n</style>\n");
        }
        self.run_staged_load(page_url, html, true, true)
    }

    fn try_republish_cached(&mut self) -> Result<(), String> {
        let font_loader = &self.font_loader;
        let font_id = self.font_id;
        let wv = self.webview.as_mut().expect("webview");
        text_metrics::with_measure_ctx_opt(font_loader, font_id, || {
            wv.render();
        });
        self.publish_webview(None, true)
    }

    fn handle_set_viewport(&mut self, params: SetViewportParams) -> Result<(), String> {
        self.viewport = (params.width, params.height);
        if let Some(wv) = self.webview.as_mut() {
            wv.resize(params.width, params.height);
        }
        self.try_republish_cached()
    }

    fn handle_set_color_scheme(&mut self, params: SetColorSchemeParams) -> Result<(), String> {
        if let Some(wv) = self.webview.as_mut() {
            wv.set_prefers_color_scheme(ipc_scheme_to_engine(params.scheme));
        }
        self.try_republish_cached()
    }

    fn handle_set_media_type(&mut self, params: SetMediaTypeParams) -> Result<(), String> {
        if let Some(wv) = self.webview.as_mut() {
            wv.set_media_type(ipc_media_to_engine(params.media_type));
        }
        self.try_republish_cached()
    }

    fn reload_history_entry(&mut self, index: usize) -> Result<(), String> {
        let url = self
            .history_url(index)
            .ok_or_else(|| "历史记录为空".to_string())?
            .to_string();
        self.history_index = index;
        match self.fetch_get(&url) {
            Ok(body) => {
                let html = String::from_utf8_lossy(&body).into_owned();
                self.run_staged_load(url, html, false, true)
            }
            Err(e) => self.show_error_page(&url, &e),
        }
    }

    fn handle_go_back(&mut self) -> Result<(), String> {
        if self.history_index == 0 {
            tracing::info!("已在历史栈起点，无法后退");
            return self.send(IpcMessageKind::Ok);
        }
        tracing::info!("后退导航");
        self.reload_history_entry(self.history_index - 1)
    }

    fn handle_go_forward(&mut self) -> Result<(), String> {
        if self.history_index + 1 >= self.history.len() {
            tracing::info!("已在历史栈末尾，无法前进");
            return self.send(IpcMessageKind::Ok);
        }
        tracing::info!("前进导航");
        self.reload_history_entry(self.history_index + 1)
    }

    fn handle_storage_op(&mut self, params: StorageOpParams) -> Result<(), String> {
        tracing::trace!("StorageOp 转发到浏览器: {:?}", params.operation);
        self.send(IpcMessageKind::StorageOp(params))
    }

    fn handle_heartbeat(&mut self) -> Result<(), String> {
        self.send(IpcMessageKind::Heartbeat)
    }

    fn handle_hit_test_link(&mut self, msg_id: u64, params: HitTestLinkParams) -> Result<(), String> {
        let href = self
            .webview
            .as_ref()
            .expect("webview")
            .hit_test_link(params.x, params.y);
        tracing::trace!("HitTestLink({msg_id}) -> {:?}", href.as_deref());
        self.send_with_id(
            msg_id,
            IpcMessageKind::HitTestLinkResult(HitTestLinkResultParams { href }),
        )
    }

    fn handle_hit_test_image(&mut self, msg_id: u64, params: HitTestLinkParams) -> Result<(), String> {
        let src = self
            .webview
            .as_ref()
            .expect("webview")
            .hit_test_image(params.x, params.y);
        tracing::trace!("HitTestImage({msg_id}) -> {:?}", src.as_deref());
        self.send_with_id(
            msg_id,
            IpcMessageKind::HitTestImageResult(HitTestLinkResultParams { href: src }),
        )
    }

    fn handle_hit_test_element(&mut self, msg_id: u64, params: HitTestLinkParams) -> Result<(), String> {
        let result = self
            .webview
            .as_ref()
            .expect("webview")
            .hit_test_element(params.x, params.y)
            .map(|hit| {
                let selector = selector_from_element_hit(&hit);
                HitTestElementResultParams {
                    tag_name: Some(hit.tag_name),
                    id: hit.id,
                    class_name: hit.class_name,
                    x: hit.x,
                    y: hit.y,
                    width: hit.width,
                    height: hit.height,
                    selector: Some(selector),
                }
            })
            .unwrap_or(HitTestElementResultParams {
                tag_name: None,
                id: None,
                class_name: None,
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                selector: None,
            });
        self.send_with_id(msg_id, IpcMessageKind::HitTestElementResult(result))
    }

    fn handle_dispatch_dom_event(&mut self, msg_id: u64, params: DispatchDomEventParams) -> Result<(), String> {
        let detail = if params.key.is_some() || params.code.is_some() {
            Some(DomEventDetail {
                key: params.key,
                code: params.code,
                ..Default::default()
            })
        } else {
            None
        };
        let result = self.dispatch_dom_at(params.selector, params.x, params.y, &params.event_type, detail);
        self.send_with_id(
            msg_id,
            IpcMessageKind::DispatchDomEventResult(DispatchDomEventResultParams {
                default_allowed: result.default_allowed,
            }),
        )
    }

    fn handle_mouse_event(&mut self, params: MouseEventParams) -> Result<(), String> {
        use zero_protocol::message::MouseEventType;
        let event_type = match params.event_type {
            MouseEventType::Down => "mousedown",
            MouseEventType::Up => "mouseup",
            MouseEventType::Move => "mousemove",
            MouseEventType::Click => "click",
            MouseEventType::DblClick => "dblclick",
        };
        // R3052：capture click 的 default_allowed（preventDefault 未调用）供 anchor 导航判定。
        let click_default_allowed = if event_type != "mousemove" {
            self.dispatch_dom_at(None, params.x, params.y, event_type, None)
                .default_allowed
        } else {
            true
        };
        // P1a form submit：click 命中 submit button → 提交 enclosing form（submit 事件）。
        // P1a checkbox：click 命中 checkbox → 翻转 checked + 派发 change。
        // dispatch_dom_at 已据命中点解析 event_target。
        if event_type == "click" {
            let target = self.event_target.clone();
            // P1a change-on-blur：focus 变化优先（mousedown→focus→click 近似）——旧焦点失焦
            // （blur + change 若 value 变），新焦点获焦（focus 若 text input）。
            if self.focus_target.as_deref() != Some(target.as_str()) {
                self.blur_focused()?;
                self.focus_if_text_input(&target)?;
            }
            // P1a form submit/checkbox/radio/reset：click 命中对应控件。
            if zero_engine::is_submit_button(&self.cached_html, &target) {
                let _ = self.submit_form_on_click_at(&target);
            } else if zero_engine::is_checkbox(&self.cached_html, &target) {
                let _ = self.toggle_checkbox_at(&target);
            } else if zero_engine::is_radio(&self.cached_html, &target) {
                let _ = self.toggle_radio_at(&target);
            } else if zero_engine::is_reset_button(&self.cached_html, &target) {
                let _ = self.reset_form_on_click_at(&target);
            } else if let Some(url) = zero_engine::anchor_click_target(
                &self.cached_html,
                &target,
                self.current_url.as_deref().unwrap_or("about:blank"),
            ) {
                // R3052：anchor `<a href>` click → 导航（仅 click 未 preventDefault）。target 非 submit/
                // checkbox/radio/reset 且 anchor_click_target 解析出可导航 URL → handle_navigate。
                // referrer = 当前页；navigation_epoch 递增。javascript:/#/mailto:/target=_blank 等已在 helper 过滤。
                if click_default_allowed {
                    let _ = self.handle_navigate(zero_protocol::message::NavigateParams {
                        url,
                        referrer: self.current_url.clone(),
                        navigation_epoch: self.navigation_epoch.wrapping_add(1),
                    });
                }
            } else if zero_engine::anchor_hash_target(&self.cached_html, &target).is_some() {
                // R3053：anchor `<a href="#sec">` click → location.hash 更新 + hashchange（SPA hash 路由）。
                // anchor_click_target 对 #hash 返 None（同文档锚不导航），故 hash 链接落到此分支。
                // 仅 click 未 preventDefault 时设 hash（spec：preventDefault 阻止 hash 变更）。
                if click_default_allowed {
                    let _ = self.set_hash_on_click_at(&target);
                }
            }
        }
        Ok(())
    }

    fn handle_keyboard_event(&mut self, params: KeyboardEventParams) -> Result<(), String> {
        use zero_protocol::message::KeyboardEventType;
        let event_type = match params.event_type {
            KeyboardEventType::Down => "keydown",
            KeyboardEventType::Up => "keyup",
            KeyboardEventType::Press => "keypress",
        };
        let detail = DomEventDetail {
            key: Some(params.key.clone()),
            code: Some(params.code.clone()),
            ..Default::default()
        };
        let target = self.event_target.clone();
        self.dispatch_dom_at(Some(target.clone()), 0.0, 0.0, event_type, Some(detail));
        // P1a form input：keydown 默认行为近似——可打印字符 → 注入字符；Backspace → 删末字符
        // （均更新 value + 派发 'input' 事件）；Enter → 单行 input 提交 enclosing form（submit 事件）。
        // 未尊重 keydown preventDefault（follow-up）；无 caret/selection。
        if matches!(params.event_type, KeyboardEventType::Down) {
            if params.key == "Tab" {
                // P1a Tab 焦点导航：经 FocusManager 算下一/上一可聚焦元素，blur 旧焦点 + focus 新。
                let forward = !params.shift;
                let current = self
                    .focus_target
                    .clone()
                    .or_else(|| (self.event_target != "body").then(|| self.event_target.clone()));
                if let Some(next) = zero_engine::next_focus_selector(&self.cached_html, current.as_deref(), forward)
                    && self.focus_target.as_deref() != Some(next.as_str())
                    && self.event_target != next
                {
                    let _ = self.blur_focused();
                    let _ = self.focus_via_tab(&next);
                }
            } else if is_printable_key(&params.key) {
                let _ = self.apply_text_input_at(&target, &params.key);
            } else if params.key == "Backspace" {
                let _ = self.apply_text_delete_at(&target);
            } else if params.key == "Enter" {
                // textarea Enter → 插入换行（不提交，real browser 语义；input Enter → submit）。
                // 否则 textarea 多行输入断裂（apply_text_input 的 '\n' append 经 _resolveInputEl 认 TEXTAREA）。
                if zero_engine::query_tag_from_html(&self.cached_html, &target).eq_ignore_ascii_case("textarea") {
                    let _ = self.apply_text_input_at(&target, "\n");
                } else {
                    let _ = self.submit_form_on_enter_at(&target);
                }
            }
        }
        Ok(())
    }

    fn handle_scroll_event(&mut self, params: ScrollEventParams) -> Result<(), String> {
        tracing::trace!("滚动事件: ({}, {})", params.delta_x, params.delta_y);
        Ok(())
    }

    fn dispatch_message(&mut self, msg: IpcMessage) -> Result<(), String> {
        match msg.kind {
            IpcMessageKind::Navigate(params) => self.handle_navigate(params),
            IpcMessageKind::LoadHtml(params) => self.handle_load_html(params),
            IpcMessageKind::SetViewport(params) => self.handle_set_viewport(params),
            IpcMessageKind::SetColorScheme(params) => self.handle_set_color_scheme(params),
            IpcMessageKind::SetMediaType(params) => self.handle_set_media_type(params),
            IpcMessageKind::GoBack => self.handle_go_back(),
            IpcMessageKind::GoForward => self.handle_go_forward(),
            IpcMessageKind::StopLoading => {
                tracing::info!("停止加载");
                self.pending_load = None;
                self.inflight_fetches.clear();
                Ok(())
            }
            IpcMessageKind::Reload => {
                if let Some(ref url) = self.current_url {
                    self.handle_navigate(NavigateParams {
                        url: url.clone(),
                        referrer: None,
                        navigation_epoch: self.navigation_epoch.wrapping_add(1),
                    })
                } else {
                    Ok(())
                }
            }
            IpcMessageKind::Heartbeat => self.handle_heartbeat(),
            IpcMessageKind::MouseEvent(params) => self.handle_mouse_event(params),
            IpcMessageKind::KeyboardEvent(params) => self.handle_keyboard_event(params),
            IpcMessageKind::ScrollEvent(params) => self.handle_scroll_event(params),
            IpcMessageKind::StorageOp(params) => self.handle_storage_op(params),
            IpcMessageKind::HitTestLink(params) => self.handle_hit_test_link(msg.id, params),
            IpcMessageKind::HitTestElement(params) => self.handle_hit_test_element(msg.id, params),
            IpcMessageKind::HitTestImage(params) => self.handle_hit_test_image(msg.id, params),
            IpcMessageKind::DispatchDomEvent(params) => self.handle_dispatch_dom_event(msg.id, params),
            IpcMessageKind::FetchRequest(_)
            | IpcMessageKind::FetchResponse(_)
            | IpcMessageKind::ImageDecodeRequest(_)
            | IpcMessageKind::ImageDecodeResult(_)
            | IpcMessageKind::CompositorFrame(_)
            | IpcMessageKind::CompositorFrameResult { .. }
            | IpcMessageKind::GetCompositorFrame
            | IpcMessageKind::CompositorFrameData { .. }
            | IpcMessageKind::TitleChanged(_)
            | IpcMessageKind::UrlChanged(_)
            | IpcMessageKind::LoadComplete
            | IpcMessageKind::LoadFailed(_)
            | IpcMessageKind::ViewPainted(_)
            | IpcMessageKind::HitTestLinkResult(_)
            | IpcMessageKind::HitTestElementResult(_)
            | IpcMessageKind::HitTestImageResult(_)
            | IpcMessageKind::DispatchDomEventResult(_)
            | IpcMessageKind::CrashNotification(_) => {
                tracing::warn!("渲染进程收到非预期消息类型（应从渲染进程发出）");
                Ok(())
            }
            IpcMessageKind::Ok | IpcMessageKind::Error(_) => Ok(()),
        }
    }

    fn run(&mut self) -> Result<(), String> {
        tracing::info!("渲染进程 {} 启动，等待 IPC 消息...", self.renderer_id);

        loop {
            if let Some(pending) = self.pending_load.as_ref()
                && Instant::now() >= pending.deadline
            {
                let url = pending.page_url.clone();
                if let Err(e) = self.show_error_page(&url, "页面加载超时（分阶段加载未完成）") {
                    if browser_ipc_disconnected(&e) {
                        tracing::info!("Browser IPC disconnected, renderer {} exiting", self.renderer_id);
                        return Ok(());
                    }
                    tracing::error!("加载超时处理失败: {e}");
                }
                continue;
            }

            if self.pending_load.is_some() || self.pending_script_prefetch.is_some() {
                self.drain_inflight_fetch_responses();
                if self.pending_load.is_some()
                    && let Err(e) = self.tick_pending_load()
                {
                    if browser_ipc_disconnected(&e) {
                        tracing::info!("Browser IPC disconnected, renderer {} exiting", self.renderer_id);
                        return Ok(());
                    }
                    tracing::error!("页面加载 tick 错误: {e}");
                }
                if self.pending_script_prefetch.is_some()
                    && let Err(e) = self.tick_script_prefetch()
                {
                    if browser_ipc_disconnected(&e) {
                        tracing::info!("Browser IPC disconnected, renderer {} exiting", self.renderer_id);
                        return Ok(());
                    }
                    tracing::error!("脚本预取 tick 错误: {e}");
                }
            }

            // R2949 FontFace.load()：drain JS 投递的字体加载请求（任意时刻可来，故每轮检查）。
            // fetch_get 字节 + load_font/register/set_resolver + async_resolver.resolve 解析 Promise。
            self.tick_font_face_loads();

            match self.recv_next_or_timeout(LOAD_TICK_INTERVAL) {
                Ok(Some(msg)) => {
                    if self.inflight_fetches.try_complete(&msg) {
                        continue;
                    }
                    if let Err(e) = self.dispatch_message(msg) {
                        if browser_ipc_disconnected(&e) {
                            tracing::info!("Browser IPC disconnected, renderer {} exiting", self.renderer_id);
                            return Ok(());
                        }
                        tracing::error!("消息处理错误: {e}");
                        if let Err(se) = self.send(IpcMessageKind::Error(e))
                            && browser_ipc_disconnected(&se)
                        {
                            tracing::info!("Browser IPC disconnected, renderer {} exiting", self.renderer_id);
                            return Ok(());
                        }
                    }
                }
                Ok(None) => {}
                Err(e) if browser_ipc_disconnected(&e) => {
                    tracing::info!("Browser IPC disconnected, renderer {} exiting", self.renderer_id);
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }
    }
}

/// 经 FrameModel（统一帧契约，T5）打包 IPC PaintSnapshot + 可选 Title。
fn publish_render_with_layout(
    outbound: &mut IpcOutbound,
    next_msg_id: &mut u64,
    frame: &zero_page_runtime::FrameModel,
    title: Option<String>,
    image_payloads: Vec<zero_protocol::IpcImagePayload>,
    navigation_epoch: u64,
) -> Result<(), String> {
    let paint = paint_export::paint_snapshot_from_primitives(
        frame.viewport.0,
        frame.viewport.1,
        frame.document_height,
        &frame.primitives,
        image_payloads,
        frame.hit_test.clone(),
        navigation_epoch,
    );
    let msg = IpcMessage {
        id: {
            let id = *next_msg_id;
            *next_msg_id += 1;
            id
        },
        kind: IpcMessageKind::ViewPainted(Box::new(paint)),
    };
    outbound.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))?;
    if let Some(title) = title {
        let msg = IpcMessage {
            id: {
                let id = *next_msg_id;
                *next_msg_id += 1;
                id
            },
            kind: IpcMessageKind::TitleChanged(title),
        };
        outbound.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))?;
    }
    Ok(())
}

fn ipc_fetch_error(status_code: u16, body: &[u8]) -> String {
    if status_code == 0 {
        let msg = String::from_utf8_lossy(body).trim().to_string();
        if msg.is_empty() {
            "网络请求失败（浏览器未能完成 HTTP 抓取）".to_string()
        } else {
            msg
        }
    } else {
        format!("HTTP {status_code}")
    }
}

fn ipc_fetch_get(
    outbound: &mut IpcOutbound,
    inbound_rx: &Receiver<IpcMessage>,
    next_fetch_id: &mut u64,
    deferred: &mut VecDeque<IpcMessage>,
    url: &str,
) -> Result<Vec<u8>, String> {
    let request_id = *next_fetch_id;
    *next_fetch_id += 1;
    let msg = IpcMessage {
        id: 0,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id,
            url: url.to_string(),
            method: "GET".into(),
            headers: Vec::new(),
            body: None,
        }),
    };
    outbound.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))?;

    // 当前请求不消费的消息先移到局部队列。若直接放回 `deferred`，下一轮会立刻
    // pop 出同一条消息并再次放回，永远无法等待目标 FetchResponse，导致单核自旋。
    let mut skipped = VecDeque::new();
    let result = (|| {
        loop {
            let msg = if let Some(m) = deferred.pop_front() {
                m
            } else {
                inbound_rx.recv().map_err(|e| format!("IPC 接收失败: {e}"))?
            };
            match msg.kind {
                IpcMessageKind::FetchResponse(FetchResponseParams {
                    request_id: rid,
                    status_code,
                    body,
                    ..
                }) if rid == request_id => {
                    if !(200..300).contains(&status_code) {
                        return Err(ipc_fetch_error(status_code, &body));
                    }
                    return Ok(body);
                }
                IpcMessageKind::Heartbeat => {
                    let reply = IpcMessage {
                        id: msg.id,
                        kind: IpcMessageKind::Heartbeat,
                    };
                    outbound.send(reply).map_err(|e| format!("IPC 发送失败: {e}"))?;
                }
                other => {
                    skipped.push_back(IpcMessage {
                        id: msg.id,
                        kind: other,
                    });
                }
            }
        }
    })();
    deferred.append(&mut skipped);
    result
}

fn ipc_scheme_to_engine(scheme: IpcColorScheme) -> PrefersColorSchemeValue {
    match scheme {
        IpcColorScheme::Light => PrefersColorSchemeValue::Light,
        IpcColorScheme::Dark => PrefersColorSchemeValue::Dark,
    }
}

/// IPC 媒体类型 → engine MediaType（DC-12 @media print；R1993）。
fn ipc_media_to_engine(media: IpcMediaType) -> MediaType {
    match media {
        IpcMediaType::Screen => MediaType::Screen,
        IpcMediaType::Print => MediaType::Print,
    }
}

fn load_system_fonts() -> (FontLoader, Option<u32>, HashMap<String, u32>) {
    let mut loader = FontLoader::new();
    let mut primary_font_id = None;
    let mut resolver = HashMap::new();
    #[cfg(target_os = "windows")]
    let primary_paths = ["C:\\Windows\\Fonts\\segoeui.ttf", "C:\\Windows\\Fonts\\arial.ttf"];
    #[cfg(target_os = "macos")]
    let primary_paths = [
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
    ];
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let primary_paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    ];

    if let Some((id, path)) = primary_paths.iter().find_map(|path| {
        let data = std::fs::read(path).ok()?;
        let id = loader.load_font(&data).ok()?;
        Some((id, *path))
    }) {
        tracing::info!("Renderer primary font: {path} (id={id})");
        primary_font_id = Some(id);
        resolver = loader.build_font_resolver();
    }
    (loader, primary_font_id, resolver)
}

fn parse_renderer_launch() -> (ProcessRole, u64) {
    let mut role = ProcessRole::Renderer;
    let mut instance_id = 0u64;
    for arg in std::env::args() {
        if let Some(value) = arg.strip_prefix("--type=") {
            role = match value {
                "renderer" => ProcessRole::Renderer,
                "browser" => ProcessRole::Browser,
                "network" => ProcessRole::Network,
                other => {
                    tracing::error!("未知子进程类型: {other}");
                    std::process::exit(2);
                }
            };
        }
        if let Some(id_str) = arg.strip_prefix("--instance-id=")
            && let Ok(id) = id_str.parse::<u64>()
        {
            instance_id = id;
        }
        // 兼容旧参数
        if let Some(id_str) = arg.strip_prefix("--renderer-id=")
            && let Ok(id) = id_str.parse::<u64>()
        {
            instance_id = id;
        }
    }
    (role, instance_id)
}

fn main() {
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_target(false)
        .init();

    let (role, renderer_id) = parse_renderer_launch();
    if role != ProcessRole::Renderer {
        tracing::error!("zero-renderer 必须以 --type=renderer 启动");
        std::process::exit(2);
    }
    tracing::info!("ZeroWeb 渲染进程启动 (type=renderer, instance-id={renderer_id})");

    #[cfg(target_os = "macos")]
    let result = if macos_app::is_bundled_app_executable() {
        macos_app::run_renderer(renderer_id)
    } else {
        RendererRuntime::new(renderer_id).run()
    };
    #[cfg(not(target_os = "macos"))]
    let result = RendererRuntime::new(renderer_id).run();

    if let Err(e) = result {
        tracing::error!("渲染进程错误退出: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod runtime_smoke {
    use super::*;
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    /// 共享字节缓冲（Send），捕获出站 IPC 字节用于断言。
    #[derive(Clone)]
    struct SharedBuf(Arc<Mutex<Vec<u8>>>);
    impl Write for SharedBuf {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// 把字节缓冲反帧化成 IpcMessage 列表。
    fn drain_messages(buf: &[u8]) -> Vec<IpcMessage> {
        let mut t = PipeTransport::new(std::io::Cursor::new(buf), std::io::empty());
        let mut msgs = Vec::new();
        while let Ok(m) = t.recv() {
            msgs.push(m);
        }
        msgs
    }

    #[test]
    fn read_input_value_for_change_textarea_uses_content() {
        // R2703：change-on-blur 值比对——textarea 取文本内容（非 value 属性，R2702 value↔内容），
        // input 取 value 属性。修复前 host 读 value 属性，textarea 无 value 属性 → focus_value/cur_val
        // 均 '' → textarea change-on-blur 永不触发。
        let ta = "<html><body><textarea id=\"t\">hello</textarea></body></html>";
        assert_eq!(
            read_input_value_for_change(ta, "#t"),
            "hello",
            "textarea value 取文本内容"
        );
        let inp = "<html><body><input id=\"i\" value=\"world\"></body></html>";
        assert_eq!(
            read_input_value_for_change(inp, "#i"),
            "world",
            "input value 取 value 属性"
        );
        // textarea 带 value 属性（非标准但存在）仍取内容（spec：textarea value 是内容）。
        let ta2 = "<html><body><textarea id=\"t\" value=\"ignored\">real</textarea></body></html>";
        assert_eq!(
            read_input_value_for_change(ta2, "#t"),
            "real",
            "textarea 忽略 value 属性取内容"
        );
    }

    /// IPC publish 回归门：FrameModel → ViewPainted 帧化（不启动 V8/WebView，避免 in-process 测试卡死）。
    #[test]
    fn publish_frame_emits_viewpainted_with_primitives() {
        use zero_render_foundation::color::Color;
        use zero_render_foundation::geometry::Rect;
        use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};

        let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
        let mut next_msg_id = 1_u64;
        let frame = zero_page_runtime::FrameModel {
            viewport: (800, 600),
            document_height: 400.0,
            primitives: RenderPrimitives {
                fills: vec![FillPrimitive {
                    rect: Rect::new(0.0, 0.0, 100.0, 100.0),
                    color: Color::rgb(255, 0, 0),
                }],
                ..RenderPrimitives::new()
            },
            hit_test: None,
        };
        publish_render_with_layout(
            &mut outbound,
            &mut next_msg_id,
            &frame,
            Some("smoke".into()),
            Vec::new(),
            0,
        )
        .expect("publish");

        let captured = buf.0.lock().unwrap().clone();
        let msgs = drain_messages(&captured);
        let painted = msgs
            .iter()
            .find_map(|m| match &m.kind {
                IpcMessageKind::ViewPainted(p) => Some(p.as_ref()),
                _ => None,
            })
            .expect("须产出 ViewPainted");
        assert!(!painted.fills.is_empty(), "ViewPainted 须含 fill 图元");
    }

    #[test]
    fn ipc_fetch_get_skips_deferred_messages_without_spinning() {
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()) as Box<dyn Write + Send>);
        let (tx, rx) = mpsc::channel();
        let mut next_fetch_id = 7;
        let mut deferred = VecDeque::from([IpcMessage {
            id: 11,
            kind: IpcMessageKind::SetViewport(SetViewportParams {
                width: 1024,
                height: 768,
            }),
        }]);
        tx.send(IpcMessage {
            id: 12,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 7,
                status_code: 200,
                headers: Vec::new(),
                body: b"image".to_vec(),
            }),
        })
        .unwrap();

        let body = ipc_fetch_get(
            &mut outbound,
            &rx,
            &mut next_fetch_id,
            &mut deferred,
            "https://example.test/image.png",
        )
        .expect("target fetch response");

        assert_eq!(body, b"image");
        assert_eq!(next_fetch_id, 8);
        assert!(matches!(
            deferred.pop_front().map(|m| m.kind),
            Some(IpcMessageKind::SetViewport(SetViewportParams {
                width: 1024,
                height: 768
            }))
        ));
        assert!(deferred.is_empty());
    }
}

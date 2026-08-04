//! ZeroWeb 渲染进程入口 — 独立进程处理页面渲染，经 IPC 向浏览器传递绘制快照。
//!
// Windows：GUI 子系统。renderer 由 browser 通过 stdin/stdout 管道 spawn，
// 不需要控制台；不加此项 Windows 会为子进程分配一个控制台窗口。
#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

mod error_page;
mod ipc_fetch;
mod js_worker;
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
    /// 异步分阶段加载（与 tab_worker 相同 tick 模型）。
    pending_load: Option<PendingLoad>,
    /// 页面 HTML/CSS/图片加载完成后的非阻塞脚本预取。
    pending_script_prefetch: Option<PendingScriptPrefetch>,
    /// 进行中的非阻塞 IPC fetch（request_id → Receiver 完成端）。
    inflight_fetches: InflightIpcFetches,
    /// in-process 测试无 browser 进程时，避免阻塞 IPC / 子资源永久 pending。
    stub_network: bool,
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
        // P1b S3（镜像 browser tab_worker）：注入生产 fetch handler（经 net pool 真实 HTTP GET）。
        // js_worker 早于 WebView 创建，但 fetch_text_async 自带 net pool（OnceLock），无需 WebView
        // 句柄，故可在 spawn 后立即注入。test 构建不注入（renderer runtime 单测用合成 handler）。
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
            pending_load: None,
            pending_script_prefetch: None,
            inflight_fetches: InflightIpcFetches::new(),
            stub_network: false,
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
        let changed = run_page_scripts(&mut ctx, js_enabled, fetch_from_cache);
        if changed {
            self.rerender_publish_webview()?;
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
        }
        let payloads = if allow_network_fetch {
            let mut fetch = |u: &str| self.fetch_get(u).ok();
            paint_export::fetch_image_payloads_with_cache(&html, &url, &mut image_cache, &mut fetch)
        } else {
            let mut no_fetch = |_u: &str| None;
            paint_export::fetch_image_payloads_with_cache(&html, &url, &mut image_cache, &mut no_fetch)
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
        )
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
                    match self.font_loader.load_font(&bytes) {
                        Ok(id) => {
                            // R2417/R2493：按 (weight, style) 构注册键——bold+italic →
                            // `{family}:700:italic`、bold → `{family}:700`、italic →
                            // `{family}:italic`、regular → plain `{family}`。painter
                            // resolve_font_id 按 want_bold×want_italic 组合查 + 逐级 fallback。
                            // bold/italic face **不**注册到 plain family——否则
                            // build_font_resolver 的「second face=bold」启发式会把
                            // family_map[family] 的次序面误配（顺序依赖错配，R2417）。
                            let want_bold = weight.is_some_and(|w| w >= 600);
                            let key = match (want_bold, is_italic) {
                                (true, true) => format!("{family}:700:italic"),
                                (true, false) => format!("{family}:700"),
                                (false, true) => format!("{family}:italic"),
                                (false, false) => family.clone(),
                            };
                            self.font_loader.register_family_alias(&key, id);
                            updated = true;
                        }
                        Err(e) => tracing::warn!(family = %family, err = %e, "live @font-face load failed"),
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
        let css = self.cached_css.clone();
        let font_loader = &self.font_loader;
        let font_id = self.font_id;
        let wv = self.webview.as_mut().expect("webview");
        text_metrics::with_measure_ctx_opt(font_loader, font_id, || {
            wv.load_html(&html, if css.is_empty() { None } else { Some(&css) });
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
        if event_type != "mousemove" {
            self.dispatch_dom_at(None, params.x, params.y, event_type, None);
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
            key: Some(params.key),
            code: Some(params.code),
        };
        self.dispatch_dom_at(Some(self.event_target.clone()), 0.0, 0.0, event_type, Some(detail));
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
                deferred.push_back(IpcMessage {
                    id: msg.id,
                    kind: other,
                });
            }
        }
    }
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

    let mut runtime = RendererRuntime::new(renderer_id);

    if let Err(e) = runtime.run() {
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
}

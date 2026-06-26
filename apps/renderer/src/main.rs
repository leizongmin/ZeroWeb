//! ZeroWeb 渲染进程入口 — 独立进程处理页面渲染，经 IPC 向浏览器传递绘制快照。

mod async_load;
mod error_page;
mod js_worker;
mod page_scripts;
mod paint_export;
mod text_metrics;

use zero_page_runtime::PageLoadHost;

use crate::js_worker::RendererJsWorker;
use crate::page_scripts::{DomDispatchResult, PageScriptContext, dispatch_dom_event, rerender, run_page_scripts};

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

use std::io;
use zero_engine::{PageScript, extract_page_scripts, resolve_document_url};
use zero_script_sandbox::extract_module_import_specifiers;

use zero_engine::{
    DomEventDetail, PrefersColorSchemeValue, RenderPipeline, RenderResult, selector_from_element_hit,
    set_char_measure_fn,
};
use zero_protocol::IpcChannel;
use zero_protocol::ProcessRole;
use zero_protocol::message::{
    DispatchDomEventParams, DispatchDomEventResultParams, FetchParams, FetchResponseParams, HitTestElementResultParams,
    HitTestLinkParams, HitTestLinkResultParams, IpcColorScheme, IpcMessage, IpcMessageKind, KeyboardEventParams,
    LoadHtmlParams, MouseEventParams, NavigateParams, ScrollEventParams, SetColorSchemeParams, SetViewportParams,
    StorageOpParams,
};
use zero_protocol::transport::PipeTransport;

/// 渲染进程 → 浏览器 IPC 发送端（stdout）。
type IpcOutbound = PipeTransport<io::Empty, Box<dyn io::Write + Send>>;

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

/// 渲染进程运行时状态。
struct RendererRuntime {
    /// 向浏览器写入 IPC（stdout）。
    outbound: IpcOutbound,
    /// 浏览器 → 渲染进程消息（stdin 读线程填充）。
    inbound_rx: Receiver<IpcMessage>,
    /// 持有 IPC 读线程 JoinHandle（仅保活，不被读；drop 即分离线程）。
    #[allow(dead_code)]
    inbound_thread: Option<JoinHandle<()>>,
    /// 渲染管线。
    pipeline: RenderPipeline,
    /// 页面运行时（B3 迁移中：将逐步接管 pipeline / page_scripts / text_metrics；当前已构造、渲染尚未路由）。
    #[allow(dead_code)]
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
        let mut pipeline = RenderPipeline::new(1280.0, 800.0);
        let (font_loader, font_id) = load_system_fonts(&mut pipeline);
        set_char_measure_fn(text_metrics::measure_char);
        let js_worker = RendererJsWorker::spawn(renderer_id);
        // B3：renderer 内部持有 WebView（external_script 委派 js_worker，避免双 V8），
        // 逐步接管 pipeline/page_scripts/text_metrics（见 doc §11）。当前已构造、尚未路由渲染。
        let webview = zero_webview::WebView::new(zero_webview::WebViewConfig {
            width: 1280,
            height: 800,
            external_script: Some(js_worker.executor()),
            ..Default::default()
        });
        Self {
            outbound,
            inbound_rx,
            inbound_thread: None,
            pipeline,
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

    fn recv_blocking(&mut self) -> Result<IpcMessage, String> {
        self.inbound_rx.recv().map_err(|e| format!("IPC 接收失败: {e}"))
    }

    fn after_page_html_loaded(&mut self, html: String, css: String) -> Result<(), String> {
        self.cached_html = html;
        self.cached_css = css;
        let js_enabled = self.javascript_enabled;
        let fetch_cache = self.build_script_fetch_cache();
        let font_loader = &self.font_loader;
        let font_id = self.font_id;
        let current_url = self.current_url.as_deref().unwrap_or("about:blank");
        let mut ctx = PageScriptContext {
            pipeline: &mut self.pipeline,
            html: &mut self.cached_html,
            css: &self.cached_css,
            url: current_url,
            js_worker: &self.js_worker,
        };
        let fetch_from_cache = |url: &str| {
            fetch_cache
                .get(url)
                .cloned()
                .ok_or_else(|| format!("script fetch failed: {url}"))
        };
        if run_page_scripts(&mut ctx, js_enabled, fetch_from_cache) {
            let result = text_metrics::with_measure_ctx_opt(font_loader, font_id, || rerender(&mut ctx));
            self.publish_render(&result, None)?;
        }
        Ok(())
    }

    /// 经浏览器进程 IPC 预抓取页面脚本与子模块（渲染进程不直连网络）。
    fn build_script_fetch_cache(&mut self) -> HashMap<String, String> {
        let base = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        if page_scripts::should_skip_scripts(&base) || self.cached_html.is_empty() {
            return HashMap::new();
        }

        let mut cache = HashMap::new();
        let mut pending = VecDeque::new();
        let mut seen = HashSet::new();

        for script in extract_page_scripts(&self.cached_html) {
            match script {
                PageScript::External(src) | PageScript::ExternalModule(src) => {
                    pending.push_back(resolve_document_url(&base, &src));
                }
                _ => {}
            }
        }

        while let Some(url) = pending.pop_front() {
            if seen.contains(&url) {
                continue;
            }
            seen.insert(url.clone());

            let body = match self.fetch_get(&url) {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("script prefetch {url}: {e}");
                    continue;
                }
            };
            let text = String::from_utf8_lossy(&body).into_owned();
            for spec in extract_module_import_specifiers(&text) {
                let dep = resolve_document_url(&url, &spec);
                if !seen.contains(&dep) {
                    pending.push_back(dep);
                }
            }
            cache.insert(url, text);
        }

        cache
    }

    fn publish_render(&mut self, result: &RenderResult, title: Option<String>) -> Result<(), String> {
        let html = self.cached_html.clone();
        let url = self.current_url.clone().unwrap_or_else(|| "about:blank".into());
        let payloads = paint_export::fetch_image_payloads_with_fetch(&html, &url, &mut |u| self.fetch_get(u).ok());
        publish_render_result(
            &mut self.outbound,
            &mut self.next_msg_id,
            &self.pipeline,
            result,
            title,
            payloads,
        )
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
            self.pipeline
                .hit_test_element(x, y)
                .map(|hit| selector_from_element_hit(&hit))
        });
        if let Some(sel) = selector {
            self.event_target = sel.clone();
            let js_enabled = self.javascript_enabled;
            let font_loader = &self.font_loader;
            let font_id = self.font_id;
            let current_url = self.current_url.as_deref().unwrap_or("about:blank");
            let mut ctx = PageScriptContext {
                pipeline: &mut self.pipeline,
                html: &mut self.cached_html,
                css: &self.cached_css,
                url: current_url,
                js_worker: &self.js_worker,
            };
            let result = dispatch_dom_event(&mut ctx, js_enabled, &sel, event_type, detail.as_ref());
            if result.html_changed {
                let render = text_metrics::with_measure_ctx_opt(font_loader, font_id, || rerender(&mut ctx));
                let _ = self.publish_render(&render, None);
            }
            result
        } else {
            DomDispatchResult {
                default_allowed: true,
                html_changed: false,
            }
        }
    }

    fn recv_next(&mut self) -> Result<IpcMessage, String> {
        if let Some(msg) = self.deferred_inbound.pop_front() {
            return Ok(msg);
        }
        self.recv_blocking()
    }

    /// 经浏览器 IPC 代理 GET 请求。
    fn fetch_get(&mut self, url: &str) -> Result<Vec<u8>, String> {
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

        let html_for_images = html.clone();
        let url_for_images = page_url.clone();

        {
            let font_loader = &self.font_loader;
            let font_id = self.font_id;
            let outbound = &mut self.outbound;
            let inbound_rx = &self.inbound_rx;
            let next_msg_id = &mut self.next_msg_id;
            let next_fetch_id = &mut self.next_fetch_id;
            let deferred = &mut self.deferred_inbound;
            let pipeline = &mut self.pipeline;
            let viewport = (pipeline.viewport_width() as u32, pipeline.viewport_height() as u32);
            let mut host = IpcLoadBridge {
                outbound,
                inbound_rx,
                next_msg_id,
                next_fetch_id,
                deferred,
                viewport,
            };

            text_metrics::with_measure_ctx_opt(font_loader, font_id, || {
                async_load::run_page_load(pipeline, &page_url, &html, &mut host)
            })?;
        }

        let font_loader = &self.font_loader;
        let font_id = self.font_id;
        let pipeline = &mut self.pipeline;
        if let Some(result) =
            text_metrics::with_measure_ctx_opt(font_loader, font_id, || pipeline.repaint_cached_viewport(""))
        {
            let payloads = paint_export::fetch_image_payloads_with_fetch(&html_for_images, &url_for_images, &mut |u| {
                self.fetch_get(u).ok()
            });
            publish_render_result(
                &mut self.outbound,
                &mut self.next_msg_id,
                &self.pipeline,
                &result,
                None,
                payloads,
            )?;
        }

        if !page_scripts::should_skip_scripts(&page_url) {
            self.after_page_html_loaded(html_for_images, self.cached_css.clone())?;
        }

        if send_complete {
            self.send(IpcMessageKind::LoadComplete)?;
            tracing::info!("页面渲染完成: {page_url}");
        }
        Ok(())
    }

    fn show_error_page(&mut self, page_url: &str, error: &str) -> Result<(), String> {
        tracing::error!("页面加载失败 ({page_url}): {error}");
        self.send(IpcMessageKind::LoadFailed(error.to_string()))?;
        let html = error_page::generate_error_page(page_url, error);
        self.run_staged_load(format!("error://{page_url}"), html, false, false)?;
        self.send(IpcMessageKind::TitleChanged("加载失败".to_string()))
    }

    fn handle_navigate(&mut self, params: NavigateParams) -> Result<(), String> {
        tracing::info!("导航到: {}", params.url);
        match self.fetch_get(&params.url) {
            Ok(body) => {
                let html = String::from_utf8_lossy(&body).into_owned();
                self.run_staged_load(params.url, html, true, true)
            }
            Err(e) => self.show_error_page(&params.url, &e),
        }
    }

    fn handle_load_html(&mut self, params: LoadHtmlParams) -> Result<(), String> {
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
        let pipeline = &mut self.pipeline;
        let Some(result) =
            text_metrics::with_measure_ctx_opt(font_loader, font_id, || pipeline.repaint_cached_viewport(""))
        else {
            return Ok(());
        };
        publish_render_result(
            &mut self.outbound,
            &mut self.next_msg_id,
            &self.pipeline,
            &result,
            None,
            Vec::new(),
        )
    }

    fn handle_set_viewport(&mut self, params: SetViewportParams) -> Result<(), String> {
        self.pipeline.set_viewport(params.width as f32, params.height as f32);
        self.try_republish_cached()
    }

    fn handle_set_color_scheme(&mut self, params: SetColorSchemeParams) -> Result<(), String> {
        self.pipeline
            .set_prefers_color_scheme(ipc_scheme_to_engine(params.scheme));
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
        let href = self.pipeline.hit_test_link(params.x, params.y);
        tracing::trace!("HitTestLink({msg_id}) -> {:?}", href.as_deref());
        self.send_with_id(
            msg_id,
            IpcMessageKind::HitTestLinkResult(HitTestLinkResultParams { href }),
        )
    }

    fn handle_hit_test_element(&mut self, msg_id: u64, params: HitTestLinkParams) -> Result<(), String> {
        let result = self
            .pipeline
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
            IpcMessageKind::GoBack => self.handle_go_back(),
            IpcMessageKind::GoForward => self.handle_go_forward(),
            IpcMessageKind::StopLoading => {
                tracing::info!("停止加载");
                Ok(())
            }
            IpcMessageKind::Reload => {
                if let Some(ref url) = self.current_url {
                    self.handle_navigate(NavigateParams {
                        url: url.clone(),
                        referrer: None,
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
            let msg = self.recv_next()?;
            if let Err(e) = self.dispatch_message(msg) {
                tracing::error!("消息处理错误: {e}");
                let _ = self.send(IpcMessageKind::Error(e.clone()));
            }
        }
    }
}

struct IpcLoadBridge<'a> {
    outbound: &'a mut IpcOutbound,
    inbound_rx: &'a Receiver<IpcMessage>,
    next_msg_id: &'a mut u64,
    next_fetch_id: &'a mut u64,
    deferred: &'a mut VecDeque<IpcMessage>,
    viewport: (u32, u32),
}

impl PageLoadHost for IpcLoadBridge<'_> {
    fn fetch_bytes(&mut self, url: &str) -> Result<Vec<u8>, String> {
        ipc_fetch_get(self.outbound, self.inbound_rx, self.next_fetch_id, self.deferred, url)
    }

    fn publish(&mut self, result: &RenderResult, title: Option<String>, _is_final: bool) -> Result<(), String> {
        let doc_h = document_height_from_layout(&result.layout);
        publish_render_with_layout(
            self.outbound,
            self.next_msg_id,
            self.viewport.0,
            self.viewport.1,
            doc_h,
            &result.primitives,
            title,
            Vec::new(),
            None,
        )
    }
}

fn document_height_from_layout(layout: &zero_layout_engine::LayoutResult) -> f32 {
    layout.root.y + layout.root.height
}

/// 注意：参数偏多，T5 FrameModel 统一后收敛为结构体入参。
#[allow(clippy::too_many_arguments)]
fn publish_render_with_layout(
    outbound: &mut IpcOutbound,
    next_msg_id: &mut u64,
    viewport_width: u32,
    viewport_height: u32,
    document_height: f32,
    primitives: &zero_render_foundation::primitive::RenderPrimitives,
    title: Option<String>,
    image_payloads: Vec<zero_protocol::IpcImagePayload>,
    hit_test: Option<zero_engine::HitTestCache>,
) -> Result<(), String> {
    let paint = paint_export::paint_snapshot_from_primitives(
        viewport_width,
        viewport_height,
        document_height,
        primitives,
        image_payloads,
        hit_test,
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
                    return Err(format!("HTTP {status_code}"));
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

fn publish_render_result(
    outbound: &mut IpcOutbound,
    next_msg_id: &mut u64,
    pipeline: &RenderPipeline,
    result: &RenderResult,
    title: Option<String>,
    image_payloads: Vec<zero_protocol::IpcImagePayload>,
) -> Result<(), String> {
    let doc_h = pipeline.document_height().unwrap_or(pipeline.viewport_height());
    publish_render_with_layout(
        outbound,
        next_msg_id,
        pipeline.viewport_width() as u32,
        pipeline.viewport_height() as u32,
        doc_h,
        &result.primitives,
        title,
        image_payloads,
        None,
    )
}

fn ipc_scheme_to_engine(scheme: IpcColorScheme) -> PrefersColorSchemeValue {
    match scheme {
        IpcColorScheme::Light => PrefersColorSchemeValue::Light,
        IpcColorScheme::Dark => PrefersColorSchemeValue::Dark,
    }
}

fn load_system_fonts(pipeline: &mut RenderPipeline) -> (FontLoader, Option<u32>) {
    let mut loader = FontLoader::new();
    let mut primary_font_id = None;
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
        pipeline.set_font_resolver(loader.build_font_resolver());
    }
    (loader, primary_font_id)
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

    /// in-process renderer load+publish 回归门：喂 LoadHtml → 须产出含图元的 ViewPainted。
    /// B3 cutover 后此测试仍过 = load+publish wiring 不坏。renderer 是 bin，靠 with_io 注入 transport 对测。
    #[test]
    fn renderer_load_html_publishes_viewpainted() {
        let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
        let (_in_tx, in_rx) = mpsc::channel();
        let mut rt = RendererRuntime::with_io(1, Box::new(buf.clone()), in_rx);
        let html = r#"<html><body><div style="width:200px;height:100px;background:red">Box</div></body></html>"#;
        rt.handle_load_html(LoadHtmlParams {
            html: html.into(),
            css: None,
            url: Some("zero://smoke".into()),
        })
        .expect("load ok");
        let captured = buf.0.lock().unwrap().clone();
        let msgs = drain_messages(&captured);
        let painted = msgs
            .iter()
            .find_map(|m| match &m.kind {
                IpcMessageKind::ViewPainted(p) => Some(p.as_ref()),
                _ => None,
            })
            .expect("须产出 ViewPainted");
        assert!(
            !(painted.fills.is_empty() && painted.rounded_rects.is_empty() && painted.images.is_empty()),
            "ViewPainted 须含可见图元（fills/rounded_rects/images）"
        );
    }
}

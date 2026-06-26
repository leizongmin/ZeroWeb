//! ZeroWeb 渲染进程入口 — 独立进程处理页面渲染，经 IPC 向浏览器传递绘制快照。

mod async_load;
mod error_page;
mod paint_export;

use async_load::PageLoadHost;

use std::collections::VecDeque;
use std::io;

use zero_engine::{PrefersColorSchemeValue, RenderPipeline, RenderResult};
use zero_protocol::IpcChannel;
use zero_protocol::message::{
    FetchParams, FetchResponseParams, HitTestLinkParams, HitTestLinkResultParams, IpcColorScheme, IpcMessage,
    IpcMessageKind, KeyboardEventParams, LoadHtmlParams, MouseEventParams, NavigateParams, ScrollEventParams,
    SetColorSchemeParams, SetViewportParams, StorageOpParams,
};
use zero_protocol::transport::PipeTransport;
use zero_render_foundation::font::loader::FontLoader;

/// 渲染进程运行时状态。
struct RendererRuntime {
    /// IPC 通道。
    channel: PipeTransport<io::Stdin, io::Stdout>,
    /// 渲染管线。
    pipeline: RenderPipeline,
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
}

impl RendererRuntime {
    /// 创建新的渲染进程运行时。
    fn new(renderer_id: u64) -> Self {
        let channel = PipeTransport::new(io::stdin(), io::stdout());
        let mut pipeline = RenderPipeline::new(1280.0, 800.0);
        load_system_fonts(&mut pipeline);
        Self {
            channel,
            pipeline,
            current_url: None,
            next_msg_id: 1,
            next_fetch_id: 1,
            renderer_id,
            history: Vec::new(),
            history_index: 0,
            deferred_inbound: VecDeque::new(),
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
        self.channel.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))
    }

    fn send_with_id(&mut self, id: u64, kind: IpcMessageKind) -> Result<(), String> {
        let msg = IpcMessage { id, kind };
        self.channel.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))
    }

    fn recv_blocking(&mut self) -> Result<IpcMessage, String> {
        self.channel.recv().map_err(|e| format!("IPC 接收失败: {e}"))
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
            &mut self.channel,
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
            let channel = &mut self.channel;
            let next_msg_id = &mut self.next_msg_id;
            let next_fetch_id = &mut self.next_fetch_id;
            let deferred = &mut self.deferred_inbound;
            let pipeline = &mut self.pipeline;
            let viewport = (pipeline.viewport_width() as u32, pipeline.viewport_height() as u32);
            let mut host = IpcLoadBridge {
                channel,
                next_msg_id,
                next_fetch_id,
                deferred,
                viewport,
            };

            async_load::run_page_load(pipeline, &page_url, &html, &mut host)?;
        }

        if let Some(result) = self.pipeline.repaint_cached_viewport("") {
            let payloads = paint_export::fetch_image_payloads_with_fetch(&html_for_images, &url_for_images, &mut |u| {
                self.fetch_get(u).ok()
            });
            publish_render_result(
                &mut self.channel,
                &mut self.next_msg_id,
                &self.pipeline,
                &result,
                None,
                payloads,
            )?;
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
        let css = params.css.as_deref().unwrap_or("");
        let mut html = params.html;
        if !css.is_empty() {
            html.push_str("\n<style>\n");
            html.push_str(css);
            html.push_str("\n</style>\n");
        }
        self.run_staged_load(page_url, html, true, true)
    }

    fn try_republish_cached(&mut self) -> Result<(), String> {
        let Some(result) = self.pipeline.repaint_cached_viewport("") else {
            return Ok(());
        };
        publish_render_result(
            &mut self.channel,
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

    fn handle_mouse_event(&mut self, params: MouseEventParams) -> Result<(), String> {
        tracing::trace!("鼠标事件: ({}, {}) {:?}", params.x, params.y, params.event_type);
        Ok(())
    }

    fn handle_keyboard_event(&mut self, params: KeyboardEventParams) -> Result<(), String> {
        tracing::trace!("键盘事件: {} {:?}", params.key, params.event_type);
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
            IpcMessageKind::FetchRequest(_)
            | IpcMessageKind::FetchResponse(_)
            | IpcMessageKind::TitleChanged(_)
            | IpcMessageKind::UrlChanged(_)
            | IpcMessageKind::LoadComplete
            | IpcMessageKind::LoadFailed(_)
            | IpcMessageKind::ViewPainted(_)
            | IpcMessageKind::HitTestLinkResult(_)
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
    channel: &'a mut PipeTransport<io::Stdin, io::Stdout>,
    next_msg_id: &'a mut u64,
    next_fetch_id: &'a mut u64,
    deferred: &'a mut VecDeque<IpcMessage>,
    viewport: (u32, u32),
}

impl PageLoadHost for IpcLoadBridge<'_> {
    fn fetch_bytes(&mut self, url: &str) -> Result<Vec<u8>, String> {
        ipc_fetch_get(self.channel, self.next_fetch_id, self.deferred, url)
    }

    fn publish(&mut self, result: &RenderResult, title: Option<String>, _is_final: bool) -> Result<(), String> {
        let doc_h = document_height_from_layout(&result.layout);
        publish_render_with_layout(
            self.channel,
            self.next_msg_id,
            self.viewport.0,
            self.viewport.1,
            doc_h,
            result,
            title,
            Vec::new(),
        )
    }
}

fn document_height_from_layout(layout: &zero_layout_engine::LayoutResult) -> f32 {
    layout.root.y + layout.root.height
}

fn publish_render_with_layout(
    channel: &mut PipeTransport<io::Stdin, io::Stdout>,
    next_msg_id: &mut u64,
    viewport_width: u32,
    viewport_height: u32,
    document_height: f32,
    result: &RenderResult,
    title: Option<String>,
    image_payloads: Vec<zero_protocol::IpcImagePayload>,
) -> Result<(), String> {
    let paint = paint_export::paint_snapshot_from_primitives(
        viewport_width,
        viewport_height,
        document_height,
        &result.primitives,
        image_payloads,
    );
    let msg = IpcMessage {
        id: {
            let id = *next_msg_id;
            *next_msg_id += 1;
            id
        },
        kind: IpcMessageKind::ViewPainted(paint),
    };
    channel.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))?;
    if let Some(title) = title {
        let msg = IpcMessage {
            id: {
                let id = *next_msg_id;
                *next_msg_id += 1;
                id
            },
            kind: IpcMessageKind::TitleChanged(title),
        };
        channel.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))?;
    }
    Ok(())
}

fn ipc_fetch_get(
    channel: &mut PipeTransport<io::Stdin, io::Stdout>,
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
    channel.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))?;

    loop {
        let msg = if let Some(m) = deferred.pop_front() {
            m
        } else {
            channel.recv().map_err(|e| format!("IPC 接收失败: {e}"))?
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
                channel.send(reply).map_err(|e| format!("IPC 发送失败: {e}"))?;
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
    channel: &mut PipeTransport<io::Stdin, io::Stdout>,
    next_msg_id: &mut u64,
    pipeline: &RenderPipeline,
    result: &RenderResult,
    title: Option<String>,
    image_payloads: Vec<zero_protocol::IpcImagePayload>,
) -> Result<(), String> {
    let doc_h = pipeline.document_height().unwrap_or(pipeline.viewport_height());
    publish_render_with_layout(
        channel,
        next_msg_id,
        pipeline.viewport_width() as u32,
        pipeline.viewport_height() as u32,
        doc_h,
        result,
        title,
        image_payloads,
    )
}

fn ipc_scheme_to_engine(scheme: IpcColorScheme) -> PrefersColorSchemeValue {
    match scheme {
        IpcColorScheme::Light => PrefersColorSchemeValue::Light,
        IpcColorScheme::Dark => PrefersColorSchemeValue::Dark,
    }
}

fn load_system_fonts(pipeline: &mut RenderPipeline) {
    let mut loader = FontLoader::new();
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
        pipeline.set_font_resolver(loader.build_font_resolver());
    }
}

fn parse_renderer_id() -> u64 {
    for arg in std::env::args() {
        if let Some(id_str) = arg.strip_prefix("--renderer-id=")
            && let Ok(id) = id_str.parse::<u64>()
        {
            return id;
        }
    }
    0
}

fn main() {
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_target(false)
        .init();

    let renderer_id = parse_renderer_id();
    tracing::info!("ZeroWeb 渲染进程启动 (id={renderer_id})");

    let mut runtime = RendererRuntime::new(renderer_id);

    if let Err(e) = runtime.run() {
        tracing::error!("渲染进程错误退出: {e}");
        std::process::exit(1);
    }
}

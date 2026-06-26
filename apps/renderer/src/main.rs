//! ZeroWeb 渲染进程入口 — 独立进程处理页面渲染，经 IPC 向浏览器传递绘制快照。

mod paint_export;

use std::io;
use std::process;

use zero_engine::RenderPipeline;
use zero_protocol::IpcChannel;
use zero_protocol::message::{
    FetchParams, HitTestLinkParams, HitTestLinkResultParams, IpcMessage, IpcMessageKind, LoadHtmlParams,
    NavigateParams, StorageOpParams,
};
use zero_protocol::transport::PipeTransport;

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
    /// 渲染进程 ID。
    renderer_id: u64,
}

impl RendererRuntime {
    /// 创建新的渲染进程运行时。
    fn new(renderer_id: u64) -> Self {
        let channel = PipeTransport::new(io::stdin(), io::stdout());
        let pipeline = RenderPipeline::new(1280.0, 800.0);
        Self {
            channel,
            pipeline,
            current_url: None,
            next_msg_id: 1,
            renderer_id,
        }
    }

    /// 分配下一个消息 ID。
    fn alloc_msg_id(&mut self) -> u64 {
        let id = self.next_msg_id;
        self.next_msg_id += 1;
        id
    }

    /// 发送 IPC 消息到浏览器主进程。
    fn send(&mut self, kind: IpcMessageKind) -> Result<(), String> {
        let msg = IpcMessage {
            id: self.alloc_msg_id(),
            kind,
        };
        self.channel.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))
    }

    /// 发送带指定 ID 的 IPC 响应。
    fn send_with_id(&mut self, id: u64, kind: IpcMessageKind) -> Result<(), String> {
        let msg = IpcMessage { id, kind };
        self.channel.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))
    }

    /// 接收来自浏览器主进程的 IPC 消息。
    fn recv(&mut self) -> Result<IpcMessage, String> {
        self.channel.recv().map_err(|e| format!("IPC 接收失败: {e}"))
    }

    /// 处理导航命令。
    fn handle_navigate(&mut self, params: NavigateParams) -> Result<(), String> {
        tracing::info!("导航到: {}", params.url);

        self.send(IpcMessageKind::UrlChanged(params.url.clone()))?;
        self.current_url = Some(params.url.clone());

        let client = zero_net::client::HttpClient::new();
        match client.get(&params.url) {
            Ok(response) => {
                let html = String::from_utf8_lossy(&response.body).into_owned();
                self.publish_html_render(&html, "", &params.url)
            }
            Err(e) => {
                tracing::error!("页面加载失败: {e}");
                self.send(IpcMessageKind::LoadFailed(format!("网络请求失败: {e}")))
            }
        }
    }

    /// 处理内联 HTML 加载。
    fn handle_load_html(&mut self, params: LoadHtmlParams) -> Result<(), String> {
        let page_url = params.url.clone().unwrap_or_else(|| "about:blank".to_string());
        tracing::info!("加载内联 HTML: {page_url}");
        self.send(IpcMessageKind::UrlChanged(page_url.clone()))?;
        self.current_url = Some(page_url.clone());
        let css = params.css.as_deref().unwrap_or("");
        self.publish_html_render(&params.html, css, &page_url)
    }

    /// 渲染 HTML 并推送 ViewPainted / TitleChanged / LoadComplete。
    fn publish_html_render(&mut self, html: &str, css: &str, page_url: &str) -> Result<(), String> {
        let title = extract_title(html);
        let result = self.pipeline.render_html(html, css);
        let doc_h = self
            .pipeline
            .document_height()
            .unwrap_or(self.pipeline.viewport_height());
        let image_payloads = paint_export::fetch_image_payloads(html, page_url);
        let paint = paint_export::paint_snapshot_from_primitives(
            self.pipeline.viewport_width() as u32,
            self.pipeline.viewport_height() as u32,
            doc_h,
            &result.primitives,
            image_payloads,
        );
        self.send(IpcMessageKind::ViewPainted(paint))?;
        if let Some(title) = title {
            self.send(IpcMessageKind::TitleChanged(title))?;
        }
        self.send(IpcMessageKind::LoadComplete)?;
        tracing::info!("页面渲染完成: {page_url}");
        Ok(())
    }

    /// 处理后退命令。
    fn handle_go_back(&mut self) -> Result<(), String> {
        tracing::info!("后退导航");
        self.send(IpcMessageKind::Ok)
    }

    /// 处理前进命令。
    fn handle_go_forward(&mut self) -> Result<(), String> {
        tracing::info!("前进导航");
        self.send(IpcMessageKind::Ok)
    }

    /// 处理网络请求（渲染进程→浏览器进程转发）。
    fn handle_fetch_request(&mut self, _params: FetchParams) -> Result<(), String> {
        // 在真实实现中，渲染进程通过 IPC 转发到浏览器进程处理网络请求
        // 这里简化处理：直接在渲染进程内使用 net crate
        Ok(())
    }

    /// 处理存储操作（渲染进程→浏览器进程转发）。
    fn handle_storage_op(&mut self, _params: StorageOpParams) -> Result<(), String> {
        // 在真实实现中，渲染进程通过 IPC 转发到浏览器进程处理存储操作
        // 这里简化处理：直接在渲染进程内使用 storage crate
        Ok(())
    }

    /// 处理心跳。
    fn handle_heartbeat(&mut self) -> Result<(), String> {
        self.send(IpcMessageKind::Heartbeat)
    }

    /// 处理链接命中测试。
    fn handle_hit_test_link(&mut self, msg_id: u64, params: HitTestLinkParams) -> Result<(), String> {
        let href = self.pipeline.hit_test_link(params.x, params.y);
        tracing::trace!("HitTestLink({msg_id}) -> {:?}", href.as_deref());
        self.send_with_id(
            msg_id,
            IpcMessageKind::HitTestLinkResult(HitTestLinkResultParams { href }),
        )
    }

    /// 主消息循环。
    fn run(&mut self) -> Result<(), String> {
        tracing::info!("渲染进程 {} 启动，等待 IPC 消息...", self.renderer_id);

        loop {
            let msg = self.recv()?;

            let result = match msg.kind {
                IpcMessageKind::Navigate(params) => self.handle_navigate(params),
                IpcMessageKind::LoadHtml(params) => self.handle_load_html(params),
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
                IpcMessageKind::MouseEvent(params) => {
                    tracing::trace!("鼠标事件: ({}, {}) {:?}", params.x, params.y, params.event_type);
                    Ok(())
                }
                IpcMessageKind::KeyboardEvent(params) => {
                    tracing::trace!("键盘事件: {} {:?}", params.key, params.event_type);
                    Ok(())
                }
                IpcMessageKind::ScrollEvent(params) => {
                    tracing::trace!("滚动事件: ({}, {})", params.delta_x, params.delta_y);
                    Ok(())
                }
                IpcMessageKind::FetchRequest(params) => self.handle_fetch_request(params),
                IpcMessageKind::StorageOp(params) => self.handle_storage_op(params),
                IpcMessageKind::HitTestLink(params) => self.handle_hit_test_link(msg.id, params),
                IpcMessageKind::FetchResponse(_)
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
            };

            if let Err(e) = result {
                tracing::error!("消息处理错误: {e}");
                // 通知浏览器主进程
                let _ = self.send(IpcMessageKind::Error(e.clone()));
                // 非致命错误，继续消息循环
            }
        }
    }
}

/// 从 HTML 中提取 `<title>` 内容。
fn extract_title(html: &str) -> Option<String> {
    let start = html.find("<title>")? + "<title>".len();
    let end = html.find("</title>")?;
    if end > start {
        Some(html[start..end].trim().to_string())
    } else {
        None
    }
}

/// 解析命令行参数中的 `--renderer-id`。
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
    // 初始化日志（输出到 stderr，不影响 IPC 管道）
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_target(false)
        .init();

    let renderer_id = parse_renderer_id();
    tracing::info!("ZeroWeb 渲染进程启动 (id={renderer_id})");

    let mut runtime = RendererRuntime::new(renderer_id);

    match runtime.run() {
        Ok(()) => {
            tracing::info!("渲染进程正常退出");
        }
        Err(e) => {
            tracing::error!("渲染进程错误退出: {e}");
            process::exit(1);
        }
    }
}

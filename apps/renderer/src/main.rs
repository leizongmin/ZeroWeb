//! ZeroWeb 渲染进程入口。
//!
//! 独立进程处理页面渲染和 JS 执行。
//! 通过 stdin/stdout 管道与浏览器主进程进行 IPC 通信。
//!
//! ## 启动方式
//!
//! ```sh
//! zero-renderer --renderer-id=1
//! ```
//!
//! 浏览器主进程通过 `ProcessManager` 自动启动和管理此进程。

use std::io;
use std::process;

use zero_engine::RenderPipeline;
use zero_protocol::IpcChannel;
use zero_protocol::message::{FetchParams, IpcMessage, IpcMessageKind, NavigateParams, StorageOpParams};
use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcColor, IpcFill, IpcGlyph, IpcRect, PaintSnapshotParams};
use zero_render_foundation::primitive::RenderPrimitives;

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

    /// 接收来自浏览器主进程的 IPC 消息。
    fn recv(&mut self) -> Result<IpcMessage, String> {
        self.channel.recv().map_err(|e| format!("IPC 接收失败: {e}"))
    }

    /// 处理导航命令。
    fn handle_navigate(&mut self, params: NavigateParams) -> Result<(), String> {
        tracing::info!("导航到: {}", params.url);

        // 报告 URL 变更
        self.send(IpcMessageKind::UrlChanged(params.url.clone()))?;
        self.current_url = Some(params.url.clone());

        // 使用 net crate 获取页面内容
        let client = zero_net::client::HttpClient::new();
        match client.get(&params.url) {
            Ok(response) => {
                let html = String::from_utf8_lossy(&response.body).into_owned();
                let title = extract_title(&html);

                // 渲染页面
                let result = self.pipeline.render_html(&html, "");
                let doc_h = self
                    .pipeline
                    .document_height()
                    .unwrap_or(self.pipeline.viewport_height());
                let paint = paint_snapshot_from_primitives(
                    self.pipeline.viewport_width() as u32,
                    self.pipeline.viewport_height() as u32,
                    doc_h,
                    &result.primitives,
                );
                self.send(IpcMessageKind::ViewPainted(paint))?;

                // 报告标题变更
                if let Some(title) = title {
                    self.send(IpcMessageKind::TitleChanged(title))?;
                }

                // 报告加载完成
                self.send(IpcMessageKind::LoadComplete)?;
                tracing::info!("页面加载完成: {}", params.url);
                Ok(())
            }
            Err(e) => {
                tracing::error!("页面加载失败: {e}");
                self.send(IpcMessageKind::LoadFailed(format!("网络请求失败: {e}")))?;
                Ok(())
            }
        }
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

    /// 主消息循环。
    fn run(&mut self) -> Result<(), String> {
        tracing::info!("渲染进程 {} 启动，等待 IPC 消息...", self.renderer_id);

        loop {
            let msg = self.recv()?;

            let result = match msg.kind {
                IpcMessageKind::Navigate(params) => self.handle_navigate(params),
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
                IpcMessageKind::FetchResponse(_)
                | IpcMessageKind::TitleChanged(_)
                | IpcMessageKind::UrlChanged(_)
                | IpcMessageKind::LoadComplete
                | IpcMessageKind::LoadFailed(_)
                | IpcMessageKind::ViewPainted(_)
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

fn paint_snapshot_from_primitives(
    viewport_width: u32,
    viewport_height: u32,
    document_height: f32,
    primitives: &RenderPrimitives,
) -> PaintSnapshotParams {
    PaintSnapshotParams {
        viewport_width,
        viewport_height,
        document_height,
        fills: primitives
            .fills
            .iter()
            .map(|f| IpcFill {
                rect: IpcRect {
                    x: f.rect.origin.x,
                    y: f.rect.origin.y,
                    width: f.rect.size.width,
                    height: f.rect.size.height,
                },
                color: IpcColor {
                    r: f.color.r,
                    g: f.color.g,
                    b: f.color.b,
                    a: f.color.a,
                },
            })
            .collect(),
        glyphs: primitives
            .glyphs
            .iter()
            .map(|g| IpcGlyph {
                x: g.x,
                y: g.y,
                font_size: g.font_size,
                glyph_id: g.glyph_id,
                font_id: g.font_id.0,
                color: IpcColor {
                    r: g.color.r,
                    g: g.color.g,
                    b: g.color.b,
                    a: g.color.a,
                },
                rotation: g.rotation,
            })
            .collect(),
    }
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

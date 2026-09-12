//! 无头协议会话 — BrowserShell 数据模型 + renderer IPC 管线。
//!
//! 发布构建只持有 renderer IPC（进程隔离）；进程内 WebView 仅用于单元测试。

use zero_browser_shell::BrowserShell;
#[cfg(not(test))]
use zero_net::{HttpClient, HttpMethod, HttpRequest};
#[cfg(not(test))]
use zero_protocol::message::{
    AutomationOperation, AutomationRequest, AutomationResult, AutomationValue, FetchParams, FramePublishMode,
    IpcMessage, IpcMessageKind, LoadHtmlParams, SetViewportParams,
};
#[cfg(not(test))]
use zero_protocol::process::RendererHandle;
#[cfg(test)]
use zero_webview::{WebView, WebViewConfig};

// ── 会话 ──

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
    #[cfg(not(test))]
    pub(super) navigation_epoch: u64,
    #[cfg(not(test))]
    pub(super) next_request_id: u64,
    /// R3282（#4）：可选 GPU 截图渲染器（`ZW_HEADLESS_GPU_SCREENSHOT=1` 启用；
    /// 默认 CPU——oracle 像素对比基线稳定）。
    pub(super) gpu_renderer: Option<zero_render_foundation::gpu::renderer::GpuRenderer>,
}

impl HeadlessSession {
    #[cfg(test)]
    pub(super) fn new(viewport_width: f32, viewport_height: f32) -> Self {
        let mut shell = BrowserShell::new();
        shell.new_tab(None);
        let config = WebViewConfig {
            width: viewport_width as u32,
            height: viewport_height as u32,
            ..Default::default()
        };
        let webview = WebView::new(config);
        Self {
            shell,
            webview,
            gpu_renderer: None,
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
        let mut shell = BrowserShell::new();
        shell.new_tab(None);
        Self {
            shell,
            renderer,
            http: HttpClient::new(),
            snapshot: crate::tab_snapshot::TabSnapshot::default(),
            navigation_epoch: 0,
            next_request_id: 1,
            gpu_renderer: None,
        }
    }
}

#[cfg(not(test))]
impl HeadlessSession {
    fn handle_renderer_message(&mut self, message: IpcMessage) -> Result<Option<Result<(), String>>, String> {
        match message.kind {
            IpcMessageKind::FetchRequest(params) => {
                self.proxy_fetch(params)?;
                Ok(None)
            }
            IpcMessageKind::CompositorFrame { paint, .. } | IpcMessageKind::ViewPainted(paint) => {
                crate::paint_ipc::apply_paint_snapshot(&mut self.snapshot, *paint);
                Ok(None)
            }
            IpcMessageKind::LoadComplete => Ok(Some(Ok(()))),
            IpcMessageKind::LoadFailed(message) | IpcMessageKind::CrashNotification(message) => Ok(Some(Err(message))),
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
        let response = self.http.send(HttpRequest {
            method,
            url: params.url,
            headers: params.headers,
            body: params.body,
        });
        match response {
            Ok(response) => self.renderer.send_fetch_response(
                params.request_id,
                response.status_code,
                response.headers,
                response.body,
            ),
            Err(error) => {
                self.renderer
                    .send_fetch_response(params.request_id, 0, Vec::new(), error.to_string().into_bytes())
            }
        }
        .map_err(|error| error.to_string())
    }

    fn wait_for_load(&mut self) -> Result<(), String> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if std::time::Instant::now() >= deadline {
                return Err("navigation timed out".into());
            }
            match self.renderer.try_recv().map_err(|error| error.to_string())? {
                Some(message) => {
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
        self.snapshot.begin_navigation(url.to_string());
        self.renderer
            .navigate(url, None, self.navigation_epoch)
            .map_err(|error| error.to_string())?;
        self.wait_for_load()
    }

    pub(super) fn load_html_renderer(&mut self, html: &str, css: Option<&str>) -> Result<(), String> {
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1).max(1);
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
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
        self.renderer
            .send(IpcMessage {
                id: request_id,
                kind: IpcMessageKind::AutomationRequest(AutomationRequest {
                    operation: AutomationOperation::ExecuteScript {
                        script: script.to_string(),
                        arguments: Vec::new(),
                    },
                }),
            })
            .map_err(|error| error.to_string())?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let response = loop {
            if std::time::Instant::now() >= deadline {
                return Err("automation request timeout".into());
            }
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
        match response.result.map_err(|error| error.message)? {
            AutomationResult::Value(AutomationValue::String(value)) => Ok(value),
            AutomationResult::Value(value) => serde_json::to_string(&value).map_err(|error| error.to_string()),
            _ => Ok(String::new()),
        }
    }
}

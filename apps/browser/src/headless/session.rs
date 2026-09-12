//! 无头协议会话 — BrowserShell 数据模型 + renderer IPC 管线。
//!
//! 发布构建只持有 renderer IPC（进程隔离）；进程内 WebView 仅用于单元测试。

use serde_json::Value;
use zero_browser_shell::BrowserShell;
use zero_net::cookie::CookieStore;
#[cfg(not(test))]
use zero_net::{HttpClient, HttpMethod, HttpRequest};
#[cfg(not(test))]
use zero_protocol::message::{
    AutomationOperation, AutomationRequest, AutomationResult, FetchParams, FramePublishMode, IpcMessage,
    IpcMessageKind, LoadHtmlParams,
};
use zero_protocol::message::{
    AutomationValue, ImeEventParams, ImeEventType, IpcColorScheme, IpcMediaType, KeyboardEventParams,
    KeyboardEventType, MouseEventParams, MouseEventType, ScrollEventParams, SetColorSchemeParams, SetMediaTypeParams,
    SetViewportParams,
};
#[cfg(not(test))]
use zero_protocol::process::RendererHandle;
#[cfg(test)]
use zero_webview::{WebView, WebViewConfig};

// ── 会话 ──

/// Page.addScriptToEvaluateOnNewDocument 登记的注入脚本。
///（ZeroWeb 单引擎无 world 隔离：一律在主 world 执行，见矩阵 createIsolatedWorld 注记。）
pub(super) struct InjectedScript {
    pub(super) identifier: String,
    pub(super) source: String,
    /// 注册时的 worldName（如 Playwright 的 utility world）；None = 主 world。
    pub(super) world_name: Option<String>,
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
    #[cfg(not(test))]
    pub(super) navigation_epoch: u64,
    #[cfg(not(test))]
    pub(super) next_request_id: u64,
    /// addScriptToEvaluateOnNewDocument 注册的脚本（新文档加载后重放）。
    pub(super) injected_scripts: Vec<InjectedScript>,
    /// Cookie jar（Storage 域 + proxy_fetch 双向接线；会话级——单会话模型即浏览器级）。
    pub(super) cookie_store: CookieStore,
    /// Emulation.setUserAgentOverride（None = 默认 UA）。
    pub(super) user_agent_override: Option<String>,
    /// Network.enable 门控（Network 域事件源开关）。
    pub(super) network_enabled: bool,
    /// Network 域事件队列（proxy_fetch 生命周期观测，transport 逐命令排空盖章）。
    pub(super) pending_network_events: Vec<(String, serde_json::Value)>,
    /// Console 事件队列（S11：renderer `ConsoleLog` → `Runtime.consoleAPICalled`，
    /// transport 逐命令排空盖章；`(level, text, args_json)`）。
    pub(super) pending_console_events: Vec<(String, String, String)>,
    /// R3282（#4）：可选 GPU 截图渲染器（`ZW_HEADLESS_GPU_SCREENSHOT=1` 启用；
    /// 默认 CPU——oracle 像素对比基线稳定）。
    pub(super) gpu_renderer: Option<zero_render_foundation::gpu::renderer::GpuRenderer>,
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
            injected_scripts: Vec::new(),
            cookie_store: CookieStore::new(),
            user_agent_override: None,
            network_enabled: false,
            pending_network_events: Vec::new(),
            pending_console_events: Vec::new(),
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
            injected_scripts: Vec::new(),
            cookie_store: CookieStore::new(),
            user_agent_override: None,
            network_enabled: false,
            pending_network_events: Vec::new(),
            pending_console_events: Vec::new(),
            gpu_renderer: None,
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
                crate::paint_ipc::apply_paint_snapshot(&mut self.snapshot, *paint);
                Ok(None)
            }
            IpcMessageKind::LoadComplete => Ok(Some(Ok(()))),
            IpcMessageKind::LoadFailed(message) | IpcMessageKind::CrashNotification(message) => Ok(Some(Err(message))),
            // S11：page console 输出 → 会话事件队列（transport 逐命令排空盖章为
            // `Runtime.consoleAPICalled`； PW 消费面 = msg.type()/text()）。
            IpcMessageKind::ConsoleLog(params) => {
                self.pending_console_events
                    .push((params.level, params.text, params.args_json));
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
            let cookie_header = self.cookie_store.cookie_header(parsed);
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
            self.pending_network_events.push((
                "Network.requestWillBeSent".to_string(),
                serde_json::json!({
                    "requestId": net_request_id,
                    "frameId": frame_id,
                    "request": {
                        "url": params.url,
                        "method": params.method,
                        "headers": request_headers,
                    },
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as f64,
                    "initiator": { "type": "other" },
                }),
            ));
        }

        let response = self.http.send(HttpRequest {
            method,
            url: params.url.clone(),
            headers,
            body: params.body,
        });
        match response {
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
                            self.cookie_store.add_from_url(cookie, parsed);
                        }
                    }
                }
                // Network 事件：responseReceived + loadingFinished
                if self.network_enabled {
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
                    self.pending_network_events.push((
                        "Network.loadingFinished".to_string(),
                        serde_json::json!({ "requestId": net_request_id }),
                    ));
                }
                self.renderer.send_fetch_response(
                    params.request_id,
                    response.status_code,
                    response.headers,
                    response.body,
                )
            }
            Err(error) => {
                if self.network_enabled {
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
        response.result.map_err(|error| error.message)
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
        event_type: KeyboardEventType,
    ) -> Result<(), String> {
        self.renderer
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                    key,
                    code,
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
}

impl HeadlessSession {
    /// 活跃标签页的主 frame id（proxy_fetch 等 session 侧无 CDP 会话上下文的场景）。
    pub(super) fn active_frame_id(&self) -> Option<String> {
        self.shell.active_tab().map(|tab| format!("zeroweb-tab-{}", tab.id().0))
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

    /// 测试进程内无 renderer：句柄桥操作不在此层执行（renderer 单测 + cdp-e2e
    /// 覆盖语义；此处仅保证编译面完整，dispatch 层形状断言用 -32000 传回）。
    pub(super) fn automation_request(
        &mut self,
        _operation: zero_protocol::message::AutomationOperation,
    ) -> Result<zero_protocol::message::AutomationResult, String> {
        Err("renderer unavailable in unit tests".into())
    }
}

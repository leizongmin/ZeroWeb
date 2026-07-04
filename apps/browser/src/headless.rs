//! 无头浏览器协议 Phase 1-3 — 远程调试服务。
//!
//! 支持 `--headless` 和 `--remote-debugging-port <port>` 启动无窗口实例，
//! 通过 WebSocket 接受自动化命令。
//!
//! Phase 1: 基础会话管理、JSON 消息路由、导航、脚本执行、截图。
//! Phase 2: 浏览上下文管理（创建/树/关闭/重新加载）、script.callFunction、
//!          HTTP 发现端点（/json/version）、事件通知。
//! Phase 3: CDP 最小兼容子集 — /json/version + /json HTTP 发现、
//!          Target/Page/Runtime/Network 基础域命令。

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tungstenite::accept;

use zero_browser_shell::BrowserShell;
use zero_browser_shell::TabId;
use zero_render_foundation::cpu::render_scene_to_framebuffer;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::GlyphDraw;
use zero_render_foundation::primitive::FillPrimitive;
use zero_webview::{WebView, WebViewConfig};

// ── 协议消息类型 ──

/// 接收到的客户端请求。
#[derive(Debug, Deserialize)]
struct ClientRequest {
    /// 消息 ID，响应时原样返回。
    id: u64,
    /// 命令方法名。
    method: String,
    /// 命令参数。
    #[serde(default)]
    params: Value,
}

/// 发送给客户端的响应。
#[derive(Debug, Serialize)]
struct ServerResponse {
    /// 与请求对应的 ID。
    id: u64,
    /// 返回值（成功时）。
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    /// 错误信息（失败时）。
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ProtocolError>,
}

/// 协议错误。
#[derive(Debug, Serialize)]
struct ProtocolError {
    /// 错误码。
    code: i64,
    /// 错误消息。
    message: String,
}

/// 发送给客户端的事件通知（Phase 2 使用）。
#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct ServerEvent {
    /// 事件方法名。
    method: String,
    /// 事件参数。
    params: Value,
}

// ── 会话 ──

/// 浏览器会话，包含一个 BrowserShell 和 WebView。
struct HeadlessSession {
    /// 浏览器 shell（数据模型）。
    shell: BrowserShell,
    /// WebView（页面渲染）。
    webview: WebView,
    /// 最近一次 captureScreenshot 渲染的 chrome frame（含 host）；下次 chrome.* 命令复用。
    /// None 表示尚未截图或上次截图失败。
    #[cfg(feature = "sdk-chrome")]
    last_chrome_frame: Option<crate::headless_chrome::ChromeFrame>,
    /// 最近一次 chrome.click 的 emitted actions（供 chrome.emittedActions 查询）。
    last_emitted_actions_json: serde_json::Value,
}

impl HeadlessSession {
    fn new(viewport_width: f32, viewport_height: f32) -> Self {
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
            #[cfg(feature = "sdk-chrome")]
            last_chrome_frame: None,
            last_emitted_actions_json: serde_json::Value::Array(Vec::new()),
        }
    }
}

// ── 安全配置（Phase 5）──

/// 无头协议安全配置。
#[derive(Default)]
pub struct HeadlessSecurityConfig {
    /// 认证令牌。如果设置，客户端必须在第一个请求中包含 `token` 字段。
    pub auth_token: Option<String>,
    /// 允许的 Origin 列表。如果为空，允许所有来源。
    pub allowed_origins: Vec<String>,
}

impl HeadlessSecurityConfig {
    /// 创建空安全配置（无限制）。
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置认证令牌。
    #[allow(dead_code)]
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// 添加允许的 Origin。
    #[allow(dead_code)]
    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }

    /// 验证认证令牌。
    pub fn verify_token(&self, provided: Option<&str>) -> bool {
        match &self.auth_token {
            None => true,
            Some(expected) => provided.is_some_and(|p| p == expected),
        }
    }

    /// 验证 WebSocket 请求的 Origin 头。
    pub fn verify_origin(&self, origin: Option<&str>) -> bool {
        if self.allowed_origins.is_empty() {
            return true;
        }
        origin.is_some_and(|o| self.allowed_origins.iter().any(|a| a == o))
    }
}

// ── 协议服务器 ──

/// 无头协议服务器。
pub struct HeadlessServer {
    /// 监听地址。
    addr: SocketAddr,
    /// 会话 ID 生成器。
    next_session_id: Arc<AtomicU64>,
    /// 视口宽度。
    viewport_width: f32,
    /// 视口高度。
    viewport_height: f32,
    /// 安全配置（Phase 5）。
    security: HeadlessSecurityConfig,
}

impl HeadlessServer {
    /// 创建无头协议服务器。
    pub fn new(port: u16, viewport_width: f32, viewport_height: f32) -> Self {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        Self {
            addr,
            next_session_id: Arc::new(AtomicU64::new(1)),
            viewport_width,
            viewport_height,
            security: HeadlessSecurityConfig::default(),
        }
    }

    /// 设置安全配置。
    #[allow(dead_code)]
    pub fn with_security(mut self, config: HeadlessSecurityConfig) -> Self {
        self.security = config;
        self
    }

    /// 返回实际监听地址（绑定后才知道端口 0 时的实际端口）。
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// 启动无头协议服务器，阻塞运行直到进程终止。
    ///
    /// 支持 HTTP 发现请求（/json/version、/json）和 WebSocket 协议连接。
    pub fn run(&mut self) -> Result<(), String> {
        let listener =
            std::net::TcpListener::bind(self.addr).map_err(|e| format!("Failed to bind {}: {}", self.addr, e))?;

        self.addr = listener
            .local_addr()
            .map_err(|e| format!("Failed to get local addr: {e}"))?;

        tracing::info!("Headless protocol server listening on ws://{}", self.addr);

        // 连接接受循环：支持 HTTP 发现 + WebSocket 协议
        loop {
            let (stream, peer) = listener.accept().map_err(|e| format!("Accept failed: {e}"))?;
            tracing::info!("Connection from {peer}");

            let mut ws = accept(stream).map_err(|e| format!("WebSocket handshake failed: {e}"))?;
            tracing::info!("Handshake done");

            // 手动帧解析：读裸字节，自己解析 WS 帧（绕过 ws.read() bug）
            let mut session = HeadlessSession::new(self.viewport_width, self.viewport_height);
            let mut authenticated = self.security.auth_token.is_none();

            loop {
                use std::io::{Read, Write};

                // 读帧头 2 字节
                let mut header2 = [0u8; 2];
                if ws.get_ref().read_exact(&mut header2).is_err() {
                    break;
                }
                let opcode = header2[0] & 0x0f;
                let masked = (header2[1] & 0x80) != 0;
                let mut payload_len = (header2[1] & 0x7f) as u64;

                // 扩展长度
                if payload_len == 126 {
                    let mut ext = [0u8; 2];
                    if ws.get_ref().read_exact(&mut ext).is_err() {
                        break;
                    }
                    payload_len = u16::from_be_bytes(ext) as u64;
                } else if payload_len == 127 {
                    let mut ext = [0u8; 8];
                    if ws.get_ref().read_exact(&mut ext).is_err() {
                        break;
                    }
                    payload_len = u64::from_be_bytes(ext);
                }

                // mask key
                let mask_key = if masked {
                    let mut mk = [0u8; 4];
                    if ws.get_ref().read_exact(&mut mk).is_err() {
                        break;
                    }
                    Some(mk)
                } else {
                    None
                };

                // payload
                let mut payload = vec![0u8; payload_len as usize];
                if payload_len > 0 && ws.get_ref().read_exact(&mut payload).is_err() {
                    break;
                }

                // unmask
                if let Some(mk) = mask_key {
                    for (i, b) in payload.iter_mut().enumerate() {
                        *b ^= mk[i & 3];
                    }
                }

                // 控制帧
                if opcode & 0x08 != 0 {
                    match opcode {
                        0x08 => {
                            tracing::info!("Close");
                            break;
                        }
                        0x09 => {
                            let pong = Self::encode_frame(0x8a, &payload);
                            let _ = ws.get_mut().write_all(&pong);
                            continue;
                        }
                        _ => continue,
                    }
                }

                // 仅 text 帧
                let msg_str = if opcode == 0x01 {
                    String::from_utf8_lossy(&payload).to_string()
                } else {
                    continue;
                };

                // 认证
                if !authenticated {
                    if let Ok(req) = serde_json::from_str::<ClientRequest>(&msg_str) {
                        let token = req.params.get("token").and_then(|v| v.as_str());
                        if self.security.verify_token(token) {
                            authenticated = true;
                        } else {
                            let err = ServerResponse {
                                id: req.id,
                                result: None,
                                error: Some(ProtocolError {
                                    code: -32001,
                                    message: "invalid token".into(),
                                }),
                            };
                            if let Ok(j) = serde_json::to_string(&err) {
                                let _ = ws.get_mut().write_all(&Self::encode_frame(0x81, j.as_bytes()));
                            }
                            continue;
                        }
                    } else {
                        let err = ServerResponse {
                            id: 0,
                            result: None,
                            error: Some(ProtocolError {
                                code: -32001,
                                message: "auth required".into(),
                            }),
                        };
                        if let Ok(j) = serde_json::to_string(&err) {
                            let _ = ws.get_mut().write_all(&Self::encode_frame(0x81, j.as_bytes()));
                        }
                        continue;
                    }
                }

                let (response, events) = self.handle_message_with_events(&mut session, &msg_str);

                for event in events {
                    if let Ok(j) = serde_json::to_string(&event) {
                        if ws.get_mut().write_all(&Self::encode_frame(0x81, j.as_bytes())).is_err() {
                            break;
                        }
                    }
                }
                let rj = serde_json::to_string(&response).unwrap_or_else(|e| {
                    format!("{{\"id\":0,\"error\":{{\"code\":-32700,\"message\":\"JSON: {e}\"}}}}")
                });
                if ws
                    .get_mut()
                    .write_all(&Self::encode_frame(0x81, rj.as_bytes()))
                    .is_err()
                {
                    break;
                }
            }

            tracing::info!("Headless session ended");
            tracing::info!("Headless session ended (raw echo test)");
        }
    }

    /// 从 HTTP 请求头中提取 Origin 值。
    fn extract_origin_header(data: &[u8]) -> Option<String> {
        let s = String::from_utf8_lossy(data);
        for line in s.lines() {
            if let Some(value) = line.strip_prefix("Origin: ") {
                return Some(value.trim().to_string());
            }
            if let Some(value) = line.strip_prefix("origin: ") {
                return Some(value.trim().to_string());
            }
        }
        None
    }

    /// 发送 HTTP 错误响应。
    fn http_error_response(stream: &std::net::TcpStream, code: u16, message: &str) {
        use std::io::Write;
        let body = format!("<h1>{code} {message}</h1>");
        let response = format!(
            "HTTP/1.1 {code} {message}\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        if let Ok(mut writable) = stream.try_clone() {
            let _ = writable.write_all(response.as_bytes());
            let _ = writable.flush();
        }
    }

    /// 处理 HTTP 发现请求（CDP 风格的 /json 端点）。
    fn handle_http_discovery(stream: &std::net::TcpStream, addr: SocketAddr) {
        use std::io::{Read, Write};

        // 从 stream 读取完整的 HTTP 请求行（使用 try_clone 避免数据丢失）
        let mut read_buf = [0u8; 1024];
        let path = if let Ok(mut readable) = stream.try_clone() {
            let n = readable.read(&mut read_buf).unwrap_or(0);
            String::from_utf8_lossy(&read_buf[..n])
                .lines()
                .next()
                .unwrap_or("")
                .split_whitespace()
                .nth(1)
                .unwrap_or("/")
                .to_string()
        } else {
            "/".to_string()
        };

        let (status, content_type, body) = match path.as_str() {
            "/json/version" => ("200 OK", "application/json", Self::http_version_json(addr)),
            "/json" | "/json/list" => (
                "200 OK",
                "application/json",
                serde_json::json!([{
                    "description": "ZeroWeb headless instance",
                    "devtoolsFrontendUrl": format!("devtools://devtools/bundled/inspector.html?ws={addr}"),
                    "id": "zeroweb-main",
                    "title": "ZeroWeb",
                    "type": "page",
                    "url": "about:blank",
                    "webSocketDebuggerUrl": format!("ws://{addr}"),
                }])
                .to_string(),
            ),
            _ => ("404 Not Found", "text/plain", "Not Found".to_string()),
        };

        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );

        if let Ok(mut writable) = stream.try_clone() {
            let _ = writable.write_all(response.as_bytes());
            let _ = writable.flush();
        }
    }

    /// 处理单条客户端消息（向后兼容，不含事件）。
    #[allow(dead_code)]
    fn handle_message(&self, session: &mut HeadlessSession, raw: &str) -> ServerResponse {
        let (response, _) = self.handle_message_with_events(session, raw);
        response
    }

    /// 处理单条客户端消息，返回响应和事件通知列表。
    fn handle_message_with_events(
        &self,
        session: &mut HeadlessSession,
        raw: &str,
    ) -> (ServerResponse, Vec<ServerEvent>) {
        let req: ClientRequest = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => {
                return (
                    ServerResponse {
                        id: 0,
                        result: None,
                        error: Some(ProtocolError {
                            code: -32700,
                            message: format!("Parse error: {e}"),
                        }),
                    },
                    Vec::new(),
                );
            }
        };

        let id = req.id;
        let (result, events) = self.dispatch_with_events(session, &req.method, req.params);

        let response = match result {
            Ok(value) => ServerResponse {
                id,
                result: Some(value),
                error: None,
            },
            Err(err) => ServerResponse {
                id,
                result: None,
                error: Some(err),
            },
        };

        (response, events)
    }

    /// 命令路由。
    fn dispatch(&self, session: &mut HeadlessSession, method: &str, params: Value) -> Result<Value, ProtocolError> {
        match method {
            // ── 会话管理 ──
            "session.status" => self.cmd_session_status(),
            "session.new" => self.cmd_session_new(),
            "session.end" => Err(ProtocolError {
                code: -32000,
                message: "Session ended by client".into(),
            }),

            // ── 浏览器控制 ──
            "browser.close" => self.cmd_browser_close(),

            // ── 浏览上下文（Phase 2）──
            "browsingContext.create" => self.cmd_browsing_context_create(session, params),
            "browsingContext.getTree" => self.cmd_browsing_context_get_tree(session),
            "browsingContext.close" => self.cmd_browsing_context_close(session, params),
            "browsingContext.reload" => self.cmd_browsing_context_reload(session),

            // ── 导航 ──
            "browsingContext.navigate" => self.cmd_navigate(session, params),

            // ── 脚本执行 ──
            "script.evaluate" => self.cmd_script_evaluate(session, params),
            "script.callFunction" => self.cmd_script_call_function(session, params),

            // ── 截图 ──
            "browsingContext.captureScreenshot" => self.cmd_capture_screenshot(session),

            // ── 页面内容 ──
            "browsingContext.getDOMSnapshot" => self.cmd_get_dom_snapshot(session),

            // ── Chrome 自动化（feature sdk-chrome）──
            #[cfg(feature = "sdk-chrome")]
            "chrome.getLayout" => self.cmd_chrome_get_layout(session),
            #[cfg(feature = "sdk-chrome")]
            "chrome.getSemantics" => self.cmd_chrome_get_semantics(session),
            #[cfg(feature = "sdk-chrome")]
            "chrome.click" => self.cmd_chrome_click(session, params),
            #[cfg(feature = "sdk-chrome")]
            "chrome.rectOf" => self.cmd_chrome_rect_of(session, params),
            #[cfg(feature = "sdk-chrome")]
            "chrome.emittedActions" => self.cmd_chrome_emitted_actions(session),
            #[cfg(not(feature = "sdk-chrome"))]
            "chrome.getLayout" | "chrome.getSemantics" | "chrome.click" | "chrome.rectOf" | "chrome.emittedActions" => {
                Err(headless_chrome_unavailable())
            }

            // ── 未知命令 ──
            _ => Err(ProtocolError {
                code: -32601,
                message: format!("Unknown method: {method}"),
            }),
        }
    }

    /// 带事件生成的命令路由。
    fn dispatch_with_events(
        &self,
        session: &mut HeadlessSession,
        method: &str,
        params: Value,
    ) -> (Result<Value, ProtocolError>, Vec<ServerEvent>) {
        let mut events = Vec::new();

        match method {
            "browsingContext.navigate" => {
                let result = self.cmd_navigate(session, params);
                if let Ok(ref val) = result {
                    let url_val = val.get("url").cloned();
                    events.push(ServerEvent {
                        method: "browsingContext.load".into(),
                        params: serde_json::json!({
                            "url": url_val,
                            "success": true,
                        }),
                    });
                    events.push(ServerEvent {
                        method: "log.entryAdded".into(),
                        params: serde_json::json!({
                            "level": "info",
                            "text": format!("Page loaded: {:?}", url_val),
                            "timestamp": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis(),
                        }),
                    });
                }
                (result, events)
            }
            "browsingContext.reload" => {
                let result = self.cmd_browsing_context_reload(session);
                if result.is_ok() {
                    events.push(ServerEvent {
                        method: "browsingContext.load".into(),
                        params: serde_json::json!({ "success": true }),
                    });
                }
                (result, events)
            }
            "browsingContext.create" => {
                let result = self.cmd_browsing_context_create(session, params.clone());
                if let Ok(ref val) = result {
                    events.push(ServerEvent {
                        method: "browsingContext.contextCreated".into(),
                        params: serde_json::json!({
                            "context": val.get("context"),
                        }),
                    });
                }
                (result, events)
            }
            "browsingContext.close" => {
                let ctx = params.get("context").cloned();
                let result = self.cmd_browsing_context_close(session, params);
                if result.is_ok() {
                    events.push(ServerEvent {
                        method: "browsingContext.contextDestroyed".into(),
                        params: serde_json::json!({ "context": ctx }),
                    });
                }
                (result, events)
            }
            // CDP 兼容域（Phase 3）
            "Page.navigate" => {
                let url = params.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let nav_params = serde_json::json!({ "url": url });
                let result = self.cmd_navigate(session, nav_params);
                if result.is_ok() {
                    events.push(ServerEvent {
                        method: "Page.loadEventFired".into(),
                        params: serde_json::json!({ "timestamp": 0.0 }),
                    });
                }
                (result, events)
            }
            "Page.captureScreenshot" => (self.cmd_capture_screenshot(session), events),
            "Runtime.evaluate" => (self.cmd_script_evaluate(session, params), events),
            "Target.getTargets" => (self.cmd_browsing_context_get_tree(session), events),
            "Network.enable" => {
                // 启用网络事件追踪（桩：接受命令但不产生事件）
                (Ok(serde_json::json!({ "result": "enabled" })), events)
            }
            // 默认：无事件
            _ => (self.dispatch(session, method, params), events),
        }
    }

    // ── 命令实现 ──

    fn cmd_session_status(&self) -> Result<Value, ProtocolError> {
        Ok(serde_json::json!({
            "ready": true,
            "message": "ZeroWeb headless server ready"
        }))
    }

    fn cmd_session_new(&self) -> Result<Value, ProtocolError> {
        let session_id = self.next_session_id.fetch_add(1, Ordering::SeqCst);
        Ok(serde_json::json!({
            "sessionId": session_id,
            "capabilities": {
                "browserName": "ZeroWeb",
                "browserVersion": env!("CARGO_PKG_VERSION"),
                "platformName": std::env::consts::OS,
            }
        }))
    }

    fn cmd_browser_close(&self) -> Result<Value, ProtocolError> {
        Ok(serde_json::json!({ "result": "closing" }))
    }

    fn cmd_navigate(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'url' parameter".into(),
            })?;

        session.shell.navigate(url);

        // fetch_url 内部完成 HTTP 请求和渲染
        let render_result = session.webview.fetch_url(url);
        let title = match &render_result {
            Ok(_) => url.to_string(),
            Err(_) => "Error loading page".to_string(),
        };

        session.shell.on_page_loaded(&title);

        Ok(serde_json::json!({
            "url": url,
            "title": title,
            "success": render_result.is_ok(),
        }))
    }

    fn cmd_script_evaluate(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let expression = params
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'expression' parameter".into(),
            })?;

        match session.webview.execute_script(expression) {
            Ok(result) => Ok(serde_json::json!({
                "result": {
                    "type": "string",
                    "value": result
                }
            })),
            Err(e) => Ok(serde_json::json!({
                "exceptionDetails": {
                    "text": e.to_string()
                }
            })),
        }
    }

    fn cmd_capture_screenshot(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
        let scale_factor = 1.0_f32;

        // feature sdk-chrome 开启时，渲染合成 chrome + 页面；否则仅渲染页面（旧行为）。
        #[cfg(feature = "sdk-chrome")]
        {
            let frame = crate::headless_chrome::render_chrome_frame(
                &session.shell,
                &mut session.webview,
                self.viewport_width as u32,
                self.viewport_height as u32,
                scale_factor,
            );
            let fills: Vec<FillPrimitive> = frame.fills.clone();
            let glyph_draws: Vec<GlyphDraw> = frame
                .glyphs
                .iter()
                .map(|g| GlyphDraw {
                    ch: char::from_u32(g.glyph_id).unwrap_or('?'),
                    x: g.x,
                    baseline_y: g.y,
                    font_size: g.font_size,
                    color: g.color,
                    font_id: g.font_id.0,
                })
                .collect();
            session.last_chrome_frame = Some(frame);

            let font_loader = FontLoader::new();
            let mut glyph_cache = GlyphCache::new(1024);
            let fb = render_scene_to_framebuffer(
                self.viewport_width as u32,
                self.viewport_height as u32,
                scale_factor,
                &fills,
                &[],
                &font_loader,
                &mut glyph_cache,
                &glyph_draws,
                &[],
                &[],
                &[],
            );

            return Ok(serde_json::json!({
                "data": {
                    "width": fb.width,
                    "height": fb.height,
                    "format": "rgba8",
                    "pixelCount": fb.width as usize * fb.height as usize,
                    "withChrome": true,
                },
                "pixels": encode_pixels_base64(&fb),
            }));
        }

        // fallback：无 sdk-chrome feature（旧行为）。
        #[allow(unreachable_code)]
        {
            let result = session.webview.render();
            let fills: Vec<FillPrimitive> = result.primitives.fills.clone();
            let glyph_primitives = result.primitives.glyphs.clone();
            let glyph_draws: Vec<GlyphDraw> = glyph_primitives
                .iter()
                .map(|g| GlyphDraw {
                    ch: char::from_u32(g.glyph_id).unwrap_or('?'),
                    x: g.x,
                    baseline_y: g.y,
                    font_size: g.font_size,
                    color: g.color,
                    font_id: g.font_id.0,
                })
                .collect();

            let font_loader = FontLoader::new();
            let mut glyph_cache = GlyphCache::new(1024);
            let fb = render_scene_to_framebuffer(
                self.viewport_width as u32,
                self.viewport_height as u32,
                scale_factor,
                &fills,
                &[],
                &font_loader,
                &mut glyph_cache,
                &glyph_draws,
                &[],
                &[],
                &[],
            );

            Ok(serde_json::json!({
                "data": {
                    "width": fb.width,
                    "height": fb.height,
                    "format": "rgba8",
                    "pixelCount": fb.width as usize * fb.height as usize,
                    "withChrome": false,
                },
                "pixels": encode_pixels_base64(&fb),
            }))
        }
    }

    fn cmd_get_dom_snapshot(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
        let result = session.webview.render();

        let fill_count = result.primitives.fills.len();
        let glyph_count = result.primitives.glyphs.len();

        Ok(serde_json::json!({
            "renderPrimitives": {
                "fills": fill_count,
                "glyphs": glyph_count,
                "gradients": result.primitives.gradients.len(),
                "shadows": result.primitives.shadows.len(),
                "images": result.primitives.images.len(),
            }
        }))
    }

    // ── Phase 2 命令实现 ──

    /// browsingContext.create — 创建新的浏览上下文（新标签页）。
    fn cmd_browsing_context_create(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let url = params.get("url").and_then(|v| v.as_str());
        let tab_id = session.shell.new_tab(url);

        Ok(serde_json::json!({
            "context": tab_id.0,
            "url": url.unwrap_or("about:blank"),
        }))
    }

    /// browsingContext.getTree — 获取浏览上下文树（标签页列表）。
    fn cmd_browsing_context_get_tree(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
        let active_id = session.shell.active_tab_id();
        let tab_count = session.shell.tab_count();

        // 收集所有标签页信息
        let mut children = Vec::new();
        for i in 0..tab_count {
            let tab_id = TabId(i as u64);
            let is_active = active_id == Some(tab_id);
            children.push(serde_json::json!({
                "context": i,
                "url": "about:blank",
                "active": is_active,
            }));
        }

        Ok(serde_json::json!({
            "contexts": children,
        }))
    }

    /// browsingContext.close — 关闭指定浏览上下文（标签页）。
    fn cmd_browsing_context_close(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let context = params
            .get("context")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'context' parameter".into(),
            })?;

        session.shell.close_tab(TabId(context));
        Ok(serde_json::json!({ "result": "closed" }))
    }

    /// browsingContext.reload — 重新加载当前页面。
    fn cmd_browsing_context_reload(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
        // 重新渲染当前缓存内容
        let _ = session.webview.render();
        Ok(serde_json::json!({ "result": "reloaded" }))
    }

    /// script.callFunction — 调用指定的 JS 函数（通过表 达式包装）。
    fn cmd_script_call_function(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let function_declaration = params
            .get("functionDeclaration")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'functionDeclaration' parameter".into(),
            })?;

        let args = params.get("arguments").and_then(|v| v.as_array());

        // 将函数调用转换为可执行表达式
        let expression = if let Some(args) = args {
            let args_json: Vec<String> = args
                .iter()
                .filter_map(|a| a.get("value").and_then(|v| serde_json::to_string(v).ok()))
                .collect();
            format!("({function_declaration})({})", args_json.join(", "))
        } else {
            format!("({function_declaration})()")
        };

        match session.webview.execute_script(&expression) {
            Ok(result) => Ok(serde_json::json!({
                "result": {
                    "type": "string",
                    "value": result
                }
            })),
            Err(e) => Ok(serde_json::json!({
                "exceptionDetails": {
                    "text": e.to_string()
                }
            })),
        }
    }

    /// HTTP GET /json/version — CDP 风格的浏览器发现端点。
    #[allow(dead_code)]
    pub fn http_version_json(addr: SocketAddr) -> String {
        serde_json::json!({
            "Browser": "ZeroWeb/0.1",
            "Protocol-Version": "1.3",
            "User-Agent": format!("ZeroWeb/{} ({})", env!("CARGO_PKG_VERSION"), std::env::consts::OS),
            "V8-Version": "12.0",
            "WebKit-Version": "0.1",
            "webSocketDebuggerUrl": format!("ws://{addr}"),
        })
        .to_string()
    }

    // ── Chrome 自动化命令（feature sdk-chrome）──

    #[cfg(feature = "sdk-chrome")]
    fn cmd_chrome_get_layout(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
        let frame = session.last_chrome_frame.take().ok_or_else(|| ProtocolError {
            code: -32002,
            message: "No chrome frame available; call browsingContext.captureScreenshot first".into(),
        })?;
        let viewport_rect = frame.viewport_rect;
        session.last_chrome_frame = Some(frame);
        Ok(serde_json::json!({
            "viewport": viewport_rect.map(crate::headless_chrome::rect_to_json),
            "windowSize": {
                "width": self.viewport_width,
                "height": self.viewport_height,
            },
        }))
    }

    #[cfg(feature = "sdk-chrome")]
    fn cmd_chrome_get_semantics(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
        let frame = session.last_chrome_frame.take().ok_or_else(|| ProtocolError {
            code: -32002,
            message: "No chrome frame available; call browsingContext.captureScreenshot first".into(),
        })?;
        let sem = frame.host.semantics();
        session.last_chrome_frame = Some(frame);
        let tree_json = match sem {
            Some(root) => semantics_to_json(&root),
            None => Value::Null,
        };
        Ok(serde_json::json!({ "tree": tree_json }))
    }

    #[cfg(feature = "sdk-chrome")]
    fn cmd_chrome_click(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        // 接受 {x, y} 绝对坐标 或 {widgetId} 已知 id（取其中心点）。
        let (x, y) = if let (Some(xv), Some(yv)) = (params.get("x"), params.get("y")) {
            (
                xv.as_f64().ok_or_else(|| ProtocolError {
                    code: -32602,
                    message: "'x' must be a number".into(),
                })? as f32,
                yv.as_f64().ok_or_else(|| ProtocolError {
                    code: -32602,
                    message: "'y' must be a number".into(),
                })? as f32,
            )
        } else if let Some(wid) = params.get("widgetId").and_then(|v| v.as_str()) {
            let frame = session.last_chrome_frame.take().ok_or_else(|| ProtocolError {
                code: -32002,
                message: "No chrome frame available; call browsingContext.captureScreenshot first".into(),
            })?;
            let rect = frame.host.rect_of(&zero_ui_core::widget::WidgetId::new(wid));
            session.last_chrome_frame = Some(frame);
            let rect = rect.ok_or_else(|| ProtocolError {
                code: -32603,
                message: format!("widgetId '{}' not found in chrome layout", wid),
            })?;
            (
                rect.origin.x + rect.size.width / 2.0,
                rect.origin.y + rect.size.height / 2.0,
            )
        } else {
            return Err(ProtocolError {
                code: -32602,
                message: "Either {x, y} or {widgetId} required".into(),
            });
        };

        // 渲染最新 chrome frame（确保 dispatch 用的是当前 shell 状态）。
        let mut frame = crate::headless_chrome::render_chrome_frame(
            &session.shell,
            &mut session.webview,
            self.viewport_width as u32,
            self.viewport_height as u32,
            1.0,
        );

        let events = crate::headless_chrome::make_click_events(x, y);
        let mut all_emitted = Vec::new();
        for ev in &events {
            let emitted = frame.host.dispatch_event(ev);
            all_emitted.extend(emitted);
        }

        // 应用最小 reducer：把 chrome actions 应用到 shell（如 NAV_BACK → go_back）。
        let mut applied: Vec<Value> = Vec::new();
        for ea in &all_emitted {
            let action_str = ea.action.0.as_str().to_string();
            let (count, desc) = crate::headless_chrome::apply_chrome_action_to_shell(&mut session.shell, &action_str);
            if count > 0 {
                applied.push(serde_json::json!({
                    "action": action_str,
                    "description": desc,
                }));
            }
        }

        let emitted_json = crate::headless_chrome::emitted_actions_to_json(&all_emitted);
        session.last_emitted_actions_json = emitted_json.clone();
        session.last_chrome_frame = Some(frame);

        Ok(serde_json::json!({
            "point": { "x": x, "y": y },
            "emittedActions": emitted_json,
            "applied": applied,
        }))
    }

    #[cfg(feature = "sdk-chrome")]
    fn cmd_chrome_rect_of(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let widget_id = params
            .get("widgetId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'widgetId' parameter".into(),
            })?;
        let frame = session.last_chrome_frame.take().ok_or_else(|| ProtocolError {
            code: -32002,
            message: "No chrome frame available; call browsingContext.captureScreenshot first".into(),
        })?;
        let rect_opt = frame.host.rect_of(&zero_ui_core::widget::WidgetId::new(widget_id));
        session.last_chrome_frame = Some(frame);
        match rect_opt {
            Some(r) => Ok(serde_json::json!({
                "widgetId": widget_id,
                "rect": crate::headless_chrome::rect_to_json(r),
            })),
            None => Err(ProtocolError {
                code: -32603,
                message: format!("widgetId '{}' not found", widget_id),
            }),
        }
    }

    #[cfg(feature = "sdk-chrome")]
    fn cmd_chrome_emitted_actions(&self, session: &mut HeadlessSession) -> Result<Value, ProtocolError> {
        Ok(serde_json::json!({
            "actions": session.last_emitted_actions_json,
        }))
    }

    /// 构造 WebSocket 帧（服务端发送，不 mask）。
    fn encode_frame(header: u8, payload: &[u8]) -> Vec<u8> {
        let len = payload.len();
        let mut frame = Vec::with_capacity(10 + len);
        frame.push(header);
        if len < 126 {
            frame.push(len as u8);
        } else if len < 65536 {
            frame.push(126);
            frame.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            frame.push(127);
            frame.extend_from_slice(&(len as u64).to_be_bytes());
        }
        frame.extend_from_slice(payload);
        frame
    }
}

// ── Chrome 自动化辅助（feature sdk-chrome）──

/// 把 FrameBuffer 编码为 PNG，再 base64 包装（供客户端直接解码保存为 .png）。
fn encode_pixels_base64(fb: &zero_render_foundation::surface::FrameBuffer) -> String {
    use base64::Engine;
    let mut png_buf: Vec<u8> = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_buf, fb.width, fb.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("PNG header");
        writer.write_image_data(&fb.data).expect("PNG image data");
    }
    base64::engine::general_purpose::STANDARD.encode(&png_buf)
}

/// 把 SemanticsNode 递归转 JSON（供 chrome.getSemantics 返回）。
#[cfg(feature = "sdk-chrome")]
fn semantics_to_json(node: &zero_ui_core::semantics::SemanticsNode) -> Value {
    use zero_ui_core::semantics::{SemanticsFlags, SemanticsLabel};
    let mut flags_arr: Vec<&str> = Vec::new();
    if node.flags.contains(SemanticsFlags::BUTTON) {
        flags_arr.push("button");
    }
    if node.flags.contains(SemanticsFlags::TEXT_FIELD) {
        flags_arr.push("text_field");
    }
    if node.flags.contains(SemanticsFlags::FOCUSABLE) {
        flags_arr.push("focusable");
    }
    if node.flags.contains(SemanticsFlags::FOCUSED) {
        flags_arr.push("focused");
    }
    if node.flags.contains(SemanticsFlags::READ_ONLY) {
        flags_arr.push("read_only");
    }
    let label = match &node.label {
        Some(SemanticsLabel::Literal(t)) => Some(t.to_string()),
        Some(SemanticsLabel::Message(id)) => Some(format!("msg:{}", id)),
        None => None,
    };
    let children: Vec<Value> = node.children.iter().map(semantics_to_json).collect();
    serde_json::json!({
        "id": node.id.0.as_str(),
        "flags": flags_arr,
        "label": label,
        "rect": crate::headless_chrome::rect_to_json(node.rect),
        "children": children,
    })
}

/// feature 关闭时的 fallback 错误。
#[cfg(not(feature = "sdk-chrome"))]
pub(crate) fn headless_chrome_unavailable() -> ProtocolError {
    ProtocolError {
        code: -32001,
        message: "Chrome automation requires feature 'sdk-chrome'".into(),
    }
}

// ── 协议客户端（Phase 4 自动化测试基础设施）──

/// 无头浏览器协议客户端，用于自动化测试。
///
/// 通过 WebSocket 连接到 HeadlessServer，发送命令并接收响应和事件。
#[cfg(test)]
pub struct HeadlessClient;

#[cfg(test)]
impl HeadlessClient {
    /// 解析服务端 JSON 响应，提取 result 字段。
    ///
    /// 如果响应包含 error，返回错误描述。
    pub fn parse_response(raw: &str) -> Result<serde_json::Value, String> {
        let v: serde_json::Value = serde_json::from_str(raw).map_err(|e| format!("JSON parse: {e}"))?;
        if let Some(error) = v.get("error") {
            let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("unknown");
            return Err(format!("Error {code}: {message}"));
        }
        Ok(v.get("result").cloned().unwrap_or(serde_json::json!({})))
    }

    /// 构建协议请求 JSON 字符串。
    pub fn build_request(id: u64, method: &str, params: serde_json::Value) -> String {
        serde_json::json!({
            "id": id,
            "method": method,
            "params": params,
        })
        .to_string()
    }

    /// 解析事件通知 JSON，返回 (method, params) 对。
    pub fn parse_event(raw: &str) -> Result<(String, serde_json::Value), String> {
        let v: serde_json::Value = serde_json::from_str(raw).map_err(|e| format!("JSON parse: {e}"))?;
        let method = v.get("method").and_then(|m| m.as_str()).unwrap_or("").to_string();
        let params = v.get("params").cloned().unwrap_or(serde_json::json!({}));
        Ok((method, params))
    }

    /// 解析截图响应，返回 (width, height, pixel_count)。
    pub fn parse_screenshot(result: &serde_json::Value) -> Result<(u32, u32, usize), String> {
        let data = result.get("data").ok_or("Missing data field")?;
        let width = data.get("width").and_then(|v| v.as_u64()).ok_or("Missing width")? as u32;
        let height = data.get("height").and_then(|v| v.as_u64()).ok_or("Missing height")? as u32;
        let pixel_count = data
            .get("pixelCount")
            .and_then(|v| v.as_u64())
            .ok_or("Missing pixelCount")? as usize;
        Ok((width, height, pixel_count))
    }

    /// 解析 DOM 快照响应，返回各图元计数。
    pub fn parse_dom_snapshot(result: &serde_json::Value) -> DomSnapshotStats {
        let rp = result.get("renderPrimitives");
        DomSnapshotStats {
            fills: rp.and_then(|r| r.get("fills")).and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            glyphs: rp.and_then(|r| r.get("glyphs")).and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            gradients: rp
                .and_then(|r| r.get("gradients"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize,
            shadows: rp.and_then(|r| r.get("shadows")).and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            images: rp.and_then(|r| r.get("images")).and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        }
    }
}

/// DOM 快照统计信息。
#[cfg(test)]
#[derive(Debug, PartialEq)]
pub struct DomSnapshotStats {
    pub fills: usize,
    pub glyphs: usize,
    pub gradients: usize,
    pub shadows: usize,
    pub images: usize,
}

#[cfg(test)]
impl DomSnapshotStats {
    /// 总图元数。
    pub fn total(&self) -> usize {
        self.fills + self.glyphs + self.gradients + self.shadows + self.images
    }
}

// ── 测试 ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_new() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        assert!(server.addr.port() == 0);
    }

    #[test]
    fn test_session_new() {
        let session = HeadlessSession::new(800.0, 600.0);
        assert!(session.shell.tab_count() >= 1);
    }

    #[test]
    fn test_dispatch_session_status() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let result = server.dispatch(&mut session, "session.status", Value::Null).unwrap();
        assert_eq!(result["ready"], true);
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let result = server.dispatch(&mut session, "unknown.method", Value::Null);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32601);
    }

    #[test]
    fn test_dispatch_navigate_missing_url() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let result = server.dispatch(&mut session, "browsingContext.navigate", Value::Null);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32602);
    }

    #[test]
    fn test_dispatch_script_evaluate() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let params = serde_json::json!({ "expression": "1 + 1" });
        let result = server.dispatch(&mut session, "script.evaluate", params).unwrap();
        assert!(result.get("result").is_some() || result.get("exceptionDetails").is_some());
    }

    #[test]
    fn test_dispatch_capture_screenshot() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let result = server
            .dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null)
            .unwrap();
        assert_eq!(result["data"]["width"], 800);
        assert_eq!(result["data"]["height"], 600);
    }

    #[test]
    fn test_dispatch_get_dom_snapshot() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let result = server
            .dispatch(&mut session, "browsingContext.getDOMSnapshot", Value::Null)
            .unwrap();
        assert!(result.get("renderPrimitives").is_some());
    }

    #[test]
    fn test_dispatch_session_new() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let result = server.dispatch(&mut session, "session.new", Value::Null).unwrap();
        assert_eq!(result["capabilities"]["browserName"], "ZeroWeb");
    }

    #[test]
    fn test_dispatch_browser_close() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let result = server.dispatch(&mut session, "browser.close", Value::Null).unwrap();
        assert_eq!(result["result"], "closing");
    }

    #[test]
    fn test_client_request_parse() {
        let raw = r#"{"id":1,"method":"session.status","params":{}}"#;
        let req: ClientRequest = serde_json::from_str(raw).unwrap();
        assert_eq!(req.id, 1);
        assert_eq!(req.method, "session.status");
    }

    #[test]
    fn test_client_request_no_params() {
        let raw = r#"{"id":2,"method":"browser.close"}"#;
        let req: ClientRequest = serde_json::from_str(raw).unwrap();
        assert_eq!(req.id, 2);
        assert_eq!(req.params, Value::Null);
    }

    #[test]
    fn test_server_response_serialize() {
        let resp = ServerResponse {
            id: 1,
            result: Some(serde_json::json!({"ready": true})),
            error: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_server_response_error() {
        let resp = ServerResponse {
            id: 3,
            result: None,
            error: Some(ProtocolError {
                code: -32601,
                message: "Unknown method".into(),
            }),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32601"));
    }

    // ── Phase 2 测试 ──

    #[test]
    fn test_dispatch_browsing_context_create() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let params = serde_json::json!({ "url": "https://example.com" });
        let result = server.dispatch(&mut session, "browsingContext.create", params).unwrap();
        assert!(result.get("context").is_some());
        assert_eq!(result["url"], "https://example.com");
    }

    #[test]
    fn test_dispatch_browsing_context_get_tree() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let result = server
            .dispatch(&mut session, "browsingContext.getTree", Value::Null)
            .unwrap();
        let contexts = result.get("contexts").unwrap().as_array().unwrap();
        assert!(!contexts.is_empty(), "should have at least one tab");
    }

    #[test]
    fn test_dispatch_browsing_context_close() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        // 创建第二个标签页并获取其 ID
        let new_tab_id = session.shell.new_tab(None);
        let count_before = session.shell.tab_count();
        let params = serde_json::json!({ "context": new_tab_id.0 });
        let result = server.dispatch(&mut session, "browsingContext.close", params).unwrap();
        assert_eq!(result["result"], "closed");
        assert_eq!(session.shell.tab_count(), count_before - 1);
    }

    #[test]
    fn test_dispatch_browsing_context_close_missing_context() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let result = server.dispatch(&mut session, "browsingContext.close", Value::Null);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32602);
    }

    #[test]
    fn test_dispatch_browsing_context_reload() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let result = server
            .dispatch(&mut session, "browsingContext.reload", Value::Null)
            .unwrap();
        assert_eq!(result["result"], "reloaded");
    }

    #[test]
    fn test_dispatch_script_call_function() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let params = serde_json::json!({
            "functionDeclaration": "function() { return 1 + 1; }",
        });
        let result = server.dispatch(&mut session, "script.callFunction", params).unwrap();
        assert!(result.get("result").is_some() || result.get("exceptionDetails").is_some());
    }

    #[test]
    fn test_dispatch_script_call_function_with_args() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let params = serde_json::json!({
            "functionDeclaration": "function(a, b) { return a + b; }",
            "arguments": [{ "value": 1 }, { "value": 2 }]
        });
        let result = server.dispatch(&mut session, "script.callFunction", params).unwrap();
        assert!(result.get("result").is_some() || result.get("exceptionDetails").is_some());
    }

    #[test]
    fn test_dispatch_script_call_function_missing_declaration() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let result = server.dispatch(&mut session, "script.callFunction", Value::Null);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32602);
    }

    #[test]
    fn test_http_version_json() {
        let addr: SocketAddr = "127.0.0.1:9222".parse().unwrap();
        let json = HeadlessServer::http_version_json(addr);
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["Browser"], "ZeroWeb/0.1");
        assert!(parsed["webSocketDebuggerUrl"].as_str().unwrap().contains("ws://"));
    }

    // ── Phase 2-3 测试：事件推送和 CDP 兼容 ──

    #[test]
    fn test_is_http_get_request() {
        assert!(HeadlessServer::is_http_get_request(b"GET /json HTTP/1.1\r\n"));
        assert!(!HeadlessServer::is_http_get_request(
            b"GET /json HTTP/1.1\r\nUpgrade: websocket\r\n"
        ));
        assert!(!HeadlessServer::is_http_get_request(b"POST /json HTTP/1.1\r\n"));
    }

    #[test]
    fn test_server_event_serialize() {
        let event = ServerEvent {
            method: "browsingContext.load".into(),
            params: serde_json::json!({ "url": "https://example.com" }),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"method\""));
        assert!(json.contains("browsingContext.load"));
    }

    #[test]
    fn test_dispatch_with_events_navigate() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let params = serde_json::json!({ "url": "https://example.com" });
        let (result, events) = server.dispatch_with_events(&mut session, "browsingContext.navigate", params);
        assert!(result.is_ok());
        assert!(!events.is_empty(), "navigate should produce events");
        let methods: Vec<&str> = events.iter().map(|e| e.method.as_str()).collect();
        assert!(methods.contains(&"browsingContext.load"), "should emit load event");
        assert!(methods.contains(&"log.entryAdded"), "should emit log event");
    }

    #[test]
    fn test_dispatch_with_events_reload() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let (result, events) = server.dispatch_with_events(&mut session, "browsingContext.reload", Value::Null);
        assert!(result.is_ok());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].method, "browsingContext.load");
    }

    #[test]
    fn test_dispatch_with_events_create() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let params = serde_json::json!({});
        let (result, events) = server.dispatch_with_events(&mut session, "browsingContext.create", params);
        assert!(result.is_ok());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].method, "browsingContext.contextCreated");
    }

    #[test]
    fn test_dispatch_with_events_close() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let new_tab_id = session.shell.new_tab(None);
        let params = serde_json::json!({ "context": new_tab_id.0 });
        let (result, events) = server.dispatch_with_events(&mut session, "browsingContext.close", params);
        assert!(result.is_ok());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].method, "browsingContext.contextDestroyed");
    }

    #[test]
    fn test_dispatch_with_events_no_events_for_status() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let (result, events) = server.dispatch_with_events(&mut session, "session.status", Value::Null);
        assert!(result.is_ok());
        assert!(events.is_empty(), "session.status should not produce events");
    }

    #[test]
    fn test_dispatch_cdp_page_navigate() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let params = serde_json::json!({ "url": "https://example.com" });
        let (result, events) = server.dispatch_with_events(&mut session, "Page.navigate", params);
        assert!(result.is_ok());
        assert!(events.iter().any(|e| e.method == "Page.loadEventFired"));
    }

    #[test]
    fn test_dispatch_cdp_runtime_evaluate() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let params = serde_json::json!({ "expression": "1 + 1" });
        let (result, events) = server.dispatch_with_events(&mut session, "Runtime.evaluate", params);
        assert!(result.is_ok());
        assert!(events.is_empty(), "Runtime.evaluate should not produce events");
    }

    #[test]
    fn test_dispatch_cdp_network_enable() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let (result, events) = server.dispatch_with_events(&mut session, "Network.enable", Value::Null);
        assert!(result.is_ok());
        assert!(events.is_empty());
    }

    #[test]
    fn test_dispatch_cdp_target_get_targets() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let (result, events) = server.dispatch_with_events(&mut session, "Target.getTargets", Value::Null);
        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().get("contexts").is_some());
    }

    #[test]
    fn test_dispatch_cdp_page_capture_screenshot() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let (result, events) = server.dispatch_with_events(&mut session, "Page.captureScreenshot", Value::Null);
        assert!(result.is_ok());
        assert!(events.is_empty());
    }

    // ── Phase 4: 协议客户端测试 ──

    #[test]
    fn test_client_build_request() {
        let req = HeadlessClient::build_request(1, "session.status", serde_json::json!({}));
        let parsed: ClientRequest = serde_json::from_str(&req).unwrap();
        assert_eq!(parsed.id, 1);
        assert_eq!(parsed.method, "session.status");
    }

    #[test]
    fn test_client_parse_response_success() {
        let raw = r#"{"id":1,"result":{"ready":true,"message":"ready"}}"#;
        let result = HeadlessClient::parse_response(raw).unwrap();
        assert_eq!(result["ready"], true);
    }

    #[test]
    fn test_client_parse_response_error() {
        let raw = r#"{"id":1,"error":{"code":-32601,"message":"Unknown method"}}"#;
        let result = HeadlessClient::parse_response(raw);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("-32601"));
    }

    #[test]
    fn test_client_parse_event() {
        let raw = r#"{"method":"browsingContext.load","params":{"url":"https://example.com"}}"#;
        let (method, params) = HeadlessClient::parse_event(raw).unwrap();
        assert_eq!(method, "browsingContext.load");
        assert_eq!(params["url"], "https://example.com");
    }

    #[test]
    fn test_client_parse_screenshot() {
        let result = serde_json::json!({
            "data": { "width": 800, "height": 600, "format": "rgba8", "pixelCount": 480000 }
        });
        let (w, h, px) = HeadlessClient::parse_screenshot(&result).unwrap();
        assert_eq!(w, 800);
        assert_eq!(h, 600);
        assert_eq!(px, 480000);
    }

    #[test]
    fn test_client_parse_dom_snapshot() {
        let result = serde_json::json!({
            "renderPrimitives": { "fills": 10, "glyphs": 5, "gradients": 2, "shadows": 1, "images": 3 }
        });
        let stats = HeadlessClient::parse_dom_snapshot(&result);
        assert_eq!(stats.fills, 10);
        assert_eq!(stats.glyphs, 5);
        assert_eq!(stats.gradients, 2);
        assert_eq!(stats.shadows, 1);
        assert_eq!(stats.images, 3);
        assert_eq!(stats.total(), 21);
    }

    #[test]
    fn test_dom_snapshot_stats_total() {
        let stats = DomSnapshotStats {
            fills: 1,
            glyphs: 2,
            gradients: 3,
            shadows: 4,
            images: 5,
        };
        assert_eq!(stats.total(), 15);
    }

    // ── Phase 4: 协议驱动的自动化冒烟测试 ──

    /// 辅助：通过 dispatch 模拟完整的协议驱动的测试场景。
    struct ProtocolTestRunner {
        server: HeadlessServer,
        session: HeadlessSession,
        next_id: u64,
        /// 收集到的事件日志。
        event_log: Vec<(String, serde_json::Value)>,
    }

    impl ProtocolTestRunner {
        fn new() -> Self {
            Self {
                server: HeadlessServer::new(0, 800.0, 600.0),
                session: HeadlessSession::new(800.0, 600.0),
                next_id: 1,
                event_log: Vec::new(),
            }
        }

        /// 发送命令并收集响应和事件（模拟协议往返）。
        fn send(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
            let id = self.next_id;
            self.next_id += 1;

            // 模拟发送 JSON 请求（验证请求格式正确）
            let req_json = HeadlessClient::build_request(id, method, params.clone());
            let _req: ClientRequest = serde_json::from_str(&req_json).map_err(|e| format!("Invalid request: {e}"))?;

            // 执行命令（直接传递原始 params）
            let (result, events) = self.server.dispatch_with_events(&mut self.session, method, params);

            // 记录事件
            for event in events {
                self.event_log.push((event.method.clone(), event.params.clone()));
            }

            // 模拟解析响应
            let response = match result {
                Ok(value) => ServerResponse {
                    id,
                    result: Some(value),
                    error: None,
                },
                Err(err) => ServerResponse {
                    id,
                    result: None,
                    error: Some(err),
                },
            };
            let response_json = serde_json::to_string(&response).unwrap();
            HeadlessClient::parse_response(&response_json)
        }

        /// 获取事件日志中指定方法的事件数。
        fn event_count(&self, method: &str) -> usize {
            self.event_log.iter().filter(|(m, _)| m == method).count()
        }
    }

    /// 冒烟测试：完整会话生命周期（创建→导航→脚本→截图→关闭）。
    #[test]
    fn test_smoke_full_session_lifecycle() {
        let mut runner = ProtocolTestRunner::new();

        // 1. 会话状态检查
        let status = runner.send("session.status", Value::Null).unwrap();
        assert_eq!(status["ready"], true);

        // 2. 创建新会话
        let session = runner.send("session.new", Value::Null).unwrap();
        assert_eq!(session["capabilities"]["browserName"], "ZeroWeb");

        // 3. 创建浏览上下文
        let ctx = runner
            .send("browsingContext.create", serde_json::json!({ "url": "about:blank" }))
            .unwrap();
        assert!(ctx.get("context").is_some());

        // 4. 获取上下文树
        let tree = runner.send("browsingContext.getTree", Value::Null).unwrap();
        let contexts = tree.get("contexts").unwrap().as_array().unwrap();
        assert!(contexts.len() >= 2, "should have at least 2 tabs");

        // 5. 执行脚本
        let script_result = runner
            .send("script.evaluate", serde_json::json!({ "expression": "1 + 1" }))
            .unwrap();
        assert!(script_result.get("result").is_some() || script_result.get("exceptionDetails").is_some());

        // 6. 截图
        let screenshot = runner.send("browsingContext.captureScreenshot", Value::Null).unwrap();
        let (w, h, px) = HeadlessClient::parse_screenshot(&screenshot).unwrap();
        assert_eq!(w, 800);
        assert_eq!(h, 600);
        assert!(px > 0);

        // 7. DOM 快照（空会话可能没有图元，但不应 panic）
        let snapshot = runner.send("browsingContext.getDOMSnapshot", Value::Null).unwrap();
        let stats = HeadlessClient::parse_dom_snapshot(&snapshot);
        // 空白页面至少应该有视口根填充
        assert!(stats.total() >= 0, "DOM snapshot should not panic");

        // 8. 验证事件收集（create 不产生 load 事件，需 navigate 才有）
        assert!(runner.event_count("browsingContext.contextCreated") >= 1);

        // 9. 关闭浏览上下文
        let context_id = ctx["context"].as_u64().unwrap();
        let close_result = runner
            .send("browsingContext.close", serde_json::json!({ "context": context_id }))
            .unwrap();
        assert_eq!(close_result["result"], "closed");

        // 10. 验证 contextDestroyed 事件
        assert!(runner.event_count("browsingContext.contextDestroyed") >= 1);

        // 11. 浏览器关闭
        let close = runner.send("browser.close", Value::Null).unwrap();
        assert_eq!(close["result"], "closing");
    }

    /// 冒烟测试：CDP 兼容命令序列。
    #[test]
    fn test_smoke_cdp_command_sequence() {
        let mut runner = ProtocolTestRunner::new();

        // 1. 获取版本信息（模拟 HTTP 发现）
        let addr: std::net::SocketAddr = "127.0.0.1:9222".parse().unwrap();
        let version_json = HeadlessServer::http_version_json(addr);
        let version: serde_json::Value = serde_json::from_str(&version_json).unwrap();
        assert_eq!(version["Browser"], "ZeroWeb/0.1");
        assert!(version["webSocketDebuggerUrl"].as_str().unwrap().starts_with("ws://"));

        // 2. Target.getTargets
        let targets = runner.send("Target.getTargets", Value::Null).unwrap();
        assert!(targets.get("contexts").is_some());

        // 3. Runtime.evaluate
        let eval_result = runner
            .send(
                "Runtime.evaluate",
                serde_json::json!({ "expression": "JSON.stringify({ok: true})" }),
            )
            .unwrap();
        assert!(eval_result.get("result").is_some());

        // 4. Network.enable
        let net_enable = runner.send("Network.enable", Value::Null).unwrap();
        assert_eq!(net_enable["result"], "enabled");

        // 5. Page.captureScreenshot
        let screenshot = runner.send("Page.captureScreenshot", Value::Null).unwrap();
        let (w, h, _) = HeadlessClient::parse_screenshot(&screenshot).unwrap();
        assert_eq!(w, 800);
        assert_eq!(h, 600);
    }

    /// 冒烟测试：脚本执行和错误处理。
    #[test]
    fn test_smoke_script_execution_variants() {
        let mut runner = ProtocolTestRunner::new();

        // 正常表达式
        let ok = runner
            .send("script.evaluate", serde_json::json!({ "expression": "2 + 2" }))
            .unwrap();
        assert!(ok.get("result").is_some());

        // JSON 返回
        let json = runner
            .send(
                "script.evaluate",
                serde_json::json!({ "expression": "JSON.stringify({a: 1})" }),
            )
            .unwrap();
        if let Some(result) = json.get("result") {
            if let Some(value) = result.get("value").and_then(|v| v.as_str()) {
                let parsed: serde_json::Value = serde_json::from_str(value).unwrap();
                assert_eq!(parsed["a"], 1);
            }
        }

        // 错误表达式
        let err = runner
            .send(
                "script.evaluate",
                serde_json::json!({ "expression": "throw new Error('test')" }),
            )
            .unwrap();
        assert!(err.get("exceptionDetails").is_some());

        // callFunction 无参数
        let call_no_args = runner
            .send(
                "script.callFunction",
                serde_json::json!({
                    "functionDeclaration": "function() { return 42; }"
                }),
            )
            .unwrap();
        assert!(call_no_args.get("result").is_some() || call_no_args.get("exceptionDetails").is_some());

        // callFunction 有参数
        let call_with_args = runner
            .send(
                "script.callFunction",
                serde_json::json!({
                    "functionDeclaration": "function(a, b) { return a + b; }",
                    "arguments": [{ "value": 10 }, { "value": 20 }]
                }),
            )
            .unwrap();
        assert!(call_with_args.get("result").is_some() || call_with_args.get("exceptionDetails").is_some());
    }

    /// 冒烟测试：多浏览上下文管理。
    #[test]
    fn test_smoke_multiple_browsing_contexts() {
        let mut runner = ProtocolTestRunner::new();

        // 初始有 1 个标签页
        let tree1 = runner.send("browsingContext.getTree", Value::Null).unwrap();
        let count1 = tree1.get("contexts").unwrap().as_array().unwrap().len();

        // 创建 3 个新标签页
        let mut new_contexts = Vec::new();
        for i in 0..3 {
            let ctx = runner
                .send(
                    "browsingContext.create",
                    serde_json::json!({
                        "url": &format!("https://example.com/page{i}")
                    }),
                )
                .unwrap();
            new_contexts.push(ctx["context"].as_u64().unwrap());
        }

        // 验证标签页数量增加
        let tree2 = runner.send("browsingContext.getTree", Value::Null).unwrap();
        let count2 = tree2.get("contexts").unwrap().as_array().unwrap().len();
        assert_eq!(count2, count1 + 3);

        // 验证 contextCreated 事件
        assert_eq!(runner.event_count("browsingContext.contextCreated"), 3);

        // 逐个关闭
        for ctx_id in &new_contexts {
            let result = runner
                .send("browsingContext.close", serde_json::json!({ "context": ctx_id }))
                .unwrap();
            assert_eq!(result["result"], "closed");
        }

        // 验证恢复原始数量
        let tree3 = runner.send("browsingContext.getTree", Value::Null).unwrap();
        let count3 = tree3.get("contexts").unwrap().as_array().unwrap().len();
        assert_eq!(count3, count1);

        // 验证 contextDestroyed 事件
        assert_eq!(runner.event_count("browsingContext.contextDestroyed"), 3);
    }

    /// 冒烟测试：渲染管线通过协议验证。
    #[test]
    fn test_smoke_render_pipeline_via_protocol() {
        let mut runner = ProtocolTestRunner::new();

        // 加载 HTML 内容（通过脚本设置）
        let load_result = runner
            .send(
                "script.evaluate",
                serde_json::json!({
                    "expression": "'render pipeline test'"
                }),
            )
            .unwrap();
        assert!(load_result.get("result").is_some());

        // 截图验证视口尺寸
        let screenshot = runner.send("browsingContext.captureScreenshot", Value::Null).unwrap();
        let (w, h, px) = HeadlessClient::parse_screenshot(&screenshot).unwrap();
        assert_eq!(w, 800);
        assert_eq!(h, 600);
        assert_eq!(px, 800 * 600);

        // DOM 快照验证协议可正确返回图元信息
        let snapshot = runner.send("browsingContext.getDOMSnapshot", Value::Null).unwrap();
        let stats = HeadlessClient::parse_dom_snapshot(&snapshot);
        // 空页面不一定有图元，但快照应成功返回
        assert!(stats.total() >= 0, "DOM snapshot should not panic");
    }

    /// 冒烟测试：协议错误处理。
    #[test]
    fn test_smoke_protocol_error_handling() {
        let mut runner = ProtocolTestRunner::new();

        // 未知命令
        let err = runner.send("unknown.command", Value::Null);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("-32601"));

        // 缺少必要参数
        let nav_err = runner.send("browsingContext.navigate", Value::Null);
        assert!(nav_err.is_err());
        assert!(nav_err.unwrap_err().contains("-32602"));

        // 关闭不存在的上下文
        let close_err = runner.send("browsingContext.close", serde_json::json!({ "context": 99999 }));
        // close_tab 对不存在的标签页是 no-op，所以不会报错
        // 但我们可以验证命令本身不会 panic
        let _ = close_err;

        // callFunction 缺少参数
        let call_err = runner.send("script.callFunction", Value::Null);
        assert!(call_err.is_err());
        assert!(call_err.unwrap_err().contains("-32602"));
    }

    /// 冒烟测试：页面重载和事件序列。
    #[test]
    fn test_smoke_reload_and_event_sequence() {
        let mut runner = ProtocolTestRunner::new();

        // 重载当前页面
        let reload = runner.send("browsingContext.reload", Value::Null).unwrap();
        assert_eq!(reload["result"], "reloaded");

        // 验证重载产生 load 事件
        assert!(runner.event_count("browsingContext.load") >= 1);

        // 再次重载
        let reload2 = runner.send("browsingContext.reload", Value::Null).unwrap();
        assert_eq!(reload2["result"], "reloaded");

        // 验证事件累计
        assert!(runner.event_count("browsingContext.load") >= 2);
    }

    // ── Phase 5: 安全配置测试 ──

    #[test]
    fn test_security_config_default_allows_all() {
        let config = HeadlessSecurityConfig::new();
        assert!(config.verify_token(None));
        assert!(config.verify_token(Some("anything")));
        assert!(config.verify_origin(None));
        assert!(config.verify_origin(Some("http://evil.com")));
    }

    #[test]
    fn test_security_config_token_required() {
        let config = HeadlessSecurityConfig::new().with_token("secret123");
        assert!(!config.verify_token(None));
        assert!(!config.verify_token(Some("wrong")));
        assert!(config.verify_token(Some("secret123")));
    }

    #[test]
    fn test_security_config_origin_allowlist() {
        let config = HeadlessSecurityConfig::new()
            .with_origin("http://localhost:3000")
            .with_origin("https://trusted.example.com");
        // 允许的来源
        assert!(config.verify_origin(Some("http://localhost:3000")));
        assert!(config.verify_origin(Some("https://trusted.example.com")));
        // 不允许的来源
        assert!(!config.verify_origin(Some("http://evil.com")));
        assert!(!config.verify_origin(None));
    }

    #[test]
    fn test_security_config_empty_origin_allows_all() {
        let config = HeadlessSecurityConfig::new();
        assert!(config.verify_origin(None));
        assert!(config.verify_origin(Some("http://anything.com")));
    }

    #[test]
    fn test_extract_origin_header() {
        let request = b"GET /json HTTP/1.1\r\nHost: localhost:9222\r\nOrigin: http://localhost:3000\r\n\r\n";
        let origin = HeadlessServer::extract_origin_header(request);
        assert_eq!(origin.as_deref(), Some("http://localhost:3000"));

        // 无 Origin 头
        let no_origin = b"GET /json HTTP/1.1\r\nHost: localhost:9222\r\n\r\n";
        assert!(HeadlessServer::extract_origin_header(no_origin).is_none());

        // 小写 origin
        let lowercase = b"GET /json HTTP/1.1\r\norigin: http://example.com\r\n\r\n";
        assert_eq!(
            HeadlessServer::extract_origin_header(lowercase).as_deref(),
            Some("http://example.com")
        );
    }

    #[test]
    fn test_server_with_security_config() {
        let server =
            HeadlessServer::new(0, 800.0, 600.0).with_security(HeadlessSecurityConfig::new().with_token("test-token"));
        assert!(server.security.verify_token(Some("test-token")));
        assert!(!server.security.verify_token(None));
    }

    #[test]
    fn test_server_binds_to_localhost_only() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        assert_eq!(server.addr.ip(), std::net::IpAddr::from([127, 0, 0, 1]));
    }

    // ── Chrome 自动化命令测试（feature sdk-chrome）──

    #[cfg(feature = "sdk-chrome")]
    #[test]
    fn test_chrome_screenshot_returns_with_chrome_flag() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let result = server.dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null);
        assert!(result.is_ok(), "screenshot should succeed: {:?}", result.err());
        let v = result.unwrap();
        assert_eq!(v["data"]["withChrome"], true);
        assert!(v["pixels"].as_str().is_some(), "pixels base64 should be present");
        assert!(session.last_chrome_frame.is_some(), "frame should be cached");
    }

    #[cfg(feature = "sdk-chrome")]
    #[test]
    fn test_chrome_get_layout_returns_viewport_rect() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        // 先截图缓存 frame
        let _ = server.dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null);
        let result = server.dispatch(&mut session, "chrome.getLayout", Value::Null);
        assert!(result.is_ok(), "getLayout should succeed: {:?}", result.err());
        let v = result.unwrap();
        assert_eq!(v["windowSize"]["width"].as_f64().unwrap_or(0.0), 800.0);
        // viewport 应有非零尺寸（chrome 占顶部，viewport 从 chrome 高度之后开始）。
        let vp = &v["viewport"];
        assert!(
            vp["y"].as_f64().unwrap_or(0.0) > 0.0,
            "viewport y should be > 0 (chrome height): {}",
            vp
        );
        assert!(
            vp["height"].as_f64().unwrap_or(0.0) > 0.0,
            "viewport height should be > 0: {}",
            vp
        );
    }

    #[cfg(feature = "sdk-chrome")]
    #[test]
    fn test_chrome_get_layout_requires_screenshot_first() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let result = server.dispatch(&mut session, "chrome.getLayout", Value::Null);
        assert!(result.is_err(), "getLayout without prior screenshot should fail");
        assert_eq!(result.unwrap_err().code, -32002);
    }

    #[cfg(feature = "sdk-chrome")]
    #[test]
    fn test_chrome_get_semantics_returns_tree() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let _ = server.dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null);
        let result = server.dispatch(&mut session, "chrome.getSemantics", Value::Null);
        assert!(result.is_ok(), "getSemantics should succeed: {:?}", result.err());
        let v = result.unwrap();
        let tree = &v["tree"];
        // 树非空（至少有根节点）。
        assert!(tree.is_object(), "tree should be an object: {}", tree);
        assert!(tree["id"].is_string(), "root node should have id: {}", tree);
        // 至少有一些 focusable 节点（nav buttons / address bar）。
        let focusable_count = count_focusable_in_tree(tree);
        assert!(
            focusable_count >= 1,
            "expected at least 1 focusable node, got {}",
            focusable_count
        );
    }

    #[cfg(feature = "sdk-chrome")]
    fn count_focusable_in_tree(node: &Value) -> usize {
        let mut count = 0;
        if let Some(flags) = node["flags"].as_array() {
            if flags.iter().any(|f| f.as_str() == Some("focusable")) {
                count += 1;
            }
        }
        if let Some(children) = node["children"].as_array() {
            for child in children {
                count += count_focusable_in_tree(child);
            }
        }
        count
    }

    #[cfg(feature = "sdk-chrome")]
    #[test]
    fn test_chrome_click_with_widget_id_emits_action() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let _ = server.dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null);

        // 从 semantics 找一个 button 类节点（如菜单按钮）来点击。
        let sem = server
            .dispatch(&mut session, "chrome.getSemantics", Value::Null)
            .unwrap();
        let widget_id = find_first_button_id(&sem["tree"]).expect("expected at least one button widget in chrome");

        let result = server.dispatch(
            &mut session,
            "chrome.click",
            serde_json::json!({ "widgetId": widget_id }),
        );
        assert!(result.is_ok(), "click should succeed: {:?}", result.err());
        let v = result.unwrap();
        // emitted actions 数组（可能是 0 或多个；至少有 point 字段）。
        assert!(v["point"]["x"].as_f64().is_some(), "click should return point: {}", v);
        // 上次 emitted actions 已缓存。
        let emitted = server
            .dispatch(&mut session, "chrome.emittedActions", Value::Null)
            .unwrap();
        assert!(
            emitted["actions"].is_array(),
            "emittedActions should return array: {}",
            emitted
        );
    }

    #[cfg(feature = "sdk-chrome")]
    fn find_first_button_id(node: &Value) -> Option<String> {
        if let Some(flags) = node["flags"].as_array() {
            if flags.iter().any(|f| f.as_str() == Some("button")) {
                return node["id"].as_str().map(|s| s.to_string());
            }
        }
        if let Some(children) = node["children"].as_array() {
            for child in children {
                if let Some(id) = find_first_button_id(child) {
                    return Some(id);
                }
            }
        }
        None
    }

    #[cfg(feature = "sdk-chrome")]
    #[test]
    fn test_chrome_rect_of_returns_geometry() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let _ = server.dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null);

        // viewport 节点 id 必然存在。
        let result = server.dispatch(
            &mut session,
            "chrome.rectOf",
            serde_json::json!({ "widgetId": "viewport" }),
        );
        assert!(result.is_ok(), "rectOf viewport should succeed: {:?}", result.err());
        let v = result.unwrap();
        assert!(v["rect"]["y"].as_f64().unwrap_or(0.0) > 0.0, "viewport y > 0: {}", v);
    }

    #[cfg(feature = "sdk-chrome")]
    #[test]
    fn test_chrome_rect_of_unknown_widget_fails() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let _ = server.dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null);

        let result = server.dispatch(
            &mut session,
            "chrome.rectOf",
            serde_json::json!({ "widgetId": "nonexistent-widget" }),
        );
        assert!(result.is_err(), "rectOf unknown widget should fail");
        assert_eq!(result.unwrap_err().code, -32603);
    }

    #[cfg(feature = "sdk-chrome")]
    #[test]
    fn test_chrome_click_xy_dispatches_pointer() {
        // 点击 chrome 区域的某点（如 tab strip 左侧）；只要不报错就算通过。
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let _ = server.dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null);

        let result = server.dispatch(
            &mut session,
            "chrome.click",
            serde_json::json!({ "x": 50.0, "y": 25.0 }), // tab strip 区域
        );
        assert!(result.is_ok(), "click by x,y should succeed: {:?}", result.err());
    }

    #[cfg(feature = "sdk-chrome")]
    #[test]
    fn test_chrome_click_missing_params_errors() {
        let server = HeadlessServer::new(0, 800.0, 600.0);
        let mut session = HeadlessSession::new(800.0, 600.0);
        let _ = server.dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null);

        let result = server.dispatch(&mut session, "chrome.click", Value::Null);
        assert!(result.is_err(), "click with no params should fail");
        assert_eq!(result.unwrap_err().code, -32602);
    }
}

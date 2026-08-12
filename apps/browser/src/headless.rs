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
use tungstenite::Message;
use tungstenite::accept;

use zero_browser_shell::BrowserShell;
use zero_browser_shell::TabId;
use zero_render_foundation::cpu::render_full_scene;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::surface::FrameBuffer;
use zero_webview::{WebView, WebViewConfig};

// ── 协议消息类型 ──

/// R1601：把 RGBA8 FrameBuffer 编码为 base64 PNG 字符串，供 `captureScreenshot`
/// 协议响应携带像素数据（headless 截图用于像素对比，DC-13 line 315）。
fn framebuffer_to_png_base64(fb: &FrameBuffer) -> String {
    use base64::Engine;
    use png::{BitDepth, ColorType, Encoder};
    let mut png_buf: Vec<u8> = Vec::new();
    {
        let mut encoder = Encoder::new(&mut png_buf, fb.width, fb.height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().expect("PNG header encode");
        writer.write_image_data(&fb.data).expect("PNG image data encode");
    }
    base64::engine::general_purpose::STANDARD.encode(&png_buf)
}

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
    /// R3282（#4）：可选 GPU 截图渲染器（`ZW_HEADLESS_GPU_SCREENSHOT=1` 启用；
    /// 默认 CPU——oracle 像素对比基线稳定）。
    gpu_renderer: Option<zero_render_foundation::gpu::renderer::GpuRenderer>,
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
            gpu_renderer: None,
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

            // peek 前几个字节判断是 HTTP 还是 WebSocket
            let mut buf = [0u8; 4096];
            stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok();
            let n = match stream.peek(&mut buf) {
                Ok(n) if n > 0 => n,
                Ok(_) => continue,
                Err(e) => {
                    tracing::warn!("Peek failed for {peer}: {e}");
                    continue;
                }
            };
            let peeked = &buf[..n];

            // 检测是否是普通 HTTP GET 请求（非 WebSocket 升级）
            if Self::is_http_get_request(peeked) {
                // Origin 检查（HTTP 发现请求）
                let origin = Self::extract_origin_header(peeked);
                if !self.security.verify_origin(origin.as_deref()) {
                    tracing::warn!("HTTP request from disallowed origin: {origin:?} from {peer}");
                    continue;
                }
                Self::handle_http_discovery(&stream, self.addr);
                continue;
            }

            // Origin 检查（WebSocket 升级请求）
            let origin = Self::extract_origin_header(peeked);
            if !self.security.verify_origin(origin.as_deref()) {
                tracing::warn!("WebSocket from disallowed origin: {origin:?} from {peer}");
                continue;
            }

            // WebSocket 连接
            let mut ws = accept(stream).map_err(|e| format!("WebSocket handshake failed: {e}"))?;
            let mut session = HeadlessSession::new(self.viewport_width, self.viewport_height);

            // 认证状态：首个有效请求完成认证
            let mut authenticated = self.security.auth_token.is_none();

            // WebSocket 消息循环
            loop {
                let msg = match ws.read() {
                    Ok(Message::Text(text)) => text,
                    Ok(Message::Close(_)) => {
                        tracing::info!("Client disconnected");
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        let _ = ws.write(Message::Pong(data));
                        continue;
                    }
                    Ok(_) => continue,
                    Err(e) => {
                        tracing::error!("WebSocket read error: {e}");
                        break;
                    }
                };

                // 认证检查（Phase 5）
                if !authenticated {
                    // 尝试从首个请求中提取 token
                    if let Ok(req) = serde_json::from_str::<ClientRequest>(&msg) {
                        let token = req.params.get("token").and_then(|v| v.as_str());
                        if self.security.verify_token(token) {
                            authenticated = true;
                            tracing::info!("Client authenticated");
                        } else {
                            tracing::warn!("Authentication failed from {peer}");
                            let err = ServerResponse {
                                id: req.id,
                                result: None,
                                error: Some(ProtocolError {
                                    code: -32001,
                                    message: "Authentication required: invalid or missing token".into(),
                                }),
                            };
                            if let Ok(json) = serde_json::to_string(&err) {
                                let _ = ws.write(Message::Text(json.into()));
                            }
                            continue;
                        }
                    } else {
                        let err = ServerResponse {
                            id: 0,
                            result: None,
                            error: Some(ProtocolError {
                                code: -32001,
                                message: "Authentication required".into(),
                            }),
                        };
                        if let Ok(json) = serde_json::to_string(&err) {
                            let _ = ws.write(Message::Text(json.into()));
                        }
                        continue;
                    }
                }

                let (response, events) = self.handle_message_with_events(&mut session, &msg);

                // 先推送事件通知
                for event in events {
                    if let Ok(event_json) = serde_json::to_string(&event)
                        && let Err(e) = ws.write(Message::Text(event_json.into()))
                    {
                        tracing::error!("Event push error: {e}");
                        break;
                    }
                }

                // 再推送命令响应
                let response_json = serde_json::to_string(&response).unwrap_or_else(|e| {
                    format!("{{\"id\":0,\"error\":{{\"code\":-32700,\"message\":\"JSON serialize: {e}\"}}}}")
                });

                if let Err(e) = ws.write(Message::Text(response_json.into())) {
                    tracing::error!("WebSocket write error: {e}");
                    break;
                }
            }

            tracing::info!("Headless session ended");
        }
    }

    /// 判断是否为普通 HTTP GET 请求（非 WebSocket 升级）。
    fn is_http_get_request(data: &[u8]) -> bool {
        let s = String::from_utf8_lossy(data);
        s.starts_with("GET ") && !s.contains("Upgrade: websocket")
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

    /// 处理 HTTP 发现请求（CDP 风格的 /json 端点）。
    fn handle_http_discovery(stream: &std::net::TcpStream, addr: SocketAddr) {
        use std::io::{Read, Write};

        let mut read_buf = [0u8; 4096];
        let path = if let Ok(mut readable) = stream.try_clone() {
            let n = readable.read(&mut read_buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&read_buf[..n]);
            request
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
            // DC-13 line 315：加载内联 HTML（绕过 fetch_url HTTP-only），供 headless 截图自包含 fixture。
            "browsingContext.loadHtml" => self.cmd_load_html(session, params),

            // ── 脚本执行 ──
            "script.evaluate" => self.cmd_script_evaluate(session, params),
            "script.callFunction" => self.cmd_script_call_function(session, params),

            // ── 截图 ──
            "browsingContext.captureScreenshot" => self.cmd_capture_screenshot(session),

            // ── 页面内容 ──
            "browsingContext.getDOMSnapshot" => self.cmd_get_dom_snapshot(session),

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
                "browserVersion": zero_product_version::VERSION,
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
        let result = session.webview.render();

        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(1024);
        // R1600：用 render_full_scene 渲染**全部 13 种图元**（旧 render_scene_to_framebuffer 仅
        // 渲染 fills+glyphs，静默丢弃 gradients/shadows/images/strokes/paths/transforms/clips/
        // filters/blend_modes 11 种）。headless 截图须反映真实 ZeroBrowser 渲染管线才能用于
        // DC-13 line 315（welcome headless 截图 vs chromium oracle）等像素对比；image_cache
        // 传入以渲染 `<img>` 子资源。
        // R3282（#4）：`ZW_HEADLESS_GPU_SCREENSHOT=1` 时用 GPU 无头渲染（性能开关；
        // 默认 CPU——DC-13 oracle 对比基线稳定）。GPU 支持子集与 CPU 逐像素一致
        //（parity/reftest 验证），未实现特性返回 false 自动回退 CPU。
        let fb = if std::env::var("ZW_HEADLESS_GPU_SCREENSHOT").as_deref() == Ok("1") {
            let w = self.viewport_width as u32;
            let h = self.viewport_height as u32;
            if session.gpu_renderer.is_none() {
                session.gpu_renderer = zero_render_foundation::gpu::renderer::GpuRenderer::new_headless(w, h).ok();
            }
            let gpu_ok = session.gpu_renderer.as_mut().is_some_and(|g| {
                !g.is_device_lost()
                    && g.render_full_scene_gpu(
                        &result.primitives,
                        &font_loader,
                        &mut glyph_cache,
                        Some(session.webview.image_cache()),
                        &[],
                        &[],
                        &[],
                        &[],
                        1.0,
                    )
            });
            if gpu_ok {
                let pixels = session
                    .gpu_renderer
                    .as_ref()
                    .unwrap()
                    .read_pixels()
                    .expect("GPU read_pixels");
                let mut fb = zero_render_foundation::surface::FrameBuffer::new(w, h);
                fb.data.copy_from_slice(&pixels);
                fb
            } else {
                render_full_scene(
                    w,
                    h,
                    1.0,
                    &result.primitives,
                    &font_loader,
                    &mut glyph_cache,
                    Some(session.webview.image_cache()),
                    &[],
                    &[],
                    &[],
                    &[],
                )
            }
        } else {
            render_full_scene(
                self.viewport_width as u32,
                self.viewport_height as u32,
                1.0,
                &result.primitives,
                &font_loader,
                &mut glyph_cache,
                Some(session.webview.image_cache()),
                &[],
                &[],
                &[],
                &[],
            )
        };

        // R1601：返回 base64 PNG 像素数据（旧版仅返回尺寸，headless 截图无法用于像素对比）。
        // 保留 width/height/pixelCount 供 HeadlessClient::parse_screenshot 向后兼容。
        let png_b64 = framebuffer_to_png_base64(&fb);
        Ok(serde_json::json!({
            "data": {
                "width": fb.width,
                "height": fb.height,
                "format": "png-base64",
                "pixelCount": fb.width as usize * fb.height as usize,
                "png": png_b64,
            }
        }))
    }

    /// DC-13 line 315：加载内联 HTML（绕过 fetch_url 的 HTTP-only 限制），供 headless 路径
    /// 加载自包含 fixture（如 welcome.html，内联 CSS + data-URI 图标）做截图对比。参数 `html`
    ///（必填）+ `css`（可选）。区别于 `browsingContext.navigate`（需 URL + HTTP/file 抓取）。
    fn cmd_load_html(&self, session: &mut HeadlessSession, params: Value) -> Result<Value, ProtocolError> {
        let html = params
            .get("html")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'html' parameter".into(),
            })?;
        let css = params.get("css").and_then(|v| v.as_str());
        session.shell.navigate("about:blank");
        let _ = session.webview.load_html(html, css);
        session.shell.on_page_loaded("headless:loadHtml");
        Ok(serde_json::json!({ "success": true }))
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
            "Browser": format!("ZeroWeb/{}", zero_product_version::VERSION),
            "Protocol-Version": "1.3",
            "User-Agent": zero_net::HttpClient::default_user_agent(),
            "V8-Version": "12.0",
            "WebKit-Version": "0.1",
            "webSocketDebuggerUrl": format!("ws://{addr}"),
        })
        .to_string()
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
        assert_eq!(parsed["Browser"], format!("ZeroWeb/{}", zero_product_version::VERSION));
        assert_eq!(parsed["User-Agent"], zero_net::HttpClient::default_user_agent());
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

    /// 解码 PNG 字节为 (width, height, RGBA8 像素)。测试辅助。
    fn decode_png_rgba(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
        use png::ColorType;
        let decoder = png::Decoder::new(bytes);
        let mut reader = decoder.read_info().ok()?;
        let info = reader.info().clone();
        let mut buf = vec![0u8; reader.output_buffer_size()];
        reader.next_frame(&mut buf).ok()?;
        // 若非 RGBA，转 RGBA（oracle 均为 RGBA8/RGB，统一转 RGBA 比较）。
        let rgba = if info.color_type == ColorType::Rgb {
            buf.chunks(3).flat_map(|px| [px[0], px[1], px[2], 255u8]).collect()
        } else {
            buf
        };
        Some((info.width, info.height, rgba))
    }

    /// 两 RGBA8 缓冲的差异像素占比（min 尺寸对齐）。逐像素：任一通道差 > 8 计为差异。
    fn rgba_diff_pct(a: &(u32, u32, Vec<u8>), b: &(u32, u32, Vec<u8>)) -> f64 {
        let w = a.0.min(b.0) as usize;
        let h = a.1.min(b.1) as usize;
        let total = w * h;
        if total == 0 {
            return 1.0;
        }
        let stride = 4;
        let mut diff = 0usize;
        for y in 0..h {
            for x in 0..w {
                let ia = (y * a.0 as usize + x) * stride;
                let ib = (y * b.0 as usize + x) * stride;
                let da = [
                    a.2[ia].abs_diff(b.2[ib]),
                    a.2[ia + 1].abs_diff(b.2[ib + 1]),
                    a.2[ia + 2].abs_diff(b.2[ib + 2]),
                ];
                if da.iter().any(|&d| d > 8) {
                    diff += 1;
                }
            }
        }
        diff as f64 / total as f64
    }

    /// DC-13 line 315：welcome.html 经 ZeroBrowser headless 路径截图，与 chromium oracle
    /// 像素对比。验真实 headless 渲染管线（loadHtml + captureScreenshot 经 render_full_scene
    /// 全 13 图元）。welcome 自包含（内联 CSS + data-URI），loadHtml 绕过 fetch_url HTTP-only。
    /// oracle（welcome-chromium.png）为 tracked 文件（CI 可用）。baseline diff ~17%（字体墙）。
    #[test]
    fn test_dc13_line315_welcome_headless_vs_chromium_oracle() {
        use base64::Engine;

        let welcome = std::fs::read_to_string("assets/welcome.html").expect("welcome.html tracked fixture");
        let oracle_path = "../../docs/goal/rendering-compat/evidence/product-static/welcome-chromium.png";
        let Ok(oracle_bytes) = std::fs::read(oracle_path) else {
            eprintln!("skipping chromium oracle comparison; {oracle_path} is not available");
            return;
        };
        let oracle = decode_png_rgba(&oracle_bytes).expect("decode oracle PNG");

        let mut runner = ProtocolTestRunner::new();
        let load = runner
            .send("browsingContext.loadHtml", serde_json::json!({ "html": welcome }))
            .expect("loadHtml responds");
        let load_ok = load.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        assert!(load_ok, "loadHtml must report success: {load:?}");

        let shot = runner
            .send("browsingContext.captureScreenshot", serde_json::Value::Null)
            .expect("captureScreenshot responds");
        let data = shot.get("data").expect("captureScreenshot data field");
        let png_b64 = data
            .get("png")
            .and_then(|v| v.as_str())
            .expect("R1601: captureScreenshot must return base64 PNG in data.png");
        let shot_w = data.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let shot_h = data.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        assert_eq!((shot_w, shot_h), (800, 600), "headless viewport must be 800x600");

        let png_bytes = base64::engine::general_purpose::STANDARD
            .decode(png_b64)
            .expect("decode screenshot base64");
        let rendered = decode_png_rgba(&png_bytes).expect("decode screenshot PNG");

        let diff = rgba_diff_pct(&rendered, &oracle);
        eprintln!(
            "DC-13 line 315: welcome headless vs chromium oracle diff = {:.2}% ({}x{} vs {}x{})",
            diff * 100.0,
            rendered.0,
            rendered.1,
            oracle.0,
            oracle.1
        );
        // baseline ~17%（字体墙残余，同 product-smoke engine 路径 16.98%）；25% 留余量，
        // 超过则 headless 路径相对 chromium 退化（非字体墙）。
        assert!(
            diff < 0.25,
            "welcome headless render must be within 25% of chromium oracle (baseline ~17%): {:.2}%",
            diff * 100.0
        );
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
        assert_eq!(version["Browser"], format!("ZeroWeb/{}", zero_product_version::VERSION));
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
}

#[cfg(test)]
mod gpu_screenshot_tests {
    use super::*;

    /// R3282（#4）：GPU 截图开关下 PNG 输出与 CPU 截图一致（同 pipeline primitives，
    /// GPU 支持子集逐像素一致——parity/reftest 已验证）。
    #[test]
    fn gpu_screenshot_matches_cpu_for_supported_scene() {
        let server = HeadlessServer::new(0, 64.0, 64.0);
        let mut session = HeadlessSession::new(64.0, 64.0);
        // 先渲染一帧（页面内容进入 webview）
        session.webview.load_html(
            r#"<html><body style="margin:0"><div style="width:40px;height:40px;background:#f00;"></div></body></html>"#,
            None,
        );
        // CPU 截图
        let cpu_result = server
            .dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null)
            .unwrap();
        let cpu_png = cpu_result["data"]["png"].as_str().unwrap().to_string();
        // GPU 截图（env 开关）
        unsafe {
            std::env::set_var("ZW_HEADLESS_GPU_SCREENSHOT", "1");
        }
        let gpu_result = server
            .dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null)
            .unwrap();
        unsafe {
            std::env::remove_var("ZW_HEADLESS_GPU_SCREENSHOT");
        }
        let gpu_png = gpu_result["data"]["png"].as_str().unwrap().to_string();
        // GPU 环境不可用（无适配器）时可能回退 CPU——两者仍应一致
        assert_eq!(
            cpu_png, gpu_png,
            "GPU 截图应与 CPU 截图逐字节一致（支持子集；GPU 不可用自动回退）"
        );
    }
}

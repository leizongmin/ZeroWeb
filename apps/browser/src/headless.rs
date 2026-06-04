//! 无头浏览器协议 Phase 1 — 远程调试服务骨架。
//!
//! 支持 `--headless` 和 `--remote-debugging-port <port>` 启动无窗口实例，
//! 通过 WebSocket 接受自动化命令。
//!
//! 协议策略：WebDriver BiDi 优先，CDP 兼容子集辅助。
//! Phase 1 只实现基础会话管理和 JSON 消息路由。

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tungstenite::Message;
use tungstenite::accept;

use zero_browser_shell::BrowserShell;
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
        Self { shell, webview }
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
        }
    }

    /// 返回实际监听地址（绑定后才知道端口 0 时的实际端口）。
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// 启动无头协议服务器，阻塞运行直到连接关闭。
    pub fn run(&mut self) -> Result<(), String> {
        let listener =
            std::net::TcpListener::bind(self.addr).map_err(|e| format!("Failed to bind {}: {}", self.addr, e))?;

        // 更新实际地址（port=0 时 OS 分配随机端口）
        self.addr = listener
            .local_addr()
            .map_err(|e| format!("Failed to get local addr: {e}"))?;

        tracing::info!("Headless protocol server listening on ws://{}", self.addr);

        // 接受单个连接（Phase 1 只支持单客户端）
        let (stream, peer) = listener.accept().map_err(|e| format!("Accept failed: {e}"))?;
        tracing::info!("Client connected from {peer}");

        let mut ws = accept(stream).map_err(|e| format!("WebSocket handshake failed: {e}"))?;

        // 初始会话
        let mut session = HeadlessSession::new(self.viewport_width, self.viewport_height);

        // 主消息循环
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

            let response = self.handle_message(&mut session, &msg);
            let response_json = serde_json::to_string(&response).unwrap_or_else(|e| {
                format!("{{\"id\":0,\"error\":{{\"code\":-32700,\"message\":\"JSON serialize: {e}\"}}}}")
            });

            if let Err(e) = ws.write(Message::Text(response_json.into())) {
                tracing::error!("WebSocket write error: {e}");
                break;
            }
        }

        tracing::info!("Headless session ended");
        Ok(())
    }

    /// 处理单条客户端消息。
    fn handle_message(&self, session: &mut HeadlessSession, raw: &str) -> ServerResponse {
        let req: ClientRequest = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => {
                return ServerResponse {
                    id: 0,
                    result: None,
                    error: Some(ProtocolError {
                        code: -32700,
                        message: format!("Parse error: {e}"),
                    }),
                };
            }
        };

        let id = req.id;
        let result = self.dispatch(session, &req.method, req.params);

        match result {
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
        }
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

            // ── 导航 ──
            "browsingContext.navigate" => self.cmd_navigate(session, params),

            // ── 脚本执行 ──
            "script.evaluate" => self.cmd_script_evaluate(session, params),

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
        let result = session.webview.render();

        // 从 RenderPrimitives 提取 fills 和 glyphs
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
            1.0,
            &fills,
            &font_loader,
            &mut glyph_cache,
            &glyph_draws,
        );

        // 转为 base64 PNG（简化版：返回 raw RGBA 尺寸信息）
        Ok(serde_json::json!({
            "data": {
                "width": fb.width,
                "height": fb.height,
                "format": "rgba8",
                "pixelCount": fb.width as usize * fb.height as usize,
            }
        }))
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
}

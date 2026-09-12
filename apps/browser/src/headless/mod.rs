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
//!
//! 模块布局（M1 拆分，cdp-protocol goal P2）：transport（本文件：accept/auth/消息循环）
//! / protocol（wire 类型）/ session（会话与 renderer IPC）/ security（token/Origin）
//! / discovery（HTTP 发现）/ domains（命令域实现）/ client + tests + gpu_screenshot_tests
//! （测试基础设施与用例）。

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use tungstenite::Message;
use tungstenite::accept;

mod client;
mod discovery;
mod domains;
mod protocol;
mod security;
mod session;

#[cfg(test)]
mod gpu_screenshot_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub use client::{DomSnapshotStats, HeadlessClient};
use protocol::{ClientRequest, ProtocolError, ServerEvent, ServerResponse};
pub use security::HeadlessSecurityConfig;
use session::HeadlessSession;

// ── 协议服务器 ──

// ── 协议服务器 ──

/// 无头协议服务器。
pub struct HeadlessServer {
    /// 监听地址。
    addr: SocketAddr,
    /// 会话 ID 生成器。
    pub(super) next_session_id: Arc<AtomicU64>,
    /// 视口宽度。
    pub(super) viewport_width: f32,
    /// 视口高度。
    pub(super) viewport_height: f32,
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
}

// ── 测试 ──

/// R3254-F10：GPU 截图 env 开关测试（gpu_screenshot_tests）与 dc13 oracle（tests）互斥——
/// `ZW_HEADLESS_GPU_SCREENSHOT` 是进程级 env，并行设置会污染其他截图测试（dc13 误走
/// GPU 截图 → llvmpipe 崩溃）。两个测试模块共用（mod tests 与 mod gpu_screenshot_tests
/// 同层，经 super:: 引用）。
#[cfg(test)]
static GPU_SCREENSHOT_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

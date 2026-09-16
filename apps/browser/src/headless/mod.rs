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
mod devtools_serve;
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

// ── 连接多路复用状态（M1-S1.5 单线程复用模型）──

/// 单条连接的多路复用状态机。
struct ConnState {
    stream: std::net::TcpStream,
    peer: SocketAddr,
    alive: bool,
    phase: Phase,
}

impl ConnState {
    fn new(stream: std::net::TcpStream, peer: SocketAddr) -> Self {
        Self {
            stream,
            peer,
            alive: true,
            phase: Phase::Peek {
                deadline: std::time::Instant::now() + std::time::Duration::from_secs(5),
            },
        }
    }
}

/// 连接阶段：请求头探测（阻塞窗口）→ WS 会话（非阻塞消息循环）。
/// HTTP GET 在 Peek 阶段同步服务完即关（快进快出，与旧行为一致）。
#[allow(clippy::large_enum_variant)]
enum Phase {
    Peek {
        deadline: std::time::Instant,
    },
    Ws {
        ws: tungstenite::WebSocket<std::net::TcpStream>,
        page_direct: bool,
        authenticated: bool,
        idle_since: std::time::Instant,
    },
}

// ── 协议服务器 ──

/// 无头协议服务器。
pub struct HeadlessServer {
    /// 监听地址（绑定后回填实际端口；Arc<Self> 跨线程共享 → 互斥内可变）。
    addr: std::sync::Mutex<SocketAddr>,
    /// 会话 ID 生成器。
    pub(super) next_session_id: Arc<AtomicU64>,
    /// 视口（宽,高，CSS px）——headless 启动参数初始化，
    /// Emulation.setDeviceMetricsOverride 运行时可变（M3 viewport 桥）。
    pub(super) viewport: std::sync::Mutex<(f32, f32)>,
    /// 安全配置（Phase 5）。
    security: HeadlessSecurityConfig,
    /// 已附接的 CDP sessionId 注册表（sessionId → targetId；
    /// Target.attachedToTarget 时登记，detach/close 时移除；
    /// 命令携带未登记 sessionId → `-32001`）。
    attached_sessions: std::sync::Mutex<std::collections::HashMap<String, String>>,
    /// Target.setAutoAttach 的 autoAttach 开关（开启后新 target 自动附接）。
    auto_attach: std::sync::atomic::AtomicBool,
    /// devtools-frontend bundle 目录（`ZW_DEVTOOLS_FRONTEND_DIR`；None = 不 serve，
    /// devtoolsFrontendUrl 维持 `devtools://` 形态）。
    devtools_frontend_dir: Option<std::path::PathBuf>,
    /// DevTools 面板 nodeId → shim `__zwSelector` 映射（`DOM.getDocument` 时重建；
    /// CSS.getComputedStyle/getMatchedStylesForNode 按 nodeId 解析节点用）。
    pub(super) devtools_node_selectors: std::sync::Mutex<std::collections::HashMap<u64, String>>,
}

impl HeadlessServer {
    /// 创建无头协议服务器。
    pub fn new(port: u16, viewport_width: f32, viewport_height: f32) -> Self {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        Self {
            addr: std::sync::Mutex::new(addr),
            next_session_id: Arc::new(AtomicU64::new(1)),
            viewport: std::sync::Mutex::new((viewport_width, viewport_height)),
            security: HeadlessSecurityConfig::default(),
            attached_sessions: std::sync::Mutex::new(std::collections::HashMap::new()),
            auto_attach: std::sync::atomic::AtomicBool::new(false),
            devtools_frontend_dir: zero_runtime_config::optional_path("ZW_DEVTOOLS_FRONTEND_DIR"),
            devtools_node_selectors: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// devtools frontend bundle 是否已 provision（决定 `/json` 的
    /// devtoolsFrontendUrl 形态与 `/devtools/` serve 面是否可用）。
    pub(super) fn devtools_serve_enabled(&self) -> bool {
        self.devtools_frontend_dir.is_some()
    }

    /// 分配新的 CDP sessionId（不透明字符串，客户端按原样回传）。
    pub(super) fn next_cdp_session(&self) -> String {
        let n = self.next_session_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("zeroweb-session-{n}")
    }

    /// 登记 CDP sessionId 与其 target 的关联（Target 域附接时调用）。
    pub(super) fn attach_session(&self, session_id: &str, target_id: &str) {
        self.attached_sessions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(session_id.to_string(), target_id.to_string());
    }

    /// 移除 sessionId 附接登记，返回是否存在过。
    pub(super) fn detach_session(&self, session_id: &str) -> bool {
        self.attached_sessions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(session_id)
            .is_some()
    }

    /// 查询 sessionId 是否已附接。
    pub(super) fn session_attached(&self, session_id: &str) -> bool {
        self.attached_sessions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains_key(session_id)
    }

    /// 查询 sessionId 附接的 targetId。
    pub(super) fn session_target(&self, session_id: &str) -> Option<String> {
        self.attached_sessions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(session_id)
            .cloned()
    }

    /// 移除某 target 的全部附接登记（closeTarget 时）。
    pub(super) fn detach_sessions_for_target(&self, target_id: &str) -> Vec<String> {
        let mut registry = self.attached_sessions.lock().unwrap_or_else(|e| e.into_inner());
        let removed: Vec<String> = registry
            .iter()
            .filter(|(_, tid)| tid.as_str() == target_id)
            .map(|(sid, _)| sid.clone())
            .collect();
        for sid in &removed {
            registry.remove(sid);
        }
        removed
    }

    /// 读写 autoAttach 开关。
    pub(super) fn set_auto_attach(&self, enabled: bool) {
        self.auto_attach.store(enabled, std::sync::atomic::Ordering::SeqCst);
    }

    pub(super) fn auto_attach_enabled(&self) -> bool {
        self.auto_attach.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// 当前视口（宽,高）。
    pub(super) fn viewport_size(&self) -> (f32, f32) {
        *self.viewport.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// 更新视口（Emulation.setDeviceMetricsOverride）。
    pub(super) fn set_viewport_size(&self, width: f32, height: f32) {
        *self.viewport.lock().unwrap_or_else(|e| e.into_inner()) = (width, height);
    }

    /// 设置安全配置。
    #[allow(dead_code)]
    pub fn with_security(mut self, config: HeadlessSecurityConfig) -> Self {
        self.security = config;
        self
    }

    /// 返回实际监听地址（绑定后才知道端口 0 时的实际端口）。
    pub fn addr(&self) -> SocketAddr {
        *self.addr.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// 启动无头协议服务器，阻塞运行直到进程终止。
    ///
    /// 支持 HTTP 发现请求（/json/version、/json）和 WebSocket 协议连接。
    /// 连接模型（devtools goal M1-S1.5）：**单线程 socket 多路复用**——DevTools
    /// frontend 在 WS 建立后还会经同一端口 lazy-load 面板模块（`import()` 动态导入），
    /// 单连接串行 accept 循环会让该 HTTP 请求在 backlog 饿死（实测：Network 面板空白）。
    /// 全部连接非阻塞 + 轮询推进（3ms tick），`HeadlessSession` 保持主线程独占
    ///（其内部含 `Rc` 非 `Send`，不跨线程；命令语义天然串行，零锁）。
    pub fn run(self: Arc<Self>) -> Result<(), String> {
        let bind_addr = *self.addr.lock().unwrap_or_else(|e| e.into_inner());
        let listener =
            std::net::TcpListener::bind(bind_addr).map_err(|e| format!("Failed to bind {}: {}", bind_addr, e))?;
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("Failed to set listener nonblocking: {e}"))?;

        let actual = listener
            .local_addr()
            .map_err(|e| format!("Failed to get local addr: {e}"))?;
        *self.addr.lock().unwrap_or_else(|e| e.into_inner()) = actual;

        tracing::info!("Headless protocol server listening on ws://{}", actual);

        // CDP 语义：target 生命周期跨客户端连接持续（同一 renderer 服务所有连接，
        // HTTP 发现枚举与 WS 会话共享同一浏览器状态）。
        let (viewport_width, viewport_height) = self.viewport_size();
        let mut session = HeadlessSession::new(viewport_width, viewport_height);

        let mut conns: Vec<ConnState> = Vec::new();
        loop {
            // 1. 接受新连接（非阻塞，一次收干）
            while let Ok((stream, peer)) = listener.accept() {
                tracing::info!("Connection from {peer}");
                let _ = stream.set_nonblocking(false);
                let _ = stream.set_nodelay(true);
                conns.push(ConnState::new(stream, peer));
            }

            // 2. 推进每条连接一步
            for conn in &mut conns {
                self.advance_connection(conn, &mut session);
            }
            conns.retain(|conn| conn.alive);

            std::thread::sleep(std::time::Duration::from_millis(3));
        }
    }

    /// 推进单条连接一个非阻塞步骤。
    fn advance_connection(&self, conn: &mut ConnState, session: &mut HeadlessSession) {
        match &mut conn.phase {
            // ── 阻塞窗口收满请求头（5s 超时）→ 分类 ──
            Phase::Peek { deadline } => {
                let mut buf = [0u8; 4096];
                match conn.stream.peek(&mut buf) {
                    Ok(0) => {
                        conn.alive = false;
                    }
                    Ok(n) => {
                        let data = &buf[..n];
                        if Self::is_http_get_request(data) {
                            conn.alive = false;
                            let origin = Self::extract_origin_header(data);
                            if !self.security.verify_origin(origin.as_deref()) {
                                tracing::warn!("HTTP request from disallowed origin: {origin:?} from {}", conn.peer);
                                return;
                            }
                            self.handle_http_discovery(&conn.stream, session);
                        } else if Self::is_ws_upgrade(data) {
                            let origin = Self::extract_origin_header(data);
                            if !self.security.verify_origin(origin.as_deref()) {
                                tracing::warn!("WebSocket from disallowed origin: {origin:?} from {}", conn.peer);
                                conn.alive = false;
                                return;
                            }
                            let page_direct = discovery::extract_request_path(data)
                                .is_some_and(|p| p.starts_with(devtools_serve::PAGE_WS_PREFIX));
                            if page_direct {
                                tracing::info!("DevTools frontend page-direct connection from {}", conn.peer);
                            }
                            match accept(conn.stream.try_clone().expect("peek stream clone")) {
                                Ok(ws) => {
                                    conn.stream.set_nonblocking(true).ok();
                                    conn.phase = Phase::Ws {
                                        ws,
                                        page_direct,
                                        authenticated: self.security.auth_token.is_none(),
                                        idle_since: std::time::Instant::now(),
                                    };
                                }
                                Err(e) => {
                                    tracing::error!("WebSocket handshake failed: {e}");
                                    conn.alive = false;
                                }
                            }
                        } else if std::time::Instant::now() > *deadline {
                            // 非 HTTP 非 WS 流量：等满 5s 再丢弃（保持旧超时语义）
                            tracing::warn!("Unrecognized connection from {}", conn.peer);
                            conn.alive = false;
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        if std::time::Instant::now() > *deadline {
                            tracing::warn!("Peek timed out for {}", conn.peer);
                            conn.alive = false;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Peek failed for {}: {e}", conn.peer);
                        conn.alive = false;
                    }
                }
            }
            // ── WS 会话：非阻塞读 → 命令处理 → 应答 ──
            Phase::Ws {
                ws,
                page_direct,
                authenticated,
                idle_since,
            } => {
                match ws.read() {
                    Ok(Message::Text(text)) => {
                        *idle_since = std::time::Instant::now();
                        if !*authenticated && !self.authenticate_first_message(&text) {
                            *authenticated = self.security.auth_token.is_none();
                            if !*authenticated {
                                let err = ServerResponse {
                                    id: 0,
                                    result: None,
                                    error: Some(ProtocolError {
                                        code: -32001,
                                        message: "Authentication required".into(),
                                    }),
                                    session_id: None,
                                };
                                if let Ok(json) = serde_json::to_string(&err) {
                                    let _ = ws.write(Message::Text(json.into()));
                                    let _ = ws.flush();
                                }
                                return;
                            }
                        }
                        let (response, events) = self.handle_message_with_events_mode(session, &text, *page_direct);
                        for event in events {
                            if let Ok(event_json) = serde_json::to_string(&event)
                                && let Err(e) = ws.write(Message::Text(event_json.into()))
                            {
                                tracing::error!("Event push error: {e}");
                                conn.alive = false;
                                return;
                            }
                        }
                        let response_json = serde_json::to_string(&response).unwrap_or_else(|e| {
                            format!("{{\"id\":0,\"error\":{{\"code\":-32700,\"message\":\"JSON serialize: {e}\"}}}}")
                        });
                        if let Err(e) = ws.write(Message::Text(response_json.into())) {
                            tracing::error!("WebSocket write error: {e}");
                            conn.alive = false;
                            return;
                        }
                        // tungstenite write() 对可入缓冲的小消息不保证落盘（should_flush=false），
                        // 命令响应必须显式 flush，否则客户端收不到任何回包。
                        if let Err(e) = ws.flush() {
                            tracing::error!("WebSocket flush error: {e}");
                            conn.alive = false;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("Client disconnected");
                        conn.alive = false;
                    }
                    Ok(Message::Ping(data)) => {
                        *idle_since = std::time::Instant::now();
                        let _ = ws.write(Message::Pong(data));
                        // tungstenite write() 对可入缓冲的小消息不保证落盘，必须显式 flush
                        let _ = ws.flush();
                    }
                    Ok(_) => {}
                    Err(tungstenite::Error::Io(ref e))
                        if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut =>
                    {
                        // 轮询超时：drain renderer 通道（fetch 代理/console 转发），并把
                        // 产生的事件即时推给客户端（S12 network.events 实测根因）。
                        self.drain_renderer_channel(session, ws);
                        if idle_since.elapsed() > Self::WS_IDLE_DEADLINE {
                            tracing::info!("WebSocket idle deadline ({}s)", Self::WS_IDLE_DEADLINE.as_secs());
                            conn.alive = false;
                        }
                    }
                    Err(e) => {
                        tracing::error!("WebSocket read error: {e}");
                        conn.alive = false;
                    }
                }
            }
        }
    }

    /// 首个消息 token 认证（Phase 5 语义保持）。
    fn authenticate_first_message(&self, raw: &str) -> bool {
        match serde_json::from_str::<ClientRequest>(raw) {
            Ok(req) => {
                let token = req.params.get("token").and_then(|v| v.as_str());
                self.security.verify_token(token)
            }
            Err(_) => false,
        }
    }

    /// WS idle 兜底断开阈值（与旧 600s read timeout 语义一致）。
    const WS_IDLE_DEADLINE: std::time::Duration = std::time::Duration::from_secs(600);

    /// S12：CDP 空闲期 drain renderer 通道——fetch 代理（`FetchRequest`/`FetchResponse`）
    /// 与 console 转发（`ConsoleLog`）是 renderer → session 单向消息；CDP 命令间歇期
    /// session 无人消费，renderer 侧 `ipc_fetch` 阻塞等待会饿死（network.events 实测：
    /// 点击 handler 的 fetch 挂起 → PW click 10s 超时）。返回本函数起始时刻（供空闲
    /// deadline 记账——真实消息处理会重置调用方的 idle_since）。
    fn drain_renderer_channel(
        &self,
        session: &mut HeadlessSession,
        ws: &mut tungstenite::WebSocket<std::net::TcpStream>,
    ) {
        let mut events: Vec<ServerEvent> = Vec::new();
        #[cfg(not(test))]
        session.drain_fetch_completions();
        while let Some(message) = session.try_recv_renderer() {
            let _ = session.handle_renderer_message(message);
        }
        // 排空产生的 CDP 事件（console/network）即时推送——单会话模型：盖章到首个
        // 已附接的页面 session（未附接则丢弃，客户端未就绪）。
        let page_session = self
            .attached_sessions
            .lock()
            .ok()
            .and_then(|map| map.keys().next().cloned());
        for (level, _text, args_json) in session.pending_console_events.drain(..) {
            let args = serde_json::from_str::<serde_json::Value>(&args_json)
                .ok()
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default();
            let cdp_type = match level.as_str() {
                "warn" => "warning",
                "debug" | "trace" => "debug",
                other => other,
            };
            events.push(ServerEvent {
                method: "Runtime.consoleAPICalled".into(),
                params: serde_json::json!({
                    "type": cdp_type,
                    "args": args.iter().map(console_value_to_remote_object).collect::<Vec<_>>(),
                    "executionContextId": 1,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                }),
                session_id: page_session.clone(),
            });
        }
        for err in session.pending_script_errors.drain(..) {
            events.push(ServerEvent {
                method: "Runtime.exceptionThrown".into(),
                params: serde_json::json!({
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                    "exceptionDetails": {
                        "text": err.text,
                        "url": err.source,
                        "lineNumber": err.line_number,
                        "columnNumber": err.column_number,
                        "exception": {
                            "type": "object",
                            "subtype": "error",
                            "description": err.text,
                        },
                    },
                }),
                session_id: page_session.clone(),
            });
        }
        for (method, params) in session.pending_network_events.drain(..) {
            events.push(ServerEvent {
                method,
                params,
                session_id: page_session.clone(),
            });
        }
        for event in events {
            if let Ok(event_json) = serde_json::to_string(&event) {
                let _ = ws.write(Message::Text(event_json.into()));
                let _ = ws.flush();
            }
        }
    }

    /// 处理单条客户端消息（向后兼容，不含事件）。
    #[allow(dead_code)]
    fn handle_message(&self, session: &mut HeadlessSession, raw: &str) -> ServerResponse {
        let (response, _) = self.handle_message_with_events(session, raw);
        response
    }

    /// 处理单条客户端消息，返回响应和事件通知列表（浏览器级连接形态）。
    fn handle_message_with_events(
        &self,
        session: &mut HeadlessSession,
        raw: &str,
    ) -> (ServerResponse, Vec<ServerEvent>) {
        self.handle_message_with_events_mode(session, raw, false)
    }

    /// 处理单条客户端消息（带连接形态）：`page_direct` = per-page ws 直连
    /// （DevTools frontend；Target.getTargetInfo 无参查询据此返回页面而非 browser）。
    ///
    /// CDP 扁平协议会话路由：请求携带 `sessionId` 时须为已附接会话（否则 `-32001`），
    /// 响应原样回显该 `sessionId`；缺省 = 浏览器级命令，路由到全局会话。
    fn handle_message_with_events_mode(
        &self,
        session: &mut HeadlessSession,
        raw: &str,
        page_direct: bool,
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
                        session_id: None,
                    },
                    Vec::new(),
                );
            }
        };

        let id = req.id;
        // 会话校验（未附接的 sessionId 直接拒绝，防串话）
        if let Some(ref sid) = req.session_id {
            if !self.session_attached(sid) {
                return (
                    ServerResponse {
                        id,
                        result: None,
                        error: Some(ProtocolError {
                            code: -32001,
                            message: format!("Session not found: {sid}"),
                        }),
                        session_id: req.session_id,
                    },
                    Vec::new(),
                );
            }
        }
        let (result, mut events) =
            self.dispatch_with_events_for(session, req.session_id.as_deref(), &req.method, req.params, page_direct);

        // 事件路由：session 级事件盖章请求的 sessionId（客户端据此投递到 child
        // session——不带会被当作浏览器级事件丢弃）；Target 域的宣告事件本身是
        // 浏览器级（新会话 id 在 params 里），保持不带。
        if let Some(sid) = req.session_id.as_deref() {
            for event in &mut events {
                if !matches!(
                    event.method.as_str(),
                    "Target.attachedToTarget" | "Target.detachedFromTarget" | "Target.targetDestroyed"
                ) {
                    event.session_id = Some(sid.to_string());
                }
            }
        }
        // Console 事件排空（S11：renderer ConsoleLog → `Runtime.consoleAPICalled`，
        // value-only remoteObject args；归当前命令会话盖章发送）。
        // S16 时序契约：console 事件先于 page lifecycle 重发——PW setContent 的
        // console tag 到达时 `_onClearLifecycle()` 清生命周期，load 族须在其后送达。
        for (level, _text, args_json) in session.pending_console_events.drain(..) {
            let args = serde_json::from_str::<serde_json::Value>(&args_json)
                .ok()
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default();
            let cdp_type = match level.as_str() {
                "warn" => "warning",
                "debug" | "trace" => "debug",
                other => other,
            };
            events.push(ServerEvent {
                method: "Runtime.consoleAPICalled".into(),
                params: serde_json::json!({
                    "type": cdp_type,
                    "args": args.iter().map(console_value_to_remote_object).collect::<Vec<_>>(),
                    // 主 world 上下文 id（S4 契约：executionContextCreated 恒 id=1）。
                    "executionContextId": 1,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                }),
                session_id: req.session_id.clone(),
            });
        }
        // Network/Page 域事件排空（proxy_fetch 生命周期产出 + document.write 落定的
        // load 生命周期重发，归当前命令会话盖章发送）
        for (method, params) in session.pending_network_events.drain(..) {
            events.push(ServerEvent {
                method,
                params,
                session_id: req.session_id.clone(),
            });
        }

        let response = match result {
            Ok(value) => ServerResponse {
                id,
                result: Some(value),
                error: None,
                session_id: req.session_id,
            },
            Err(err) => {
                // S78 诊断：-32000 Channel 类错误通常意味着 renderer 子进程已死或
                // IPC 通道断裂——renderer 的 stderr 临终输出只进内部 tail 缓冲，
                // 在错误路径透出（热路径零开销），否则死因排障不可见。
                if err.code == -32000 {
                    let tail = session.renderer_stderr_tail();
                    if !tail.is_empty() {
                        tracing::error!("renderer stderr tail on channel error: {tail}");
                    }
                }
                ServerResponse {
                    id,
                    result: None,
                    error: Some(err),
                    session_id: req.session_id,
                }
            }
        };

        (response, events)
    }
}

/// S11：console 逐参值 → value-only CDP remoteObject（`__zw_undefined__` 标记 →
/// `{type:"undefined"}`；对象/数组保结构内联 value——headless 单机无句柄需求）。
fn console_value_to_remote_object(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Null => serde_json::json!({ "type": "object", "subtype": "null", "value": null }),
        serde_json::Value::Bool(v) => serde_json::json!({ "type": "boolean", "value": v }),
        serde_json::Value::Number(v) => serde_json::json!({ "type": "number", "value": v }),
        serde_json::Value::String(s) if s == "__zw_undefined__" => serde_json::json!({ "type": "undefined" }),
        serde_json::Value::String(s) => serde_json::json!({ "type": "string", "value": s }),
        serde_json::Value::Array(items) => serde_json::json!({
            "type": "object", "subtype": "array",
            "value": items.iter().map(console_value_to_remote_object).collect::<Vec<_>>(),
        }),
        serde_json::Value::Object(entries) => serde_json::json!({
            "type": "object",
            "value": entries
                .iter()
                .map(|(k, v)| (k.clone(), console_value_to_remote_object(v)))
                .collect::<serde_json::Map<String, serde_json::Value>>(),
        }),
    }
}

// ── 测试 ──

/// R3254-F10：GPU 截图 env 开关测试（gpu_screenshot_tests）与 dc13 oracle（tests）互斥——
/// `ZW_HEADLESS_GPU_SCREENSHOT` 是进程级 env，并行设置会污染其他截图测试（dc13 误走
/// GPU 截图 → llvmpipe 崩溃）。两个测试模块共用（mod tests 与 mod gpu_screenshot_tests
/// 同层，经 super:: 引用）。
#[cfg(test)]
static GPU_SCREENSHOT_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

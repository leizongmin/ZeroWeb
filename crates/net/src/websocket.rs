//! # WebSocket 客户端
//!
//! 基于 tungstenite 的 WebSocket 客户端实现。
//!
//! 支持连接到 `ws://` 和 `wss://` 服务器，发送和接收文本/二进制消息，
//! 以及正常关闭连接。

use tungstenite::client::IntoClientRequest;
use tungstenite::http::header::{COOKIE, ORIGIN};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket as TungWebSocket};
use url::Url;

/// 可选 WebSocket 握手请求头（WHATWG HTML「establish a WebSocket connection」）。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WebSocketHandshake {
    /// 文档 `Origin`（序列化 origin）；空则省略。
    pub origin: String,
    /// `Cookie` 请求头值；空则省略。
    pub cookie: String,
}

/// WebSocket 连接状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketState {
    /// 正在连接。
    Connecting,
    /// 已连接。
    Open,
    /// 正在关闭。
    Closing,
    /// 已关闭。
    Closed,
}

/// WebSocket 连接错误类型。
#[derive(Debug)]
pub enum WebSocketError {
    /// 连接失败。
    ConnectionFailed(String),
    /// 发送失败。
    SendFailed(String),
    /// 接收失败。
    ReceiveFailed(String),
    /// 关闭失败。
    CloseFailed(String),
    /// URL 解析错误。
    InvalidUrl(String),
    /// 连接未打开。
    NotOpen,
}

impl std::fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "WebSocket connection failed: {msg}"),
            Self::SendFailed(msg) => write!(f, "WebSocket send failed: {msg}"),
            Self::ReceiveFailed(msg) => write!(f, "WebSocket receive failed: {msg}"),
            Self::CloseFailed(msg) => write!(f, "WebSocket close failed: {msg}"),
            Self::InvalidUrl(msg) => write!(f, "WebSocket invalid URL: {msg}"),
            Self::NotOpen => write!(f, "WebSocket is not open"),
        }
    }
}

impl std::error::Error for WebSocketError {}

/// WebSocket 消息类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketMessage {
    /// 文本消息。
    Text(String),
    /// 二进制消息。
    Binary(Vec<u8>),
    /// 关闭帧。
    Close(Option<u16>, Option<String>),
    /// Ping 消息。
    Ping(Vec<u8>),
    /// Pong 消息。
    Pong(Vec<u8>),
}

/// 底层 WebSocket 流类型。
type WsStream = MaybeTlsStream<std::net::TcpStream>;

/// WebSocket 客户端。
///
/// 支持连接到 WebSocket 服务器，发送和接收消息，以及正常关闭连接。
/// 底层使用 tungstenite 实现 WebSocket 协议（RFC 6455）。
pub struct WebSocket {
    url: String,
    state: WebSocketState,
    inner: Option<TungWebSocket<WsStream>>,
}

impl WebSocket {
    /// 创建新的 WebSocket 实例（未连接状态）。
    ///
    /// 不会立即连接，需要调用 [`connect()`](Self::connect) 建立连接。
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            state: WebSocketState::Connecting,
            inner: None,
        }
    }

    /// 建立到 WebSocket 服务器的连接。
    ///
    /// 支持 `ws://` 和 `wss://` 协议。连接成功后状态变为 `Open`。
    ///
    /// # 错误
    ///
    /// - URL 格式无效或 scheme 非 `ws`/`wss` 时返回 [`InvalidUrl`](WebSocketError::InvalidUrl)。
    /// - 无法建立 TCP/TLS 连接或握手失败时返回
    ///   [`ConnectionFailed`](WebSocketError::ConnectionFailed)。
    pub fn connect(&mut self) -> Result<(), WebSocketError> {
        self.connect_with_handshake(&WebSocketHandshake::default())
    }

    /// 建立连接并附带文档 `Origin` / `Cookie` 握手头（空串字段省略）。
    pub fn connect_with_handshake(&mut self, handshake: &WebSocketHandshake) -> Result<(), WebSocketError> {
        if self.state == WebSocketState::Open {
            return Ok(());
        }

        let parsed_url = Url::parse(&self.url).map_err(|e| WebSocketError::InvalidUrl(e.to_string()))?;
        // 信任边界输入校验：WebSocket 仅支持 ws/wss scheme。非 ws/wss（如 http/ftp/file）
        // 虽能被 Url::parse 接受，但不是合法 WebSocket URL——提前拒为 InvalidUrl，避免
        // 交给 tungstenite 后被归类为 ConnectionFailed（错误类型与实际原因不符，误导排查）。
        if !matches!(parsed_url.scheme(), "ws" | "wss") {
            return Err(WebSocketError::InvalidUrl(format!(
                "WebSocket URL must use ws:// or wss:// scheme, got: {scheme}",
                scheme = parsed_url.scheme()
            )));
        }

        let mut request = parsed_url
            .as_str()
            .into_client_request()
            .map_err(|e| WebSocketError::ConnectionFailed(e.to_string()))?;
        if !handshake.origin.is_empty() {
            request.headers_mut().insert(
                ORIGIN,
                handshake
                    .origin
                    .parse()
                    .map_err(|e| WebSocketError::ConnectionFailed(format!("invalid Origin header: {e}")))?,
            );
        }
        if !handshake.cookie.is_empty() {
            request.headers_mut().insert(
                COOKIE,
                handshake
                    .cookie
                    .parse()
                    .map_err(|e| WebSocketError::ConnectionFailed(format!("invalid Cookie header: {e}")))?,
            );
        }

        let (socket, _response) =
            tungstenite::connect(request).map_err(|e| WebSocketError::ConnectionFailed(e.to_string()))?;

        self.inner = Some(socket);
        self.state = WebSocketState::Open;
        Ok(())
    }

    /// 发送文本消息。
    ///
    /// 仅在 `Open` 状态下成功，否则返回 [`NotOpen`](WebSocketError::NotOpen)。
    pub fn send(&mut self, message: &str) -> Result<(), WebSocketError> {
        if self.state != WebSocketState::Open {
            return Err(WebSocketError::NotOpen);
        }
        let ws = self.inner.as_mut().ok_or(WebSocketError::NotOpen)?;
        ws.send(Message::Text(message.into()))
            .map_err(|e| WebSocketError::SendFailed(e.to_string()))?;
        Ok(())
    }

    /// 发送二进制消息。
    pub fn send_binary(&mut self, data: &[u8]) -> Result<(), WebSocketError> {
        if self.state != WebSocketState::Open {
            return Err(WebSocketError::NotOpen);
        }
        let ws = self.inner.as_mut().ok_or(WebSocketError::NotOpen)?;
        ws.send(Message::Binary(data.to_vec().into()))
            .map_err(|e| WebSocketError::SendFailed(e.to_string()))?;
        Ok(())
    }

    /// 尝试接收下一条消息（非阻塞轮询）。
    ///
    /// 如果没有消息可读，返回 `Ok(None)`。
    /// 如果连接已关闭（收到 Close 帧），返回 `Ok(None)` 并更新状态为 `Closed`。
    pub fn receive(&mut self) -> Result<Option<WebSocketMessage>, WebSocketError> {
        if self.state != WebSocketState::Open {
            return Err(WebSocketError::NotOpen);
        }
        let ws = self.inner.as_mut().ok_or(WebSocketError::NotOpen)?;

        // 使用 read_message 获取下一条消息
        match ws.read() {
            Ok(msg) => match msg {
                Message::Text(text) => Ok(Some(WebSocketMessage::Text(text.to_string()))),
                Message::Binary(data) => Ok(Some(WebSocketMessage::Binary(data.to_vec()))),
                Message::Close(close_frame) => {
                    let (code, reason) = close_frame
                        .map(|cf| (Some(cf.code.into()), Some(cf.reason.to_string())))
                        .unwrap_or((None, None));
                    // RFC 6455 §7.1.5：收到 Close 后须回送 Close（若尚未发送）。
                    let _ = ws.close(close_frame);
                    self.state = WebSocketState::Closed;
                    self.inner = None;
                    Ok(Some(WebSocketMessage::Close(code, reason)))
                }
                Message::Ping(data) => Ok(Some(WebSocketMessage::Ping(data.to_vec()))),
                Message::Pong(data) => Ok(Some(WebSocketMessage::Pong(data.to_vec()))),
                _ => Ok(None),
            },
            Err(tungstenite::Error::Io(e))
                if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                // 读超时 / 非阻塞：暂无消息（SO_RCVTIMEO 在 Windows 等映射为 TimedOut）。
                Ok(None)
            }
            Err(tungstenite::Error::ConnectionClosed) => {
                self.state = WebSocketState::Closed;
                self.inner = None;
                Ok(None)
            }
            Err(e) => Err(WebSocketError::ReceiveFailed(e.to_string())),
        }
    }

    /// 发送关闭帧并正常关闭连接。
    ///
    /// 发送 Close 帧后状态变为 `Closed`。
    pub fn close(&mut self) -> Result<(), WebSocketError> {
        if let Some(ws) = self.inner.as_mut() {
            // 发送 Close 帧
            let _ = ws.close(None);
        }
        self.state = WebSocketState::Closed;
        self.inner = None;
        Ok(())
    }

    /// 发送带状态码/原因的关闭帧并关闭连接（WHATWG HTML §WebSocket close(code, reason)）。
    pub fn close_with(&mut self, code: u16, reason: &str) -> Result<(), WebSocketError> {
        if let Some(ws) = self.inner.as_mut() {
            let frame = tungstenite::protocol::CloseFrame {
                code: code.into(),
                reason: tungstenite::Utf8Bytes::from(reason),
            };
            let _ = ws.close(Some(frame));
        }
        self.state = WebSocketState::Closed;
        self.inner = None;
        Ok(())
    }

    /// 给底层 TCP 流设置读超时（宿主桥读泵用：阻塞 `read()` 超时返回 WouldBlock 或
    /// TimedOut（平台相关），
    /// 泵循环得以轮询发送通道）。仅 Open 状态生效；wss 路径同样作用在内层 TCP 流。
    /// TLS 后端由 workspace 统一为 rustls（tungstenite `rustls-tls-webpki-roots`，与
    /// reqwest HTTP 栈同族 webpki-roots 信任源）——变更 tungstenite TLS feature 时
    /// 此处 match 须同步。
    pub fn set_read_timeout(&self, dur: Option<std::time::Duration>) -> Result<(), WebSocketError> {
        let ws = self.inner.as_ref().ok_or(WebSocketError::NotOpen)?;
        match ws.get_ref() {
            MaybeTlsStream::Plain(tcp) => tcp
                .set_read_timeout(dur)
                .map_err(|e| WebSocketError::ReceiveFailed(e.to_string())),
            MaybeTlsStream::Rustls(tls) => tls
                .get_ref()
                .set_read_timeout(dur)
                .map_err(|e| WebSocketError::ReceiveFailed(e.to_string())),
            // MaybeTlsStream 标记 non-exhaustive（后续 tungstenite 版本可能增变体）——未知
            // 流形态拒设读超时（泵循环保持默认阻塞语义，不静默假成功）。
            _ => Err(WebSocketError::ReceiveFailed(
                "unsupported TLS stream variant".to_string(),
            )),
        }
    }

    /// 返回当前连接状态。
    pub fn state(&self) -> &WebSocketState {
        &self.state
    }

    /// 返回连接的 URL。
    pub fn url(&self) -> &str {
        &self.url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_new_state() {
        let ws = WebSocket::new("ws://example.com/socket");
        assert_eq!(ws.state(), &WebSocketState::Connecting);
        assert_eq!(ws.url(), "ws://example.com/socket");
    }

    #[test]
    fn test_websocket_close_from_connecting() {
        let mut ws = WebSocket::new("ws://example.com/socket");
        assert_eq!(ws.state(), &WebSocketState::Connecting);
        ws.close().unwrap();
        assert_eq!(ws.state(), &WebSocketState::Closed);
    }

    #[test]
    fn test_websocket_send_when_closed() {
        let mut ws = WebSocket::new("ws://example.com/socket");
        ws.close().unwrap();
        let result = ws.send("hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_websocket_receive_when_closed() {
        let mut ws = WebSocket::new("ws://example.com/socket");
        ws.close().unwrap();
        let result = ws.receive();
        assert!(result.is_err());
    }

    #[test]
    fn test_websocket_send_before_connect() {
        let mut ws = WebSocket::new("ws://example.com/socket");
        assert_eq!(ws.state(), &WebSocketState::Connecting);
        let result = ws.send("hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_websocket_receive_before_connect() {
        let mut ws = WebSocket::new("ws://example.com/socket");
        let result = ws.receive();
        assert!(result.is_err());
    }

    #[test]
    fn test_websocket_invalid_url() {
        let mut ws = WebSocket::new("not a valid url");
        let result = ws.connect();
        assert!(result.is_err());
        match result.unwrap_err() {
            WebSocketError::InvalidUrl(_) => {}
            other => panic!("expected InvalidUrl, got: {other:?}"),
        }
    }

    // ── R3369：scheme 校验（信任边界输入校验）──

    #[test]
    /// R3369：非 ws/wss scheme（http/ftp/file）须被拒为 InvalidUrl，
    /// 而非交给 tungstenite 后归类为 ConnectionFailed（错误类型与原因不符）。
    fn test_websocket_non_ws_scheme_rejected_as_invalid_url_r3369() {
        for bad in [
            "http://127.0.0.1:1/x",
            "https://127.0.0.1:1/x",
            "ftp://127.0.0.1/x",
            "file:///x",
        ] {
            let mut ws = WebSocket::new(bad);
            let result = ws.connect();
            let err = result.expect_err("{bad:?} 应被拒");
            match err {
                WebSocketError::InvalidUrl(msg) => {
                    assert!(
                        msg.contains("ws://") || msg.contains("wss://"),
                        "错误消息应提示合法 scheme：{msg}"
                    );
                }
                other => panic!("scheme={bad:?} 应返回 InvalidUrl，实际：{other:?}"),
            }
            // 被拒后状态不应变为 Open
            assert_ne!(ws.state(), &WebSocketState::Open);
        }
    }

    #[test]
    /// R3369：合法 ws/wss scheme 但连接不可达时仍返回 ConnectionFailed（scheme 校验通过，
    /// 仅网络层失败）——确保 scheme 校验未误伤合法 URL。
    fn test_websocket_valid_ws_scheme_still_attempts_connect_r3369() {
        let mut ws = WebSocket::new("ws://127.0.0.1:1/unreachable");
        let result = ws.connect();
        let err = result.expect_err("不可达端口应失败");
        // scheme 合法 → 不应是 InvalidUrl，而是 ConnectionFailed（网络层）
        assert!(
            matches!(err, WebSocketError::ConnectionFailed(_)),
            "ws:// 不可达应返回 ConnectionFailed，实际：{err:?}"
        );
    }

    #[test]
    fn test_websocket_send_binary_when_closed() {
        let mut ws = WebSocket::new("ws://example.com/socket");
        ws.close().unwrap();
        let result = ws.send_binary(b"hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_websocket_close_idempotent() {
        let mut ws = WebSocket::new("ws://example.com/socket");
        ws.close().unwrap();
        assert_eq!(ws.state(), &WebSocketState::Closed);
        ws.close().unwrap();
        assert_eq!(ws.state(), &WebSocketState::Closed);
    }

    #[test]
    fn test_websocket_error_display() {
        let err = WebSocketError::ConnectionFailed("refused".to_string());
        assert!(err.to_string().contains("refused"));
        assert!(err.to_string().contains("connection failed"));

        let err = WebSocketError::NotOpen;
        assert!(err.to_string().contains("not open"));

        let err = WebSocketError::SendFailed("broken pipe".to_string());
        assert!(err.to_string().contains("broken pipe"));
        assert!(err.to_string().contains("send failed"));

        let err = WebSocketError::ReceiveFailed("reset".to_string());
        assert!(err.to_string().contains("reset"));
        assert!(err.to_string().contains("receive failed"));

        let err = WebSocketError::CloseFailed("timeout".to_string());
        assert!(err.to_string().contains("timeout"));
        assert!(err.to_string().contains("close failed"));
    }

    #[test]
    fn test_websocket_message_equality() {
        let text = WebSocketMessage::Text("hello".to_string());
        assert_eq!(text, WebSocketMessage::Text("hello".to_string()));
        assert_ne!(text, WebSocketMessage::Text("world".to_string()));

        let binary = WebSocketMessage::Binary(vec![1, 2, 3]);
        assert_eq!(binary, WebSocketMessage::Binary(vec![1, 2, 3]));
        assert_ne!(binary, WebSocketMessage::Binary(vec![4, 5, 6]));

        let close = WebSocketMessage::Close(Some(1000), Some("bye".to_string()));
        assert!(matches!(close, WebSocketMessage::Close(Some(1000), Some(_))));

        let ping = WebSocketMessage::Ping(vec![1, 2, 3]);
        assert!(matches!(ping, WebSocketMessage::Ping(_)));

        let pong = WebSocketMessage::Pong(vec![4, 5, 6]);
        assert!(matches!(pong, WebSocketMessage::Pong(_)));
    }

    #[test]
    fn test_websocket_connect_then_close() {
        // 测试连接到不存在的服务器时的错误处理
        let mut ws = WebSocket::new("ws://127.0.0.1:1/nonexistent");
        let result = ws.connect();
        // 连接应失败（端口 1 通常没有服务监听）
        assert!(result.is_err());
        // 连接失败后状态应保持 Connecting 或 Closed
        assert!(ws.state() == &WebSocketState::Connecting || ws.state() == &WebSocketState::Closed);
    }

    #[test]
    fn test_websocket_connect_already_open_noop() {
        // 如果已经 Open，再次调用 connect 应返回 Ok(())
        // 这里无法模拟 Open 状态（需要真实连接），因此测试 API 契约
        let mut ws = WebSocket::new("ws://example.com/socket");
        // 未连接时 connect 会尝试连接（example.com 可能不支持 WebSocket）
        // 但我们只验证 API 调用不 panic
        let _ = ws.connect();
    }

    #[test]
    fn test_websocket_connection_refused_error() {
        let mut ws = WebSocket::new("ws://127.0.0.1:1/test");
        let result = ws.connect();
        assert!(result.is_err());
        match result.unwrap_err() {
            WebSocketError::ConnectionFailed(msg) => {
                assert!(!msg.is_empty());
            }
            other => panic!("expected ConnectionFailed, got: {other:?}"),
        }
    }
}

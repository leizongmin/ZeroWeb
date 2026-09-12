//! 无头协议 wire 类型 — 客户端请求 / 服务端响应 / 协议错误 / 事件通知。

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── 协议消息类型 ──

/// 接收到的客户端请求。
///
/// CDP 扁平协议：浏览器级命令不带 `sessionId`；附接目标上的命令携带
/// `Target.attachedToTarget` 分配的 `sessionId`，响应必须原样回显。
#[derive(Debug, Deserialize)]
pub(super) struct ClientRequest {
    /// 消息 ID，响应时原样返回。
    pub(super) id: u64,
    /// 命令方法名。
    pub(super) method: String,
    /// 命令参数。
    #[serde(default)]
    pub(super) params: Value,
    /// CDP 会话 ID（附接目标上的命令携带；缺省 = 浏览器级命令）。
    #[serde(default, rename = "sessionId")]
    pub(super) session_id: Option<String>,
}

/// 发送给客户端的响应。
#[derive(Debug, Serialize)]
pub(super) struct ServerResponse {
    /// 与请求对应的 ID。
    pub(super) id: u64,
    /// 返回值（成功时）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) result: Option<Value>,
    /// 错误信息（失败时）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) error: Option<ProtocolError>,
    /// 回显请求的 CDP 会话 ID（仅当请求携带时序列化）。
    #[serde(skip_serializing_if = "Option::is_none", rename = "sessionId")]
    pub(super) session_id: Option<String>,
}

/// 协议错误。
#[derive(Debug, Serialize)]
pub(super) struct ProtocolError {
    /// 错误码。
    pub(super) code: i64,
    /// 错误消息。
    pub(super) message: String,
}

/// 发送给客户端的事件通知（Phase 2 使用）。
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub(super) struct ServerEvent {
    /// 事件方法名。
    pub(super) method: String,
    /// 事件参数。
    pub(super) params: Value,
}

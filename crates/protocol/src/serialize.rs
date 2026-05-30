//! 二进制序列化与反序列化。

use crate::{IpcMessage, ProtocolError};

/// 将 IPC 消息序列化为二进制。
pub fn serialize(msg: &IpcMessage) -> Result<Vec<u8>, ProtocolError> {
    bincode::serialize(msg).map_err(|e| ProtocolError::Serialization(e.to_string()))
}

/// 从二进制反序列化 IPC 消息。
pub fn deserialize(data: &[u8]) -> Result<IpcMessage, ProtocolError> {
    bincode::deserialize(data).map_err(|e| ProtocolError::Deserialization(e.to_string()))
}

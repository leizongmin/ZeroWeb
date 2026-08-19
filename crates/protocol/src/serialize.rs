//! 二进制序列化与反序列化。

use crate::{IpcMessage, ProtocolError};

/// OPTIMIZATION（2026-08-19）：高频小消息跳过 size 预遍历。`bincode::serialize`
/// 内部先跑 `serialized_size()`（SizeChecker 完整遍历消息树）再精确 `with_capacity`
/// 写第二遍——对高频定长小消息（鼠标/键盘事件，序列化输出 ~14B < 48B 下限），
/// 预分配固定容量直接 `serialize_into` 一次写完，省掉整趟 size 遍历。
/// 背景：IpcMessageKind 变体 28→56（service-workers/IndexedDB 等合入）后 derive
/// 代码膨胀，ipc_roundtrip_10000 基线 348µs → 503µs（1.4x）。
/// 注意：只对**输出必然 ≤ 下限**的定长变体启用——带 String/Vec 的消息（Navigate、
/// Fetch、大 body）曾统一走 64B 起步 + Vec 倍增，多次 realloc + io::Write 泛型
/// 边界的开销反而使 ipc_serialize_10000（Navigate）219µs → 399µs，故此类消息
/// 保留原 `bincode::serialize` 路径（size 双遍历 + 精确容量一次成）。
#[inline]
fn fixed_capacity_for(msg: &IpcMessage) -> Option<usize> {
    use crate::IpcMessageKind;
    match &msg.kind {
        // 定长参数（f32×2 + u8 + 枚举等）：id(u64) + 变体索引(u32) + 参数 < 48B
        IpcMessageKind::MouseEvent(_) => Some(48),
        IpcMessageKind::KeyboardEvent(_) => Some(48),
        IpcMessageKind::ScrollEvent(_) => Some(48),
        // 其余消息含 String/Vec，输出大小不定 → None（走精确 size 路径）
        _ => None,
    }
}

/// 将 IPC 消息序列化为二进制。
pub fn serialize(msg: &IpcMessage) -> Result<Vec<u8>, ProtocolError> {
    if let Some(cap) = fixed_capacity_for(msg) {
        let mut buf = Vec::with_capacity(cap);
        bincode::serialize_into(&mut buf, msg).map_err(|e| ProtocolError::Serialization(e.to_string()))?;
        Ok(buf)
    } else {
        bincode::serialize(msg).map_err(|e| ProtocolError::Serialization(e.to_string()))
    }
}

/// 从二进制反序列化 IPC 消息。
pub fn deserialize(data: &[u8]) -> Result<IpcMessage, ProtocolError> {
    bincode::deserialize(data).map_err(|e| ProtocolError::Deserialization(e.to_string()))
}

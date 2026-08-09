//! 测试模块：将 5000+ 行测试拆分为 3 个子模块。

// 将 crate 根的所有公开项重新导出，供子模块 `use super::*;` 使用。
pub use crate::*;

/// IPC 消息序列化/反序列化往返辅助函数。
pub fn roundtrip(msg: IpcMessage) -> IpcMessage {
    let bytes = serialize(&msg).expect("serialize");
    deserialize(&bytes).expect("deserialize")
}

/// 基于 Vec 的内存 mock IpcChannel，用于验证 trait 契约。
pub struct MockChannel {
    queue: Vec<IpcMessage>,
    closed: bool,
}

impl MockChannel {
    /// 创建新的 MockChannel。
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            closed: false,
        }
    }
}

impl crate::IpcChannel for MockChannel {
    fn send(&mut self, msg: IpcMessage) -> Result<(), ProtocolError> {
        if self.closed {
            return Err(ProtocolError::Channel("channel closed".into()));
        }
        self.queue.push(msg);
        Ok(())
    }

    fn recv(&mut self) -> Result<IpcMessage, ProtocolError> {
        if self.closed {
            return Err(ProtocolError::Channel("channel closed".into()));
        }
        if self.queue.is_empty() {
            return Err(ProtocolError::Channel("empty".into()));
        }
        Ok(self.queue.remove(0))
    }

    fn try_recv(&mut self) -> Result<Option<IpcMessage>, ProtocolError> {
        if self.closed {
            return Err(ProtocolError::Channel("channel closed".into()));
        }
        if self.queue.is_empty() {
            return Ok(None);
        }
        Ok(Some(self.queue.remove(0)))
    }

    fn close(&mut self) {
        self.closed = true;
    }
}

mod channel_and_advanced;
mod compositor_protocol;
mod comprehensive_coverage;
mod edge_cases;
mod edge_cases_extra;
mod serialize_basic;
mod serialize_coverage;
mod serialize_exhaustive;

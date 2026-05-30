//! IPC 通道抽象。

use crate::{IpcMessage, ProtocolError};

/// IPC 通道抽象 — 定义进程间通信的接口。
/// 实际传输机制（管道、socket、共享内存）由宿主层实现。
pub trait IpcChannel {
    /// 发送消息。
    fn send(&mut self, msg: IpcMessage) -> Result<(), ProtocolError>;
    /// 接收消息（阻塞）。
    fn recv(&mut self) -> Result<IpcMessage, ProtocolError>;
    /// 尝试接收消息（非阻塞）。
    fn try_recv(&mut self) -> Result<Option<IpcMessage>, ProtocolError>;
    /// 关闭通道。
    fn close(&mut self);
}

/// 进程角色。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessRole {
    /// 浏览器主进程。
    Browser,
    /// 渲染进程。
    Renderer,
}

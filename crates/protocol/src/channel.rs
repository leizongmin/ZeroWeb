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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessRole {
    /// 浏览器主进程（UI、Tab、网络/存储策略）。
    Browser,
    /// 渲染进程（页面解析、布局、绘制、脚本；不直连网络）。
    Renderer,
    /// 网络服务进程（当前由 Browser 进程承载，后续可独立拆分）。
    Network,
    /// 图像解码进程（D1：PNG/JPEG/WebP 解码隔离，由渲染进程 spawn）。
    ImageDecoder,
}

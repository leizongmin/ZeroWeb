//! # zero-host-runtime
//!
//! 平台宿主 — 窗口、事件循环、surface、输入法。
//!
//! 基于 winit 提供跨平台窗口管理和事件循环。

#![warn(missing_docs)]

pub mod event;
pub mod window;

/// 宿主运行时错误
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    /// 窗口创建失败
    #[error("窗口创建失败: {0}")]
    WindowCreationFailed(String),
    /// GPU 设备请求失败
    #[error("GPU 设备请求失败: {0}")]
    GpuRequestFailed(String),
    /// 事件循环错误
    #[error("事件循环错误: {0}")]
    EventLoopError(String),
}

/// 宿主运行时结果
pub type HostResult<T> = Result<T, HostError>;

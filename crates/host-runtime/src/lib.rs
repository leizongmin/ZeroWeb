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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_error_window_creation_message() {
        let err = HostError::WindowCreationFailed("display not found".into());
        let msg = err.to_string();
        assert!(msg.contains("display not found"), "message: {msg}");
    }

    #[test]
    fn test_host_error_gpu_request_message() {
        let err = HostError::GpuRequestFailed("no adapter".into());
        let msg = err.to_string();
        assert!(msg.contains("no adapter"), "message: {msg}");
    }

    #[test]
    fn test_host_error_event_loop_message() {
        let err = HostError::EventLoopError("interrupted".into());
        let msg = err.to_string();
        assert!(msg.contains("interrupted"), "message: {msg}");
    }

    #[test]
    fn test_host_result_ok_and_err() {
        let ok: HostResult<()> = Ok(());
        assert!(ok.is_ok());

        let err: HostResult<()> = Err(HostError::EventLoopError("x".into()));
        assert!(err.is_err());
    }
}

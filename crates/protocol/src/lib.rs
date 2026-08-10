//! # zero-protocol
//!
//! 多进程 IPC 与协议定义。

#![warn(missing_docs)]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(test, allow(unused_variables))]
#![allow(clippy::approx_constant)]
#![allow(clippy::useless_vec)]

pub mod channel;
pub mod compositor_types;
pub mod frame_shm;
pub mod gpu_mailbox;
pub mod message;
pub mod paint_snapshot;
pub mod process;
pub mod serialize;
pub mod transport;

#[cfg(target_os = "linux")]
pub mod fd_socket_linux;

#[cfg(windows)]
pub mod job;

pub use channel::*;
pub use compositor_types::*;
pub use frame_shm::*;
pub use gpu_mailbox::*;
pub use message::*;
pub use paint_snapshot::*;
pub use process::*;
pub use serialize::*;
pub use transport::*;

use thiserror::Error;

/// 协议错误类型。
#[derive(Error, Debug)]
pub enum ProtocolError {
    /// 序列化错误。
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// 反序列化错误。
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    /// 通道错误。
    #[error("Channel error: {0}")]
    Channel(String),
    /// 进程错误。
    #[error("Process error: {0}")]
    Process(String),
}

impl ProtocolError {
    /// 是否为 IPC 对端断开导致的通道错误（可安全终止会话）。
    pub fn is_disconnected(&self) -> bool {
        match self {
            Self::Channel(msg) => crate::transport::is_disconnected_channel_message(msg),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests;

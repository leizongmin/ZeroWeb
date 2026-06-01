//! # zero-protocol
//!
//! 多进程 IPC 与协议定义。

#![warn(missing_docs)]

pub mod channel;
pub mod message;
pub mod serialize;

pub use channel::*;
pub use message::*;
pub use serialize::*;

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

#[cfg(test)]
mod tests;

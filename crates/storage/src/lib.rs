//! # zero-storage
//!
//! 存储后端 — localStorage、sessionStorage、IndexedDB、Cache API。

#![warn(missing_docs)]

pub mod indexed_db;
pub mod local_storage;
pub mod storage_manager;

pub use indexed_db::*;
pub use local_storage::*;
pub use storage_manager::*;

use thiserror::Error;

/// 存储操作错误类型。
#[derive(Error, Debug)]
pub enum StorageError {
    /// 超出配额限制。
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
    /// 无效键名。
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    /// Object Store 未找到。
    #[error("Store not found: {0}")]
    StoreNotFound(String),
    /// 键未找到。
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    /// 序列化错误。
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// 数据库错误。
    #[error("Database error: {0}")]
    Database(String),
}

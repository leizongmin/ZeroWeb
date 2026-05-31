//! # zero-storage
//!
//! 存储后端 — localStorage、sessionStorage、IndexedDB、Cache API。

#![warn(missing_docs)]

pub mod cache_api;
pub mod indexed_db;
pub mod local_storage;
pub mod storage_manager;

pub use cache_api::*;
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

#[cfg(test)]
mod tests {
    use crate::cache_api::{CacheRequest, CacheResponse, CacheStorage};
    use crate::indexed_db::{IdbDatabase, IdbKey};
    use crate::local_storage::{StorageType, WebStorage};

    /// 测试 IndexedDB 数据库名称存储是否正确。
    #[test]
    fn test_idb_database_name() {
        let db = IdbDatabase::new("my-app-db", 3);
        assert_eq!(db.name, "my-app-db");
        assert_eq!(db.version, 3);
        assert!(db.store_names().is_empty());
    }

    /// 测试 IndexedDB 创建 object store 后再删除，store_names 应为空。
    #[test]
    fn test_idb_delete_object_store() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", Some("id"), false).unwrap();
        assert_eq!(db.store_names().len(), 1);

        db.delete_object_store("items").unwrap();
        assert!(db.store_names().is_empty());
        assert!(!db.has_store("items"));
    }

    /// 测试 localStorage 更新已存在的键：设置 "a" 为 "1"，再设为 "2"，get 应返回 "2"。
    #[test]
    fn test_local_storage_update_existing() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("a", "1").unwrap();
        assert_eq!(storage.get("a"), Some("1"));

        storage.set("a", "2").unwrap();
        assert_eq!(storage.get("a"), Some("2"));
        assert_eq!(storage.len(), 1);
    }

    /// 测试 sessionStorage 设置多个键后调用 clear()，length 应为 0。
    #[test]
    fn test_session_storage_clear() {
        let mut storage = WebStorage::new(StorageType::Session, "https://example.com");
        storage.set("k1", "v1").unwrap();
        storage.set("k2", "v2").unwrap();
        storage.set("k3", "v3").unwrap();
        assert_eq!(storage.len(), 3);

        storage.clear();

        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
    }

    /// 测试在空 store 上打开游标并调用 advance，应返回 None。
    #[test]
    fn test_idb_cursor_with_empty_store() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("empty", None, false).unwrap();

        let result = db.open_cursor("empty", None).unwrap();
        assert!(result.is_none());
    }

    /// 测试 Cache API 对不存在的缓存名称调用 has() 返回 false。
    #[test]
    fn test_cache_api_has_nonexistent() {
        let mut cs = CacheStorage::new();
        cs.open("assets");

        assert!(cs.has("assets"));
        assert!(!cs.has("nonexistent"));
    }
}

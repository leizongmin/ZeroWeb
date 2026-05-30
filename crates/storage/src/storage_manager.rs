//! 存储管理器 — 管理多个源的 localStorage/sessionStorage。

use std::collections::HashMap;

use crate::local_storage::{StorageType, WebStorage};

/// localStorage 默认最大容量（5 MB）。
const DEFAULT_MAX_SIZE: usize = 5 * 1024 * 1024;

/// 存储管理器 — 管理多个源的 localStorage/sessionStorage。
pub struct StorageManager {
    /// localStorage 实例（按 origin 分组）。
    local_stores: HashMap<String, WebStorage>,
    /// sessionStorage 实例（按 origin 分组）。
    session_stores: HashMap<String, WebStorage>,
    /// 每个源的最大容量。
    default_max_size: usize,
}

impl StorageManager {
    /// 创建新的存储管理器。
    pub fn new() -> Self {
        Self::with_max_size(DEFAULT_MAX_SIZE)
    }

    /// 创建带自定义最大容量的存储管理器。
    pub fn with_max_size(default_max_size: usize) -> Self {
        Self {
            local_stores: HashMap::new(),
            session_stores: HashMap::new(),
            default_max_size,
        }
    }

    /// 获取指定源的 localStorage（如不存在则创建）。
    pub fn local_storage(&mut self, origin: &str) -> &mut WebStorage {
        self.local_stores
            .entry(origin.to_string())
            .or_insert_with(|| {
                WebStorage::new_with_max_size(StorageType::Local, origin, self.default_max_size)
            })
    }

    /// 获取指定源的 sessionStorage（如不存在则创建）。
    pub fn session_storage(&mut self, origin: &str) -> &mut WebStorage {
        self.session_stores
            .entry(origin.to_string())
            .or_insert_with(|| {
                WebStorage::new_with_max_size(StorageType::Session, origin, self.default_max_size)
            })
    }

    /// 清除指定源的所有存储。
    pub fn clear_origin(&mut self, origin: &str) {
        if let Some(store) = self.local_stores.get_mut(origin) {
            store.clear();
        }
        if let Some(store) = self.session_stores.get_mut(origin) {
            store.clear();
        }
    }

    /// 清除所有 localStorage。
    pub fn clear_all_local(&mut self) {
        for store in self.local_stores.values_mut() {
            store.clear();
        }
    }

    /// 清除所有 sessionStorage。
    pub fn clear_all_session(&mut self) {
        for store in self.session_stores.values_mut() {
            store.clear();
        }
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_local_storage() {
        let mut manager = StorageManager::new();
        let storage = manager.local_storage("https://example.com");
        assert_eq!(storage.origin(), "https://example.com");
        assert_eq!(storage.storage_type(), StorageType::Local);
    }

    #[test]
    fn test_manager_session_storage() {
        let mut manager = StorageManager::new();
        let storage = manager.session_storage("https://example.com");
        assert_eq!(storage.origin(), "https://example.com");
        assert_eq!(storage.storage_type(), StorageType::Session);
    }

    #[test]
    fn test_manager_different_origins() {
        let mut manager = StorageManager::new();
        manager
            .local_storage("https://a.com")
            .set("key", "a")
            .unwrap();
        manager
            .local_storage("https://b.com")
            .set("key", "b")
            .unwrap();

        assert_eq!(manager.local_storage("https://a.com").get("key"), Some("a"));
        assert_eq!(manager.local_storage("https://b.com").get("key"), Some("b"));
    }

    #[test]
    fn test_manager_clear_origin() {
        let mut manager = StorageManager::new();
        manager
            .local_storage("https://a.com")
            .set("key", "value")
            .unwrap();
        manager
            .session_storage("https://a.com")
            .set("sk", "sv")
            .unwrap();
        manager
            .local_storage("https://b.com")
            .set("key", "value")
            .unwrap();

        manager.clear_origin("https://a.com");

        assert!(manager.local_storage("https://a.com").is_empty());
        assert!(manager.session_storage("https://a.com").is_empty());
        assert!(!manager.local_storage("https://b.com").is_empty());
    }

    #[test]
    fn test_manager_clear_all() {
        let mut manager = StorageManager::new();
        manager
            .local_storage("https://a.com")
            .set("key", "value")
            .unwrap();
        manager
            .local_storage("https://b.com")
            .set("key", "value")
            .unwrap();
        manager
            .session_storage("https://a.com")
            .set("sk", "sv")
            .unwrap();

        manager.clear_all_local();
        assert!(manager.local_storage("https://a.com").is_empty());
        assert!(manager.local_storage("https://b.com").is_empty());

        // sessionStorage unaffected
        assert!(!manager.session_storage("https://a.com").is_empty());

        manager.clear_all_session();
        assert!(manager.session_storage("https://a.com").is_empty());
    }
}

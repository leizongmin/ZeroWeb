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
            .or_insert_with(|| WebStorage::new_with_max_size(StorageType::Local, origin, self.default_max_size))
    }

    /// 获取指定源的 sessionStorage（如不存在则创建）。
    pub fn session_storage(&mut self, origin: &str) -> &mut WebStorage {
        self.session_stores
            .entry(origin.to_string())
            .or_insert_with(|| WebStorage::new_with_max_size(StorageType::Session, origin, self.default_max_size))
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
        manager.local_storage("https://a.com").set("key", "a").unwrap();
        manager.local_storage("https://b.com").set("key", "b").unwrap();

        assert_eq!(manager.local_storage("https://a.com").get("key"), Some("a"));
        assert_eq!(manager.local_storage("https://b.com").get("key"), Some("b"));
    }

    #[test]
    fn test_manager_clear_origin() {
        let mut manager = StorageManager::new();
        manager.local_storage("https://a.com").set("key", "value").unwrap();
        manager.session_storage("https://a.com").set("sk", "sv").unwrap();
        manager.local_storage("https://b.com").set("key", "value").unwrap();

        manager.clear_origin("https://a.com");

        assert!(manager.local_storage("https://a.com").is_empty());
        assert!(manager.session_storage("https://a.com").is_empty());
        assert!(!manager.local_storage("https://b.com").is_empty());
    }

    #[test]
    fn test_manager_clear_all() {
        let mut manager = StorageManager::new();
        manager.local_storage("https://a.com").set("key", "value").unwrap();
        manager.local_storage("https://b.com").set("key", "value").unwrap();
        manager.session_storage("https://a.com").set("sk", "sv").unwrap();

        manager.clear_all_local();
        assert!(manager.local_storage("https://a.com").is_empty());
        assert!(manager.local_storage("https://b.com").is_empty());

        // sessionStorage unaffected
        assert!(!manager.session_storage("https://a.com").is_empty());

        manager.clear_all_session();
        assert!(manager.session_storage("https://a.com").is_empty());
    }

    // ── 新增测试 ──

    #[test]
    fn test_manager_default() {
        let mut manager = StorageManager::default();
        let storage = manager.local_storage("https://example.com");
        assert_eq!(storage.origin(), "https://example.com");
    }

    #[test]
    fn test_manager_per_origin_isolation() {
        let mut manager = StorageManager::new();
        manager.local_storage("https://a.com").set("x", "1").unwrap();
        manager.local_storage("https://b.com").set("x", "2").unwrap();
        manager.local_storage("https://c.com").set("x", "3").unwrap();

        assert_eq!(manager.local_storage("https://a.com").get("x"), Some("1"));
        assert_eq!(manager.local_storage("https://b.com").get("x"), Some("2"));
        assert_eq!(manager.local_storage("https://c.com").get("x"), Some("3"));

        // Removing from one origin does not affect others
        manager.local_storage("https://b.com").remove("x");
        assert_eq!(manager.local_storage("https://a.com").get("x"), Some("1"));
        assert_eq!(manager.local_storage("https://b.com").get("x"), None);
        assert_eq!(manager.local_storage("https://c.com").get("x"), Some("3"));
    }

    #[test]
    fn test_manager_session_cleared_per_origin() {
        let mut manager = StorageManager::new();
        manager.session_storage("https://a.com").set("s", "v1").unwrap();
        manager.session_storage("https://b.com").set("s", "v2").unwrap();
        manager.clear_origin("https://a.com");
        assert!(manager.session_storage("https://a.com").is_empty());
        assert!(!manager.session_storage("https://b.com").is_empty());
    }

    #[test]
    fn test_manager_local_and_session_independent() {
        let mut manager = StorageManager::new();
        manager.local_storage("https://a.com").set("key", "local-val").unwrap();
        manager
            .session_storage("https://a.com")
            .set("key", "session-val")
            .unwrap();
        assert_eq!(manager.local_storage("https://a.com").get("key"), Some("local-val"));
        assert_eq!(manager.session_storage("https://a.com").get("key"), Some("session-val"));
        // Clear local does not affect session
        manager.clear_all_local();
        assert!(manager.local_storage("https://a.com").is_empty());
        assert_eq!(manager.session_storage("https://a.com").get("key"), Some("session-val"));
    }

    #[test]
    fn test_manager_custom_max_size() {
        let mut manager = StorageManager::with_max_size(50);
        let result = manager.local_storage("https://example.com").set("k", &"x".repeat(50));
        assert!(result.is_err());
    }

    /// 测试 sessionStorage 源隔离：一个源的 sessionStorage 不会泄露到另一个源。
    #[test]
    fn test_session_storage_isolation() {
        let mut manager = StorageManager::new();

        // 两个不同源各自写入同名键
        manager
            .session_storage("https://alpha.com")
            .set("token", "aaa")
            .unwrap();
        manager.session_storage("https://beta.com").set("token", "bbb").unwrap();

        // 各自只能看到自己的数据
        assert_eq!(manager.session_storage("https://alpha.com").get("token"), Some("aaa"));
        assert_eq!(manager.session_storage("https://beta.com").get("token"), Some("bbb"));

        // 清除一个源不影响另一个
        manager.clear_origin("https://alpha.com");
        assert!(manager.session_storage("https://alpha.com").is_empty());
        assert_eq!(manager.session_storage("https://beta.com").get("token"), Some("bbb"));
    }
}

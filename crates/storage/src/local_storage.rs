//! Web Storage 实现 — localStorage 和 sessionStorage。

use std::collections::HashMap;

use crate::StorageError;

/// localStorage 默认最大容量（5 MB）。
const DEFAULT_MAX_SIZE: usize = 5 * 1024 * 1024;

/// 存储类型。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageType {
    /// localStorage — 持久存储，关闭浏览器后数据仍保留。
    Local,
    /// sessionStorage — 会话存储，关闭标签页后数据清除。
    Session,
}

/// Web Storage 实现 — localStorage 和 sessionStorage。
pub struct WebStorage {
    /// 存储数据。
    data: HashMap<String, String>,
    /// 存储类型。
    storage_type: StorageType,
    /// 所属源。
    origin: String,
    /// 最大容量（字节数）。
    max_size: usize,
}

impl WebStorage {
    /// 创建新的 WebStorage 实例。
    pub fn new(storage_type: StorageType, origin: &str) -> Self {
        Self::new_with_max_size(storage_type, origin, DEFAULT_MAX_SIZE)
    }

    /// 创建带自定义最大容量的 WebStorage 实例。
    pub fn new_with_max_size(storage_type: StorageType, origin: &str, max_size: usize) -> Self {
        Self {
            data: HashMap::new(),
            storage_type,
            origin: origin.to_string(),
            max_size,
        }
    }

    /// 获取项。
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    /// 设置项（返回旧值）。
    pub fn set(&mut self, key: &str, value: &str) -> Result<Option<String>, StorageError> {
        if key.is_empty() {
            return Err(StorageError::InvalidKey("key cannot be empty".to_string()));
        }

        let new_entry_size = key.len() + value.len();
        let old_size = self
            .data
            .get(key)
            .map(|old| key.len() + old.len())
            .unwrap_or(0);
        let used_after = self.used_size() - old_size + new_entry_size;

        if used_after > self.max_size {
            return Err(StorageError::QuotaExceeded(format!(
                "used {} bytes exceeds max {} bytes",
                used_after, self.max_size
            )));
        }

        Ok(self.data.insert(key.to_string(), value.to_string()))
    }

    /// 移除项（返回旧值）。
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }

    /// 清空所有数据。
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// 键数量。
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 获取指定索引的键名。
    pub fn key(&self, index: usize) -> Option<&str> {
        self.data.keys().nth(index).map(|s| s.as_str())
    }

    /// 是否包含键。
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// 估算已用字节数。
    pub fn used_size(&self) -> usize {
        self.data
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum()
    }

    /// 获取存储类型。
    pub fn storage_type(&self) -> StorageType {
        self.storage_type
    }

    /// 获取源。
    pub fn origin(&self) -> &str {
        &self.origin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_new_local() {
        let storage = WebStorage::new(StorageType::Local, "https://example.com");
        assert_eq!(storage.storage_type(), StorageType::Local);
        assert_eq!(storage.origin(), "https://example.com");
        assert!(storage.is_empty());
    }

    #[test]
    fn test_storage_new_session() {
        let storage = WebStorage::new(StorageType::Session, "https://example.com");
        assert_eq!(storage.storage_type(), StorageType::Session);
        assert_eq!(storage.origin(), "https://example.com");
    }

    #[test]
    fn test_storage_set_get() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("key1", "value1").unwrap();
        assert_eq!(storage.get("key1"), Some("value1"));
        assert_eq!(storage.get("nonexistent"), None);
    }

    #[test]
    fn test_storage_set_returns_old() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        assert_eq!(storage.set("key1", "value1").unwrap(), None);
        assert_eq!(storage.set("key1", "value2").unwrap(), Some("value1".to_string()));
        assert_eq!(storage.get("key1"), Some("value2"));
    }

    #[test]
    fn test_storage_remove() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("key1", "value1").unwrap();
        assert_eq!(storage.remove("key1"), Some("value1".to_string()));
        assert_eq!(storage.get("key1"), None);
    }

    #[test]
    fn test_storage_remove_nonexistent() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        assert_eq!(storage.remove("nonexistent"), None);
    }

    #[test]
    fn test_storage_clear() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("key1", "value1").unwrap();
        storage.set("key2", "value2").unwrap();
        storage.clear();
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn test_storage_len() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        assert_eq!(storage.len(), 0);
        storage.set("key1", "value1").unwrap();
        assert_eq!(storage.len(), 1);
        storage.set("key2", "value2").unwrap();
        assert_eq!(storage.len(), 2);
    }

    #[test]
    fn test_storage_is_empty() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        assert!(storage.is_empty());
        storage.set("key1", "value1").unwrap();
        assert!(!storage.is_empty());
    }

    #[test]
    fn test_storage_key_index() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("alpha", "1").unwrap();
        storage.set("beta", "2").unwrap();
        // key() returns by insertion-order index (HashMap iteration order)
        let keys: Vec<&str> = (0..storage.len()).filter_map(|i| storage.key(i)).collect();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"alpha"));
        assert!(keys.contains(&"beta"));
        assert_eq!(storage.key(99), None);
    }

    #[test]
    fn test_storage_contains_key() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        assert!(!storage.contains_key("key1"));
        storage.set("key1", "value1").unwrap();
        assert!(storage.contains_key("key1"));
    }

    #[test]
    fn test_storage_quota_exceeded() {
        let mut storage = WebStorage::new_with_max_size(
            StorageType::Local,
            "https://example.com",
            100,
        );
        // Each entry costs key.len() + value.len()
        storage.set("a", &"x".repeat(49)).unwrap(); // 1 + 49 = 50
        storage.set("b", &"y".repeat(49)).unwrap(); // 1 + 49 = 50, total = 100
        let result = storage.set("c", "z");
        assert!(result.is_err());
    }

    #[test]
    fn test_storage_invalid_key() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        let result = storage.set("", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_storage_used_size() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        assert_eq!(storage.used_size(), 0);
        storage.set("abc", "12345").unwrap(); // 3 + 5 = 8
        assert_eq!(storage.used_size(), 8);
    }
}

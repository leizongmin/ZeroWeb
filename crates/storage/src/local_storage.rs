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
        let old_size = self.data.get(key).map(|old| key.len() + old.len()).unwrap_or(0);
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
        self.data.iter().map(|(k, v)| k.len() + v.len()).sum()
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
        let mut storage = WebStorage::new_with_max_size(StorageType::Local, "https://example.com", 100);
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

    // ── 新增测试 ──

    #[test]
    fn test_storage_clear_resets_size() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("key1", "value1").unwrap();
        storage.set("key2", "value2").unwrap();
        assert!(storage.used_size() > 0);
        storage.clear();
        assert_eq!(storage.used_size(), 0);
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn test_storage_key_enumeration_all() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("x", "1").unwrap();
        storage.set("y", "2").unwrap();
        storage.set("z", "3").unwrap();
        let mut keys: Vec<&str> = (0..storage.len()).filter_map(|i| storage.key(i)).collect();
        keys.sort();
        assert_eq!(keys, vec!["x", "y", "z"]);
    }

    #[test]
    fn test_storage_length_tracking_after_remove() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("a", "1").unwrap();
        storage.set("b", "2").unwrap();
        assert_eq!(storage.len(), 2);
        storage.remove("a");
        assert_eq!(storage.len(), 1);
        storage.remove("b");
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
    }

    #[test]
    fn test_storage_large_value() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        let large = "x".repeat(100_000);
        storage.set("big", &large).unwrap();
        assert_eq!(storage.get("big"), Some(large.as_str()));
        assert_eq!(storage.used_size(), 3 + 100_000); // "big" + value
    }

    #[test]
    fn test_storage_json_roundtrip() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        let obj = serde_json::json!({"name": "Alice", "age": 30, "tags": [1, 2, 3]});
        storage.set("user", &obj.to_string()).unwrap();
        let retrieved: serde_json::Value = serde_json::from_str(storage.get("user").unwrap()).unwrap();
        assert_eq!(retrieved, obj);
    }

    #[test]
    fn test_storage_null_undefined_as_strings() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("null_val", "null").unwrap();
        storage.set("undefined_val", "undefined").unwrap();
        assert_eq!(storage.get("null_val"), Some("null"));
        assert_eq!(storage.get("undefined_val"), Some("undefined"));
    }

    #[test]
    fn test_storage_quota_with_update() {
        let mut storage = WebStorage::new_with_max_size(StorageType::Local, "https://example.com", 50);
        storage.set("k", &"a".repeat(48)).unwrap(); // 1 + 48 = 49
        // Updating same key to larger value that exceeds quota
        let result = storage.set("k", &"b".repeat(50));
        assert!(result.is_err());
        // Original value preserved
        assert_eq!(storage.get("k"), Some("a".repeat(48).as_str()));
    }

    /// key(index) 当 index >= length 时应返回 None
    #[test]
    fn test_web_storage_key_out_of_bounds() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("a", "1").unwrap();
        storage.set("b", "2").unwrap();
        assert_eq!(storage.len(), 2);
        // 索引等于长度 → None
        assert_eq!(storage.key(2), None);
        // 索引远超长度 → None
        assert_eq!(storage.key(100), None);
        // 空存储时索引 0 → None
        storage.clear();
        assert_eq!(storage.key(0), None);
    }

    /// 设置多个键、删除其中一个后，used_size 应准确反映剩余数据
    #[test]
    fn test_web_storage_used_size_accuracy() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("key1", "aaaa").unwrap(); // 4 + 4 = 8
        storage.set("key2", "bbbbb").unwrap(); // 4 + 5 = 9
        storage.set("key3", "cccccc").unwrap(); // 4 + 6 = 10
        assert_eq!(storage.used_size(), 27); // 8 + 9 + 10

        storage.remove("key2");
        assert_eq!(storage.used_size(), 18); // 8 + 10
        assert_eq!(storage.len(), 2);
    }

    /// 设置一个恰好填满 max_size 的键值对应成功
    #[test]
    fn test_web_storage_set_exactly_at_limit() {
        let max_size = 100;
        let mut storage = WebStorage::new_with_max_size(StorageType::Local, "https://example.com", max_size);
        // key 长度 3，value 长度 97，合计 100，恰好等于 max_size
        let value = "x".repeat(97);
        let result = storage.set("key", &value);
        assert!(result.is_ok(), "恰好等于 max_size 时应成功");
        assert_eq!(storage.used_size(), 100);

        // 超出 1 字节应失败
        let result2 = storage.set("k2", "y");
        assert!(result2.is_err(), "超出 max_size 时应失败");
    }

    /// 清空所有数据后重新设置新项，验证干净状态。
    #[test]
    fn test_web_storage_clear_then_set() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        // 初始数据
        storage.set("old_key1", "old_value1").unwrap();
        storage.set("old_key2", "old_value2").unwrap();
        storage.set("old_key3", "old_value3").unwrap();
        assert_eq!(storage.len(), 3);
        assert_eq!(storage.used_size(), 3 * (8 + 10)); // 3 * ("old_keyN" + "old_valueN") = 54

        // 清空
        storage.clear();
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.used_size(), 0);
        assert_eq!(storage.get("old_key1"), None);
        assert_eq!(storage.get("old_key2"), None);
        assert_eq!(storage.get("old_key3"), None);

        // 重新设置全新的键值对
        storage.set("new_a", "alpha").unwrap();
        storage.set("new_b", "beta").unwrap();
        assert_eq!(storage.len(), 2);
        assert!(!storage.is_empty());
        assert_eq!(storage.get("new_a"), Some("alpha"));
        assert_eq!(storage.get("new_b"), Some("beta"));
        // 旧数据不应残留
        assert_eq!(storage.get("old_key1"), None);
        assert_eq!(storage.contains_key("old_key1"), false);
        // used_size 应只反映新数据
        assert_eq!(storage.used_size(), (5 + 5) + (5 + 4)); // "new_a"+"alpha" + "new_b"+"beta"
    }

    /// 测试 localStorage clear() 操作：设置多个项，调用 clear()，验证全部被清除。
    #[test]
    fn test_local_storage_clear() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("user", "alice").unwrap();
        storage.set("theme", "dark").unwrap();
        storage.set("lang", "zh").unwrap();
        assert_eq!(storage.len(), 3);
        assert!(storage.used_size() > 0);

        storage.clear();

        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
        assert_eq!(storage.used_size(), 0);
        assert_eq!(storage.get("user"), None);
        assert_eq!(storage.get("theme"), None);
        assert_eq!(storage.get("lang"), None);
    }
}

//! Web Storage 实现 — localStorage 和 sessionStorage。

use indexmap::IndexMap;

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
    /// 存储数据——IndexMap 保留插入序（WHATWG Web Storage §4.1：key(n) 须按插入序返回；
    /// setItem 既有键**保留位置**，新键追加末尾；removeItem 后剩余键序不变，重加追加末尾）。
    data: IndexMap<String, String>,
    /// 存储类型。
    storage_type: StorageType,
    /// 所属源。
    origin: String,
    /// 最大容量（字节数）。
    max_size: usize,
    /// 当前已用字节数（增量维护，避免 O(n) 遍历）。
    used_bytes: usize,
}

impl WebStorage {
    /// 创建新的 WebStorage 实例。
    pub fn new(storage_type: StorageType, origin: &str) -> Self {
        Self::new_with_max_size(storage_type, origin, DEFAULT_MAX_SIZE)
    }

    /// 创建带自定义最大容量的 WebStorage 实例。
    pub fn new_with_max_size(storage_type: StorageType, origin: &str, max_size: usize) -> Self {
        Self {
            data: IndexMap::new(),
            storage_type,
            origin: origin.to_string(),
            max_size,
            used_bytes: 0,
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
        let used_after = self.used_bytes - old_size + new_entry_size;

        if used_after > self.max_size {
            return Err(StorageError::QuotaExceeded(format!(
                "used {} bytes exceeds max {} bytes",
                used_after, self.max_size
            )));
        }

        self.used_bytes = used_after;
        Ok(self.data.insert(key.to_string(), value.to_string()))
    }

    /// 移除项（返回旧值）。
    pub fn remove(&mut self, key: &str) -> Option<String> {
        // shift_remove 保留剩余键的插入序（swap_remove 会重排）；R3226 Web Storage §4.1 插入序。
        let old = self.data.shift_remove(key)?;
        self.used_bytes = self.used_bytes.saturating_sub(key.len() + old.len());
        Some(old)
    }

    /// 清空所有数据。
    pub fn clear(&mut self) {
        self.data.clear();
        self.used_bytes = 0;
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
        self.used_bytes
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
        // R3226：key(n) 按插入序返回（IndexMap 保留插入序，非 HashMap 任意序）。
        let keys: Vec<&str> = (0..storage.len()).filter_map(|i| storage.key(i)).collect();
        assert_eq!(keys, vec!["alpha", "beta"]);
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
        // R3226：插入序（旧 HashMap 须 sort 后比对，现 IndexMap 保留插入序）。
        let keys: Vec<&str> = (0..storage.len()).filter_map(|i| storage.key(i)).collect();
        assert_eq!(keys, vec!["x", "y", "z"]);
    }

    /// R3226：Web Storage key(n) 插入序——WHATWG Web Storage §4.1。
    /// setItem 既有键**保留位置**（不移到末尾）；removeItem 后剩余键序不变；删除后重加**追加末尾**。
    #[test]
    fn test_storage_insertion_order_r3226() {
        fn keys_of(s: &WebStorage) -> Vec<&str> {
            (0..s.len()).filter_map(|i| s.key(i)).collect()
        }
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        // 初始插入序：z, a, m（非字典序）。
        storage.set("z", "1").unwrap();
        storage.set("a", "2").unwrap();
        storage.set("m", "3").unwrap();
        assert_eq!(keys_of(&storage), vec!["z", "a", "m"], "key(n) 须按插入序（非字典序）");

        // setItem 既有键：保留位置，不移到末尾。
        storage.set("z", "updated").unwrap();
        assert_eq!(keys_of(&storage), vec!["z", "a", "m"], "更新既有键须保留位置");
        assert_eq!(storage.get("z"), Some("updated"));

        // removeItem 中间键：剩余键序不变。
        storage.remove("a").unwrap();
        assert_eq!(keys_of(&storage), vec!["z", "m"], "removeItem 后剩余键序不变");

        // 删除后重加：追加末尾（spec 插入序语义）。
        storage.set("a", "re-added").unwrap();
        assert_eq!(keys_of(&storage), vec!["z", "m", "a"], "删除后重加须追加末尾");

        // 新键始终追加末尾。
        storage.set("new", "4").unwrap();
        assert_eq!(keys_of(&storage), vec!["z", "m", "a", "new"]);
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

    /// 测试移除不存在的键 → 不应报错，返回 None。
    #[test]
    fn test_web_storage_remove_nonexistent() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        // 移除从未设置过的键
        let result = storage.remove("nonexistent_key");
        assert_eq!(result, None, "移除不存在的键应返回 None，不应 panic");

        // 设置后移除，再移除一次
        storage.set("temp", "value").unwrap();
        assert_eq!(storage.remove("temp"), Some("value".to_string()));
        // 第二次移除同一键（已不存在）
        assert_eq!(storage.remove("temp"), None, "重复移除已删除的键应返回 None");

        // 空存储上移除
        storage.clear();
        assert_eq!(storage.remove("any_key"), None, "清空后移除应返回 None");
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

    /// 测试 sessionStorage 在 max_size 限制下，用更短值覆盖已满的存储后可以再添加新键。
    #[test]
    fn test_storage_shrink_then_add_within_quota() {
        let mut storage = WebStorage::new_with_max_size(StorageType::Session, "https://example.com", 20);

        // "k" (1) + "1234567890123456789" (19) = 20 → 恰好填满
        storage.set("k", "1234567890123456789").unwrap();
        assert_eq!(storage.used_size(), 20);

        // 再添加任何新键都会超出配额
        assert!(storage.set("x", "y").is_err());

        // 用更短的值覆盖 → 释放空间
        let old = storage.set("k", "ab").unwrap();
        assert_eq!(old, Some("1234567890123456789".to_string()));
        assert_eq!(storage.used_size(), 3); // "k"(1) + "ab"(2) = 3

        // 现在可以添加新键了
        storage.set("x", "y").unwrap(); // 1 + 1 = 2，合计 5
        assert_eq!(storage.len(), 2);
        assert_eq!(storage.used_size(), 5);
    }

    // ── localStorage 边界条件测试 ──

    /// 测试 WebStorage 空键名被拒绝。
    #[test]
    fn test_web_storage_empty_key_rejected() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        let result = storage.set("", "value");
        assert!(result.is_err());
        if let Err(e) = result {
            let msg = format!("{e}");
            assert!(msg.contains("empty") || msg.contains("cannot be empty"));
        }
    }

    /// 测试 WebStorage 多次 clear 后状态一致。
    #[test]
    fn test_web_storage_multiple_clear() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");

        // 第一次 clear（空存储）
        storage.clear();
        assert!(storage.is_empty());
        assert_eq!(storage.used_size(), 0);

        // 写入数据后 clear
        storage.set("a", "1").unwrap();
        storage.clear();
        assert!(storage.is_empty());

        // 再次 clear（已清空）
        storage.clear();
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.used_size(), 0);
    }

    /// 测试 WebStorage 用更长的值覆盖已有键，used_size 应增加。
    #[test]
    fn test_web_storage_overwrite_with_longer_value() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("k", "a").unwrap(); // 1 + 1 = 2
        assert_eq!(storage.used_size(), 2);

        let old = storage.set("k", "abcdef").unwrap(); // 1 + 6 = 7
        assert_eq!(old, Some("a".to_string()));
        assert_eq!(storage.used_size(), 7);
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.get("k"), Some("abcdef"));
    }

    /// 测试 WebStorage 配额限制下更新值为恰好等于配额的值。
    #[test]
    fn test_web_storage_quota_exact_update() {
        let mut storage = WebStorage::new_with_max_size(StorageType::Local, "test", 20);

        // 初始占 10 字节
        storage.set("ab", "12345678").unwrap(); // 2 + 8 = 10

        // 更新为恰好占 20 字节（填满配额）
        let result = storage.set("ab", "123456789012345678");
        assert!(result.is_ok(), "更新到恰好等于配额应成功");
        assert_eq!(storage.used_size(), 20);

        // 再加任何新键都应失败
        assert!(storage.set("x", "y").is_err());
    }

    /// 测试 WebStorage contains_key 对空字符串键名。
    #[test]
    fn test_web_storage_contains_empty_key() {
        let storage = WebStorage::new(StorageType::Local, "https://example.com");
        // 空键名不允许 set，所以 contains_key("") 应始终为 false
        assert!(!storage.contains_key(""));
    }

    /// 测试 WebStorage 在极小配额（1 字节）下的行为。
    #[test]
    fn test_web_storage_tiny_quota() {
        let mut storage = WebStorage::new_with_max_size(StorageType::Session, "test", 1);
        // 键长度 1 + 值长度 0 = 1，恰好等于配额
        let result = storage.set("k", "");
        assert!(result.is_ok(), "1 字节配额 + 空值应成功");
        assert_eq!(storage.used_size(), 1);

        // 任何其他操作都会超配额
        assert!(storage.set("k", "x").is_err());
        assert!(storage.set("a", "b").is_err());
    }

    /// 测试 WebStorage remove 返回被删除的旧值。
    #[test]
    fn test_web_storage_remove_returns_old_value() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("key1", "value1").unwrap();
        storage.set("key2", "value2").unwrap();

        let old1 = storage.remove("key1");
        assert_eq!(old1, Some("value1".to_string()));

        let old2 = storage.remove("key2");
        assert_eq!(old2, Some("value2".to_string()));

        assert!(storage.is_empty());
    }

    /// 测试 WebStorage set 返回旧值的完整语义。
    #[test]
    fn test_web_storage_set_returns_old_value_semantics() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");

        // 首次设置：返回 None
        let result = storage.set("token", "abc");
        assert_eq!(result.unwrap(), None);

        // 覆盖：返回旧值
        let result = storage.set("token", "def");
        assert_eq!(result.unwrap(), Some("abc".to_string()));

        // 再次覆盖：返回上一次的值
        let result = storage.set("token", "ghi");
        assert_eq!(result.unwrap(), Some("def".to_string()));

        // 删除后重新设置：返回 None
        storage.remove("token");
        let result = storage.set("token", "jkl");
        assert_eq!(result.unwrap(), None);
    }

    /// 测试 WebStorage key() 在 remove 后索引保持有效。
    #[test]
    fn test_web_storage_key_after_remove() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("a", "1").unwrap();
        storage.set("b", "2").unwrap();
        storage.set("c", "3").unwrap();

        // 删除中间的键
        storage.remove("b");
        assert_eq!(storage.len(), 2);

        // R3226：剩余键保留插入序（a, c），无需 sort。
        let keys: Vec<String> = (0..storage.len())
            .filter_map(|i| storage.key(i).map(|s| s.to_string()))
            .collect();
        assert_eq!(keys, vec!["a", "c"]);
    }

    /// 测试 WebStorage 大量键的 set/get 性能（回归测试）。
    #[test]
    fn test_web_storage_many_keys() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        let count = 500;

        for i in 0..count {
            storage.set(&format!("key-{i}"), &format!("value-{i}")).unwrap();
        }
        assert_eq!(storage.len(), count);

        // 随机验证几个
        assert_eq!(storage.get("key-0"), Some("value-0"));
        assert_eq!(storage.get("key-250"), Some("value-250"));
        assert_eq!(storage.get("key-499"), Some("value-499"));
        assert_eq!(storage.get("key-500"), None);

        // clear 后全部清空
        storage.clear();
        assert!(storage.is_empty());
        assert_eq!(storage.get("key-0"), None);
    }
}

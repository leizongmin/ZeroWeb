//! IndexedDB 基础实现。

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::StorageError;

/// IndexedDB 键类型。
#[derive(Debug, Clone, PartialEq)]
pub enum IdbKey {
    /// 数值键。
    Number(f64),
    /// 字符串键。
    String(String),
    /// 二进制键。
    Binary(Vec<u8>),
    /// 数组键（复合键）。
    Array(Vec<IdbKey>),
}

impl IdbKey {
    /// 内部比较辅助，返回 Ordering。
    fn cmp_key(&self, other: &Self) -> Ordering {
        match (self, other) {
            (IdbKey::Number(a), IdbKey::Number(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (IdbKey::Number(_), IdbKey::String(_)) => Ordering::Less,
            (IdbKey::Number(_), IdbKey::Binary(_)) => Ordering::Less,
            (IdbKey::Number(_), IdbKey::Array(_)) => Ordering::Less,

            (IdbKey::String(_), IdbKey::Number(_)) => Ordering::Greater,
            (IdbKey::String(a), IdbKey::String(b)) => a.cmp(b),
            (IdbKey::String(_), IdbKey::Binary(_)) => Ordering::Less,
            (IdbKey::String(_), IdbKey::Array(_)) => Ordering::Less,

            (IdbKey::Binary(_), IdbKey::Number(_)) => Ordering::Greater,
            (IdbKey::Binary(_), IdbKey::String(_)) => Ordering::Greater,
            (IdbKey::Binary(a), IdbKey::Binary(b)) => a.cmp(b),
            (IdbKey::Binary(_), IdbKey::Array(_)) => Ordering::Less,

            (IdbKey::Array(_), IdbKey::Number(_)) => Ordering::Greater,
            (IdbKey::Array(_), IdbKey::String(_)) => Ordering::Greater,
            (IdbKey::Array(_), IdbKey::Binary(_)) => Ordering::Greater,
            (IdbKey::Array(a), IdbKey::Array(b)) => {
                for (ak, bk) in a.iter().zip(b.iter()) {
                    match ak.cmp_key(bk) {
                        Ordering::Equal => continue,
                        other => return other,
                    }
                }
                a.len().cmp(&b.len())
            }
        }
    }
}

impl PartialOrd for IdbKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

impl Eq for IdbKey {}

impl Ord for IdbKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_key(other)
    }
}

/// 数据记录。
#[derive(Debug, Clone, PartialEq)]
pub struct IdbRecord {
    /// 主键值。
    pub key: IdbKey,
    /// JSON 值。
    pub value: serde_json::Value,
}

/// Object Store（对象仓库）。
pub struct IdbObjectStore {
    /// 仓库名称。
    pub name: String,
    /// 主键路径。
    pub key_path: Option<String>,
    /// 是否自增主键。
    pub auto_increment: bool,
    /// 数据记录。
    records: Vec<IdbRecord>,
    /// 自增计数器。
    next_key: u64,
}

/// 事务模式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IdbTransactionMode {
    /// 只读事务。
    ReadOnly,
    /// 读写事务。
    ReadWrite,
}

/// IndexedDB 数据库。
pub struct IdbDatabase {
    /// 数据库名称。
    pub name: String,
    /// 版本号。
    pub version: u32,
    /// Object stores。
    stores: HashMap<String, IdbObjectStore>,
}

impl IdbDatabase {
    /// 创建新的 IndexedDB 数据库。
    pub fn new(name: &str, version: u32) -> Self {
        Self {
            name: name.to_string(),
            version,
            stores: HashMap::new(),
        }
    }

    /// 创建 Object Store。
    pub fn create_object_store(
        &mut self,
        name: &str,
        key_path: Option<&str>,
        auto_increment: bool,
    ) -> Result<(), StorageError> {
        if self.stores.contains_key(name) {
            return Err(StorageError::Database(format!(
                "Object store '{}' already exists",
                name
            )));
        }
        self.stores.insert(
            name.to_string(),
            IdbObjectStore {
                name: name.to_string(),
                key_path: key_path.map(|s| s.to_string()),
                auto_increment,
                records: Vec::new(),
                next_key: 1,
            },
        );
        Ok(())
    }

    /// 删除 Object Store。
    pub fn delete_object_store(&mut self, name: &str) -> Result<(), StorageError> {
        if self.stores.remove(name).is_none() {
            return Err(StorageError::StoreNotFound(name.to_string()));
        }
        Ok(())
    }

    /// 获取 Object Store 名称列表。
    pub fn store_names(&self) -> Vec<&str> {
        self.stores.keys().map(|s| s.as_str()).collect()
    }

    /// 是否包含指定 Object Store。
    pub fn has_store(&self, name: &str) -> bool {
        self.stores.contains_key(name)
    }

    /// 在指定 store 中添加记录（如主键已存在则报错）。
    pub fn add(
        &mut self,
        store_name: &str,
        value: serde_json::Value,
        key: Option<IdbKey>,
    ) -> Result<IdbKey, StorageError> {
        let store = self.stores.get_mut(store_name).ok_or_else(|| {
            StorageError::StoreNotFound(store_name.to_string())
        })?;

        let key = match key {
            Some(k) => k,
            None if store.auto_increment => {
                let k = IdbKey::Number(store.next_key as f64);
                store.next_key += 1;
                k
            }
            None => {
                return Err(StorageError::Database(
                    "No key provided and auto_increment is false".to_string(),
                ));
            }
        };

        // Check for duplicate key
        if store.records.iter().any(|r| r.key == key) {
            return Err(StorageError::Database(format!(
                "Key already exists in store '{}'",
                store_name
            )));
        }

        store.records.push(IdbRecord { key: key.clone(), value });
        Ok(key)
    }

    /// 在指定 store 中放入记录（覆盖已有记录）。
    pub fn put(
        &mut self,
        store_name: &str,
        value: serde_json::Value,
        key: Option<IdbKey>,
    ) -> Result<IdbKey, StorageError> {
        let store = self.stores.get_mut(store_name).ok_or_else(|| {
            StorageError::StoreNotFound(store_name.to_string())
        })?;

        let key = match key {
            Some(k) => k,
            None if store.auto_increment => {
                let k = IdbKey::Number(store.next_key as f64);
                store.next_key += 1;
                k
            }
            None => {
                return Err(StorageError::Database(
                    "No key provided and auto_increment is false".to_string(),
                ));
            }
        };

        // Overwrite if key exists
        if let Some(record) = store.records.iter_mut().find(|r| r.key == key) {
            record.value = value;
        } else {
            store.records.push(IdbRecord { key: key.clone(), value });
        }

        Ok(key)
    }

    /// 获取记录。
    pub fn get(&self, store_name: &str, key: &IdbKey) -> Option<&IdbRecord> {
        self.stores
            .get(store_name)?
            .records
            .iter()
            .find(|r| &r.key == key)
    }

    /// 删除记录。
    pub fn delete(&mut self, store_name: &str, key: &IdbKey) -> Result<bool, StorageError> {
        let store = self.stores.get_mut(store_name).ok_or_else(|| {
            StorageError::StoreNotFound(store_name.to_string())
        })?;
        let before = store.records.len();
        store.records.retain(|r| &r.key != key);
        Ok(store.records.len() < before)
    }

    /// 获取 store 中所有记录。
    pub fn get_all(&self, store_name: &str) -> Result<Vec<&IdbRecord>, StorageError> {
        let store = self.stores.get(store_name).ok_or_else(|| {
            StorageError::StoreNotFound(store_name.to_string())
        })?;
        Ok(store.records.iter().collect())
    }

    /// 清空 store 中所有记录。
    pub fn clear_store(&mut self, store_name: &str) -> Result<(), StorageError> {
        let store = self.stores.get_mut(store_name).ok_or_else(|| {
            StorageError::StoreNotFound(store_name.to_string())
        })?;
        store.records.clear();
        Ok(())
    }

    /// 获取 store 中记录数量。
    pub fn count(&self, store_name: &str) -> Result<usize, StorageError> {
        let store = self.stores.get(store_name).ok_or_else(|| {
            StorageError::StoreNotFound(store_name.to_string())
        })?;
        Ok(store.records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idb_database_new() {
        let db = IdbDatabase::new("testdb", 1);
        assert_eq!(db.name, "testdb");
        assert_eq!(db.version, 1);
        assert!(db.store_names().is_empty());
    }

    #[test]
    fn test_idb_create_store() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("users", Some("id"), false).unwrap();
        assert!(db.has_store("users"));
        assert_eq!(db.store_names().len(), 1);
    }

    #[test]
    fn test_idb_delete_store() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("users", Some("id"), false).unwrap();
        db.delete_object_store("users").unwrap();
        assert!(!db.has_store("users"));
    }

    #[test]
    fn test_idb_store_names() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("a", None, false).unwrap();
        db.create_object_store("b", None, false).unwrap();
        let names = db.store_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }

    #[test]
    fn test_idb_add_record() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("users", Some("id"), false).unwrap();
        let key = IdbKey::String("user1".to_string());
        let value = serde_json::json!({"name": "Alice"});
        let returned_key = db.add("users", value, Some(key.clone())).unwrap();
        assert_eq!(returned_key, key);

        let record = db.get("users", &IdbKey::String("user1".to_string())).unwrap();
        assert_eq!(record.value["name"], "Alice");
    }

    #[test]
    fn test_idb_add_with_auto_key() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("items", None, true).unwrap();

        let k1 = db.add("items", serde_json::json!({"v": 1}), None).unwrap();
        let k2 = db.add("items", serde_json::json!({"v": 2}), None).unwrap();

        assert_eq!(k1, IdbKey::Number(1.0));
        assert_eq!(k2, IdbKey::Number(2.0));
        assert_eq!(db.count("items").unwrap(), 2);
    }

    #[test]
    fn test_idb_put_overwrite() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("users", Some("id"), false).unwrap();
        let key = IdbKey::String("user1".to_string());
        db.add("users", serde_json::json!({"name": "Alice"}), Some(key.clone())).unwrap();
        db.put("users", serde_json::json!({"name": "Bob"}), Some(key.clone())).unwrap();

        let record = db.get("users", &key).unwrap();
        assert_eq!(record.value["name"], "Bob");
        assert_eq!(db.count("users").unwrap(), 1);
    }

    #[test]
    fn test_idb_get_record() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("store", None, false).unwrap();
        let key = IdbKey::Number(42.0);
        db.add("store", serde_json::json!("hello"), Some(key.clone())).unwrap();
        assert!(db.get("store", &key).is_some());
    }

    #[test]
    fn test_idb_get_nonexistent() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("store", None, false).unwrap();
        assert_eq!(db.get("store", &IdbKey::String("nope".to_string())), None);
    }

    #[test]
    fn test_idb_delete_record() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("store", None, false).unwrap();
        let key = IdbKey::String("k".to_string());
        db.add("store", serde_json::json!(1), Some(key.clone())).unwrap();
        let deleted = db.delete("store", &key).unwrap();
        assert!(deleted);
        assert_eq!(db.get("store", &key), None);
    }

    #[test]
    fn test_idb_get_all() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add("store", serde_json::json!(1), Some(IdbKey::Number(1.0))).unwrap();
        db.add("store", serde_json::json!(2), Some(IdbKey::Number(2.0))).unwrap();
        let all = db.get_all("store").unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_idb_clear_store() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add("store", serde_json::json!(1), Some(IdbKey::Number(1.0))).unwrap();
        db.clear_store("store").unwrap();
        assert_eq!(db.count("store").unwrap(), 0);
    }

    #[test]
    fn test_idb_count() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("store", None, false).unwrap();
        assert_eq!(db.count("store").unwrap(), 0);
        db.add("store", serde_json::json!("a"), Some(IdbKey::String("k1".to_string()))).unwrap();
        db.add("store", serde_json::json!("b"), Some(IdbKey::String("k2".to_string()))).unwrap();
        assert_eq!(db.count("store").unwrap(), 2);
    }

    #[test]
    fn test_idb_key_ordering() {
        let num_key = IdbKey::Number(1.0);
        let str_key = IdbKey::String("a".to_string());
        let bin_key = IdbKey::Binary(vec![1, 2]);
        let arr_key = IdbKey::Array(vec![IdbKey::Number(1.0)]);

        assert!(num_key < str_key);
        assert!(str_key < bin_key);
        assert!(bin_key < arr_key);
    }

    #[test]
    fn test_idb_duplicate_key_add() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("store", None, false).unwrap();
        let key = IdbKey::String("dup".to_string());
        db.add("store", serde_json::json!(1), Some(key.clone())).unwrap();
        let result = db.add("store", serde_json::json!(2), Some(key));
        assert!(result.is_err());
    }

    #[test]
    fn test_idb_delete_nonexistent_store() {
        let mut db = IdbDatabase::new("testdb", 1);
        let result = db.delete_object_store("noexist");
        assert!(result.is_err());
    }
}

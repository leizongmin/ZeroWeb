//! IndexedDB 基础实现。

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;

use super::cursor::{CursorDirection, IdbCursor, IdbCursorWithValue, IdbTransaction};
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

/// IndexedDB 键范围，用于查询和游标迭代。
#[derive(Debug, Clone, PartialEq)]
pub struct IdbKeyRange {
    /// 下界（包含）。
    lower: Option<IdbKey>,
    /// 上界（包含）。
    upper: Option<IdbKey>,
    /// 下界是否为开区间（不包含）。
    lower_open: bool,
    /// 上界是否为开区间（不包含）。
    upper_open: bool,
}

impl IdbKeyRange {
    /// 创建只包含单个键的范围。
    ///
    /// 等价于 Web IDL 的 `IDBKeyRange.only(value)`。
    pub fn only(value: IdbKey) -> Self {
        Self {
            lower: Some(value.clone()),
            upper: Some(value),
            lower_open: false,
            upper_open: false,
        }
    }

    /// 创建从 `lower` 到正无穷的范围。
    ///
    /// 等价于 `IDBKeyRange.lowerBound(lower, open)`。
    pub fn lower_bound(lower: IdbKey, open: bool) -> Self {
        Self {
            lower: Some(lower),
            upper: None,
            lower_open: open,
            upper_open: false,
        }
    }

    /// 创建从负无穷到 `upper` 的范围。
    ///
    /// 等价于 `IDBKeyRange.upperBound(upper, open)`。
    pub fn upper_bound(upper: IdbKey, open: bool) -> Self {
        Self {
            lower: None,
            upper: Some(upper),
            lower_open: false,
            upper_open: open,
        }
    }

    /// 创建有界的范围。
    ///
    /// 等价于 `IDBKeyRange.bound(lower, upper, lowerOpen, upperOpen)`。
    pub fn bound(lower: IdbKey, upper: IdbKey, lower_open: bool, upper_open: bool) -> Self {
        Self {
            lower: Some(lower),
            upper: Some(upper),
            lower_open,
            upper_open,
        }
    }

    /// 判断给定键是否在本范围内。
    pub fn contains(&self, key: &IdbKey) -> bool {
        if let Some(ref lower) = self.lower {
            match key.cmp(lower) {
                Ordering::Less => return false,
                Ordering::Equal if self.lower_open => return false,
                _ => {}
            }
        }
        if let Some(ref upper) = self.upper {
            match key.cmp(upper) {
                Ordering::Greater => return false,
                Ordering::Equal if self.upper_open => return false,
                _ => {}
            }
        }
        true
    }

    /// 获取下界。
    pub fn lower(&self) -> Option<&IdbKey> {
        self.lower.as_ref()
    }

    /// 获取上界。
    pub fn upper(&self) -> Option<&IdbKey> {
        self.upper.as_ref()
    }

    /// 下界是否为开区间。
    pub fn lower_open(&self) -> bool {
        self.lower_open
    }

    /// 上界是否为开区间。
    pub fn upper_open(&self) -> bool {
        self.upper_open
    }
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

impl std::hash::Hash for IdbKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Discriminant first, then variant data
        std::mem::discriminant(self).hash(state);
        match self {
            // 规范化 -0.0 → +0.0：f64 的 == 认为 -0.0 == +0.0（派生 PartialEq 视为相等），
            // Hash/Eq 契约要求 a==b ⇒ hash(a)==hash(b)。旧实现用 n.to_bits() 区分两者违反契约，
            // 致 HashSet 在 SipHash RandomState 偶然同桶 + Eq 相等时去重（flaky len=1 vs len=2）。
            // 归一化后与 Eq 一致，且符合 JS Set/Map「-0 与 +0 为同一键」语义。
            IdbKey::Number(n) => {
                let normalized = if *n == 0.0 { 0.0_f64 } else { *n };
                normalized.to_bits().hash(state);
            }
            IdbKey::String(s) => s.hash(state),
            IdbKey::Binary(b) => b.hash(state),
            IdbKey::Array(a) => a.hash(state),
        }
    }
}

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

/// 索引记录条目，存储索引键到主键的映射。
#[derive(Debug, Clone)]
struct IndexEntry {
    /// 索引键值。
    index_key: IdbKey,
    /// 对应的主键。
    primary_key: IdbKey,
}

/// Object Store 索引。
pub struct IdbIndex {
    /// 索引名称。
    pub name: String,
    /// 索引键路径（用于从记录值中提取索引键）。
    pub key_path: String,
    /// 是否唯一索引。
    pub unique: bool,
    /// 是否支持多条目（multiEntry）。
    pub multi_entry: bool,
    /// 索引条目（按索引键排序，相同索引键按主键排序）。
    entries: Vec<IndexEntry>,
}

impl IdbIndex {
    /// 创建新的索引。
    fn new(name: &str, key_path: &str, unique: bool, multi_entry: bool) -> Self {
        Self {
            name: name.to_string(),
            key_path: key_path.to_string(),
            unique,
            multi_entry,
            entries: Vec::new(),
        }
    }

    /// 从 JSON 值中提取索引键，支持 multiEntry。
    fn extract_keys(&self, value: &serde_json::Value) -> Vec<IdbKey> {
        let val = match value.pointer(&format!("/{}", self.key_path.replace('.', "/"))) {
            Some(v) => v,
            None => return Vec::new(),
        };

        if self.multi_entry {
            if let serde_json::Value::Array(arr) = val {
                arr.iter().filter_map(json_value_to_idb_key).collect()
            } else {
                json_value_to_idb_key(val).into_iter().collect()
            }
        } else {
            json_value_to_idb_key(val).into_iter().collect()
        }
    }

    /// 重建索引（从全部记录中重新生成索引条目）。
    fn rebuild(&mut self, records: &[IdbRecord]) -> Result<(), StorageError> {
        self.entries.clear();
        for record in records {
            self.add_entry_from_record(record)?;
        }
        self.entries.sort_by(|a, b| match a.index_key.cmp(&b.index_key) {
            Ordering::Equal => a.primary_key.cmp(&b.primary_key),
            other => other,
        });
        Ok(())
    }

    /// 从单条记录添加索引条目。
    fn add_entry_from_record(&mut self, record: &IdbRecord) -> Result<(), StorageError> {
        let keys = self.extract_keys(&record.value);
        for index_key in keys {
            // 唯一索引检查
            if self.unique && self.entries.iter().any(|e| e.index_key == index_key) {
                return Err(StorageError::Database(format!(
                    "Unique index '{}' constraint violation for key {:?}",
                    self.name, index_key
                )));
            }
            self.entries.push(IndexEntry {
                index_key,
                primary_key: record.key.clone(),
            });
        }
        Ok(())
    }

    /// 获取所有索引条目（按索引键排序）。
    fn sorted_entries(&self) -> &[IndexEntry] {
        &self.entries
    }

    /// 根据索引键查找主键列表。
    fn get_primary_keys(&self, index_key: &IdbKey) -> Vec<&IdbKey> {
        self.entries
            .iter()
            .filter(|e| &e.index_key == index_key)
            .map(|e| &e.primary_key)
            .collect()
    }

    /// 根据主键删除索引条目。
    fn remove_by_primary_key(&mut self, primary_key: &IdbKey) {
        self.entries.retain(|e| &e.primary_key != primary_key);
    }
}

/// 将 serde_json::Value 转换为 IdbKey。
fn json_value_to_idb_key(val: &serde_json::Value) -> Option<IdbKey> {
    match val {
        serde_json::Value::Number(n) => n.as_f64().map(IdbKey::Number),
        serde_json::Value::String(s) => Some(IdbKey::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let keys: Option<Vec<IdbKey>> = arr.iter().map(json_value_to_idb_key).collect();
            keys.map(IdbKey::Array)
        }
        _ => None,
    }
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
    /// 索引集合。
    indexes: HashMap<String, IdbIndex>,
}

/// 事务中缓冲的变更操作。
#[derive(Debug, Clone)]
pub enum TxMutation {
    /// 添加记录（主键已存在则报错）。
    Add {
        /// 目标 store 名称。
        store: String,
        /// JSON 值。
        value: serde_json::Value,
        /// 主键。
        key: IdbKey,
    },
    /// 放入记录（覆盖已有）。
    Put {
        /// 目标 store 名称。
        store: String,
        /// JSON 值。
        value: serde_json::Value,
        /// 主键。
        key: IdbKey,
    },
    /// 删除记录。
    Delete {
        /// 目标 store 名称。
        store: String,
        /// 主键。
        key: IdbKey,
    },
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
                indexes: HashMap::new(),
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

    /// 重命名 Object Store。
    ///
    /// 将指定名称的 store 重命名为新名称。
    pub fn rename_object_store(&mut self, old_name: &str, new_name: &str) -> Result<(), StorageError> {
        let mut store = self
            .stores
            .remove(old_name)
            .ok_or_else(|| StorageError::StoreNotFound(old_name.to_string()))?;
        store.name = new_name.to_string();
        if self.stores.contains_key(new_name) {
            return Err(StorageError::Database(format!(
                "Object store '{}' already exists",
                new_name
            )));
        }
        self.stores.insert(new_name.to_string(), store);
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
        let store = self
            .stores
            .get_mut(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;

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

        store.records.push(IdbRecord {
            key: key.clone(),
            value: value.clone(),
        });
        // 更新索引
        let record = store.records.last().unwrap();
        for idx in store.indexes.values_mut() {
            idx.add_entry_from_record(record)?;
        }
        Ok(key)
    }

    /// 在指定 store 中放入记录（覆盖已有记录）。
    pub fn put(
        &mut self,
        store_name: &str,
        value: serde_json::Value,
        key: Option<IdbKey>,
    ) -> Result<IdbKey, StorageError> {
        let store = self
            .stores
            .get_mut(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;

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
            // 先从索引中移除旧条目，再添加新条目
            for idx in store.indexes.values_mut() {
                idx.remove_by_primary_key(&key);
            }
            record.value = value.clone();
            for idx in store.indexes.values_mut() {
                idx.add_entry_from_record(record)?;
            }
        } else {
            store.records.push(IdbRecord {
                key: key.clone(),
                value: value.clone(),
            });
            let record = store.records.last().unwrap();
            for idx in store.indexes.values_mut() {
                idx.add_entry_from_record(record)?;
            }
        }

        Ok(key)
    }

    /// 获取记录。
    pub fn get(&self, store_name: &str, key: &IdbKey) -> Option<&IdbRecord> {
        self.stores.get(store_name)?.records.iter().find(|r| &r.key == key)
    }

    /// 删除记录。
    pub fn delete(&mut self, store_name: &str, key: &IdbKey) -> Result<bool, StorageError> {
        let store = self
            .stores
            .get_mut(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        let before = store.records.len();
        // 从索引中移除
        for idx in store.indexes.values_mut() {
            idx.remove_by_primary_key(key);
        }
        store.records.retain(|r| &r.key != key);
        Ok(store.records.len() < before)
    }

    /// 获取 store 中所有记录（无范围限制）。
    pub fn get_all(&self, store_name: &str) -> Result<Vec<&IdbRecord>, StorageError> {
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        Ok(store.records.iter().collect())
    }

    /// 获取 store 中指定键范围内的记录（按键排序）。
    pub fn get_all_with_range(&self, store_name: &str, range: &IdbKeyRange) -> Result<Vec<&IdbRecord>, StorageError> {
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        let mut results: Vec<&IdbRecord> = store.records.iter().filter(|r| range.contains(&r.key)).collect();
        results.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(results)
    }

    /// 清空 store 中所有记录。
    pub fn clear_store(&mut self, store_name: &str) -> Result<(), StorageError> {
        let store = self
            .stores
            .get_mut(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        store.records.clear();
        for idx in store.indexes.values_mut() {
            idx.entries.clear();
        }
        Ok(())
    }

    /// 获取 store 中记录数量（无范围限制）。
    pub fn count(&self, store_name: &str) -> Result<usize, StorageError> {
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        Ok(store.records.len())
    }

    /// 获取 store 中指定键范围内的记录数量。
    pub fn count_with_range(&self, store_name: &str, range: &IdbKeyRange) -> Result<usize, StorageError> {
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        Ok(store.records.iter().filter(|r| range.contains(&r.key)).count())
    }

    /// 在指定 store 上创建索引。
    pub fn create_index(
        &mut self,
        store_name: &str,
        index_name: &str,
        key_path: &str,
        unique: bool,
        multi_entry: bool,
    ) -> Result<(), StorageError> {
        let store = self
            .stores
            .get_mut(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;

        if store.indexes.contains_key(index_name) {
            return Err(StorageError::Database(format!("Index '{}' already exists", index_name)));
        }

        let mut index = IdbIndex::new(index_name, key_path, unique, multi_entry);
        // 从已有记录重建索引
        index.rebuild(&store.records)?;
        store.indexes.insert(index_name.to_string(), index);
        Ok(())
    }

    /// 删除指定 store 上的索引。
    pub fn delete_index(&mut self, store_name: &str, index_name: &str) -> Result<(), StorageError> {
        let store = self
            .stores
            .get_mut(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        if store.indexes.remove(index_name).is_none() {
            return Err(StorageError::Database(format!("Index '{}' not found", index_name)));
        }
        Ok(())
    }

    /// 获取指定索引的名称列表。
    pub fn index_names(&self, store_name: &str) -> Result<Vec<&str>, StorageError> {
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        Ok(store.indexes.keys().map(|s| s.as_str()).collect())
    }

    /// 通过索引获取记录（按索引键精确匹配）。
    pub fn get_from_index(
        &self,
        store_name: &str,
        index_name: &str,
        key: &IdbKey,
    ) -> Result<Vec<&IdbRecord>, StorageError> {
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        let index = store
            .indexes
            .get(index_name)
            .ok_or_else(|| StorageError::Database(format!("Index '{}' not found", index_name)))?;

        let primary_keys = index.get_primary_keys(key);
        let mut results = Vec::with_capacity(primary_keys.len());
        for pk in primary_keys {
            if let Some(record) = store.records.iter().find(|r| &r.key == pk) {
                results.push(record);
            }
        }
        Ok(results)
    }

    /// 获取索引中所有记录（按索引键排序）。
    pub fn get_all_from_index(&self, store_name: &str, index_name: &str) -> Result<Vec<&IdbRecord>, StorageError> {
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        let index = store
            .indexes
            .get(index_name)
            .ok_or_else(|| StorageError::Database(format!("Index '{}' not found", index_name)))?;

        let mut results = Vec::new();
        for entry in index.sorted_entries() {
            if let Some(record) = store.records.iter().find(|r| r.key == entry.primary_key)
                && !results.iter().any(|r: &&IdbRecord| r.key == record.key)
            {
                results.push(record);
            }
        }
        Ok(results)
    }

    /// 获取索引中指定范围内的记录（按索引键排序）。
    pub fn get_all_from_index_with_range(
        &self,
        store_name: &str,
        index_name: &str,
        range: &IdbKeyRange,
    ) -> Result<Vec<&IdbRecord>, StorageError> {
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        let index = store
            .indexes
            .get(index_name)
            .ok_or_else(|| StorageError::Database(format!("Index '{}' not found", index_name)))?;

        let mut results = Vec::new();
        for entry in index.sorted_entries() {
            if range.contains(&entry.index_key)
                && let Some(record) = store.records.iter().find(|r| r.key == entry.primary_key)
                && !results.iter().any(|r: &&IdbRecord| r.key == record.key)
            {
                results.push(record);
            }
        }
        Ok(results)
    }

    /// 获取索引中指定范围内的记录数量。
    pub fn count_from_index(
        &self,
        store_name: &str,
        index_name: &str,
        range: Option<&IdbKeyRange>,
    ) -> Result<usize, StorageError> {
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        let index = store
            .indexes
            .get(index_name)
            .ok_or_else(|| StorageError::Database(format!("Index '{}' not found", index_name)))?;

        let count = match range {
            Some(r) => index
                .sorted_entries()
                .iter()
                .filter(|e| r.contains(&e.index_key))
                .count(),
            None => index.entries.len(),
        };
        Ok(count)
    }

    /// 在 store 上打开游标，按键排序迭代记录。
    ///
    /// 返回初始游标位置（指向第一条匹配记录），通过 `cursor.advance()` 移动。
    pub fn open_cursor(
        &self,
        store_name: &str,
        range: Option<&IdbKeyRange>,
    ) -> Result<Option<IdbCursorWithValue>, StorageError> {
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;

        let mut sorted: Vec<(IdbKey, usize)> = store
            .records
            .iter()
            .enumerate()
            .map(|(i, r)| (r.key.clone(), i))
            .collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));

        let positions: Vec<usize> = sorted
            .into_iter()
            .filter(|(k, _)| range.is_none_or(|r| r.contains(k)))
            .map(|(_, i)| i)
            .collect();

        if positions.is_empty() {
            return Ok(None);
        }

        let mut cursor = IdbCursorWithValue {
            direction: CursorDirection::Next,
            positions,
            current: 0,
            store_name: store_name.to_string(),
        };
        cursor.advance(0);
        Ok(Some(cursor))
    }

    /// 在 store 上打开键游标，只迭代键不迭代值。
    pub fn open_key_cursor(
        &self,
        store_name: &str,
        range: Option<&IdbKeyRange>,
    ) -> Result<Option<IdbCursor>, StorageError> {
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;

        let mut sorted: Vec<IdbKey> = store.records.iter().map(|r| r.key.clone()).collect();
        sorted.sort();

        let keys: Vec<IdbKey> = sorted
            .into_iter()
            .filter(|k| range.is_none_or(|r| r.contains(k)))
            .collect();

        if keys.is_empty() {
            return Ok(None);
        }

        Ok(Some(IdbCursor {
            direction: CursorDirection::Next,
            keys,
            current: 0,
            store_name: store_name.to_string(),
        }))
    }

    /// 在索引上打开游标，按索引键排序迭代。
    pub fn open_cursor_on_index(
        &self,
        store_name: &str,
        index_name: &str,
        range: Option<&IdbKeyRange>,
    ) -> Result<Option<IdbCursorWithValue>, StorageError> {
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        let index = store
            .indexes
            .get(index_name)
            .ok_or_else(|| StorageError::Database(format!("Index '{}' not found", index_name)))?;

        let entries: Vec<&IndexEntry> = index
            .sorted_entries()
            .iter()
            .filter(|e| range.is_none_or(|r| r.contains(&e.index_key)))
            .collect();

        if entries.is_empty() {
            return Ok(None);
        }

        // 收集对应记录在 store.records 中的索引
        let mut positions = Vec::new();
        let mut seen_keys = std::collections::HashSet::new();
        for entry in &entries {
            if seen_keys.insert(entry.primary_key.clone())
                && let Some(pos) = store.records.iter().position(|r| r.key == entry.primary_key)
            {
                positions.push(pos);
            }
        }

        let mut cursor = IdbCursorWithValue {
            direction: CursorDirection::Next,
            positions,
            current: 0,
            store_name: store_name.to_string(),
        };
        cursor.advance(0);
        Ok(Some(cursor))
    }

    /// 从游标获取当前记录。
    pub fn cursor_record(&self, cursor: &IdbCursorWithValue) -> Option<&IdbRecord> {
        let store = self.stores.get(cursor.store_name())?;
        let pos = cursor.positions.get(cursor.current)?;
        store.records.get(*pos)
    }

    /// 从键游标获取当前键。
    pub fn cursor_key<'a>(&self, cursor: &'a IdbCursor) -> Option<&'a IdbKey> {
        cursor.keys.get(cursor.current)
    }

    /// 创建事务。
    pub fn transaction(
        &mut self,
        store_names: &[&str],
        mode: IdbTransactionMode,
    ) -> Result<IdbTransaction, StorageError> {
        for name in store_names {
            if !self.stores.contains_key(*name) {
                return Err(StorageError::StoreNotFound(name.to_string()));
            }
        }
        Ok(IdbTransaction {
            store_names: store_names.iter().map(|s| s.to_string()).collect(),
            mode,
            db_name: self.name.clone(),
            db_version: self.version,
            aborted: false,
            committed: false,
            mutations: RefCell::new(Vec::new()),
        })
    }

    /// 在事务范围内添加记录（缓冲，提交时生效）。
    pub fn tx_add(
        &mut self,
        tx: &IdbTransaction,
        store_name: &str,
        value: serde_json::Value,
        key: Option<IdbKey>,
    ) -> Result<IdbKey, StorageError> {
        tx.check_active(store_name)?;
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        // 解析主键（自增逻辑）
        let key = match key {
            Some(k) => k,
            None if store.auto_increment => IdbKey::Number(store.next_key as f64),
            None => {
                return Err(StorageError::Database(
                    "No key provided and auto_increment is false".to_string(),
                ));
            }
        };
        // 检查 store 中是否已存在相同主键
        if store.records.iter().any(|r| r.key == key) {
            return Err(StorageError::Database(format!(
                "Key already exists in store '{}'",
                store_name
            )));
        }
        // 检查缓冲区中是否已有相同主键的 Add 操作
        let mutations = tx.mutations.borrow();
        if mutations.iter().any(|m| match m {
            TxMutation::Add { store: s, key: k, .. } | TxMutation::Put { store: s, key: k, .. } => {
                s == store_name && k == &key
            }
            _ => false,
        }) {
            return Err(StorageError::Database(format!(
                "Key already exists in store '{}'",
                store_name
            )));
        }
        drop(mutations);
        // 自增计数器立即推进（与浏览器 IndexedDB 行为一致）
        if store.auto_increment && matches!(key, IdbKey::Number(n) if n == store.next_key as f64) {
            let _ = store;
            let store = self.stores.get_mut(store_name).unwrap();
            store.next_key += 1;
        }

        tx.mutations.borrow_mut().push(TxMutation::Add {
            store: store_name.to_string(),
            value,
            key: key.clone(),
        });
        Ok(key)
    }

    /// 在事务范围内放入记录（缓冲，提交时生效）。
    pub fn tx_put(
        &mut self,
        tx: &IdbTransaction,
        store_name: &str,
        value: serde_json::Value,
        key: Option<IdbKey>,
    ) -> Result<IdbKey, StorageError> {
        tx.check_active(store_name)?;
        let store = self
            .stores
            .get_mut(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
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
        tx.mutations.borrow_mut().push(TxMutation::Put {
            store: store_name.to_string(),
            value,
            key: key.clone(),
        });
        Ok(key)
    }

    /// 在事务范围内删除记录（缓冲，提交时生效）。
    pub fn tx_delete(&mut self, tx: &IdbTransaction, store_name: &str, key: &IdbKey) -> Result<bool, StorageError> {
        tx.check_active(store_name)?;
        let exists_in_store = self
            .stores
            .get(store_name)
            .map(|s| s.records.iter().any(|r| r.key == *key))
            .unwrap_or(false);
        let exists_in_buffer = tx.mutations.borrow().iter().any(|m| match m {
            TxMutation::Add { store: s, key: k, .. } | TxMutation::Put { store: s, key: k, .. } => {
                s == store_name && k == key
            }
            _ => false,
        });
        let found = exists_in_store || exists_in_buffer;
        tx.mutations.borrow_mut().push(TxMutation::Delete {
            store: store_name.to_string(),
            key: key.clone(),
        });
        Ok(found)
    }

    /// 在事务范围内获取记录（包含缓冲区的未提交变更）。
    pub fn tx_get(
        &self,
        tx: &IdbTransaction,
        store_name: &str,
        key: &IdbKey,
    ) -> Result<Option<IdbRecord>, StorageError> {
        tx.check_active(store_name)?;
        // 从缓冲区逆序查找最新的变更
        let mutations = tx.mutations.borrow();
        for m in mutations.iter().rev() {
            match m {
                TxMutation::Delete { store, key: k } if store == store_name && k == key => {
                    return Ok(None);
                }
                TxMutation::Put {
                    store, key: k, value, ..
                }
                | TxMutation::Add {
                    store, key: k, value, ..
                } if store == store_name && k == key => {
                    return Ok(Some(IdbRecord {
                        key: key.clone(),
                        value: value.clone(),
                    }));
                }
                _ => {}
            }
        }
        // 缓冲区没有相关变更，从 store 读取
        Ok(self.get(store_name, key).cloned())
    }

    /// 提交事务，将缓冲的变更应用到 store。
    pub fn commit_tx(&mut self, tx: &mut IdbTransaction) -> Result<(), StorageError> {
        tx.commit()?;
        let mutations = tx.mutations.borrow_mut().drain(..).collect::<Vec<_>>();
        for m in mutations {
            match m {
                TxMutation::Add { store, value, key } => {
                    self.add(&store, value, Some(key))?;
                }
                TxMutation::Put { store, value, key } => {
                    self.put(&store, value, Some(key))?;
                }
                TxMutation::Delete { store, key } => {
                    self.delete(&store, &key)?;
                }
            }
        }
        Ok(())
    }
}

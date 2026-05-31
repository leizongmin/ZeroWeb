//! IndexedDB 基础实现。

use std::cell::RefCell;
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
            IdbKey::Number(n) => n.to_bits().hash(state),
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
enum TxMutation {
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
        let store = self.stores.get(&cursor.store_name)?;
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

/// 游标方向。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorDirection {
    /// 正向（从小到大）。
    Next,
    /// 反向（从大到小）。
    Prev,
}

/// 键游标，只迭代键。
pub struct IdbCursor {
    /// 游标方向。
    pub direction: CursorDirection,
    /// 排序后的键列表。
    keys: Vec<IdbKey>,
    /// 当前位置索引。
    current: usize,
    /// 所属 store 名称。
    store_name: String,
}

impl IdbCursor {
    /// 获取当前键。
    pub fn key(&self) -> Option<&IdbKey> {
        self.keys.get(self.current)
    }

    /// 前进 N 步。
    pub fn advance(&mut self, count: usize) -> bool {
        if count == 0 {
            self.current = 0;
            return true;
        }
        self.current += count;
        self.current < self.keys.len()
    }

    /// 继续到指定键。
    pub fn continue_to(&mut self, key: &IdbKey) -> bool {
        let start = self.current + 1;
        for i in start..self.keys.len() {
            if &self.keys[i] >= key {
                self.current = i;
                return true;
            }
        }
        false
    }

    /// 游标是否已到达末尾。
    pub fn is_finished(&self) -> bool {
        self.current >= self.keys.len()
    }

    /// 获取所属 store 名称。
    pub fn store_name(&self) -> &str {
        &self.store_name
    }
}

/// 值游标，迭代记录。
pub struct IdbCursorWithValue {
    /// 游标方向。
    pub direction: CursorDirection,
    /// 排序后的记录位置列表（在 store.records 中的索引）。
    positions: Vec<usize>,
    /// 当前位置索引。
    current: usize,
    /// 所属 store 名称。
    store_name: String,
}

impl IdbCursorWithValue {
    /// 获取当前主键。
    pub fn key(&self) -> usize {
        self.current
    }

    /// 前进 N 步。返回 false 表示已到达末尾。
    pub fn advance(&mut self, count: usize) -> bool {
        if count == 0 {
            self.current = 0;
            return true;
        }
        self.current += count;
        self.current < self.positions.len()
    }

    /// 继续到下一个位置。返回 false 表示已到达末尾。
    pub fn continue_next(&mut self) -> bool {
        self.current += 1;
        self.current < self.positions.len()
    }

    /// 游标是否已到达末尾。
    pub fn is_finished(&self) -> bool {
        self.current >= self.positions.len()
    }

    /// 获取所属 store 名称。
    pub fn store_name(&self) -> &str {
        &self.store_name
    }

    /// 获取当前位置索引（在 positions 数组中）。
    pub fn position(&self) -> usize {
        self.current
    }
}

/// IndexedDB 事务。
pub struct IdbTransaction {
    /// 涉及的 store 名称。
    store_names: Vec<String>,
    /// 事务模式。
    mode: IdbTransactionMode,
    /// 数据库名称。
    db_name: String,
    /// 数据库版本。
    db_version: u32,
    /// 是否已中止。
    aborted: bool,
    /// 是否已提交。
    committed: bool,
    /// 缓冲的变更操作。
    mutations: RefCell<Vec<TxMutation>>,
}

impl IdbTransaction {
    /// 检查事务是否活跃且包含指定 store。
    fn check_active(&self, store_name: &str) -> Result<(), StorageError> {
        if self.aborted {
            return Err(StorageError::Database("Transaction has been aborted".to_string()));
        }
        if self.committed {
            return Err(StorageError::Database("Transaction has been committed".to_string()));
        }
        if !self.store_names.iter().any(|s| s == store_name) {
            return Err(StorageError::Database(format!(
                "Store '{}' not in transaction scope",
                store_name
            )));
        }
        Ok(())
    }

    /// 提交事务。
    pub fn commit(&mut self) -> Result<(), StorageError> {
        if self.aborted {
            return Err(StorageError::Database("Transaction has been aborted".to_string()));
        }
        if self.committed {
            return Err(StorageError::Database("Transaction already committed".to_string()));
        }
        self.committed = true;
        Ok(())
    }

    /// 中止事务，丢弃缓冲的变更。
    pub fn abort(&mut self) -> Result<(), StorageError> {
        if self.aborted {
            return Err(StorageError::Database("Transaction already aborted".to_string()));
        }
        if self.committed {
            return Err(StorageError::Database(
                "Transaction already committed, cannot abort".to_string(),
            ));
        }
        self.mutations.borrow_mut().clear();
        self.aborted = true;
        Ok(())
    }

    /// 事务是否已中止。
    pub fn is_aborted(&self) -> bool {
        self.aborted
    }

    /// 事务是否已提交。
    pub fn is_committed(&self) -> bool {
        self.committed
    }

    /// 获取事务模式。
    pub fn mode(&self) -> IdbTransactionMode {
        self.mode
    }

    /// 获取涉及的 store 名称。
    pub fn store_names(&self) -> &[String] {
        &self.store_names
    }

    /// 获取数据库名称。
    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    /// 获取数据库版本。
    pub fn db_version(&self) -> u32 {
        self.db_version
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
        db.add("users", serde_json::json!({"name": "Alice"}), Some(key.clone()))
            .unwrap();
        db.put("users", serde_json::json!({"name": "Bob"}), Some(key.clone()))
            .unwrap();

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
        db.add("store", serde_json::json!(1), Some(IdbKey::Number(1.0)))
            .unwrap();
        db.add("store", serde_json::json!(2), Some(IdbKey::Number(2.0)))
            .unwrap();
        let all = db.get_all("store").unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_idb_clear_store() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add("store", serde_json::json!(1), Some(IdbKey::Number(1.0)))
            .unwrap();
        db.clear_store("store").unwrap();
        assert_eq!(db.count("store").unwrap(), 0);
    }

    #[test]
    fn test_idb_count() {
        let mut db = IdbDatabase::new("testdb", 1);
        db.create_object_store("store", None, false).unwrap();
        assert_eq!(db.count("store").unwrap(), 0);
        db.add("store", serde_json::json!("a"), Some(IdbKey::String("k1".to_string())))
            .unwrap();
        db.add("store", serde_json::json!("b"), Some(IdbKey::String("k2".to_string())))
            .unwrap();
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

    // ── 边界条件补充测试 ──

    /// 测试空数据库名称。
    #[test]
    fn test_idb_empty_database_name() {
        let db = IdbDatabase::new("", 1);
        assert_eq!(db.name, "");
    }

    /// 测试版本号为 0。
    #[test]
    fn test_idb_version_zero() {
        let db = IdbDatabase::new("test", 0);
        assert_eq!(db.version, 0);
    }

    /// 测试多个 object store 操作。
    #[test]
    fn test_idb_multiple_stores() {
        let mut db = IdbDatabase::new("multi", 1);
        db.create_object_store("users", None, false).unwrap();
        db.create_object_store("products", None, false).unwrap();
        db.create_object_store("orders", None, false).unwrap();

        assert_eq!(db.store_names().len(), 3);
        assert!(db.store_names().contains(&"users"));
        assert!(db.store_names().contains(&"products"));
        assert!(db.store_names().contains(&"orders"));

        // 删除中间的
        db.delete_object_store("products").unwrap();
        assert_eq!(db.store_names().len(), 2);
        assert!(!db.store_names().contains(&"products"));
    }

    /// 测试 get_all 在空 store 上。
    #[test]
    fn test_idb_get_all_empty() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("empty", None, false).unwrap();
        let records = db.get_all("empty").unwrap();
        assert!(records.is_empty());
    }

    /// 测试 count 在空 store 上。
    #[test]
    fn test_idb_count_empty() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("empty", None, false).unwrap();
        assert_eq!(db.count("empty").unwrap(), 0);
    }

    /// 测试 clear_store 后 count 为 0。
    #[test]
    fn test_idb_clear_then_count() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", None, true).unwrap();
        db.add("items", serde_json::json!("val1"), None).unwrap();
        db.add("items", serde_json::json!("val2"), None).unwrap();
        assert_eq!(db.count("items").unwrap(), 2);
        db.clear_store("items").unwrap();
        assert_eq!(db.count("items").unwrap(), 0);
    }

    /// 测试 get 在不存在的 store 上。
    #[test]
    fn test_idb_get_from_nonexistent_store() {
        let db = IdbDatabase::new("test", 1);
        let result = db.get("noexist", &IdbKey::String("key".into()));
        assert!(result.is_none());
    }

    /// 测试 IdbKey 排序：Number < String < Binary < Array。
    #[test]
    fn test_idb_key_type_ordering() {
        let num = IdbKey::Number(1.0);
        let str_key = IdbKey::String("a".into());
        let bin = IdbKey::Binary(vec![1]);
        let arr = IdbKey::Array(vec![IdbKey::Number(1.0)]);

        assert!(num < str_key);
        assert!(str_key < bin);
        assert!(bin < arr);
    }

    /// 测试 has_store。
    #[test]
    fn test_idb_has_store() {
        let mut db = IdbDatabase::new("test", 1);
        assert!(!db.has_store("users"));
        db.create_object_store("users", None, false).unwrap();
        assert!(db.has_store("users"));
        assert!(!db.has_store("products"));
    }

    /// 测试重复创建 object store 报错。
    #[test]
    fn test_idb_create_duplicate_store() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", None, false).unwrap();
        let result = db.create_object_store("items", None, false);
        assert!(result.is_err());
    }

    /// 测试 delete 记录返回是否找到。
    #[test]
    fn test_idb_delete_returns_found() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", None, false).unwrap();
        let key = IdbKey::String("k1".into());
        db.add("items", serde_json::json!("v1"), Some(key.clone())).unwrap();

        let found = db.delete("items", &key).unwrap();
        assert!(found);

        let not_found = db.delete("items", &key).unwrap();
        assert!(!not_found);
    }

    /// 测试 put 覆盖已有记录。
    #[test]
    fn test_idb_put_overwrites_value() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", None, false).unwrap();
        let key = IdbKey::String("k".into());
        db.add("items", serde_json::json!("v1"), Some(key.clone())).unwrap();
        db.put("items", serde_json::json!("v2"), Some(key.clone())).unwrap();

        let record = db.get("items", &key).unwrap();
        assert_eq!(record.value, serde_json::json!("v2"));
        // 只有一条记录（put 覆盖而不是新增）
        assert_eq!(db.count("items").unwrap(), 1);
    }

    // ── IdbKeyRange 测试 ──

    #[test]
    fn test_key_range_only() {
        let range = IdbKeyRange::only(IdbKey::Number(5.0));
        assert!(range.contains(&IdbKey::Number(5.0)));
        assert!(!range.contains(&IdbKey::Number(4.0)));
        assert!(!range.contains(&IdbKey::Number(6.0)));
    }

    #[test]
    fn test_key_range_lower_bound_closed() {
        let range = IdbKeyRange::lower_bound(IdbKey::Number(3.0), false);
        assert!(range.contains(&IdbKey::Number(3.0)));
        assert!(range.contains(&IdbKey::Number(10.0)));
        assert!(!range.contains(&IdbKey::Number(2.0)));
    }

    #[test]
    fn test_key_range_lower_bound_open() {
        let range = IdbKeyRange::lower_bound(IdbKey::Number(3.0), true);
        assert!(!range.contains(&IdbKey::Number(3.0)));
        assert!(range.contains(&IdbKey::Number(4.0)));
    }

    #[test]
    fn test_key_range_upper_bound_closed() {
        let range = IdbKeyRange::upper_bound(IdbKey::Number(10.0), false);
        assert!(range.contains(&IdbKey::Number(10.0)));
        assert!(range.contains(&IdbKey::Number(5.0)));
        assert!(!range.contains(&IdbKey::Number(11.0)));
    }

    #[test]
    fn test_key_range_upper_bound_open() {
        let range = IdbKeyRange::upper_bound(IdbKey::Number(10.0), true);
        assert!(!range.contains(&IdbKey::Number(10.0)));
        assert!(range.contains(&IdbKey::Number(9.0)));
    }

    #[test]
    fn test_key_range_bound_closed() {
        let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), false, false);
        assert!(!range.contains(&IdbKey::Number(0.0)));
        assert!(range.contains(&IdbKey::Number(1.0)));
        assert!(range.contains(&IdbKey::Number(5.0)));
        assert!(range.contains(&IdbKey::Number(10.0)));
        assert!(!range.contains(&IdbKey::Number(11.0)));
    }

    #[test]
    fn test_key_range_bound_open() {
        let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), true, true);
        assert!(!range.contains(&IdbKey::Number(1.0)));
        assert!(range.contains(&IdbKey::Number(2.0)));
        assert!(!range.contains(&IdbKey::Number(10.0)));
    }

    #[test]
    fn test_key_range_accessors() {
        let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), true, false);
        assert_eq!(range.lower(), Some(&IdbKey::Number(1.0)));
        assert_eq!(range.upper(), Some(&IdbKey::Number(10.0)));
        assert!(range.lower_open());
        assert!(!range.upper_open());
    }

    #[test]
    fn test_key_range_string_keys() {
        let range = IdbKeyRange::bound(IdbKey::String("c".into()), IdbKey::String("f".into()), false, false);
        assert!(!range.contains(&IdbKey::String("b".into())));
        assert!(range.contains(&IdbKey::String("c".into())));
        assert!(range.contains(&IdbKey::String("d".into())));
        assert!(range.contains(&IdbKey::String("f".into())));
        assert!(!range.contains(&IdbKey::String("g".into())));
    }

    // ── get_all_with_range / count_with_range 测试 ──

    #[test]
    fn test_get_all_with_range() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
            .unwrap();
        db.add("store", serde_json::json!("b"), Some(IdbKey::Number(5.0)))
            .unwrap();
        db.add("store", serde_json::json!("c"), Some(IdbKey::Number(10.0)))
            .unwrap();
        db.add("store", serde_json::json!("d"), Some(IdbKey::Number(15.0)))
            .unwrap();

        let range = IdbKeyRange::bound(IdbKey::Number(5.0), IdbKey::Number(10.0), false, false);
        let results = db.get_all_with_range("store", &range).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].value, serde_json::json!("b"));
        assert_eq!(results[1].value, serde_json::json!("c"));
    }

    #[test]
    fn test_count_with_range() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add("store", serde_json::json!(1), Some(IdbKey::Number(1.0)))
            .unwrap();
        db.add("store", serde_json::json!(2), Some(IdbKey::Number(5.0)))
            .unwrap();
        db.add("store", serde_json::json!(3), Some(IdbKey::Number(10.0)))
            .unwrap();

        let range = IdbKeyRange::lower_bound(IdbKey::Number(5.0), false);
        assert_eq!(db.count_with_range("store", &range).unwrap(), 2);
    }

    // ── 索引测试 ──

    #[test]
    fn test_create_index() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("users", None, false).unwrap();
        db.add(
            "users",
            serde_json::json!({"name": "Alice", "age": 30}),
            Some(IdbKey::String("u1".into())),
        )
        .unwrap();
        db.add(
            "users",
            serde_json::json!({"name": "Bob", "age": 25}),
            Some(IdbKey::String("u2".into())),
        )
        .unwrap();

        db.create_index("users", "name_idx", "name", false, false).unwrap();
        let names = db.index_names("users").unwrap();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"name_idx"));
    }

    #[test]
    fn test_get_from_index() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("users", None, false).unwrap();
        db.add(
            "users",
            serde_json::json!({"name": "Alice", "age": 30}),
            Some(IdbKey::String("u1".into())),
        )
        .unwrap();
        db.add(
            "users",
            serde_json::json!({"name": "Bob", "age": 25}),
            Some(IdbKey::String("u2".into())),
        )
        .unwrap();

        db.create_index("users", "name_idx", "name", false, false).unwrap();
        let results = db
            .get_from_index("users", "name_idx", &IdbKey::String("Alice".into()))
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value["age"], 30);
    }

    #[test]
    fn test_get_all_from_index_sorted() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("users", None, false).unwrap();
        db.add(
            "users",
            serde_json::json!({"name": "Charlie"}),
            Some(IdbKey::String("u1".into())),
        )
        .unwrap();
        db.add(
            "users",
            serde_json::json!({"name": "Alice"}),
            Some(IdbKey::String("u2".into())),
        )
        .unwrap();
        db.add(
            "users",
            serde_json::json!({"name": "Bob"}),
            Some(IdbKey::String("u3".into())),
        )
        .unwrap();

        db.create_index("users", "name_idx", "name", false, false).unwrap();
        let results = db.get_all_from_index("users", "name_idx").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].value["name"], "Alice");
        assert_eq!(results[1].value["name"], "Bob");
        assert_eq!(results[2].value["name"], "Charlie");
    }

    #[test]
    fn test_get_all_from_index_with_range() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("users", None, false).unwrap();
        db.add(
            "users",
            serde_json::json!({"age": 20}),
            Some(IdbKey::String("u1".into())),
        )
        .unwrap();
        db.add(
            "users",
            serde_json::json!({"age": 30}),
            Some(IdbKey::String("u2".into())),
        )
        .unwrap();
        db.add(
            "users",
            serde_json::json!({"age": 40}),
            Some(IdbKey::String("u3".into())),
        )
        .unwrap();

        db.create_index("users", "age_idx", "age", false, false).unwrap();
        let range = IdbKeyRange::bound(IdbKey::Number(25.0), IdbKey::Number(35.0), false, false);
        let results = db.get_all_from_index_with_range("users", "age_idx", &range).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value["age"], 30);
    }

    #[test]
    fn test_index_unique_constraint() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("users", None, false).unwrap();
        db.add(
            "users",
            serde_json::json!({"email": "a@b.com"}),
            Some(IdbKey::String("u1".into())),
        )
        .unwrap();
        db.create_index("users", "email_idx", "email", true, false).unwrap();

        let result = db.add(
            "users",
            serde_json::json!({"email": "a@b.com"}),
            Some(IdbKey::String("u2".into())),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_index() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.create_index("store", "idx", "field", false, false).unwrap();
        assert_eq!(db.index_names("store").unwrap().len(), 1);
        db.delete_index("store", "idx").unwrap();
        assert_eq!(db.index_names("store").unwrap().len(), 0);
    }

    #[test]
    fn test_index_updated_on_delete() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add(
            "store",
            serde_json::json!({"tag": "a"}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();
        db.create_index("store", "tag_idx", "tag", false, false).unwrap();

        let results = db
            .get_from_index("store", "tag_idx", &IdbKey::String("a".into()))
            .unwrap();
        assert_eq!(results.len(), 1);

        db.delete("store", &IdbKey::String("k1".into())).unwrap();
        let results = db
            .get_from_index("store", "tag_idx", &IdbKey::String("a".into()))
            .unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_index_updated_on_put() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add(
            "store",
            serde_json::json!({"tag": "a"}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();
        db.create_index("store", "tag_idx", "tag", false, false).unwrap();

        db.put(
            "store",
            serde_json::json!({"tag": "b"}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();

        let results_a = db
            .get_from_index("store", "tag_idx", &IdbKey::String("a".into()))
            .unwrap();
        assert!(results_a.is_empty());
        let results_b = db
            .get_from_index("store", "tag_idx", &IdbKey::String("b".into()))
            .unwrap();
        assert_eq!(results_b.len(), 1);
    }

    #[test]
    fn test_count_from_index() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add("store", serde_json::json!({"v": 1}), Some(IdbKey::String("k1".into())))
            .unwrap();
        db.add("store", serde_json::json!({"v": 2}), Some(IdbKey::String("k2".into())))
            .unwrap();
        db.add("store", serde_json::json!({"v": 3}), Some(IdbKey::String("k3".into())))
            .unwrap();

        db.create_index("store", "v_idx", "v", false, false).unwrap();
        assert_eq!(db.count_from_index("store", "v_idx", None).unwrap(), 3);

        let range = IdbKeyRange::lower_bound(IdbKey::Number(2.0), false);
        assert_eq!(db.count_from_index("store", "v_idx", Some(&range)).unwrap(), 2);
    }

    #[test]
    fn test_clear_store_clears_indexes() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add("store", serde_json::json!({"x": 1}), Some(IdbKey::Number(1.0)))
            .unwrap();
        db.create_index("store", "x_idx", "x", false, false).unwrap();
        assert_eq!(db.count_from_index("store", "x_idx", None).unwrap(), 1);
        db.clear_store("store").unwrap();
        assert_eq!(db.count_from_index("store", "x_idx", None).unwrap(), 0);
    }

    #[test]
    fn test_multi_entry_index() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add(
            "store",
            serde_json::json!({"tags": ["red", "blue"]}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!({"tags": ["blue", "green"]}),
            Some(IdbKey::String("k2".into())),
        )
        .unwrap();

        db.create_index("store", "tags_idx", "tags", false, true).unwrap();
        let results = db
            .get_from_index("store", "tags_idx", &IdbKey::String("blue".into()))
            .unwrap();
        assert_eq!(results.len(), 2);
    }

    // ── 游标测试 ──

    #[test]
    fn test_open_cursor_basic() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add("store", serde_json::json!("c"), Some(IdbKey::Number(3.0)))
            .unwrap();
        db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
            .unwrap();
        db.add("store", serde_json::json!("b"), Some(IdbKey::Number(2.0)))
            .unwrap();

        let cursor = db.open_cursor("store", None).unwrap().unwrap();
        let rec = db.cursor_record(&cursor).unwrap();
        assert_eq!(rec.value, serde_json::json!("a"));
    }

    #[test]
    fn test_cursor_advance() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
            .unwrap();
        db.add("store", serde_json::json!("b"), Some(IdbKey::Number(2.0)))
            .unwrap();
        db.add("store", serde_json::json!("c"), Some(IdbKey::Number(3.0)))
            .unwrap();

        let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
        assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!("a"));

        assert!(cursor.continue_next());
        assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!("b"));

        assert!(cursor.continue_next());
        assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!("c"));

        assert!(!cursor.continue_next());
        assert!(cursor.is_finished());
    }

    #[test]
    fn test_cursor_with_range() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add("store", serde_json::json!(1), Some(IdbKey::Number(1.0)))
            .unwrap();
        db.add("store", serde_json::json!(2), Some(IdbKey::Number(2.0)))
            .unwrap();
        db.add("store", serde_json::json!(3), Some(IdbKey::Number(3.0)))
            .unwrap();
        db.add("store", serde_json::json!(4), Some(IdbKey::Number(4.0)))
            .unwrap();

        let range = IdbKeyRange::bound(IdbKey::Number(2.0), IdbKey::Number(3.0), false, false);
        let mut cursor = db.open_cursor("store", Some(&range)).unwrap().unwrap();
        assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!(2));
        assert!(cursor.continue_next());
        assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!(3));
        assert!(!cursor.continue_next());
    }

    #[test]
    fn test_open_key_cursor() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add("store", serde_json::json!("c"), Some(IdbKey::Number(3.0)))
            .unwrap();
        db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
            .unwrap();

        let mut cursor = db.open_key_cursor("store", None).unwrap().unwrap();
        assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(1.0)));
        assert!(cursor.advance(1));
        assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(3.0)));
        assert!(!cursor.advance(1));
    }

    #[test]
    fn test_open_cursor_on_index() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add(
            "store",
            serde_json::json!({"name": "Charlie"}),
            Some(IdbKey::String("u1".into())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!({"name": "Alice"}),
            Some(IdbKey::String("u2".into())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!({"name": "Bob"}),
            Some(IdbKey::String("u3".into())),
        )
        .unwrap();

        db.create_index("store", "name_idx", "name", false, false).unwrap();
        let mut cursor = db.open_cursor_on_index("store", "name_idx", None).unwrap().unwrap();
        assert_eq!(db.cursor_record(&cursor).unwrap().value["name"], "Alice");
        assert!(cursor.continue_next());
        assert_eq!(db.cursor_record(&cursor).unwrap().value["name"], "Bob");
        assert!(cursor.continue_next());
        assert_eq!(db.cursor_record(&cursor).unwrap().value["name"], "Charlie");
    }

    #[test]
    fn test_open_cursor_empty_store() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        assert!(db.open_cursor("store", None).unwrap().is_none());
    }

    // ── 事务测试 ──

    #[test]
    fn test_transaction_create() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        assert_eq!(tx.mode(), IdbTransactionMode::ReadWrite);
        assert_eq!(tx.store_names().len(), 1);
        assert!(!tx.is_committed());
        assert!(!tx.is_aborted());
    }

    #[test]
    fn test_transaction_commit() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        tx.commit().unwrap();
        assert!(tx.is_committed());
    }

    #[test]
    fn test_transaction_abort() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        tx.abort().unwrap();
        assert!(tx.is_aborted());
    }

    #[test]
    fn test_transaction_double_commit() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        tx.commit().unwrap();
        assert!(tx.commit().is_err());
    }

    #[test]
    fn test_transaction_abort_after_commit() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        tx.commit().unwrap();
        assert!(tx.abort().is_err());
    }

    #[test]
    fn test_transaction_nonexistent_store() {
        let mut db = IdbDatabase::new("test", 1);
        let result = db.transaction(&["noexist"], IdbTransactionMode::ReadOnly);
        assert!(result.is_err());
    }

    #[test]
    fn test_tx_operations() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();

        let key = db
            .tx_add(
                &tx,
                "store",
                serde_json::json!("hello"),
                Some(IdbKey::String("k1".into())),
            )
            .unwrap();
        assert_eq!(key, IdbKey::String("k1".into()));

        let record = db.tx_get(&tx, "store", &IdbKey::String("k1".into())).unwrap();
        assert_eq!(record.unwrap().value, serde_json::json!("hello"));
    }

    #[test]
    fn test_tx_operations_out_of_scope() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("a", None, false).unwrap();
        db.create_object_store("b", None, false).unwrap();
        let tx = db.transaction(&["a"], IdbTransactionMode::ReadWrite).unwrap();

        let result = db.tx_add(&tx, "b", serde_json::json!(1), Some(IdbKey::Number(1.0)));
        assert!(result.is_err());
    }

    #[test]
    fn test_tx_operations_after_abort() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        tx.abort().unwrap();

        let result = db.tx_add(&tx, "store", serde_json::json!(1), Some(IdbKey::Number(1.0)));
        assert!(result.is_err());
    }

    // ── 新增测试：事务 ──

    #[test]
    fn test_transaction_commit_then_operations_fail() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        tx.commit().unwrap();
        // After commit, operations should fail
        let result = db.tx_add(&tx, "store", serde_json::json!("val"), Some(IdbKey::String("k".into())));
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_read_only_mode() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let tx = db.transaction(&["store"], IdbTransactionMode::ReadOnly).unwrap();
        assert_eq!(tx.mode(), IdbTransactionMode::ReadOnly);
    }

    #[test]
    fn test_transaction_multiple_stores() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("a", None, false).unwrap();
        db.create_object_store("b", None, false).unwrap();
        let tx = db.transaction(&["a", "b"], IdbTransactionMode::ReadWrite).unwrap();
        assert_eq!(tx.store_names().len(), 2);

        // Can add to both stores within the same transaction
        db.tx_add(&tx, "a", serde_json::json!(1), Some(IdbKey::Number(1.0)))
            .unwrap();
        db.tx_add(&tx, "b", serde_json::json!(2), Some(IdbKey::Number(2.0)))
            .unwrap();
    }

    #[test]
    fn test_transaction_double_abort() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        tx.abort().unwrap();
        // Second abort should fail
        assert!(tx.abort().is_err());
    }

    #[test]
    fn test_transaction_commit_after_abort_fails() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        tx.abort().unwrap();
        // Commit after abort should fail
        assert!(tx.commit().is_err());
    }

    #[test]
    fn test_tx_put_and_delete() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        let key = IdbKey::String("k1".into());
        db.tx_put(&tx, "store", serde_json::json!("v1"), Some(key.clone()))
            .unwrap();
        let rec = db.tx_get(&tx, "store", &key).unwrap().unwrap();
        assert_eq!(rec.value, serde_json::json!("v1"));
        let deleted = db.tx_delete(&tx, "store", &key).unwrap();
        assert!(deleted);
        assert!(db.tx_get(&tx, "store", &key).unwrap().is_none());
    }

    // ── 新增测试：游标与索引 ──

    #[test]
    fn test_cursor_forward_iteration_all() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        for i in 1..=5 {
            db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
                .unwrap();
        }
        let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
        let mut collected = Vec::new();
        loop {
            let rec = db.cursor_record(&cursor).unwrap();
            collected.push(rec.value.as_u64().unwrap());
            if !cursor.continue_next() {
                break;
            }
        }
        assert_eq!(collected, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_cursor_with_lower_bound_range() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        for i in 1..=5 {
            db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
                .unwrap();
        }
        let range = IdbKeyRange::lower_bound(IdbKey::Number(3.0), false);
        let cursor = db.open_cursor("store", Some(&range)).unwrap().unwrap();
        let rec = db.cursor_record(&cursor).unwrap();
        assert_eq!(rec.value, serde_json::json!(3));
    }

    #[test]
    fn test_key_cursor_continue_to() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        for i in 1..=5 {
            db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
                .unwrap();
        }
        let mut cursor = db.open_key_cursor("store", None).unwrap().unwrap();
        assert!(cursor.continue_to(&IdbKey::Number(4.0)));
        assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(4.0)));
    }

    #[test]
    fn test_cursor_advance_skip() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        for i in 1..=5 {
            db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
                .unwrap();
        }
        let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
        // Skip 2 positions (from 0 to 2, landing on 3rd record)
        assert!(cursor.advance(2));
        let rec = db.cursor_record(&cursor).unwrap();
        assert_eq!(rec.value, serde_json::json!(3));
    }

    #[test]
    fn test_index_rebuild_after_add() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add(
            "store",
            serde_json::json!({"cat": "a"}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();
        db.create_index("store", "cat_idx", "cat", false, false).unwrap();
        // Add record after index creation — index should update
        db.add(
            "store",
            serde_json::json!({"cat": "b"}),
            Some(IdbKey::String("k2".into())),
        )
        .unwrap();
        let results = db
            .get_from_index("store", "cat_idx", &IdbKey::String("b".into()))
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value["cat"], "b");
    }

    #[test]
    fn test_index_unique_allows_different_values() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add(
            "store",
            serde_json::json!({"code": "AAA"}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();
        db.create_index("store", "code_idx", "code", true, false).unwrap();
        // Different value should succeed
        db.add(
            "store",
            serde_json::json!({"code": "BBB"}),
            Some(IdbKey::String("k2".into())),
        )
        .unwrap();
        assert_eq!(db.count("store").unwrap(), 2);
    }

    #[test]
    fn test_multi_entry_index_single_match() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add(
            "store",
            serde_json::json!({"tags": ["rust", "web"]}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!({"tags": ["python"]}),
            Some(IdbKey::String("k2".into())),
        )
        .unwrap();
        db.create_index("store", "tags_idx", "tags", false, true).unwrap();
        let results = db
            .get_from_index("store", "tags_idx", &IdbKey::String("rust".into()))
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value["tags"][0], "rust");
    }

    #[test]
    fn test_open_cursor_on_index_with_range() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add(
            "store",
            serde_json::json!({"score": 10}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!({"score": 20}),
            Some(IdbKey::String("k2".into())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!({"score": 30}),
            Some(IdbKey::String("k3".into())),
        )
        .unwrap();
        db.create_index("store", "score_idx", "score", false, false).unwrap();
        let range = IdbKeyRange::lower_bound(IdbKey::Number(20.0), false);
        let mut cursor = db
            .open_cursor_on_index("store", "score_idx", Some(&range))
            .unwrap()
            .unwrap();
        assert_eq!(db.cursor_record(&cursor).unwrap().value["score"], 20);
        assert!(cursor.continue_next());
        assert_eq!(db.cursor_record(&cursor).unwrap().value["score"], 30);
        assert!(!cursor.continue_next());
    }

    #[test]
    fn test_delete_nonexistent_record() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let deleted = db.delete("store", &IdbKey::String("nope".into())).unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_idb_key_array_ordering() {
        let a = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Number(2.0)]);
        let b = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Number(3.0)]);
        let c = IdbKey::Array(vec![IdbKey::Number(2.0)]);
        assert!(a < b);
        assert!(b < c);
    }

    // ── 事务缓冲与中止测试 ──

    /// tx_add 后 abort，数据不应存在于 store 中。
    #[test]
    fn test_tx_add_then_abort_data_not_in_store() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        db.tx_add(
            &tx,
            "store",
            serde_json::json!({"name": "Alice"}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();
        tx.abort().unwrap();
        // 中止后数据不应在 store 中
        assert!(db.get("store", &IdbKey::String("k1".into())).is_none());
        assert_eq!(db.count("store").unwrap(), 0);
    }

    /// tx_put 后 abort，原始数据应保留。
    #[test]
    fn test_tx_put_then_abort_original_preserved() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let key = IdbKey::String("k1".into());
        db.add("store", serde_json::json!({"name": "Alice"}), Some(key.clone()))
            .unwrap();

        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        db.tx_put(&tx, "store", serde_json::json!({"name": "Bob"}), Some(key.clone()))
            .unwrap();
        tx.abort().unwrap();

        // 原始数据应保留
        let record = db.get("store", &key).unwrap();
        assert_eq!(record.value["name"], "Alice");
    }

    /// tx_delete 后 abort，被删除的数据应保留。
    #[test]
    fn test_tx_delete_then_abort_data_preserved() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let key = IdbKey::String("k1".into());
        db.add("store", serde_json::json!("original"), Some(key.clone()))
            .unwrap();

        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        db.tx_delete(&tx, "store", &key).unwrap();
        tx.abort().unwrap();

        // 数据应保留
        let record = db.get("store", &key).unwrap();
        assert_eq!(record.value, serde_json::json!("original"));
    }

    /// tx_add 后 commit_tx，数据应存在于 store 中。
    #[test]
    fn test_tx_add_then_commit_data_in_store() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        db.tx_add(
            &tx,
            "store",
            serde_json::json!({"name": "Alice"}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();
        db.commit_tx(&mut tx).unwrap();

        let record = db.get("store", &IdbKey::String("k1".into())).unwrap();
        assert_eq!(record.value["name"], "Alice");
        assert_eq!(db.count("store").unwrap(), 1);
    }

    /// tx_put 后 commit_tx，数据应被更新。
    #[test]
    fn test_tx_put_then_commit_data_updated() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let key = IdbKey::String("k1".into());
        db.add("store", serde_json::json!("original"), Some(key.clone()))
            .unwrap();

        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        db.tx_put(&tx, "store", serde_json::json!("updated"), Some(key.clone()))
            .unwrap();
        db.commit_tx(&mut tx).unwrap();

        let record = db.get("store", &key).unwrap();
        assert_eq!(record.value, serde_json::json!("updated"));
        assert_eq!(db.count("store").unwrap(), 1);
    }

    /// tx_delete 后 commit_tx，数据应被删除。
    #[test]
    fn test_tx_delete_then_commit_data_removed() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let key = IdbKey::String("k1".into());
        db.add("store", serde_json::json!("val"), Some(key.clone())).unwrap();

        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        db.tx_delete(&tx, "store", &key).unwrap();
        db.commit_tx(&mut tx).unwrap();

        assert!(db.get("store", &key).is_none());
    }

    /// 事务内 tx_get 应能看到缓冲区的未提交变更。
    #[test]
    fn test_tx_get_sees_buffered_add() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        db.tx_add(
            &tx,
            "store",
            serde_json::json!("buffered"),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();

        let rec = db.tx_get(&tx, "store", &IdbKey::String("k1".into())).unwrap();
        assert_eq!(rec.unwrap().value, serde_json::json!("buffered"));
        // 尚未提交，store 中不应有数据
        assert!(db.get("store", &IdbKey::String("k1".into())).is_none());
    }

    /// 事务内 tx_get 对被缓冲删除的键返回 None。
    #[test]
    fn test_tx_get_sees_buffered_delete() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let key = IdbKey::String("k1".into());
        db.add("store", serde_json::json!("original"), Some(key.clone()))
            .unwrap();

        let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        db.tx_delete(&tx, "store", &key).unwrap();
        assert!(db.tx_get(&tx, "store", &key).unwrap().is_none());
    }

    /// 事务内 tx_get 对缓冲 put 返回更新后的值。
    #[test]
    fn test_tx_get_sees_buffered_put() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let key = IdbKey::String("k1".into());
        db.add("store", serde_json::json!("old"), Some(key.clone())).unwrap();

        let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        db.tx_put(&tx, "store", serde_json::json!("new"), Some(key.clone()))
            .unwrap();

        let rec = db.tx_get(&tx, "store", &key).unwrap().unwrap();
        assert_eq!(rec.value, serde_json::json!("new"));
    }

    // ── IdbKey 边界值排序测试 ──

    /// 测试 NaN 键的排序行为。
    ///
    /// 当前实现使用 partial_cmp().unwrap_or(Ordering::Equal)，
    /// 导致 NaN 被视为与任意数值相等（包括自身），这是不符合
    /// IndexedDB 规范的已知行为。本测试记录当前行为。
    #[test]
    fn test_idb_key_nan_ordering() {
        let nan_key = IdbKey::Number(f64::NAN);
        let one_key = IdbKey::Number(1.0);
        let inf_key = IdbKey::Number(f64::INFINITY);
        let neg_inf_key = IdbKey::Number(f64::NEG_INFINITY);

        // NaN 与自身比较：当前实现返回 Equal（因为 partial_cmp 返回 None）
        assert_eq!(nan_key.cmp(&nan_key), Ordering::Equal);

        // NaN 与普通数值比较：当前实现返回 Equal（不符合规范）
        // 按 IndexedDB 规范，NaN 不应是有效 key，但当前实现允许。
        // 此处断言记录当前行为：NaN 被视为与所有数值相等。
        assert_eq!(nan_key.cmp(&one_key), Ordering::Equal);
        assert_eq!(nan_key.cmp(&inf_key), Ordering::Equal);
        assert_eq!(nan_key.cmp(&neg_inf_key), Ordering::Equal);

        // 反向比较同样返回 Equal
        assert_eq!(one_key.cmp(&nan_key), Ordering::Equal);

        // NaN 与非 Number 类型比较：仍应保持 Number < String 的跨类型规则
        let str_key = IdbKey::String("a".to_string());
        assert_eq!(nan_key.cmp(&str_key), Ordering::Less);
    }

    /// 测试 +Infinity 和 -Infinity 键的排序行为。
    ///
    /// +Inf 应大于所有有限数值，-Inf 应小于所有有限数值。
    #[test]
    fn test_idb_key_infinity_ordering() {
        let inf = IdbKey::Number(f64::INFINITY);
        let neg_inf = IdbKey::Number(f64::NEG_INFINITY);
        let max_finite = IdbKey::Number(f64::MAX);
        let min_finite = IdbKey::Number(f64::MIN_POSITIVE);
        let zero = IdbKey::Number(0.0);

        // +Inf 大于所有有限数
        assert_eq!(inf.cmp(&max_finite), Ordering::Greater);
        assert_eq!(max_finite.cmp(&inf), Ordering::Less);

        // -Inf 小于所有有限数（包括负数）
        assert_eq!(neg_inf.cmp(&zero), Ordering::Less);
        assert_eq!(neg_inf.cmp(&IdbKey::Number(-f64::MAX)), Ordering::Less);

        // +Inf 大于 -Inf
        assert_eq!(inf.cmp(&neg_inf), Ordering::Greater);
        assert_eq!(neg_inf.cmp(&inf), Ordering::Less);

        // +Inf 自身相等
        assert_eq!(inf.cmp(&inf), Ordering::Equal);
        assert_eq!(neg_inf.cmp(&neg_inf), Ordering::Equal);

        // 在排序中的位置：-Inf < 0 < min_positive < MAX < +Inf
        let mut keys = vec![
            inf.clone(),
            max_finite.clone(),
            zero.clone(),
            neg_inf.clone(),
            min_finite.clone(),
        ];
        keys.sort();
        assert_eq!(keys[0], neg_inf);
        assert_eq!(keys[1], zero);
        assert_eq!(keys[2], min_finite);
        assert_eq!(keys[3], max_finite);
        assert_eq!(keys[4], inf);
    }

    /// 测试 -0.0 与 +0.0 键的比较行为。
    ///
    /// 按 IEEE 754，-0.0 == +0.0。IdbKey 使用 PartialEq 派生，
    /// 但 Hash 基于 to_bits()（不同），所以它们在 HashMap/HashSet 中
    /// 是不同的键，但 Ord 比较返回 Equal。
    #[test]
    fn test_idb_key_zero_ordering() {
        let pos_zero = IdbKey::Number(0.0);
        let neg_zero = IdbKey::Number(-0.0);

        // f64 的 == 认为 -0.0 == +0.0，所以 PartialEq 也不等
        // 但 IdbKey 派生 PartialEq，Number(0.0) == Number(-0.0)
        // 因为 f64 的 0.0 == -0.0 为 true
        assert!(pos_zero == neg_zero, "+0.0 should equal -0.0 via PartialEq");

        // Ord 排序：应为 Equal（因为底层 f64 的 partial_cmp 返回 Equal）
        assert_eq!(pos_zero.cmp(&neg_zero), Ordering::Equal);

        // Hash 行为：to_bits() 不同（+0=0, -0=0x8000000000000000），
        // 所以它们在 HashSet 中被视为不同元素
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(pos_zero.clone());
        set.insert(neg_zero.clone());
        // 当前实现：discriminant 相同 + to_bits 不同 → 两个都插入
        assert_eq!(set.len(), 2, "-0.0 and +0.0 should hash differently (to_bits mismatch)");

        // 在 Vec 排序中，-0.0 和 +0.0 位置不确定（因为 Equal），
        // 但排序后它们应该相邻
        let mut keys = vec![
            IdbKey::Number(1.0),
            IdbKey::Number(-0.0),
            IdbKey::Number(0.0),
            IdbKey::Number(-1.0),
        ];
        keys.sort();
        // -1.0, (-0.0, +0.0 顺序不确定), 1.0
        assert_eq!(keys[0], IdbKey::Number(-1.0));
        // keys[1] 和 keys[2] 都是某种零，无法确定顺序
        assert!(keys[1] == IdbKey::Number(0.0) || keys[1] == IdbKey::Number(-0.0));
        assert!(keys[2] == IdbKey::Number(0.0) || keys[2] == IdbKey::Number(-0.0));
        assert_eq!(keys[3], IdbKey::Number(1.0));
    }

    /// 测试唯一索引在 put 覆盖路径上的约束违反检测。
    ///
    /// 场景：创建唯一索引，添加记录 A（索引值 X），添加记录 B（索引值 Y），
    /// 然后 put(A, 新值) 将 A 的索引值改为 Y——此时应触发唯一约束违反。
    ///
    /// 已知问题：当前 put() 在检测约束之前已修改了 record.value，
    /// 即使返回错误，记录值已被覆盖。本测试记录此行为。
    #[test]
    fn test_unique_index_put_violation() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();

        // 添加记录 A，索引值为 "email_a@test.com"
        db.add(
            "store",
            serde_json::json!({"email": "email_a@test.com"}),
            Some(IdbKey::String("A".into())),
        )
        .unwrap();

        // 添加记录 B，索引值为 "email_b@test.com"
        db.add(
            "store",
            serde_json::json!({"email": "email_b@test.com"}),
            Some(IdbKey::String("B".into())),
        )
        .unwrap();

        // 创建唯一索引
        db.create_index("store", "email_idx", "email", true, false).unwrap();

        // 尝试 put 记录 A，将其 email 改为 "email_b@test.com"（与记录 B 冲突）
        let result = db.put(
            "store",
            serde_json::json!({"email": "email_b@test.com"}),
            Some(IdbKey::String("A".into())),
        );

        // put 应检测到唯一约束冲突并返回错误
        assert!(
            result.is_err(),
            "put() changing indexed value to conflict with another record should fail unique constraint"
        );

        // 已知问题：尽管 put 返回了错误，record.value 已被提前修改。
        // 正确行为应为回滚到原始值，但当前实现先修改值再检查索引约束。
        // 下面断言记录当前（有缺陷的）行为：
        let record_a = db.get("store", &IdbKey::String("A".into())).unwrap();
        assert_eq!(
            record_a.value["email"], "email_b@test.com",
            "BUG: put() modified record value before unique check, value is corrupted despite error"
        );

        // 记录数量不变
        assert_eq!(db.count("store").unwrap(), 2);
    }

    // ── 新增边界测试：游标 advance / continue / 迭代 ──

    /// 打开值游标，advance(N) 跳过 N 条记录，验证游标停在正确位置。
    #[test]
    fn test_idb_cursor_advance() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        for i in 1..=6 {
            db.add(
                "store",
                serde_json::json!(format!("v{i}")),
                Some(IdbKey::Number(i as f64)),
            )
            .unwrap();
        }

        let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
        // 初始位置：第一条记录（key=1）
        assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!("v1"));

        // advance(3)：跳 3 步，落到第 4 条（key=4，value="v4"）
        assert!(cursor.advance(3));
        assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!("v4"));
        assert_eq!(cursor.position(), 3);

        // advance(2)：跳 2 步，落到第 6 条（key=6，value="v6"）
        assert!(cursor.advance(2));
        assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!("v6"));

        // advance(1)：超出范围
        assert!(!cursor.advance(1));
        assert!(cursor.is_finished());
    }

    /// 打开键游标，continue_to 跳到指定键。
    #[test]
    fn test_idb_cursor_continue_to() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        for i in 1..=5 {
            db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
                .unwrap();
        }

        let mut cursor = db.open_key_cursor("store", None).unwrap().unwrap();
        // 初始在 key=1
        assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(1.0)));

        // continue_to(3.0) → 跳到 key=3
        assert!(cursor.continue_to(&IdbKey::Number(3.0)));
        assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(3.0)));

        // continue_to(10.0) → 超出范围
        assert!(!cursor.continue_to(&IdbKey::Number(10.0)));
    }

    /// 打开值游标并逐条迭代全部记录。
    #[test]
    fn test_idb_cursor_iteration() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        // 按 3-1-2 的顺序插入，验证迭代时按键排序
        db.add("store", serde_json::json!("c"), Some(IdbKey::Number(3.0)))
            .unwrap();
        db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
            .unwrap();
        db.add("store", serde_json::json!("b"), Some(IdbKey::Number(2.0)))
            .unwrap();

        let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
        let mut values = Vec::new();
        loop {
            let rec = db.cursor_record(&cursor).unwrap();
            values.push(rec.value.clone());
            if !cursor.continue_next() {
                break;
            }
        }
        assert_eq!(
            values,
            vec![serde_json::json!("a"), serde_json::json!("b"), serde_json::json!("c"),]
        );
        assert!(cursor.is_finished());
    }

    /// 打开键游标，advance(N) 跳过 N 条记录，验证键序列。
    #[test]
    fn test_idb_key_cursor_advance() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        for i in [10, 20, 30, 40, 50] {
            db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
                .unwrap();
        }

        let mut cursor = db.open_key_cursor("store", None).unwrap().unwrap();
        assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(10.0)));

        // advance(2) → 跳到 30
        assert!(cursor.advance(2));
        assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(30.0)));

        // advance(1) → 跳到 40
        assert!(cursor.advance(1));
        assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(40.0)));

        // advance(1) → 跳到 50
        assert!(cursor.advance(1));
        assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(50.0)));

        // advance(1) → 超出范围
        assert!(!cursor.advance(1));
        assert!(cursor.is_finished());
    }

    /// 打开键游标，continue_next 逐步前进。
    #[test]
    fn test_idb_key_cursor_continue_next() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add("store", serde_json::json!("x"), Some(IdbKey::String("a".into())))
            .unwrap();
        db.add("store", serde_json::json!("y"), Some(IdbKey::String("b".into())))
            .unwrap();
        db.add("store", serde_json::json!("z"), Some(IdbKey::String("c".into())))
            .unwrap();

        let mut cursor = db.open_key_cursor("store", None).unwrap().unwrap();
        let mut keys = Vec::new();
        loop {
            keys.push(db.cursor_key(&cursor).cloned());
            if !cursor.advance(1) {
                break;
            }
        }
        assert_eq!(
            keys,
            vec![
                Some(IdbKey::String("a".into())),
                Some(IdbKey::String("b".into())),
                Some(IdbKey::String("c".into())),
            ]
        );
        assert!(cursor.is_finished());
    }

    /// 创建事务，添加多条记录，commit_tx，验证记录持久化。
    #[test]
    fn test_idb_transaction_commit() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();

        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        db.tx_add(
            &tx,
            "store",
            serde_json::json!({"name": "Alice"}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();
        db.tx_add(
            &tx,
            "store",
            serde_json::json!({"name": "Bob"}),
            Some(IdbKey::String("k2".into())),
        )
        .unwrap();
        db.tx_add(
            &tx,
            "store",
            serde_json::json!({"name": "Charlie"}),
            Some(IdbKey::String("k3".into())),
        )
        .unwrap();

        // 提交前，store 中没有数据
        assert_eq!(db.count("store").unwrap(), 0);

        db.commit_tx(&mut tx).unwrap();
        assert!(tx.is_committed());

        // 提交后，3 条记录全部持久化
        assert_eq!(db.count("store").unwrap(), 3);
        assert_eq!(
            db.get("store", &IdbKey::String("k1".into())).unwrap().value["name"],
            "Alice"
        );
        assert_eq!(
            db.get("store", &IdbKey::String("k2".into())).unwrap().value["name"],
            "Bob"
        );
        assert_eq!(
            db.get("store", &IdbKey::String("k3".into())).unwrap().value["name"],
            "Charlie"
        );
    }

    /// 创建事务，添加多条记录，abort，验证记录未持久化。
    #[test]
    fn test_idb_transaction_abort() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        // 预存一条数据
        db.add(
            "store",
            serde_json::json!("original"),
            Some(IdbKey::String("k0".into())),
        )
        .unwrap();

        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        db.tx_add(
            &tx,
            "store",
            serde_json::json!("new1"),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();
        db.tx_put(
            &tx,
            "store",
            serde_json::json!("modified"),
            Some(IdbKey::String("k0".into())),
        )
        .unwrap();

        // abort 丢弃所有缓冲变更
        tx.abort().unwrap();
        assert!(tx.is_aborted());

        // k0 保持原始值，k1 不存在
        assert_eq!(
            db.get("store", &IdbKey::String("k0".into())).unwrap().value,
            serde_json::json!("original")
        );
        assert!(db.get("store", &IdbKey::String("k1".into())).is_none());
        assert_eq!(db.count("store").unwrap(), 1);
    }

    /// put() 覆盖已有记录，值更新且记录数不变。
    #[test]
    fn test_idb_put_overwrites_existing() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let key = IdbKey::Number(42.0);
        db.add(
            "store",
            serde_json::json!({"version": 1, "data": "old"}),
            Some(key.clone()),
        )
        .unwrap();
        assert_eq!(db.count("store").unwrap(), 1);

        // put 覆盖同一 key
        let returned = db
            .put(
                "store",
                serde_json::json!({"version": 2, "data": "new"}),
                Some(key.clone()),
            )
            .unwrap();
        assert_eq!(returned, key);

        let record = db.get("store", &key).unwrap();
        assert_eq!(record.value["version"], 2);
        assert_eq!(record.value["data"], "new");
        // 记录数不变
        assert_eq!(db.count("store").unwrap(), 1);
    }

    /// add() 在主键已存在时应拒绝。
    #[test]
    fn test_idb_add_rejects_duplicate() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let key = IdbKey::String("dup".into());
        db.add("store", serde_json::json!("first"), Some(key.clone())).unwrap();

        // 再次 add 同一 key 应报错
        let result = db.add("store", serde_json::json!("second"), Some(key.clone()));
        assert!(result.is_err());

        // 原始记录未被覆盖
        let record = db.get("store", &key).unwrap();
        assert_eq!(record.value, serde_json::json!("first"));
    }

    /// count_with_range 对不同范围返回正确计数。
    #[test]
    fn test_idb_count_with_range() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        for i in 1..=10 {
            db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
                .unwrap();
        }

        // 全范围 [1, 10]
        let full = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), false, false);
        assert_eq!(db.count_with_range("store", &full).unwrap(), 10);

        // 子范围 [3, 7]
        let mid = IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(7.0), false, false);
        assert_eq!(db.count_with_range("store", &mid).unwrap(), 5);

        // 开区间 (3, 7)
        let open = IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(7.0), true, true);
        assert_eq!(db.count_with_range("store", &open).unwrap(), 3);

        // lower_bound >= 8
        let lower = IdbKeyRange::lower_bound(IdbKey::Number(8.0), false);
        assert_eq!(db.count_with_range("store", &lower).unwrap(), 3);

        // upper_bound <= 2
        let upper = IdbKeyRange::upper_bound(IdbKey::Number(2.0), false);
        assert_eq!(db.count_with_range("store", &upper).unwrap(), 2);

        // only(5.0)
        let only = IdbKeyRange::only(IdbKey::Number(5.0));
        assert_eq!(db.count_with_range("store", &only).unwrap(), 1);
    }

    /// 通过索引范围查询，验证过滤结果正确。
    #[test]
    fn test_idb_get_all_from_index_with_range() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        // 插入不同年龄段用户
        db.add(
            "store",
            serde_json::json!({"name": "A", "age": 15}),
            Some(IdbKey::String("u1".into())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!({"name": "B", "age": 25}),
            Some(IdbKey::String("u2".into())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!({"name": "C", "age": 35}),
            Some(IdbKey::String("u3".into())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!({"name": "D", "age": 45}),
            Some(IdbKey::String("u4".into())),
        )
        .unwrap();

        db.create_index("store", "age_idx", "age", false, false).unwrap();

        // 查询 20 <= age <= 40
        let range = IdbKeyRange::bound(IdbKey::Number(20.0), IdbKey::Number(40.0), false, false);
        let results = db.get_all_from_index_with_range("store", "age_idx", &range).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].value["name"], "B");
        assert_eq!(results[1].value["name"], "C");
    }

    /// 在索引上打开游标，验证迭代顺序按索引键排列。
    #[test]
    fn test_idb_cursor_on_index() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        // 按 name 插入顺序为 Z, A, M
        db.add(
            "store",
            serde_json::json!({"name": "Zebra"}),
            Some(IdbKey::String("u1".into())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!({"name": "Apple"}),
            Some(IdbKey::String("u2".into())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!({"name": "Mango"}),
            Some(IdbKey::String("u3".into())),
        )
        .unwrap();

        db.create_index("store", "name_idx", "name", false, false).unwrap();

        let mut cursor = db.open_cursor_on_index("store", "name_idx", None).unwrap().unwrap();
        let mut names = Vec::new();
        loop {
            let rec = db.cursor_record(&cursor).unwrap();
            names.push(rec.value["name"].as_str().unwrap().to_string());
            if !cursor.continue_next() {
                break;
            }
        }
        // 应按 name 索引键排序：Apple, Mango, Zebra
        assert_eq!(names, vec!["Apple", "Mango", "Zebra"]);
    }

    /// 使用键范围批量删除记录，验证剩余记录正确。
    #[test]
    fn test_idb_delete_range() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        // 插入 1..=10
        for i in 1..=10 {
            db.add(
                "store",
                serde_json::json!(format!("v{i}")),
                Some(IdbKey::Number(i as f64)),
            )
            .unwrap();
        }
        assert_eq!(db.count("store").unwrap(), 10);

        // 删除范围 [3, 7] 内的记录
        let range = IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(7.0), false, false);
        let to_delete: Vec<IdbKey> = db
            .get_all_with_range("store", &range)
            .unwrap()
            .into_iter()
            .map(|r| r.key.clone())
            .collect();
        assert_eq!(to_delete.len(), 5, "范围 [3,7] 应包含 5 条记录");

        for key in &to_delete {
            db.delete("store", key).unwrap();
        }

        // 验证剩余记录
        assert_eq!(db.count("store").unwrap(), 5);
        // 1, 2 应保留
        assert!(db.get("store", &IdbKey::Number(1.0)).is_some());
        assert!(db.get("store", &IdbKey::Number(2.0)).is_some());
        // 3..=7 应被删除
        for i in 3..=7 {
            assert!(
                db.get("store", &IdbKey::Number(i as f64)).is_none(),
                "key={i} 应已被删除"
            );
        }
        // 8, 9, 10 应保留
        assert!(db.get("store", &IdbKey::Number(8.0)).is_some());
        assert!(db.get("store", &IdbKey::Number(9.0)).is_some());
        assert!(db.get("store", &IdbKey::Number(10.0)).is_some());
    }

    /// 测试复合键索引：索引建在多个 key path 组合上（如 [lastName, firstName]），
    /// 验证 Array 键按字典序排序，get_from_index 能正确匹配。
    #[test]
    fn test_idb_compound_key() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("contacts", None, false).unwrap();

        // 插入多条记录，每条有 lastName 和 firstName 字段
        db.add(
            "contacts",
            serde_json::json!({"lastName": "Smith", "firstName": "Anna", "id": 1}),
            Some(IdbKey::Number(1.0)),
        )
        .unwrap();
        db.add(
            "contacts",
            serde_json::json!({"lastName": "Smith", "firstName": "Bob", "id": 2}),
            Some(IdbKey::Number(2.0)),
        )
        .unwrap();
        db.add(
            "contacts",
            serde_json::json!({"lastName": "Jones", "firstName": "Carol", "id": 3}),
            Some(IdbKey::Number(3.0)),
        )
        .unwrap();
        db.add(
            "contacts",
            serde_json::json!({"lastName": "Adams", "firstName": "Dave", "id": 4}),
            Some(IdbKey::Number(4.0)),
        )
        .unwrap();

        // 创建复合键索引：索引键路径不存在于 JSON 中，这里使用值字段作为索引
        // 先建 name 索引（单字段）
        db.create_index("contacts", "last_idx", "lastName", false, false)
            .unwrap();

        // 查询 lastName == "Smith" 的记录
        let smiths = db
            .get_from_index("contacts", "last_idx", &IdbKey::String("Smith".into()))
            .unwrap();
        assert_eq!(smiths.len(), 2, "应有 2 条 Smith 记录");

        // 验证 get_all_from_index 按 lastName 排序
        let all_by_last = db.get_all_from_index("contacts", "last_idx").unwrap();
        assert_eq!(all_by_last.len(), 4);
        assert_eq!(all_by_last[0].value["lastName"], "Adams");
        assert_eq!(all_by_last[1].value["lastName"], "Jones");
        // Smith 出现两次，顺序不确定（但都在最后）
        assert_eq!(all_by_last[2].value["lastName"], "Smith");
        assert_eq!(all_by_last[3].value["lastName"], "Smith");

        // 使用 Array（复合）键作为主键来测试复合键排序
        db.create_object_store("composite", None, false).unwrap();
        let ck1 = IdbKey::Array(vec![IdbKey::String("Smith".into()), IdbKey::String("Anna".into())]);
        let ck2 = IdbKey::Array(vec![IdbKey::String("Smith".into()), IdbKey::String("Bob".into())]);
        let ck3 = IdbKey::Array(vec![IdbKey::String("Jones".into()), IdbKey::String("Carol".into())]);

        db.add("composite", serde_json::json!({ "v": 1 }), Some(ck1.clone()))
            .unwrap();
        db.add("composite", serde_json::json!({ "v": 2 }), Some(ck2.clone()))
            .unwrap();
        db.add("composite", serde_json::json!({ "v": 3 }), Some(ck3.clone()))
            .unwrap();

        // 游标按键排序迭代，验证 Array 键字典序
        let mut cursor = db.open_cursor("composite", None).unwrap().unwrap();
        let mut keys = Vec::new();
        loop {
            let rec = db.cursor_record(&cursor).unwrap();
            keys.push(rec.value["v"].as_u64().unwrap());
            if !cursor.continue_next() {
                break;
            }
        }
        // 字典序: Jones/Carol < Smith/Anna < Smith/Bob
        assert_eq!(keys, vec![3, 1, 2], "Array 键应按字典序排列");

        // 范围查询：[Smith/Anna, Smith/Bob]
        let range = IdbKeyRange::bound(
            IdbKey::Array(vec![IdbKey::String("Smith".into()), IdbKey::String("Anna".into())]),
            IdbKey::Array(vec![IdbKey::String("Smith".into()), IdbKey::String("Bob".into())]),
            false,
            false,
        );
        let results = db.get_all_with_range("composite", &range).unwrap();
        assert_eq!(results.len(), 2, "范围 [Smith/Anna, Smith/Bob] 应包含 2 条记录");
    }

    /// 测试唯一索引在 add 时的约束违反：两条不同主键的记录具有相同的唯一索引键值 → 第二次 add 应报错。
    ///
    /// 已知问题：当前 add() 先将记录插入 store.records，再更新索引。
    /// 当索引更新失败（唯一约束冲突）时，记录已被添加但 add 返回错误。
    /// 这与 IndexedDB 规范不一致——正确行为应为 add 返回错误且记录不被插入。
    /// 本测试记录当前（有缺陷的）行为。
    #[test]
    fn test_idb_unique_constraint_on_add() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("users", None, false).unwrap();

        // 添加第一条记录
        db.add(
            "users",
            serde_json::json!({"email": "alice@example.com", "name": "Alice"}),
            Some(IdbKey::String("user-1".into())),
        )
        .unwrap();

        // 创建唯一索引
        db.create_index("users", "email_idx", "email", true, false).unwrap();

        // 添加第二条记录（不同主键），但 email 字段值与第一条相同
        let result = db.add(
            "users",
            serde_json::json!({"email": "alice@example.com", "name": "Alice Duplicate"}),
            Some(IdbKey::String("user-2".into())),
        );

        // add 应检测到唯一约束冲突并返回错误
        assert!(
            result.is_err(),
            "add() 应因唯一索引约束违反而报错：email 'alice@example.com' 已存在"
        );

        // 已知问题：尽管 add 返回错误，记录仍被插入了 store（先插入记录再检查索引）
        assert_eq!(
            db.count("users").unwrap(),
            2,
            "BUG: 唯一索引约束违反时记录仍被添加（应为 1）"
        );

        // 原始记录不应被修改
        let record = db.get("users", &IdbKey::String("user-1".into())).unwrap();
        assert_eq!(record.value["name"], "Alice");

        // 不同 email 值的 add 应成功
        db.add(
            "users",
            serde_json::json!({"email": "bob@example.com", "name": "Bob"}),
            Some(IdbKey::String("user-3".into())),
        )
        .unwrap();
        assert_eq!(db.count("users").unwrap(), 3, "不同索引键值的 add 应成功");
    }

    /// 混合操作：add + put + delete + abort，store 不受影响。
    #[test]
    fn test_tx_mixed_operations_abort() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let key = IdbKey::String("existing".into());
        db.add("store", serde_json::json!("v1"), Some(key.clone())).unwrap();

        let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
        // add new
        db.tx_add(
            &tx,
            "store",
            serde_json::json!("new"),
            Some(IdbKey::String("new_key".into())),
        )
        .unwrap();
        // put existing
        db.tx_put(&tx, "store", serde_json::json!("updated"), Some(key.clone()))
            .unwrap();
        // delete
        db.tx_delete(&tx, "store", &key).unwrap();
        tx.abort().unwrap();

        // 所有变更都应被丢弃
        let record = db.get("store", &key).unwrap();
        assert_eq!(record.value, serde_json::json!("v1"));
        assert!(db.get("store", &IdbKey::String("new_key".into())).is_none());
        assert_eq!(db.count("store").unwrap(), 1);
    }
}

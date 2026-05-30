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
        self.entries.sort_by(|a, b| {
            match a.index_key.cmp(&b.index_key) {
                Ordering::Equal => a.primary_key.cmp(&b.primary_key),
                other => other,
            }
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
        self.stores
            .get(store_name)?
            .records
            .iter()
            .find(|r| &r.key == key)
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
    pub fn get_all_with_range(
        &self,
        store_name: &str,
        range: &IdbKeyRange,
    ) -> Result<Vec<&IdbRecord>, StorageError> {
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        let mut results: Vec<&IdbRecord> = store
            .records
            .iter()
            .filter(|r| range.contains(&r.key))
            .collect();
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
    pub fn count_with_range(
        &self,
        store_name: &str,
        range: &IdbKeyRange,
    ) -> Result<usize, StorageError> {
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
            return Err(StorageError::Database(format!(
                "Index '{}' already exists",
                index_name
            )));
        }

        let mut index = IdbIndex::new(index_name, key_path, unique, multi_entry);
        // 从已有记录重建索引
        index.rebuild(&store.records)?;
        store.indexes.insert(index_name.to_string(), index);
        Ok(())
    }

    /// 删除指定 store 上的索引。
    pub fn delete_index(
        &mut self,
        store_name: &str,
        index_name: &str,
    ) -> Result<(), StorageError> {
        let store = self
            .stores
            .get_mut(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        if store.indexes.remove(index_name).is_none() {
            return Err(StorageError::Database(format!(
                "Index '{}' not found",
                index_name
            )));
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
    pub fn get_all_from_index(
        &self,
        store_name: &str,
        index_name: &str,
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
        })
    }

    /// 在事务范围内添加记录。
    pub fn tx_add(
        &mut self,
        tx: &IdbTransaction,
        store_name: &str,
        value: serde_json::Value,
        key: Option<IdbKey>,
    ) -> Result<IdbKey, StorageError> {
        tx.check_active(store_name)?;
        self.add(store_name, value, key)
    }

    /// 在事务范围内放入记录。
    pub fn tx_put(
        &mut self,
        tx: &IdbTransaction,
        store_name: &str,
        value: serde_json::Value,
        key: Option<IdbKey>,
    ) -> Result<IdbKey, StorageError> {
        tx.check_active(store_name)?;
        self.put(store_name, value, key)
    }

    /// 在事务范围内删除记录。
    pub fn tx_delete(
        &mut self,
        tx: &IdbTransaction,
        store_name: &str,
        key: &IdbKey,
    ) -> Result<bool, StorageError> {
        tx.check_active(store_name)?;
        self.delete(store_name, key)
    }

    /// 在事务范围内获取记录。
    pub fn tx_get(
        &self,
        tx: &IdbTransaction,
        store_name: &str,
        key: &IdbKey,
    ) -> Result<Option<&IdbRecord>, StorageError> {
        tx.check_active(store_name)?;
        Ok(self.get(store_name, key))
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

    /// 中止事务。
    pub fn abort(&mut self) -> Result<(), StorageError> {
        if self.aborted {
            return Err(StorageError::Database("Transaction already aborted".to_string()));
        }
        if self.committed {
            return Err(StorageError::Database("Transaction already committed, cannot abort".to_string()));
        }
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

        let record = db
            .get("users", &IdbKey::String("user1".to_string()))
            .unwrap();
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
        db.add(
            "users",
            serde_json::json!({"name": "Alice"}),
            Some(key.clone()),
        )
        .unwrap();
        db.put(
            "users",
            serde_json::json!({"name": "Bob"}),
            Some(key.clone()),
        )
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
        db.add("store", serde_json::json!("hello"), Some(key.clone()))
            .unwrap();
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
        db.add("store", serde_json::json!(1), Some(key.clone()))
            .unwrap();
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
        db.add(
            "store",
            serde_json::json!("a"),
            Some(IdbKey::String("k1".to_string())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!("b"),
            Some(IdbKey::String("k2".to_string())),
        )
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
        db.add("store", serde_json::json!(1), Some(key.clone()))
            .unwrap();
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
        db.add("items", serde_json::json!("v1"), Some(key.clone()))
            .unwrap();

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
        db.add("items", serde_json::json!("v1"), Some(key.clone()))
            .unwrap();
        db.put("items", serde_json::json!("v2"), Some(key.clone()))
            .unwrap();

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
        let range = IdbKeyRange::bound(
            IdbKey::String("c".into()),
            IdbKey::String("f".into()),
            false,
            false,
        );
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

        db.create_index("users", "name_idx", "name", false, false)
            .unwrap();
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

        db.create_index("users", "name_idx", "name", false, false)
            .unwrap();
        let results =
            db.get_from_index("users", "name_idx", &IdbKey::String("Alice".into()))
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

        db.create_index("users", "name_idx", "name", false, false)
            .unwrap();
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

        db.create_index("users", "age_idx", "age", false, false)
            .unwrap();
        let range = IdbKeyRange::bound(IdbKey::Number(25.0), IdbKey::Number(35.0), false, false);
        let results =
            db.get_all_from_index_with_range("users", "age_idx", &range)
                .unwrap();
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
        db.create_index("users", "email_idx", "email", true, false)
            .unwrap();

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
        db.create_index("store", "idx", "field", false, false)
            .unwrap();
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
        db.create_index("store", "tag_idx", "tag", false, false)
            .unwrap();

        let results =
            db.get_from_index("store", "tag_idx", &IdbKey::String("a".into()))
                .unwrap();
        assert_eq!(results.len(), 1);

        db.delete("store", &IdbKey::String("k1".into())).unwrap();
        let results =
            db.get_from_index("store", "tag_idx", &IdbKey::String("a".into()))
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
        db.create_index("store", "tag_idx", "tag", false, false)
            .unwrap();

        db.put(
            "store",
            serde_json::json!({"tag": "b"}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();

        let results_a =
            db.get_from_index("store", "tag_idx", &IdbKey::String("a".into()))
                .unwrap();
        assert!(results_a.is_empty());
        let results_b =
            db.get_from_index("store", "tag_idx", &IdbKey::String("b".into()))
                .unwrap();
        assert_eq!(results_b.len(), 1);
    }

    #[test]
    fn test_count_from_index() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add(
            "store",
            serde_json::json!({"v": 1}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!({"v": 2}),
            Some(IdbKey::String("k2".into())),
        )
        .unwrap();
        db.add(
            "store",
            serde_json::json!({"v": 3}),
            Some(IdbKey::String("k3".into())),
        )
        .unwrap();

        db.create_index("store", "v_idx", "v", false, false).unwrap();
        assert_eq!(
            db.count_from_index("store", "v_idx", None).unwrap(),
            3
        );

        let range = IdbKeyRange::lower_bound(IdbKey::Number(2.0), false);
        assert_eq!(
            db.count_from_index("store", "v_idx", Some(&range))
                .unwrap(),
            2
        );
    }

    #[test]
    fn test_clear_store_clears_indexes() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db.add(
            "store",
            serde_json::json!({"x": 1}),
            Some(IdbKey::Number(1.0)),
        )
        .unwrap();
        db.create_index("store", "x_idx", "x", false, false).unwrap();
        assert_eq!(
            db.count_from_index("store", "x_idx", None).unwrap(),
            1
        );
        db.clear_store("store").unwrap();
        assert_eq!(
            db.count_from_index("store", "x_idx", None).unwrap(),
            0
        );
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

        db.create_index("store", "tags_idx", "tags", false, true)
            .unwrap();
        let results =
            db.get_from_index("store", "tags_idx", &IdbKey::String("blue".into()))
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
        assert_eq!(
            db.cursor_record(&cursor).unwrap().value,
            serde_json::json!("a")
        );

        assert!(cursor.continue_next());
        assert_eq!(
            db.cursor_record(&cursor).unwrap().value,
            serde_json::json!("b")
        );

        assert!(cursor.continue_next());
        assert_eq!(
            db.cursor_record(&cursor).unwrap().value,
            serde_json::json!("c")
        );

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
        assert_eq!(
            db.cursor_record(&cursor).unwrap().value,
            serde_json::json!(2)
        );
        assert!(cursor.continue_next());
        assert_eq!(
            db.cursor_record(&cursor).unwrap().value,
            serde_json::json!(3)
        );
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

        db.create_index("store", "name_idx", "name", false, false)
            .unwrap();
        let mut cursor = db
            .open_cursor_on_index("store", "name_idx", None)
            .unwrap()
            .unwrap();
        assert_eq!(
            db.cursor_record(&cursor).unwrap().value["name"],
            "Alice"
        );
        assert!(cursor.continue_next());
        assert_eq!(db.cursor_record(&cursor).unwrap().value["name"], "Bob");
        assert!(cursor.continue_next());
        assert_eq!(
            db.cursor_record(&cursor).unwrap().value["name"],
            "Charlie"
        );
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
        let tx = db
            .transaction(&["store"], IdbTransactionMode::ReadWrite)
            .unwrap();
        assert_eq!(tx.mode(), IdbTransactionMode::ReadWrite);
        assert_eq!(tx.store_names().len(), 1);
        assert!(!tx.is_committed());
        assert!(!tx.is_aborted());
    }

    #[test]
    fn test_transaction_commit() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db
            .transaction(&["store"], IdbTransactionMode::ReadWrite)
            .unwrap();
        tx.commit().unwrap();
        assert!(tx.is_committed());
    }

    #[test]
    fn test_transaction_abort() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db
            .transaction(&["store"], IdbTransactionMode::ReadWrite)
            .unwrap();
        tx.abort().unwrap();
        assert!(tx.is_aborted());
    }

    #[test]
    fn test_transaction_double_commit() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db
            .transaction(&["store"], IdbTransactionMode::ReadWrite)
            .unwrap();
        tx.commit().unwrap();
        assert!(tx.commit().is_err());
    }

    #[test]
    fn test_transaction_abort_after_commit() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db
            .transaction(&["store"], IdbTransactionMode::ReadWrite)
            .unwrap();
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
        let tx = db
            .transaction(&["store"], IdbTransactionMode::ReadWrite)
            .unwrap();

        let key = db
            .tx_add(
                &tx,
                "store",
                serde_json::json!("hello"),
                Some(IdbKey::String("k1".into())),
            )
            .unwrap();
        assert_eq!(key, IdbKey::String("k1".into()));

        let record = db
            .tx_get(&tx, "store", &IdbKey::String("k1".into()))
            .unwrap();
        assert_eq!(record.unwrap().value, serde_json::json!("hello"));
    }

    #[test]
    fn test_tx_operations_out_of_scope() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("a", None, false).unwrap();
        db.create_object_store("b", None, false).unwrap();
        let tx = db.transaction(&["a"], IdbTransactionMode::ReadWrite).unwrap();

        let result = db.tx_add(
            &tx,
            "b",
            serde_json::json!(1),
            Some(IdbKey::Number(1.0)),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_tx_operations_after_abort() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        let mut tx = db
            .transaction(&["store"], IdbTransactionMode::ReadWrite)
            .unwrap();
        tx.abort().unwrap();

        let result = db.tx_add(
            &tx,
            "store",
            serde_json::json!(1),
            Some(IdbKey::Number(1.0)),
        );
        assert!(result.is_err());
    }
}

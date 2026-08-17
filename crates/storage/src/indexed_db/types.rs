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
    /// Date 键，值为 Unix epoch 毫秒。
    Date(f64),
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
    /// 是否为合法 IndexedDB key（W3C IndexedDB §3.1.6）——Number 不可为 NaN；Array 递归校验所有元素。
    /// R3227：NaN key 须拒（DataError）；旧 cmp_key 对 NaN `partial_cmp().unwrap_or(Equal)` 致 NaN 与任意键
    /// 「相等」，破坏排序/去重。在 add/put 入口校验，避免 NaN 入库。
    pub fn is_valid_key(&self) -> bool {
        match self {
            IdbKey::Number(n) => !n.is_nan(),
            IdbKey::Date(milliseconds) => milliseconds.is_finite(),
            IdbKey::String(_) | IdbKey::Binary(_) => true,
            IdbKey::Array(ks) => ks.iter().all(|k| k.is_valid_key()),
        }
    }

    /// 内部比较辅助，返回 Ordering。
    fn cmp_key(&self, other: &Self) -> Ordering {
        match (self, other) {
            (IdbKey::Number(a), IdbKey::Number(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (IdbKey::Number(_), IdbKey::Date(_)) => Ordering::Less,
            (IdbKey::Number(_), IdbKey::String(_)) => Ordering::Less,
            (IdbKey::Number(_), IdbKey::Binary(_)) => Ordering::Less,
            (IdbKey::Number(_), IdbKey::Array(_)) => Ordering::Less,

            (IdbKey::Date(_), IdbKey::Number(_)) => Ordering::Greater,
            (IdbKey::Date(a), IdbKey::Date(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (IdbKey::Date(_), IdbKey::String(_)) => Ordering::Less,
            (IdbKey::Date(_), IdbKey::Binary(_)) => Ordering::Less,
            (IdbKey::Date(_), IdbKey::Array(_)) => Ordering::Less,

            (IdbKey::String(_), IdbKey::Number(_)) => Ordering::Greater,
            (IdbKey::String(_), IdbKey::Date(_)) => Ordering::Greater,
            (IdbKey::String(a), IdbKey::String(b)) => a.cmp(b),
            (IdbKey::String(_), IdbKey::Binary(_)) => Ordering::Less,
            (IdbKey::String(_), IdbKey::Array(_)) => Ordering::Less,

            (IdbKey::Binary(_), IdbKey::Number(_)) => Ordering::Greater,
            (IdbKey::Binary(_), IdbKey::Date(_)) => Ordering::Greater,
            (IdbKey::Binary(_), IdbKey::String(_)) => Ordering::Greater,
            (IdbKey::Binary(a), IdbKey::Binary(b)) => a.cmp(b),
            (IdbKey::Binary(_), IdbKey::Array(_)) => Ordering::Less,

            (IdbKey::Array(_), IdbKey::Number(_)) => Ordering::Greater,
            (IdbKey::Array(_), IdbKey::Date(_)) => Ordering::Greater,
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
            IdbKey::Date(milliseconds) => {
                let normalized = if *milliseconds == 0.0 { 0.0_f64 } else { *milliseconds };
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
        self.sort_entries();
        Ok(())
    }

    /// 校验：从 `value` 提取的 index keys 加入后是否违反 unique 约束（排除 `primary_key` 自身旧条目；
    /// 含同批重复检测——multiEntry 同 record 多个相同 key）。非 mutating——供 add/put 在 mutate
    /// record 前预校验，闭合跨 index 原子性（R3228）。非 unique index 恒 Ok。
    fn check_unique(&self, primary_key: &IdbKey, value: &serde_json::Value) -> Result<(), StorageError> {
        if !self.unique {
            return Ok(());
        }
        let keys = self.extract_keys(value);
        let mut seen: Vec<IdbKey> = Vec::with_capacity(keys.len());
        for index_key in &keys {
            let conflict = self
                .entries
                .iter()
                .any(|e| &e.index_key == index_key && &e.primary_key != primary_key)
                || seen.iter().any(|s| s == index_key);
            if conflict {
                return Err(StorageError::Database(format!(
                    "Unique index '{}' constraint violation for key {:?}",
                    self.name, index_key
                )));
            }
            seen.push(index_key.clone());
        }
        Ok(())
    }

    /// 仅提交（不校验）——从 record 提取 keys 批量 push。供 add/put 预校验（check_unique）全过后调用，
    /// 避免重复校验。rebuild 等无预校验场景用 [`add_entry_from_record`]（校验 + 提交）。
    ///
    /// R3385：push 后须重排，维持 [`sorted_entries`] 的「按索引键排序，相同索引键按主键排序」
    /// 不变量。旧实现仅 push 不重排，致 `add`/`put` 后 `entries` 失序，
    /// `get_all_from_index` / `get_all_from_index_with_range` / 经索引游标返回的记录违反
    /// W3C IndexedDB「按索引键有序」契约（cursor / getAllFromIndex 须按 index-key 序）。
    fn commit_entry_from_record(&mut self, record: &IdbRecord) {
        let primary_key = record.key.clone();
        let keys = self.extract_keys(&record.value);
        for index_key in keys {
            self.entries.push(IndexEntry {
                index_key,
                primary_key: primary_key.clone(),
            });
        }
        self.sort_entries();
    }

    /// 按（索引键，主键）字典序重排条目，与 [`rebuild`] 的排序口径一致。
    fn sort_entries(&mut self) {
        self.entries.sort_by(|a, b| match a.index_key.cmp(&b.index_key) {
            Ordering::Equal => a.primary_key.cmp(&b.primary_key),
            other => other,
        });
    }

    /// 从单条记录添加索引条目（校验 + 提交，原子——R3228：部分 key 违 unique 不再部分提交）。
    fn add_entry_from_record(&mut self, record: &IdbRecord) -> Result<(), StorageError> {
        self.check_unique(&record.key, &record.value)?;
        self.commit_entry_from_record(record);
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
    /// 清空 object store。
    Clear {
        /// 目标 store 名称。
        store: String,
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

/// R3341：auto-increment key generator 的「显式数值 key 推进」规则（W3C IndexedDB §1.8.2）。
///
/// 当向使用 key generator 的 store 提供一个**数值** key 时，若该值 ≥ 生成器当前值，
/// 生成器推进到 `providedKey + 1`（取整后 +1，因生成器只产整数键）。返回推进后的新值；
/// 显式 key 小于当前值或非数值时不推进（取 max 语义，避免回退）。
///
/// - `current`：生成器当前值（store.next_key 或 tx-local tx_next）。
/// - `explicit`：调用方提供的显式 key。
fn advance_generator_for_explicit_key(current: u64, explicit: &IdbKey) -> u64 {
    if let IdbKey::Number(n) = explicit
        && n.is_finite()
    {
        // 生成器只产整数键；floor(n) + 1 为「providedKey 之后下一个整数」。
        // n 为负或 < current 时 max 保持 current（不回退）。
        let candidate = (n.floor() as i128).saturating_add(1);
        if candidate > current as i128 {
            return candidate.clamp(1, u64::MAX as i128) as u64;
        }
    }
    current
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

/// IndexedDB object store schema 摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdbObjectStoreInfo {
    /// Object store 名称。
    pub name: String,
    /// Inline key path；`None` 表示 out-of-line key。
    pub key_path: Option<String>,
    /// 是否启用 key generator。
    pub auto_increment: bool,
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

    /// 获取 object store schema 摘要，按名称排序。
    pub fn store_info(&self) -> Vec<IdbObjectStoreInfo> {
        let mut stores = self
            .stores
            .values()
            .map(|store| IdbObjectStoreInfo {
                name: store.name.clone(),
                key_path: store.key_path.clone(),
                auto_increment: store.auto_increment,
            })
            .collect::<Vec<_>>();
        stores.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        stores
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
            Some(k) => {
                // R3227：校验显式 key 合法性（NaN 拒，IndexedDB §3.1.6）
                if !k.is_valid_key() {
                    return Err(StorageError::Database(
                        "Invalid key (NaN is not a valid IndexedDB key)".to_string(),
                    ));
                }
                // R3341：显式数值 key ≥ 生成器当前值时推进（W3C §1.8.2）；非数值或更小不推进。
                if store.auto_increment {
                    store.next_key = advance_generator_for_explicit_key(store.next_key, &k);
                }
                k
            }
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

        // R3228：原子性——预校验所有 index（不 mutate record/index），全过后再 push record + commit index。
        // 旧实现先 push record 再 add index，index 违例时 record 已入库（数据不一致）。
        for idx in store.indexes.values() {
            idx.check_unique(&key, &value)?;
        }

        store.records.push(IdbRecord {
            key: key.clone(),
            value: value.clone(),
        });
        // 更新索引（已预校验，commit 不再失败）
        let record = store.records.last().unwrap();
        for idx in store.indexes.values_mut() {
            idx.commit_entry_from_record(record);
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
            Some(k) => {
                // R3227：校验显式 key 合法性（NaN 拒，IndexedDB §3.1.6）
                if !k.is_valid_key() {
                    return Err(StorageError::Database(
                        "Invalid key (NaN is not a valid IndexedDB key)".to_string(),
                    ));
                }
                // R3341：显式数值 key ≥ 生成器当前值时推进（W3C §1.8.2）。
                if store.auto_increment {
                    store.next_key = advance_generator_for_explicit_key(store.next_key, &k);
                }
                k
            }
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
            // R3228：原子性——预校验所有 index（用新 value，排除本 primary_key 旧条目），全过后再 mutate。
            // 旧实现先 remove 旧 index + mutate value 再 add 新 index，违例时 value 已变 + index 已部分移除。
            for idx in store.indexes.values() {
                idx.check_unique(&key, &value)?;
            }
            record.value = value.clone();
            for idx in store.indexes.values_mut() {
                idx.remove_by_primary_key(&key);
                idx.commit_entry_from_record(record);
            }
        } else {
            // 新键：同 add 路径（预校验 + push + commit）。
            for idx in store.indexes.values() {
                idx.check_unique(&key, &value)?;
            }
            store.records.push(IdbRecord {
                key: key.clone(),
                value: value.clone(),
            });
            let record = store.records.last().unwrap();
            for idx in store.indexes.values_mut() {
                idx.commit_entry_from_record(record);
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
            key_gens: RefCell::new(HashMap::new()),
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
        // R3229：事务局部 key generator（auto-inc 推进 tx-local key_gens，不触碰 live store.next_key；
        // commit_tx 写回，abort 丢弃 → store.next_key 未改自动回滚，闭合 W3C IndexedDB §5.10）。
        let store_next_key = store.next_key;
        let auto_inc = store.auto_increment;
        let mut key_gens = tx.key_gens.borrow_mut();
        let tx_next = key_gens.entry(store_name.to_string()).or_insert(store_next_key);
        let effective_next = *tx_next;
        // 解析主键（自增逻辑）
        let key = match key {
            Some(k) => {
                // R3227：校验显式 key 合法性（NaN 拒，IndexedDB §3.1.6）
                if !k.is_valid_key() {
                    return Err(StorageError::Database(
                        "Invalid key (NaN is not a valid IndexedDB key)".to_string(),
                    ));
                }
                k
            }
            None if auto_inc => IdbKey::Number(effective_next as f64),
            None => {
                return Err(StorageError::Database(
                    "No key provided and auto_increment is false".to_string(),
                ));
            }
        };
        // auto-inc 推进：auto 分配的 key 推进 +1；显式数值 key ≥ 当前值时推进到 providedKey+1（R3341 W3C §1.8.2 max 语义）。
        // 旧实现仅匹配 key == effective_next（窄匹配），显式 key > 当前值时不推进（漏 §1.8.2）。
        if auto_inc {
            *tx_next = advance_generator_for_explicit_key(effective_next, &key);
            // auto 分配（key == effective_next 的 Number）时 advance 已给出 effective_next+1，等价旧 +1 推进；
            // 显式 key 时按 §1.8.2 max 推进。
        }
        drop(key_gens);
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
        // R3229：key generator 推进已移至事务局部 key_gens（见上方 key 解析），不再触碰 live store.next_key。

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
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        // R3229：事务局部 key generator（同 tx_add——auto-inc 推进 tx-local key_gens，不触碰 live store.next_key）。
        let store_next_key = store.next_key;
        let auto_inc = store.auto_increment;
        let mut key_gens = tx.key_gens.borrow_mut();
        let tx_next = key_gens.entry(store_name.to_string()).or_insert(store_next_key);
        let effective_next = *tx_next;
        let key = match key {
            Some(k) => {
                // R3227：校验显式 key 合法性（NaN 拒，IndexedDB §3.1.6）
                if !k.is_valid_key() {
                    return Err(StorageError::Database(
                        "Invalid key (NaN is not a valid IndexedDB key)".to_string(),
                    ));
                }
                k
            }
            None if auto_inc => IdbKey::Number(effective_next as f64),
            None => {
                return Err(StorageError::Database(
                    "No key provided and auto_increment is false".to_string(),
                ));
            }
        };
        // R3341：auto-inc 推进（auto 分配 +1；显式数值 key ≥ 当前值时按 §1.8.2 max 推进），同 tx_add。
        if auto_inc {
            *tx_next = advance_generator_for_explicit_key(effective_next, &key);
        }
        drop(key_gens);
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

    /// 在事务范围内清空 store（缓冲，提交时生效）。
    pub fn tx_clear(&mut self, tx: &IdbTransaction, store_name: &str) -> Result<(), StorageError> {
        tx.check_active(store_name)?;
        tx.mutations.borrow_mut().push(TxMutation::Clear {
            store: store_name.to_string(),
        });
        Ok(())
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
                TxMutation::Clear { store } if store == store_name => return Ok(None),
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

    /// 获取事务可见的全部记录，按主键排序。
    pub fn tx_get_all(&self, tx: &IdbTransaction, store_name: &str) -> Result<Vec<IdbRecord>, StorageError> {
        tx.check_active(store_name)?;
        let store = self
            .stores
            .get(store_name)
            .ok_or_else(|| StorageError::StoreNotFound(store_name.to_string()))?;
        let mut records = store
            .records
            .iter()
            .map(|record| (record.key.clone(), record.value.clone()))
            .collect::<HashMap<_, _>>();
        for mutation in tx.mutations.borrow().iter() {
            match mutation {
                TxMutation::Add { store, key, value } | TxMutation::Put { store, key, value }
                    if store == store_name =>
                {
                    records.insert(key.clone(), value.clone());
                }
                TxMutation::Delete { store, key } if store == store_name => {
                    records.remove(key);
                }
                TxMutation::Clear { store } if store == store_name => {
                    records.clear();
                }
                _ => {}
            }
        }
        let mut records = records
            .into_iter()
            .map(|(key, value)| IdbRecord { key, value })
            .collect::<Vec<_>>();
        records.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(records)
    }

    /// 提交事务，将缓冲的变更应用到 store。
    pub fn commit_tx(&mut self, tx: &mut IdbTransaction) -> Result<(), StorageError> {
        tx.commit()?;
        let mutations = tx.mutations.borrow_mut().drain(..).collect::<Vec<_>>();
        // R3386：原子性预校验——先模拟应用全量变更（构建 commit 后的虚拟记录集 + index 键），
        // 检查 unique 索引冲突。旧实现直接逐条 self.add/put/delete 并 `?` 提前返回：缓冲记录间
        // 的 index 唯一性冲突（tx_add 阶段未对 buffered 记录预检）会在 commit 中途暴露——第一条
        // add 已入库，第二条 add 报错返回 → 部分提交，违反 W3C IndexedDB 事务原子性（§1.6）。
        // 预校验全过后再 apply，apply 阶段不再因 index 冲突中途失败。
        self.precheck_commit(&mutations)?;
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
                TxMutation::Clear { store } => {
                    self.clear_store(&store)?;
                }
            }
        }
        // R3229：写回事务局部 key generator 推进（auto-inc 增量）到 live store.next_key。
        // 取 max——tx 从 store.next_key 起单调推进；若 store.next_key 期间被独立推进（不应发生），保守保留大值。
        let key_gens = tx.key_gens.borrow();
        for (store_name, tx_next) in key_gens.iter() {
            if let Some(store) = self.stores.get_mut(store_name)
                && *tx_next > store.next_key
            {
                store.next_key = *tx_next;
            }
        }
        Ok(())
    }

    /// R3386：模拟提交后的记录集，预校验每条 unique 索引在 commit 后态无冲突。
    ///
    /// 构建每个 store「commit 后」的虚拟记录表（live 记录叠加 buffered Add/Put/Delete），
    /// 对每条 unique 索引重算键集检测重复。冲突则整批拒绝（commit_tx 不 mutate live store，
    /// 事务原子回滚）。
    fn precheck_commit(&self, mutations: &[TxMutation]) -> Result<(), StorageError> {
        // 按 store 分组虚拟记录：(primary_key, value)，后者用于提取 index 键。
        // 同一 primary_key 后到的 Add/Put 覆盖前者；Delete 移除。
        use std::collections::HashMap as StdMap;
        let mut per_store: StdMap<String, StdMap<IdbKey, serde_json::Value>> = StdMap::new();
        // 标记被 buffered Delete/Put 覆盖的 live key，避免重复计入。
        let mut deleted_keys: StdMap<String, std::collections::HashSet<IdbKey>> = StdMap::new();
        let mut cleared_stores = std::collections::HashSet::new();

        for m in mutations {
            match m {
                TxMutation::Add { store, value, key } => {
                    per_store
                        .entry(store.clone())
                        .or_default()
                        .insert(key.clone(), value.clone());
                }
                TxMutation::Put { store, value, key } => {
                    per_store
                        .entry(store.clone())
                        .or_default()
                        .insert(key.clone(), value.clone());
                    // Put 覆盖 live 记录：标记其 live 值不再独立计入（buffered 值已取代）。
                    deleted_keys.entry(store.clone()).or_default().insert(key.clone());
                }
                TxMutation::Delete { store, key } => {
                    per_store.entry(store.clone()).or_default().remove(key);
                    deleted_keys.entry(store.clone()).or_default().insert(key.clone());
                }
                TxMutation::Clear { store } => {
                    per_store.entry(store.clone()).or_default().clear();
                    deleted_keys.entry(store.clone()).or_default().clear();
                    cleared_stores.insert(store.clone());
                }
            }
        }

        // 对每个有 buffered 变更的 store，构建 commit 后记录集并校验 unique 索引。
        for (store_name, buffered) in &per_store {
            let Some(store) = self.stores.get(store_name) else {
                continue;
            };
            let deleted = deleted_keys.get(store_name);
            // commit 后记录集 = live 记录（排除被 buffered Put/Delete 覆盖的）∪ buffered Add/Put。
            // 收集 (primary_key, value) 对。
            let mut committed: Vec<(IdbKey, serde_json::Value)> = Vec::new();
            if !cleared_stores.contains(store_name) {
                for r in &store.records {
                    if let Some(true) = deleted.map(|d| d.contains(&r.key)) {
                        continue;
                    }
                    committed.push((r.key.clone(), r.value.clone()));
                }
            }
            for (k, v) in buffered {
                committed.push((k.clone(), v.clone()));
            }
            // 对每个 unique 索引，提取所有记录的 index 键，检测重复。
            for idx in store.indexes.values() {
                if !idx.unique {
                    continue;
                }
                let mut seen: Vec<IdbKey> = Vec::new();
                for (_pk, value) in &committed {
                    for ik in idx.extract_keys(value) {
                        if seen.iter().any(|s| s == &ik) {
                            return Err(StorageError::Database(format!(
                                "Unique index '{}' constraint violation for key {:?}",
                                idx.name, ik
                            )));
                        }
                        seen.push(ik);
                    }
                }
            }
        }
        Ok(())
    }
}

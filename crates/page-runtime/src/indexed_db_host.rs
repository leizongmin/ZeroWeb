//! IndexedDB 页面宿主。
//!
//! 本模块解析 `zero-engine` 同步 wire 请求，并在共享 [`StorageManager`] 上执行
//! per-origin 数据库与 schema 操作。

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use zero_engine::IndexedDbHandler;
use zero_storage::{IdbKey, IdbKeyRange, IdbTransaction, IdbTransactionMode, StorageError, StorageManager};

mod cursor;
#[cfg(test)]
#[path = "indexed_db_host/cursor_tests.rs"]
mod cursor_tests;
#[cfg(test)]
#[path = "indexed_db_host/persistence_tests.rs"]
mod persistence_tests;

use cursor::{
    ActiveIndexedDbCursor, CursorStep, IndexedDbCursorDirection, open_transaction_cursor, step_transaction_cursor,
};

/// IndexedDB key 的 JSON wire 表示。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum IndexedDbKeyWire {
    /// Number key；字符串编码保留 Infinity 与负零。
    Number(String),
    /// Date key；值为 Unix epoch 毫秒的字符串编码。
    Date(String),
    /// String key。
    String(String),
    /// Binary key。
    Binary(Vec<u8>),
    /// Array key。
    Array(Vec<IndexedDbKeyWire>),
}

impl IndexedDbKeyWire {
    /// 转换为 storage key，并校验非法 Number/Date。
    pub fn into_storage_key(self) -> Result<IdbKey, String> {
        let key = match self {
            Self::Number(value) => IdbKey::Number(parse_wire_number(&value)?),
            Self::Date(value) => {
                let milliseconds = parse_wire_number(&value)?;
                if !milliseconds.is_finite() {
                    return Err("DataError: Date key must be finite".to_string());
                }
                IdbKey::Date(milliseconds)
            }
            Self::String(value) => IdbKey::String(value),
            Self::Binary(value) => IdbKey::Binary(value),
            Self::Array(values) => IdbKey::Array(
                values
                    .into_iter()
                    .map(Self::into_storage_key)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        };
        if !key.is_valid_key() {
            return Err("DataError: invalid IndexedDB key".to_string());
        }
        Ok(key)
    }
}

impl From<&IdbKey> for IndexedDbKeyWire {
    fn from(key: &IdbKey) -> Self {
        match key {
            IdbKey::Number(value) => Self::Number(format_wire_number(*value)),
            IdbKey::Date(value) => Self::Date(format_wire_number(*value)),
            IdbKey::String(value) => Self::String(value.clone()),
            IdbKey::Binary(value) => Self::Binary(value.clone()),
            IdbKey::Array(values) => Self::Array(values.iter().map(Self::from).collect()),
        }
    }
}

fn parse_wire_number(value: &str) -> Result<f64, String> {
    match value {
        "Infinity" => Ok(f64::INFINITY),
        "-Infinity" => Ok(f64::NEG_INFINITY),
        _ => value
            .parse::<f64>()
            .map_err(|_| "DataError: invalid numeric IndexedDB key".to_string()),
    }
}

fn format_wire_number(value: f64) -> String {
    if value == f64::INFINITY {
        "Infinity".to_string()
    } else if value == f64::NEG_INFINITY {
        "-Infinity".to_string()
    } else if value == 0.0 && value.is_sign_negative() {
        "-0".to_string()
    } else {
        value.to_string()
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum IndexedDbRequest {
    ConnectionCapabilities,
    Inspect {
        name: String,
    },
    SyncSchema {
        name: String,
        version: u64,
        stores: Vec<IndexedDbStoreSchema>,
    },
    Open {
        name: String,
        version: u64,
    },
    DeleteDatabase {
        name: String,
    },
    Databases,
    CreateObjectStore {
        database: String,
        name: String,
        #[serde(default)]
        key_path: Option<IndexedDbIndexKeyPath>,
        #[serde(default)]
        auto_increment: bool,
    },
    DeleteObjectStore {
        database: String,
        name: String,
    },
    StoreNames {
        database: String,
    },
    BeginTransaction {
        database: String,
        stores: Vec<String>,
        mode: IndexedDbTransactionMode,
    },
    TransactionAdd {
        transaction: u64,
        store: String,
        value: Value,
        #[serde(default)]
        key: Option<IndexedDbKeyWire>,
    },
    TransactionPut {
        transaction: u64,
        store: String,
        value: Value,
        #[serde(default)]
        key: Option<IndexedDbKeyWire>,
    },
    TransactionGet {
        transaction: u64,
        store: String,
        key: IndexedDbKeyWire,
    },
    TransactionDelete {
        transaction: u64,
        store: String,
        key: IndexedDbKeyWire,
    },
    TransactionDeleteRange {
        transaction: u64,
        store: String,
        range: IndexedDbKeyRangeWire,
    },
    TransactionClear {
        transaction: u64,
        store: String,
    },
    TransactionCount {
        transaction: u64,
        store: String,
        #[serde(default)]
        query: Option<IndexedDbQueryWire>,
    },
    TransactionGetAll {
        transaction: u64,
        store: String,
        #[serde(default)]
        query: Option<IndexedDbQueryWire>,
        #[serde(default)]
        count: Option<usize>,
        #[serde(default)]
        keys_only: bool,
    },
    TransactionIndexGetAll {
        transaction: u64,
        store: String,
        index: String,
        #[serde(default)]
        query: Option<IndexedDbQueryWire>,
        #[serde(default)]
        count: Option<usize>,
    },
    TransactionOpenCursor {
        transaction: u64,
        store: String,
        #[serde(default)]
        index: Option<String>,
        #[serde(default)]
        query: Option<IndexedDbQueryWire>,
        direction: IndexedDbCursorDirection,
        #[serde(default)]
        key_only: bool,
    },
    TransactionCursorContinue {
        transaction: u64,
        cursor: u64,
        #[serde(default)]
        key: Option<IndexedDbKeyWire>,
    },
    TransactionCursorContinuePrimaryKey {
        transaction: u64,
        cursor: u64,
        key: IndexedDbKeyWire,
        primary_key: IndexedDbKeyWire,
    },
    TransactionCursorAdvance {
        transaction: u64,
        cursor: u64,
        count: u32,
    },
    CommitTransaction {
        transaction: u64,
    },
    AbortTransaction {
        transaction: u64,
    },
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum IndexedDbTransactionMode {
    Readonly,
    Readwrite,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
enum IndexedDbQueryWire {
    Key(IndexedDbKeyWire),
    Range(IndexedDbKeyRangeWire),
}

#[derive(Debug, Deserialize)]
struct IndexedDbKeyRangeWire {
    #[serde(default)]
    lower: Option<IndexedDbKeyWire>,
    #[serde(default)]
    upper: Option<IndexedDbKeyWire>,
    #[serde(default, rename = "lowerOpen")]
    lower_open: bool,
    #[serde(default, rename = "upperOpen")]
    upper_open: bool,
}

#[derive(Clone)]
enum IndexedDbQuery {
    Key(IdbKey),
    Range(IdbKeyRange),
}

#[derive(Debug, Deserialize)]
struct IndexedDbStoreSchema {
    name: String,
    #[serde(default, rename = "keyPath")]
    key_path: Option<IndexedDbIndexKeyPath>,
    #[serde(default, rename = "autoIncrement")]
    auto_increment: bool,
    #[serde(default)]
    indexes: Vec<IndexedDbIndexSchema>,
}

#[derive(Debug, Deserialize)]
struct IndexedDbIndexSchema {
    name: String,
    #[serde(rename = "keyPath")]
    key_path: IndexedDbIndexKeyPath,
    #[serde(default)]
    unique: bool,
    #[serde(default, rename = "multiEntry")]
    multi_entry: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
enum IndexedDbIndexKeyPath {
    String(String),
    Sequence(Vec<String>),
}

#[derive(Default)]
struct IndexedDbTransactionRegistry {
    next_id: u64,
    active: HashMap<u64, ActiveIndexedDbTransaction>,
}

struct ActiveIndexedDbTransaction {
    origin: String,
    database: String,
    transaction: IdbTransaction,
    mutation_generation: u64,
    next_cursor_id: u64,
    cursors: HashMap<u64, ActiveIndexedDbCursor>,
}

/// 构造由页面运行路径共享的 IndexedDB handler。
pub fn indexed_db_handler(storage: Arc<Mutex<StorageManager>>) -> IndexedDbHandler {
    let transactions = Arc::new(Mutex::new(IndexedDbTransactionRegistry::default()));
    Arc::new(move |origin, request| handle_request(&storage, &transactions, origin, request))
}

fn handle_request(
    storage: &Mutex<StorageManager>,
    transactions: &Mutex<IndexedDbTransactionRegistry>,
    origin: &str,
    request: &str,
) -> Result<String, String> {
    if origin == "null" {
        return Err("SecurityError: IndexedDB is unavailable for opaque origins".to_string());
    }

    let request: IndexedDbRequest =
        serde_json::from_str(request).map_err(|error| format!("DataError: invalid IndexedDB request: {error}"))?;
    let mut storage = storage
        .lock()
        .map_err(|_| "UnknownError: IndexedDB storage lock is poisoned".to_string())?;
    let mut transactions = transactions
        .lock()
        .map_err(|_| "UnknownError: IndexedDB transaction lock is poisoned".to_string())?;
    let response = dispatch_request(&mut storage, &mut transactions, origin, request)?;
    serde_json::to_string(&response).map_err(|error| format!("UnknownError: failed to serialize response: {error}"))
}

fn dispatch_request(
    storage: &mut StorageManager,
    transactions: &mut IndexedDbTransactionRegistry,
    origin: &str,
    request: IndexedDbRequest,
) -> Result<Value, String> {
    match request {
        IndexedDbRequest::ConnectionCapabilities => Ok(json!({
            "crossRenderer": false,
            "transactionScheduling": false,
        })),
        IndexedDbRequest::Inspect { name } => {
            let database = storage.indexed_db(origin, &name).map(database_schema_json);
            Ok(json!({"database": database}))
        }
        IndexedDbRequest::SyncSchema { name, version, stores } => sync_schema(storage, origin, &name, version, stores),
        IndexedDbRequest::Open { name, version } => {
            let old_version = storage
                .indexed_db(origin, &name)
                .map(|database| database.version)
                .unwrap_or(0);
            if version == 0 {
                return Err("TypeError: IndexedDB version must be greater than zero".to_string());
            }
            if version < old_version {
                return Err(format!(
                    "VersionError: requested version {version} is lower than current version {old_version}"
                ));
            }
            let database = storage.open_indexed_db(origin, &name, version).map_err(storage_error)?;
            let mut stores = database.store_names();
            stores.sort_unstable();
            Ok(json!({
                "name": database.name,
                "version": database.version,
                "oldVersion": old_version,
                "upgradeNeeded": old_version != database.version,
                "stores": stores,
            }))
        }
        IndexedDbRequest::DeleteDatabase { name } => {
            abort_database_transactions(transactions, origin, &name);
            let old_version = storage
                .indexed_db(origin, &name)
                .map(|database| database.version)
                .unwrap_or(0);
            Ok(json!({
                "deleted": storage.try_delete_indexed_db(origin, &name).map_err(storage_error)?,
                "oldVersion": old_version,
            }))
        }
        IndexedDbRequest::Databases => {
            let databases = storage
                .indexed_db_info(origin)
                .into_iter()
                .map(|database| json!({"name": database.name, "version": database.version}))
                .collect::<Vec<_>>();
            Ok(json!({"databases": databases}))
        }
        IndexedDbRequest::CreateObjectStore {
            database,
            name,
            key_path,
            auto_increment,
        } => {
            let database_ref = storage
                .indexed_db(origin, &database)
                .ok_or_else(|| "NotFoundError: IndexedDB database does not exist".to_string())?;
            if database_ref.has_store(&name) {
                return Err(format!("ConstraintError: object store '{name}' already exists"));
            }
            storage
                .mutate_indexed_db(origin, &database, |candidate| {
                    candidate.create_object_store_with_key_path(
                        &name,
                        key_path.as_ref().map(storage_index_key_path),
                        auto_increment,
                    )
                })
                .map_err(storage_error)?;
            Ok(json!({"created": true}))
        }
        IndexedDbRequest::DeleteObjectStore { database, name } => {
            let database_ref = storage
                .indexed_db(origin, &database)
                .ok_or_else(|| "NotFoundError: IndexedDB database does not exist".to_string())?;
            if !database_ref.has_store(&name) {
                return Err(format!("NotFoundError: object store '{name}' does not exist"));
            }
            storage
                .mutate_indexed_db(origin, &database, |candidate| candidate.delete_object_store(&name))
                .map_err(storage_error)?;
            Ok(json!({"deleted": true}))
        }
        IndexedDbRequest::StoreNames { database } => {
            let database = storage
                .indexed_db(origin, &database)
                .ok_or_else(|| "NotFoundError: IndexedDB database does not exist".to_string())?;
            let mut stores = database.store_names();
            stores.sort_unstable();
            Ok(json!({"stores": stores}))
        }
        IndexedDbRequest::BeginTransaction { database, stores, mode } => {
            let database_ref = storage
                .indexed_db_mut(origin, &database)
                .ok_or_else(|| "NotFoundError: IndexedDB database does not exist".to_string())?;
            let store_refs = stores.iter().map(String::as_str).collect::<Vec<_>>();
            let storage_mode = match mode {
                IndexedDbTransactionMode::Readonly => IdbTransactionMode::ReadOnly,
                IndexedDbTransactionMode::Readwrite => IdbTransactionMode::ReadWrite,
            };
            let transaction = database_ref
                .transaction(&store_refs, storage_mode)
                .map_err(storage_error)?;
            transactions.next_id = transactions
                .next_id
                .checked_add(1)
                .ok_or_else(|| "UnknownError: IndexedDB transaction id overflow".to_string())?;
            let transaction_id = transactions.next_id;
            transactions.active.insert(
                transaction_id,
                ActiveIndexedDbTransaction {
                    origin: origin.to_string(),
                    database,
                    transaction,
                    mutation_generation: 0,
                    next_cursor_id: 0,
                    cursors: HashMap::new(),
                },
            );
            Ok(json!({"transaction": transaction_id}))
        }
        IndexedDbRequest::TransactionAdd {
            transaction,
            store,
            value,
            key,
        } => {
            let active = active_transaction_mut(transactions, origin, transaction)?;
            require_write_transaction(active)?;
            let database = active_database_mut(storage, active)?;
            let key = key.map(IndexedDbKeyWire::into_storage_key).transpose()?;
            let key = database
                .tx_add(&active.transaction, &store, value, key)
                .map_err(storage_error)?;
            active.mutation_generation += 1;
            Ok(json!({"key": IndexedDbKeyWire::from(&key)}))
        }
        IndexedDbRequest::TransactionPut {
            transaction,
            store,
            value,
            key,
        } => {
            let active = active_transaction_mut(transactions, origin, transaction)?;
            require_write_transaction(active)?;
            let database = active_database_mut(storage, active)?;
            let key = key.map(IndexedDbKeyWire::into_storage_key).transpose()?;
            let key = database
                .tx_put(&active.transaction, &store, value, key)
                .map_err(storage_error)?;
            active.mutation_generation += 1;
            Ok(json!({"key": IndexedDbKeyWire::from(&key)}))
        }
        IndexedDbRequest::TransactionGet {
            transaction,
            store,
            key,
        } => {
            let active = active_transaction_mut(transactions, origin, transaction)?;
            let database = active_database_mut(storage, active)?;
            let key = key.into_storage_key()?;
            let record = database
                .tx_get(&active.transaction, &store, &key)
                .map_err(storage_error)?
                .map(|record| {
                    json!({
                        "key": IndexedDbKeyWire::from(&record.key),
                        "value": record.value,
                    })
                });
            Ok(json!({"record": record}))
        }
        IndexedDbRequest::TransactionDelete {
            transaction,
            store,
            key,
        } => {
            let active = active_transaction_mut(transactions, origin, transaction)?;
            require_write_transaction(active)?;
            let database = active_database_mut(storage, active)?;
            let key = key.into_storage_key()?;
            let deleted = database
                .tx_delete(&active.transaction, &store, &key)
                .map_err(storage_error)?;
            active.mutation_generation += 1;
            Ok(json!({"deleted": deleted}))
        }
        IndexedDbRequest::TransactionDeleteRange {
            transaction,
            store,
            range,
        } => {
            let active = active_transaction_mut(transactions, origin, transaction)?;
            require_write_transaction(active)?;
            let database = active_database_mut(storage, active)?;
            let range = range.into_storage_range()?;
            let keys = database
                .tx_get_all(&active.transaction, &store)
                .map_err(storage_error)?
                .into_iter()
                .filter(|record| range.contains(&record.key))
                .map(|record| record.key)
                .collect::<Vec<_>>();
            for key in &keys {
                database
                    .tx_delete(&active.transaction, &store, key)
                    .map_err(storage_error)?;
            }
            if !keys.is_empty() {
                active.mutation_generation += 1;
            }
            Ok(json!({"deleted": keys.len()}))
        }
        IndexedDbRequest::TransactionClear { transaction, store } => {
            let active = active_transaction_mut(transactions, origin, transaction)?;
            require_write_transaction(active)?;
            let database = active_database_mut(storage, active)?;
            database.tx_clear(&active.transaction, &store).map_err(storage_error)?;
            active.mutation_generation += 1;
            Ok(json!({"cleared": true}))
        }
        IndexedDbRequest::TransactionCount {
            transaction,
            store,
            query,
        } => {
            let active = active_transaction_mut(transactions, origin, transaction)?;
            let database = active_database_mut(storage, active)?;
            let query = query.map(IndexedDbQueryWire::into_storage_query).transpose()?;
            let count = database
                .tx_get_all(&active.transaction, &store)
                .map_err(storage_error)?
                .iter()
                .filter(|record| query.as_ref().is_none_or(|query| query.matches(&record.key)))
                .count();
            Ok(json!({"count": count}))
        }
        IndexedDbRequest::TransactionGetAll {
            transaction,
            store,
            query,
            count,
            keys_only,
        } => {
            let active = active_transaction_mut(transactions, origin, transaction)?;
            let database = active_database_mut(storage, active)?;
            let query = query.map(IndexedDbQueryWire::into_storage_query).transpose()?;
            let records = database
                .tx_get_all(&active.transaction, &store)
                .map_err(storage_error)?
                .into_iter()
                .filter(|record| query.as_ref().is_none_or(|query| query.matches(&record.key)))
                .take(count.unwrap_or(usize::MAX))
                .map(|record| {
                    if keys_only {
                        json!({"key": IndexedDbKeyWire::from(&record.key)})
                    } else {
                        json!({
                            "key": IndexedDbKeyWire::from(&record.key),
                            "value": record.value,
                        })
                    }
                })
                .collect::<Vec<_>>();
            Ok(json!({"records": records}))
        }
        IndexedDbRequest::TransactionIndexGetAll {
            transaction,
            store,
            index,
            query,
            count,
        } => {
            let active = active_transaction_mut(transactions, origin, transaction)?;
            let database = active_database_mut(storage, active)?;
            let query = query.map(IndexedDbQueryWire::into_storage_query).transpose()?;
            let entries = database
                .tx_get_all_from_index(&active.transaction, &store, &index)
                .map_err(storage_error)?
                .into_iter()
                .filter(|entry| query.as_ref().is_none_or(|query| query.matches(&entry.index_key)))
                .take(count.unwrap_or(usize::MAX))
                .map(|entry| {
                    json!({
                        "key": IndexedDbKeyWire::from(&entry.index_key),
                        "primaryKey": IndexedDbKeyWire::from(&entry.primary_key),
                        "value": entry.value,
                    })
                })
                .collect::<Vec<_>>();
            Ok(json!({"entries": entries}))
        }
        IndexedDbRequest::TransactionOpenCursor {
            transaction,
            store,
            index,
            query,
            direction,
            key_only,
        } => open_transaction_cursor(
            storage,
            transactions,
            origin,
            transaction,
            &store,
            index.as_deref(),
            query,
            direction,
            key_only,
        ),
        IndexedDbRequest::TransactionCursorContinue {
            transaction,
            cursor,
            key,
        } => step_transaction_cursor(
            storage,
            transactions,
            origin,
            transaction,
            cursor,
            CursorStep::Continue(key.map(IndexedDbKeyWire::into_storage_key).transpose()?),
        ),
        IndexedDbRequest::TransactionCursorContinuePrimaryKey {
            transaction,
            cursor,
            key,
            primary_key,
        } => step_transaction_cursor(
            storage,
            transactions,
            origin,
            transaction,
            cursor,
            CursorStep::ContinuePrimaryKey(key.into_storage_key()?, primary_key.into_storage_key()?),
        ),
        IndexedDbRequest::TransactionCursorAdvance {
            transaction,
            cursor,
            count,
        } => {
            if count == 0 {
                return Err("TypeError: cursor advance count must be greater than zero".to_string());
            }
            step_transaction_cursor(
                storage,
                transactions,
                origin,
                transaction,
                cursor,
                CursorStep::Advance(count as usize),
            )
        }
        IndexedDbRequest::CommitTransaction { transaction } => {
            let mut active = remove_active_transaction(transactions, origin, transaction)?;
            storage
                .commit_indexed_db_transaction(&active.origin, &active.database, &mut active.transaction)
                .map_err(storage_error)?;
            Ok(json!({"committed": true}))
        }
        IndexedDbRequest::AbortTransaction { transaction } => {
            let mut active = remove_active_transaction(transactions, origin, transaction)?;
            active.transaction.abort().map_err(storage_error)?;
            Ok(json!({"aborted": true}))
        }
    }
}

fn abort_database_transactions(transactions: &mut IndexedDbTransactionRegistry, origin: &str, database: &str) {
    let ids = transactions
        .active
        .iter()
        .filter(|(_, active)| active.origin == origin && active.database == database)
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();
    for id in ids {
        if let Some(mut active) = transactions.active.remove(&id) {
            let _ = active.transaction.abort();
        }
    }
}

impl IndexedDbQueryWire {
    fn into_storage_query(self) -> Result<IndexedDbQuery, String> {
        match self {
            Self::Key(key) => Ok(IndexedDbQuery::Key(key.into_storage_key()?)),
            Self::Range(range) => Ok(IndexedDbQuery::Range(range.into_storage_range()?)),
        }
    }
}

impl IndexedDbKeyRangeWire {
    fn into_storage_range(self) -> Result<IdbKeyRange, String> {
        let lower = self.lower.map(IndexedDbKeyWire::into_storage_key).transpose()?;
        let upper = self.upper.map(IndexedDbKeyWire::into_storage_key).transpose()?;
        match (lower, upper) {
            (Some(lower), Some(upper)) => Ok(IdbKeyRange::bound(lower, upper, self.lower_open, self.upper_open)),
            (Some(lower), None) => Ok(IdbKeyRange::lower_bound(lower, self.lower_open)),
            (None, Some(upper)) => Ok(IdbKeyRange::upper_bound(upper, self.upper_open)),
            (None, None) => Err("DataError: IndexedDB key range has no bounds".to_string()),
        }
    }
}

impl IndexedDbQuery {
    fn matches(&self, key: &IdbKey) -> bool {
        match self {
            Self::Key(query) => query == key,
            Self::Range(range) => range.contains(key),
        }
    }
}

fn active_transaction_mut<'a>(
    transactions: &'a mut IndexedDbTransactionRegistry,
    origin: &str,
    transaction: u64,
) -> Result<&'a mut ActiveIndexedDbTransaction, String> {
    let active = transactions
        .active
        .get_mut(&transaction)
        .ok_or_else(|| "TransactionInactiveError: IndexedDB transaction does not exist".to_string())?;
    if active.origin != origin {
        return Err("SecurityError: IndexedDB transaction belongs to another origin".to_string());
    }
    Ok(active)
}

fn remove_active_transaction(
    transactions: &mut IndexedDbTransactionRegistry,
    origin: &str,
    transaction: u64,
) -> Result<ActiveIndexedDbTransaction, String> {
    active_transaction_mut(transactions, origin, transaction)?;
    transactions
        .active
        .remove(&transaction)
        .ok_or_else(|| "TransactionInactiveError: IndexedDB transaction does not exist".to_string())
}

fn require_write_transaction(active: &ActiveIndexedDbTransaction) -> Result<(), String> {
    if active.transaction.mode() == IdbTransactionMode::ReadOnly {
        return Err("ReadOnlyError: IndexedDB transaction is read-only".to_string());
    }
    Ok(())
}

fn active_database_mut<'a>(
    storage: &'a mut StorageManager,
    active: &ActiveIndexedDbTransaction,
) -> Result<&'a mut zero_storage::IdbDatabase, String> {
    storage
        .indexed_db_mut(&active.origin, &active.database)
        .ok_or_else(|| "NotFoundError: IndexedDB database does not exist".to_string())
}

fn sync_schema(
    storage: &mut StorageManager,
    origin: &str,
    name: &str,
    version: u64,
    stores: Vec<IndexedDbStoreSchema>,
) -> Result<Value, String> {
    if version == 0 {
        return Err("TypeError: IndexedDB version must be greater than zero".to_string());
    }
    let requested_names = stores.iter().map(|store| store.name.clone()).collect::<HashSet<_>>();
    if requested_names.len() != stores.len() {
        return Err("ConstraintError: duplicate object store name in schema".to_string());
    }
    for store in &stores {
        let index_names = store
            .indexes
            .iter()
            .map(|index| index.name.as_str())
            .collect::<HashSet<_>>();
        if index_names.len() != store.indexes.len() {
            return Err(format!(
                "ConstraintError: duplicate index name in object store '{}'",
                store.name
            ));
        }
        if store
            .indexes
            .iter()
            .any(|index| index.multi_entry && matches!(&index.key_path, IndexedDbIndexKeyPath::Sequence(_)))
        {
            return Err(format!(
                "InvalidAccessError: multiEntry index in object store '{}' cannot use a compound key path",
                store.name
            ));
        }
    }

    let mut replaced_names = HashSet::new();
    if let Some(database) = storage.indexed_db(origin, name) {
        if version < database.version {
            return Err(format!(
                "VersionError: requested version {version} is lower than current version {}",
                database.version
            ));
        }
        let current_stores = database.store_info();
        for current in &current_stores {
            if let Some(requested) = stores.iter().find(|store| store.name == current.name)
                && (!optional_key_paths_match(&current.key_path, &requested.key_path)
                    || requested.auto_increment != current.auto_increment)
            {
                replaced_names.insert(current.name.clone());
            }
        }
        let schema_changed = current_stores.len() != stores.len()
            || current_stores
                .iter()
                .any(|current| !requested_names.contains(current.name.as_str()))
            || current_stores.iter().any(|current| {
                stores
                    .iter()
                    .find(|store| store.name == current.name)
                    .is_some_and(|requested| !index_schemas_match(&current.indexes, &requested.indexes))
            })
            || !replaced_names.is_empty();
        if schema_changed && version == database.version {
            return Err("InvalidStateError: object store schema changes require a version upgrade".to_string());
        }
        for requested_store in &stores {
            if replaced_names.contains(&requested_store.name) {
                continue;
            }
            let Some(current_store) = current_stores
                .iter()
                .find(|current| current.name == requested_store.name)
            else {
                continue;
            };
            for requested_index in &requested_store.indexes {
                let unchanged = current_store.indexes.iter().any(|current| {
                    current.name == requested_index.name
                        && index_key_paths_match(&current.key_path, &requested_index.key_path)
                        && current.unique == requested_index.unique
                        && current.multi_entry == requested_index.multi_entry
                });
                if !unchanged {
                    database
                        .validate_index_with_key_path(
                            &requested_store.name,
                            &requested_index.name,
                            storage_index_key_path(&requested_index.key_path),
                            requested_index.unique,
                            requested_index.multi_entry,
                        )
                        .map_err(storage_error)?;
                }
            }
        }
    }

    let mut database = storage
        .indexed_db(origin, name)
        .cloned()
        .unwrap_or_else(|| zero_storage::IdbDatabase::new(name, version));
    database.version = version;
    let existing_names = database
        .store_names()
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    for existing in existing_names {
        if !requested_names.contains(existing.as_str()) || replaced_names.contains(&existing) {
            database.delete_object_store(&existing).map_err(storage_error)?;
        }
    }
    for requested in &stores {
        if !database.has_store(&requested.name) {
            database
                .create_object_store_with_key_path(
                    &requested.name,
                    requested.key_path.as_ref().map(storage_index_key_path),
                    requested.auto_increment,
                )
                .map_err(storage_error)?;
        }
    }
    for requested_store in &stores {
        let current_store = database
            .store_info()
            .into_iter()
            .find(|store| store.name == requested_store.name)
            .ok_or_else(|| "NotFoundError: IndexedDB object store does not exist".to_string())?;
        for current_index in &current_store.indexes {
            let unchanged = requested_store.indexes.iter().any(|requested| {
                requested.name == current_index.name
                    && index_key_paths_match(&current_index.key_path, &requested.key_path)
                    && requested.unique == current_index.unique
                    && requested.multi_entry == current_index.multi_entry
            });
            if !unchanged {
                database
                    .delete_index(&requested_store.name, &current_index.name)
                    .map_err(storage_error)?;
            }
        }
        let current_names = database
            .index_names(&requested_store.name)
            .map_err(storage_error)?
            .into_iter()
            .map(str::to_string)
            .collect::<HashSet<_>>();
        for requested_index in &requested_store.indexes {
            if !current_names.contains(&requested_index.name) {
                database
                    .create_index_with_key_path(
                        &requested_store.name,
                        &requested_index.name,
                        storage_index_key_path(&requested_index.key_path),
                        requested_index.unique,
                        requested_index.multi_entry,
                    )
                    .map_err(storage_error)?;
            }
        }
    }
    let schema = database_schema_json(&database);
    storage.replace_indexed_db(origin, database).map_err(storage_error)?;
    Ok(json!({"database": schema}))
}

fn index_schemas_match(current: &[zero_storage::IdbIndexInfo], requested: &[IndexedDbIndexSchema]) -> bool {
    current.len() == requested.len()
        && current.iter().all(|current| {
            requested.iter().any(|requested| {
                requested.name == current.name
                    && index_key_paths_match(&current.key_path, &requested.key_path)
                    && requested.unique == current.unique
                    && requested.multi_entry == current.multi_entry
            })
        })
}

fn storage_index_key_path(key_path: &IndexedDbIndexKeyPath) -> zero_storage::IdbIndexKeyPath {
    match key_path {
        IndexedDbIndexKeyPath::String(value) => zero_storage::IdbIndexKeyPath::String(value.clone()),
        IndexedDbIndexKeyPath::Sequence(values) => zero_storage::IdbIndexKeyPath::Sequence(values.clone()),
    }
}

fn index_key_paths_match(current: &zero_storage::IdbIndexKeyPath, requested: &IndexedDbIndexKeyPath) -> bool {
    match (current, requested) {
        (zero_storage::IdbIndexKeyPath::String(current), IndexedDbIndexKeyPath::String(requested)) => {
            current == requested
        }
        (zero_storage::IdbIndexKeyPath::Sequence(current), IndexedDbIndexKeyPath::Sequence(requested)) => {
            current == requested
        }
        _ => false,
    }
}

fn optional_key_paths_match(
    current: &Option<zero_storage::IdbIndexKeyPath>,
    requested: &Option<IndexedDbIndexKeyPath>,
) -> bool {
    match (current, requested) {
        (None, None) => true,
        (Some(current), Some(requested)) => index_key_paths_match(current, requested),
        _ => false,
    }
}

fn database_schema_json(database: &zero_storage::indexed_db::IdbDatabase) -> Value {
    let stores = database
        .store_info()
        .into_iter()
        .map(|store| {
            let indexes = store
                .indexes
                .into_iter()
                .map(|index| {
                    let key_path = match index.key_path {
                        zero_storage::IdbIndexKeyPath::String(value) => json!(value),
                        zero_storage::IdbIndexKeyPath::Sequence(values) => json!(values),
                    };
                    json!({
                        "name": index.name,
                        "keyPath": key_path,
                        "unique": index.unique,
                        "multiEntry": index.multi_entry,
                    })
                })
                .collect::<Vec<_>>();
            let key_path = match store.key_path {
                Some(zero_storage::IdbIndexKeyPath::String(value)) => json!(value),
                Some(zero_storage::IdbIndexKeyPath::Sequence(values)) => json!(values),
                None => Value::Null,
            };
            json!({
                "name": store.name,
                "keyPath": key_path,
                "autoIncrement": store.auto_increment,
                "indexes": indexes,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "name": database.name,
        "version": database.version,
        "stores": stores,
    })
}

fn storage_error(error: StorageError) -> String {
    match error {
        StorageError::QuotaExceeded(message) => format!("QuotaExceededError: {message}"),
        StorageError::InvalidKey(message) => format!("DataError: {message}"),
        StorageError::StoreNotFound(store) => format!("NotFoundError: object store '{store}' does not exist"),
        StorageError::KeyNotFound(key) => format!("NotFoundError: key '{key}' does not exist"),
        StorageError::Serialization(message) => format!("DataCloneError: {message}"),
        StorageError::Io(message) => format!("UnknownError: IndexedDB persistence failed: {message}"),
        StorageError::Database(message)
            if message.contains("Key already exists") || message.contains("Unique index") =>
        {
            format!("ConstraintError: {message}")
        }
        StorageError::Database(message) if message.contains("Transaction") => {
            format!("TransactionInactiveError: {message}")
        }
        StorageError::Database(message) => format!("UnknownError: {message}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_wire_preserves_date_type_and_non_json_numbers() {
        let keys = [
            IdbKey::Number(f64::INFINITY),
            IdbKey::Number(-0.0),
            IdbKey::Date(1_700_000_000_000.0),
            IdbKey::Array(vec![IdbKey::Date(0.0), IdbKey::String("x".to_string())]),
        ];
        for key in keys {
            let wire = IndexedDbKeyWire::from(&key);
            let json = serde_json::to_string(&wire).unwrap();
            let decoded: IndexedDbKeyWire = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded.into_storage_key().unwrap(), key);
        }

        assert!(
            IndexedDbKeyWire::Date("Infinity".to_string())
                .into_storage_key()
                .unwrap_err()
                .starts_with("DataError:")
        );
        assert!(
            IndexedDbKeyWire::Number("NaN".to_string())
                .into_storage_key()
                .unwrap_err()
                .starts_with("DataError:")
        );
    }

    fn call(handler: &IndexedDbHandler, origin: &str, request: Value) -> Result<Value, String> {
        handler(origin, &request.to_string()).and_then(|response| {
            serde_json::from_str(&response).map_err(|error| format!("invalid test response: {error}"))
        })
    }

    #[test]
    fn open_schema_and_reopen_share_origin_state() {
        let handler = indexed_db_handler(Arc::new(Mutex::new(StorageManager::new())));
        let opened = call(
            &handler,
            "https://app.example",
            json!({"op": "open", "name": "app", "version": 1}),
        )
        .unwrap();
        assert_eq!(opened["oldVersion"], 0);
        assert_eq!(opened["upgradeNeeded"], true);

        call(
            &handler,
            "https://app.example",
            json!({
                "op": "create_object_store",
                "database": "app",
                "name": "items",
                "key_path": "id",
                "auto_increment": true
            }),
        )
        .unwrap();
        let reopened = call(
            &handler,
            "https://app.example",
            json!({"op": "open", "name": "app", "version": 1}),
        )
        .unwrap();
        assert_eq!(reopened["oldVersion"], 1);
        assert_eq!(reopened["upgradeNeeded"], false);
        assert_eq!(reopened["stores"], json!(["items"]));
    }

    #[test]
    fn sync_schema_commits_only_the_supplied_snapshot() {
        let handler = indexed_db_handler(Arc::new(Mutex::new(StorageManager::new())));
        let empty = call(&handler, "https://app.example", json!({"op": "inspect", "name": "app"})).unwrap();
        assert!(empty["database"].is_null());

        call(
            &handler,
            "https://app.example",
            json!({
                "op": "sync_schema",
                "name": "app",
                "version": 1,
                "stores": [
                    {"name": "items", "keyPath": "id", "autoIncrement": true},
                    {"name": "logs", "keyPath": null, "autoIncrement": false}
                ]
            }),
        )
        .unwrap();
        let inspected = call(&handler, "https://app.example", json!({"op": "inspect", "name": "app"})).unwrap();
        assert_eq!(inspected["database"]["version"], 1);
        assert_eq!(
            inspected["database"]["stores"],
            json!([
                {"name": "items", "keyPath": "id", "autoIncrement": true, "indexes": []},
                {"name": "logs", "keyPath": null, "autoIncrement": false, "indexes": []}
            ])
        );

        call(
            &handler,
            "https://app.example",
            json!({
                "op": "sync_schema",
                "name": "app",
                "version": 2,
                "stores": [{"name": "items", "keyPath": "id", "autoIncrement": true}]
            }),
        )
        .unwrap();
        let upgraded = call(&handler, "https://app.example", json!({"op": "inspect", "name": "app"})).unwrap();
        assert_eq!(upgraded["database"]["version"], 2);
        assert_eq!(upgraded["database"]["stores"].as_array().unwrap().len(), 1);

        let same_version_change = call(
            &handler,
            "https://app.example",
            json!({
                "op": "sync_schema",
                "name": "app",
                "version": 2,
                "stores": [
                    {"name": "items", "keyPath": "id", "autoIncrement": true},
                    {"name": "extra", "keyPath": null, "autoIncrement": false}
                ]
            }),
        )
        .unwrap_err();
        assert!(same_version_change.starts_with("InvalidStateError:"));

        let replaced = call(
            &handler,
            "https://app.example",
            json!({
                "op": "sync_schema",
                "name": "app",
                "version": 3,
                "stores": [{"name": "items", "keyPath": "slug", "autoIncrement": false}]
            }),
        )
        .unwrap();
        assert_eq!(replaced["database"]["version"], 3);
        assert_eq!(replaced["database"]["stores"][0]["keyPath"], "slug");
    }

    #[test]
    fn transaction_wire_commits_aborts_and_isolates_origins() {
        let handler = indexed_db_handler(Arc::new(Mutex::new(StorageManager::new())));
        call(
            &handler,
            "https://app.example",
            json!({
                "op": "sync_schema",
                "name": "app",
                "version": 1,
                "stores": [{"name": "items", "keyPath": null, "autoIncrement": true}]
            }),
        )
        .unwrap();

        let write = call(
            &handler,
            "https://app.example",
            json!({
                "op": "begin_transaction",
                "database": "app",
                "stores": ["items"],
                "mode": "readwrite"
            }),
        )
        .unwrap()["transaction"]
            .as_u64()
            .unwrap();
        let date_key = json!({"type": "date", "value": "10"});
        call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_add",
                "transaction": write,
                "store": "items",
                "value": {"kind": "date"},
                "key": date_key.clone()
            }),
        )
        .unwrap();
        call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_put",
                "transaction": write,
                "store": "items",
                "value": {"kind": "updated"},
                "key": date_key.clone()
            }),
        )
        .unwrap();
        let buffered = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_get",
                "transaction": write,
                "store": "items",
                "key": date_key.clone()
            }),
        )
        .unwrap();
        assert_eq!(buffered["record"]["value"]["kind"], "updated");
        call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_delete",
                "transaction": write,
                "store": "items",
                "key": date_key.clone()
            }),
        )
        .unwrap();
        let deleted = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_get",
                "transaction": write,
                "store": "items",
                "key": date_key.clone()
            }),
        )
        .unwrap();
        assert!(deleted["record"].is_null());
        call(
            &handler,
            "https://app.example",
            json!({"op": "abort_transaction", "transaction": write}),
        )
        .unwrap();

        let committed_write = call(
            &handler,
            "https://app.example",
            json!({
                "op": "begin_transaction",
                "database": "app",
                "stores": ["items"],
                "mode": "readwrite"
            }),
        )
        .unwrap()["transaction"]
            .as_u64()
            .unwrap();
        let generated = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_add",
                "transaction": committed_write,
                "store": "items",
                "value": {"kind": "generated"}
            }),
        )
        .unwrap();
        assert_eq!(generated["key"], json!({"type": "number", "value": "1"}));
        call(
            &handler,
            "https://app.example",
            json!({"op": "commit_transaction", "transaction": committed_write}),
        )
        .unwrap();

        let read = call(
            &handler,
            "https://app.example",
            json!({
                "op": "begin_transaction",
                "database": "app",
                "stores": ["items"],
                "mode": "readonly"
            }),
        )
        .unwrap()["transaction"]
            .as_u64()
            .unwrap();
        let aborted_record = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_get",
                "transaction": read,
                "store": "items",
                "key": date_key
            }),
        )
        .unwrap();
        assert!(aborted_record["record"].is_null());
        let committed_record = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_get",
                "transaction": read,
                "store": "items",
                "key": {"type": "number", "value": "1"}
            }),
        )
        .unwrap();
        assert_eq!(committed_record["record"]["value"]["kind"], "generated");
        assert!(
            call(
                &handler,
                "https://other.example",
                json!({
                    "op": "transaction_get",
                    "transaction": read,
                    "store": "items",
                    "key": {"type": "number", "value": "1"}
                }),
            )
            .unwrap_err()
            .starts_with("SecurityError:")
        );
        assert!(
            call(
                &handler,
                "https://app.example",
                json!({
                    "op": "transaction_put",
                    "transaction": read,
                    "store": "items",
                    "value": {},
                    "key": {"type": "number", "value": "1"}
                }),
            )
            .unwrap_err()
            .starts_with("ReadOnlyError:")
        );
        call(
            &handler,
            "https://app.example",
            json!({"op": "commit_transaction", "transaction": read}),
        )
        .unwrap();
    }

    #[test]
    fn transaction_query_and_clear_use_buffered_view() {
        let handler = indexed_db_handler(Arc::new(Mutex::new(StorageManager::new())));
        call(
            &handler,
            "https://app.example",
            json!({
                "op": "sync_schema",
                "name": "app",
                "version": 1,
                "stores": [{"name": "items", "keyPath": null, "autoIncrement": false}]
            }),
        )
        .unwrap();
        let write = call(
            &handler,
            "https://app.example",
            json!({
                "op": "begin_transaction",
                "database": "app",
                "stores": ["items"],
                "mode": "readwrite"
            }),
        )
        .unwrap()["transaction"]
            .as_u64()
            .unwrap();
        for key in 1..=3 {
            call(
                &handler,
                "https://app.example",
                json!({
                    "op": "transaction_add",
                    "transaction": write,
                    "store": "items",
                    "value": {"value": key},
                    "key": {"type": "number", "value": key.to_string()}
                }),
            )
            .unwrap();
        }
        let range = json!({
            "lower": {"type": "number", "value": "2"},
            "upper": {"type": "number", "value": "3"},
            "lowerOpen": false,
            "upperOpen": false
        });
        let count = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_count",
                "transaction": write,
                "store": "items",
                "query": {"type": "range", "value": range.clone()}
            }),
        )
        .unwrap();
        assert_eq!(count["count"], 2);
        let records = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_get_all",
                "transaction": write,
                "store": "items",
                "query": {"type": "range", "value": range.clone()},
                "count": 1,
                "keys_only": false
            }),
        )
        .unwrap();
        assert_eq!(records["records"].as_array().unwrap().len(), 1);
        assert_eq!(records["records"][0]["value"]["value"], 2);
        call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_delete_range",
                "transaction": write,
                "store": "items",
                "range": range
            }),
        )
        .unwrap();
        let after_delete = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_count",
                "transaction": write,
                "store": "items"
            }),
        )
        .unwrap();
        assert_eq!(after_delete["count"], 1);
        call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_clear",
                "transaction": write,
                "store": "items"
            }),
        )
        .unwrap();
        let after_clear = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_count",
                "transaction": write,
                "store": "items"
            }),
        )
        .unwrap();
        assert_eq!(after_clear["count"], 0);
        call(
            &handler,
            "https://app.example",
            json!({"op": "abort_transaction", "transaction": write}),
        )
        .unwrap();

        let seed = call(
            &handler,
            "https://app.example",
            json!({
                "op": "begin_transaction",
                "database": "app",
                "stores": ["items"],
                "mode": "readwrite"
            }),
        )
        .unwrap()["transaction"]
            .as_u64()
            .unwrap();
        call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_add",
                "transaction": seed,
                "store": "items",
                "value": {"value": "kept"},
                "key": {"type": "string", "value": "key"}
            }),
        )
        .unwrap();
        call(
            &handler,
            "https://app.example",
            json!({"op": "commit_transaction", "transaction": seed}),
        )
        .unwrap();
        let clear = call(
            &handler,
            "https://app.example",
            json!({
                "op": "begin_transaction",
                "database": "app",
                "stores": ["items"],
                "mode": "readwrite"
            }),
        )
        .unwrap()["transaction"]
            .as_u64()
            .unwrap();
        call(
            &handler,
            "https://app.example",
            json!({"op": "transaction_clear", "transaction": clear, "store": "items"}),
        )
        .unwrap();
        call(
            &handler,
            "https://app.example",
            json!({"op": "commit_transaction", "transaction": clear}),
        )
        .unwrap();
        let read = call(
            &handler,
            "https://app.example",
            json!({
                "op": "begin_transaction",
                "database": "app",
                "stores": ["items"],
                "mode": "readonly"
            }),
        )
        .unwrap()["transaction"]
            .as_u64()
            .unwrap();
        let final_count = call(
            &handler,
            "https://app.example",
            json!({"op": "transaction_count", "transaction": read, "store": "items"}),
        )
        .unwrap();
        assert_eq!(final_count["count"], 0);
        call(
            &handler,
            "https://app.example",
            json!({"op": "commit_transaction", "transaction": read}),
        )
        .unwrap();
    }

    #[test]
    fn transaction_index_view_uses_buffered_records_and_typed_keys() {
        let handler = indexed_db_handler(Arc::new(Mutex::new(StorageManager::new())));
        call(
            &handler,
            "https://app.example",
            json!({
                "op": "sync_schema",
                "name": "app",
                "version": 1,
                "stores": [{
                    "name": "items",
                    "keyPath": null,
                    "autoIncrement": false,
                    "indexes": [
                        {"name": "by_group", "keyPath": "group"},
                        {"name": "by_tags", "keyPath": "tags", "multiEntry": true},
                        {"name": "by_when", "keyPath": "when"},
                        {"name": "by_compound", "keyPath": ["first", "last"]}
                    ]
                }]
            }),
        )
        .unwrap();
        let write = call(
            &handler,
            "https://app.example",
            json!({
                "op": "begin_transaction",
                "database": "app",
                "stores": ["items"],
                "mode": "readwrite"
            }),
        )
        .unwrap()["transaction"]
            .as_u64()
            .unwrap();
        for (key, value) in [
            (
                "1",
                json!({
                    "group": "b",
                    "first": "Ada",
                    "last": "Lovelace",
                    "tags": ["x", "y"],
                    "when": {"__zwIdbType": "date", "value": "20"}
                }),
            ),
            (
                "2",
                json!({
                    "group": "a",
                    "first": "Grace",
                    "last": "Hopper",
                    "tags": ["y"],
                    "when": {"__zwIdbType": "date", "value": "10"}
                }),
            ),
        ] {
            call(
                &handler,
                "https://app.example",
                json!({
                    "op": "transaction_add",
                    "transaction": write,
                    "store": "items",
                    "value": value,
                    "key": {"type": "number", "value": key}
                }),
            )
            .unwrap();
        }
        let groups = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_index_get_all",
                "transaction": write,
                "store": "items",
                "index": "by_group"
            }),
        )
        .unwrap();
        assert_eq!(groups["entries"][0]["key"], json!({"type": "string", "value": "a"}));
        assert_eq!(
            groups["entries"][0]["primaryKey"],
            json!({"type": "number", "value": "2"})
        );
        assert_eq!(groups["entries"][1]["key"], json!({"type": "string", "value": "b"}));

        let tags = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_index_get_all",
                "transaction": write,
                "store": "items",
                "index": "by_tags",
                "query": {"type": "key", "value": {"type": "string", "value": "y"}}
            }),
        )
        .unwrap();
        assert_eq!(tags["entries"].as_array().unwrap().len(), 2);

        let dates = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_index_get_all",
                "transaction": write,
                "store": "items",
                "index": "by_when"
            }),
        )
        .unwrap();
        assert_eq!(dates["entries"][0]["key"], json!({"type": "date", "value": "10"}));
        assert_eq!(dates["entries"][1]["key"], json!({"type": "date", "value": "20"}));
        let compounds = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_index_get_all",
                "transaction": write,
                "store": "items",
                "index": "by_compound"
            }),
        )
        .unwrap();
        assert_eq!(
            compounds["entries"][0]["key"],
            json!({
                "type": "array",
                "value": [
                    {"type": "string", "value": "Ada"},
                    {"type": "string", "value": "Lovelace"}
                ]
            })
        );
        call(
            &handler,
            "https://app.example",
            json!({"op": "commit_transaction", "transaction": write}),
        )
        .unwrap();
    }

    #[test]
    fn transaction_cursors_own_and_step_buffered_entries() {
        let handler = indexed_db_handler(Arc::new(Mutex::new(StorageManager::new())));
        call(
            &handler,
            "https://app.example",
            json!({
                "op": "sync_schema",
                "name": "app",
                "version": 1,
                "stores": [{
                    "name": "items",
                    "keyPath": null,
                    "autoIncrement": false,
                    "indexes": [{"name": "by_group", "keyPath": "group"}]
                }]
            }),
        )
        .unwrap();
        let transaction = call(
            &handler,
            "https://app.example",
            json!({
                "op": "begin_transaction",
                "database": "app",
                "stores": ["items"],
                "mode": "readwrite"
            }),
        )
        .unwrap()["transaction"]
            .as_u64()
            .unwrap();
        for (key, group) in [(1, "a"), (2, "b"), (3, "b"), (4, "c")] {
            call(
                &handler,
                "https://app.example",
                json!({
                    "op": "transaction_add",
                    "transaction": transaction,
                    "store": "items",
                    "value": {"group": group, "key": key},
                    "key": {"type": "number", "value": key.to_string()}
                }),
            )
            .unwrap();
        }

        let opened = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_open_cursor",
                "transaction": transaction,
                "store": "items",
                "index": "by_group",
                "direction": "next",
                "key_only": false
            }),
        )
        .unwrap();
        let cursor = opened["cursor"].as_u64().unwrap();
        assert_eq!(opened["entry"]["key"], json!({"type": "string", "value": "a"}));
        assert_eq!(opened["entry"]["value"]["key"], 1);

        let continued = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_cursor_continue",
                "transaction": transaction,
                "cursor": cursor,
                "key": {"type": "string", "value": "b"}
            }),
        )
        .unwrap();
        assert_eq!(
            continued["entry"]["primaryKey"],
            json!({"type": "number", "value": "2"})
        );
        let advanced = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_cursor_advance",
                "transaction": transaction,
                "cursor": cursor,
                "count": 2
            }),
        )
        .unwrap();
        assert_eq!(advanced["entry"]["key"], json!({"type": "string", "value": "c"}));

        let unique = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_open_cursor",
                "transaction": transaction,
                "store": "items",
                "index": "by_group",
                "direction": "prevunique",
                "key_only": true
            }),
        )
        .unwrap();
        assert_eq!(unique["entry"]["key"], json!({"type": "string", "value": "c"}));
        assert!(unique["entry"].get("value").is_none());
        let unique_cursor = unique["cursor"].as_u64().unwrap();
        let unique_next = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_cursor_continue",
                "transaction": transaction,
                "cursor": unique_cursor
            }),
        )
        .unwrap();
        assert_eq!(unique_next["entry"]["key"], json!({"type": "string", "value": "b"}));
        assert_eq!(
            unique_next["entry"]["primaryKey"],
            json!({"type": "number", "value": "2"})
        );

        assert!(
            call(
                &handler,
                "https://app.example",
                json!({
                    "op": "transaction_cursor_continue",
                    "transaction": transaction,
                    "cursor": cursor,
                    "key": {"type": "string", "value": "c"}
                }),
            )
            .unwrap_err()
            .starts_with("DataError:")
        );
        call(
            &handler,
            "https://app.example",
            json!({"op": "commit_transaction", "transaction": transaction}),
        )
        .unwrap();
        assert!(
            call(
                &handler,
                "https://app.example",
                json!({
                    "op": "transaction_cursor_continue",
                    "transaction": transaction,
                    "cursor": cursor
                }),
            )
            .unwrap_err()
            .starts_with("TransactionInactiveError:")
        );
    }

    #[test]
    fn databases_are_origin_scoped_and_deletable() {
        let handler = indexed_db_handler(Arc::new(Mutex::new(StorageManager::new())));
        call(
            &handler,
            "https://a.example",
            json!({"op": "open", "name": "beta", "version": 2}),
        )
        .unwrap();
        call(
            &handler,
            "https://a.example",
            json!({"op": "open", "name": "alpha", "version": 1}),
        )
        .unwrap();
        call(
            &handler,
            "https://b.example",
            json!({"op": "open", "name": "alpha", "version": 4}),
        )
        .unwrap();

        let databases = call(&handler, "https://a.example", json!({"op": "databases"})).unwrap();
        assert_eq!(
            databases["databases"],
            json!([{"name": "alpha", "version": 1}, {"name": "beta", "version": 2}])
        );
        let deleted = call(
            &handler,
            "https://a.example",
            json!({"op": "delete_database", "name": "alpha"}),
        )
        .unwrap();
        assert_eq!(deleted, json!({"deleted": true, "oldVersion": 1}));

        let isolated = call(&handler, "https://b.example", json!({"op": "databases"})).unwrap();
        assert_eq!(isolated["databases"], json!([{"name": "alpha", "version": 4}]));
    }

    #[test]
    fn invalid_operations_return_named_errors() {
        let handler = indexed_db_handler(Arc::new(Mutex::new(StorageManager::new())));
        assert!(
            handler("null", r#"{"op":"databases"}"#)
                .unwrap_err()
                .starts_with("SecurityError:")
        );
        assert!(
            handler("https://app.example", r#"{"op":"unknown"}"#)
                .unwrap_err()
                .starts_with("DataError:")
        );

        call(
            &handler,
            "https://app.example",
            json!({"op": "open", "name": "app", "version": 2}),
        )
        .unwrap();
        assert!(
            call(
                &handler,
                "https://app.example",
                json!({"op": "open", "name": "app", "version": 1})
            )
            .unwrap_err()
            .starts_with("VersionError:")
        );
        assert!(
            call(
                &handler,
                "https://app.example",
                json!({"op": "delete_object_store", "database": "app", "name": "missing"})
            )
            .unwrap_err()
            .starts_with("NotFoundError:")
        );
    }
}

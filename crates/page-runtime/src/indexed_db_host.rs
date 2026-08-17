//! IndexedDB 页面宿主。
//!
//! 本模块解析 `zero-engine` 同步 wire 请求，并在共享 [`StorageManager`] 上执行
//! per-origin 数据库与 schema 操作。

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use zero_engine::IndexedDbHandler;
use zero_storage::{IdbKey, StorageError, StorageManager};

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
    Inspect {
        name: String,
    },
    SyncSchema {
        name: String,
        version: u32,
        stores: Vec<IndexedDbStoreSchema>,
    },
    Open {
        name: String,
        version: u32,
    },
    DeleteDatabase {
        name: String,
    },
    Databases,
    CreateObjectStore {
        database: String,
        name: String,
        #[serde(default)]
        key_path: Option<String>,
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
}

#[derive(Debug, Deserialize)]
struct IndexedDbStoreSchema {
    name: String,
    #[serde(default, rename = "keyPath")]
    key_path: Option<String>,
    #[serde(default, rename = "autoIncrement")]
    auto_increment: bool,
}

/// 构造由页面运行路径共享的 IndexedDB handler。
pub fn indexed_db_handler(storage: Arc<Mutex<StorageManager>>) -> IndexedDbHandler {
    Arc::new(move |origin, request| handle_request(&storage, origin, request))
}

fn handle_request(storage: &Mutex<StorageManager>, origin: &str, request: &str) -> Result<String, String> {
    if origin == "null" {
        return Err("SecurityError: IndexedDB is unavailable for opaque origins".to_string());
    }

    let request: IndexedDbRequest =
        serde_json::from_str(request).map_err(|error| format!("DataError: invalid IndexedDB request: {error}"))?;
    let mut storage = storage
        .lock()
        .map_err(|_| "UnknownError: IndexedDB storage lock is poisoned".to_string())?;
    let response = dispatch_request(&mut storage, origin, request)?;
    serde_json::to_string(&response).map_err(|error| format!("UnknownError: failed to serialize response: {error}"))
}

fn dispatch_request(storage: &mut StorageManager, origin: &str, request: IndexedDbRequest) -> Result<Value, String> {
    match request {
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
            let old_version = storage
                .indexed_db(origin, &name)
                .map(|database| database.version)
                .unwrap_or(0);
            Ok(json!({
                "deleted": storage.delete_indexed_db(origin, &name),
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
            let database = storage
                .indexed_db_mut(origin, &database)
                .ok_or_else(|| "NotFoundError: IndexedDB database does not exist".to_string())?;
            if database.has_store(&name) {
                return Err(format!("ConstraintError: object store '{name}' already exists"));
            }
            database
                .create_object_store(&name, key_path.as_deref(), auto_increment)
                .map_err(storage_error)?;
            Ok(json!({"created": true}))
        }
        IndexedDbRequest::DeleteObjectStore { database, name } => {
            let database = storage
                .indexed_db_mut(origin, &database)
                .ok_or_else(|| "NotFoundError: IndexedDB database does not exist".to_string())?;
            if !database.has_store(&name) {
                return Err(format!("NotFoundError: object store '{name}' does not exist"));
            }
            database.delete_object_store(&name).map_err(storage_error)?;
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
    }
}

fn sync_schema(
    storage: &mut StorageManager,
    origin: &str,
    name: &str,
    version: u32,
    stores: Vec<IndexedDbStoreSchema>,
) -> Result<Value, String> {
    if version == 0 {
        return Err("TypeError: IndexedDB version must be greater than zero".to_string());
    }
    let requested_names = stores.iter().map(|store| store.name.clone()).collect::<HashSet<_>>();
    if requested_names.len() != stores.len() {
        return Err("ConstraintError: duplicate object store name in schema".to_string());
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
                && (requested.key_path != current.key_path || requested.auto_increment != current.auto_increment)
            {
                replaced_names.insert(current.name.clone());
            }
        }
        let schema_changed = current_stores.len() != stores.len()
            || current_stores
                .iter()
                .any(|current| !requested_names.contains(current.name.as_str()))
            || !replaced_names.is_empty();
        if schema_changed && version == database.version {
            return Err("InvalidStateError: object store schema changes require a version upgrade".to_string());
        }
    }

    let database = storage.open_indexed_db(origin, name, version).map_err(storage_error)?;
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
    for requested in stores {
        if !database.has_store(&requested.name) {
            database
                .create_object_store(&requested.name, requested.key_path.as_deref(), requested.auto_increment)
                .map_err(storage_error)?;
        }
    }
    Ok(json!({"database": database_schema_json(database)}))
}

fn database_schema_json(database: &zero_storage::indexed_db::IdbDatabase) -> Value {
    let stores = database
        .store_info()
        .into_iter()
        .map(|store| {
            json!({
                "name": store.name,
                "keyPath": store.key_path,
                "autoIncrement": store.auto_increment,
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
    format!("UnknownError: {error}")
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
                {"name": "items", "keyPath": "id", "autoIncrement": true},
                {"name": "logs", "keyPath": null, "autoIncrement": false}
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

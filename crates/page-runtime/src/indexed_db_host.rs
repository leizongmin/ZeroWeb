//! IndexedDB 页面宿主。
//!
//! 本模块解析 `zero-engine` 同步 wire 请求，并在共享 [`StorageManager`] 上执行
//! per-origin 数据库与 schema 操作。

use std::sync::{Arc, Mutex};

use serde::Deserialize;
use serde_json::{Value, json};
use zero_engine::IndexedDbHandler;
use zero_storage::{StorageError, StorageManager};

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum IndexedDbRequest {
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

fn storage_error(error: StorageError) -> String {
    format!("UnknownError: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;

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

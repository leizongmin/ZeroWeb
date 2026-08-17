use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;

static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("zeroweb-page-indexeddb-{}-{sequence}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        let _ = fs::remove_file(&path);
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
        let _ = fs::remove_file(&self.path);
    }
}

fn call(handler: &IndexedDbHandler, origin: &str, request: Value) -> Result<Value, String> {
    handler(origin, &request.to_string())
        .and_then(|response| serde_json::from_str(&response).map_err(|error| format!("invalid test response: {error}")))
}

fn persistent_handler(path: &Path) -> IndexedDbHandler {
    let storage = StorageManager::with_indexed_db_persistence(path).unwrap();
    indexed_db_handler(Arc::new(Mutex::new(storage)))
}

fn create_schema(handler: &IndexedDbHandler, origin: &str) {
    call(
        handler,
        origin,
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
}

fn begin_transaction(handler: &IndexedDbHandler, origin: &str, mode: &str) -> u64 {
    call(
        handler,
        origin,
        json!({
            "op": "begin_transaction",
            "database": "app",
            "stores": ["items"],
            "mode": mode
        }),
    )
    .unwrap()["transaction"]
        .as_u64()
        .unwrap()
}

#[test]
fn indexed_db_handler_reads_committed_data_after_manager_rebuild() {
    let directory = TestDirectory::new();
    let origin = "https://app.example";
    {
        let handler = persistent_handler(directory.path());
        create_schema(&handler, origin);
        let transaction = begin_transaction(&handler, origin, "readwrite");
        call(
            &handler,
            origin,
            json!({
                "op": "transaction_add",
                "transaction": transaction,
                "store": "items",
                "value": {"group": "stable", "payload": {"nested": [1, 2, 3]}},
                "key": {"type": "date", "value": "1700000000000"}
            }),
        )
        .unwrap();
        call(
            &handler,
            origin,
            json!({"op": "commit_transaction", "transaction": transaction}),
        )
        .unwrap();
    }

    let handler = persistent_handler(directory.path());
    let inspected = call(&handler, origin, json!({"op": "inspect", "name": "app"})).unwrap();
    assert_eq!(inspected["database"]["stores"][0]["indexes"][0]["name"], "by_group");
    let transaction = begin_transaction(&handler, origin, "readonly");
    let record = call(
        &handler,
        origin,
        json!({
            "op": "transaction_get",
            "transaction": transaction,
            "store": "items",
            "key": {"type": "date", "value": "1700000000000"}
        }),
    )
    .unwrap();
    assert_eq!(record["record"]["value"]["payload"]["nested"], json!([1, 2, 3]));
    let indexed = call(
        &handler,
        origin,
        json!({
            "op": "transaction_index_get_all",
            "transaction": transaction,
            "store": "items",
            "index": "by_group"
        }),
    )
    .unwrap();
    assert_eq!(indexed["entries"][0]["value"]["group"], "stable");
    assert!(
        call(
            &handler,
            "https://isolated.example",
            json!({"op": "inspect", "name": "app"})
        )
        .unwrap()["database"]
            .is_null()
    );
}

#[test]
fn indexed_db_handler_reports_persistence_failure_without_committing_live_data() {
    let directory = TestDirectory::new();
    let origin = "https://app.example";
    let handler = persistent_handler(directory.path());
    create_schema(&handler, origin);
    let transaction = begin_transaction(&handler, origin, "readwrite");
    call(
        &handler,
        origin,
        json!({
            "op": "transaction_add",
            "transaction": transaction,
            "store": "items",
            "value": {"group": "unstable"},
            "key": {"type": "string", "value": "new"}
        }),
    )
    .unwrap();

    fs::remove_dir_all(directory.path()).unwrap();
    fs::write(directory.path(), b"not-a-directory").unwrap();
    let error = call(
        &handler,
        origin,
        json!({"op": "commit_transaction", "transaction": transaction}),
    )
    .unwrap_err();
    assert!(error.starts_with("UnknownError: IndexedDB persistence failed:"));

    let read = begin_transaction(&handler, origin, "readonly");
    let record = call(
        &handler,
        origin,
        json!({
            "op": "transaction_get",
            "transaction": read,
            "store": "items",
            "key": {"type": "string", "value": "new"}
        }),
    )
    .unwrap();
    assert!(record["record"].is_null());
}

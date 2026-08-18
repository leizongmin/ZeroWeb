use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{IndexedDbOwner, WebView, WebViewBuilder, WebViewConfig};

static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("zeroweb-webview-indexeddb-{}-{sequence}", std::process::id()));
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

fn webview_with_owner(owner: IndexedDbOwner, origin: &str) -> WebView {
    let mut webview = WebView::new_with_indexed_db_owner(WebViewConfig::default(), owner);
    webview.prepare_document_state(origin);
    webview.execute_script("0").unwrap();
    webview
}

fn write_record(webview: &mut WebView, payload: &str) {
    let script = format!(
        r#"
        (() => {{
          const call = request => {{
            const wire = __zw_idb(JSON.stringify(request));
            if (wire.startsWith("__zw_idb_err:")) throw new Error(wire);
            return JSON.parse(wire.slice("__zw_idb_ok:".length));
          }};
          call({{
            op: "sync_schema",
            name: "app",
            version: 1,
            stores: [{{name: "items", keyPath: null, autoIncrement: false, indexes: []}}]
          }});
          const transaction = call({{
            op: "begin_transaction",
            database: "app",
            stores: ["items"],
            mode: "readwrite"
          }}).transaction;
          call({{
            op: "transaction_put",
            transaction,
            store: "items",
            key: {{type: "string", value: "key"}},
            value: {{payload: {payload:?}}}
          }});
          call({{op: "commit_transaction", transaction}});
          return "done";
        }})()
        "#
    );
    assert_eq!(webview.execute_script(&script).unwrap(), "done");
}

fn read_record(webview: &mut WebView) -> String {
    webview
        .execute_script(
            r#"
            (() => {
              const call = request => {
                const wire = __zw_idb(JSON.stringify(request));
                if (wire.startsWith("__zw_idb_err:")) throw new Error(wire);
                return JSON.parse(wire.slice("__zw_idb_ok:".length));
              };
              const database = call({op: "inspect", name: "app"}).database;
              if (database === null) return "missing";
              const transaction = call({
                op: "begin_transaction",
                database: "app",
                stores: ["items"],
                mode: "readonly"
              }).transaction;
              const record = call({
                op: "transaction_get",
                transaction,
                store: "items",
                key: {type: "string", value: "key"}
              }).record;
              call({op: "commit_transaction", transaction});
              return record === null ? "missing" : String(record.value.payload);
            })()
            "#,
        )
        .unwrap()
}

#[test]
fn embedded_webviews_share_only_the_injected_owner() {
    let origin = "https://embedded.example/page";
    let owner = IndexedDbOwner::in_memory();
    let mut first = webview_with_owner(owner.clone(), origin);
    write_record(&mut first, "shared");

    let mut second = WebViewBuilder::new().indexed_db_owner(owner).build();
    second.prepare_document_state(origin);
    second.execute_script("0").unwrap();
    assert_eq!(read_record(&mut second), "shared");

    let mut private = webview_with_owner(IndexedDbOwner::in_memory(), origin);
    assert_eq!(read_record(&mut private), "missing");
}

#[test]
fn persistent_owner_reads_record_after_webview_rebuild() {
    let directory = TestDirectory::new();
    let origin = "https://persistent.example/page";
    {
        let owner = IndexedDbOwner::persistent(directory.path()).unwrap();
        let mut webview = webview_with_owner(owner, origin);
        write_record(&mut webview, "restored");
    }

    let owner = IndexedDbOwner::persistent(directory.path()).unwrap();
    let mut restored = webview_with_owner(owner, origin);
    assert_eq!(read_record(&mut restored), "restored");
}

#[test]
fn webview_exposes_javascript_safe_integer_database_version() {
    let mut webview = webview_with_owner(IndexedDbOwner::in_memory(), "https://large-version.example/page");
    webview
        .execute_script(
            r#"
            globalThis.__largeVersions = [];
            const request = indexedDB.open("large-version", Number.MAX_SAFE_INTEGER);
            request.onupgradeneeded = event => {
              __largeVersions.push(String(event.oldVersion));
              __largeVersions.push(String(event.newVersion));
            };
            request.onsuccess = event => {
              __largeVersions.push(String(event.target.result.version));
            };
            "#,
        )
        .unwrap();
    webview.execute_script("0").unwrap();
    assert_eq!(
        webview.execute_script("__largeVersions.join('|')").unwrap(),
        "0|9007199254740991|9007199254740991"
    );
}

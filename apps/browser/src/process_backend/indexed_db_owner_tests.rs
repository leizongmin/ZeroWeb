use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::*;
use zero_protocol::{AutomationOperation, AutomationResult, AutomationValue};

fn lock_multiprocess_tests() -> std::sync::MutexGuard<'static, ()> {
    crate::tests::MULTIPROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

fn execute_script(
    backend: &mut ProcessTabBackend,
    snapshots: &mut HashMap<TabId, TabSnapshot>,
    tab_id: TabId,
    request_id: u64,
    script: &str,
) -> Result<AutomationValue, String> {
    backend.send_automation_request(
        tab_id,
        request_id,
        AutomationOperation::ExecuteScript {
            script: script.to_string(),
            arguments: Vec::new(),
        },
    )?;
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        backend.poll(snapshots, &mut HashMap::new(), Some(tab_id), true);
        if let Some(response) = backend.take_automation_response(tab_id, request_id) {
            return match response.result {
                Ok(AutomationResult::Value(value)) => Ok(value),
                Ok(other) => Err(format!("unexpected automation result: {other:?}")),
                Err(error) => Err(format!("{:?}: {}", error.code, error.message)),
            };
        }
        if Instant::now() >= deadline {
            return Err(format!("automation request {request_id} timed out"));
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn wait_for_value(
    backend: &mut ProcessTabBackend,
    snapshots: &mut HashMap<TabId, TabSnapshot>,
    tab_id: TabId,
    next_request_id: &mut u64,
    expression: &str,
) -> String {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let request_id = *next_request_id;
        *next_request_id += 1;
        if let Ok(AutomationValue::String(value)) = execute_script(
            backend,
            snapshots,
            tab_id,
            request_id,
            &format!("return String({expression});"),
        ) && value != "pending"
            && value != "undefined"
        {
            return value;
        }
        assert!(Instant::now() < deadline, "IndexedDB result did not settle");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn load_blank_page(
    backend: &mut ProcessTabBackend,
    snapshots: &mut HashMap<TabId, TabSnapshot>,
    tab_id: TabId,
    url: &str,
) {
    snapshots.insert(
        tab_id,
        TabSnapshot {
            url: Some(url.to_string()),
            navigation_epoch: 1,
            ..Default::default()
        },
    );
    backend.ensure_renderer(tab_id, (800, 600));
    backend.load_html(tab_id, "<!doctype html><html><body></body></html>", None, Some(url), 1);
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        backend.poll(snapshots, &mut HashMap::new(), Some(tab_id), true);
        if snapshots
            .get(&tab_id)
            .is_some_and(|snapshot| snapshot.last_render.is_some() || snapshot.compositor_submission.is_some())
        {
            return;
        }
        assert!(Instant::now() < deadline, "renderer did not load blank page");
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn commit_navigation(
    backend: &mut ProcessTabBackend,
    tab_id: TabId,
    renderer_id: u64,
    url: &str,
    navigation_epoch: u64,
) {
    backend.tab_to_renderer.insert(tab_id, renderer_id);
    backend.stage_indexed_db_navigation(renderer_id, url, navigation_epoch);
    backend.handle_navigation_committed(
        tab_id,
        renderer_id,
        NavigationCommittedParams {
            url: url.to_string(),
            navigation_epoch,
        },
    );
}

#[test]
fn private_owner_remains_available_when_regular_persistence_initialization_failed() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    backend.indexed_db_init_error = Some("corrupt database".to_string());
    let private = TabId(701);
    commit_navigation(&mut backend, private, 91, "https://private.example/page", 1);
    backend.set_tab_private(private, true);
    backend.handle_indexed_db_request(
        private,
        1,
        IndexedDbRequestParams {
            request: r#"{"op":"sync_schema","name":"app","version":1,"stores":[]}"#.to_string(),
        },
    );

    assert!(
        backend
            .private_storage
            .lock()
            .unwrap()
            .indexed_db("https://private.example", "app")
            .is_some()
    );
    assert!(
        backend
            .storage
            .lock()
            .unwrap()
            .indexed_db("https://private.example", "app")
            .is_none()
    );
}

#[test]
fn indexed_db_uses_committed_origin_instead_of_snapshot_url() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(702);
    commit_navigation(&mut backend, tab_id, 92, "https://committed.example/page", 4);

    backend.handle_indexed_db_request(
        tab_id,
        1,
        IndexedDbRequestParams {
            request: r#"{"op":"sync_schema","name":"app","version":1,"stores":[]}"#.to_string(),
        },
    );

    let storage = backend.storage.lock().unwrap();
    assert!(storage.indexed_db("https://committed.example", "app").is_some());
    assert!(storage.indexed_db("https://snapshot.example", "app").is_none());
}

#[test]
fn redirected_document_commits_final_origin() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(705);
    backend.tab_to_renderer.insert(tab_id, 95);
    backend.stage_indexed_db_navigation(95, "https://start.example/page", 6);
    backend.update_pending_indexed_db_navigation_from_fetch(
        tab_id,
        &[
            ("X-Zero-Resource-Type".to_string(), "document".to_string()),
            ("X-Zero-Final-URL".to_string(), "https://final.example/page".to_string()),
        ],
    );
    backend.handle_navigation_committed(
        tab_id,
        95,
        NavigationCommittedParams {
            url: "https://final.example/page".to_string(),
            navigation_epoch: 6,
        },
    );

    assert_eq!(
        backend.indexed_db_origins.get(&95).map(String::as_str),
        Some("https://final.example")
    );
    assert!(!backend.pending_indexed_db_navigations.contains_key(&95));
}

#[test]
fn mismatched_navigation_commit_does_not_enable_indexed_db() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(703);
    backend.tab_to_renderer.insert(tab_id, 93);
    backend.stage_indexed_db_navigation(93, "https://expected.example/page", 5);
    backend.handle_navigation_committed(
        tab_id,
        93,
        NavigationCommittedParams {
            url: "https://spoofed.example/page".to_string(),
            navigation_epoch: 5,
        },
    );

    assert!(!backend.indexed_db_origins.contains_key(&93));
    assert_eq!(
        backend.pending_indexed_db_navigations.get(&93),
        Some(&PendingIndexedDbNavigation {
            url: "https://expected.example/page".to_string(),
            navigation_epoch: 5,
        })
    );
    backend.handle_indexed_db_request(
        tab_id,
        1,
        IndexedDbRequestParams {
            request: r#"{"op":"sync_schema","name":"app","version":1,"stores":[]}"#.to_string(),
        },
    );
    let storage = backend.storage.lock().unwrap();
    assert!(storage.indexed_db("https://expected.example", "app").is_none());
    assert!(storage.indexed_db("https://spoofed.example", "app").is_none());
}

#[test]
fn navigation_start_revokes_previous_origin_and_transaction_registry() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(704);
    commit_navigation(&mut backend, tab_id, 94, "https://old.example/page", 1);
    backend
        .indexed_db_handlers
        .insert(94, zero_page_runtime::indexed_db_handler(Arc::clone(&backend.storage)));
    let mut snapshot = TabSnapshot {
        url: Some("https://old.example/page".to_string()),
        navigation_epoch: 1,
        ..Default::default()
    };

    backend.handle_navigation_started(
        tab_id,
        94,
        &mut snapshot,
        NavigationStartedParams {
            url: "https://new.example/page".to_string(),
            navigation_epoch: 2,
        },
    );

    assert_eq!(snapshot.navigation_epoch, 2);
    assert_eq!(snapshot.url.as_deref(), Some("https://new.example/page"));
    assert!(!backend.indexed_db_origins.contains_key(&94));
    assert!(!backend.indexed_db_handlers.contains_key(&94));
    assert_eq!(
        backend.pending_indexed_db_navigations.get(&94),
        Some(&PendingIndexedDbNavigation {
            url: "https://new.example/page".to_string(),
            navigation_epoch: 2,
        })
    );
}

#[test]
fn independent_renderers_share_browser_owned_indexed_db() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let Some(renderer_bin) = resolve_renderer_binary() else {
        eprintln!("zero-renderer binary unavailable; skipping process ownership smoke");
        return;
    };
    let mut backend = ProcessTabBackend::with_renderer_bin(renderer_bin);
    let mut snapshots = HashMap::new();
    let first = TabId(801);
    let second = TabId(802);
    let origin_url = "https://shared.example/page";
    load_blank_page(&mut backend, &mut snapshots, first, origin_url);
    load_blank_page(&mut backend, &mut snapshots, second, origin_url);

    let mut request_id = 1;
    execute_script(
        &mut backend,
        &mut snapshots,
        first,
        request_id,
        r#"
          globalThis.__ownerWrite = "pending";
          var open = indexedDB.open("owner-db", 1);
          open.onupgradeneeded = function () {
            open.result.createObjectStore("items");
          };
          open.onsuccess = function () {
            var tx = open.result.transaction("items", "readwrite");
            tx.objectStore("items").put({value:"shared"}, "key");
            tx.oncomplete = function () { globalThis.__ownerWrite = "done"; };
          };
          return "started";
        "#,
    )
    .unwrap();
    request_id += 1;
    assert_eq!(
        wait_for_value(
            &mut backend,
            &mut snapshots,
            first,
            &mut request_id,
            "globalThis.__ownerWrite"
        ),
        "done"
    );

    execute_script(
        &mut backend,
        &mut snapshots,
        second,
        request_id,
        r#"
          globalThis.__ownerRead = "pending";
          var open = indexedDB.open("owner-db");
          open.onsuccess = function () {
            var get = open.result.transaction("items").objectStore("items").get("key");
            get.onsuccess = function () {
              globalThis.__ownerRead = get.result && get.result.value;
            };
          };
          return "started";
        "#,
    )
    .unwrap();
    request_id += 1;
    assert_eq!(
        wait_for_value(
            &mut backend,
            &mut snapshots,
            second,
            &mut request_id,
            "globalThis.__ownerRead"
        ),
        "shared"
    );

    let private = TabId(803);
    backend.set_tab_private(private, true);
    load_blank_page(&mut backend, &mut snapshots, private, origin_url);
    execute_script(
        &mut backend,
        &mut snapshots,
        private,
        request_id,
        r#"
          globalThis.__privateIsolation = "pending";
          var open = indexedDB.open("owner-db", 1);
          open.onupgradeneeded = function () {
            globalThis.__privateIsolation =
              open.result.objectStoreNames.contains("items") ? "leaked" : "isolated";
            open.result.createObjectStore("private-items");
          };
          return "started";
        "#,
    )
    .unwrap();
    request_id += 1;
    assert_eq!(
        wait_for_value(
            &mut backend,
            &mut snapshots,
            private,
            &mut request_id,
            "globalThis.__privateIsolation"
        ),
        "isolated"
    );

    load_blank_page(&mut backend, &mut snapshots, second, "https://isolated.example/page");
    execute_script(
        &mut backend,
        &mut snapshots,
        second,
        request_id,
        r#"
          globalThis.__ownerIsolation = "pending";
          var open = indexedDB.open("owner-db", 1);
          open.onupgradeneeded = function () {
            globalThis.__ownerIsolation =
              open.result.objectStoreNames.contains("items") ? "leaked" : "isolated";
            open.result.createObjectStore("other");
          };
          return "started";
        "#,
    )
    .unwrap();
    request_id += 1;
    assert_eq!(
        wait_for_value(
            &mut backend,
            &mut snapshots,
            second,
            &mut request_id,
            "globalThis.__ownerIsolation"
        ),
        "isolated"
    );
}

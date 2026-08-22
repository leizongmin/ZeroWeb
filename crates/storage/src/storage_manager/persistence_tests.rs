use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;
use crate::indexed_db::{IdbIndexKeyPath, IdbKey};

static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "zeroweb-indexeddb-persistence-{}-{sequence}",
            std::process::id()
        ));
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

#[test]
fn indexed_db_persistence_round_trips_schema_records_indexes_and_generator() {
    let directory = TestDirectory::new();
    let origin = "https://app.example";
    {
        let mut manager = StorageManager::with_indexed_db_persistence(directory.path()).unwrap();
        manager.open_indexed_db(origin, "app", 3).unwrap();
        manager
            .mutate_indexed_db(origin, "app", |database| {
                database.create_object_store("items", None, true)?;
                database.create_index_with_key_path(
                    "items",
                    "by_tags",
                    IdbIndexKeyPath::String("tags".to_string()),
                    false,
                    true,
                )?;
                database.create_index_with_key_path(
                    "items",
                    "by_name",
                    IdbIndexKeyPath::Sequence(vec!["last".to_string(), "first".to_string()]),
                    true,
                    false,
                )?;
                database.add(
                    "items",
                    serde_json::json!({
                        "first": "Ada",
                        "last": "Lovelace",
                        "tags": ["math", "code"],
                        "wire": {"__zwIdbType": "graph", "root": {"__zwIdbType": "ref", "value": 0}}
                    }),
                    None,
                )?;
                let removed = database.add(
                    "items",
                    serde_json::json!({"first": "Grace", "last": "Hopper", "tags": ["code"]}),
                    Some(IdbKey::Number(10.0)),
                )?;
                database.delete("items", &removed)?;
                database.create_object_store("keys", None, false)?;
                database.put(
                    "keys",
                    serde_json::json!("date"),
                    Some(IdbKey::Date(1_700_000_000_000.0)),
                )?;
                database.put(
                    "keys",
                    serde_json::json!("array"),
                    Some(IdbKey::Array(vec![
                        IdbKey::String("nested".to_string()),
                        IdbKey::Number(f64::INFINITY),
                    ])),
                )?;
                database.put(
                    "keys",
                    serde_json::json!("binary"),
                    Some(IdbKey::Binary(vec![0, 1, 127, 255])),
                )?;
                database.put("keys", serde_json::json!("negative-zero"), Some(IdbKey::Number(-0.0)))?;
                Ok(())
            })
            .unwrap();

        manager
            .open_indexed_db("https://isolated.example", "app", 1)
            .unwrap()
            .create_object_store("other", None, false)
            .unwrap();
        manager
            .mutate_indexed_db("https://isolated.example", "app", |_| Ok(()))
            .unwrap();
    }

    let mut restored = StorageManager::with_indexed_db_persistence(directory.path()).unwrap();
    let database = restored.indexed_db(origin, "app").unwrap();
    assert_eq!(database.version, 3);
    assert_eq!(database.count("items").unwrap(), 1);
    let indexed_record = database
        .get_from_index("items", "by_tags", &IdbKey::String("code".to_string()))
        .unwrap()[0];
    assert_eq!(indexed_record.value["first"], "Ada");
    assert_eq!(indexed_record.value["wire"]["__zwIdbType"], "graph");
    assert_eq!(
        database
            .get(
                "keys",
                &IdbKey::Array(vec![
                    IdbKey::String("nested".to_string()),
                    IdbKey::Number(f64::INFINITY),
                ]),
            )
            .unwrap()
            .value,
        "array"
    );
    assert_eq!(
        database
            .get("keys", &IdbKey::Binary(vec![0, 1, 127, 255]))
            .unwrap()
            .value,
        "binary"
    );
    assert_eq!(
        database.get("keys", &IdbKey::Number(-0.0)).unwrap().value,
        "negative-zero"
    );
    assert!(restored.indexed_db(origin, "app").unwrap().has_store("items"));
    assert!(
        restored
            .indexed_db("https://isolated.example", "app")
            .unwrap()
            .has_store("other")
    );
    assert!(
        !restored
            .indexed_db("https://isolated.example", "app")
            .unwrap()
            .has_store("items")
    );

    let generated = restored
        .mutate_indexed_db(origin, "app", |database| {
            database.add(
                "items",
                serde_json::json!({"first": "Katherine", "last": "Johnson", "tags": []}),
                None,
            )
        })
        .unwrap();
    assert_eq!(generated, IdbKey::Number(11.0));
}

#[test]
fn indexed_db_persistence_rejects_corrupt_files() {
    let directory = TestDirectory::new();
    let origin_directory = directory.path().join("invalid-origin");
    fs::create_dir_all(&origin_directory).unwrap();
    fs::write(origin_directory.join("database.json"), b"{not-json").unwrap();

    let error = match StorageManager::with_indexed_db_persistence(directory.path()) {
        Ok(_) => panic!("corrupt IndexedDB persistence data must be rejected"),
        Err(error) => error,
    };
    assert!(matches!(error, StorageError::Serialization(_)));
}

#[test]
fn indexed_db_persistence_failure_keeps_live_database_unchanged() {
    let directory = TestDirectory::new();
    let origin = "https://app.example";
    let mut manager = StorageManager::with_indexed_db_persistence(directory.path()).unwrap();
    manager.open_indexed_db(origin, "app", 1).unwrap();
    manager
        .mutate_indexed_db(origin, "app", |database| {
            database.create_object_store("items", None, false)?;
            database.put(
                "items",
                serde_json::json!("before"),
                Some(IdbKey::String("stable".to_string())),
            )?;
            Ok(())
        })
        .unwrap();

    fs::remove_dir_all(directory.path()).unwrap();
    fs::write(directory.path(), b"not-a-directory").unwrap();
    let error = manager
        .mutate_indexed_db(origin, "app", |database| {
            database.put(
                "items",
                serde_json::json!("after"),
                Some(IdbKey::String("new".to_string())),
            )?;
            Ok(())
        })
        .unwrap_err();
    assert!(matches!(error, StorageError::Io(_)));
    let database = manager.indexed_db(origin, "app").unwrap();
    assert_eq!(
        database
            .get("items", &IdbKey::String("stable".to_string()))
            .unwrap()
            .value,
        "before"
    );
    assert!(database.get("items", &IdbKey::String("new".to_string())).is_none());
}

#[test]
fn indexed_db_persistence_recovers_backup_and_removes_orphan_temp_file() {
    let directory = TestDirectory::new();
    {
        let mut manager = StorageManager::with_indexed_db_persistence(directory.path()).unwrap();
        manager.open_indexed_db("https://app.example", "app", 1).unwrap();
    }
    let origin_directory = fs::read_dir(directory.path()).unwrap().next().unwrap().unwrap().path();
    let database_path = fs::read_dir(&origin_directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .unwrap();
    let backup_path =
        database_path.with_file_name(format!("{}.bak", database_path.file_name().unwrap().to_string_lossy()));
    fs::rename(&database_path, &backup_path).unwrap();
    let orphan = origin_directory.join(".orphan.tmp");
    fs::write(&orphan, b"incomplete").unwrap();

    let manager = StorageManager::with_indexed_db_persistence(directory.path()).unwrap();
    assert!(manager.indexed_db("https://app.example", "app").is_some());
    assert!(database_path.exists());
    assert!(!backup_path.exists());
    assert!(!orphan.exists());
}

#[test]
fn cache_storage_persistence_round_trips_named_caches_entries_and_empty_cache() {
    let directory = TestDirectory::new();
    let origin = "https://cache.example";
    {
        let mut manager = StorageManager::with_persistence(directory.path()).unwrap();
        manager.open_cache_storage_cache(origin, "empty").unwrap();
        manager
            .mutate_cache_storage(origin, |cache_storage| {
                cache_storage.open("assets").put(
                    crate::cache_api::CacheRequest::with_method_and_headers(
                        "https://cache.example/app/data.txt?version=1#old",
                        "GET",
                        vec![("Accept-Language".to_string(), "en".to_string())],
                    ),
                    crate::cache_api::CacheResponse {
                        url: "https://cache.example/app/data.txt?version=1".to_string(),
                        status: 201,
                        status_text: "Created".to_string(),
                        response_type: "basic".to_string(),
                        headers: [
                            ("Content-Type".to_string(), "text/plain".to_string()),
                            ("Vary".to_string(), "Accept-Language".to_string()),
                        ]
                        .into(),
                        body: b"cached body".to_vec(),
                    },
                )?;
                Ok(())
            })
            .unwrap();
    }

    let restored = StorageManager::with_persistence(directory.path()).unwrap();
    let cache_storage = restored.cache_storage_ref(origin).unwrap();
    assert_eq!(cache_storage.keys(), vec!["empty", "assets"]);
    assert!(cache_storage.get("empty").unwrap().is_empty());
    let matched = cache_storage
        .get("assets")
        .unwrap()
        .match_request_with_options(
            &crate::cache_api::CacheRequest::with_method_and_headers(
                "https://cache.example/app/data.txt?version=2#new",
                "GET",
                vec![("Accept-Language".to_string(), "en".to_string())],
            ),
            crate::cache_api::CacheQueryOptions {
                ignore_search: true,
                ignore_method: false,
                ignore_vary: false,
            },
        )
        .unwrap();
    assert_eq!(matched.status, 201);
    assert_eq!(matched.status_text, "Created");
    assert_eq!(matched.response_type, "basic");
    assert_eq!(matched.url, "https://cache.example/app/data.txt?version=1");
    assert_eq!(matched.headers.get("Content-Type"), Some(&"text/plain".to_string()));
    assert_eq!(matched.body, b"cached body");
}

#[test]
fn cache_storage_persistence_deletes_origin_file_after_last_cache_removed() {
    let directory = TestDirectory::new();
    let origin = "https://cache-delete.example";
    {
        let mut manager = StorageManager::with_persistence(directory.path()).unwrap();
        manager.open_cache_storage_cache(origin, "v1").unwrap();
        assert!(manager.delete_cache_storage_cache(origin, "v1").unwrap());
        assert!(manager.cache_storage_ref(origin).is_none());
    }

    let restored = StorageManager::with_persistence(directory.path()).unwrap();
    assert!(restored.cache_storage_ref(origin).is_none());
}

#[test]
fn cache_storage_persistence_failure_keeps_live_storage_unchanged() {
    let directory = TestDirectory::new();
    let origin = "https://cache-failure.example";
    let mut manager = StorageManager::with_persistence(directory.path()).unwrap();
    manager
        .mutate_cache_storage(origin, |cache_storage| {
            cache_storage.open("assets").put(
                crate::cache_api::CacheRequest::new("https://cache-failure.example/stable.txt"),
                crate::cache_api::CacheResponse::ok(b"stable".to_vec()),
            )?;
            Ok(())
        })
        .unwrap();

    fs::remove_dir_all(directory.path()).unwrap();
    fs::write(directory.path(), b"not-a-directory").unwrap();
    let error = manager
        .mutate_cache_storage(origin, |cache_storage| {
            cache_storage.open("assets").put(
                crate::cache_api::CacheRequest::new("https://cache-failure.example/new.txt"),
                crate::cache_api::CacheResponse::ok(b"new".to_vec()),
            )?;
            Ok(())
        })
        .unwrap_err();
    assert!(matches!(error, StorageError::Io(_)));

    let cache = manager.cache_storage_ref(origin).unwrap().get("assets").unwrap();
    assert!(
        cache
            .match_request(&crate::cache_api::CacheRequest::new(
                "https://cache-failure.example/stable.txt"
            ))
            .is_some()
    );
    assert!(
        cache
            .match_request(&crate::cache_api::CacheRequest::new(
                "https://cache-failure.example/new.txt"
            ))
            .is_none()
    );
}

// IDB types coverage round 2 - targeting uncovered paths in types.rs

use super::cursor::CursorDirection;
use super::types::*;

// ── IdbDatabase rename_object_store coverage ──

#[test]
fn test_rename_object_store_basic() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store1", Some("id"), false).unwrap();
    db.rename_object_store("store1", "store2").unwrap();
    assert!(!db.has_store("store1"));
    assert!(db.has_store("store2"));
}

#[test]
fn test_rename_object_store_not_found() {
    let mut db = IdbDatabase::new("test", 1);
    let result = db.rename_object_store("nonexistent", "new_name");
    assert!(result.is_err());
}

#[test]
fn test_rename_object_store_target_exists() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store1", Some("id"), false).unwrap();
    db.create_object_store("store2", Some("id"), false).unwrap();
    let result = db.rename_object_store("store1", "store2");
    assert!(result.is_err());
}

// ── IdbDatabase add with auto_increment ──

#[test]
fn test_add_auto_increment_no_key() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, true).unwrap();
    let key1 = db.add("store", serde_json::json!({"name": "a"}), None).unwrap();
    let key2 = db.add("store", serde_json::json!({"name": "b"}), None).unwrap();
    assert!(key1 < key2);
}

#[test]
fn test_add_no_key_no_auto_increment() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let result = db.add("store", serde_json::json!({"name": "a"}), None);
    assert!(result.is_err());
}

// ── IdbDatabase put (overwrite) ──

#[test]
fn test_put_overwrites_existing() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    let key = IdbKey::String("k1".to_string());
    db.add("store", serde_json::json!({"id": "k1", "val": 1}), Some(key.clone()))
        .unwrap();
    db.put("store", serde_json::json!({"id": "k1", "val": 2}), Some(key.clone()))
        .unwrap();
    let record = db.get("store", &key).unwrap();
    assert_eq!(record.value["val"], 2);
}

// ── IdbDatabase delete ──

#[test]
fn test_delete_record() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    let key = IdbKey::String("k1".to_string());
    db.add("store", serde_json::json!({"id": "k1"}), Some(key.clone()))
        .unwrap();
    db.delete("store", &key).unwrap();
    assert!(db.get("store", &key).is_none());
}

// ── IdbDatabase clear_store ──

#[test]
fn test_clear_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "k1"}),
        Some(IdbKey::String("k1".to_string())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "k2"}),
        Some(IdbKey::String("k2".to_string())),
    )
    .unwrap();
    db.clear_store("store").unwrap();
    assert_eq!(db.count("store").unwrap(), 0);
}

// ── IdbDatabase count_with_range ──

#[test]
fn test_count_with_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    for i in 0..10 {
        let key = IdbKey::Number(i as f64);
        db.add("store", serde_json::json!({"id": i}), Some(key)).unwrap();
    }
    let range = IdbKeyRange::lower_bound(IdbKey::Number(5.0), false);
    let count = db.count_with_range("store", &range).unwrap();
    assert_eq!(count, 5);
}

// ── IdbDatabase get_all and get_all_with_range ──

#[test]
fn test_get_all_records() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "a"}),
        Some(IdbKey::String("a".to_string())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "b"}),
        Some(IdbKey::String("b".to_string())),
    )
    .unwrap();
    let all = db.get_all("store").unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn test_get_all_with_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    for i in 0..5 {
        db.add("store", serde_json::json!({"id": i}), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }
    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(3.0), false, false);
    let results = db.get_all_with_range("store", &range).unwrap();
    assert_eq!(results.len(), 3);
}

// ── IdbKeyRange upper_bound and contains ──

#[test]
fn test_key_range_upper_bound() {
    let range = IdbKeyRange::upper_bound(IdbKey::Number(10.0), true);
    assert!(range.contains(&IdbKey::Number(5.0)));
    assert!(!range.contains(&IdbKey::Number(10.0))); // open upper bound
}

#[test]
fn test_key_range_contains_bounds() {
    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), false, false);
    assert!(range.contains(&IdbKey::Number(1.0)));
    assert!(range.contains(&IdbKey::Number(10.0)));
    assert!(range.contains(&IdbKey::Number(5.0)));
    assert!(!range.contains(&IdbKey::Number(0.0)));
    assert!(!range.contains(&IdbKey::Number(11.0)));
}

// ── IdbKeyRange accessors ──

#[test]
fn test_key_range_accessors() {
    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), true, false);
    assert_eq!(range.lower(), Some(&IdbKey::Number(1.0)));
    assert_eq!(range.upper(), Some(&IdbKey::Number(10.0)));
    assert!(range.lower_open());
    assert!(!range.upper_open());
}

// ── IdbKey cross-type ordering ──

#[test]
fn test_idb_key_cross_type_ordering() {
    let num = IdbKey::Number(1.0);
    let str = IdbKey::String("a".to_string());
    let bin = IdbKey::Binary(vec![1]);
    let arr = IdbKey::Array(vec![]);
    assert!(num < str);
    assert!(str < bin);
    assert!(bin < arr);
}

#[test]
fn test_idb_key_array_comparison() {
    let k1 = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::String("a".to_string())]);
    let k2 = IdbKey::Array(vec![IdbKey::Number(2.0), IdbKey::String("a".to_string())]);
    let k3 = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::String("b".to_string())]);
    assert!(k1 < k2);
    assert!(k1 < k3);
}

#[test]
fn test_idb_key_binary_comparison() {
    let k1 = IdbKey::Binary(vec![1, 2, 3]);
    let k2 = IdbKey::Binary(vec![1, 2, 4]);
    assert!(k1 < k2);
}

// ── Index operations coverage ──

#[test]
fn test_create_index_and_get_from_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "u1", "name": "Alice"}),
        Some(IdbKey::String("u1".to_string())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "u2", "name": "Bob"}),
        Some(IdbKey::String("u2".to_string())),
    )
    .unwrap();
    db.create_index("store", "name_idx", "name", false, false).unwrap();
    let records = db
        .get_from_index("store", "name_idx", &IdbKey::String("Alice".to_string()))
        .unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].value["id"], "u1");
}

#[test]
fn test_get_all_from_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "u1", "name": "Alice"}),
        Some(IdbKey::String("u1".to_string())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "u2", "name": "Bob"}),
        Some(IdbKey::String("u2".to_string())),
    )
    .unwrap();
    db.create_index("store", "name_idx", "name", false, false).unwrap();
    let all = db.get_all_from_index("store", "name_idx").unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn test_count_from_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "u1", "name": "Alice"}),
        Some(IdbKey::String("u1".to_string())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "u2", "name": "Bob"}),
        Some(IdbKey::String("u2".to_string())),
    )
    .unwrap();
    db.create_index("store", "name_idx", "name", false, false).unwrap();
    let range = IdbKeyRange::lower_bound(IdbKey::String("A".to_string()), false);
    let count = db.count_from_index("store", "name_idx", Some(&range)).unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_delete_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    db.create_index("store", "idx1", "name", false, false).unwrap();
    let names = db.index_names("store").unwrap();
    assert!(names.contains(&"idx1"));
    db.delete_index("store", "idx1").unwrap();
    let names = db.index_names("store").unwrap();
    assert!(!names.contains(&"idx1"));
}

// ── Transaction coverage ──

#[test]
fn test_transaction_commit() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(
        &mut tx,
        "store",
        serde_json::json!({"id": "k1"}),
        Some(IdbKey::String("k1".to_string())),
    )
    .unwrap();
    db.tx_add(
        &mut tx,
        "store",
        serde_json::json!({"id": "k2"}),
        Some(IdbKey::String("k2".to_string())),
    )
    .unwrap();
    db.commit_tx(&mut tx).unwrap();
    assert_eq!(db.count("store").unwrap(), 2);
}

#[test]
fn test_transaction_put_and_get() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "k1", "val": 1}),
        Some(IdbKey::String("k1".to_string())),
    )
    .unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_put(
        &mut tx,
        "store",
        serde_json::json!({"id": "k1", "val": 42}),
        Some(IdbKey::String("k1".to_string())),
    )
    .unwrap();
    db.commit_tx(&mut tx).unwrap();
    let record = db.get("store", &IdbKey::String("k1".to_string())).unwrap();
    assert_eq!(record.value["val"], 42);
}

#[test]
fn test_transaction_delete() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "k1"}),
        Some(IdbKey::String("k1".to_string())),
    )
    .unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_delete(&mut tx, "store", &IdbKey::String("k1".to_string()))
        .unwrap();
    db.commit_tx(&mut tx).unwrap();
    assert_eq!(db.count("store").unwrap(), 0);
}

// ── store management ──

#[test]
fn test_store_names_and_has_store() {
    let mut db = IdbDatabase::new("test", 1);
    assert!(db.store_names().is_empty());
    db.create_object_store("s1", Some("id"), false).unwrap();
    db.create_object_store("s2", None, true).unwrap();
    assert_eq!(db.store_names().len(), 2);
    assert!(db.has_store("s1"));
    assert!(db.has_store("s2"));
}

#[test]
fn test_delete_object_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("s1", Some("id"), false).unwrap();
    db.delete_object_store("s1").unwrap();
    assert!(!db.has_store("s1"));
}

// ── Cursor operations ──

#[test]
fn test_open_cursor_basic() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    for i in 0..5 {
        db.add("store", serde_json::json!({"id": i}), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }
    let cursor = db.open_cursor("store", None).unwrap();
    assert!(cursor.is_some());
}

#[test]
fn test_open_key_cursor() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    for i in 0..3 {
        db.add("store", serde_json::json!({"id": i}), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }
    let cursor = db.open_key_cursor("store", None).unwrap();
    assert!(cursor.is_some());
}

#[test]
fn test_cursor_on_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", Some("id"), false).unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "u1", "name": "Alice"}),
        Some(IdbKey::String("u1".to_string())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"id": "u2", "name": "Bob"}),
        Some(IdbKey::String("u2".to_string())),
    )
    .unwrap();
    db.create_index("store", "name_idx", "name", false, false).unwrap();
    let cursor = db.open_cursor_on_index("store", "name_idx", None).unwrap();
    assert!(cursor.is_some());
}

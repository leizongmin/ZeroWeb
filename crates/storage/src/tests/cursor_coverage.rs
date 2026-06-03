//! IndexedDB cursor coverage tests - using public API.

use serde_json::json;
use zero_storage::indexed_db::*;

#[test]
fn test_idb_cursor_advance_via_database() {
    let mut db = IdbDatabase::new("test_cursor_db", 1);
    db.create_object_store("items", None, false).unwrap();

    db.add("items", json!("a"), Some(IdbKey::Number(1.0))).unwrap();
    db.add("items", json!("b"), Some(IdbKey::Number(2.0))).unwrap();
    db.add("items", json!("c"), Some(IdbKey::Number(3.0))).unwrap();

    if let Ok(Some(mut cursor)) = db.open_cursor("items", None) {
        // First record
        assert!(!cursor.is_finished());
        // Advance
        let advanced = cursor.advance(1);
        assert!(advanced);
    }
}

#[test]
fn test_idb_cursor_empty_store() {
    let mut db = IdbDatabase::new("test_cursor_empty", 1);
    db.create_object_store("empty", None, false).unwrap();
    let result = db.open_cursor("empty", None);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_idb_cursor_continue_next() {
    let mut db = IdbDatabase::new("test_cursor_next", 1);
    db.create_object_store("items", None, false).unwrap();

    db.add("items", json!("a"), Some(IdbKey::Number(1.0))).unwrap();
    db.add("items", json!("b"), Some(IdbKey::Number(2.0))).unwrap();

    if let Ok(Some(mut cursor)) = db.open_cursor("items", None) {
        assert!(cursor.continue_next());
        // After continuing past the last record
        assert!(cursor.is_finished() || !cursor.continue_next());
    }
}

#[test]
fn test_idb_cursor_with_key_range() {
    let mut db = IdbDatabase::new("test_cursor_range", 1);
    db.create_object_store("items", None, false).unwrap();

    db.add("items", json!("a"), Some(IdbKey::Number(1.0))).unwrap();
    db.add("items", json!("b"), Some(IdbKey::Number(2.0))).unwrap();
    db.add("items", json!("c"), Some(IdbKey::Number(3.0))).unwrap();

    let range = IdbKeyRange::bound(IdbKey::Number(2.0), IdbKey::Number(3.0), false, false);
    if let Ok(Some(cursor)) = db.open_cursor("items", Some(&range)) {
        assert!(!cursor.is_finished());
    }
}

#[test]
fn test_idb_cursor_is_finished() {
    let mut db = IdbDatabase::new("test_cursor_finished", 1);
    db.create_object_store("items", None, false).unwrap();
    db.add("items", json!("a"), Some(IdbKey::Number(1.0))).unwrap();

    if let Ok(Some(mut cursor)) = db.open_cursor("items", None) {
        assert!(!cursor.is_finished());
        cursor.advance(10);
        assert!(cursor.is_finished());
    }
}

//! IndexedDB cursor 和 types 错误路径覆盖率测试。

use super::cursor::*;
use super::types::*;
use crate::StorageError;
use serde_json::json;
use std::cell::RefCell;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════
// Cursor edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_cursor_advance_zero_resets() {
    let mut cursor = IdbCursor {
        direction: CursorDirection::Next,
        keys: vec![IdbKey::Number(1.0), IdbKey::Number(2.0), IdbKey::Number(3.0)],
        current: 2,
        store_name: "test".to_string(),
    };
    assert!(cursor.advance(0)); // 重置到 0
    assert_eq!(cursor.current, 0);
}

#[test]
fn test_cursor_advance_beyond_end() {
    let mut cursor = IdbCursor {
        direction: CursorDirection::Next,
        keys: vec![IdbKey::Number(1.0), IdbKey::Number(2.0)],
        current: 0,
        store_name: "test".to_string(),
    };
    assert!(!cursor.advance(5)); // 超出范围
}

#[test]
fn test_cursor_continue_to_found() {
    let mut cursor = IdbCursor {
        direction: CursorDirection::Next,
        keys: vec![IdbKey::Number(1.0), IdbKey::Number(3.0), IdbKey::Number(5.0)],
        current: 0,
        store_name: "test".to_string(),
    };
    assert!(cursor.continue_to(&IdbKey::Number(3.0)));
    assert_eq!(cursor.current, 1);
}

#[test]
fn test_cursor_continue_to_not_found() {
    let mut cursor = IdbCursor {
        direction: CursorDirection::Next,
        keys: vec![IdbKey::Number(1.0), IdbKey::Number(3.0)],
        current: 0,
        store_name: "test".to_string(),
    };
    assert!(!cursor.continue_to(&IdbKey::Number(10.0)));
}

#[test]
fn test_cursor_is_finished() {
    let mut cursor = IdbCursor {
        direction: CursorDirection::Next,
        keys: vec![IdbKey::Number(1.0)],
        current: 0,
        store_name: "test".to_string(),
    };
    assert!(!cursor.is_finished());
    cursor.advance(1);
    assert!(cursor.is_finished());
}

#[test]
fn test_cursor_key_at_current() {
    let cursor = IdbCursor {
        direction: CursorDirection::Next,
        keys: vec![IdbKey::Number(1.0), IdbKey::Number(2.0)],
        current: 1,
        store_name: "test".to_string(),
    };
    assert_eq!(cursor.key(), Some(&IdbKey::Number(2.0)));
}

#[test]
fn test_cursor_with_value_advance_zero() {
    let mut cursor = IdbCursorWithValue {
        direction: CursorDirection::Next,
        positions: vec![0, 1, 2],
        current: 2,
        store_name: "test".to_string(),
    };
    assert!(cursor.advance(0));
    assert_eq!(cursor.current, 0);
}

#[test]
fn test_cursor_with_value_advance_beyond() {
    let mut cursor = IdbCursorWithValue {
        direction: CursorDirection::Next,
        positions: vec![0, 1],
        current: 0,
        store_name: "test".to_string(),
    };
    assert!(!cursor.advance(10));
}

#[test]
fn test_cursor_with_value_continue_next() {
    let mut cursor = IdbCursorWithValue {
        direction: CursorDirection::Next,
        positions: vec![0, 1, 2],
        current: 0,
        store_name: "test".to_string(),
    };
    assert!(cursor.continue_next());
    assert!(!cursor.is_finished());
    assert!(cursor.continue_next());
    assert!(!cursor.continue_next()); // 到末尾
    assert!(cursor.is_finished());
}

#[test]
fn test_cursor_with_value_position() {
    let cursor = IdbCursorWithValue {
        direction: CursorDirection::Next,
        positions: vec![0, 5, 10],
        current: 1,
        store_name: "test".to_string(),
    };
    assert_eq!(cursor.position(), 1);
}

#[test]
fn test_cursor_store_name() {
    let cursor = IdbCursor {
        direction: CursorDirection::Next,
        keys: vec![],
        current: 0,
        store_name: "mystore".to_string(),
    };
    assert_eq!(cursor.store_name(), "mystore");
}

#[test]
fn test_cursor_with_value_store_name() {
    let cursor = IdbCursorWithValue {
        direction: CursorDirection::Next,
        positions: vec![],
        current: 0,
        store_name: "mystore".to_string(),
    };
    assert_eq!(cursor.store_name(), "mystore");
}

// ═══════════════════════════════════════════════════════════════════════
// Transaction error paths
// ═══════════════════════════════════════════════════════════════════════

fn make_tx() -> IdbTransaction {
    IdbTransaction {
        store_names: vec!["store".to_string()],
        mode: IdbTransactionMode::ReadWrite,
        db_name: "test".to_string(),
        db_version: 1,
        aborted: false,
        committed: false,
        mutations: RefCell::new(Vec::new()),
        key_gens: RefCell::new(HashMap::new()),
    }
}

#[test]
fn test_transaction_check_active_aborted() {
    let mut tx = make_tx();
    tx.aborted = true;
    assert!(tx.check_active("store").is_err());
}

#[test]
fn test_transaction_check_active_committed() {
    let mut tx = make_tx();
    tx.committed = true;
    assert!(tx.check_active("store").is_err());
}

#[test]
fn test_transaction_check_active_store_not_in_scope() {
    let tx = make_tx();
    assert!(tx.check_active("other_store").is_err());
}

#[test]
fn test_transaction_check_active_ok() {
    let tx = make_tx();
    assert!(tx.check_active("store").is_ok());
}

#[test]
fn test_transaction_commit_aborted() {
    let mut tx = make_tx();
    tx.aborted = true;
    assert!(tx.commit().is_err());
}

#[test]
fn test_transaction_commit_already_committed() {
    let mut tx = make_tx();
    tx.committed = true;
    assert!(tx.commit().is_err());
}

#[test]
fn test_transaction_commit_ok() {
    let mut tx = make_tx();
    assert!(tx.commit().is_ok());
    assert!(tx.is_committed());
}

#[test]
fn test_transaction_abort_already_aborted() {
    let mut tx = make_tx();
    tx.aborted = true;
    assert!(tx.abort().is_err());
}

#[test]
fn test_transaction_abort_after_commit() {
    let mut tx = make_tx();
    tx.committed = true;
    assert!(tx.abort().is_err());
}

#[test]
fn test_transaction_abort_ok() {
    let mut tx = make_tx();
    assert!(tx.abort().is_ok());
    assert!(tx.is_aborted());
}

#[test]
fn test_transaction_accessors() {
    let tx = IdbTransaction {
        store_names: vec!["s1".to_string(), "s2".to_string()],
        mode: IdbTransactionMode::ReadOnly,
        db_name: "mydb".to_string(),
        db_version: 3,
        aborted: false,
        committed: false,
        mutations: RefCell::new(Vec::new()),
        key_gens: RefCell::new(HashMap::new()),
    };
    assert_eq!(tx.mode(), IdbTransactionMode::ReadOnly);
    assert_eq!(tx.store_names().len(), 2);
    assert_eq!(tx.db_name(), "mydb");
    assert_eq!(tx.db_version(), 3);
    assert!(!tx.is_committed());
}

// ═══════════════════════════════════════════════════════════════════════
// IdbDatabase helpers
// ═══════════════════════════════════════════════════════════════════════

fn make_test_db() -> IdbDatabase {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("store", None, false).unwrap();
    db
}

#[test]
fn test_create_store_already_exists() {
    let mut db = make_test_db();
    let result = db.create_object_store("store", None, false);
    assert!(result.is_err());
}

#[test]
fn test_delete_store_not_found() {
    let mut db = make_test_db();
    let result = db.delete_object_store("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_db_add_no_key_no_auto_increment() {
    let mut db = make_test_db();
    let result = db.add("store", json!("value"), None);
    assert!(result.is_err());
}

#[test]
fn test_db_add_duplicate_key() {
    let mut db = make_test_db();
    db.add("store", json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    let result = db.add("store", json!("v2"), Some(IdbKey::Number(1.0)));
    assert!(result.is_err());
}

#[test]
fn test_db_add_success() {
    let mut db = make_test_db();
    let result = db.add("store", json!("value"), Some(IdbKey::Number(1.0)));
    assert!(result.is_ok());
}

#[test]
fn test_db_add_auto_increment() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("ai_store", None, true).unwrap();
    let key = db.add("ai_store", json!("value"), None).unwrap();
    assert_eq!(key, IdbKey::Number(1.0));
}

#[test]
fn test_db_put_no_key_no_auto_increment() {
    let mut db = make_test_db();
    let result = db.put("store", json!("value"), None);
    assert!(result.is_err());
}

#[test]
fn test_db_put_success() {
    let mut db = make_test_db();
    let result = db.put("store", json!("value"), Some(IdbKey::Number(1.0)));
    assert!(result.is_ok());
}

#[test]
fn test_db_put_auto_increment() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("ai_store", None, true).unwrap();
    let key = db.put("ai_store", json!("value"), None).unwrap();
    assert_eq!(key, IdbKey::Number(1.0));
}

#[test]
fn test_db_get() {
    let mut db = make_test_db();
    db.add("store", json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    let record = db.get("store", &IdbKey::Number(1.0));
    assert!(record.is_some());
    let missing = db.get("store", &IdbKey::Number(99.0));
    assert!(missing.is_none());
}

#[test]
fn test_db_delete() {
    let mut db = make_test_db();
    db.add("store", json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    let result = db.delete("store", &IdbKey::Number(1.0));
    assert!(result.is_ok());
    assert!(result.unwrap());
    let result2 = db.delete("store", &IdbKey::Number(99.0));
    assert!(result2.is_ok());
    assert!(!result2.unwrap());
}

#[test]
fn test_db_get_all() {
    let mut db = make_test_db();
    db.add("store", json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    db.add("store", json!("v2"), Some(IdbKey::Number(2.0))).unwrap();
    let all = db.get_all("store").unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn test_db_count() {
    let mut db = make_test_db();
    db.add("store", json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    db.add("store", json!("v2"), Some(IdbKey::Number(2.0))).unwrap();
    assert_eq!(db.count("store").unwrap(), 2);
}

#[test]
fn test_db_clear_store() {
    let mut db = make_test_db();
    db.add("store", json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    db.clear_store("store").unwrap();
    assert_eq!(db.count("store").unwrap(), 0);
}

#[test]
fn test_db_store_names() {
    let db = make_test_db();
    assert!(db.store_names().contains(&"store"));
    assert!(db.has_store("store"));
    assert!(!db.has_store("nonexistent"));
}

// ═══════════════════════════════════════════════════════════════════════
// Index operations
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_create_index_already_exists() {
    let mut db = make_test_db();
    db.create_index("store", "idx", "field", false, false).unwrap();
    let result = db.create_index("store", "idx", "field", false, false);
    assert!(result.is_err());
}

#[test]
fn test_delete_index_not_found() {
    let mut db = make_test_db();
    let result = db.delete_index("store", "nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_index_names() {
    let mut db = make_test_db();
    db.create_index("store", "idx1", "field1", false, false).unwrap();
    db.create_index("store", "idx2", "field2", false, false).unwrap();
    let names = db.index_names("store").unwrap();
    assert_eq!(names.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════
// IdbKey comparison edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_idb_key_compare_different_types() {
    assert_eq!(
        IdbKey::Number(1.0).cmp(&IdbKey::String("a".to_string())),
        std::cmp::Ordering::Less
    );
    assert_eq!(
        IdbKey::String("a".to_string()).cmp(&IdbKey::Binary(vec![1])),
        std::cmp::Ordering::Less
    );
    assert_eq!(
        IdbKey::Binary(vec![1]).cmp(&IdbKey::Array(vec![])),
        std::cmp::Ordering::Less
    );
}

#[test]
fn test_idb_key_array_comparison_different_lengths() {
    let a = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    let b = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Number(2.0)]);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
}

#[test]
fn test_idb_key_binary_comparison() {
    let a = IdbKey::Binary(vec![1, 2]);
    let b = IdbKey::Binary(vec![1, 3]);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
}

// ═══════════════════════════════════════════════════════════════════════
// IdbKeyRange edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_key_range_contains_lower_open() {
    let range = IdbKeyRange::bound(IdbKey::Number(5.0), IdbKey::Number(10.0), true, false);
    assert!(!range.contains(&IdbKey::Number(5.0)));
    assert!(range.contains(&IdbKey::Number(6.0)));
    assert!(range.contains(&IdbKey::Number(10.0)));
}

#[test]
fn test_key_range_contains_upper_open() {
    let range = IdbKeyRange::bound(IdbKey::Number(5.0), IdbKey::Number(10.0), false, true);
    assert!(range.contains(&IdbKey::Number(5.0)));
    assert!(range.contains(&IdbKey::Number(9.0)));
    assert!(!range.contains(&IdbKey::Number(10.0)));
}

#[test]
fn test_key_range_contains_only() {
    let range = IdbKeyRange::only(IdbKey::Number(5.0));
    assert!(range.contains(&IdbKey::Number(5.0)));
    assert!(!range.contains(&IdbKey::Number(6.0)));
}

#[test]
fn test_key_range_accessors() {
    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), true, false);
    assert!(range.lower().is_some());
    assert!(range.upper().is_some());
    assert!(range.lower_open());
    assert!(!range.upper_open());
}

#[test]
fn test_key_range_lower_bound() {
    let range = IdbKeyRange::lower_bound(IdbKey::Number(5.0), false);
    assert!(range.contains(&IdbKey::Number(5.0)));
    assert!(range.contains(&IdbKey::Number(100.0)));
}

#[test]
fn test_key_range_upper_bound() {
    let range = IdbKeyRange::upper_bound(IdbKey::Number(5.0), false);
    assert!(range.contains(&IdbKey::Number(5.0)));
    assert!(range.contains(&IdbKey::Number(1.0)));
    assert!(!range.contains(&IdbKey::Number(10.0)));
}

// ═══════════════════════════════════════════════════════════════════════
// open_cursor_on_index error paths
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_open_cursor_on_index_store_not_found() {
    let db = make_test_db();
    let result = db.open_cursor_on_index("nonexistent", "idx", None);
    assert!(result.is_err());
}

#[test]
fn test_open_cursor_on_index_index_not_found() {
    let db = make_test_db();
    let result = db.open_cursor_on_index("store", "nonexistent", None);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// tx_add / tx_put / tx_delete tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tx_add_no_key_no_auto_increment() {
    let mut db = make_test_db();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    let result = db.tx_add(&tx, "store", json!("value"), None);
    assert!(result.is_err());
}

#[test]
fn test_tx_add_duplicate_key_in_store() {
    let mut db = make_test_db();
    db.add("store", json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    let result = db.tx_add(&tx, "store", json!("v2"), Some(IdbKey::Number(1.0)));
    assert!(result.is_err());
}

#[test]
fn test_tx_add_duplicate_key_in_buffer() {
    let mut db = make_test_db();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(&tx, "store", json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    let result = db.tx_add(&tx, "store", json!("v2"), Some(IdbKey::Number(1.0)));
    assert!(result.is_err());
}

#[test]
fn test_tx_add_success() {
    let mut db = make_test_db();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    let result = db.tx_add(&tx, "store", json!("value"), Some(IdbKey::Number(1.0)));
    assert!(result.is_ok());
}

#[test]
fn test_tx_add_auto_increment() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("ai_store", None, true).unwrap();
    let tx = db.transaction(&["ai_store"], IdbTransactionMode::ReadWrite).unwrap();
    let key = db.tx_add(&tx, "ai_store", json!("value"), None).unwrap();
    assert_eq!(key, IdbKey::Number(1.0));
}

#[test]
fn test_tx_put_no_key_no_auto_increment() {
    let mut db = make_test_db();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    let result = db.tx_put(&tx, "store", json!("value"), None);
    assert!(result.is_err());
}

#[test]
fn test_tx_put_success() {
    let mut db = make_test_db();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    let result = db.tx_put(&tx, "store", json!("value"), Some(IdbKey::Number(1.0)));
    assert!(result.is_ok());
}

#[test]
fn test_tx_delete_existing_key() {
    let mut db = make_test_db();
    db.add("store", json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    let result = db.tx_delete(&tx, "store", &IdbKey::Number(1.0));
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_tx_delete_nonexistent_key() {
    let mut db = make_test_db();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    let result = db.tx_delete(&tx, "store", &IdbKey::Number(99.0));
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

// ═══════════════════════════════════════════════════════════════════════
// tx_get tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tx_get_from_store() {
    let mut db = make_test_db();
    db.add("store", json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadOnly).unwrap();
    let result = db.tx_get(&tx, "store", &IdbKey::Number(1.0)).unwrap();
    assert!(result.is_some());
}

#[test]
fn test_tx_get_from_buffer() {
    let mut db = make_test_db();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_put(&tx, "store", json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    // 从缓冲区获取
    let result = db.tx_get(&tx, "store", &IdbKey::Number(1.0)).unwrap();
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// commit_tx tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_commit_tx_add() {
    let mut db = make_test_db();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(&tx, "store", json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    db.commit_tx(&mut tx).unwrap();
    assert_eq!(db.count("store").unwrap(), 1);
}

#[test]
fn test_commit_tx_put_and_delete() {
    let mut db = make_test_db();
    db.add("store", json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_put(&tx, "store", json!("v1-updated"), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.tx_delete(&tx, "store", &IdbKey::Number(1.0)).unwrap();
    db.commit_tx(&mut tx).unwrap();
    assert_eq!(db.count("store").unwrap(), 0);
}

// ═══════════════════════════════════════════════════════════════════════
// cursor_record / cursor_key
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_cursor_record_and_key() {
    let mut db = make_test_db();
    db.add("store", json!({"field": "value"}), Some(IdbKey::Number(1.0)))
        .unwrap();

    let cursor = IdbCursorWithValue {
        direction: CursorDirection::Next,
        positions: vec![0],
        current: 0,
        store_name: "store".to_string(),
    };
    let record = db.cursor_record(&cursor);
    assert!(record.is_some());

    let key_cursor = IdbCursor {
        direction: CursorDirection::Next,
        keys: vec![IdbKey::Number(1.0)],
        current: 0,
        store_name: "store".to_string(),
    };
    let key = db.cursor_key(&key_cursor);
    assert!(key.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// Transaction store not in scope
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_transaction_store_not_found() {
    let mut db = make_test_db();
    let result = db.transaction(&["nonexistent"], IdbTransactionMode::ReadOnly);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// rename_object_store
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_rename_object_store() {
    let mut db = make_test_db();
    db.rename_object_store("store", "new_store").unwrap();
    assert!(!db.has_store("store"));
    assert!(db.has_store("new_store"));
}

// ═══════════════════════════════════════════════════════════════════════
// IdbRecord creation
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_idb_record_creation() {
    let record = IdbRecord {
        key: IdbKey::String("test".to_string()),
        value: json!({"name": "test"}),
    };
    assert_eq!(record.key, IdbKey::String("test".to_string()));
}

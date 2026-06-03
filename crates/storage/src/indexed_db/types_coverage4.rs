// IDB types.rs 覆盖率补充测试 — 聚焦未覆盖的公共方法。

use super::types::*;
use serde_json::json;

/// Helper: 创建一个带有 store 和数据的数据库。
fn make_db_with_data() -> IdbDatabase {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", Some("id"), false).unwrap();
    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(
        &tx,
        "items",
        json!({"id": 1, "name": "a", "cat": "x"}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();
    db.tx_add(
        &tx,
        "items",
        json!({"id": 2, "name": "b", "cat": "y"}),
        Some(IdbKey::Number(2.0)),
    )
    .unwrap();
    db.tx_add(
        &tx,
        "items",
        json!({"id": 3, "name": "c", "cat": "x"}),
        Some(IdbKey::Number(3.0)),
    )
    .unwrap();
    db.tx_add(
        &tx,
        "items",
        json!({"id": 4, "name": "d", "cat": "z"}),
        Some(IdbKey::Number(4.0)),
    )
    .unwrap();
    let mut tx = tx;
    db.commit_tx(&mut tx).unwrap();
    db
}

#[test]
fn test_rename_object_store_self_rename() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    // 重命名为自身应该成功
    assert!(db.rename_object_store("items", "items").is_ok());
    assert!(db.has_store("items"));
}

#[test]
fn test_rename_object_store_to_existing() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("a", None, false).unwrap();
    db.create_object_store("b", None, false).unwrap();
    // 重命名到已存在的名称应该失败
    assert!(db.rename_object_store("a", "b").is_err());
}

#[test]
fn test_rename_object_store_nonexistent() {
    let mut db = IdbDatabase::new("test", 1);
    assert!(db.rename_object_store("nonexistent", "new_name").is_err());
}

#[test]
fn test_get_all_with_range_empty() {
    let db = make_db_with_data();
    // 空范围（下界 > 上界）
    let range = IdbKeyRange::bound(IdbKey::Number(10.0), IdbKey::Number(1.0), false, false);
    let result = db.get_all_with_range("items", &range).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_count_with_range_various() {
    let db = make_db_with_data();
    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(2.0), false, false);
    let count = db.count_with_range("items", &range).unwrap();
    assert!(count <= 2);
}

#[test]
fn test_get_all_from_index_with_range() {
    let mut db = make_db_with_data();
    db.create_index("items", "cat_idx", "cat", false, false).unwrap();
    let range = IdbKeyRange::only(IdbKey::String("x".to_string()));
    let results = db.get_all_from_index_with_range("items", "cat_idx", &range).unwrap();
    assert!(results.len() >= 1);
}

#[test]
fn test_count_from_index() {
    let mut db = make_db_with_data();
    db.create_index("items", "cat_idx", "cat", false, false).unwrap();
    let count = db.count_from_index("items", "cat_idx", None).unwrap();
    assert!(count >= 1);
    let range = IdbKeyRange::only(IdbKey::String("x".to_string()));
    let count = db.count_from_index("items", "cat_idx", Some(&range)).unwrap();
    assert!(count >= 1);
}

#[test]
fn test_count_from_index_errors() {
    let db = make_db_with_data();
    assert!(db.count_from_index("nonexistent", "idx", None).is_err());
    assert!(db.count_from_index("items", "nonexistent", None).is_err());
}

#[test]
fn test_open_cursor_empty() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("empty", None, false).unwrap();
    let result = db.open_cursor("empty", None).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_open_key_cursor_empty() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("empty", None, false).unwrap();
    let result = db.open_key_cursor("empty", None).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_open_key_cursor_with_data() {
    let db = make_db_with_data();
    let result = db.open_key_cursor("items", None).unwrap();
    assert!(result.is_some());
    let cursor = result.unwrap();
    assert_eq!(cursor.store_name(), "items");
}

#[test]
fn test_open_cursor_with_range() {
    let db = make_db_with_data();
    let range = IdbKeyRange::lower_bound(IdbKey::Number(2.0), false);
    let result = db.open_cursor("items", Some(&range)).unwrap();
    assert!(result.is_some());
}

#[test]
fn test_open_cursor_on_index_empty() {
    let mut db = make_db_with_data();
    db.create_index("items", "cat_idx", "cat", false, false).unwrap();
    let range = IdbKeyRange::only(IdbKey::String("nonexistent".to_string()));
    let result = db.open_cursor_on_index("items", "cat_idx", Some(&range)).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_commit_tx_mixed_operations() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", Some("id"), false).unwrap();

    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(&tx, "items", json!({"id": 1, "v": "a"}), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.tx_add(&tx, "items", json!({"id": 2, "v": "b"}), Some(IdbKey::Number(2.0)))
        .unwrap();
    db.tx_add(&tx, "items", json!({"id": 3, "v": "c"}), Some(IdbKey::Number(3.0)))
        .unwrap();
    let mut tx = tx;
    db.commit_tx(&mut tx).unwrap();

    // 混合操作：put + delete + add
    let tx2 = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_put(
        &tx2,
        "items",
        json!({"id": 1, "v": "updated"}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();
    db.tx_delete(&tx2, "items", &IdbKey::Number(2.0)).unwrap();
    db.tx_add(&tx2, "items", json!({"id": 4, "v": "new"}), Some(IdbKey::Number(4.0)))
        .unwrap();
    let mut tx2 = tx2;
    db.commit_tx(&mut tx2).unwrap();

    assert!(db.get("items", &IdbKey::Number(1.0)).is_some());
    assert!(db.get("items", &IdbKey::Number(2.0)).is_none());
    assert!(db.get("items", &IdbKey::Number(3.0)).is_some());
    assert!(db.get("items", &IdbKey::Number(4.0)).is_some());
}

#[test]
fn test_idb_key_range_bound_open() {
    let range = IdbKeyRange::bound(
        IdbKey::Number(1.0),
        IdbKey::Number(5.0),
        true, // lower open
        true, // upper open
    );
    assert!(!range.contains(&IdbKey::Number(1.0)));
    assert!(!range.contains(&IdbKey::Number(5.0)));
    assert!(range.contains(&IdbKey::Number(3.0)));
    assert!(range.contains(&IdbKey::Number(2.0)));
    assert!(range.contains(&IdbKey::Number(4.0)));
}

#[test]
fn test_idb_key_range_only() {
    let range = IdbKeyRange::only(IdbKey::String("hello".to_string()));
    assert!(range.contains(&IdbKey::String("hello".to_string())));
    assert!(!range.contains(&IdbKey::String("world".to_string())));
    assert!(!range.contains(&IdbKey::Number(1.0)));
}

#[test]
fn test_idb_key_cmp_cross_type() {
    // Number < String < Binary < Array
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
fn test_put_overwrite_existing() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", Some("id"), false).unwrap();

    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(&tx, "items", json!({"name": "first"}), Some(IdbKey::Number(1.0)))
        .unwrap();
    let mut tx = tx;
    db.commit_tx(&mut tx).unwrap();

    let tx2 = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_put(&tx2, "items", json!({"name": "updated"}), Some(IdbKey::Number(1.0)))
        .unwrap();
    let mut tx2 = tx2;
    db.commit_tx(&mut tx2).unwrap();

    let record = db.get("items", &IdbKey::Number(1.0)).unwrap();
    assert_eq!(record.value["name"], "updated");
}

#[test]
fn test_delete_nonexistent_key() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    // 删除不存在的键返回 false
    let result = db.delete("items", &IdbKey::Number(999.0)).unwrap();
    assert!(!result);
}

#[test]
fn test_transaction_readonly() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    let tx = db.transaction(&["items"], IdbTransactionMode::ReadOnly).unwrap();
    assert_eq!(tx.mode(), IdbTransactionMode::ReadOnly);
}

#[test]
fn test_transaction_nonexistent_store() {
    let mut db = IdbDatabase::new("test", 1);
    let result = db.transaction(&["nonexistent"], IdbTransactionMode::ReadOnly);
    assert!(result.is_err());
}

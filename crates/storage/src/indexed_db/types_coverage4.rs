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
    // Number < Date < String < Binary < Array
    assert_eq!(IdbKey::Number(1.0).cmp(&IdbKey::Date(0.0)), std::cmp::Ordering::Less);
    assert_eq!(
        IdbKey::Date(0.0).cmp(&IdbKey::String("a".to_string())),
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
fn test_idb_date_key_validity_hash_and_lookup() {
    use std::collections::HashSet;

    assert!(IdbKey::Date(0.0).is_valid_key());
    assert!(!IdbKey::Date(f64::NAN).is_valid_key());
    assert!(!IdbKey::Date(f64::INFINITY).is_valid_key());

    let mut keys = HashSet::new();
    keys.insert(IdbKey::Date(-0.0));
    keys.insert(IdbKey::Date(0.0));
    assert_eq!(keys.len(), 1);

    let mut db = IdbDatabase::new("dates", 1);
    db.create_object_store("items", None, false).unwrap();
    db.add("items", json!("number"), Some(IdbKey::Number(10.0))).unwrap();
    db.add("items", json!("date"), Some(IdbKey::Date(10.0))).unwrap();
    assert_eq!(db.get("items", &IdbKey::Date(10.0)).unwrap().value, "date");
    assert_eq!(db.get("items", &IdbKey::Number(10.0)).unwrap().value, "number");
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

#[test]
fn test_cursor_position_and_finished() {
    let db = make_db_with_data();
    let cursor = db.open_cursor("items", None).unwrap().unwrap();
    assert!(!cursor.is_finished());
    assert_eq!(cursor.position(), 0);
    assert_eq!(cursor.store_name(), "items");
}

#[test]
fn test_idb_cursor_key() {
    let db = make_db_with_data();
    // Open a key cursor and check the key
    let cursor = db.open_key_cursor("items", None).unwrap().unwrap();
    assert!(cursor.key().is_some());
}

#[test]
fn test_idb_cursor_value() {
    let db = make_db_with_data();
    let cursor = db.open_cursor("items", None).unwrap().unwrap();
    let val = db.cursor_record(&cursor);
    assert!(val.is_some());
}

#[test]
fn test_key_cursor_advance() {
    let db = make_db_with_data();
    let mut cursor = db.open_key_cursor("items", None).unwrap().unwrap();
    assert!(cursor.key().is_some());
    assert!(cursor.advance(1));
}

// ── 覆盖 types.rs 剩余未覆盖行 ──

/// 覆盖 line 235: extract_keys 中 key_path 不存在于 JSON 值时返回空 Vec
#[test]
fn test_extract_keys_missing_key_path() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", Some("id"), false).unwrap();

    // 创建索引，key_path 为 "tags"（JSON 中不存在此字段）
    db.create_index("items", "tag_idx", "tags", false, false).unwrap();

    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    // 添加一个不含 "tags" 字段的记录 — extract_keys 返回空 Vec（line 235）
    db.tx_add(&tx, "items", json!({"id": 1, "name": "a"}), Some(IdbKey::Number(1.0)))
        .unwrap();
    let mut tx = tx;
    db.commit_tx(&mut tx).unwrap();

    // 记录存在但索引中无匹配
    let record = db.get("items", &IdbKey::Number(1.0)).unwrap();
    assert_eq!(record.value["name"], "a");

    // 从索引查询应返回空（因为 key_path 不匹配，没有索引条目）
    let idx_result = db
        .get_from_index("items", "tag_idx", &IdbKey::String("nonexistent".to_string()))
        .unwrap();
    assert!(idx_result.is_empty());
}

/// 覆盖 line 542: add_entry_from_record 在新增记录（非覆盖）时被调用
#[test]
fn test_add_record_with_index_triggers_add_entry() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", Some("id"), false).unwrap();

    // 创建索引
    db.create_index("items", "cat_idx", "cat", false, false).unwrap();

    // 添加第一条记录 — 走 else 分支（新增），触发 line 542
    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(&tx, "items", json!({"id": 1, "cat": "x"}), Some(IdbKey::Number(1.0)))
        .unwrap();
    let mut tx = tx;
    db.commit_tx(&mut tx).unwrap();

    // 添加第二条记录 — 同样走 else 分支
    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(&tx, "items", json!({"id": 2, "cat": "y"}), Some(IdbKey::Number(2.0)))
        .unwrap();
    let mut tx = tx;
    db.commit_tx(&mut tx).unwrap();

    // 验证索引正常工作 — 通过 get_from_index 查询
    let result = db
        .get_from_index("items", "cat_idx", &IdbKey::String("x".to_string()))
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].value["id"], 1);
}

/// 覆盖 lines 996-999: tx_put 中 auto_increment 为 true 且 key 为 None
#[test]
fn test_tx_put_auto_increment() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, true).unwrap(); // auto_increment = true

    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    // key 为 None，auto_increment = true → 走 lines 996-999
    let key = db.tx_put(&tx, "items", json!({"name": "auto"}), None).unwrap();
    assert_eq!(key, IdbKey::Number(1.0)); // 第一个自动生成 key = 1

    let mut tx = tx;
    db.commit_tx(&mut tx).unwrap();

    // 验证记录已存储
    let record = db.get("items", &IdbKey::Number(1.0)).unwrap();
    assert_eq!(record.value["name"], "auto");

    // 再添加一条，key 应该自增为 2
    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    let key2 = db.tx_put(&tx, "items", json!({"name": "auto2"}), None).unwrap();
    assert_eq!(key2, IdbKey::Number(2.0));
}

/// 覆盖 line 1027: tx_delete 中匹配 TxMutation::Delete 等其他 variant 的 _ => false
#[test]
fn test_tx_delete_with_prior_delete_mutation() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", Some("id"), false).unwrap();

    // 先添加一条记录
    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(&tx, "items", json!({"id": 1}), Some(IdbKey::Number(1.0)))
        .unwrap();
    let mut tx = tx;
    db.commit_tx(&mut tx).unwrap();

    // 在同一事务中先 put 一条，再 delete — 确保 _ => false (line 1027) 被覆盖
    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_put(&tx, "items", json!({"id": 2}), Some(IdbKey::Number(2.0)))
        .unwrap();
    // delete 会遍历 mutations，遇到 Put 匹配成功，然后添加 Delete mutation
    let found1 = db.tx_delete(&tx, "items", &IdbKey::Number(2.0)).unwrap();
    assert!(found1); // put 在 buffer 中存在

    // 再次 delete 同一个 key — 此时 mutations 里有 Put 和 Delete，
    // 遍历时 Delete variant 命中 _ => false (line 1027)
    let found2 = db.tx_delete(&tx, "items", &IdbKey::Number(2.0)).unwrap();
    // 第二次 delete：mutations 中有 Put (match), Delete (_ => false)
    // exists_in_buffer = true (因为 Put 匹配到了)
    assert!(found2);
}

/// 覆盖 line 1063: tx_get 中匹配不相关的 mutation（_ => {} 跳过）
#[test]
fn test_tx_get_skips_unrelated_mutations() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", Some("id"), false).unwrap();

    // 添加两条记录
    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(&tx, "items", json!({"id": 1, "val": "a"}), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.tx_add(&tx, "items", json!({"id": 2, "val": "b"}), Some(IdbKey::Number(2.0)))
        .unwrap();
    let mut tx = tx;
    db.commit_tx(&mut tx).unwrap();

    // 新事务：put key=1，然后 get key=2
    // tx_get 遍历 mutations 时遇到 Put{key=1}（不匹配 key=2），走 _ => {} (line 1063)
    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_put(
        &tx,
        "items",
        json!({"id": 1, "val": "updated"}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();

    // get key=2 会遍历 mutations，Put{key=1} 不匹配，走 _ => {} 跳过
    let result = db.tx_get(&tx, "items", &IdbKey::Number(2.0)).unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().value["val"], "b");
}

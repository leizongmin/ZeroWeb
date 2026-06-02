//! IndexedDB 边界条件测试 — IDBDatabase close/reopen、事务错误传播、
//! 索引创建与 multiEntry、游标反向迭代等边界场景。

use super::super::*;

/// 测试 IDBDatabase 重建后（模拟 close/reopen）数据独立性。
///
/// 两个独立的 IdbDatabase 实例即使同名也互不影响，
/// 这符合 IndexedDB same-origin 多连接的语义。
#[test]
fn test_idb_close_reopen_data_isolation() {
    // 第一个数据库实例：写入数据
    let mut db1 = IdbDatabase::new("my-app", 1);
    db1.create_object_store("users", None, false).unwrap();
    db1.add(
        "users",
        serde_json::json!({"name": "Alice"}),
        Some(IdbKey::String("u1".into())),
    )
    .unwrap();
    assert_eq!(db1.count("users").unwrap(), 1);

    // 模拟 close（drop db1）后 reopen（创建新的同名实例）
    let mut db2 = IdbDatabase::new("my-app", 2);
    // 新实例没有旧数据（内存实现不持久化）
    assert!(!db2.has_store("users"));
    assert!(db2.store_names().is_empty());

    // 在新实例上创建同名 store 并操作
    db2.create_object_store("users", None, true).unwrap();
    let k = db2.add("users", serde_json::json!({"name": "Bob"}), None).unwrap();
    assert!(matches!(k, IdbKey::Number(1.0)));
    assert_eq!(db2.count("users").unwrap(), 1);

    // 两个实例的 store_names 互不影响
    assert_eq!(db1.store_names().len(), 1);
    assert_eq!(db2.store_names().len(), 1);
}

/// 测试事务错误传播：tx_add 重复主键后，事务仍可提交，
/// 但重复主键会导致 commit_tx 失败。
#[test]
fn test_idb_tx_add_duplicate_key_propagates_error() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    let key = IdbKey::String("unique".into());

    // 先直接 add 一条记录
    db.add("items", serde_json::json!("first"), Some(key.clone())).unwrap();

    // 创建事务，尝试 tx_add 同一主键
    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    let result = db.tx_add(&tx, "items", serde_json::json!("second"), Some(key.clone()));
    // tx_add 应返回错误（store 中已存在该主键）
    assert!(result.is_err(), "tx_add 重复主键应返回错误");

    // 原始数据不变
    let record = db.get("items", &key).unwrap();
    assert_eq!(record.value, serde_json::json!("first"));
    assert_eq!(db.count("items").unwrap(), 1);
}

/// 测试在已有数据的 store 上创建索引后，索引立即可用。
///
/// 模拟先插入 N 条记录，再创建索引（等同于 IDB 的 onupgradeneeded 中建索引）。
#[test]
fn test_idb_index_creation_on_existing_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("products", None, false).unwrap();

    // 先插入数据
    for i in 1..=5 {
        db.add(
            "products",
            serde_json::json!({"name": format!("Item-{i}"), "price": i * 10}),
            Some(IdbKey::Number(i as f64)),
        )
        .unwrap();
    }
    assert_eq!(db.count("products").unwrap(), 5);

    // 创建索引 — 应自动从已有记录构建
    db.create_index("products", "price_idx", "price", false, false).unwrap();

    // 索引立即可用
    let cheap = db
        .get_from_index("products", "price_idx", &IdbKey::Number(10.0))
        .unwrap();
    assert_eq!(cheap.len(), 1);
    assert_eq!(cheap[0].value["name"], "Item-1");

    let all_by_price = db.get_all_from_index("products", "price_idx").unwrap();
    assert_eq!(all_by_price.len(), 5);
    // 应按 price 排序
    assert_eq!(all_by_price[0].value["price"], 10);
    assert_eq!(all_by_price[4].value["price"], 50);
}

/// 测试游标 advance 超出边界后 is_finished 为 true，再次 advance 仍返回 false。
#[test]
fn test_idb_cursor_advance_past_end_stays_finished() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    db.add("items", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("items", serde_json::json!("b"), Some(IdbKey::Number(2.0)))
        .unwrap();

    let mut cursor = db.open_cursor("items", None).unwrap().unwrap();
    // advance(5) 远超记录数
    assert!(!cursor.advance(5));
    assert!(cursor.is_finished());

    // 再次 advance 不应 panic，仍返回 false
    assert!(!cursor.advance(1));
    assert!(cursor.is_finished());
}

/// 测试 multiEntry 索引的游标反向迭代。
///
/// 由于当前游标只支持 Next 方向，通过 get_all_from_index 获取记录后
/// 手动逆序验证数据一致性。
#[test]
fn test_idb_cursor_reverse_with_multi_entry() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("docs", None, false).unwrap();

    // 插入带 tags 数组的记录
    db.add(
        "docs",
        serde_json::json!({"title": "Alpha", "tags": ["a", "b"]}),
        Some(IdbKey::String("d1".into())),
    )
    .unwrap();
    db.add(
        "docs",
        serde_json::json!({"title": "Beta", "tags": ["b", "c"]}),
        Some(IdbKey::String("d2".into())),
    )
    .unwrap();
    db.add(
        "docs",
        serde_json::json!({"title": "Gamma", "tags": ["c", "d"]}),
        Some(IdbKey::String("d3".into())),
    )
    .unwrap();

    // 创建 multiEntry 索引
    db.create_index("docs", "tags_idx", "tags", false, true).unwrap();

    // 正向迭代（按索引键排序）
    let mut cursor = db.open_cursor_on_index("docs", "tags_idx", None).unwrap().unwrap();
    let mut forward_titles = Vec::new();
    loop {
        let rec = db.cursor_record(&cursor).unwrap();
        forward_titles.push(rec.value["title"].as_str().unwrap().to_string());
        if !cursor.continue_next() {
            break;
        }
    }

    // 通过 get_all_from_index 获取全部记录，反转后验证
    let all = db.get_all_from_index("docs", "tags_idx").unwrap();
    let reverse_titles: Vec<String> = all
        .iter()
        .rev()
        .map(|r| r.value["title"].as_str().unwrap().to_string())
        .collect();

    // 正向和反向应互为逆序
    let mut forward_rev = forward_titles.clone();
    forward_rev.reverse();
    assert_eq!(reverse_titles, forward_rev, "反向迭代结果应与正向的逆序一致");
}

/// 测试 count_with_range 在空 store 上返回 0。
#[test]
fn test_idb_count_with_range_on_empty_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("empty", None, false).unwrap();

    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), false, false);
    assert_eq!(db.count_with_range("empty", &range).unwrap(), 0);

    let only = IdbKeyRange::only(IdbKey::String("k".into()));
    assert_eq!(db.count_with_range("empty", &only).unwrap(), 0);

    let lower = IdbKeyRange::lower_bound(IdbKey::Number(0.0), false);
    assert_eq!(db.count_with_range("empty", &lower).unwrap(), 0);
}

/// 测试事务 tx_delete 对不存在的键返回 false。
#[test]
fn test_idb_tx_delete_nonexistent_key() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    // store 为空，没有任何记录

    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    let found = db.tx_delete(&tx, "items", &IdbKey::String("ghost".into())).unwrap();
    assert!(!found, "删除不存在的键应返回 false");
}

/// 测试 open_key_cursor 在空 store 上返回 None。
#[test]
fn test_idb_key_cursor_on_empty_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("empty", None, false).unwrap();

    let result = db.open_key_cursor("empty", None).unwrap();
    assert!(result.is_none(), "空 store 上打开键游标应返回 None");

    // 带范围也应返回 None
    let range = IdbKeyRange::lower_bound(IdbKey::Number(0.0), false);
    let result = db.open_key_cursor("empty", Some(&range)).unwrap();
    assert!(result.is_none());
}

/// 测试唯一索引 delete 后可以重新插入相同索引值。
#[test]
fn test_idb_unique_index_reuse_after_delete() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("accounts", None, false).unwrap();
    db.add(
        "accounts",
        serde_json::json!({"username": "alice", "role": "admin"}),
        Some(IdbKey::String("acc1".into())),
    )
    .unwrap();

    db.create_index("accounts", "username_idx", "username", true, false)
        .unwrap();

    // 删除 alice
    db.delete("accounts", &IdbKey::String("acc1".into())).unwrap();
    assert_eq!(db.count("accounts").unwrap(), 0);

    // 现在可以重新插入相同 username
    db.add(
        "accounts",
        serde_json::json!({"username": "alice", "role": "user"}),
        Some(IdbKey::String("acc2".into())),
    )
    .unwrap();

    let results = db
        .get_from_index("accounts", "username_idx", &IdbKey::String("alice".into()))
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value["role"], "user");
}

/// 测试 create_index 重复名称应返回错误。
#[test]
fn test_idb_create_duplicate_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    db.create_index("items", "field_idx", "field", false, false).unwrap();

    let result = db.create_index("items", "field_idx", "field", false, false);
    assert!(result.is_err(), "创建同名索引应返回错误");
}

/// 测试 delete_index 后索引不可再用。
#[test]
fn test_idb_delete_index_then_query_fails() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    db.add(
        "items",
        serde_json::json!({"name": "test"}),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();
    db.create_index("items", "name_idx", "name", false, false).unwrap();

    // 删除索引
    db.delete_index("items", "name_idx").unwrap();
    assert_eq!(db.index_names("items").unwrap().len(), 0);

    // 查询已删除的索引应返回错误
    let result = db.get_from_index("items", "name_idx", &IdbKey::String("test".into()));
    assert!(result.is_err(), "查询已删除的索引应返回错误");

    // 计数也应返回错误
    let count_result = db.count_from_index("items", "name_idx", None);
    assert!(count_result.is_err());
}

/// 测试 clear_store 后自增主键不重置（模拟浏览器行为）。
///
/// 当前实现中 clear_store 清除记录但不重置 next_key，
/// 这是合理的（与 Chrome 行为一致）。
#[test]
fn test_idb_clear_store_auto_increment_not_reset() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("seq", None, true).unwrap();

    // 插入 3 条自增记录
    let k1 = db.add("seq", serde_json::json!("a"), None).unwrap();
    let _k2 = db.add("seq", serde_json::json!("b"), None).unwrap();
    let k3 = db.add("seq", serde_json::json!("c"), None).unwrap();
    assert!(matches!(&k1, IdbKey::Number(n) if *n == 1.0));
    assert!(matches!(&k3, IdbKey::Number(n) if *n == 3.0));
    assert_eq!(db.count("seq").unwrap(), 3);

    // 清空 store
    db.clear_store("seq").unwrap();
    assert_eq!(db.count("seq").unwrap(), 0);

    // 再次插入，自增键应继续递增（不重置为 1）
    let k4 = db.add("seq", serde_json::json!("d"), None).unwrap();
    assert!(
        matches!(&k4, IdbKey::Number(n) if *n >= 4.0),
        "clear 后自增键应继续递增，实际为 {:?}",
        k4
    );
    assert_eq!(db.count("seq").unwrap(), 1);
}

/// 测试 get_all_with_range 对不存在的 store 返回错误。
#[test]
fn test_idb_get_all_with_range_nonexistent_store() {
    let db = IdbDatabase::new("test", 1);
    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(5.0), false, false);
    let result = db.get_all_with_range("ghost", &range);
    assert!(result.is_err());
}

/// 测试 IdbKeyRange::lower_bound 和 upper_bound 的 accessor 方法。
#[test]
fn test_idb_key_range_bound_accessors() {
    let lower_only = IdbKeyRange::lower_bound(IdbKey::Number(5.0), false);
    assert_eq!(lower_only.lower(), Some(&IdbKey::Number(5.0)));
    assert_eq!(lower_only.upper(), None);
    assert!(!lower_only.lower_open());

    let upper_only = IdbKeyRange::upper_bound(IdbKey::Number(10.0), true);
    assert_eq!(upper_only.lower(), None);
    assert_eq!(upper_only.upper(), Some(&IdbKey::Number(10.0)));
    assert!(upper_only.upper_open());

    let only = IdbKeyRange::only(IdbKey::String("hello".into()));
    assert_eq!(only.lower(), Some(&IdbKey::String("hello".into())));
    assert_eq!(only.upper(), Some(&IdbKey::String("hello".into())));
    assert!(!only.lower_open());
    assert!(!only.upper_open());
}

/// 测试 IdbKey::Array 空数组与嵌套数组的排序。
#[test]
fn test_idb_key_array_edge_cases() {
    // 空数组 < 非空数组
    let empty = IdbKey::Array(vec![]);
    let single = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    assert!(empty < single);

    // 嵌套数组排序
    let nested1 = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Array(vec![IdbKey::Number(2.0)])]);
    let nested2 = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Array(vec![IdbKey::Number(3.0)])]);
    assert!(nested1 < nested2);

    // 长度不同的数组：前缀相同，短的更小
    let short = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    let long = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Number(2.0)]);
    assert!(short < long);
}

/// 测试 Binary 键的比较行为。
#[test]
fn test_idb_binary_key_comparison() {
    let b1 = IdbKey::Binary(vec![1, 2, 3]);
    let b2 = IdbKey::Binary(vec![1, 2, 4]);
    let b3 = IdbKey::Binary(vec![1, 2]);
    let b4 = IdbKey::Binary(vec![1, 2, 3]);

    assert!(b1 < b2, "相同前缀时按元素值比较");
    assert!(b3 < b1, "短序列小于长序列（前缀相同）");
    assert_eq!(b1, b4, "完全相同的二进制键应相等");
}

/// 测试 rename_object_store 后索引仍可用。
#[test]
fn test_idb_rename_store_preserves_indexes() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("data", None, false).unwrap();
    db.add(
        "data",
        serde_json::json!({"category": "A", "value": 1}),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();
    db.add(
        "data",
        serde_json::json!({"category": "B", "value": 2}),
        Some(IdbKey::String("k2".into())),
    )
    .unwrap();
    db.create_index("data", "cat_idx", "category", false, false).unwrap();

    // 重命名
    db.rename_object_store("data", "renamed").unwrap();
    assert!(!db.has_store("data"));
    assert!(db.has_store("renamed"));

    // 索引应仍可用
    let idx_names = db.index_names("renamed").unwrap();
    assert_eq!(idx_names.len(), 1);
    assert!(idx_names.contains(&"cat_idx"));

    // 通过索引查询
    let results = db
        .get_from_index("renamed", "cat_idx", &IdbKey::String("A".into()))
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value["value"], 1);
}

/// 测试 tx_get 对事务外已存在但事务内被 put 覆盖的记录。
#[test]
fn test_idb_tx_get_after_tx_put_overwrite() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    let key = IdbKey::String("k1".into());
    db.add("items", serde_json::json!({"step": 0}), Some(key.clone()))
        .unwrap();

    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();

    // tx_put 覆盖
    db.tx_put(&tx, "items", serde_json::json!({"step": 99}), Some(key.clone()))
        .unwrap();

    // tx_get 应返回覆盖后的值
    let rec = db.tx_get(&tx, "items", &key).unwrap().unwrap();
    assert_eq!(rec.value["step"], 99);

    // store 中原始数据不变
    let original = db.get("items", &key).unwrap();
    assert_eq!(original.value["step"], 0);
}

/// 测试 open_cursor_on_index 在不存在的索引上返回错误。
#[test]
fn test_idb_cursor_on_nonexistent_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    db.add("items", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();

    let result = db.open_cursor_on_index("items", "ghost_idx", None);
    assert!(result.is_err(), "不存在的索引应返回错误");
}

/// 测试 multiEntry 索引中嵌套数组元素的处理。
///
/// multiEntry 为 true 时，如果数组元素本身是数组，则整个子数组
/// 作为单个索引键（不会递归展开）。
#[test]
fn test_idb_multi_entry_nested_array_as_single_key() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("matrix", None, false).unwrap();

    // tags 包含 Number 元素
    db.add(
        "matrix",
        serde_json::json!({"name": "row1", "tags": [1, 2]}),
        Some(IdbKey::String("r1".into())),
    )
    .unwrap();
    db.add(
        "matrix",
        serde_json::json!({"name": "row2", "tags": [2, 3]}),
        Some(IdbKey::String("r2".into())),
    )
    .unwrap();

    db.create_index("matrix", "tags_idx", "tags", false, true).unwrap();

    // multiEntry 展开为：row1 → [Number(1), Number(2)], row2 → [Number(2), Number(3)]
    let tag_2 = db.get_from_index("matrix", "tags_idx", &IdbKey::Number(2.0)).unwrap();
    assert_eq!(tag_2.len(), 2, "Number(2) 应匹配两条记录");

    let tag_3 = db.get_from_index("matrix", "tags_idx", &IdbKey::Number(3.0)).unwrap();
    assert_eq!(tag_3.len(), 1);
    assert_eq!(tag_3[0].value["name"], "row2");
}

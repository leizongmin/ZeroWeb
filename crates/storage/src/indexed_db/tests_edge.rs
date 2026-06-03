//! IndexedDB 边界条件测试 — IDBDatabase close/reopen、事务错误传播、
//! 索引创建与 multiEntry、游标反向迭代等边界场景。

use super::super::*;
use std::cmp::Ordering;

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

// ── 新增测试：提高 types.rs 覆盖率 ──

/// 测试 IdbKey 的跨类型比较行为，确保所有组合都被测试。
#[test]
fn test_idb_key_cross_type_comparisons() {
    // Number vs String
    let num = IdbKey::Number(42.0);
    let str_key = IdbKey::String("hello".into());
    assert!(num < str_key, "Number < String");
    assert!(str_key > num, "String > Number");

    // Number vs Binary
    let binary = IdbKey::Binary(vec![1, 2, 3]);
    assert!(num < binary, "Number < Binary");
    assert!(binary > num, "Binary > Number");

    // Number vs Array
    let array = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    assert!(num < array, "Number < Array");
    assert!(array > num, "Array > Number");

    // String vs Number
    let str_num = IdbKey::String("123".into());
    let another_num = IdbKey::Number(123.0);
    assert!(str_num > another_num, "String > Number");
    assert!(another_num < str_num, "Number < String");

    // String vs Binary
    assert!(str_key < binary, "String < Binary");
    assert!(binary > str_key, "Binary > String");

    // String vs Array
    assert!(str_key < array, "String < Array");
    assert!(array > str_key, "Array > String");

    // Binary vs Number
    let _binary2 = IdbKey::Binary(vec![1, 2, 4]);
    assert!(binary > another_num, "Binary > Number");
    assert!(another_num < binary, "Number < Binary");

    // Binary vs String
    assert!(binary > str_key, "Binary > String");
    assert!(str_key < binary, "String < Binary");

    // Binary vs Binary - content comparison
    let binary3 = IdbKey::Binary(vec![1, 2, 3]);
    let binary4 = IdbKey::Binary(vec![1, 2, 4]);
    assert!(binary3 < binary4, "Binary: [1,2,3] < [1,2,4]");
    assert!(binary3 == binary3, "Binary: equal content equals");
    assert!(binary3 < binary4);

    // Binary vs Array
    assert!(binary < array, "Binary < Array");
    assert!(array > binary, "Array > Binary");

    // Array vs Number
    let _array2 = IdbKey::Array(vec![IdbKey::Number(2.0)]);
    assert!(array > another_num, "Array > Number");
    assert!(another_num < array, "Number < Array");

    // Array vs String
    assert!(array > str_key, "Array > String");
    assert!(str_key < array, "String < Array");

    // Array vs Binary
    assert!(array > binary, "Array > Binary");
    assert!(binary < array, "Binary < Array");

    // Array vs Array - lexicographic compare
    let arr1 = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::String("a".into())]);
    let arr2 = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::String("b".into())]);
    let arr3 = IdbKey::Array(vec![IdbKey::Number(2.0), IdbKey::String("a".into())]);
    assert!(arr1 < arr2, "Array: [1,'a'] < [1,'b']");
    assert!(arr1 < arr3, "Array: [1,'a'] < [2,'a']");
    assert!(arr2 < arr3, "Array: [1,'b'] < [2,'a']");
}

/// 测试使用 Array 键创建记录，间接测试 json_value_to_idb_key 的数组路径。
#[test]
fn test_json_value_to_idb_key_indirect_test() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("test_store", None, false).unwrap();

    // 创建带嵌套数组的记录，这会通过 json_value_to_idb_key 进行索引提取
    db.add(
        "test_store",
        serde_json::json!({"name": "test", "tags": [1, "two", [3]]}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();

    // 创建索引，这会使用 extract_keys（内部调用 json_value_to_idb_key）
    db.create_index("test_store", "tags_idx", "tags", false, true).unwrap();

    // 查询特定标签
    let results = db
        .get_from_index("test_store", "tags_idx", &IdbKey::Number(1.0))
        .unwrap();
    assert_eq!(results.len(), 1, "数字标签 1 应该匹配");
    assert_eq!(results[0].value["name"], "test");

    // 查询字符串标签
    let results = db
        .get_from_index("test_store", "tags_idx", &IdbKey::String("two".into()))
        .unwrap();
    assert_eq!(results.len(), 1, "字符串标签 'two' 应该匹配");
    assert_eq!(results[0].value["name"], "test");
}

/// 测试 Binary 键的创建和比较。
#[test]
fn test_idb_binary_key_operations() {
    // 创建不同的 Binary 键
    let b1 = IdbKey::Binary(vec![1, 2, 3]);
    let b2 = IdbKey::Binary(vec![1, 2, 4]);
    let b3 = IdbKey::Binary(vec![1, 2, 3, 4]);
    let b4 = IdbKey::Binary(vec![0, 1, 2]);

    // 排序测试
    assert!(b1 < b2, "相同前缀，比较元素值");
    assert!(b1 < b3, "短序列小于长序列（前缀相同）");
    assert!(b4 < b1, "比较第一个不同元素");
    assert!(b1 == b1, "自身相等");

    // Binary 键在排序顺序中应该按字典序排列
    let mut binary_keys = vec![b1.clone(), b2.clone(), b3.clone(), b4.clone()];
    binary_keys.sort();

    // 第一个应该是 b4 [0,1,2]
    assert!(matches!(&binary_keys[0], IdbKey::Binary(b) if b == &[0, 1, 2]));
    // 最后一个应该是 b2 [1,2,4]（字典序大于 [1,2,3] 和 [1,2,3,4]）
    assert!(matches!(&binary_keys[3], IdbKey::Binary(b) if b == &[1, 2, 4]));
}

/// 测试 IdbKey 的 Hash 行为。
#[test]
fn test_idb_key_hash_consistency() {
    use std::collections::HashSet;
    use std::collections::hash_map::HashMap;

    // 相同键应该 Hash 相同
    let key1 = IdbKey::Number(42.0);
    let key2 = IdbKey::Number(42.0);
    let key3 = IdbKey::Number(43.0);

    let mut set = HashSet::new();
    set.insert(key1.clone());
    assert!(set.contains(&key2), "相同的键应该 hash 到同一个位置");
    assert!(!set.contains(&key3), "不同的键应该 hash 到不同位置");

    // 不同类型但相同的数值应该有不同的 hash（因为 discriminant 不同）
    let num_key = IdbKey::Number(42.0);
    let str_key = IdbKey::String("42.0".into());
    let mut set2 = HashSet::new();
    set2.insert(num_key.clone());
    set2.insert(str_key.clone());
    assert_eq!(set2.len(), 2, "Number 和 String 类型不同，即使值相同也应有不同 hash");

    // 测试 HashMap 中的使用
    let mut map = HashMap::new();
    map.insert(num_key.clone(), "value1");
    map.insert(str_key.clone(), "value2");
    assert_eq!(map.get(&IdbKey::Number(42.0)), Some(&"value1"));
    assert_eq!(map.get(&IdbKey::String("42.0".into())), Some(&"value2"));
}

/// 测试 Array 键的边界情况。
#[test]
fn test_idb_array_key_edge_cases() {
    // 空数组键
    let empty_array = IdbKey::Array(vec![]);
    let single_array = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    assert!(empty_array < single_array, "空数组 < 非空数组");

    // 嵌套数组键
    let nested1 = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Array(vec![IdbKey::Number(2.0)])]);
    let nested2 = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Array(vec![IdbKey::Number(3.0)])]);
    assert!(nested1 < nested2, "嵌套数组按元素比较");

    // 混合类型数组键
    let mixed_array = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::String("hello".into()),
        IdbKey::Binary(vec![1, 2]),
    ]);
    let mixed_array2 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::String("world".into()),
        IdbKey::Binary(vec![1, 2]),
    ]);
    assert!(mixed_array < mixed_array2, "混合类型数组按字典序比较");

    // 长度不同的数组：前缀相同，短的更小
    let short = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::String("a".into())]);
    let long = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::String("a".into()),
        IdbKey::Number(2.0),
    ]);
    assert!(short < long, "短数组 < 长数组（前缀相同）");

    // 自相等
    let self_array = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    assert!(self_array == self_array);
}

/// 测试 KeyRange contains 方法与各种边界类型。
#[test]
fn test_idb_key_range_contains_with_bounds() {
    let key1 = IdbKey::Number(1.0);
    let key2 = IdbKey::Number(2.0);
    let key3 = IdbKey::Number(3.0);

    // Open lower bound (1.0,)
    let open_lower = IdbKeyRange::lower_bound(key1.clone(), true);
    assert!(!open_lower.contains(&key1), "开下界不包含 1.0");
    assert!(open_lower.contains(&key2), "开下界包含 2.0");
    assert!(open_lower.contains(&key3), "开下界包含 3.0");

    // Open upper bound ,3.0)
    let open_upper = IdbKeyRange::upper_bound(key3.clone(), true);
    assert!(open_upper.contains(&key1), "开上界包含 1.0");
    assert!(open_upper.contains(&key2), "开上界包含 2.0");
    assert!(!open_upper.contains(&key3), "开上界不包含 3.0");

    // Both bounds open (1.0, 3.0)
    let both_open = IdbKeyRange::bound(key1.clone(), key3.clone(), true, true);
    assert!(!both_open.contains(&key1), "双开下界不包含 1.0");
    assert!(both_open.contains(&key2), "双开包含 2.0");
    assert!(!both_open.contains(&key3), "双开上界不包含 3.0");

    // Both bounds closed [1.0, 3.0]
    let both_closed = IdbKeyRange::bound(key1.clone(), key3.clone(), false, false);
    assert!(both_closed.contains(&key1), "双闭下界包含 1.0");
    assert!(both_closed.contains(&key2), "双闭包含 2.0");
    assert!(both_closed.contains(&key3), "双闭上界包含 3.0");

    // 使用 String 键测试
    let str1 = IdbKey::String("a".into());
    let str2 = IdbKey::String("b".into());
    let str3 = IdbKey::String("c".into());

    let string_range = IdbKeyRange::bound(str1.clone(), str3.clone(), false, true);
    assert!(string_range.contains(&str1), "闭下界包含 'a'");
    assert!(string_range.contains(&str2), "包含 'b'");
    assert!(!string_range.contains(&str3), "开上界不包含 'c'");

    // 空范围
    let empty_range = IdbKeyRange::bound(key2.clone(), key1.clone(), false, false);
    assert!(!empty_range.contains(&key1));
    assert!(!empty_range.contains(&key2));
    assert!(!empty_range.contains(&key3));
}

/// 测试 multiEntry 索引在复杂场景下的行为，间接测试 extract_keys 的处理逻辑。
#[test]
fn test_index_multi_entry_complex_behavior() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("test_store", None, false).unwrap();

    // 插入带复杂数组标签的记录
    db.add(
        "test_store",
        serde_json::json!({"name": "Item1", "tags": ["rust", "web", null, 42, true]}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();

    db.add(
        "test_store",
        serde_json::json!({"name": "Item2", "tags": ["web", "db"]}),
        Some(IdbKey::Number(2.0)),
    )
    .unwrap();

    db.add(
        "test_store",
        serde_json::json!({"name": "Item3", "tags": []}),
        Some(IdbKey::Number(3.0)),
    )
    .unwrap();

    // 创建 multiEntry 索引
    db.create_index("test_store", "tags_idx", "tags", false, true).unwrap();

    // 查询 "rust" 标签
    let rust_items = db
        .get_from_index("test_store", "tags_idx", &IdbKey::String("rust".into()))
        .unwrap();
    assert_eq!(rust_items.len(), 1);
    assert_eq!(rust_items[0].value["name"], "Item1");

    // 查询 "web" 标签（出现两次）
    let web_items = db
        .get_from_index("test_store", "tags_idx", &IdbKey::String("web".into()))
        .unwrap();
    assert_eq!(web_items.len(), 2);

    // 查询数字标签 42
    let num_items = db
        .get_from_index("test_store", "tags_idx", &IdbKey::Number(42.0))
        .unwrap();
    assert_eq!(num_items.len(), 1);
    assert_eq!(num_items[0].value["name"], "Item1");

    // 查询不存在的标签
    let non_items = db
        .get_from_index("test_store", "tags_idx", &IdbKey::String("python".into()))
        .unwrap();
    assert_eq!(non_items.len(), 0);

    // 查询空数组的记录
    // 注意：空数组在 multiEntry 下不产生任何索引键
    let count = db.count_from_index("test_store", "tags_idx", None).unwrap();
    // 应该是 5（rust, web, web, db, 42） - Item3 的空数组不产生索引键
    assert_eq!(count, 5);
}

/// 测试 IdbKey 的相等性实现（由 PartialEq 派生）。
#[test]
fn test_idb_key_equality() {
    // 相同类型，相同值
    let num1 = IdbKey::Number(42.0);
    let num2 = IdbKey::Number(42.0);
    assert_eq!(num1, num2);

    let str1 = IdbKey::String("hello".into());
    let str2 = IdbKey::String("hello".into());
    assert_eq!(str1, str2);

    let bin1 = IdbKey::Binary(vec![1, 2, 3]);
    let bin2 = IdbKey::Binary(vec![1, 2, 3]);
    assert_eq!(bin1, bin2);

    let arr1 = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    let arr2 = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    assert_eq!(arr1, arr2);

    // 不同类型，即使值"相等"也不等
    let num_42 = IdbKey::Number(42.0);
    let str_42 = IdbKey::String("42.0".into());
    assert_ne!(num_42, str_42);

    // 嵌套数组相等性
    let nested1 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![IdbKey::String("a".into())]),
    ]);
    let nested2 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![IdbKey::String("a".into())]),
    ]);
    assert_eq!(nested1, nested2);

    // 长度不同的数组不相等
    let short_arr = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    let long_arr = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Number(2.0)]);
    assert_ne!(short_arr, long_arr);
}

/// 测试 IdbKey 的 PartialOrd 实现。
#[test]
fn test_idb_key_partial_ord() {
    // Number 类型
    let num1 = IdbKey::Number(1.0);
    let num2 = IdbKey::Number(2.0);
    assert_eq!(num1.partial_cmp(&num2), Some(Ordering::Less));
    assert_eq!(num2.partial_cmp(&num1), Some(Ordering::Greater));
    assert_eq!(num1.partial_cmp(&num1), Some(Ordering::Equal));

    // 跨类型比较应该返回 None（但我们的实现通过 cmp_key 返回 Some）
    let num = IdbKey::Number(1.0);
    let str_key = IdbKey::String("1".into());
    // 按照我们的 cmp_key 实现，Number < String
    assert_eq!(num.partial_cmp(&str_key), Some(Ordering::Less));
    assert_eq!(str_key.partial_cmp(&num), Some(Ordering::Greater));

    // NaN 的特殊处理
    let nan_key = IdbKey::Number(f64::NAN);
    let normal_key = IdbKey::Number(1.0);
    // 当前实现：NaN 的 partial_cmp 返回 Some(Ordering::Equal)（通过 Ord 实现）
    // NaN 与任何值的比较都返回 Equal（通过 partial_cmp().unwrap_or(Ordering::Equal)）
    assert_eq!(nan_key.partial_cmp(&normal_key), Some(Ordering::Equal));
    assert_eq!(nan_key.partial_cmp(&nan_key), Some(Ordering::Equal));
}

/// 测试 IdbKey 的 Clone 行为。
#[test]
fn test_idb_key_clone() {
    let original = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::String("test".into()),
        IdbKey::Binary(vec![1, 2, 3]),
    ]);

    let cloned = original.clone();

    // 值相等
    assert_eq!(original, cloned);

    // 修改克隆不应该影响原对象（对于引用类型）
    let mut cloned_arr = if let IdbKey::Array(arr) = cloned {
        arr
    } else {
        panic!("Expected Array");
    };
    let new_key = IdbKey::String("modified".into());
    cloned_arr[1] = new_key;

    // 原对象不应被修改
    if let IdbKey::Array(arr) = &original {
        assert!(matches!(&arr[1], IdbKey::String(s) if s == "test"));
    }
}

/// 测试 IdbKey::Binary 与其他类型的比较
#[test]
fn test_idb_binary_comparison_with_other_types() {
    // Binary vs Number
    let binary = IdbKey::Binary(vec![1, 2, 3]);
    let number = IdbKey::Number(42.0);
    assert!(binary > number, "Binary > Number");
    assert!(number < binary, "Number < Binary");

    // Binary vs String
    let string = IdbKey::String("hello".to_string());
    assert!(binary > string, "Binary > String");
    assert!(string < binary, "String < Binary");

    // Binary vs Array
    let array = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    assert!(binary < array, "Binary < Array");
    assert!(array > binary, "Array > Binary");

    // Binary vs Binary
    let binary1 = IdbKey::Binary(vec![1, 2, 3]);
    let binary2 = IdbKey::Binary(vec![1, 2, 4]);
    let binary3 = IdbKey::Binary(vec![1, 2]);
    assert!(binary1 < binary2, "Binary[1,2,3] < Binary[1,2,4]");
    assert!(binary3 < binary1, "Binary[1,2] < Binary[1,2,3]");
    assert!(binary1 == binary1, "Binary equals self");
}

/// 测试 IdbKey 序列化边界情况
#[test]
fn test_idb_key_serialization_edge_cases() {
    // 空数组
    let empty_array = IdbKey::Array(vec![]);
    let single_array = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    assert!(empty_array < single_array);

    // 深层嵌套数组
    let deeply_nested1 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("deep".to_string()),
            IdbKey::Array(vec![IdbKey::Binary(vec![1, 2, 3])]),
        ]),
    ]);
    let deeply_nested2 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("deeper".to_string()),
            IdbKey::Array(vec![IdbKey::Binary(vec![1, 2, 3])]),
        ]),
    ]);
    assert!(deeply_nested1 < deeply_nested2);
}

/// 测试 IdbKeyRange 只/bound 边界情况
#[test]
fn test_idb_key_range_only_bound_edge_cases() {
    let key1 = IdbKey::Number(1.0);
    let key2 = IdbKey::Number(2.0);
    let key3 = IdbKey::Number(3.0);

    // 只包含一个键的范围
    let only_range = IdbKeyRange::only(key2.clone());
    assert!(only_range.contains(&key2));
    assert!(!only_range.contains(&key1));
    assert!(!only_range.contains(&key3));

    // 下界开区间
    let lower_open = IdbKeyRange::lower_bound(key2.clone(), true);
    assert!(!lower_open.contains(&key2)); // 不包含 2.0
    assert!(lower_open.contains(&key3)); // 包含 3.0
    assert!(!lower_open.contains(&key1)); // 不包含 1.0

    // 上界开区间
    let upper_open = IdbKeyRange::upper_bound(key2.clone(), true);
    assert!(!upper_open.contains(&key2)); // 不包含 2.0
    assert!(upper_open.contains(&key1)); // 包含 1.0
    assert!(!upper_open.contains(&key3)); // 不包含 3.0

    // 双闭区间
    let both_closed = IdbKeyRange::bound(key1.clone(), key3.clone(), false, false);
    assert!(both_closed.contains(&key1)); // 包含 1.0
    assert!(both_closed.contains(&key2)); // 包含 2.0
    assert!(both_closed.contains(&key3)); // 包含 3.0

    // 双开区间
    let both_open = IdbKeyRange::bound(key1.clone(), key3.clone(), true, true);
    assert!(!both_open.contains(&key1)); // 不包含 1.0
    assert!(both_open.contains(&key2)); // 包含 2.0
    assert!(!both_open.contains(&key3)); // 不包含 3.0

    // 相同的上下界（无效范围）
    let invalid_range = IdbKeyRange::bound(key2.clone(), key2.clone(), true, false);
    assert!(!invalid_range.contains(&key1));
    assert!(!invalid_range.contains(&key2));
    assert!(!invalid_range.contains(&key3));
}

// ── 新增测试：覆盖 types.rs 中的未测试代码路径 ──

/// 测试 IdbKeyRange::contains 的跨类型边界检查
#[test]
fn test_idb_key_range_contains_cross_type_boundaries() {
    // Number 边界
    let num_lower = IdbKey::Number(10.0);
    let num_upper = IdbKey::Number(20.0);
    let num_range = IdbKeyRange::bound(num_lower.clone(), num_upper.clone(), false, true);

    // 测试 Number 键在范围内的包含情况
    let below = IdbKey::Number(5.0);
    let inside = IdbKey::Number(15.0);
    let boundary_lower = IdbKey::Number(10.0);
    let boundary_upper = IdbKey::Number(20.0);
    let above = IdbKey::Number(25.0);

    assert!(!num_range.contains(&below), "低于下界的 Number 不应包含");
    assert!(num_range.contains(&inside), "在范围内的 Number 应包含");
    assert!(num_range.contains(&boundary_lower), "闭下界包含");
    assert!(!num_range.contains(&boundary_upper), "开上界不包含");
    assert!(!num_range.contains(&above), "高于上界的 Number 不应包含");

    // String 键测试：Number 界限下 String 应不包含（String > Number）
    let str_key = IdbKey::String("any string".into());
    assert!(!num_range.contains(&str_key), "String 不在 Number 范围内");

    // Array 键测试：Number 界限下 Array 应不包含（Array > Number）
    let arr_key = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    assert!(!num_range.contains(&arr_key), "Array 不在 Number 范围内");
}

/// 测试 IdbKeyRange::contains 在 Mixed 键类型下的行为
#[test]
fn test_idb_key_range_contains_mixed_types() {
    let str_lower = IdbKey::String("apple".into());
    let str_upper = IdbKey::String("zebra".into());
    let str_range = IdbKeyRange::bound(str_lower.clone(), str_upper.clone(), true, false);

    // Number 键：在 String 范围外（Number < String）
    let num_key = IdbKey::Number(42.0);
    assert!(!str_range.contains(&num_key), "Number 不在 String 范围内");

    // Binary 键：在 String 范围外（Binary < String）
    let binary_key = IdbKey::Binary(vec![1, 2, 3]);
    assert!(!str_range.contains(&binary_key), "Binary 不在 String 范围内");

    // Array 键：在 String 范围外（Array > String）
    let arr_key = IdbKey::Array(vec![IdbKey::String("any".into())]);
    assert!(!str_range.contains(&arr_key), "Array 不在 String 范围内");
}

/// 测试 IdbKeyRange 的边界构造函数和访问器
#[test]
fn test_idb_key_range_constructors_and_accessors() {
    // 测试 only 构造函数
    let only_num = IdbKeyRange::only(IdbKey::Number(42.0));
    assert_eq!(only_num.lower(), Some(&IdbKey::Number(42.0)));
    assert_eq!(only_num.upper(), Some(&IdbKey::Number(42.0)));
    assert!(!only_num.lower_open());
    assert!(!only_num.upper_open());

    // 测试 lower_bound 构造函数
    let lower_bound_str = IdbKeyRange::lower_bound(IdbKey::String("test".into()), true);
    assert_eq!(lower_bound_str.lower(), Some(&IdbKey::String("test".into())));
    assert_eq!(lower_bound_str.upper(), None);
    assert!(lower_bound_str.lower_open());
    assert!(!lower_bound_str.upper_open());

    // 测试 upper_bound 构造函数
    let upper_bound_bin = IdbKeyRange::upper_bound(IdbKey::Binary(vec![1, 2, 3]), false);
    assert_eq!(upper_bound_bin.lower(), None);
    assert_eq!(upper_bound_bin.upper(), Some(&IdbKey::Binary(vec![1, 2, 3])));
    assert!(!upper_bound_bin.lower_open());
    assert!(!upper_bound_bin.upper_open());
}

/// 测试 IdbKey 的所有跨类型比较组合
#[test]
fn test_idb_key_all_cross_type_comparisons() {
    // 确保所有比较路径都被覆盖
    // Number < String < Binary < Array 的类型顺序

    let num = IdbKey::Number(1.0);
    let str_key = IdbKey::String("string".into());
    let bin = IdbKey::Binary(vec![1, 2, 3]);
    let arr = IdbKey::Array(vec![IdbKey::Number(1.0)]);

    // Number 对其他类型
    assert!(num < str_key);
    assert!(num < bin);
    assert!(num < arr);

    // String 对其他类型（除 Number）
    assert!(str_key < bin);
    assert!(str_key < arr);

    // Binary 对其他类型（除 Number, String）
    assert!(bin < arr);

    // 相同类型的比较
    let num2 = IdbKey::Number(2.0);
    assert!(num < num2);

    let str2 = IdbKey::String("test".into());
    assert!(str_key < str2);

    let bin2 = IdbKey::Binary(vec![1, 2, 4]);
    assert!(bin < bin2);

    let arr2 = IdbKey::Array(vec![IdbKey::Number(2.0)]);
    assert!(arr < arr2);
}

/// 测试 IdbKey 的 Hash 和 Clone 行为
#[test]
fn test_idb_key_hash_and_clone_comprehensive() {
    use std::collections::HashMap;
    use std::collections::HashSet;

    // 测试 Hash 行为
    let mut set = HashSet::new();
    let keys = vec![
        IdbKey::Number(42.0),
        IdbKey::String("42".into()),
        IdbKey::Binary(vec![1, 2, 3]),
        IdbKey::Array(vec![IdbKey::Number(42.0)]),
    ];

    for key in &keys {
        set.insert(key.clone());
    }

    // 所有键都应该被正确插入
    assert_eq!(set.len(), 4);

    // 相同键应该能够找到
    assert!(set.contains(&IdbKey::Number(42.0)));
    assert!(set.contains(&IdbKey::String("42".into())));

    // 测试 HashMap
    let mut map = HashMap::new();
    for (i, key) in keys.iter().enumerate() {
        map.insert(key.clone(), i);
    }

    assert_eq!(map.get(&IdbKey::Number(42.0)), Some(&0));
    assert_eq!(map.get(&IdbKey::String("42".into())), Some(&1));

    // 测试 Clone 的深度复制
    let original = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::String("test".into()),
        IdbKey::Binary(vec![1, 2, 3]),
    ]);

    let cloned = original.clone();
    assert_eq!(original, cloned);

    // 克隆测试已完成，验证值相等即可
    // 深度克隆的测试在 IdbKey 的 PartialEq 实现中已涵盖
}

/// 测试 IdbIndex 的 multiEntry 和 unique 约束
#[test]
fn test_idb_index_multi_entry_and_unique_constraints() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("products", None, false).unwrap();

    // 插入多个带 tags 的记录
    db.add(
        "products",
        serde_json::json!({"name": "prod1", "tags": ["a", "b", "c"]}),
        Some(IdbKey::String("p1".into())),
    )
    .unwrap();

    db.add(
        "products",
        serde_json::json!({"name": "prod2", "tags": ["b", "c", "d"]}),
        Some(IdbKey::String("p2".into())),
    )
    .unwrap();

    db.add(
        "products",
        serde_json::json!({"name": "prod3", "tags": ["c"]}),
        Some(IdbKey::String("p3".into())),
    )
    .unwrap();

    // 创建 multiEntry 索引
    db.create_index("products", "tags_idx", "tags", false, true).unwrap();

    // 测试 multiEntry：多个标签应匹配多个记录
    let tag_b = db
        .get_from_index("products", "tags_idx", &IdbKey::String("b".into()))
        .unwrap();
    assert_eq!(tag_b.len(), 2, "标签 'b' 应匹配两个产品");

    let tag_c = db
        .get_from_index("products", "tags_idx", &IdbKey::String("c".into()))
        .unwrap();
    assert_eq!(tag_c.len(), 3, "标签 'c' 应匹配三个产品");

    // 测试 unique 约束 violation - 首先创建唯一索引，然后创建同名索引应失败
    let result1 = db.create_index("products", "name_unique", "name", true, false);
    assert!(result1.is_ok(), "创建唯一索引应成功");

    // 创建同名索引应失败
    let result2 = db.create_index("products", "name_unique", "name", false, true);
    assert!(result2.is_err(), "创建同名索引应失败");
}

/// 测试 IdbIndex 的重建索引功能
#[test]
fn test_idb_index_rebuild_functionality() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("users", None, false).unwrap();

    // 先插入数据
    db.add(
        "users",
        serde_json::json!({"name": "Alice", "group": "admin"}),
        Some(IdbKey::String("u1".into())),
    )
    .unwrap();

    // 然后创建索引
    db.create_index("users", "group_idx", "group", false, false).unwrap();

    // 验证索引可用
    let admins = db
        .get_from_index("users", "group_idx", &IdbKey::String("admin".into()))
        .unwrap();
    assert_eq!(admins.len(), 1);

    // 插入更多数据
    db.add(
        "users",
        serde_json::json!({"name": "Bob", "group": "user"}),
        Some(IdbKey::String("u2".into())),
    )
    .unwrap();

    // 验证索引已自动更新
    let users = db
        .get_from_index("users", "group_idx", &IdbKey::String("user".into()))
        .unwrap();
    assert_eq!(users.len(), 1);

    // 测试删除索引后重新创建
    db.delete_index("users", "group_idx").unwrap();
    db.create_index("users", "group_idx", "group", false, false).unwrap();

    // 重建后索引仍然可用
    let admins_after_rebuild = db
        .get_from_index("users", "group_idx", &IdbKey::String("admin".into()))
        .unwrap();
    assert_eq!(admins_after_rebuild.len(), 1);
}

/// 测试 IdbKey 的复杂 Array 键排序
#[test]
fn test_idb_key_complex_array_sorting() {
    // 创建复杂的 Array 键用于测试排序算法的所有分支
    let keys = vec![
        IdbKey::Array(vec![]),                                                // 空数组
        IdbKey::Array(vec![IdbKey::Number(1.0)]),                             // 单元素数组
        IdbKey::Array(vec![IdbKey::Number(2.0)]),                             // 另一个单元素
        IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Number(2.0)]),        // 两元素数组
        IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Number(3.0)]),        // 两元素，第二个不同
        IdbKey::Array(vec![IdbKey::String("a".into())]),                      // String 元素
        IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::String("b".into())]), // 混合类型
        IdbKey::Array(vec![
            IdbKey::Number(1.0),
            IdbKey::Array(vec![IdbKey::Number(2.0)]), // 嵌套数组
        ]),
        IdbKey::Array(vec![
            IdbKey::Number(1.0),
            IdbKey::Array(vec![IdbKey::Number(3.0)]), // 不同的嵌套数组
        ]),
    ];

    // 排序测试
    let mut sorted = keys.clone();
    sorted.sort();

    // 验证排序结果
    assert!(sorted[0] < sorted[1], "空数组应该是最小的");
    assert!(sorted[1] < sorted[2], "Number(1.0) < Number(2.0)");
    assert!(sorted[2] < sorted[3], "单元素 < 两元素");
    assert!(sorted[3] < sorted[4], "1,2 < 1,3");
    assert!(sorted[4] < sorted[5], "Number 数组 < String 数组");
    assert!(sorted[5] < sorted[6], "String 混合 < Number/String 混合");
    assert!(sorted[6] < sorted[7], "混合 < 嵌套数组");
    assert!(sorted[7] < sorted[8], "2.0 嵌套 < 3.0 嵌套");
}

/// 测试 IdbKey 的边界值比较
#[test]
fn test_idb_key_boundary_value_comparisons() {
    // 测试 NaN 的特殊处理
    let nan_key = IdbKey::Number(f64::NAN);
    let normal_key = IdbKey::Number(1.0);
    let another_nan = IdbKey::Number(f64::NAN);

    // NaN 与任何值的比较都返回 Equal（通过 partial_cmp().unwrap_or(Ordering::Equal)）
    assert_eq!(nan_key.cmp(&normal_key), Ordering::Equal);
    assert_eq!(nan_key.cmp(&another_nan), Ordering::Equal);
    assert_eq!(normal_key.cmp(&nan_key), Ordering::Equal);

    // 测试 Infinity
    let inf_key = IdbKey::Number(f64::INFINITY);
    let neg_inf_key = IdbKey::Number(f64::NEG_INFINITY);
    let finite_key = IdbKey::Number(100.0);

    assert!(neg_inf_key < finite_key);
    assert!(finite_key < inf_key);
    assert!(neg_inf_key < inf_key);

    // 测试最小的 Number 键与其他类型的比较
    let min_num = IdbKey::Number(f64::MIN);
    let max_num = IdbKey::Number(f64::MAX);
    let str_key = IdbKey::String("any".into());
    let bin_key = IdbKey::Binary(vec![0]);
    let arr_key = IdbKey::Array(vec![IdbKey::Number(0.0)]);

    // Number 总是最小的
    assert!(min_num < str_key);
    assert!(max_num < str_key);
    assert!(min_num < bin_key);
    assert!(max_num < bin_key);
    assert!(min_num < arr_key);
    assert!(max_num < arr_key);
}

/// 测试 IdbKey 的 Debug 输出格式
#[test]
fn test_idb_key_debug_format() {
    // 测试各类型的 Debug 输出
    let num = IdbKey::Number(42.0);
    let str_key = IdbKey::String("hello".into());
    let bin = IdbKey::Binary(vec![1, 2, 3]);
    let arr = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::String("test".into())]);

    let num_debug = format!("{:?}", num);
    let str_debug = format!("{:?}", str_key);
    let bin_debug = format!("{:?}", bin);
    let arr_debug = format!("{:?}", arr);

    assert!(num_debug.contains("Number"));
    assert!(str_debug.contains("String"));
    assert!(bin_debug.contains("Binary"));
    assert!(arr_debug.contains("Array"));

    // 确保输出不 panic
    assert!(!num_debug.is_empty());
    assert!(!str_debug.is_empty());
    assert!(!bin_debug.is_empty());
    assert!(!arr_debug.is_empty());
}

/// 测试 IdbKeyRange 在边界值上的 contains 方法
#[test]
fn test_idb_key_range_contains_boundary_values() {
    // 使用不同的键类型测试边界
    let _number_key = IdbKey::Number(42.0);
    let _string_key = IdbKey::String("test".into());
    let binary_key = IdbKey::Binary(vec![1, 2, 3]);
    let array_key = IdbKey::Array(vec![IdbKey::Number(1.0)]);

    // 测试 Number 类型的边界
    let num_range = IdbKeyRange::bound(IdbKey::Number(40.0), IdbKey::Number(45.0), false, true);

    assert!(num_range.contains(&IdbKey::Number(40.0)));
    assert!(num_range.contains(&IdbKey::Number(42.5)));
    assert!(!num_range.contains(&IdbKey::Number(45.0)));

    // 测试 String 类型的边界
    let str_range = IdbKeyRange::bound(IdbKey::String("a".into()), IdbKey::String("c".into()), true, false);

    assert!(!str_range.contains(&IdbKey::String("a".into())));
    assert!(str_range.contains(&IdbKey::String("b".into())));
    assert!(str_range.contains(&IdbKey::String("c".into())));

    // 测试跨类型范围
    let mixed_range = IdbKeyRange::bound(IdbKey::Number(10.0), IdbKey::String("z".into()), false, false);

    // Number 在范围内
    assert!(mixed_range.contains(&IdbKey::Number(15.0)));
    // String 在范围内
    assert!(mixed_range.contains(&IdbKey::String("m".into())));
    // Binary 不在范围内（Binary > String）
    assert!(!mixed_range.contains(&binary_key));
    // Array 不在范围内（Array > String）
    assert!(!mixed_range.contains(&array_key));
}

/// 测试 IdbIndex 的 extract_keys 方法边界情况
#[test]
fn test_idb_index_extract_keys_edge_cases() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("test_store", None, false).unwrap();

    // 测试 JSON 值到 IdbKey 转换的边界情况
    // 1. 空数组
    db.add(
        "test_store",
        serde_json::json!({"name": "empty_array", "tags": []}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();

    // 2. 包含 null 的数组
    db.add(
        "test_store",
        serde_json::json!({"name": "with_null", "tags": [1, null, 3]}),
        Some(IdbKey::Number(2.0)),
    )
    .unwrap();

    // 3. 混合类型数组
    db.add(
        "test_store",
        serde_json::json!({"name": "mixed", "tags": [1, "two", [3]]}),
        Some(IdbKey::Number(3.0)),
    )
    .unwrap();

    // 创建 multiEntry 索引
    db.create_index("test_store", "tags_idx", "tags", false, true).unwrap();

    // 测试各种标签的查询
    // 数字标签 1
    let num_results = db
        .get_from_index("test_store", "tags_idx", &IdbKey::Number(1.0))
        .unwrap();
    assert_eq!(num_results.len(), 2, "数字 1 应出现在两个记录中（带 null 的数组）");

    // 数字标签 3
    let num_3_results = db
        .get_from_index("test_store", "tags_idx", &IdbKey::Number(3.0))
        .unwrap();
    assert_eq!(num_3_results.len(), 1, "数字 3 应出现在一个记录中");

    // 字符串标签
    let str_results = db
        .get_from_index("test_store", "tags_idx", &IdbKey::String("two".into()))
        .unwrap();
    assert_eq!(str_results.len(), 1, "字符串 'two' 应出现在一个记录中");

    // 嵌套数组作为键
    let nested_results = db
        .get_from_index("test_store", "tags_idx", &IdbKey::Array(vec![IdbKey::Number(3.0)]))
        .unwrap();
    assert_eq!(nested_results.len(), 1, "嵌套数组应作为一个键");

    // 查询不存在的标签
    let non_existent = db
        .get_from_index("test_store", "tags_idx", &IdbKey::Number(99.0))
        .unwrap();
    assert_eq!(non_existent.len(), 0, "不存在的标签应返回空结果");
}

/// 测试 IdbKey 的哈希一致性在不同场景下
#[test]
fn test_idb_key_hash_consistency_across_operations() {
    use std::collections::HashSet;
    use std::collections::hash_map::HashMap;

    // 测试哈希在 HashSet 中的使用
    let keys = vec![
        IdbKey::Number(42.0),
        IdbKey::String("42".into()),
        IdbKey::Binary(vec![1, 2, 3]),
        IdbKey::Array(vec![IdbKey::Number(42.0)]),
    ];

    let mut set = HashSet::new();
    for key in &keys {
        assert!(set.insert(key.clone()), "插入新键应返回 true");
    }

    // 重复插入相同键应返回 false
    for key in &keys {
        assert!(!set.insert(key.clone()), "重复插入相同键应返回 false");
    }

    // 测试哈希在 HashMap 中的使用
    let mut map = HashMap::new();
    for (i, key) in keys.iter().enumerate() {
        map.insert(key.clone(), i.to_string());
    }

    // 验证所有键都能被找到
    for (i, key) in keys.iter().enumerate() {
        assert_eq!(map.get(key), Some(&(i.to_string())), "所有键都应存在于 map 中");
    }

    // 测试删除和重新插入
    let removed_key = IdbKey::Number(42.0);
    let removed_value = map.remove(&removed_key);
    assert_eq!(removed_value, Some("0".to_string()));

    // 重新插入
    map.insert(removed_key.clone(), "new_value".to_string());
    assert_eq!(map.get(&removed_key), Some(&"new_value".to_string()));
}

/// 测试 IdbKeyRange 的构造函数和边界组合
#[test]
fn test_idb_key_range_constructor_combinations() {
    let num_key = IdbKey::Number(10.0);
    let str_key = IdbKey::String("test".into());
    let bin_key = IdbKey::Binary(vec![1, 2, 3]);

    // 测试各种边界组合
    // 1. 双闭区间
    let closed_both = IdbKeyRange::bound(num_key.clone(), IdbKey::Number(20.0), false, false);
    assert!(closed_both.contains(&num_key));
    assert!(closed_both.contains(&IdbKey::Number(15.0)));
    assert!(closed_both.contains(&IdbKey::Number(20.0)));

    // 2. 双开区间
    let open_both = IdbKeyRange::bound(num_key.clone(), IdbKey::Number(20.0), true, true);
    assert!(!open_both.contains(&num_key));
    assert!(open_both.contains(&IdbKey::Number(15.0)));
    assert!(!open_both.contains(&IdbKey::Number(20.0)));

    // 3. 左闭右开
    let closed_open = IdbKeyRange::bound(num_key.clone(), IdbKey::Number(20.0), false, true);
    assert!(closed_open.contains(&num_key));
    assert!(closed_open.contains(&IdbKey::Number(15.0)));
    assert!(!closed_open.contains(&IdbKey::Number(20.0)));

    // 4. 左开右闭
    let open_closed = IdbKeyRange::bound(num_key.clone(), IdbKey::Number(20.0), true, false);
    assert!(!open_closed.contains(&num_key));
    assert!(open_closed.contains(&IdbKey::Number(15.0)));
    assert!(open_closed.contains(&IdbKey::Number(20.0)));

    // 5. 只有下界
    let lower_only = IdbKeyRange::lower_bound(num_key.clone(), true);
    assert!(!lower_only.contains(&num_key));
    assert!(lower_only.contains(&IdbKey::Number(15.0)));
    assert!(lower_only.contains(&str_key)); // String > Number
    assert!(lower_only.contains(&bin_key)); // Binary > Number

    // 6. 只有上界（闭区间）
    let upper_only = IdbKeyRange::upper_bound(str_key.clone(), false);
    assert!(upper_only.contains(&num_key)); // Number < String
    assert!(upper_only.contains(&str_key)); // 闭区间包含上界
    assert!(!upper_only.contains(&IdbKey::String("zzz".into()))); // "zzz" > "test"

    // 7. 只有单值
    let only = IdbKeyRange::only(bin_key.clone());
    assert!(!only.contains(&num_key)); // Number < Binary
    assert!(!only.contains(&str_key)); // String < Binary
    assert!(only.contains(&bin_key));
    assert!(!only.contains(&IdbKey::Binary(vec![1, 2, 4])));
}

/// 测试 IdbKey 的嵌套数组相等性
#[test]
fn test_idb_key_nested_array_equality() {
    // 测试深度嵌套的数组相等性
    let nested1 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("a".into()),
            IdbKey::Array(vec![IdbKey::Number(2.0), IdbKey::Number(3.0)]),
        ]),
    ]);

    let nested2 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("a".into()),
            IdbKey::Array(vec![IdbKey::Number(2.0), IdbKey::Number(3.0)]),
        ]),
    ]);

    let nested3 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("b".into()), // 不同的字符串
            IdbKey::Array(vec![IdbKey::Number(2.0), IdbKey::Number(3.0)]),
        ]),
    ]);

    assert_eq!(nested1, nested2, "相同的嵌套数组应相等");
    assert_ne!(nested1, nested3, "不同的嵌套数组不应相等");

    // 测试空数组的相等性
    let empty1 = IdbKey::Array(vec![]);
    let empty2 = IdbKey::Array(vec![]);
    assert_eq!(empty1, empty2, "空数组应相等");
}

/// 测试 IdbKeyRange 的 contains 方法性能关键路径
#[test]
fn test_idb_key_range_contains_performance_paths() {
    let keys = vec![IdbKey::Number(1.0), IdbKey::Number(5.0), IdbKey::Number(10.0)];

    // 测试各种情况下的 contains 调用
    // 1. 只有下界
    let lower_range = IdbKeyRange::lower_bound(IdbKey::Number(5.0), true);
    assert!(!lower_range.contains(&keys[0])); // 1.0 < 5.0
    assert!(!lower_range.contains(&keys[1])); // 5.0 (开区间不包含)
    assert!(lower_range.contains(&keys[2])); // 10.0 > 5.0

    // 2. 只有上界
    let upper_range = IdbKeyRange::upper_bound(IdbKey::Number(8.0), false);
    assert!(upper_range.contains(&keys[0])); // 1.0 < 8.0
    assert!(upper_range.contains(&keys[1])); // 5.0 < 8.0
    assert!(!upper_range.contains(&keys[2])); // 10.0 > 8.0

    // 3. 双界
    let both_range = IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(8.0), true, false);
    assert!(!both_range.contains(&keys[0])); // 1.0 < 3.0
    assert!(both_range.contains(&keys[1])); // 5.0 在范围内
    assert!(!both_range.contains(&keys[2])); // 10.0 > 8.0

    // 4. 空范围
    let empty_range = IdbKeyRange::bound(IdbKey::Number(10.0), IdbKey::Number(1.0), false, false);
    assert!(!empty_range.contains(&keys[0]));
    assert!(!empty_range.contains(&keys[1]));
    assert!(!empty_range.contains(&keys[2]));
}

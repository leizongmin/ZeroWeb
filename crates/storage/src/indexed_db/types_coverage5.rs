//! IndexedDB types.rs 覆盖率提升测试（第 5 轮）
//! 聚焦于唯一索引冲突、multiEntry、json_value_to_idb_key 边界、
//! 事务缓冲区重复键检查、rename_object_store 等。

use super::super::*;
use serde_json::json;

/// 测试唯一索引冲突：插入两条具有相同索引键的记录应报错。
#[test]
fn test_unique_index_violation_on_add() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 创建唯一索引
    db.create_index("store", "idx", "name", true, false).unwrap();

    db.add("store", json!({"name": "alice"}), Some(IdbKey::Number(1.0)))
        .unwrap();

    // 第二条记录具有相同的 name 索引键，应冲突
    let result = db.add("store", json!({"name": "alice"}), Some(IdbKey::Number(2.0)));
    assert!(result.is_err(), "唯一索引冲突应报错");
    match result {
        Err(StorageError::Database(msg)) => {
            assert!(msg.contains("Unique index"), "错误消息应提及唯一索引: {msg}");
        }
        _ => panic!("期望 Database 错误"),
    }
}

/// 测试 multiEntry 索引：数组值中的每个元素都作为索引键。
#[test]
fn test_multi_entry_index_extraction() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 创建 multiEntry 索引
    db.create_index("store", "tags", "tags", false, true).unwrap();

    // 添加含数组的记录
    db.add(
        "store",
        json!({"tags": ["rust", "browser", "web"]}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();

    db.add(
        "store",
        json!({"tags": ["rust", "security"]}),
        Some(IdbKey::Number(2.0)),
    )
    .unwrap();

    // 通过索引查找 "rust" 应找到两条记录
    let results = db
        .get_from_index("store", "tags", &IdbKey::String("rust".into()))
        .unwrap();
    assert_eq!(results.len(), 2, "应找到 2 条含 'rust' 标签的记录");

    // 通过索引查找 "browser" 应只找到一条
    let results = db
        .get_from_index("store", "tags", &IdbKey::String("browser".into()))
        .unwrap();
    assert_eq!(results.len(), 1, "应找到 1 条含 'browser' 标签的记录");
}

/// 测试 multiEntry 索引：非数组值（标量）也应能建立索引。
#[test]
fn test_multi_entry_index_scalar_value() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    db.create_index("store", "tag", "tag", false, true).unwrap();

    db.add("store", json!({"tag": "hello"}), Some(IdbKey::Number(1.0)))
        .unwrap();

    let results = db
        .get_from_index("store", "tag", &IdbKey::String("hello".into()))
        .unwrap();
    assert_eq!(results.len(), 1);
}

/// 测试索引 key_path 指向 Object/Bool/Null 时不生成索引条目（json_value_to_idb_key 返回 None）。
#[test]
fn test_index_json_unsupported_key_types() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    // key_path "val" 指向的值是 Object
    db.create_index("store", "idx", "val", false, false).unwrap();

    db.add("store", json!({"val": {"nested": true}}), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", json!({"val": true}), Some(IdbKey::Number(2.0)))
        .unwrap();
    db.add("store", json!({"val": null}), Some(IdbKey::Number(3.0)))
        .unwrap();

    // 这些值都不是有效的 IdbKey，索引应为空
    let count = db.count_from_index("store", "idx", None).unwrap();
    assert_eq!(count, 0, "Object/Bool/Null 不应生成索引条目");
}

/// 测试 multiEntry 索引中数组含非有效键元素时的行为。
#[test]
fn test_multi_entry_index_array_with_unsupported_elements() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "tags", "tags", false, true).unwrap();

    // 数组中混合有效和无效键值
    db.add(
        "store",
        json!({"tags": [1, "ok", true, null, {"obj": 1}]}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();

    // 只有 Number(1) 和 String("ok") 应生成索引条目
    let results = db.get_from_index("store", "tags", &IdbKey::Number(1.0)).unwrap();
    assert_eq!(results.len(), 1, "Number 键应被索引");

    let results = db
        .get_from_index("store", "tags", &IdbKey::String("ok".into()))
        .unwrap();
    assert_eq!(results.len(), 1, "String 键应被索引");
}

/// 测试 multiEntry 索引中数组完全由无效键组成时无索引条目。
#[test]
fn test_multi_entry_index_all_invalid_elements() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "items", "items", false, true).unwrap();

    db.add(
        "store",
        json!({"items": [true, false, null]}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();

    let count = db.count_from_index("store", "items", None).unwrap();
    assert_eq!(count, 0, "全无效元素不应生成索引条目");
}

/// 测试复合数组键（Array 中含 Number 和 String）的 IdbKey 比较。
#[test]
fn test_idb_key_array_with_mixed_types_comparison() {
    let a = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::String("a".into())]);
    let b = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::String("b".into())]);
    assert!(a < b, "第二个元素 String(a) < String(b)");
}

/// 测试 rename_object_store 正常路径。
#[test]
fn test_rename_object_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("old_name", None, false).unwrap();

    db.rename_object_store("old_name", "new_name").unwrap();

    assert!(!db.has_store("old_name"), "旧名称不应存在");
    assert!(db.has_store("new_name"), "新名称应存在");

    let names = db.store_names();
    assert!(names.contains(&"new_name"));
}

/// 测试 rename_object_store 源不存在时报错。
#[test]
fn test_rename_object_store_source_not_found() {
    let mut db = IdbDatabase::new("test", 1);
    let result = db.rename_object_store("nonexistent", "new");
    assert!(result.is_err());
}

/// 测试 rename_object_store 目标已存在时报错。
#[test]
fn test_rename_object_store_target_exists() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("a", None, false).unwrap();
    db.create_object_store("b", None, false).unwrap();

    let result = db.rename_object_store("a", "b");
    assert!(result.is_err(), "目标名称已存在时应报错");
}

/// 测试 put 覆盖已有记录时更新索引。
#[test]
fn test_put_overwrite_updates_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "idx", "name", false, false).unwrap();

    db.add("store", json!({"name": "alice"}), Some(IdbKey::Number(1.0)))
        .unwrap();

    // put 覆盖同一主键
    db.put("store", json!({"name": "bob"}), Some(IdbKey::Number(1.0)))
        .unwrap();

    // 新索引键 "bob" 应能查到
    let results = db
        .get_from_index("store", "idx", &IdbKey::String("bob".into()))
        .unwrap();
    assert_eq!(results.len(), 1);

    // 旧索引键 "alice" 应查不到
    let results = db
        .get_from_index("store", "idx", &IdbKey::String("alice".into()))
        .unwrap();
    assert_eq!(results.len(), 0, "旧索引键应被移除");
}

/// 测试事务中 tx_add 检测缓冲区中重复键。
#[test]
fn test_tx_add_buffer_duplicate_key() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();

    // 第一次 add
    db.tx_add(&tx, "store", json!({"a": 1}), Some(IdbKey::Number(1.0)))
        .unwrap();

    // 第二次 add 相同主键 → 应检测到缓冲区重复
    let result = db.tx_add(&tx, "store", json!({"a": 2}), Some(IdbKey::Number(1.0)));
    assert!(result.is_err(), "缓冲区重复键应报错");
}

/// 测试 tx_delete 对缓冲区中存在的键返回 found=true。
#[test]
fn test_tx_delete_buffer_key_found() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();

    // 先在缓冲区添加
    db.tx_add(&tx, "store", json!({"a": 1}), Some(IdbKey::Number(1.0)))
        .unwrap();

    // 从缓冲区删除
    let found = db.tx_delete(&tx, "store", &IdbKey::Number(1.0)).unwrap();
    assert!(found, "缓冲区中存在的键应返回 found=true");
}

/// 测试 tx_get 从缓冲区读取未提交的 Put 变更。
#[test]
fn test_tx_get_reads_buffered_put() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();

    db.tx_put(&tx, "store", json!({"v": 42}), Some(IdbKey::Number(1.0)))
        .unwrap();

    let result = db.tx_get(&tx, "store", &IdbKey::Number(1.0)).unwrap();
    assert!(result.is_some(), "应能从缓冲区读到 Put 的记录");
    assert_eq!(result.unwrap().value, json!({"v": 42}));
}

/// 测试 tx_get 对缓冲区中 Delete 后的键返回 None。
#[test]
fn test_tx_get_returns_none_after_buffered_delete() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 先提交一条记录
    db.add("store", json!({"x": 1}), Some(IdbKey::Number(1.0))).unwrap();

    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();

    // 在事务中删除
    db.tx_delete(&tx, "store", &IdbKey::Number(1.0)).unwrap();

    // 读取应返回 None
    let result = db.tx_get(&tx, "store", &IdbKey::Number(1.0)).unwrap();
    assert!(result.is_none(), "Delete 后应返回 None");
}

/// 测试 commit_tx 将事务变更应用到 store。
#[test]
fn test_commit_tx_applies_mutations() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();

    db.tx_add(&tx, "store", json!({"val": 100}), Some(IdbKey::Number(1.0)))
        .unwrap();

    db.commit_tx(&mut tx).unwrap();

    // 事务提交后，记录应在 store 中
    let record = db.get("store", &IdbKey::Number(1.0));
    assert!(record.is_some());
    assert_eq!(record.unwrap().value, json!({"val": 100}));
}

/// 测试唯一索引在 put 覆盖时不报冲突。
#[test]
fn test_unique_index_put_overwrite_same_key() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "idx", "name", true, false).unwrap();

    db.add("store", json!({"name": "alice"}), Some(IdbKey::Number(1.0)))
        .unwrap();

    // put 同一主键但不同索引值 → 应成功
    let result = db.put("store", json!({"name": "bob"}), Some(IdbKey::Number(1.0)));
    assert!(result.is_ok(), "put 覆盖同一主键应成功");
}

/// 测试 get_all_from_index_with_range 按索引范围查询。
#[test]
fn test_get_all_from_index_with_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "age", "age", false, false).unwrap();

    db.add("store", json!({"age": 20}), Some(IdbKey::Number(1.0))).unwrap();
    db.add("store", json!({"age": 30}), Some(IdbKey::Number(2.0))).unwrap();
    db.add("store", json!({"age": 40}), Some(IdbKey::Number(3.0))).unwrap();

    let range = IdbKeyRange::bound(IdbKey::Number(25.0), IdbKey::Number(35.0), false, false);
    let results = db.get_all_from_index_with_range("store", "age", &range).unwrap();
    assert_eq!(results.len(), 1, "范围 [25, 35] 内应只有 age=30");
}

/// 测试 count_from_index 有范围和无范围。
#[test]
fn test_count_from_index_with_and_without_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "score", "score", false, false).unwrap();

    db.add("store", json!({"score": 10}), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", json!({"score": 20}), Some(IdbKey::Number(2.0)))
        .unwrap();
    db.add("store", json!({"score": 30}), Some(IdbKey::Number(3.0)))
        .unwrap();

    // 无范围
    let count = db.count_from_index("store", "score", None).unwrap();
    assert_eq!(count, 3);

    // 有范围
    let range = IdbKeyRange::bound(IdbKey::Number(15.0), IdbKey::Number(25.0), false, false);
    let count = db.count_from_index("store", "score", Some(&range)).unwrap();
    assert_eq!(count, 1, "范围 [15, 25] 内应只有 score=20");
}

/// 测试 open_cursor_on_index 返回有效游标。
#[test]
fn test_open_cursor_on_index_sorted() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "name", "name", false, false).unwrap();

    db.add("store", json!({"name": "charlie"}), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", json!({"name": "alice"}), Some(IdbKey::Number(2.0)))
        .unwrap();
    db.add("store", json!({"name": "bob"}), Some(IdbKey::Number(3.0)))
        .unwrap();

    let cursor = db.open_cursor_on_index("store", "name", None).unwrap();
    assert!(cursor.is_some());

    let cursor = cursor.unwrap();
    let record = db.cursor_record(&cursor).unwrap();
    // 游标应指向某条有效记录
    assert!(record.value["name"].is_string());
}

/// 测试 open_key_cursor 按键排序。
#[test]
fn test_open_key_cursor_sorted() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    db.add("store", json!({"v": 3}), Some(IdbKey::Number(30.0))).unwrap();
    db.add("store", json!({"v": 1}), Some(IdbKey::Number(10.0))).unwrap();
    db.add("store", json!({"v": 2}), Some(IdbKey::Number(20.0))).unwrap();

    let cursor = db.open_key_cursor("store", None).unwrap();
    assert!(cursor.is_some());
    let cursor = cursor.unwrap();
    let key = db.cursor_key(&cursor).unwrap();
    assert_eq!(*key, IdbKey::Number(10.0), "第一个键应为 10（排序后）");
}

/// 测试 open_cursor 按键排序。
#[test]
fn test_open_cursor_sorted() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    db.add("store", json!({"v": "c"}), Some(IdbKey::String("charlie".into())))
        .unwrap();
    db.add("store", json!({"v": "a"}), Some(IdbKey::String("alice".into())))
        .unwrap();

    let cursor = db.open_cursor("store", None).unwrap();
    assert!(cursor.is_some());
    let cursor = cursor.unwrap();
    let record = db.cursor_record(&cursor).unwrap();
    assert_eq!(record.value["v"], "a");
}

/// 测试 open_cursor 空结果返回 None。
#[test]
fn test_open_cursor_empty() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    let result = db.open_cursor("store", None).unwrap();
    assert!(result.is_none(), "空 store 的游标应返回 None");
}

/// 测试 open_key_cursor 空结果返回 None。
#[test]
fn test_open_key_cursor_empty() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    let result = db.open_key_cursor("store", None).unwrap();
    assert!(result.is_none(), "空 store 的键游标应返回 None");
}

/// 测试 IdbKey Array 键比较：长度不同时，短的更小。
#[test]
fn test_idb_key_array_length_comparison() {
    let a = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    let b = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Number(2.0)]);
    assert!(a < b, "短数组应小于长数组（前缀相同时）");
}

/// 测试 IdbKey Binary 键比较。
#[test]
fn test_idb_key_binary_comparison() {
    let a = IdbKey::Binary(vec![1, 2, 3]);
    let b = IdbKey::Binary(vec![1, 2, 4]);
    assert!(a < b, "二进制键应按字典序比较");

    let c = IdbKey::Binary(vec![1, 2]);
    assert!(c < a, "短二进制键应更小");
}

/// 测试 IdbKeyRange::lower_bound 和 upper_bound 的 contains。
#[test]
fn test_idb_key_range_lower_upper_bound_contains() {
    let lower = IdbKeyRange::lower_bound(IdbKey::Number(5.0), false);
    assert!(lower.contains(&IdbKey::Number(5.0)), "闭区间应包含下界");
    assert!(lower.contains(&IdbKey::Number(10.0)));
    assert!(!lower.contains(&IdbKey::Number(4.0)));

    let lower_open = IdbKeyRange::lower_bound(IdbKey::Number(5.0), true);
    assert!(!lower_open.contains(&IdbKey::Number(5.0)), "开区间不应包含下界");
    assert!(lower_open.contains(&IdbKey::Number(6.0)));

    let upper = IdbKeyRange::upper_bound(IdbKey::Number(5.0), false);
    assert!(upper.contains(&IdbKey::Number(5.0)));
    assert!(upper.contains(&IdbKey::Number(0.0)));
    assert!(!upper.contains(&IdbKey::Number(6.0)));

    let upper_open = IdbKeyRange::upper_bound(IdbKey::Number(5.0), true);
    assert!(!upper_open.contains(&IdbKey::Number(5.0)));
    assert!(upper_open.contains(&IdbKey::Number(4.0)));
}

/// 测试 IdbKeyRange accessor 方法。
#[test]
fn test_idb_key_range_accessors() {
    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), true, false);
    assert_eq!(range.lower(), Some(&IdbKey::Number(1.0)));
    assert_eq!(range.upper(), Some(&IdbKey::Number(10.0)));
    assert!(range.lower_open());
    assert!(!range.upper_open());
}

/// 测试 IdbKeyRange::only 的 accessor。
#[test]
fn test_idb_key_range_only() {
    let range = IdbKeyRange::only(IdbKey::Number(42.0));
    assert!(range.contains(&IdbKey::Number(42.0)));
    assert!(!range.contains(&IdbKey::Number(41.0)));
    assert!(!range.lower_open());
    assert!(!range.upper_open());
}

/// 测试索引不存在时的报错。
#[test]
fn test_index_not_found_errors() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // get_from_index 索引不存在
    let result = db.get_from_index("store", "nonexistent", &IdbKey::Number(1.0));
    assert!(result.is_err());

    // delete_index 索引不存在
    let result = db.delete_index("store", "nonexistent");
    assert!(result.is_err());

    // index_names 空列表
    let names = db.index_names("store").unwrap();
    assert!(names.is_empty());
}

/// 测试 clear_store 同时清空索引。
#[test]
fn test_clear_store_clears_indexes() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "name", "name", false, false).unwrap();

    db.add("store", json!({"name": "alice"}), Some(IdbKey::Number(1.0)))
        .unwrap();

    db.clear_store("store").unwrap();

    assert_eq!(db.count("store").unwrap(), 0);

    // 索引也应被清空
    let count = db.count_from_index("store", "name", None).unwrap();
    assert_eq!(count, 0, "清空 store 后索引应也为空");
}

/// 测试 get_all_with_range 按键范围过滤。
#[test]
fn test_get_all_with_range_filtering() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    for i in 1..=5 {
        db.add("store", json!({"v": i}), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }

    let range = IdbKeyRange::bound(IdbKey::Number(2.0), IdbKey::Number(4.0), false, false);
    let results = db.get_all_with_range("store", &range).unwrap();
    assert_eq!(results.len(), 3, "范围 [2, 4] 内应有 3 条记录");
}

/// 测试 count_with_range 按键范围计数。
#[test]
fn test_count_with_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    for i in 1..=10 {
        db.add("store", json!({"v": i}), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }

    let range = IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(7.0), false, false);
    let count = db.count_with_range("store", &range).unwrap();
    assert_eq!(count, 5, "范围 [3, 7] 内应有 5 条记录");
}

/// 测试 delete 返回是否实际删除了记录。
#[test]
fn test_delete_returns_found() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", json!({"v": 1}), Some(IdbKey::Number(1.0))).unwrap();

    // 删除存在的键
    let found = db.delete("store", &IdbKey::Number(1.0)).unwrap();
    assert!(found, "删除存在的键应返回 true");

    // 删除不存在的键
    let found = db.delete("store", &IdbKey::Number(999.0)).unwrap();
    assert!(!found, "删除不存在的键应返回 false");
}

/// 测试自增主键的事务提交。
#[test]
fn test_auto_increment_tx_commit() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, true).unwrap();

    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();

    let key1 = db.tx_add(&tx, "store", json!({"v": 1}), None).unwrap();
    assert_eq!(key1, IdbKey::Number(1.0));

    let key2 = db.tx_add(&tx, "store", json!({"v": 2}), None).unwrap();
    assert_eq!(key2, IdbKey::Number(2.0));

    db.commit_tx(&mut tx).unwrap();

    let count = db.count("store").unwrap();
    assert_eq!(count, 2);
}

/// 测试 add 不提供键且非自增时报错。
#[test]
fn test_add_no_key_no_auto_increment() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    let result = db.add("store", json!({"v": 1}), None);
    assert!(result.is_err(), "不提供键且非自增应报错");
}

/// 测试 put 不提供键且非自增时报错。
#[test]
fn test_put_no_key_no_auto_increment() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    let result = db.put("store", json!({"v": 1}), None);
    assert!(result.is_err(), "不提供键且非自增应报错");
}

/// 测试对不存在的 store 操作报错。
#[test]
fn test_operations_on_nonexistent_store() {
    let mut db = IdbDatabase::new("test", 1);

    assert!(db.add("nope", json!(1), Some(IdbKey::Number(1.0))).is_err());
    assert!(db.put("nope", json!(1), Some(IdbKey::Number(1.0))).is_err());
    assert!(db.delete("nope", &IdbKey::Number(1.0)).is_err());
    assert!(db.get_all("nope").is_err());
    assert!(db.clear_store("nope").is_err());
    assert!(db.count("nope").is_err());
    assert!(db.create_index("nope", "idx", "f", false, false).is_err());
}

/// 测试重复创建 store 报错。
#[test]
fn test_create_duplicate_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    let result = db.create_object_store("store", None, false);
    assert!(result.is_err(), "重复创建 store 应报错");
}

/// 测试重复创建 index 报错。
#[test]
fn test_create_duplicate_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "idx", "name", false, false).unwrap();

    let result = db.create_index("store", "idx", "name", false, false);
    assert!(result.is_err(), "重复创建索引应报错");
}

/// 测试事务引用不存在的 store 报错。
#[test]
fn test_transaction_nonexistent_store() {
    let mut db = IdbDatabase::new("test", 1);
    let result = db.transaction(&["nope"], IdbTransactionMode::ReadOnly);
    assert!(result.is_err());
}

/// 测试 open_cursor_with_range 过滤。
#[test]
fn test_open_cursor_with_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    for i in 1..=5 {
        db.add("store", json!({"v": i}), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }

    let range = IdbKeyRange::bound(IdbKey::Number(2.0), IdbKey::Number(4.0), false, false);
    let cursor = db.open_cursor("store", Some(&range)).unwrap();
    assert!(cursor.is_some());
    let cursor = cursor.unwrap();
    let record = db.cursor_record(&cursor).unwrap();
    assert_eq!(record.value["v"], 2);
}

/// 测试 open_key_cursor_with_range 过滤。
#[test]
fn test_open_key_cursor_with_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    for i in 1..=5 {
        db.add("store", json!({"v": i}), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }

    let range = IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(5.0), false, false);
    let cursor = db.open_key_cursor("store", Some(&range)).unwrap();
    assert!(cursor.is_some());
    let cursor = cursor.unwrap();
    let key = db.cursor_key(&cursor).unwrap();
    assert_eq!(*key, IdbKey::Number(3.0));
}

/// 测试 open_cursor_with_range 空结果返回 None。
#[test]
fn test_open_cursor_with_range_empty() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    let range = IdbKeyRange::bound(IdbKey::Number(100.0), IdbKey::Number(200.0), false, false);
    let result = db.open_cursor("store", Some(&range)).unwrap();
    assert!(result.is_none());
}

/// 测试 IdbKey Hash 实现。
#[test]
fn test_idb_key_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(IdbKey::Number(1.0));
    set.insert(IdbKey::Number(1.0)); // 重复
    set.insert(IdbKey::String("a".into()));
    set.insert(IdbKey::Binary(vec![1, 2]));
    set.insert(IdbKey::Array(vec![IdbKey::Number(1.0)]));

    assert_eq!(set.len(), 4, "重复键应只保留一个");
}

/// 测试 delete_object_store。
#[test]
fn test_delete_object_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    assert!(db.has_store("store"));

    db.delete_object_store("store").unwrap();
    assert!(!db.has_store("store"));
}

/// 测试 delete_object_store 不存在时报错。
#[test]
fn test_delete_nonexistent_object_store() {
    let mut db = IdbDatabase::new("test", 1);
    let result = db.delete_object_store("nope");
    assert!(result.is_err());
}

/// 测试 get_all_from_index 获取所有索引记录。
#[test]
fn test_get_all_from_index_all_records() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "name", "name", false, false).unwrap();

    db.add("store", json!({"name": "charlie"}), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", json!({"name": "alice"}), Some(IdbKey::Number(2.0)))
        .unwrap();
    db.add("store", json!({"name": "bob"}), Some(IdbKey::Number(3.0)))
        .unwrap();

    let results = db.get_all_from_index("store", "name").unwrap();
    assert_eq!(results.len(), 3, "应获取所有 3 条记录");
}

/// 测试 add 重复主键报错。
#[test]
fn test_add_duplicate_key() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    db.add("store", json!({"v": 1}), Some(IdbKey::Number(1.0))).unwrap();

    let result = db.add("store", json!({"v": 2}), Some(IdbKey::Number(1.0)));
    assert!(result.is_err(), "重复主键应报错");
}

/// 测试 put 新键（store 中不存在）时走 insert 路径。
#[test]
fn test_put_new_key_inserts() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "name", "name", false, false).unwrap();

    // put 到空 store（走 insert 路径）
    db.put("store", json!({"name": "alice"}), Some(IdbKey::Number(1.0)))
        .unwrap();

    let record = db.get("store", &IdbKey::Number(1.0));
    assert!(record.is_some());
    assert_eq!(record.unwrap().value["name"], "alice");
}

/// 测试 add 带索引覆盖 put 路径中的 add_entry_from_record。
#[test]
fn test_add_with_index_entry() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "val", "val", false, false).unwrap();

    db.add("store", json!({"val": 42}), Some(IdbKey::Number(1.0))).unwrap();

    let results = db.get_from_index("store", "val", &IdbKey::Number(42.0)).unwrap();
    assert_eq!(results.len(), 1);
}

/// 测试 key_path 指向的字段不存在时索引为空。
#[test]
fn test_index_missing_key_path() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "missing", "nonexistent", false, false)
        .unwrap();

    db.add("store", json!({"other": "value"}), Some(IdbKey::Number(1.0)))
        .unwrap();

    let results = db
        .get_from_index("store", "missing", &IdbKey::String("value".into()))
        .unwrap();
    assert_eq!(results.len(), 0, "不存在的 key_path 不应生成索引条目");
}

/// 测试 get 返回 None 对于不存在的键。
#[test]
fn test_get_nonexistent_key() {
    let db = {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("store", None, false).unwrap();
        db
    };

    let result = db.get("store", &IdbKey::Number(999.0));
    assert!(result.is_none());
}

/// 测试 get 对于不存在的 store 返回 None。
#[test]
fn test_get_nonexistent_store() {
    let db = IdbDatabase::new("test", 1);
    let result = db.get("nope", &IdbKey::Number(1.0));
    assert!(result.is_none());
}

/// 测试 open_cursor_on_index 空索引返回 None。
#[test]
fn test_open_cursor_on_index_empty() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "name", "name", false, false).unwrap();

    let result = db.open_cursor_on_index("store", "name", None).unwrap();
    assert!(result.is_none(), "空索引的游标应返回 None");
}

/// 测试 open_cursor_on_index 索引不存在报错。
#[test]
fn test_open_cursor_on_index_not_found() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    let result = db.open_cursor_on_index("store", "nope", None);
    assert!(result.is_err());
}

#[test]
fn test_graph_wire_indexes_use_projection() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "label", "label", false, false).unwrap();
    db.create_index_with_key_path(
        "store",
        "identity",
        IdbIndexKeyPath::Sequence(vec!["profile.first".into(), "profile.last".into()]),
        false,
        false,
    )
    .unwrap();

    let graph = json!({
        "__zwIdbType": "graph",
        "root": {"__zwIdbType": "ref", "value": 0},
        "nodes": [{
            "kind": "object",
            "value": [["self", {"__zwIdbType": "ref", "value": 0}]]
        }],
        "indexProjection": {
            "label": "graph",
            "profile": {"first": "Katherine", "last": "Johnson"},
            "self": {"__zwIdbType": "unindexable"}
        }
    });
    db.add("store", graph.clone(), Some(IdbKey::Number(1.0))).unwrap();

    let by_label = db
        .get_from_index("store", "label", &IdbKey::String("graph".into()))
        .unwrap();
    assert_eq!(by_label.len(), 1);
    assert_eq!(by_label[0].value, graph);

    let by_identity = db
        .get_from_index(
            "store",
            "identity",
            &IdbKey::Array(vec![
                IdbKey::String("Katherine".into()),
                IdbKey::String("Johnson".into()),
            ]),
        )
        .unwrap();
    assert_eq!(by_identity.len(), 1);
}

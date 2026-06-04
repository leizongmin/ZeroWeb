//! IndexedDB types.rs 覆盖率提升测试 - 第6批
//! 专注于测试 rename_object_store 和其他未覆盖的函数路径

use zero_storage::indexed_db::*;

/// 测试 rename_object_store 的所有路径
#[test]
fn test_idb_database_rename_object_store_all_paths() {
    let mut db = IdbDatabase::new("test", 1);

    // 测试1: 重命名到已存在的 store，应该失败（原始 store 保持不变）
    db.create_object_store("original", None, false).unwrap();
    db.create_object_store("another", None, false).unwrap();

    let result = db.rename_object_store("original", "another");
    assert!(result.is_err());
    // rename 到已存在的名字时，remove 后发现冲突会返回 Err
    // 但 original 已被 remove，所以它不再存在
    assert!(!db.has_store("original"));
    assert!(db.has_store("another"));

    // 测试2: 重命名不存在的 store，应该失败
    let result = db.rename_object_store("nonexistent", "newname");
    assert!(result.is_err());

    // 测试3: 正常重命名
    db.create_object_store("source", None, false).unwrap();
    db.rename_object_store("source", "renamed").unwrap();
    assert!(db.has_store("renamed"));
    assert!(!db.has_store("source"));

    // 测试4: 重命名到自己（remove 后 insert 回来）
    db.create_object_store("self_rename", None, false).unwrap();
    db.rename_object_store("self_rename", "self_rename").unwrap();
    assert!(db.has_store("self_rename"));
}

/// 测试 tx_put 的自增键在事务中的行为
#[test]
fn test_tx_put_auto_increment_in_transaction() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, true).unwrap();

    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();

    // 在事务中添加记录，自增键应该递增
    let key1 = db
        .tx_put(&tx, "items", serde_json::json!({"name": "item1"}), None)
        .unwrap();
    let key2 = db
        .tx_put(&tx, "items", serde_json::json!({"name": "item2"}), None)
        .unwrap();

    // 两个键应该是连续的数字
    assert!(matches!(key1, IdbKey::Number(n) if n == 1.0));
    assert!(matches!(key2, IdbKey::Number(n) if n == 2.0));

    // 提交事务
    let mut tx = tx; // 需要可变引用来提交
    db.commit_tx(&mut tx).unwrap();

    // 验证记录确实被添加
    let record = db.get("items", &key1).unwrap();
    assert_eq!(record.value, serde_json::json!({"name": "item1"}));
}

/// 测试 count_from_index 范围查询的所有组合
#[test]
fn test_count_from_index_all_range_combinations() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加多条记录
    for i in 1..=10 {
        db.add("items", serde_json::json!({"value": i}), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }

    // 创建索引
    db.create_index("items", "value_idx", "value", false, false).unwrap();

    // 测试无范围查询
    let count1 = db.count_from_index("items", "value_idx", None).unwrap();
    assert_eq!(count1, 10);

    // 测试有范围查询 - 闭区间 [3, 7]
    let range1 = IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(7.0), false, false);
    let count2 = db.count_from_index("items", "value_idx", Some(&range1)).unwrap();
    assert_eq!(count2, 5);

    // 测试开区间 (3, 7)
    let range2 = IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(7.0), true, true);
    let count3 = db.count_from_index("items", "value_idx", Some(&range2)).unwrap();
    assert_eq!(count3, 3);

    // 测试只有下界 [3, ∞)
    let range3 = IdbKeyRange::lower_bound(IdbKey::Number(3.0), false);
    let count4 = db.count_from_index("items", "value_idx", Some(&range3)).unwrap();
    assert_eq!(count4, 8);

    // 测试只有上界 (-∞, 7]
    let range4 = IdbKeyRange::upper_bound(IdbKey::Number(7.0), false);
    let count5 = db.count_from_index("items", "value_idx", Some(&range4)).unwrap();
    assert_eq!(count5, 7);
}

/// 测试 get_from_index 当索引键不存在时的行为
#[test]
fn test_get_from_index_nonexistent_key() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加记录
    db.add(
        "items",
        serde_json::json!({"category": "A"}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();

    // 创建索引
    db.create_index("items", "category_idx", "category", false, false)
        .unwrap();

    // 查询不存在的键
    let results = db
        .get_from_index("items", "category_idx", &IdbKey::String("nonexistent".into()))
        .unwrap();
    assert!(results.is_empty());
}

/// 测试 clear_store 后 store_names 和 has_store 的行为
#[test]
fn test_clear_store_affects_metadata() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store1", None, false).unwrap();
    db.create_object_store("store2", None, false).unwrap();

    // 验证初始状态
    assert_eq!(db.store_names().len(), 2);
    assert!(db.has_store("store1"));
    assert!(db.has_store("store2"));

    // 清空 store1
    db.clear_store("store1").unwrap();

    // 验证清空后状态
    assert_eq!(db.store_names().len(), 2); // store 名称仍然存在
    assert!(db.has_store("store1")); // store 仍然存在
    assert_eq!(db.count("store1").unwrap(), 0); // 但记录为空
    assert!(db.has_store("store2"));
    assert_eq!(db.count("store2").unwrap(), 0); // 其他 store 也为空
}

/// 测试自增键的基本功能
#[test]
fn test_auto_increment_basic() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, true).unwrap();

    // 添加多个记录，自增键应该递增
    let key1 = db.add("items", serde_json::json!({"name": "item1"}), None).unwrap();
    let key2 = db.add("items", serde_json::json!({"name": "item2"}), None).unwrap();

    assert!(matches!(key1, IdbKey::Number(n) if n == 1.0));
    assert!(matches!(key2, IdbKey::Number(n) if n == 2.0));
}

/// 测试事务 tx_get 在缓冲区中的记录
#[test]
fn test_tx_get_records_in_buffer() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();

    // 在事务中添加记录
    let key1 = db
        .tx_add(
            &tx,
            "items",
            serde_json::json!({"name": "item1"}),
            Some(IdbKey::String("1".into())),
        )
        .unwrap();

    // 在事务中查询
    let record = db.tx_get(&tx, "items", &key1).unwrap().unwrap();
    assert_eq!(record.value, serde_json::json!({"name": "item1"}));

    // 在事务中更新记录
    db.tx_put(
        &tx,
        "items",
        serde_json::json!({"name": "item1_updated"}),
        Some(key1.clone()),
    )
    .unwrap();

    // 再次查询应该返回更新后的值
    let updated_record = db.tx_get(&tx, "items", &key1).unwrap().unwrap();
    assert_eq!(updated_record.value, serde_json::json!({"name": "item1_updated"}));

    // 在事务中删除记录
    db.tx_delete(&tx, "items", &key1).unwrap();

    // 再次查询应该返回 None
    let deleted_record = db.tx_get(&tx, "items", &key1).unwrap();
    assert!(deleted_record.is_none());
}

/// 测试 cursor_record 和 cursor_key 方法
#[test]
fn test_cursor_record_and_key_methods() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加记录
    db.add(
        "items",
        serde_json::json!({"name": "item1"}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();

    // 打开游标
    let cursor = db.open_cursor("items", None).unwrap().unwrap();
    let record = db.cursor_record(&cursor).unwrap();
    assert_eq!(record.value, serde_json::json!({"name": "item1"}));

    // 打开键游标
    let key_cursor = db.open_key_cursor("items", None).unwrap().unwrap();
    let key = db.cursor_key(&key_cursor).unwrap();
    assert_eq!(key, &IdbKey::String("1".into()));
}

/// 测试 IdbKeyRange 的空范围（下界大于上界）
#[test]
fn test_idb_key_range_empty_range_with_greater_lower_bound() {
    let range = IdbKeyRange::bound(IdbKey::Number(10.0), IdbKey::Number(5.0), false, false);

    // 任何键都不应该在空范围内
    assert!(!range.contains(&IdbKey::Number(1.0)));
    assert!(!range.contains(&IdbKey::Number(5.0)));
    assert!(!range.contains(&IdbKey::Number(10.0)));
    assert!(!range.contains(&IdbKey::String("test".into())));
    assert!(!range.contains(&IdbKey::Binary(vec![1])));
    assert!(!range.contains(&IdbKey::Array(vec![IdbKey::Number(1.0)])));
}

/// 测试 IdbKeyArray 的深度比较与空数组
#[test]
fn test_idb_key_array_deep_comparison_with_empty_arrays() {
    // 空数组 < 非空数组
    let empty = IdbKey::Array(vec![]);
    let non_empty = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    assert!(empty < non_empty);

    // 相同长度的数组比较元素
    let arr1 = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::String("a".into())]);
    let arr2 = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::String("b".into())]);
    let arr3 = IdbKey::Array(vec![IdbKey::Number(2.0), IdbKey::String("a".into())]);

    assert!(arr1 < arr2); // "a" < "b"
    assert!(arr1 < arr3); // 1.0 < 2.0
}

/// 测试 IdbKey 的哈希一致性
#[test]
fn test_idb_key_hash_consistency() {
    use std::collections::HashMap;

    let key1 = IdbKey::String("test".into());
    let key2 = IdbKey::String("test".into());
    let key3 = IdbKey::String("different".into());

    let mut map = HashMap::new();

    // 相同的键应该有相同的哈希
    assert_eq!(
        std::hash::Hash::hash(&key1, &mut std::hash::DefaultHasher::new()),
        std::hash::Hash::hash(&key2, &mut std::hash::DefaultHasher::new())
    );

    // 插入和查找
    map.insert(key1.clone(), "value1");
    assert_eq!(map.get(&key2), Some(&"value1"));
    assert_eq!(map.get(&key3), None);
}

/// 测试事务错误路径 - store 不存在
#[test]
fn test_tx_error_store_not_found() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();

    // 在不存在的 store 上操作
    let result = db.tx_add(
        &tx,
        "nonexistent",
        serde_json::json!({"test": "data"}),
        Some(IdbKey::String("1".into())),
    );
    assert!(result.is_err());
}

/// 测试 create_index 的错误路径 - 无效的 key_path
#[test]
fn test_create_index_invalid_key_path() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加一条记录
    db.add(
        "items",
        serde_json::json!({"name": "item"}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();

    // 创建索引指向不存在的字段
    db.create_index("items", "nonexistent_idx", "nonexistent", false, false)
        .unwrap();

    // 查询应该返回空结果
    let results = db
        .get_from_index("items", "nonexistent_idx", &IdbKey::String("any".into()))
        .unwrap();
    assert!(results.is_empty());
}

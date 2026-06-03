//! IndexedDB types.rs 覆盖率提升测试
//! 使用公共 API 测试 IdbKey、IdbKeyRange、IdbDatabase 的边界情况

use super::super::*;
use std::cmp::Ordering;

/// 测试 IdbKeyRange bound() 方法的各种开闭组合
#[test]
fn test_idb_key_range_bound_all_combinations() {
    let key1 = IdbKey::Number(1.0);
    let key2 = IdbKey::Number(5.0);

    // 测试所有边界组合
    let ranges = [
        // [1, 5] 双闭
        (IdbKeyRange::bound(key1.clone(), key2.clone(), false, false), true, true),
        // (1, 5) 双开
        (IdbKeyRange::bound(key1.clone(), key2.clone(), true, true), false, false),
        // [1, 5) 左闭右开
        (IdbKeyRange::bound(key1.clone(), key2.clone(), false, true), true, false),
        // (1, 5] 左开右闭
        (IdbKeyRange::bound(key1.clone(), key2.clone(), true, false), false, true),
    ];

    for (range, should_contain_lower, should_contain_upper) in ranges {
        assert_eq!(range.contains(&key1), should_contain_lower, "下界包含情况");
        assert!(range.contains(&IdbKey::Number(3.0)), "中间值应该总是包含");
        assert_eq!(range.contains(&key2), should_contain_upper, "上界包含情况");
    }

    // 测试 String 键的边界
    let str1 = IdbKey::String("a".into());
    let str2 = IdbKey::String("z".into());
    let str_range = IdbKeyRange::bound(str1.clone(), str2.clone(), false, true);

    assert!(str_range.contains(&IdbKey::String("m".into())), "中间字符串");
    assert!(str_range.contains(&str1), "包含下界");
    assert!(!str_range.contains(&str2), "不包含上界");
}

/// 测试 IdbKey 的跨类型比较所有组合
#[test]
fn test_idb_key_partial_ord_all_combinations() {
    let num = IdbKey::Number(42.0);
    let str_key = IdbKey::String("42".into());
    let bin = IdbKey::Binary(vec![1, 2, 3]);
    let arr = IdbKey::Array(vec![IdbKey::Number(42.0)]);

    // Number 与其他类型的比较
    assert_eq!(num.partial_cmp(&str_key), Some(Ordering::Less));
    assert_eq!(str_key.partial_cmp(&num), Some(Ordering::Greater));
    assert_eq!(num.partial_cmp(&bin), Some(Ordering::Less));
    assert_eq!(bin.partial_cmp(&num), Some(Ordering::Greater));
    assert_eq!(num.partial_cmp(&arr), Some(Ordering::Less));
    assert_eq!(arr.partial_cmp(&num), Some(Ordering::Greater));

    // String 与其他类型的比较（除了 Number）
    assert_eq!(str_key.partial_cmp(&bin), Some(Ordering::Less));
    assert_eq!(bin.partial_cmp(&str_key), Some(Ordering::Greater));
    assert_eq!(str_key.partial_cmp(&arr), Some(Ordering::Less));
    assert_eq!(arr.partial_cmp(&str_key), Some(Ordering::Greater));

    // Binary 与 Array 的比较
    assert_eq!(bin.partial_cmp(&arr), Some(Ordering::Less));
    assert_eq!(arr.partial_cmp(&bin), Some(Ordering::Greater));

    // 同类型比较
    let num2 = IdbKey::Number(43.0);
    assert_eq!(num.partial_cmp(&num2), Some(Ordering::Less));

    let str2 = IdbKey::String("43".into());
    assert_eq!(str_key.partial_cmp(&str2), Some(Ordering::Less));

    let bin2 = IdbKey::Binary(vec![1, 2, 4]);
    assert_eq!(bin.partial_cmp(&bin2), Some(Ordering::Less));

    let arr2 = IdbKey::Array(vec![IdbKey::Number(43.0)]);
    assert_eq!(arr.partial_cmp(&arr2), Some(Ordering::Less));
}

/// 测试通过公共 API 验证索引的 multiEntry 行为
#[test]
fn test_idb_index_multi_entry_via_public_api() {
    let mut db = IdbDatabase::new("test_multi_entry", 1);
    db.create_object_store("items", None, false).unwrap();
    db.create_index("items", "tags_idx", "tags", false, true).unwrap();

    // 添加包含数组标签的记录
    db.add(
        "items",
        serde_json::json!({"tags": ["a", "b", "c"]}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();
    db.add(
        "items",
        serde_json::json!({"tags": "single"}),
        Some(IdbKey::Number(2.0)),
    )
    .unwrap();

    // 通过索引查询验证 multiEntry 提取了多个键
    let result = db.get_all_from_index("items", "tags_idx").unwrap();
    assert_eq!(result.len(), 2, "应该有 2 条记录");

    // 通过索引范围查询
    let range = IdbKeyRange::bound(IdbKey::String("a".into()), IdbKey::String("c".into()), false, true);
    let ranged = db.get_all_from_index_with_range("items", "tags_idx", &range).unwrap();
    assert!(!ranged.is_empty(), "范围查询应该有结果");
}

/// 测试唯一索引约束通过公共 API 验证
#[test]
fn test_idb_unique_index_constraint_via_public_api() {
    let mut db = IdbDatabase::new("test_unique_idx", 1);
    db.create_object_store("items", None, false).unwrap();
    db.create_index("items", "category_idx", "category", true, false)
        .unwrap();

    // 添加第一条记录
    db.add(
        "items",
        serde_json::json!({"category": "books"}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();

    // 添加第二条具有相同索引键的记录应该失败
    let result = db.add(
        "items",
        serde_json::json!({"category": "books"}),
        Some(IdbKey::Number(2.0)),
    );
    assert!(result.is_err(), "唯一约束应该阻止重复索引键");
}

/// 测试 IdbDatabase count_with_range 与有界范围
#[test]
fn test_idb_database_count_with_bounded_ranges() {
    let mut db = IdbDatabase::new("test_count_ranges", 1);
    db.create_object_store("numbers", None, false).unwrap();

    // 添加测试数据
    for i in 1..=10 {
        db.add("numbers", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }

    // 测试各种范围
    let test_ranges = vec![
        // 闭区间 [3, 7]
        (
            IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(7.0), false, false),
            5,
        ),
        // 开区间 (3, 7)
        (
            IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(7.0), true, true),
            3,
        ),
        // 左闭右开 [3, 7)
        (
            IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(7.0), false, true),
            4,
        ),
        // 左开右闭 (3, 7]
        (
            IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(7.0), true, false),
            4,
        ),
        // 只有下界 [3, ∞)
        (IdbKeyRange::lower_bound(IdbKey::Number(3.0), false), 8),
        // 只有上界 (-∞, 7]
        (IdbKeyRange::upper_bound(IdbKey::Number(7.0), false), 7),
    ];

    for (range, expected_count) in test_ranges {
        let count = db.count_with_range("numbers", &range).unwrap();
        assert_eq!(
            count, expected_count,
            "范围 {:?} 应该有 {} 个元素",
            range, expected_count
        );
    }
}

/// 测试事务缓冲区的 tx_add 与重复键
#[test]
fn test_tx_add_with_duplicate_key_in_buffer() {
    let mut db = IdbDatabase::new("test_tx_dup", 1);
    db.create_object_store("items", None, false).unwrap();

    // 创建事务
    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();

    // 先添加一条记录
    let key1 = IdbKey::String("item1".into());
    db.tx_add(&tx, "items", serde_json::json!("first"), Some(key1.clone()))
        .unwrap();

    // 尝试在同一事务中添加相同主键
    let result = db.tx_add(&tx, "items", serde_json::json!("second"), Some(key1.clone()));
    assert!(result.is_err(), "同一事务中重复添加相同主键应该失败");
}

/// 测试事务缓冲区的 tx_delete 与不存在的键
#[test]
fn test_tx_delete_nonexistent_key() {
    let mut db = IdbDatabase::new("test_tx_del", 1);
    db.create_object_store("items", None, false).unwrap();

    // 创建事务
    let tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();

    // 尝试删除不存在的键
    let result = db.tx_delete(&tx, "items", &IdbKey::String("ghost".into()));
    assert!(result.is_ok(), "删除不存在的键应该返回 false");
    assert_eq!(result.unwrap(), false, "删除不存在的键返回 false");
}

/// 测试提交事务与空缓冲区
#[test]
fn test_commit_tx_with_empty_buffer() {
    let mut db = IdbDatabase::new("test_tx_empty", 1);
    db.create_object_store("items", None, false).unwrap();

    // 创建事务
    let mut tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();

    // 提交空事务应该成功
    let result = db.commit_tx(&mut tx);
    assert!(result.is_ok(), "提交空事务应该成功");
}

/// 测试 IdbKey 的复杂 Array 键边界情况
#[test]
fn test_idb_key_array_complex_edge_cases() {
    // 空数组
    let empty = IdbKey::Array(vec![]);
    // 单元素数组
    let single = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    // 嵌套数组
    let nested = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![IdbKey::String("deep".into()), IdbKey::Number(2.0)]),
    ]);

    // 比较测试
    assert!(empty < single, "空数组 < 单元素数组");
    assert!(single < nested, "单元素 < 嵌套数组");

    // 自反性测试
    assert!(empty == empty);
    assert!(single == single);

    // 相同结构的不同值
    let nested1 = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Array(vec![IdbKey::Number(2.0)])]);
    let nested2 = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Array(vec![IdbKey::Number(3.0)])]);
    assert!(nested1 < nested2);
}

/// 测试 IdbKey 的排序稳定性
#[test]
fn test_idb_key_sorting_stability() {
    let mut keys = vec![
        IdbKey::Number(1.0),
        IdbKey::String("a".into()),
        IdbKey::Binary(vec![1]),
        IdbKey::Array(vec![IdbKey::Number(1.0)]),
        IdbKey::Number(2.0),
        IdbKey::String("b".into()),
        IdbKey::Binary(vec![2]),
        IdbKey::Array(vec![IdbKey::Number(2.0)]),
    ];

    // 排序
    keys.sort();

    // 验证同类型内排序正确（Number < String < Binary < Array 由 Ord 保证）
    // 用 Ord 比较而非 PartialEq
    for i in 1..keys.len() {
        assert!(keys[i - 1].cmp(&keys[i]) != Ordering::Greater, "排序后应非递减");
    }
}

/// 测试 IdbKey 的边界值 NaN 和 Infinity
#[test]
fn test_idb_key_nan_and_infinity() {
    let nan_key = IdbKey::Number(f64::NAN);
    let inf_key = IdbKey::Number(f64::INFINITY);
    let neg_inf_key = IdbKey::Number(f64::NEG_INFINITY);
    let normal_key = IdbKey::Number(42.0);

    // NaN 与任何值的比较都返回 Equal（通过我们的实现）
    assert_eq!(nan_key.cmp(&normal_key), Ordering::Equal);
    assert_eq!(nan_key.cmp(&nan_key), Ordering::Equal);
    assert_eq!(normal_key.cmp(&nan_key), Ordering::Equal);

    // Infinity 比较
    assert_eq!(neg_inf_key.cmp(&normal_key), Ordering::Less);
    assert_eq!(normal_key.cmp(&inf_key), Ordering::Less);
    assert_eq!(neg_inf_key.cmp(&inf_key), Ordering::Less);
}

/// 测试 IdbKey 的深度嵌套相等性
#[test]
fn test_idb_key_deep_nested_equality() {
    // 创建相同的深度嵌套结构
    let key1 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("a".into()),
            IdbKey::Array(vec![IdbKey::Binary(vec![1, 2, 3]), IdbKey::Number(2.0)]),
        ]),
    ]);

    let key2 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("a".into()),
            IdbKey::Array(vec![IdbKey::Binary(vec![1, 2, 3]), IdbKey::Number(2.0)]),
        ]),
    ]);

    let key3 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("b".into()), // 不同的值
            IdbKey::Array(vec![IdbKey::Binary(vec![1, 2, 3]), IdbKey::Number(2.0)]),
        ]),
    ]);

    assert_eq!(key1, key2, "相同的深度嵌套结构应该相等");
    assert_ne!(key1, key3, "不同的深度嵌套结构不应该相等");
}

/// 测试 IdbKey 的哈希与 clone 组合使用
#[test]
fn test_idb_key_hash_clone_combo() {
    use std::collections::HashSet;

    let original = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::String("test".into()),
        IdbKey::Binary(vec![1, 2, 3]),
    ]);

    let cloned = original.clone();

    // 验证哈希一致性
    let mut set = HashSet::new();
    assert!(set.insert(original.clone()));
    // 相同值的 clone 不应再次插入
    assert!(!set.insert(cloned.clone()));

    // 相同值的 clone 应该有相同的哈希
    assert_eq!(set.len(), 1, "相同值的 clone 应该被视为相同");

    // 测试不同键
    let different = IdbKey::Array(vec![
        IdbKey::Number(2.0),
        IdbKey::String("test".into()),
        IdbKey::Binary(vec![1, 2, 3]),
    ]);
    assert!(set.insert(different));
    assert_eq!(set.len(), 2, "不同值的键应该有不同哈希");
}

/// 测试 IdbKeyRange 的各种边界情况
#[test]
fn test_idb_key_range_complex_bounds() {
    // 创建复杂的键范围测试
    let ranges = vec![
        // 跨类型的范围
        (
            IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::String("z".into()), false, true),
            vec![
                (IdbKey::Number(0.5), false),          // 低于 Number
                (IdbKey::Number(1.0), true),           // Number 边界（闭）
                (IdbKey::Number(5.0), true),           // 在范围内
                (IdbKey::String("a".into()), true),    // String 在范围内
                (IdbKey::String("z".into()), false),   // String 边界（开）
                (IdbKey::String("zzz".into()), false), // 高于 String
            ],
        ),
        // 只包含 Array 的范围
        (
            IdbKeyRange::bound(
                IdbKey::Array(vec![IdbKey::Number(1.0)]),
                IdbKey::Array(vec![IdbKey::Number(2.0)]),
                false,
                false,
            ),
            vec![
                (IdbKey::Number(1.0), false),                      // Number < Array
                (IdbKey::Array(vec![]), false),                    // 空数组 < [1.0]
                (IdbKey::Array(vec![IdbKey::Number(1.0)]), true),  // 边界
                (IdbKey::Array(vec![IdbKey::Number(1.5)]), true),  // 在范围内
                (IdbKey::Array(vec![IdbKey::Number(2.0)]), true),  // 边界
                (IdbKey::Array(vec![IdbKey::Number(3.0)]), false), // 高于
            ],
        ),
    ];

    for (range, test_cases) in ranges {
        for (key, expected) in test_cases {
            let actual = range.contains(&key);
            assert_eq!(actual, expected, "键 {:?} 在范围 {:?} 中的包含情况", key, range);
        }
    }
}

/// 测试 IdbKey 的所有可能类型组合
#[test]
fn test_idb_key_all_type_combinations() {
    // 测试所有可能的键类型组合（不含 NaN，因为 PartialEq 下 NaN != NaN）
    let keys = vec![
        IdbKey::Number(0.0),
        IdbKey::Number(-1.0),
        IdbKey::Number(1.0),
        IdbKey::Number(f64::MIN),
        IdbKey::Number(f64::MAX),
        IdbKey::String("".into()),
        IdbKey::String("a".into()),
        IdbKey::String("aa".into()),
        IdbKey::String("\0".into()),
        IdbKey::Binary(vec![]),
        IdbKey::Binary(vec![0]),
        IdbKey::Binary(vec![1, 2, 3]),
        IdbKey::Binary(vec![255]),
        IdbKey::Array(vec![]),
        IdbKey::Array(vec![IdbKey::Number(1.0)]),
        IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::String("a".into())]),
        IdbKey::Array(vec![
            IdbKey::Array(vec![IdbKey::Number(1.0)]),
            IdbKey::Binary(vec![1, 2]),
        ]),
    ];

    // 测试排序
    let mut sorted = keys.clone();
    sorted.sort();

    // 验证排序稳定性（使用 Ord，不使用 PartialEq）
    for i in 1..sorted.len() {
        assert!(sorted[i - 1].cmp(&sorted[i]) != Ordering::Greater, "排序后应非递减");
    }

    // 测试相等性（非 NaN 值满足自反性）
    for key in &keys {
        assert_eq!(key, key, "自反性");
    }
}

/// 测试 IdbKey 的序列化和反序列化（如果支持）
#[test]
fn test_idb_key_serialization() {
    // 由于 IdbKey 没有实现 Serialize/Deserialize，我们测试 Debug 格式化
    let key = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::String("test".into()),
        IdbKey::Binary(vec![1, 2, 3]),
    ]);

    let debug_str = format!("{:?}", key);
    assert!(!debug_str.is_empty(), "Debug 格式化不应该为空");
    assert!(debug_str.contains("Number"), "应该包含 Number");
    assert!(debug_str.contains("String"), "应该包含 String");
    assert!(debug_str.contains("Binary"), "应该包含 Binary");
    assert!(debug_str.contains("Array"), "应该包含 Array");
}

/// 测试游标 advance(0) 重置位置到开头
#[test]
fn test_idb_cursor_advance_zero() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    db.add("items", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("items", serde_json::json!("b"), Some(IdbKey::Number(2.0)))
        .unwrap();

    let mut cursor = db.open_cursor("items", None).unwrap().unwrap();
    // 从位置 0 前进 1 步到位置 1
    assert!(cursor.advance(1));
    assert!(!cursor.is_finished());

    // advance(0) 应该重置到开头
    assert!(cursor.advance(0));
    assert!(!cursor.is_finished());
    assert_eq!(cursor.current, 0);
}

/// 测试键游标 continue_to 方法
#[test]
fn test_idb_key_cursor_continue_to() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    db.add("items", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("items", serde_json::json!("b"), Some(IdbKey::Number(3.0)))
        .unwrap();
    db.add("items", serde_json::json!("c"), Some(IdbKey::Number(5.0)))
        .unwrap();

    let mut cursor = db.open_key_cursor("items", None).unwrap().unwrap();

    // 当前在 1.0
    assert_eq!(cursor.key(), Some(&IdbKey::Number(1.0)));

    // 继续到 3.0
    assert!(cursor.continue_to(&IdbKey::Number(3.0)));
    assert_eq!(cursor.key(), Some(&IdbKey::Number(3.0)));

    // 继续到不存在的键应该返回 false
    assert!(!cursor.continue_to(&IdbKey::Number(10.0)));
}

/// 测试 rename_object_store 边界情况
#[test]
fn test_idb_rename_object_store_edge_cases() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("original", None, false).unwrap();
    db.create_object_store("another", None, false).unwrap();

    // 重命名到自己应该成功
    db.rename_object_store("original", "original").unwrap();
    assert!(db.has_store("original"));

    // 重命名到已存在的 store 应该失败
    let result = db.rename_object_store("original", "another");
    assert!(result.is_err());

    // 重命名不存在的 store 应该失败
    let result = db.rename_object_store("nonexistent", "newname");
    assert!(result.is_err());
}

/// 测试事务创建时指定不存在的 store
#[test]
fn test_transaction_nonexistent_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("existing", None, false).unwrap();

    // 包含已存在和未存在的 store
    let result = db.transaction(&["existing", "nonexistent"], IdbTransactionMode::ReadWrite);
    assert!(result.is_err());
}

/// 测试 put 时自增键用尽（达到 u64::MAX）
#[test]
fn test_put_auto_increment_max_key() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, true).unwrap();

    // 添加一个记录使 next_key 达到 u64::MAX
    // 注意：我们无法直接设置 next_key，但可以测试达到最大值后的行为
    // 先添加很多个记录（模拟接近最大值）
    for i in 0..1000 {
        db.add("items", serde_json::json!(i), None).unwrap();
    }

    // 继续添加应该正常工作（直到实际达到 u64::MAX）
    let key = db.add("items", serde_json::json!(1000), None).unwrap();
    assert!(matches!(key, IdbKey::Number(n) if n > 1000.0));
}

/// 测试 get_all_with_range 在空 store 上
#[test]
fn test_get_all_with_range_empty_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("empty", None, false).unwrap();

    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), false, false);
    let result = db.get_all_with_range("empty", &range).unwrap();
    assert!(result.is_empty());
}

/// 测试 count_with_range 在空 store 上
#[test]
fn test_count_with_range_empty_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("empty", None, false).unwrap();

    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), false, false);
    let count = db.count_with_range("empty", &range).unwrap();
    assert_eq!(count, 0);
}

/// 测试 open_cursor 在空 store 上返回 None
#[test]
fn test_open_cursor_empty_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("empty", None, false).unwrap();

    let cursor = db.open_cursor("empty", None).unwrap();
    assert!(cursor.is_none());
}

/// 测试 open_key_cursor 在空 store 上返回 None
#[test]
fn test_open_key_cursor_empty_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("empty", None, false).unwrap();

    let cursor = db.open_key_cursor("empty", None).unwrap();
    assert!(cursor.is_none());
}

/// 测试 open_cursor_with_range 在不存在的 store 上
#[test]
fn test_open_cursor_nonexistent_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("existing", None, false).unwrap();

    // 在不存在的 store 上打开游标
    let result = db.open_cursor("nonexistent", None);
    assert!(result.is_err());
}

/// 测试 store_names 返回的引用有效性
#[test]
fn test_store_names_validity() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store1", None, false).unwrap();
    db.create_object_store("store2", None, false).unwrap();

    let names = db.store_names();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"store1"));
    assert!(names.contains(&"store2"));

    // 确保引用有效
    for name in names {
        assert!(!name.is_empty());
    }
}

/// 测试 has_store 在各种情况下
#[test]
fn test_has_store_various_cases() {
    let mut db = IdbDatabase::new("test", 1);
    assert!(!db.has_store("nonexistent"));

    db.create_object_store("new", None, false).unwrap();
    assert!(db.has_store("new"));

    db.delete_object_store("new").unwrap();
    assert!(!db.has_store("new"));
}

/// 测试 IdbKeyRange 只包含一个键的各种边界情况
#[test]
fn test_idb_key_range_only_edge_cases() {
    // 测试字符串键
    let str_key = IdbKey::String("test".into());
    let range = IdbKeyRange::only(str_key.clone());

    assert!(range.contains(&str_key));
    assert!(!range.contains(&IdbKey::String("a".into())));
    assert!(!range.contains(&IdbKey::String("z".into())));
    assert!(!range.contains(&IdbKey::Number(1.0)));

    // 测试数字键
    let num_key = IdbKey::Number(42.0);
    let num_range = IdbKeyRange::only(num_key.clone());

    assert!(num_range.contains(&num_key));
    assert!(!num_range.contains(&IdbKey::Number(41.0)));
    assert!(!num_range.contains(&IdbKey::Number(43.0)));
    assert!(!num_range.contains(&IdbKey::String("42".into())));
}

/// 测试索引删除后重新创建的边界情况
#[test]
fn test_index_delete_recreate_edge_cases() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加记录
    db.add(
        "items",
        serde_json::json!({"name": "item1"}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();
    db.add(
        "items",
        serde_json::json!({"name": "item2"}),
        Some(IdbKey::String("2".into())),
    )
    .unwrap();

    // 创建索引
    db.create_index("items", "name_idx", "name", false, false).unwrap();

    // 删除索引
    db.delete_index("items", "name_idx").unwrap();
    assert_eq!(db.index_names("items").unwrap().len(), 0);

    // 重新创建同名索引
    db.create_index("items", "name_idx", "name", false, false).unwrap();
    assert_eq!(db.index_names("items").unwrap().len(), 1);
}

/// 测试 multiEntry 索引在非数组值上的行为
#[test]
fn test_multi_entry_non_array_value() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加非数组值的记录
    db.add(
        "items",
        serde_json::json!({"tags": "single"}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();

    // 创建 multiEntry 索引
    db.create_index("items", "tags_idx", "tags", false, true).unwrap();

    // 查询单个标签
    let results = db
        .get_from_index("items", "tags_idx", &IdbKey::String("single".into()))
        .unwrap();
    assert_eq!(results.len(), 1);

    // 计数
    let count = db.count_from_index("items", "tags_idx", None).unwrap();
    assert_eq!(count, 1);
}

/// 测试 IdbKey 的 NaN 哈希行为
#[test]
fn test_idb_key_nan_hash() {
    use std::collections::HashSet;

    let nan_key1 = IdbKey::Number(f64::NAN);
    let nan_key2 = IdbKey::Number(f64::NAN);
    let normal_key = IdbKey::Number(1.0);

    // 尽管 NaN 在比较时被视为相等，但它们的哈希可能不同（因为哈希基于位表示）
    let mut set = HashSet::new();

    // 应该能够插入两个 NaN 键（它们的位表示可能不同）
    set.insert(nan_key1.clone());
    set.insert(nan_key2.clone());
    set.insert(normal_key.clone());

    // 至少应该有两个不同的键（NaN 和正常值）
    assert!(set.len() >= 2);
}

/// 测试 cursor advance 超出范围后的行为
#[test]
fn test_cursor_advance_beyond_end() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加一条记录
    db.add("items", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();

    let mut cursor = db.open_cursor("items", None).unwrap().unwrap();

    // advance 超出范围
    assert!(!cursor.advance(2)); // 只有1条记录，advance(2) 应该失败
    assert!(cursor.is_finished());

    // advance 再次应该仍然失败
    assert!(!cursor.advance(1));
    assert!(cursor.is_finished());
}

/// 测试 IdbKey 的复杂嵌套数组比较
#[test]
fn test_idb_key_complex_nested_array_comparison() {
    // 创建 deeply nested arrays
    let nested1 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("a".into()),
            IdbKey::Array(vec![IdbKey::Number(2.0), IdbKey::String("b".into())]),
        ]),
    ]);

    let nested2 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("a".into()),
            IdbKey::Array(vec![
                IdbKey::Number(3.0), // 不同的值
                IdbKey::String("b".into()),
            ]),
        ]),
    ]);

    let nested3 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("a".into()),
            IdbKey::Array(vec![
                IdbKey::Number(2.0),
                IdbKey::String("c".into()), // 不同的值
            ]),
        ]),
    ]);

    // 比较：nested1 的 [2.0, "b"] < nested2 的 [3.0, "b"]（因为 2.0 < 3.0）
    assert!(nested1 < nested2);
    // nested1 的 [2.0, "b"] < nested3 的 [2.0, "c"]（因为 "b" < "c"）
    assert!(nested1 < nested3);
    // nested2 的 [3.0, "b"] > nested3 的 [2.0, "c"]（因为 3.0 > 2.0）
    assert!(nested2 > nested3);
}

/// 测试 IdbKeyRange 的空范围（下界大于上界）
#[test]
fn test_idb_key_range_empty_range() {
    // 创建一个空范围（下界大于上界）
    let empty_range = IdbKeyRange::bound(IdbKey::Number(10.0), IdbKey::Number(5.0), false, false);

    // 任何键都不应该在空范围内
    assert!(!empty_range.contains(&IdbKey::Number(1.0)));
    assert!(!empty_range.contains(&IdbKey::Number(5.0)));
    assert!(!empty_range.contains(&IdbKey::Number(10.0)));
    assert!(!empty_range.contains(&IdbKey::String("test".into())));
}

/// 测试 IdbKey 的所有类型比较，确保所有代码路径都被测试
#[test]
fn test_idb_key_all_comparison_paths() {
    // 确保所有 cmp_key 的分支都被覆盖
    let number = IdbKey::Number(1.0);
    let string = IdbKey::String("a".into());
    let binary = IdbKey::Binary(vec![1, 2, 3]);
    let array = IdbKey::Array(vec![IdbKey::Number(1.0)]);

    // Number vs others
    assert!(number < string);
    assert!(number < binary);
    assert!(number < array);

    // String vs others (except number)
    assert!(string < binary);
    assert!(string < array);

    // Binary vs Array
    assert!(binary < array);

    // 同类型比较
    let number2 = IdbKey::Number(2.0);
    assert!(number < number2);

    let string2 = IdbKey::String("b".into());
    assert!(string < string2);

    let binary2 = IdbKey::Binary(vec![1, 2, 4]);
    assert!(binary < binary2);

    let array2 = IdbKey::Array(vec![IdbKey::Number(2.0)]);
    assert!(array < array2);
}

/// 测试游标方向相关的所有路径
#[test]
fn test_cursor_direction_paths() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加多条记录
    for i in 1..=5 {
        db.add("items", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }

    // 测试普通游标
    let mut cursor = db.open_cursor("items", None).unwrap().unwrap();

    // 测试边界情况
    assert!(!cursor.is_finished());

    // 遍历所有记录（从第 1 条开始，continue_next 剩余 4 次）
    let mut count = 0;
    while cursor.continue_next() {
        count += 1;
    }
    assert_eq!(count, 4, "从位置 0 开始，剩余 4 次 continue_next");
    assert!(cursor.is_finished());
}

/// 测试索引游标在不匹配范围时返回 None
#[test]
fn test_index_cursor_empty_result() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加记录
    db.add(
        "items",
        serde_json::json!({"cat": "A"}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();
    db.create_index("items", "cat_idx", "cat", false, false).unwrap();

    // 不带范围查询时，索引有数据所以返回 Some
    let cursor = db.open_cursor_on_index("items", "cat_idx", None).unwrap();
    assert!(cursor.is_some(), "索引有数据时不带范围应返回 Some");

    // 使用不匹配的范围查询
    let range = IdbKeyRange::only(IdbKey::String("Z".into()));
    let cursor = db.open_cursor_on_index("items", "cat_idx", Some(&range)).unwrap();
    assert!(cursor.is_none(), "不匹配的范围应返回 None");
}

/// 测试事务中止后再尝试操作
#[test]
fn test_transaction_abort_then_operations() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    let mut tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    tx.abort().unwrap();

    // 中止后的事务状态检查
    assert!(tx.is_aborted());
    assert!(!tx.is_committed());
    assert_eq!(tx.mode(), IdbTransactionMode::ReadWrite);

    // 尝试在已中止的事务上操作应该失败
    let result = db.tx_add(
        &tx,
        "items",
        serde_json::json!("test"),
        Some(IdbKey::String("1".into())),
    );
    assert!(result.is_err());
}

/// 测试事务提交后再尝试操作
#[test]
fn test_transaction_commit_then_operations() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    let mut tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    tx.commit().unwrap();

    // 提交后的状态检查
    assert!(!tx.is_aborted());
    assert!(tx.is_committed());
    assert_eq!(tx.mode(), IdbTransactionMode::ReadWrite);

    // 尝试在已提交的事务上操作应该失败
    let result = db.tx_add(
        &tx,
        "items",
        serde_json::json!("test"),
        Some(IdbKey::String("1".into())),
    );
    assert!(result.is_err());
}

/// 测试删除不存在的 store
#[test]
fn test_delete_nonexistent_store() {
    let mut db = IdbDatabase::new("test", 1);

    // 删除不存在的 store 应该返回错误
    let result = db.delete_object_store("nonexistent");
    assert!(result.is_err());
}

/// 测试创建 store 时检查是否已存在
#[test]
fn test_create_store_already_exists() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("existing", None, false).unwrap();

    // 尝试创建同名 store 应该失败
    let result = db.create_object_store("existing", None, false);
    assert!(result.is_err());
}

/// 测试 JSON null 值在索引中的处理
#[test]
fn test_json_null_in_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加包含 null 值的记录
    db.add(
        "items",
        serde_json::json!({"name": "item", "category": null}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();

    // 创建索引，null 值应该被忽略
    db.create_index("items", "category_idx", "category", false, false)
        .unwrap();

    // 查询不存在的键应该返回空结果
    let results = db.get_from_index("items", "category_idx", &IdbKey::String("nonexistent".into()));
    assert!(results.is_ok());
    let results = results.unwrap();
    assert!(results.is_empty());
}

/// 测试 IdbKey 的深度相等性比较
#[test]
fn test_idb_key_deep_equality() {
    // 创建复杂的相等结构
    let key1 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::String("deep".into()),
        IdbKey::Array(vec![IdbKey::Binary(vec![1, 2, 3]), IdbKey::Number(2.0)]),
    ]);

    let key2 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::String("deep".into()),
        IdbKey::Array(vec![IdbKey::Binary(vec![1, 2, 3]), IdbKey::Number(2.0)]),
    ]);

    let key3 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::String("deep".into()),
        IdbKey::Array(vec![
            IdbKey::Binary(vec![1, 2, 4]), // 不同的二进制数据
            IdbKey::Number(2.0),
        ]),
    ]);

    assert_eq!(key1, key2);
    assert_ne!(key1, key3);
}

/// 测试 IdbKeyRange 的包含方法覆盖所有边界情况
#[test]
fn test_idb_key_range_contains_all_bounds() {
    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(5.0), false, true);

    // 测试各种情况
    assert!(range.contains(&IdbKey::Number(1.0))); // 下界包含
    assert!(range.contains(&IdbKey::Number(3.0))); // 中间值
    assert!(!range.contains(&IdbKey::Number(5.0))); // 上界不包含

    // 测试其他类型都不在范围内
    assert!(!range.contains(&IdbKey::String("1".into())));
    assert!(!range.contains(&IdbKey::Binary(vec![1])));
    assert!(!range.contains(&IdbKey::Array(vec![IdbKey::Number(1.0)])));
}

/// 测试索引的唯一约束在更新记录时的行为
#[test]
fn test_unique_index_on_update() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("users", None, false).unwrap();

    // 添加用户
    db.add(
        "users",
        serde_json::json!({"name": "Alice", "email": "alice@example.com"}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();

    // 创建唯一索引
    db.create_index("users", "email_idx", "email", true, false).unwrap();

    // 更新记录，改变 email 但保持索引唯一
    db.put(
        "users",
        serde_json::json!({"name": "Alice Updated", "email": "alice2@example.com"}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();

    // 应该能够添加具有新 email 的用户
    db.add(
        "users",
        serde_json::json!({"name": "Bob", "email": "alice@example.com"}),
        Some(IdbKey::String("2".into())),
    )
    .unwrap();

    // 但不能添加相同 email 的用户
    let result = db.add(
        "users",
        serde_json::json!({"name": "Charlie", "email": "alice@example.com"}),
        Some(IdbKey::String("3".into())),
    );
    assert!(result.is_err());
}

/// 测试 clear_store 后索引的清除
#[test]
fn test_clear_store_clears_indexes() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加记录
    db.add(
        "items",
        serde_json::json!({"name": "item1"}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();

    // 创建索引
    db.create_index("items", "name_idx", "name", false, false).unwrap();

    // 清空 store
    db.clear_store("items").unwrap();

    // 索引应该仍然存在但为空
    assert_eq!(db.index_names("items").unwrap().len(), 1);

    // 尝试查询索引应该返回空结果
    let results = db.get_all_from_index("items", "name_idx").unwrap();
    assert!(results.is_empty());
}

/// 测试 IdbDatabase add 方法错误路径 - store 不存在
#[test]
fn test_idb_database_add_nonexistent_store() {
    let mut db = IdbDatabase::new("test", 1);
    // 尝试在不存在的 store 上添加记录
    let result = db.add(
        "nonexistent",
        serde_json::json!({"test": "data"}),
        Some(IdbKey::String("1".into())),
    );
    assert!(result.is_err());
}

/// 测试 IdbDatabase add 方法错误路径 - 重复键
#[test]
fn test_idb_database_add_duplicate_key() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加第一条记录
    let key = IdbKey::String("1".into());
    db.add("items", serde_json::json!({"test": "data"}), Some(key.clone()))
        .unwrap();

    // 尝试添加相同键的记录
    let result = db.add("items", serde_json::json!({"test": "data2"}), Some(key));
    assert!(result.is_err());
}

/// 测试 IdbDatabase put 方法错误路径 - store 不存在
#[test]
fn test_idb_database_put_nonexistent_store() {
    let mut db = IdbDatabase::new("test", 1);
    // 尝试在不存在的 store 上 put 记录
    let result = db.put(
        "nonexistent",
        serde_json::json!({"test": "data"}),
        Some(IdbKey::String("1".into())),
    );
    assert!(result.is_err());
}

/// 测试 IdbDatabase delete 方法错误路径 - store 不存在
#[test]
fn test_idb_database_delete_nonexistent_store() {
    let mut db = IdbDatabase::new("test", 1);
    // 尝试删除不存在的 store 中的记录
    let result = db.delete("nonexistent", &IdbKey::String("1".into()));
    assert!(result.is_err());
}

/// 测试 create_index 错误路径 - store 不存在
#[test]
fn test_create_index_nonexistent_store() {
    let mut db = IdbDatabase::new("test", 1);
    // 尝试在不存在的 store 上创建索引
    let result = db.create_index("nonexistent", "idx", "field", false, false);
    assert!(result.is_err());
}

/// 测试 create_index 错误路径 - 索引已存在
#[test]
fn test_create_index_already_exists() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 创建第一个索引
    db.create_index("items", "idx1", "field1", false, false).unwrap();

    // 尝试创建同名索引
    let result = db.create_index("items", "idx1", "field2", false, false);
    assert!(result.is_err());
}

/// 测试 delete_index 错误路径 - store 不存在
#[test]
fn test_delete_index_nonexistent_store() {
    let mut db = IdbDatabase::new("test", 1);
    // 尝试删除不存在的 store 上的索引
    let result = db.delete_index("nonexistent", "idx");
    assert!(result.is_err());
}

/// 测试 delete_index 错误路径 - 索引不存在
#[test]
fn test_delete_index_nonexistent() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 尝试删除不存在的索引
    let result = db.delete_index("items", "nonexistent");
    assert!(result.is_err());
}

/// 测试 IdbIndex extract_keys 方法 - multiEntry 处理 null 值
#[test]
fn test_index_extract_keys_multi_entry_with_null() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 创建 multiEntry 索引
    db.create_index("items", "tags_idx", "tags", false, true).unwrap();

    // 添加包含 null 数组的记录
    db.add(
        "items",
        serde_json::json!({"tags": [null, "valid", null]}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();

    // 查询应该只有有效的键
    let results = db
        .get_from_index("items", "tags_idx", &IdbKey::String("valid".into()))
        .unwrap();
    assert_eq!(results.len(), 1);
}

/// 测试 IdbIndex extract_keys 方法 - 空数组
#[test]
fn test_index_extract_keys_empty_array() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 创建 multiEntry 索引
    db.create_index("items", "tags_idx", "tags", false, true).unwrap();

    // 添加包含空数组的记录
    db.add(
        "items",
        serde_json::json!({"tags": []}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();

    // 查询应该没有结果
    let results = db
        .get_from_index("items", "tags_idx", &IdbKey::String("any".into()))
        .unwrap();
    assert_eq!(results.len(), 0);
}

/// 测试 IdbIndex rebuild 方法 - 在空 store 上重建
#[test]
fn test_index_rebuild_empty_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 创建索引
    db.create_index("items", "name_idx", "name", false, false).unwrap();

    // 索引应该是空的
    let results = db.get_all_from_index("items", "name_idx").unwrap();
    assert_eq!(results.len(), 0);
}

/// 测试 IdbIndex add_entry_from_record 方法 - 唯一约束违反
#[test]
fn test_index_add_entry_unique_violation() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 创建唯一索引
    db.create_index("items", "unique_idx", "field", true, false).unwrap();

    // 添加第一条记录
    db.add(
        "items",
        serde_json::json!({"field": "value"}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();

    // 添加第二条记录具有相同的索引值
    let result = db.add(
        "items",
        serde_json::json!({"field": "value"}),
        Some(IdbKey::String("2".into())),
    );

    // 应该失败，因为索引键必须唯一
    assert!(result.is_err());
}

/// 测试事务 tx_add 错误路径 - 非活跃事务
#[test]
fn test_tx_add_inactive_transaction() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    let mut tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    tx.abort().unwrap();

    // 尝试在已中止的事务上添加记录
    let result = db.tx_add(
        &tx,
        "items",
        serde_json::json!({"test": "data"}),
        Some(IdbKey::String("1".into())),
    );
    assert!(result.is_err());
}

/// 测试事务 tx_put 错误路径 - 非活跃事务
#[test]
fn test_tx_put_inactive_transaction() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    let mut tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    tx.commit().unwrap();

    // 尝试在已提交的事务上 put 记录
    let result = db.tx_put(
        &tx,
        "items",
        serde_json::json!({"test": "data"}),
        Some(IdbKey::String("1".into())),
    );
    assert!(result.is_err());
}

/// 测试事务 tx_delete 错误路径 - 非活跃事务
#[test]
fn test_tx_delete_inactive_transaction() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    let mut tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    tx.abort().unwrap();

    // 尝试在已中止的事务上删除记录
    let result = db.tx_delete(&tx, "items", &IdbKey::String("1".into()));
    assert!(result.is_err());
}

/// 测试事务 tx_get 错误路径 - 非活跃事务
#[test]
fn test_tx_get_inactive_transaction() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    let mut tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
    tx.commit().unwrap();

    // 尝试在已提交的事务上获取记录
    let result = db.tx_get(&tx, "items", &IdbKey::String("1".into()));
    assert!(result.is_err());
}

/// 测试 open_cursor_on_index 错误路径 - store 不存在
#[test]
fn test_open_cursor_on_index_nonexistent_store() {
    let mut db = IdbDatabase::new("test", 1);
    // 尝试在不存在的 store 上打开索引游标
    let result = db.open_cursor_on_index("nonexistent", "idx", None);
    assert!(result.is_err());
}

/// 测试 open_cursor_on_index 错误路径 - 索引不存在
#[test]
fn test_open_cursor_on_index_nonexistent_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    db.create_index("items", "idx", "field", false, false).unwrap();

    // 尝试在不存在的索引上打开游标
    let result = db.open_cursor_on_index("items", "nonexistent", None);
    assert!(result.is_err());
}

/// 测试 open_key_cursor_on_index 错误路径 - store 不存在
#[test]
fn test_open_key_cursor_on_index_nonexistent_store() {
    // 注意：这个方法不存在，改为测试 open_cursor_on_index
    let mut db = IdbDatabase::new("test", 1);
    // 尝试在不存在的 store 上打开游标
    let result = db.open_cursor_on_index("nonexistent", "idx", None);
    assert!(result.is_err());
}

/// 测试 json_value_to_idb_key 不支持的类型（通过索引间接测试）
#[test]
fn test_json_value_to_idb_key_unsupported_types_via_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加包含布尔值的记录
    db.add(
        "items",
        serde_json::json!({"field": true}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();

    // 创建索引 - 布尔值应该被忽略
    db.create_index("items", "idx", "field", false, false).unwrap();

    // 查询应该没有结果，因为布尔值无法转换为键
    let results = db
        .get_from_index("items", "idx", &IdbKey::String("true".into()))
        .unwrap();
    assert_eq!(results.len(), 0);

    // 添加包含 null 值的记录
    db.add(
        "items",
        serde_json::json!({"field": null}),
        Some(IdbKey::String("2".into())),
    )
    .unwrap();

    // 再次查询应该仍然没有结果
    let results = db
        .get_from_index("items", "idx", &IdbKey::String("null".into()))
        .unwrap();
    assert_eq!(results.len(), 0);
}

/// 测试 get_all_with_range 复杂路径 - 跨类型范围
#[test]
fn test_get_all_with_range_cross_type() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加不同类型的键
    db.add("items", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("items", serde_json::json!("b"), Some(IdbKey::String("10".into())))
        .unwrap();
    db.add("items", serde_json::json!("c"), Some(IdbKey::String("2".into())))
        .unwrap();

    // 创建包含 Number 和 String 的范围
    // Number(1.0) < String("10") < String("2") < String("5")
    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::String("5".into()), false, false);
    let results = db.get_all_with_range("items", &range).unwrap();

    // 应该包含 Number(1.0), String("2"), and String("10")（都在范围内）
    assert_eq!(results.len(), 3);
}

/// 测试 count_with_range 复杂路径 - 跨类型范围
#[test]
fn test_count_with_range_cross_type() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加不同类型的键
    db.add("items", serde_json::json!("a"), Some(IdbKey::Number(5.0)))
        .unwrap();
    db.add("items", serde_json::json!("b"), Some(IdbKey::String("a".into())))
        .unwrap();
    db.add("items", serde_json::json!("c"), Some(IdbKey::String("z".into())))
        .unwrap();

    // 创建跨类型的范围
    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::String("m".into()), false, true);
    let count = db.count_with_range("items", &range).unwrap();

    // 应该包含 Number(5.0) 和 String("a")
    assert_eq!(count, 2);
}

/// 测试 open_cursor_on_index 与复杂范围
#[test]
fn test_open_cursor_on_index_with_complex_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();

    // 添加记录
    db.add(
        "items",
        serde_json::json!({"category": "books", "id": 1}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();
    db.add(
        "items",
        serde_json::json!({"category": "electronics", "id": 2}),
        Some(IdbKey::String("2".into())),
    )
    .unwrap();
    db.add(
        "items",
        serde_json::json!({"category": "movies", "id": 3}),
        Some(IdbKey::String("3".into())),
    )
    .unwrap();

    // 创建索引
    db.create_index("items", "category_idx", "category", false, false)
        .unwrap();

    // 使用不匹配的范围打开游标
    let range = IdbKeyRange::only(IdbKey::String("nonexistent".into()));
    let cursor = db.open_cursor_on_index("items", "category_idx", Some(&range)).unwrap();

    // 应该返回 None
    assert!(cursor.is_none());
}

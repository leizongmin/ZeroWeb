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

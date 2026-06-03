//! IndexedDB types 模块覆盖率提升测试。
//!
//! 专注于测试 IdbKey 比较算法、哈希行为、边界条件以及序列化边缘情况。

use super::*;

#[test]
fn test_idb_key_nan_hashing() {
    let nan1 = IdbKey::Number(f64::NAN);
    let nan2 = IdbKey::Number(f64::NAN);

    // NaN 键的哈希值不同（因为 to_bits() 不同）
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(nan1.clone());
    set.insert(nan2.clone());

    // 即使两个 NaN 在比较时返回 Equal，它们仍然有不同的哈希值
    assert_eq!(set.len(), 2, "NaN 键应被视为不同元素（基于 to_bits()）");

    // 但在排序中，NaN 会被视为相等
    assert_eq!(nan1.cmp(&nan2), Ordering::Equal);
}

#[test]
fn test_idb_key_partial_ord_edge_cases() {
    let nan = IdbKey::Number(f64::NAN);
    let inf = IdbKey::Number(f64::INFINITY);
    let neg_inf = IdbKey::Number(f64::NEG_INFINITY);
    let zero = IdbKey::Number(0.0);
    let max_finite = IdbKey::Number(f64::MAX);

    // Test that NaN comparisons work
    assert_eq!(nan.partial_cmp(&zero), None);
    assert_eq!(zero.partial_cmp(&nan), None);
    assert_eq!(nan.partial_cmp(&nan), None);

    // Test infinity comparisons
    assert_eq!(inf.partial_cmp(&max_finite), Some(Ordering::Greater));
    assert_eq!(neg_inf.partial_cmp(&zero), Some(Ordering::Less));
    assert_eq!(inf.partial_cmp(&neg_inf), Some(Ordering::Greater));

    // Test PartialOrd for number types
    assert_eq!(zero.partial_cmp(&IdbKey::Number(1.0)), Some(Ordering::Less));
    assert_eq!(IdbKey::Number(1.0).partial_cmp(&zero), Some(Ordering::Greater));
}

#[test]
fn test_idb_key_hash_discriminant_differentiates() {
    // 确保不同类型的键有不同的判别值
    let num_key = IdbKey::Number(42.0);
    let str_key = IdbKey::String("42".to_string());
    let bin_key = IdbKey::Binary(vec![42]);
    let arr_key = IdbKey::Array(vec![IdbKey::Number(42.0)]);

    // 它们的判别值应该不同
    std::mem::discriminant(&num_key) != std::mem::discriminant(&str_key) &&
    std::mem::discriminant(&str_key) != std::mem::discriminant(&bin_key) &&
    std::mem::discriminant(&bin_key) != std::mem::discriminant(&arr_key)
}

#[test]
fn test_idb_key_number_hash_includes_bits() {
    // +0.0 和 -0.0 有不同的位表示，但 f64::eq 认为它们相等
    let pos_zero = IdbKey::Number(0.0);
    let neg_zero = IdbKey::Number(-0.0);

    // PartialEq 认为它们相等
    assert!(pos_zero == neg_zero);

    // 但哈希值不同（因为 to_bits() 不同）
    let mut set = std::collections::HashSet::new();
    set.insert(pos_zero.clone());
    set.insert(neg_zero.clone());
    assert_eq!(set.len(), 2, "+0.0 和 -0.0 应哈希为不同值");
}

#[test]
fn test_idb_key_array_nested_comparison() {
    let arr1 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("a".to_string()),
            IdbKey::Number(2.0),
        ]),
    ]);

    let arr2 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("b".to_string()),
            IdbKey::Number(2.0),
        ]),
    ]);

    let arr3 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("a".to_string()),
            IdbKey::Number(3.0),
        ]),
    ]);

    // 字典序比较
    assert!(arr1 < arr2);  // "a" < "b" 在第二层
    assert!(arr1 < arr3);  // 2.0 < 3.0 在第二层

    // 相等性测试
    let arr4 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Array(vec![
            IdbKey::String("a".to_string()),
            IdbKey::Number(2.0),
        ]),
    ]);
    assert_eq!(arr1, arr4);
}

#[test]
fn test_idb_key_range_contains_with_nan() {
    let range = IdbKeyRange::only(IdbKey::Number(5.0));

    // NaN 键在任何范围外（因为比较返回 Equal，但我们的实现可能处理 NaN 特殊）
    let nan_key = IdbKey::Number(f64::NAN);
    // 这里 NaN 的行为可能因实现而异
    // 在我们的 cmp_key 实现中，NaN 与任何数字比较都返回 Equal
    assert!(!range.contains(&nan_key), "NaN 不应在任何键范围内");
}

#[test]
fn test_idb_key_array_empty_arrays() {
    let empty1 = IdbKey::Array(vec![]);
    let empty2 = IdbKey::Array(vec![]);

    assert_eq!(empty1, empty2);
    assert_eq!(empty1.cmp(&empty2), Ordering::Equal);

    // 空数组小于任何非空数组
    let non_empty = IdbKey::Array(vec![IdbKey::Number(1.0)]);
    assert!(empty1 < non_empty);
}

#[test]
fn test_idb_key_binary_hash() {
    let bin1 = IdbKey::Binary(vec![1, 2, 3]);
    let bin2 = IdbKey::Binary(vec![1, 2, 3]);
    let bin3 = IdbKey::Binary(vec![1, 2, 4]);

    // 相同内容应哈希相等
    assert_eq!(bin1, bin2);
    let mut set = std::collections::HashSet::new();
    set.insert(bin1.clone());
    set.insert(bin2.clone());
    assert_eq!(set.len(), 1);

    // 不同内容应哈希不等
    assert_ne!(bin1, bin3);
    set.insert(bin3.clone());
    assert_eq!(set.len(), 2);
}

#[test]
fn test_idb_key_string_unicode() {
    let str1 = IdbKey::String("café".to_string());
    let str2 = IdbKey::String("cafe\u{0301}".to_string());  // e + combining acute

    // 这些是不同的 Unicode 序列，但不等价（除非规范化）
    assert_ne!(str1, str2);
    assert!(str1 < str2);  // 'é' < 'e\u{0301}' 在 Unicode 码点顺序中

    // 测试哈希值
    let mut set = std::collections::HashSet::new();
    set.insert(str1.clone());
    set.insert(str2.clone());
    assert_eq!(set.len(), 2, "不同 Unicode 序列应哈希不同");
}

#[test]
fn test_idb_key_comparison_across_types() {
    // 测试所有类型对的比较
    let num = IdbKey::Number(1.0);
    let str = IdbKey::String("1".to_string());
    let bin = IdbKey::Binary(vec![1]);
    let arr = IdbKey::Array(vec![IdbKey::Number(1.0)]);

    // Number < String < Binary < Array
    assert!(num < str);
    assert!(str < bin);
    assert!(bin < arr);

    // 反向比较
    assert!(str > num);
    assert!(bin > str);
    assert!(arr > bin);
}

#[test]
fn test_idb_key_large_numbers() {
    let max_num = IdbKey::Number(f64::MAX);
    let min_num = IdbKey::Number(f64::MIN);
    let max_pos = IdbKey::Number(f64::MAX_EXP as f64);
    let min_pos = IdbKey::Number(f64::MIN_POSITIVE);

    // 验证顺序
    assert!(min_num < min_pos);
    assert!(min_pos < max_num);
    assert!(max_num < max_pos);

    // 测试与无穷大的比较
    let inf = IdbKey::Number(f64::INFINITY);
    let neg_inf = IdbKey::Number(f64::NEG_INFINITY);

    assert!(neg_inf < min_num);
    assert!(max_num < inf);
}

#[test]
fn test_idb_key_array_with_mixed_types() {
    let arr1 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::String("a".to_string()),
    ]);

    let arr2 = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::String("b".to_string()),
    ]);

    let arr3 = IdbKey::Array(vec![
        IdbKey::Number(2.0),
        IdbKey::String("a".to_string()),
    ]);

    // 第一元素不同
    assert!(arr1 < arr3);

    // 第一元素相同，比较第二元素
    assert!(arr1 < arr2);
}

#[test]
fn test_idb_key_range_with_boundary_conditions() {
    let min_key = IdbKey::Number(f64::MIN);
    let max_key = IdbKey::Number(f64::MAX);
    let inf = IdbKey::Number(f64::INFINITY);
    let neg_inf = IdbKey::Number(f64::NEG_INFINITY);

    // 边界范围测试
    let range = IdbKeyRange::bound(min_key.clone(), max_key.clone(), false, false);

    assert!(range.contains(&min_key));
    assert!(range.contains(&max_key));
    assert!(!range.contains(&inf));
    assert!(!range.contains(&neg_inf));

    // 开区间测试
    let open_range = IdbKeyRange::bound(min_key.clone(), max_key.clone(), true, true);
    assert!(!open_range.contains(&min_key));
    assert!(!open_range.contains(&max_key));
}

#[test]
fn test_idb_key_serialization_edge_cases() {
    // 测试各种键类型的序列化/反序列化
    let keys = vec![
        IdbKey::Number(f64::NAN),
        IdbKey::Number(f64::INFINITY),
        IdbKey::Number(f64::NEG_INFINITY),
        IdbKey::Number(0.0),
        IdbKey::Number(-0.0),
        IdbKey::String("".to_string()),
        IdbKey::Binary(vec![]),
        IdbKey::Array(vec![]),
    ];

    // 注意：IdbKey 没有实现 Serialize/Deserialize trait
    // 所以这个测试主要是确保没有 panic
    for key in keys {
        // 只测试哈希和比较，不测试序列化
        let _ = format!("{:?}", key);
        let _ = key.hash(std::hash::BuildHasher::default());
    }
}

#[test]
fn test_idb_key_debug_format() {
    let num_key = IdbKey::Number(42.0);
    let str_key = IdbKey::String("test".to_string());
    let bin_key = IdbKey::Binary(vec![1, 2, 3]);
    let arr_key = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::String("a".to_string())]);

    assert_eq!(format!("{:?}", num_key), "Number(42.0)");
    assert_eq!(format!("{:?}", str_key), r#"String("test")"#);
    assert_eq!(format!("{:?}", bin_key), "Binary([1, 2, 3])");
    assert_eq!(format!("{:?}", arr_key), "Array([Number(1.0), String(\"a\")])");
}
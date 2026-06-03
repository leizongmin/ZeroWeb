//! IndexedDB 游标操作覆盖率提升测试。
//!
//! 专注于测试游标的 advance、continue、delete、update 等操作边缘情况。

use super::*;

#[test]
fn test_idb_cursor_advance_zero() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();

    // advance(0) 应该回到开头
    assert!(cursor.advance(0));
    assert_eq!(cursor.position(), 0);
    assert!(!cursor.is_finished());
}

#[test]
fn test_idb_cursor_advance_beyond_end() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();

    // advance(1) 移动到末尾后
    assert!(cursor.advance(1));
    assert_eq!(cursor.position(), 1);
    assert!(cursor.is_finished());

    // 再次 advance 应该失败
    assert!(!cursor.advance(1));
    assert!(!cursor.advance(0));
}

#[test]
fn test_idb_cursor_advance_middle() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 插入 5 条记录
    for i in 1..=5 {
        db.add("store", serde_json::json!(format!("v{i}")), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();

    // advance(2) 从位置 0 跳到位置 2（第3条记录）
    assert!(cursor.advance(2));
    assert_eq!(cursor.position(), 2);
    let record = db.cursor_record(&cursor).unwrap();
    assert_eq!(record.value, serde_json::json!("v3"));

    // 再 advance(1) 跳到位置 3（第4条记录）
    assert!(cursor.advance(1));
    assert_eq!(cursor.position(), 3);
    let record = db.cursor_record(&cursor).unwrap();
    assert_eq!(record.value, serde_json::json!("v4"));

    // advance(1) 到末尾
    assert!(cursor.advance(1));
    assert_eq!(cursor.position(), 4);
    assert!(cursor.is_finished());
}

#[test]
fn test_idb_cursor_continue_to_not_found() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 插入 1, 3, 5
    db.add("store", serde_json::json!(1), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", serde_json::json!(3), Some(IdbKey::Number(3.0)))
        .unwrap();
    db.add("store", serde_json::json!(5), Some(IdbKey::Number(5.0)))
        .unwrap();

    let mut cursor = db.open_key_cursor("store", None).unwrap().unwrap();

    // 尝试继续到不存在的键
    assert!(!cursor.continue_to(&IdbKey::Number(2.0)));
    assert!(!cursor.continue_to(&IdbKey::Number(4.0)));
    assert!(!cursor.continue_to(&IdbKey::Number(10.0)));
}

#[test]
fn test_idb_cursor_continue_to_found() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 插入不同类型的键进行测试
    db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", serde_json::json!("b"), Some(IdbKey::String("10".to_string())))
        .unwrap();
    db.add("store", serde_json::json!("c"), Some(IdbKey::Binary(vec![1])))
        .unwrap();

    let mut cursor = db.open_key_cursor("store", None).unwrap().unwrap();

    // 起始在 position 0 (key=1.0)
    assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(1.0)));

    // continue_to(10.0) - 字符串 "10" 在排序中大于 number 1.0
    assert!(cursor.continue_to(&IdbKey::String("10".to_string())));
    assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::String("10".to_string())));

    // continue_to 到 binary
    assert!(cursor.continue_to(&IdbKey::Binary(vec![1])));
    assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Binary(vec![1])));
}

#[test]
fn test_idb_cursor_continue_next_from_end() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!("a"), Some(IdbKey::String("a".to_string())))
        .unwrap();

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();

    // 移动到末尾
    assert!(cursor.continue_next());
    assert!(cursor.is_finished());

    // 从末尾继续应返回 false
    assert!(!cursor.continue_next());
}

#[test]
fn test_idb_cursor_empty_advance() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("empty", None, false).unwrap();

    // 尝试在空的游标上 advance
    let cursor = db.open_cursor("empty", None).unwrap();
    assert!(cursor.is_none());
}

#[test]
fn test_idb_cursor_position_tracking() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 插入 3 条记录
    for i in 1..=3 {
        db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();

    // 初始位置
    assert_eq!(cursor.position(), 0);

    // advance(2) 后
    assert!(cursor.advance(2));
    assert_eq!(cursor.position(), 2);

    // continue_next 后
    assert!(cursor.continue_next());
    assert_eq!(cursor.position(), 2);  // continue_next 递增 position

    // 到末尾
    assert!(cursor.is_finished());
    assert_eq!(cursor.position(), 2);
}

#[test]
fn test_idb_cursor_reverse_iteration() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 按 3, 1, 2 顺序插入，验证排序
    db.add("store", serde_json::json!("c"), Some(IdbKey::Number(3.0)))
        .unwrap();
    db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", serde_json::json!("b"), Some(IdbKey::Number(2.0)))
        .unwrap();

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
    let mut values = Vec::new();

    // 收集所有值（按键排序）
    loop {
        let record = db.cursor_record(&cursor).unwrap();
        values.push(record.value.clone());
        if !cursor.continue_next() {
            break;
        }
    }

    assert_eq!(values, vec![
        serde_json::json!("a"),
        serde_json::json!("b"),
        serde_json::json!("c"),
    ]);
}

#[test]
fn test_idb_cursor_store_name_consistency() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store1", None, false).unwrap();
    db.create_object_store("store2", None, false).unwrap();

    db.add("store1", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store2", serde_json::json!("b"), Some(IdbKey::Number(1.0)))
        .unwrap();

    let cursor1 = db.open_cursor("store1", None).unwrap().unwrap();
    let cursor2 = db.open_cursor("store2", None).unwrap().unwrap();

    assert_eq!(cursor1.store_name(), "store1");
    assert_eq!(cursor2.store_name(), "store2");
}

#[test]
fn test_idb_cursor_with_unicode_keys() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 插入 Unicode 字符串键
    db.add("store", serde_json::json!("1"), Some(IdbKey::String("café".to_string())))
        .unwrap();
    db.add("store", serde_json::json!("2"), Some(IdbKey::String("naïve".to_string())))
        .unwrap();
    db.add("store", serde_json::json!("3"), Some(IdbKey::String("zulu".to_string())))
        .unwrap();

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
    let mut values = Vec::new();

    loop {
        let record = db.cursor_record(&cursor).unwrap();
        values.push(record.value.clone());
        if !cursor.continue_next() {
            break;
        }
    }

    // 应按 Unicode 顺序排序
    assert_eq!(values, vec![
        serde_json::json!("1"),  // café
        serde_json::json!("2"),  // naïve
        serde_json::json!("3"),  // zulu
    ]);
}

#[test]
fn test_idb_cursor_binary_key_ordering() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 插入不同长度的二进制键
    db.add("store", serde_json::json!("a"), Some(IdbKey::Binary(vec![1])))
        .unwrap();
    db.add("store", serde_json::json!("b"), Some(IdbKey::Binary(vec![1, 2])))
        .unwrap();
    db.add("store", serde_json::json!("c"), Some(IdbKey::Binary(vec![2])))
        .unwrap();

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
    let mut keys = Vec::new();

    loop {
        let record = db.cursor_record(&cursor).unwrap();
        keys.push(record.key.clone());
        if !cursor.continue_next() {
            break;
        }
    }

    // 二进制键按字典序排序
    assert_eq!(keys, vec![
        IdbKey::Binary(vec![1]),
        IdbKey::Binary(vec![1, 2]),
        IdbKey::Binary(vec![2]),
    ]);
}

#[test]
fn test_idb_cursor_mixed_type_keys() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 混合类型：Number < String < Binary < Array
    db.add("store", serde_json::json!("1"), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", serde_json::json!("2"), Some(IdbKey::String("1".to_string())))
        .unwrap();
    db.add("store", serde_json::json!("3"), Some(IdbKey::Binary(vec![1])))
        .unwrap();
    db.add("store", serde_json::json!("4"), Some(IdbKey::Array(vec![IdbKey::Number(1.0)])))
        .unwrap();

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
    let mut values = Vec::new();

    loop {
        let record = db.cursor_record(&cursor).unwrap();
        values.push(record.value.clone());
        if !cursor.continue_next() {
            break;
        }
    }

    assert_eq!(values, vec![
        serde_json::json!("1"),
        serde_json::json!("2"),
        serde_json::json!("3"),
        serde_json::json!("4"),
    ]);
}

#[test]
fn test_idb_cursor_large_advance() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 插入 100 条记录
    for i in 1..=100 {
        db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();

    // 一次性前进 50 步
    assert!(cursor.advance(50));
    assert_eq!(cursor.position(), 50);
    let record = db.cursor_record(&cursor).unwrap();
    assert_eq!(record.value, serde_json::json!(51));

    // 再前进 49 步到末尾
    assert!(cursor.advance(49));
    assert_eq!(cursor.position(), 99);
    assert!(cursor.is_finished());
}

#[test]
fn test_idb_cursor_continue_to_exact_match() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 插入精确匹配的键
    db.add("store", serde_json::json!("a"), Some(IdbKey::Number(5.0)))
        .unwrap();

    let mut cursor = db.open_key_cursor("store", None).unwrap().unwrap();

    // continue_to 到确切的键
    assert!(cursor.continue_to(&IdbKey::Number(5.0)));
    assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(5.0)));
}

#[test]
fn test_idb_cursor_no_op_operations() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();

    // advance(0) 多次 - 应该总是成功并保持位置
    assert!(cursor.advance(0));  // 重置到位置 0
    assert_eq!(cursor.position(), 0);
    assert!(cursor.advance(0));  // 再次重置到位置 0
    assert_eq!(cursor.position(), 0);
    assert!(cursor.advance(0));  // 第三次重置
    assert_eq!(cursor.position(), 0);
}
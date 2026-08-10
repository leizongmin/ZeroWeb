// Auto-generated test file — split from indexed_db.rs
use std::cmp::Ordering;

use super::super::*;

#[test]
fn test_delete_nonexistent_record() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let deleted = db.delete("store", &IdbKey::String("nope".into())).unwrap();
    assert!(!deleted);
}

#[test]
fn test_idb_key_array_ordering() {
    let a = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Number(2.0)]);
    let b = IdbKey::Array(vec![IdbKey::Number(1.0), IdbKey::Number(3.0)]);
    let c = IdbKey::Array(vec![IdbKey::Number(2.0)]);
    assert!(a < b);
    assert!(b < c);
}

// ── 事务缓冲与中止测试 ──

/// tx_add 后 abort，数据不应存在于 store 中。
#[test]
fn test_tx_add_then_abort_data_not_in_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(
        &tx,
        "store",
        serde_json::json!({"name": "Alice"}),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();
    tx.abort().unwrap();
    // 中止后数据不应在 store 中
    assert!(db.get("store", &IdbKey::String("k1".into())).is_none());
    assert_eq!(db.count("store").unwrap(), 0);
}

/// tx_put 后 abort，原始数据应保留。
#[test]
fn test_tx_put_then_abort_original_preserved() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let key = IdbKey::String("k1".into());
    db.add("store", serde_json::json!({"name": "Alice"}), Some(key.clone()))
        .unwrap();

    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_put(&tx, "store", serde_json::json!({"name": "Bob"}), Some(key.clone()))
        .unwrap();
    tx.abort().unwrap();

    // 原始数据应保留
    let record = db.get("store", &key).unwrap();
    assert_eq!(record.value["name"], "Alice");
}

/// tx_delete 后 abort，被删除的数据应保留。
#[test]
fn test_tx_delete_then_abort_data_preserved() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let key = IdbKey::String("k1".into());
    db.add("store", serde_json::json!("original"), Some(key.clone()))
        .unwrap();

    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_delete(&tx, "store", &key).unwrap();
    tx.abort().unwrap();

    // 数据应保留
    let record = db.get("store", &key).unwrap();
    assert_eq!(record.value, serde_json::json!("original"));
}

/// tx_add 后 commit_tx，数据应存在于 store 中。
#[test]
fn test_tx_add_then_commit_data_in_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(
        &tx,
        "store",
        serde_json::json!({"name": "Alice"}),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();
    db.commit_tx(&mut tx).unwrap();

    let record = db.get("store", &IdbKey::String("k1".into())).unwrap();
    assert_eq!(record.value["name"], "Alice");
    assert_eq!(db.count("store").unwrap(), 1);
}

/// tx_put 后 commit_tx，数据应被更新。
#[test]
fn test_tx_put_then_commit_data_updated() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let key = IdbKey::String("k1".into());
    db.add("store", serde_json::json!("original"), Some(key.clone()))
        .unwrap();

    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_put(&tx, "store", serde_json::json!("updated"), Some(key.clone()))
        .unwrap();
    db.commit_tx(&mut tx).unwrap();

    let record = db.get("store", &key).unwrap();
    assert_eq!(record.value, serde_json::json!("updated"));
    assert_eq!(db.count("store").unwrap(), 1);
}

/// tx_delete 后 commit_tx，数据应被删除。
#[test]
fn test_tx_delete_then_commit_data_removed() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let key = IdbKey::String("k1".into());
    db.add("store", serde_json::json!("val"), Some(key.clone())).unwrap();

    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_delete(&tx, "store", &key).unwrap();
    db.commit_tx(&mut tx).unwrap();

    assert!(db.get("store", &key).is_none());
}

/// 事务内 tx_get 应能看到缓冲区的未提交变更。
#[test]
fn test_tx_get_sees_buffered_add() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(
        &tx,
        "store",
        serde_json::json!("buffered"),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();

    let rec = db.tx_get(&tx, "store", &IdbKey::String("k1".into())).unwrap();
    assert_eq!(rec.unwrap().value, serde_json::json!("buffered"));
    // 尚未提交，store 中不应有数据
    assert!(db.get("store", &IdbKey::String("k1".into())).is_none());
}

/// 事务内 tx_get 对被缓冲删除的键返回 None。
#[test]
fn test_tx_get_sees_buffered_delete() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let key = IdbKey::String("k1".into());
    db.add("store", serde_json::json!("original"), Some(key.clone()))
        .unwrap();

    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_delete(&tx, "store", &key).unwrap();
    assert!(db.tx_get(&tx, "store", &key).unwrap().is_none());
}

/// 事务内 tx_get 对缓冲 put 返回更新后的值。
#[test]
fn test_tx_get_sees_buffered_put() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let key = IdbKey::String("k1".into());
    db.add("store", serde_json::json!("old"), Some(key.clone())).unwrap();

    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_put(&tx, "store", serde_json::json!("new"), Some(key.clone()))
        .unwrap();

    let rec = db.tx_get(&tx, "store", &key).unwrap().unwrap();
    assert_eq!(rec.value, serde_json::json!("new"));
}

// ── IdbKey 边界值排序测试 ──

/// 测试 NaN 键的排序行为。
///
/// 当前实现使用 partial_cmp().unwrap_or(Ordering::Equal)，
/// 导致 NaN 被视为与任意数值相等（包括自身），这是不符合
/// IndexedDB 规范的已知行为。本测试记录当前行为。
#[test]
fn test_idb_key_nan_ordering() {
    let nan_key = IdbKey::Number(f64::NAN);
    let one_key = IdbKey::Number(1.0);
    let inf_key = IdbKey::Number(f64::INFINITY);
    let neg_inf_key = IdbKey::Number(f64::NEG_INFINITY);

    // NaN 与自身比较：当前实现返回 Equal（因为 partial_cmp 返回 None）
    assert_eq!(nan_key.cmp(&nan_key), Ordering::Equal);

    // NaN 与普通数值比较：当前实现返回 Equal（不符合规范）
    // 按 IndexedDB 规范，NaN 不应是有效 key，但当前实现允许。
    // 此处断言记录当前行为：NaN 被视为与所有数值相等。
    assert_eq!(nan_key.cmp(&one_key), Ordering::Equal);
    assert_eq!(nan_key.cmp(&inf_key), Ordering::Equal);
    assert_eq!(nan_key.cmp(&neg_inf_key), Ordering::Equal);

    // 反向比较同样返回 Equal
    assert_eq!(one_key.cmp(&nan_key), Ordering::Equal);

    // NaN 与非 Number 类型比较：仍应保持 Number < String 的跨类型规则
    let str_key = IdbKey::String("a".to_string());
    assert_eq!(nan_key.cmp(&str_key), Ordering::Less);
}

/// 测试 +Infinity 和 -Infinity 键的排序行为。
///
/// +Inf 应大于所有有限数值，-Inf 应小于所有有限数值。
#[test]
fn test_idb_key_infinity_ordering() {
    let inf = IdbKey::Number(f64::INFINITY);
    let neg_inf = IdbKey::Number(f64::NEG_INFINITY);
    let max_finite = IdbKey::Number(f64::MAX);
    let min_finite = IdbKey::Number(f64::MIN_POSITIVE);
    let zero = IdbKey::Number(0.0);

    // +Inf 大于所有有限数
    assert_eq!(inf.cmp(&max_finite), Ordering::Greater);
    assert_eq!(max_finite.cmp(&inf), Ordering::Less);

    // -Inf 小于所有有限数（包括负数）
    assert_eq!(neg_inf.cmp(&zero), Ordering::Less);
    assert_eq!(neg_inf.cmp(&IdbKey::Number(-f64::MAX)), Ordering::Less);

    // +Inf 大于 -Inf
    assert_eq!(inf.cmp(&neg_inf), Ordering::Greater);
    assert_eq!(neg_inf.cmp(&inf), Ordering::Less);

    // +Inf 自身相等
    assert_eq!(inf.cmp(&inf), Ordering::Equal);
    assert_eq!(neg_inf.cmp(&neg_inf), Ordering::Equal);

    // 在排序中的位置：-Inf < 0 < min_positive < MAX < +Inf
    let mut keys = vec![
        inf.clone(),
        max_finite.clone(),
        zero.clone(),
        neg_inf.clone(),
        min_finite.clone(),
    ];
    keys.sort();
    assert_eq!(keys[0], neg_inf);
    assert_eq!(keys[1], zero);
    assert_eq!(keys[2], min_finite);
    assert_eq!(keys[3], max_finite);
    assert_eq!(keys[4], inf);
}

/// 测试 -0.0 与 +0.0 键的比较行为。
///
/// 按 IEEE 754，-0.0 == +0.0。IdbKey 派生 PartialEq（f64 ==）故两者相等；
/// Hash 已归一化 -0.0 → +0.0（与 Eq 契约一致：a==b ⇒ hash(a)==hash(b)，
/// 且符合 JS Set/Map「-0 与 +0 为同一键」语义），故 HashSet 中为同一键。
/// Ord 比较返回 Equal。
#[test]
fn test_idb_key_zero_ordering() {
    let pos_zero = IdbKey::Number(0.0);
    let neg_zero = IdbKey::Number(-0.0);

    // f64 的 == 认为 -0.0 == +0.0，所以 PartialEq 也不等
    // 但 IdbKey 派生 PartialEq，Number(0.0) == Number(-0.0)
    // 因为 f64 的 0.0 == -0.0 为 true
    assert!(pos_zero == neg_zero, "+0.0 should equal -0.0 via PartialEq");

    // Ord 排序：应为 Equal（因为底层 f64 的 partial_cmp 返回 Equal）
    assert_eq!(pos_zero.cmp(&neg_zero), Ordering::Equal);

    // Hash 行为：Hash 已归一化 -0.0 → +0.0（to_bits 相同），
    // 与 PartialEq 一致（a==b ⇒ hash(a)==hash(b)），故 HashSet 视为同一键。
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(pos_zero.clone());
    set.insert(neg_zero.clone());
    // 归一化后两者 hash 相同 + Eq 相等 → 第二次插入去重，set.len() == 1（确定，非 flaky）。
    // 旧实现 to_bits 不同致与 Eq 契约冲突，len() 随 SipHash RandomState 同桶与否而变（flaky）。
    assert_eq!(set.len(), 1, "-0.0 and +0.0 应为同一键（Hash 归一化 + PartialEq 相等）");

    // 在 Vec 排序中，-0.0 和 +0.0 位置不确定（因为 Equal），
    // 但排序后它们应该相邻
    let mut keys = vec![
        IdbKey::Number(1.0),
        IdbKey::Number(-0.0),
        IdbKey::Number(0.0),
        IdbKey::Number(-1.0),
    ];
    keys.sort();
    // -1.0, (-0.0, +0.0 顺序不确定), 1.0
    assert_eq!(keys[0], IdbKey::Number(-1.0));
    // keys[1] 和 keys[2] 都是某种零，无法确定顺序
    assert!(keys[1] == IdbKey::Number(0.0) || keys[1] == IdbKey::Number(-0.0));
    assert!(keys[2] == IdbKey::Number(0.0) || keys[2] == IdbKey::Number(-0.0));
    assert_eq!(keys[3], IdbKey::Number(1.0));
}

/// 测试唯一索引在 put 覆盖路径上的约束违反检测。
///
/// 场景：创建唯一索引，添加记录 A（索引值 X），添加记录 B（索引值 Y），
/// 然后 put(A, 新值) 将 A 的索引值改为 Y——此时应触发唯一约束违反。
///
/// 已知问题：当前 put() 在检测约束之前已修改了 record.value，
/// 即使返回错误，记录值已被覆盖。本测试记录此行为。
#[test]
fn test_unique_index_put_violation() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    // 添加记录 A，索引值为 "email_a@test.com"
    db.add(
        "store",
        serde_json::json!({"email": "email_a@test.com"}),
        Some(IdbKey::String("A".into())),
    )
    .unwrap();

    // 添加记录 B，索引值为 "email_b@test.com"
    db.add(
        "store",
        serde_json::json!({"email": "email_b@test.com"}),
        Some(IdbKey::String("B".into())),
    )
    .unwrap();

    // 创建唯一索引
    db.create_index("store", "email_idx", "email", true, false).unwrap();

    // 尝试 put 记录 A，将其 email 改为 "email_b@test.com"（与记录 B 冲突）
    let result = db.put(
        "store",
        serde_json::json!({"email": "email_b@test.com"}),
        Some(IdbKey::String("A".into())),
    );

    // put 应检测到唯一约束冲突并返回错误
    assert!(
        result.is_err(),
        "put() changing indexed value to conflict with another record should fail unique constraint"
    );

    // R3228 已修复：put 违 unique 时 record.value 不再被提前修改（预校验后才 mutate，
    // 违例回滚原值）。旧实现先 mutate value 再检查索引，违例时值已损坏。
    let record_a = db.get("store", &IdbKey::String("A".into())).unwrap();
    assert_eq!(
        record_a.value["email"], "email_a@test.com",
        "R3228: put 违 unique 须保持原值 email_a（不再提前修改为 email_b）"
    );

    // 记录数量不变
    assert_eq!(db.count("store").unwrap(), 2);
}

// ── 新增边界测试：游标 advance / continue / 迭代 ──

/// 打开值游标，advance(N) 跳过 N 条记录，验证游标停在正确位置。
#[test]
fn test_idb_cursor_advance() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    for i in 1..=6 {
        db.add(
            "store",
            serde_json::json!(format!("v{i}")),
            Some(IdbKey::Number(i as f64)),
        )
        .unwrap();
    }

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
    // 初始位置：第一条记录（key=1）
    assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!("v1"));

    // advance(3)：跳 3 步，落到第 4 条（key=4，value="v4"）
    assert!(cursor.advance(3));
    assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!("v4"));
    assert_eq!(cursor.position(), 3);

    // advance(2)：跳 2 步，落到第 6 条（key=6，value="v6"）
    assert!(cursor.advance(2));
    assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!("v6"));

    // advance(1)：超出范围
    assert!(!cursor.advance(1));
    assert!(cursor.is_finished());
}

/// 打开键游标，continue_to 跳到指定键。
#[test]
fn test_idb_cursor_continue_to() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    for i in 1..=5 {
        db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }

    let mut cursor = db.open_key_cursor("store", None).unwrap().unwrap();
    // 初始在 key=1
    assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(1.0)));

    // continue_to(3.0) → 跳到 key=3
    assert!(cursor.continue_to(&IdbKey::Number(3.0)));
    assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(3.0)));

    // continue_to(10.0) → 超出范围
    assert!(!cursor.continue_to(&IdbKey::Number(10.0)));
}

/// 打开值游标并逐条迭代全部记录。
#[test]
fn test_idb_cursor_iteration() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    // 按 3-1-2 的顺序插入，验证迭代时按键排序
    db.add("store", serde_json::json!("c"), Some(IdbKey::Number(3.0)))
        .unwrap();
    db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", serde_json::json!("b"), Some(IdbKey::Number(2.0)))
        .unwrap();

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
    let mut values = Vec::new();
    loop {
        let rec = db.cursor_record(&cursor).unwrap();
        values.push(rec.value.clone());
        if !cursor.continue_next() {
            break;
        }
    }
    assert_eq!(
        values,
        vec![serde_json::json!("a"), serde_json::json!("b"), serde_json::json!("c"),]
    );
    assert!(cursor.is_finished());
}

/// 打开键游标，advance(N) 跳过 N 条记录，验证键序列。
#[test]
fn test_idb_key_cursor_advance() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    for i in [10, 20, 30, 40, 50] {
        db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }

    let mut cursor = db.open_key_cursor("store", None).unwrap().unwrap();
    assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(10.0)));

    // advance(2) → 跳到 30
    assert!(cursor.advance(2));
    assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(30.0)));

    // advance(1) → 跳到 40
    assert!(cursor.advance(1));
    assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(40.0)));

    // advance(1) → 跳到 50
    assert!(cursor.advance(1));
    assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(50.0)));

    // advance(1) → 超出范围
    assert!(!cursor.advance(1));
    assert!(cursor.is_finished());
}

/// 打开键游标，continue_next 逐步前进。
#[test]
fn test_idb_key_cursor_continue_next() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!("x"), Some(IdbKey::String("a".into())))
        .unwrap();
    db.add("store", serde_json::json!("y"), Some(IdbKey::String("b".into())))
        .unwrap();
    db.add("store", serde_json::json!("z"), Some(IdbKey::String("c".into())))
        .unwrap();

    let mut cursor = db.open_key_cursor("store", None).unwrap().unwrap();
    let mut keys = Vec::new();
    loop {
        keys.push(db.cursor_key(&cursor).cloned());
        if !cursor.advance(1) {
            break;
        }
    }
    assert_eq!(
        keys,
        vec![
            Some(IdbKey::String("a".into())),
            Some(IdbKey::String("b".into())),
            Some(IdbKey::String("c".into())),
        ]
    );
    assert!(cursor.is_finished());
}

/// 创建事务，添加多条记录，commit_tx，验证记录持久化。
#[test]
fn test_idb_transaction_commit() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();

    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(
        &tx,
        "store",
        serde_json::json!({"name": "Alice"}),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();
    db.tx_add(
        &tx,
        "store",
        serde_json::json!({"name": "Bob"}),
        Some(IdbKey::String("k2".into())),
    )
    .unwrap();
    db.tx_add(
        &tx,
        "store",
        serde_json::json!({"name": "Charlie"}),
        Some(IdbKey::String("k3".into())),
    )
    .unwrap();

    // 提交前，store 中没有数据
    assert_eq!(db.count("store").unwrap(), 0);

    db.commit_tx(&mut tx).unwrap();
    assert!(tx.is_committed());

    // 提交后，3 条记录全部持久化
    assert_eq!(db.count("store").unwrap(), 3);
    assert_eq!(
        db.get("store", &IdbKey::String("k1".into())).unwrap().value["name"],
        "Alice"
    );
    assert_eq!(
        db.get("store", &IdbKey::String("k2".into())).unwrap().value["name"],
        "Bob"
    );
    assert_eq!(
        db.get("store", &IdbKey::String("k3".into())).unwrap().value["name"],
        "Charlie"
    );
}

/// 创建事务，添加多条记录，abort，验证记录未持久化。
#[test]
fn test_idb_transaction_abort() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    // 预存一条数据
    db.add(
        "store",
        serde_json::json!("original"),
        Some(IdbKey::String("k0".into())),
    )
    .unwrap();

    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    db.tx_add(
        &tx,
        "store",
        serde_json::json!("new1"),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();
    db.tx_put(
        &tx,
        "store",
        serde_json::json!("modified"),
        Some(IdbKey::String("k0".into())),
    )
    .unwrap();

    // abort 丢弃所有缓冲变更
    tx.abort().unwrap();
    assert!(tx.is_aborted());

    // k0 保持原始值，k1 不存在
    assert_eq!(
        db.get("store", &IdbKey::String("k0".into())).unwrap().value,
        serde_json::json!("original")
    );
    assert!(db.get("store", &IdbKey::String("k1".into())).is_none());
    assert_eq!(db.count("store").unwrap(), 1);
}

/// put() 覆盖已有记录，值更新且记录数不变。
#[test]
fn test_idb_put_overwrites_existing() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let key = IdbKey::Number(42.0);
    db.add(
        "store",
        serde_json::json!({"version": 1, "data": "old"}),
        Some(key.clone()),
    )
    .unwrap();
    assert_eq!(db.count("store").unwrap(), 1);

    // put 覆盖同一 key
    let returned = db
        .put(
            "store",
            serde_json::json!({"version": 2, "data": "new"}),
            Some(key.clone()),
        )
        .unwrap();
    assert_eq!(returned, key);

    let record = db.get("store", &key).unwrap();
    assert_eq!(record.value["version"], 2);
    assert_eq!(record.value["data"], "new");
    // 记录数不变
    assert_eq!(db.count("store").unwrap(), 1);
}

/// add() 在主键已存在时应拒绝。
#[test]
fn test_idb_add_rejects_duplicate() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let key = IdbKey::String("dup".into());
    db.add("store", serde_json::json!("first"), Some(key.clone())).unwrap();

    // 再次 add 同一 key 应报错
    let result = db.add("store", serde_json::json!("second"), Some(key.clone()));
    assert!(result.is_err());

    // 原始记录未被覆盖
    let record = db.get("store", &key).unwrap();
    assert_eq!(record.value, serde_json::json!("first"));
}

/// count_with_range 对不同范围返回正确计数。
#[test]
fn test_idb_count_with_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    for i in 1..=10 {
        db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }

    // 全范围 [1, 10]
    let full = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), false, false);
    assert_eq!(db.count_with_range("store", &full).unwrap(), 10);

    // 子范围 [3, 7]
    let mid = IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(7.0), false, false);
    assert_eq!(db.count_with_range("store", &mid).unwrap(), 5);

    // 开区间 (3, 7)
    let open = IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(7.0), true, true);
    assert_eq!(db.count_with_range("store", &open).unwrap(), 3);

    // lower_bound >= 8
    let lower = IdbKeyRange::lower_bound(IdbKey::Number(8.0), false);
    assert_eq!(db.count_with_range("store", &lower).unwrap(), 3);

    // upper_bound <= 2
    let upper = IdbKeyRange::upper_bound(IdbKey::Number(2.0), false);
    assert_eq!(db.count_with_range("store", &upper).unwrap(), 2);

    // only(5.0)
    let only = IdbKeyRange::only(IdbKey::Number(5.0));
    assert_eq!(db.count_with_range("store", &only).unwrap(), 1);
}

/// 通过索引范围查询，验证过滤结果正确。
#[test]
fn test_idb_get_all_from_index_with_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    // 插入不同年龄段用户
    db.add(
        "store",
        serde_json::json!({"name": "A", "age": 15}),
        Some(IdbKey::String("u1".into())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"name": "B", "age": 25}),
        Some(IdbKey::String("u2".into())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"name": "C", "age": 35}),
        Some(IdbKey::String("u3".into())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"name": "D", "age": 45}),
        Some(IdbKey::String("u4".into())),
    )
    .unwrap();

    db.create_index("store", "age_idx", "age", false, false).unwrap();

    // 查询 20 <= age <= 40
    let range = IdbKeyRange::bound(IdbKey::Number(20.0), IdbKey::Number(40.0), false, false);
    let results = db.get_all_from_index_with_range("store", "age_idx", &range).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].value["name"], "B");
    assert_eq!(results[1].value["name"], "C");
}

/// 在索引上打开游标，验证迭代顺序按索引键排列。
#[test]
fn test_idb_cursor_on_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    // 按 name 插入顺序为 Z, A, M
    db.add(
        "store",
        serde_json::json!({"name": "Zebra"}),
        Some(IdbKey::String("u1".into())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"name": "Apple"}),
        Some(IdbKey::String("u2".into())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"name": "Mango"}),
        Some(IdbKey::String("u3".into())),
    )
    .unwrap();

    db.create_index("store", "name_idx", "name", false, false).unwrap();

    let mut cursor = db.open_cursor_on_index("store", "name_idx", None).unwrap().unwrap();
    let mut names = Vec::new();
    loop {
        let rec = db.cursor_record(&cursor).unwrap();
        names.push(rec.value["name"].as_str().unwrap().to_string());
        if !cursor.continue_next() {
            break;
        }
    }
    // 应按 name 索引键排序：Apple, Mango, Zebra
    assert_eq!(names, vec!["Apple", "Mango", "Zebra"]);
}

/// 使用键范围批量删除记录，验证剩余记录正确。
#[test]
fn test_idb_delete_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    // 插入 1..=10
    for i in 1..=10 {
        db.add(
            "store",
            serde_json::json!(format!("v{i}")),
            Some(IdbKey::Number(i as f64)),
        )
        .unwrap();
    }
    assert_eq!(db.count("store").unwrap(), 10);

    // 删除范围 [3, 7] 内的记录
    let range = IdbKeyRange::bound(IdbKey::Number(3.0), IdbKey::Number(7.0), false, false);
    let to_delete: Vec<IdbKey> = db
        .get_all_with_range("store", &range)
        .unwrap()
        .into_iter()
        .map(|r| r.key.clone())
        .collect();
    assert_eq!(to_delete.len(), 5, "范围 [3,7] 应包含 5 条记录");

    for key in &to_delete {
        db.delete("store", key).unwrap();
    }

    // 验证剩余记录
    assert_eq!(db.count("store").unwrap(), 5);
    // 1, 2 应保留
    assert!(db.get("store", &IdbKey::Number(1.0)).is_some());
    assert!(db.get("store", &IdbKey::Number(2.0)).is_some());
    // 3..=7 应被删除
    for i in 3..=7 {
        assert!(
            db.get("store", &IdbKey::Number(i as f64)).is_none(),
            "key={i} 应已被删除"
        );
    }
    // 8, 9, 10 应保留
    assert!(db.get("store", &IdbKey::Number(8.0)).is_some());
    assert!(db.get("store", &IdbKey::Number(9.0)).is_some());
    assert!(db.get("store", &IdbKey::Number(10.0)).is_some());
}

/// 测试复合键索引：索引建在多个 key path 组合上（如 [lastName, firstName]），
/// 验证 Array 键按字典序排序，get_from_index 能正确匹配。
#[test]
fn test_idb_compound_key() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("contacts", None, false).unwrap();

    // 插入多条记录，每条有 lastName 和 firstName 字段
    db.add(
        "contacts",
        serde_json::json!({"lastName": "Smith", "firstName": "Anna", "id": 1}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();
    db.add(
        "contacts",
        serde_json::json!({"lastName": "Smith", "firstName": "Bob", "id": 2}),
        Some(IdbKey::Number(2.0)),
    )
    .unwrap();
    db.add(
        "contacts",
        serde_json::json!({"lastName": "Jones", "firstName": "Carol", "id": 3}),
        Some(IdbKey::Number(3.0)),
    )
    .unwrap();
    db.add(
        "contacts",
        serde_json::json!({"lastName": "Adams", "firstName": "Dave", "id": 4}),
        Some(IdbKey::Number(4.0)),
    )
    .unwrap();

    // 创建复合键索引：索引键路径不存在于 JSON 中，这里使用值字段作为索引
    // 先建 name 索引（单字段）
    db.create_index("contacts", "last_idx", "lastName", false, false)
        .unwrap();

    // 查询 lastName == "Smith" 的记录
    let smiths = db
        .get_from_index("contacts", "last_idx", &IdbKey::String("Smith".into()))
        .unwrap();
    assert_eq!(smiths.len(), 2, "应有 2 条 Smith 记录");

    // 验证 get_all_from_index 按 lastName 排序
    let all_by_last = db.get_all_from_index("contacts", "last_idx").unwrap();
    assert_eq!(all_by_last.len(), 4);
    assert_eq!(all_by_last[0].value["lastName"], "Adams");
    assert_eq!(all_by_last[1].value["lastName"], "Jones");
    // Smith 出现两次，顺序不确定（但都在最后）
    assert_eq!(all_by_last[2].value["lastName"], "Smith");
    assert_eq!(all_by_last[3].value["lastName"], "Smith");

    // 使用 Array（复合）键作为主键来测试复合键排序
    db.create_object_store("composite", None, false).unwrap();
    let ck1 = IdbKey::Array(vec![IdbKey::String("Smith".into()), IdbKey::String("Anna".into())]);
    let ck2 = IdbKey::Array(vec![IdbKey::String("Smith".into()), IdbKey::String("Bob".into())]);
    let ck3 = IdbKey::Array(vec![IdbKey::String("Jones".into()), IdbKey::String("Carol".into())]);

    db.add("composite", serde_json::json!({ "v": 1 }), Some(ck1.clone()))
        .unwrap();
    db.add("composite", serde_json::json!({ "v": 2 }), Some(ck2.clone()))
        .unwrap();
    db.add("composite", serde_json::json!({ "v": 3 }), Some(ck3.clone()))
        .unwrap();

    // 游标按键排序迭代，验证 Array 键字典序
    let mut cursor = db.open_cursor("composite", None).unwrap().unwrap();
    let mut keys = Vec::new();
    loop {
        let rec = db.cursor_record(&cursor).unwrap();
        keys.push(rec.value["v"].as_u64().unwrap());
        if !cursor.continue_next() {
            break;
        }
    }
    // 字典序: Jones/Carol < Smith/Anna < Smith/Bob
    assert_eq!(keys, vec![3, 1, 2], "Array 键应按字典序排列");

    // 范围查询：[Smith/Anna, Smith/Bob]
    let range = IdbKeyRange::bound(
        IdbKey::Array(vec![IdbKey::String("Smith".into()), IdbKey::String("Anna".into())]),
        IdbKey::Array(vec![IdbKey::String("Smith".into()), IdbKey::String("Bob".into())]),
        false,
        false,
    );
    let results = db.get_all_with_range("composite", &range).unwrap();
    assert_eq!(results.len(), 2, "范围 [Smith/Anna, Smith/Bob] 应包含 2 条记录");
}

/// 测试唯一索引在 add 时的约束违反：两条不同主键的记录具有相同的唯一索引键值 → 第二次 add 应报错。
///
/// 已知问题：当前 add() 先将记录插入 store.records，再更新索引。
/// 当索引更新失败（唯一约束冲突）时，记录已被添加但 add 返回错误。
/// 这与 IndexedDB 规范不一致——正确行为应为 add 返回错误且记录不被插入。
/// 本测试记录当前（有缺陷的）行为。
#[test]
fn test_idb_unique_constraint_on_add() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("users", None, false).unwrap();

    // 添加第一条记录
    db.add(
        "users",
        serde_json::json!({"email": "alice@example.com", "name": "Alice"}),
        Some(IdbKey::String("user-1".into())),
    )
    .unwrap();

    // 创建唯一索引
    db.create_index("users", "email_idx", "email", true, false).unwrap();

    // 添加第二条记录（不同主键），但 email 字段值与第一条相同
    let result = db.add(
        "users",
        serde_json::json!({"email": "alice@example.com", "name": "Alice Duplicate"}),
        Some(IdbKey::String("user-2".into())),
    );

    // add 应检测到唯一约束冲突并返回错误
    assert!(
        result.is_err(),
        "add() 应因唯一索引约束违反而报错：email 'alice@example.com' 已存在"
    );

    // R3228 已修复：add 违 unique 时记录不再插入（预校验所有 index 全过后才 push record）。
    // 旧实现先 push record 再检查索引，违例时记录已入库。
    assert_eq!(
        db.count("users").unwrap(),
        1,
        "R3228: 违 unique 的 add 不应插入记录（仅 user-1 在库）"
    );

    // 原始记录不应被修改
    let record = db.get("users", &IdbKey::String("user-1".into())).unwrap();
    assert_eq!(record.value["name"], "Alice");

    // 不同 email 值的 add 应成功
    db.add(
        "users",
        serde_json::json!({"email": "bob@example.com", "name": "Bob"}),
        Some(IdbKey::String("user-3".into())),
    )
    .unwrap();
    // R3228：user-2 违 unique 未入库 → 总数 = user-1 + user-3 = 2（旧 buggy 行为 user-2 已入库致 3）。
    assert_eq!(
        db.count("users").unwrap(),
        2,
        "不同索引键值的 add 应成功（user-2 未入库）"
    );
}

/// R3227：NaN 不可作 IndexedDB key（W3C IndexedDB §3.1.6 → DataError）。
/// 旧 cmp_key 对 NaN `partial_cmp().unwrap_or(Equal)` 致 NaN 与任意键「相等」（破坏排序/去重）；
/// add/put 入口现校验 is_valid_key 拒绝 NaN（含 Array 内嵌 NaN）。
#[test]
fn test_idb_nan_key_rejected_r3227() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("s", None, false).unwrap();

    // NaN Number key → add 拒绝。
    let r = db.add("s", serde_json::json!("v"), Some(IdbKey::Number(f64::NAN)));
    assert!(r.is_err(), "R3227: NaN Number key 须拒绝（add）");

    // NaN Number key → put 拒绝。
    let r = db.put("s", serde_json::json!("v"), Some(IdbKey::Number(f64::NAN)));
    assert!(r.is_err(), "R3227: NaN Number key 须拒绝（put）");

    // Array 内嵌 NaN → 拒绝（递归校验）。
    let arr_nan = IdbKey::Array(vec![
        IdbKey::Number(1.0),
        IdbKey::Number(f64::NAN),
        IdbKey::String("x".into()),
    ]);
    let r = db.add("s", serde_json::json!("v"), Some(arr_nan));
    assert!(r.is_err(), "R3227: Array 内嵌 NaN 须拒绝（递归校验）");

    // 合法 Number key（含 Infinity，§3.1.6 仅拒 NaN）→ 接受。
    db.add("s", serde_json::json!("v1"), Some(IdbKey::Number(1.0))).unwrap();
    db.add("s", serde_json::json!("vinf"), Some(IdbKey::Number(f64::INFINITY)))
        .unwrap();
    assert_eq!(db.count("s").unwrap(), 2, "R3227: 合法 Number（含 Infinity）须接受");

    // 合法 Array key（无 NaN）→ 接受。
    let arr_ok = IdbKey::Array(vec![IdbKey::Number(2.0), IdbKey::String("y".into())]);
    db.add("s", serde_json::json!("v2"), Some(arr_ok)).unwrap();
    assert_eq!(db.count("s").unwrap(), 3, "R3227: 合法 Array key 须接受");
}

/// R3229：事务 abort 时 auto-increment key generator 须回滚（W3C IndexedDB §5.10）。
/// 旧实现 tx_add 立即推进 live store.next_key，abort 清缓冲但 next_key 不回滚 → 下次 add 跳过被丢弃的 key（浪费 + spec 偏差）。
/// 现实现：auto-inc 推进事务局部 key_gens，commit_tx 写回，abort 丢弃（store.next_key 未改 → 自动回滚）。
#[test]
fn test_idb_auto_inc_rollback_on_abort_r3229() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("s", None, /* auto_increment */ true).unwrap();

    // tx_add（auto-inc 分配 key 1，推进 tx-local generator 到 2）→ abort。
    let mut tx = db.transaction(&["s"], IdbTransactionMode::ReadWrite).unwrap();
    let k1 = db.tx_add(&tx, "s", serde_json::json!("v1"), None).unwrap();
    assert_eq!(k1, IdbKey::Number(1.0));
    tx.abort().unwrap();

    // R3229：abort 后 key generator 回滚 → 下一次 add 复用 key 1（旧实现 next_key=2 → 用 key 2，浪费）。
    let k2 = db.add("s", serde_json::json!("v2"), None).unwrap();
    assert_eq!(
        k2,
        IdbKey::Number(1.0),
        "R3229: abort 后 key generator 须回滚（下次 add 复用 key 1，非 2）"
    );

    // commit 路径仍推进写回：tx_add + commit_tx → key generator 写回 store.next_key。
    let mut tx2 = db.transaction(&["s"], IdbTransactionMode::ReadWrite).unwrap();
    let k3 = db.tx_add(&tx2, "s", serde_json::json!("v3"), None).unwrap();
    assert_eq!(k3, IdbKey::Number(2.0));
    db.commit_tx(&mut tx2).unwrap();
    // commit 写回后 → 下次 add 用 key 3（key 2 已被 tx2 用，next_key=3）。
    let k4 = db.add("s", serde_json::json!("v4"), None).unwrap();
    assert_eq!(
        k4,
        IdbKey::Number(3.0),
        "R3229: commit 路径 key generator 仍推进写回（next_key=3）"
    );
}

/// 混合操作：add + put + delete + abort，store 不受影响。
#[test]
fn test_tx_mixed_operations_abort() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let key = IdbKey::String("existing".into());
    db.add("store", serde_json::json!("v1"), Some(key.clone())).unwrap();

    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    // add new
    db.tx_add(
        &tx,
        "store",
        serde_json::json!("new"),
        Some(IdbKey::String("new_key".into())),
    )
    .unwrap();
    // put existing
    db.tx_put(&tx, "store", serde_json::json!("updated"), Some(key.clone()))
        .unwrap();
    // delete
    db.tx_delete(&tx, "store", &key).unwrap();
    tx.abort().unwrap();

    // 所有变更都应被丢弃
    let record = db.get("store", &key).unwrap();
    assert_eq!(record.value, serde_json::json!("v1"));
    assert!(db.get("store", &IdbKey::String("new_key".into())).is_none());
    assert_eq!(db.count("store").unwrap(), 1);
}

/// 测试重命名 Object Store 后数据仍可通过新名称访问。
///
/// 验证：
/// 1. 旧名称不再存在
/// 2. 新名称可用
/// 3. 数据和索引在重命名后保持完整
#[test]
fn test_idb_object_store_rename() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("old_store", Some("id"), false).unwrap();

    // 插入数据并创建索引
    db.add(
        "old_store",
        serde_json::json!({"id": 1, "name": "Alice"}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();
    db.add(
        "old_store",
        serde_json::json!({"id": 2, "name": "Bob"}),
        Some(IdbKey::Number(2.0)),
    )
    .unwrap();
    db.create_index("old_store", "name_idx", "name", false, false).unwrap();

    // 重命名
    db.rename_object_store("old_store", "new_store").unwrap();

    // 旧名称不再存在
    assert!(!db.has_store("old_store"), "旧名称应不再存在");
    // 新名称可用
    assert!(db.has_store("new_store"), "新名称应可用");
    assert!(db.store_names().contains(&"new_store"));

    // 数据完整：可以通过新名称访问记录
    let record = db.get("new_store", &IdbKey::Number(1.0)).unwrap();
    assert_eq!(record.value["name"], "Alice");
    assert_eq!(db.count("new_store").unwrap(), 2);

    // 索引完整：可以通过新名称使用索引查询
    let results = db
        .get_from_index("new_store", "name_idx", &IdbKey::String("Bob".into()))
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value["id"], 2);

    // 可以在新名称上继续添加数据
    db.add(
        "new_store",
        serde_json::json!({"id": 3, "name": "Charlie"}),
        Some(IdbKey::Number(3.0)),
    )
    .unwrap();
    assert_eq!(db.count("new_store").unwrap(), 3);

    // 重命名不存在的 store 应报错
    let result = db.rename_object_store("nonexistent", "another");
    assert!(result.is_err());

    // 重命名为已存在的名称应报错
    db.create_object_store("other", None, false).unwrap();
    let result = db.rename_object_store("new_store", "other");
    assert!(result.is_err(), "重命名为已存在的 store 名称应报错");
}

/// 测试 multi-entry 索引在数组值上的行为。
///
/// multiEntry 索引将数组中的每个元素作为独立的索引键。
/// 验证：
/// 1. 数组值中的每个元素都能通过索引查到对应记录
/// 2. 不同记录中的相同数组元素都能被查到
/// 3. 非数组值在 multiEntry 索引中作为单一键处理
/// 4. 范围查询在 multi-entry 索引上正确工作
#[test]
fn test_idb_index_multi_entry() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("articles", None, false).unwrap();

    // 插入多条记录，tags 字段为数组
    db.add(
        "articles",
        serde_json::json!({"title": "Rust Guide", "tags": ["rust", "programming", "tutorial"]}),
        Some(IdbKey::String("a1".into())),
    )
    .unwrap();
    db.add(
        "articles",
        serde_json::json!({"title": "Web Dev", "tags": ["javascript", "html", "programming"]}),
        Some(IdbKey::String("a2".into())),
    )
    .unwrap();
    db.add(
        "articles",
        serde_json::json!({"title": "CSS Tips", "tags": ["css", "html"]}),
        Some(IdbKey::String("a3".into())),
    )
    .unwrap();

    // 创建 multiEntry 索引
    db.create_index("articles", "tags_idx", "tags", false, true).unwrap();

    // 场景 1：查询 "programming" 标签 → 应找到 2 条记录（a1 和 a2）
    let programming = db
        .get_from_index("articles", "tags_idx", &IdbKey::String("programming".into()))
        .unwrap();
    assert_eq!(programming.len(), 2, "programming 标签应匹配 2 条记录");

    // 场景 2：查询 "rust" 标签 → 应找到 1 条记录（a1）
    let rust = db
        .get_from_index("articles", "tags_idx", &IdbKey::String("rust".into()))
        .unwrap();
    assert_eq!(rust.len(), 1);
    assert_eq!(rust[0].value["title"], "Rust Guide");

    // 场景 3：查询 "html" 标签 → 应找到 2 条记录（a2 和 a3）
    let html = db
        .get_from_index("articles", "tags_idx", &IdbKey::String("html".into()))
        .unwrap();
    assert_eq!(html.len(), 2);

    // 场景 4：查询不存在的标签 → 0 条记录
    let unknown = db
        .get_from_index("articles", "tags_idx", &IdbKey::String("python".into()))
        .unwrap();
    assert!(unknown.is_empty());

    // 场景 5：索引计数正确
    // multiEntry 索引条目数 = 所有数组元素总数 = 3 + 3 + 2 = 8
    assert_eq!(db.count_from_index("articles", "tags_idx", None).unwrap(), 8);

    // 场景 6：范围查询在 multi-entry 索引上正确工作
    // 查询标签 >= "html" 的记录（html, javascript, programming, rust, tutorial）
    let range = IdbKeyRange::lower_bound(IdbKey::String("html".into()), false);
    let range_results = db
        .get_all_from_index_with_range("articles", "tags_idx", &range)
        .unwrap();
    // 应该包含所有记录（所有标签 >= "html"）
    assert_eq!(range_results.len(), 3, "范围查询应返回包含匹配标签的 3 条记录");
}

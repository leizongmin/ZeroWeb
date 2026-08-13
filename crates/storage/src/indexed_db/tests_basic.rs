// Auto-generated test file — split from indexed_db.rs
use super::super::*;

#[test]
fn test_idb_database_new() {
    let db = IdbDatabase::new("testdb", 1);
    assert_eq!(db.name, "testdb");
    assert_eq!(db.version, 1);
    assert!(db.store_names().is_empty());
}

#[test]
fn test_idb_create_store() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("users", Some("id"), false).unwrap();
    assert!(db.has_store("users"));
    assert_eq!(db.store_names().len(), 1);
}

#[test]
fn test_idb_delete_store() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("users", Some("id"), false).unwrap();
    db.delete_object_store("users").unwrap();
    assert!(!db.has_store("users"));
}

#[test]
fn test_idb_store_names() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("a", None, false).unwrap();
    db.create_object_store("b", None, false).unwrap();
    let names = db.store_names();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"a"));
    assert!(names.contains(&"b"));
}

#[test]
fn test_idb_add_record() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("users", Some("id"), false).unwrap();
    let key = IdbKey::String("user1".to_string());
    let value = serde_json::json!({"name": "Alice"});
    let returned_key = db.add("users", value, Some(key.clone())).unwrap();
    assert_eq!(returned_key, key);

    let record = db.get("users", &IdbKey::String("user1".to_string())).unwrap();
    assert_eq!(record.value["name"], "Alice");
}

#[test]
fn test_idb_add_with_auto_key() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("items", None, true).unwrap();

    let k1 = db.add("items", serde_json::json!({"v": 1}), None).unwrap();
    let k2 = db.add("items", serde_json::json!({"v": 2}), None).unwrap();

    assert_eq!(k1, IdbKey::Number(1.0));
    assert_eq!(k2, IdbKey::Number(2.0));
    assert_eq!(db.count("items").unwrap(), 2);
}

#[test]
fn test_idb_put_overwrite() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("users", Some("id"), false).unwrap();
    let key = IdbKey::String("user1".to_string());
    db.add("users", serde_json::json!({"name": "Alice"}), Some(key.clone()))
        .unwrap();
    db.put("users", serde_json::json!({"name": "Bob"}), Some(key.clone()))
        .unwrap();

    let record = db.get("users", &key).unwrap();
    assert_eq!(record.value["name"], "Bob");
    assert_eq!(db.count("users").unwrap(), 1);
}

#[test]
fn test_idb_get_record() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("store", None, false).unwrap();
    let key = IdbKey::Number(42.0);
    db.add("store", serde_json::json!("hello"), Some(key.clone())).unwrap();
    assert!(db.get("store", &key).is_some());
}

#[test]
fn test_idb_get_nonexistent() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("store", None, false).unwrap();
    assert_eq!(db.get("store", &IdbKey::String("nope".to_string())), None);
}

#[test]
fn test_idb_delete_record() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("store", None, false).unwrap();
    let key = IdbKey::String("k".to_string());
    db.add("store", serde_json::json!(1), Some(key.clone())).unwrap();
    let deleted = db.delete("store", &key).unwrap();
    assert!(deleted);
    assert_eq!(db.get("store", &key), None);
}

#[test]
fn test_idb_get_all() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!(1), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", serde_json::json!(2), Some(IdbKey::Number(2.0)))
        .unwrap();
    let all = db.get_all("store").unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn test_idb_clear_store() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!(1), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.clear_store("store").unwrap();
    assert_eq!(db.count("store").unwrap(), 0);
}

#[test]
fn test_idb_count() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("store", None, false).unwrap();
    assert_eq!(db.count("store").unwrap(), 0);
    db.add("store", serde_json::json!("a"), Some(IdbKey::String("k1".to_string())))
        .unwrap();
    db.add("store", serde_json::json!("b"), Some(IdbKey::String("k2".to_string())))
        .unwrap();
    assert_eq!(db.count("store").unwrap(), 2);
}

#[test]
fn test_idb_key_ordering() {
    let num_key = IdbKey::Number(1.0);
    let str_key = IdbKey::String("a".to_string());
    let bin_key = IdbKey::Binary(vec![1, 2]);
    let arr_key = IdbKey::Array(vec![IdbKey::Number(1.0)]);

    assert!(num_key < str_key);
    assert!(str_key < bin_key);
    assert!(bin_key < arr_key);
}

#[test]
fn test_idb_duplicate_key_add() {
    let mut db = IdbDatabase::new("testdb", 1);
    db.create_object_store("store", None, false).unwrap();
    let key = IdbKey::String("dup".to_string());
    db.add("store", serde_json::json!(1), Some(key.clone())).unwrap();
    let result = db.add("store", serde_json::json!(2), Some(key));
    assert!(result.is_err());
}

#[test]
fn test_idb_delete_nonexistent_store() {
    let mut db = IdbDatabase::new("testdb", 1);
    let result = db.delete_object_store("noexist");
    assert!(result.is_err());
}

// ── 边界条件补充测试 ──

/// 测试空数据库名称。
#[test]
fn test_idb_empty_database_name() {
    let db = IdbDatabase::new("", 1);
    assert_eq!(db.name, "");
}

/// 测试版本号为 0。
#[test]
fn test_idb_version_zero() {
    let db = IdbDatabase::new("test", 0);
    assert_eq!(db.version, 0);
}

/// 测试多个 object store 操作。
#[test]
fn test_idb_multiple_stores() {
    let mut db = IdbDatabase::new("multi", 1);
    db.create_object_store("users", None, false).unwrap();
    db.create_object_store("products", None, false).unwrap();
    db.create_object_store("orders", None, false).unwrap();

    assert_eq!(db.store_names().len(), 3);
    assert!(db.store_names().contains(&"users"));
    assert!(db.store_names().contains(&"products"));
    assert!(db.store_names().contains(&"orders"));

    // 删除中间的
    db.delete_object_store("products").unwrap();
    assert_eq!(db.store_names().len(), 2);
    assert!(!db.store_names().contains(&"products"));
}

/// 测试 get_all 在空 store 上。
#[test]
fn test_idb_get_all_empty() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("empty", None, false).unwrap();
    let records = db.get_all("empty").unwrap();
    assert!(records.is_empty());
}

/// 测试 count 在空 store 上。
#[test]
fn test_idb_count_empty() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("empty", None, false).unwrap();
    assert_eq!(db.count("empty").unwrap(), 0);
}

/// 测试 clear_store 后 count 为 0。
#[test]
fn test_idb_clear_then_count() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, true).unwrap();
    db.add("items", serde_json::json!("val1"), None).unwrap();
    db.add("items", serde_json::json!("val2"), None).unwrap();
    assert_eq!(db.count("items").unwrap(), 2);
    db.clear_store("items").unwrap();
    assert_eq!(db.count("items").unwrap(), 0);
}

/// 测试 get 在不存在的 store 上。
#[test]
fn test_idb_get_from_nonexistent_store() {
    let db = IdbDatabase::new("test", 1);
    let result = db.get("noexist", &IdbKey::String("key".into()));
    assert!(result.is_none());
}

/// 测试 IdbKey 排序：Number < String < Binary < Array。
#[test]
fn test_idb_key_type_ordering() {
    let num = IdbKey::Number(1.0);
    let str_key = IdbKey::String("a".into());
    let bin = IdbKey::Binary(vec![1]);
    let arr = IdbKey::Array(vec![IdbKey::Number(1.0)]);

    assert!(num < str_key);
    assert!(str_key < bin);
    assert!(bin < arr);
}

/// 测试 has_store。
#[test]
fn test_idb_has_store() {
    let mut db = IdbDatabase::new("test", 1);
    assert!(!db.has_store("users"));
    db.create_object_store("users", None, false).unwrap();
    assert!(db.has_store("users"));
    assert!(!db.has_store("products"));
}

/// 测试重复创建 object store 报错。
#[test]
fn test_idb_create_duplicate_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    let result = db.create_object_store("items", None, false);
    assert!(result.is_err());
}

/// 测试 delete 记录返回是否找到。
#[test]
fn test_idb_delete_returns_found() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    let key = IdbKey::String("k1".into());
    db.add("items", serde_json::json!("v1"), Some(key.clone())).unwrap();

    let found = db.delete("items", &key).unwrap();
    assert!(found);

    let not_found = db.delete("items", &key).unwrap();
    assert!(!not_found);
}

/// 测试 put 覆盖已有记录。
#[test]
fn test_idb_put_overwrites_value() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("items", None, false).unwrap();
    let key = IdbKey::String("k".into());
    db.add("items", serde_json::json!("v1"), Some(key.clone())).unwrap();
    db.put("items", serde_json::json!("v2"), Some(key.clone())).unwrap();

    let record = db.get("items", &key).unwrap();
    assert_eq!(record.value, serde_json::json!("v2"));
    // 只有一条记录（put 覆盖而不是新增）
    assert_eq!(db.count("items").unwrap(), 1);
}

// ── IdbKeyRange 测试 ──

#[test]
fn test_key_range_only() {
    let range = IdbKeyRange::only(IdbKey::Number(5.0));
    assert!(range.contains(&IdbKey::Number(5.0)));
    assert!(!range.contains(&IdbKey::Number(4.0)));
    assert!(!range.contains(&IdbKey::Number(6.0)));
}

#[test]
fn test_key_range_lower_bound_closed() {
    let range = IdbKeyRange::lower_bound(IdbKey::Number(3.0), false);
    assert!(range.contains(&IdbKey::Number(3.0)));
    assert!(range.contains(&IdbKey::Number(10.0)));
    assert!(!range.contains(&IdbKey::Number(2.0)));
}

#[test]
fn test_key_range_lower_bound_open() {
    let range = IdbKeyRange::lower_bound(IdbKey::Number(3.0), true);
    assert!(!range.contains(&IdbKey::Number(3.0)));
    assert!(range.contains(&IdbKey::Number(4.0)));
}

#[test]
fn test_key_range_upper_bound_closed() {
    let range = IdbKeyRange::upper_bound(IdbKey::Number(10.0), false);
    assert!(range.contains(&IdbKey::Number(10.0)));
    assert!(range.contains(&IdbKey::Number(5.0)));
    assert!(!range.contains(&IdbKey::Number(11.0)));
}

#[test]
fn test_key_range_upper_bound_open() {
    let range = IdbKeyRange::upper_bound(IdbKey::Number(10.0), true);
    assert!(!range.contains(&IdbKey::Number(10.0)));
    assert!(range.contains(&IdbKey::Number(9.0)));
}

#[test]
fn test_key_range_bound_closed() {
    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), false, false);
    assert!(!range.contains(&IdbKey::Number(0.0)));
    assert!(range.contains(&IdbKey::Number(1.0)));
    assert!(range.contains(&IdbKey::Number(5.0)));
    assert!(range.contains(&IdbKey::Number(10.0)));
    assert!(!range.contains(&IdbKey::Number(11.0)));
}

#[test]
fn test_key_range_bound_open() {
    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), true, true);
    assert!(!range.contains(&IdbKey::Number(1.0)));
    assert!(range.contains(&IdbKey::Number(2.0)));
    assert!(!range.contains(&IdbKey::Number(10.0)));
}

#[test]
fn test_key_range_accessors() {
    let range = IdbKeyRange::bound(IdbKey::Number(1.0), IdbKey::Number(10.0), true, false);
    assert_eq!(range.lower(), Some(&IdbKey::Number(1.0)));
    assert_eq!(range.upper(), Some(&IdbKey::Number(10.0)));
    assert!(range.lower_open());
    assert!(!range.upper_open());
}

#[test]
fn test_key_range_string_keys() {
    let range = IdbKeyRange::bound(IdbKey::String("c".into()), IdbKey::String("f".into()), false, false);
    assert!(!range.contains(&IdbKey::String("b".into())));
    assert!(range.contains(&IdbKey::String("c".into())));
    assert!(range.contains(&IdbKey::String("d".into())));
    assert!(range.contains(&IdbKey::String("f".into())));
    assert!(!range.contains(&IdbKey::String("g".into())));
}

// ── get_all_with_range / count_with_range 测试 ──

#[test]
fn test_get_all_with_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", serde_json::json!("b"), Some(IdbKey::Number(5.0)))
        .unwrap();
    db.add("store", serde_json::json!("c"), Some(IdbKey::Number(10.0)))
        .unwrap();
    db.add("store", serde_json::json!("d"), Some(IdbKey::Number(15.0)))
        .unwrap();

    let range = IdbKeyRange::bound(IdbKey::Number(5.0), IdbKey::Number(10.0), false, false);
    let results = db.get_all_with_range("store", &range).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].value, serde_json::json!("b"));
    assert_eq!(results[1].value, serde_json::json!("c"));
}

#[test]
fn test_count_with_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!(1), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", serde_json::json!(2), Some(IdbKey::Number(5.0)))
        .unwrap();
    db.add("store", serde_json::json!(3), Some(IdbKey::Number(10.0)))
        .unwrap();

    let range = IdbKeyRange::lower_bound(IdbKey::Number(5.0), false);
    assert_eq!(db.count_with_range("store", &range).unwrap(), 2);
}

// ── 索引测试 ──

#[test]
fn test_create_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("users", None, false).unwrap();
    db.add(
        "users",
        serde_json::json!({"name": "Alice", "age": 30}),
        Some(IdbKey::String("u1".into())),
    )
    .unwrap();
    db.add(
        "users",
        serde_json::json!({"name": "Bob", "age": 25}),
        Some(IdbKey::String("u2".into())),
    )
    .unwrap();

    db.create_index("users", "name_idx", "name", false, false).unwrap();
    let names = db.index_names("users").unwrap();
    assert_eq!(names.len(), 1);
    assert!(names.contains(&"name_idx"));
}

#[test]
fn test_get_from_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("users", None, false).unwrap();
    db.add(
        "users",
        serde_json::json!({"name": "Alice", "age": 30}),
        Some(IdbKey::String("u1".into())),
    )
    .unwrap();
    db.add(
        "users",
        serde_json::json!({"name": "Bob", "age": 25}),
        Some(IdbKey::String("u2".into())),
    )
    .unwrap();

    db.create_index("users", "name_idx", "name", false, false).unwrap();
    let results = db
        .get_from_index("users", "name_idx", &IdbKey::String("Alice".into()))
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value["age"], 30);
}

#[test]
fn test_get_all_from_index_sorted() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("users", None, false).unwrap();
    db.add(
        "users",
        serde_json::json!({"name": "Charlie"}),
        Some(IdbKey::String("u1".into())),
    )
    .unwrap();
    db.add(
        "users",
        serde_json::json!({"name": "Alice"}),
        Some(IdbKey::String("u2".into())),
    )
    .unwrap();
    db.add(
        "users",
        serde_json::json!({"name": "Bob"}),
        Some(IdbKey::String("u3".into())),
    )
    .unwrap();

    db.create_index("users", "name_idx", "name", false, false).unwrap();
    let results = db.get_all_from_index("users", "name_idx").unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].value["name"], "Alice");
    assert_eq!(results[1].value["name"], "Bob");
    assert_eq!(results[2].value["name"], "Charlie");
}

/// R3385 回归锁定：`add` 在 `create_index` **之后** 插入记录时，索引条目经
/// `commit_entry_from_record` 提交（而非 rebuild），返回结果仍须按索引键有序。
/// 旧实现 commit 仅 push 不重排 → 失序，违反 W3C「getAllFromIndex 按索引键序」。
/// （`test_get_all_from_index_sorted` 先 add 再 create_index 走 rebuild，漏此路径。）
#[test]
fn test_get_all_from_index_sorted_after_post_index_add_r3385() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("users", None, false).unwrap();
    // 先建索引，再 add（走 commit_entry_from_record 路径）。
    db.create_index("users", "name_idx", "name", false, false).unwrap();
    db.add(
        "users",
        serde_json::json!({"name": "Charlie"}),
        Some(IdbKey::String("u1".into())),
    )
    .unwrap();
    db.add(
        "users",
        serde_json::json!({"name": "Alice"}),
        Some(IdbKey::String("u2".into())),
    )
    .unwrap();
    db.add(
        "users",
        serde_json::json!({"name": "Bob"}),
        Some(IdbKey::String("u3".into())),
    )
    .unwrap();

    let results = db.get_all_from_index("users", "name_idx").unwrap();
    assert_eq!(results.len(), 3);
    // 须按索引键（name 字典序）有序：Alice < Bob < Charlie。
    assert_eq!(results[0].value["name"], "Alice");
    assert_eq!(results[1].value["name"], "Bob");
    assert_eq!(results[2].value["name"], "Charlie");
}

/// R3385 回归锁定：经索引的范围查询（get_all_from_index_with_range）在
/// create_index 后 add 的记录上也须按键序返回完整结果集。
#[test]
fn test_get_all_from_index_with_range_after_post_index_add_r3385() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("users", None, false).unwrap();
    db.create_index("users", "name_idx", "name", false, false).unwrap();
    // 倒序插入（C, A, B），逼出 commit 路径的失序。
    for (name, pk) in [("Charlie", "u1"), ("Alice", "u2"), ("Bob", "u3")] {
        db.add(
            "users",
            serde_json::json!({"name": name}),
            Some(IdbKey::String(pk.into())),
        )
        .unwrap();
    }
    // 范围 [A, D] 闭区间，应返回全部 3 条且有序（"Charlie" < "D"）。
    let range = IdbKeyRange::bound(IdbKey::String("A".into()), IdbKey::String("D".into()), false, false);
    let results = db.get_all_from_index_with_range("users", "name_idx", &range).unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].value["name"], "Alice");
    assert_eq!(results[1].value["name"], "Bob");
    assert_eq!(results[2].value["name"], "Charlie");
}

#[test]
fn test_get_all_from_index_with_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("users", None, false).unwrap();
    db.add(
        "users",
        serde_json::json!({"age": 20}),
        Some(IdbKey::String("u1".into())),
    )
    .unwrap();
    db.add(
        "users",
        serde_json::json!({"age": 30}),
        Some(IdbKey::String("u2".into())),
    )
    .unwrap();
    db.add(
        "users",
        serde_json::json!({"age": 40}),
        Some(IdbKey::String("u3".into())),
    )
    .unwrap();

    db.create_index("users", "age_idx", "age", false, false).unwrap();
    let range = IdbKeyRange::bound(IdbKey::Number(25.0), IdbKey::Number(35.0), false, false);
    let results = db.get_all_from_index_with_range("users", "age_idx", &range).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value["age"], 30);
}

#[test]
fn test_index_unique_constraint() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("users", None, false).unwrap();
    db.add(
        "users",
        serde_json::json!({"email": "a@b.com"}),
        Some(IdbKey::String("u1".into())),
    )
    .unwrap();
    db.create_index("users", "email_idx", "email", true, false).unwrap();

    let result = db.add(
        "users",
        serde_json::json!({"email": "a@b.com"}),
        Some(IdbKey::String("u2".into())),
    );
    assert!(result.is_err());
}

#[test]
fn test_delete_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.create_index("store", "idx", "field", false, false).unwrap();
    assert_eq!(db.index_names("store").unwrap().len(), 1);
    db.delete_index("store", "idx").unwrap();
    assert_eq!(db.index_names("store").unwrap().len(), 0);
}

#[test]
fn test_index_updated_on_delete() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add(
        "store",
        serde_json::json!({"tag": "a"}),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();
    db.create_index("store", "tag_idx", "tag", false, false).unwrap();

    let results = db
        .get_from_index("store", "tag_idx", &IdbKey::String("a".into()))
        .unwrap();
    assert_eq!(results.len(), 1);

    db.delete("store", &IdbKey::String("k1".into())).unwrap();
    let results = db
        .get_from_index("store", "tag_idx", &IdbKey::String("a".into()))
        .unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_index_updated_on_put() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add(
        "store",
        serde_json::json!({"tag": "a"}),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();
    db.create_index("store", "tag_idx", "tag", false, false).unwrap();

    db.put(
        "store",
        serde_json::json!({"tag": "b"}),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();

    let results_a = db
        .get_from_index("store", "tag_idx", &IdbKey::String("a".into()))
        .unwrap();
    assert!(results_a.is_empty());
    let results_b = db
        .get_from_index("store", "tag_idx", &IdbKey::String("b".into()))
        .unwrap();
    assert_eq!(results_b.len(), 1);
}

#[test]
fn test_count_from_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!({"v": 1}), Some(IdbKey::String("k1".into())))
        .unwrap();
    db.add("store", serde_json::json!({"v": 2}), Some(IdbKey::String("k2".into())))
        .unwrap();
    db.add("store", serde_json::json!({"v": 3}), Some(IdbKey::String("k3".into())))
        .unwrap();

    db.create_index("store", "v_idx", "v", false, false).unwrap();
    assert_eq!(db.count_from_index("store", "v_idx", None).unwrap(), 3);

    let range = IdbKeyRange::lower_bound(IdbKey::Number(2.0), false);
    assert_eq!(db.count_from_index("store", "v_idx", Some(&range)).unwrap(), 2);
}

#[test]
fn test_clear_store_clears_indexes() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!({"x": 1}), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.create_index("store", "x_idx", "x", false, false).unwrap();
    assert_eq!(db.count_from_index("store", "x_idx", None).unwrap(), 1);
    db.clear_store("store").unwrap();
    assert_eq!(db.count_from_index("store", "x_idx", None).unwrap(), 0);
}

#[test]
fn test_multi_entry_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add(
        "store",
        serde_json::json!({"tags": ["red", "blue"]}),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"tags": ["blue", "green"]}),
        Some(IdbKey::String("k2".into())),
    )
    .unwrap();

    db.create_index("store", "tags_idx", "tags", false, true).unwrap();
    let results = db
        .get_from_index("store", "tags_idx", &IdbKey::String("blue".into()))
        .unwrap();
    assert_eq!(results.len(), 2);
}

// ── 游标测试 ──

#[test]
fn test_open_cursor_basic() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!("c"), Some(IdbKey::Number(3.0)))
        .unwrap();
    db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", serde_json::json!("b"), Some(IdbKey::Number(2.0)))
        .unwrap();

    let cursor = db.open_cursor("store", None).unwrap().unwrap();
    let rec = db.cursor_record(&cursor).unwrap();
    assert_eq!(rec.value, serde_json::json!("a"));
}

#[test]
fn test_cursor_advance() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", serde_json::json!("b"), Some(IdbKey::Number(2.0)))
        .unwrap();
    db.add("store", serde_json::json!("c"), Some(IdbKey::Number(3.0)))
        .unwrap();

    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
    assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!("a"));

    assert!(cursor.continue_next());
    assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!("b"));

    assert!(cursor.continue_next());
    assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!("c"));

    assert!(!cursor.continue_next());
    assert!(cursor.is_finished());
}

#[test]
fn test_cursor_with_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!(1), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.add("store", serde_json::json!(2), Some(IdbKey::Number(2.0)))
        .unwrap();
    db.add("store", serde_json::json!(3), Some(IdbKey::Number(3.0)))
        .unwrap();
    db.add("store", serde_json::json!(4), Some(IdbKey::Number(4.0)))
        .unwrap();

    let range = IdbKeyRange::bound(IdbKey::Number(2.0), IdbKey::Number(3.0), false, false);
    let mut cursor = db.open_cursor("store", Some(&range)).unwrap().unwrap();
    assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!(2));
    assert!(cursor.continue_next());
    assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!(3));
    assert!(!cursor.continue_next());
}

#[test]
fn test_open_key_cursor() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add("store", serde_json::json!("c"), Some(IdbKey::Number(3.0)))
        .unwrap();
    db.add("store", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
        .unwrap();

    let mut cursor = db.open_key_cursor("store", None).unwrap().unwrap();
    assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(1.0)));
    assert!(cursor.advance(1));
    assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(3.0)));
    assert!(!cursor.advance(1));
}

#[test]
fn test_open_cursor_on_index() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add(
        "store",
        serde_json::json!({"name": "Charlie"}),
        Some(IdbKey::String("u1".into())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"name": "Alice"}),
        Some(IdbKey::String("u2".into())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"name": "Bob"}),
        Some(IdbKey::String("u3".into())),
    )
    .unwrap();

    db.create_index("store", "name_idx", "name", false, false).unwrap();
    let mut cursor = db.open_cursor_on_index("store", "name_idx", None).unwrap().unwrap();
    assert_eq!(db.cursor_record(&cursor).unwrap().value["name"], "Alice");
    assert!(cursor.continue_next());
    assert_eq!(db.cursor_record(&cursor).unwrap().value["name"], "Bob");
    assert!(cursor.continue_next());
    assert_eq!(db.cursor_record(&cursor).unwrap().value["name"], "Charlie");
}

#[test]
fn test_open_cursor_empty_store() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    assert!(db.open_cursor("store", None).unwrap().is_none());
}

// ── 事务测试 ──

#[test]
fn test_transaction_create() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    assert_eq!(tx.mode(), IdbTransactionMode::ReadWrite);
    assert_eq!(tx.store_names().len(), 1);
    assert!(!tx.is_committed());
    assert!(!tx.is_aborted());
}

#[test]
fn test_transaction_commit() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    tx.commit().unwrap();
    assert!(tx.is_committed());
}

#[test]
fn test_transaction_abort() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    tx.abort().unwrap();
    assert!(tx.is_aborted());
}

#[test]
fn test_transaction_double_commit() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    tx.commit().unwrap();
    assert!(tx.commit().is_err());
}

#[test]
fn test_transaction_abort_after_commit() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    tx.commit().unwrap();
    assert!(tx.abort().is_err());
}

#[test]
fn test_transaction_nonexistent_store() {
    let mut db = IdbDatabase::new("test", 1);
    let result = db.transaction(&["noexist"], IdbTransactionMode::ReadOnly);
    assert!(result.is_err());
}

#[test]
fn test_tx_operations() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();

    let key = db
        .tx_add(
            &tx,
            "store",
            serde_json::json!("hello"),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();
    assert_eq!(key, IdbKey::String("k1".into()));

    let record = db.tx_get(&tx, "store", &IdbKey::String("k1".into())).unwrap();
    assert_eq!(record.unwrap().value, serde_json::json!("hello"));
}

#[test]
fn test_tx_operations_out_of_scope() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("a", None, false).unwrap();
    db.create_object_store("b", None, false).unwrap();
    let tx = db.transaction(&["a"], IdbTransactionMode::ReadWrite).unwrap();

    let result = db.tx_add(&tx, "b", serde_json::json!(1), Some(IdbKey::Number(1.0)));
    assert!(result.is_err());
}

#[test]
fn test_tx_operations_after_abort() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    tx.abort().unwrap();

    let result = db.tx_add(&tx, "store", serde_json::json!(1), Some(IdbKey::Number(1.0)));
    assert!(result.is_err());
}

// ── 新增测试：事务 ──

#[test]
fn test_transaction_commit_then_operations_fail() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    tx.commit().unwrap();
    // After commit, operations should fail
    let result = db.tx_add(&tx, "store", serde_json::json!("val"), Some(IdbKey::String("k".into())));
    assert!(result.is_err());
}

#[test]
fn test_transaction_read_only_mode() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadOnly).unwrap();
    assert_eq!(tx.mode(), IdbTransactionMode::ReadOnly);
}

#[test]
fn test_transaction_multiple_stores() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("a", None, false).unwrap();
    db.create_object_store("b", None, false).unwrap();
    let tx = db.transaction(&["a", "b"], IdbTransactionMode::ReadWrite).unwrap();
    assert_eq!(tx.store_names().len(), 2);

    // Can add to both stores within the same transaction
    db.tx_add(&tx, "a", serde_json::json!(1), Some(IdbKey::Number(1.0)))
        .unwrap();
    db.tx_add(&tx, "b", serde_json::json!(2), Some(IdbKey::Number(2.0)))
        .unwrap();
}

#[test]
fn test_transaction_double_abort() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    tx.abort().unwrap();
    // Second abort should fail
    assert!(tx.abort().is_err());
}

#[test]
fn test_transaction_commit_after_abort_fails() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let mut tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    tx.abort().unwrap();
    // Commit after abort should fail
    assert!(tx.commit().is_err());
}

#[test]
fn test_tx_put_and_delete() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    let tx = db.transaction(&["store"], IdbTransactionMode::ReadWrite).unwrap();
    let key = IdbKey::String("k1".into());
    db.tx_put(&tx, "store", serde_json::json!("v1"), Some(key.clone()))
        .unwrap();
    let rec = db.tx_get(&tx, "store", &key).unwrap().unwrap();
    assert_eq!(rec.value, serde_json::json!("v1"));
    let deleted = db.tx_delete(&tx, "store", &key).unwrap();
    assert!(deleted);
    assert!(db.tx_get(&tx, "store", &key).unwrap().is_none());
}

// ── 新增测试：游标与索引 ──

#[test]
fn test_cursor_forward_iteration_all() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    for i in 1..=5 {
        db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }
    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
    let mut collected = Vec::new();
    loop {
        let rec = db.cursor_record(&cursor).unwrap();
        collected.push(rec.value.as_u64().unwrap());
        if !cursor.continue_next() {
            break;
        }
    }
    assert_eq!(collected, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_cursor_with_lower_bound_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    for i in 1..=5 {
        db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }
    let range = IdbKeyRange::lower_bound(IdbKey::Number(3.0), false);
    let cursor = db.open_cursor("store", Some(&range)).unwrap().unwrap();
    let rec = db.cursor_record(&cursor).unwrap();
    assert_eq!(rec.value, serde_json::json!(3));
}

#[test]
fn test_key_cursor_continue_to() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    for i in 1..=5 {
        db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }
    let mut cursor = db.open_key_cursor("store", None).unwrap().unwrap();
    assert!(cursor.continue_to(&IdbKey::Number(4.0)));
    assert_eq!(db.cursor_key(&cursor), Some(&IdbKey::Number(4.0)));
}

#[test]
fn test_cursor_advance_skip() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    for i in 1..=5 {
        db.add("store", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
            .unwrap();
    }
    let mut cursor = db.open_cursor("store", None).unwrap().unwrap();
    // Skip 2 positions (from 0 to 2, landing on 3rd record)
    assert!(cursor.advance(2));
    let rec = db.cursor_record(&cursor).unwrap();
    assert_eq!(rec.value, serde_json::json!(3));
}

#[test]
fn test_index_rebuild_after_add() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add(
        "store",
        serde_json::json!({"cat": "a"}),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();
    db.create_index("store", "cat_idx", "cat", false, false).unwrap();
    // Add record after index creation — index should update
    db.add(
        "store",
        serde_json::json!({"cat": "b"}),
        Some(IdbKey::String("k2".into())),
    )
    .unwrap();
    let results = db
        .get_from_index("store", "cat_idx", &IdbKey::String("b".into()))
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value["cat"], "b");
}

#[test]
fn test_index_unique_allows_different_values() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add(
        "store",
        serde_json::json!({"code": "AAA"}),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();
    db.create_index("store", "code_idx", "code", true, false).unwrap();
    // Different value should succeed
    db.add(
        "store",
        serde_json::json!({"code": "BBB"}),
        Some(IdbKey::String("k2".into())),
    )
    .unwrap();
    assert_eq!(db.count("store").unwrap(), 2);
}

#[test]
fn test_multi_entry_index_single_match() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add(
        "store",
        serde_json::json!({"tags": ["rust", "web"]}),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"tags": ["python"]}),
        Some(IdbKey::String("k2".into())),
    )
    .unwrap();
    db.create_index("store", "tags_idx", "tags", false, true).unwrap();
    let results = db
        .get_from_index("store", "tags_idx", &IdbKey::String("rust".into()))
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value["tags"][0], "rust");
}

#[test]
fn test_open_cursor_on_index_with_range() {
    let mut db = IdbDatabase::new("test", 1);
    db.create_object_store("store", None, false).unwrap();
    db.add(
        "store",
        serde_json::json!({"score": 10}),
        Some(IdbKey::String("k1".into())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"score": 20}),
        Some(IdbKey::String("k2".into())),
    )
    .unwrap();
    db.add(
        "store",
        serde_json::json!({"score": 30}),
        Some(IdbKey::String("k3".into())),
    )
    .unwrap();
    db.create_index("store", "score_idx", "score", false, false).unwrap();
    let range = IdbKeyRange::lower_bound(IdbKey::Number(20.0), false);
    let mut cursor = db
        .open_cursor_on_index("store", "score_idx", Some(&range))
        .unwrap()
        .unwrap();
    assert_eq!(db.cursor_record(&cursor).unwrap().value["score"], 20);
    assert!(cursor.continue_next());
    assert_eq!(db.cursor_record(&cursor).unwrap().value["score"], 30);
    assert!(!cursor.continue_next());
}

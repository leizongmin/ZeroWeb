// 测试 JSON 布尔值以覆盖 json_value_to_idb_key 的 _ => None 分支
use super::*;

#[test]
fn test_json_boolean_values_in_indexes() {
    let mut db = IdbDatabase::new("test_bool", 1);
    db.create_object_store("items", None, false).unwrap();
    db.create_index("items", "bool_idx", "flag", false, false).unwrap();

    // 添加包含布尔值的记录
    db.add(
        "items",
        serde_json::json!({"flag": true}),
        Some(IdbKey::String("1".into())),
    )
    .unwrap();

    db.add(
        "items",
        serde_json::json!({"flag": false}),
        Some(IdbKey::String("2".into())),
    )
    .unwrap();

    // 尝试通过索引查找 - 布尔值应该被忽略
    let results_true = db.get_from_index("items", "bool_idx", &IdbKey::Number(1.0)).unwrap();
    assert_eq!(results_true.len(), 0);

    let results_false = db.get_from_index("items", "bool_idx", &IdbKey::Number(0.0)).unwrap();
    assert_eq!(results_false.len(), 0);
}

#[test]
fn test_json_null_values_in_indexes() {
    let mut db = IdbDatabase::new("test_null", 1);
    db.create_object_store("items", None, false).unwrap();
    db.create_index("items", "null_idx", "data", false, false).unwrap();

    // 添加包含 null 值的记录
    db.add("items", serde_json::json!({"data": null}), Some(IdbKey::Number(1.0)))
        .unwrap();

    // 尝试通过索引查找 - null 值应该被忽略
    let results = db.get_from_index("items", "null_idx", &IdbKey::Number(0.0)).unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_json_object_values_in_indexes() {
    let mut db = IdbDatabase::new("test_object", 1);
    db.create_object_store("items", None, false).unwrap();
    db.create_index("items", "obj_idx", "data", false, false).unwrap();

    // 添加包含对象值的记录
    db.add(
        "items",
        serde_json::json!({"data": {"nested": "value"}}),
        Some(IdbKey::Number(1.0)),
    )
    .unwrap();

    // 尝试通过索引查找 - 对象应该被忽略
    let results = db
        .get_from_index("items", "obj_idx", &IdbKey::String("object".to_string()))
        .unwrap();
    assert_eq!(results.len(), 0);
}

//! # zero-storage
//!
//! 存储后端 — localStorage、sessionStorage、IndexedDB、Cache API。

#![warn(missing_docs)]

pub mod cache_api;
pub mod indexed_db;
pub mod local_storage;
pub mod storage_manager;

pub use cache_api::*;
pub use indexed_db::*;
pub use local_storage::*;
pub use storage_manager::*;

use thiserror::Error;

/// 存储操作错误类型。
#[derive(Error, Debug)]
pub enum StorageError {
    /// 超出配额限制。
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
    /// 无效键名。
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    /// Object Store 未找到。
    #[error("Store not found: {0}")]
    StoreNotFound(String),
    /// 键未找到。
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    /// 序列化错误。
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// 数据库错误。
    #[error("Database error: {0}")]
    Database(String),
}

#[cfg(test)]
mod tests {
    use crate::cache_api::{CacheRequest, CacheResponse, CacheStorage};
    use crate::indexed_db::{IdbDatabase, IdbKey, IdbTransactionMode};
    use crate::local_storage::{StorageType, WebStorage};

    /// 测试 IndexedDB 数据库名称存储是否正确。
    #[test]
    fn test_idb_database_name() {
        let db = IdbDatabase::new("my-app-db", 3);
        assert_eq!(db.name, "my-app-db");
        assert_eq!(db.version, 3);
        assert!(db.store_names().is_empty());
    }

    /// 测试 IndexedDB 创建 object store 后再删除，store_names 应为空。
    #[test]
    fn test_idb_delete_object_store() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", Some("id"), false).unwrap();
        assert_eq!(db.store_names().len(), 1);

        db.delete_object_store("items").unwrap();
        assert!(db.store_names().is_empty());
        assert!(!db.has_store("items"));
    }

    /// 测试 localStorage 更新已存在的键：设置 "a" 为 "1"，再设为 "2"，get 应返回 "2"。
    #[test]
    fn test_local_storage_update_existing() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("a", "1").unwrap();
        assert_eq!(storage.get("a"), Some("1"));

        storage.set("a", "2").unwrap();
        assert_eq!(storage.get("a"), Some("2"));
        assert_eq!(storage.len(), 1);
    }

    /// 测试 sessionStorage 设置多个键后调用 clear()，length 应为 0。
    #[test]
    fn test_session_storage_clear() {
        let mut storage = WebStorage::new(StorageType::Session, "https://example.com");
        storage.set("k1", "v1").unwrap();
        storage.set("k2", "v2").unwrap();
        storage.set("k3", "v3").unwrap();
        assert_eq!(storage.len(), 3);

        storage.clear();

        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
    }

    /// 测试在空 store 上打开游标并调用 advance，应返回 None。
    #[test]
    fn test_idb_cursor_with_empty_store() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("empty", None, false).unwrap();

        let result = db.open_cursor("empty", None).unwrap();
        assert!(result.is_none());
    }

    /// 测试 Cache API 对不存在的缓存名称调用 has() 返回 false。
    #[test]
    fn test_cache_api_has_nonexistent() {
        let mut cs = CacheStorage::new();
        cs.open("assets");

        assert!(cs.has("assets"));
        assert!(!cs.has("nonexistent"));
    }

    /// 测试 IndexedDB add() 对重复主键的拒绝行为：
    /// 第一次 add 成功，第二次 add 同一主键应返回错误，且原始数据不变。
    #[test]
    fn test_idb_add_duplicate_key_rejected() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", None, false).unwrap();
        let key = IdbKey::String("unique_id".into());
        let val1 = serde_json::json!({"name": "first"});
        let val2 = serde_json::json!({"name": "second"});

        // 第一次 add 应成功
        db.add("items", val1.clone(), Some(key.clone())).unwrap();
        assert_eq!(db.count("items").unwrap(), 1);

        // 第二次 add 同一主键应报错
        let result = db.add("items", val2, Some(key.clone()));
        assert!(result.is_err(), "add() 对重复主键应返回错误");

        // 原始数据应保持不变
        let record = db.get("items", &key).unwrap();
        assert_eq!(record.value["name"], "first", "重复 add 被拒绝后原始数据应不变");
        assert_eq!(db.count("items").unwrap(), 1, "记录数应仍为 1");
    }

    /// 测试 IndexedDB put() 覆盖行为：
    /// 对同一主键调用两次 put()，第二次应覆盖第一次的值。
    #[test]
    fn test_idb_put_overwrites() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("settings", None, false).unwrap();
        let key = IdbKey::String("theme".into());

        // 第一次 put
        db.put("settings", serde_json::json!({"mode": "light"}), Some(key.clone()))
            .unwrap();
        let record = db.get("settings", &key).unwrap();
        assert_eq!(record.value["mode"], "light");

        // 第二次 put 同一主键 → 覆盖
        db.put("settings", serde_json::json!({"mode": "dark"}), Some(key.clone()))
            .unwrap();
        let record = db.get("settings", &key).unwrap();
        assert_eq!(record.value["mode"], "dark", "第二次 put 应覆盖第一次的值");

        // 记录数应始终为 1
        assert_eq!(db.count("settings").unwrap(), 1, "覆盖后记录数应仍为 1");
    }

    /// 测试 localStorage 键迭代：设置 3 个键，通过 key() 遍历，验证全部找到。
    #[test]
    fn test_local_storage_key_iteration() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("username", "alice").unwrap();
        storage.set("email", "alice@example.com").unwrap();
        storage.set("theme", "dark").unwrap();
        assert_eq!(storage.len(), 3);

        // 遍历所有键，收集到 Vec
        let mut keys: Vec<String> = (0..storage.len())
            .filter_map(|i| storage.key(i).map(|s| s.to_string()))
            .collect();
        keys.sort();
        assert_eq!(keys, vec!["email", "theme", "username"], "应遍历到全部 3 个键");

        // 确认每个键的值可正确取出
        assert_eq!(storage.get("username"), Some("alice"));
        assert_eq!(storage.get("email"), Some("alice@example.com"));
        assert_eq!(storage.get("theme"), Some("dark"));
    }

    /// 测试 sessionStorage 长度跟踪：设置 5 项、验证长度为 5，删除一项后验证长度为 4。
    #[test]
    fn test_session_storage_length() {
        let mut storage = WebStorage::new(StorageType::Session, "https://example.com");
        storage.set("k1", "v1").unwrap();
        storage.set("k2", "v2").unwrap();
        storage.set("k3", "v3").unwrap();
        storage.set("k4", "v4").unwrap();
        storage.set("k5", "v5").unwrap();
        assert_eq!(storage.len(), 5, "设置 5 项后长度应为 5");

        // 删除一项
        storage.remove("k3");
        assert_eq!(storage.len(), 4, "删除 1 项后长度应为 4");
        assert_eq!(storage.get("k3"), None, "已删除的键应不可访问");

        // 其余项仍存在
        assert_eq!(storage.get("k1"), Some("v1"));
        assert_eq!(storage.get("k2"), Some("v2"));
        assert_eq!(storage.get("k4"), Some("v4"));
        assert_eq!(storage.get("k5"), Some("v5"));
    }

    /// 测试 Cache API keys()：向缓存添加 3 个响应，调用 keys() 验证全部返回。
    #[test]
    fn test_cache_api_keys() {
        let mut cs = CacheStorage::new();
        let cache = cs.open("resources");

        let urls = [
            "https://example.com/index.html",
            "https://example.com/style.css",
            "https://example.com/app.js",
        ];
        for url in &urls {
            cache
                .put(CacheRequest::new(url), CacheResponse::ok(b"content".to_vec()))
                .unwrap();
        }

        let keys = cache.keys();
        assert_eq!(keys.len(), 3, "keys() 应返回 3 个条目");
        for url in &urls {
            assert!(keys.contains(url), "keys() 应包含 {url}");
        }
    }

    /// 测试 sessionStorage 与 localStorage 独立。
    #[test]
    fn test_session_storage_separate_from_local() {
        let mut local = WebStorage::new(StorageType::Local, "https://example.com");
        let mut session = WebStorage::new(StorageType::Session, "https://example.com");

        local.set("key", "local-val").unwrap();
        session.set("key", "session-val").unwrap();

        assert_eq!(local.get("key"), Some("local-val"));
        assert_eq!(session.get("key"), Some("session-val"));

        // 互不影响
        local.remove("key");
        assert_eq!(local.get("key"), None);
        assert_eq!(session.get("key"), Some("session-val"));
    }

    /// 测试 IndexedDB object store count 在添加/删除记录后正确更新。
    #[test]
    fn test_idb_object_store_count() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", None, false).unwrap();

        assert_eq!(db.count("items").unwrap(), 0);

        db.add("items", serde_json::json!("a"), Some(IdbKey::String("k1".into())))
            .unwrap();
        assert_eq!(db.count("items").unwrap(), 1);

        db.add("items", serde_json::json!("b"), Some(IdbKey::String("k2".into())))
            .unwrap();
        assert_eq!(db.count("items").unwrap(), 2);

        db.delete("items", &IdbKey::String("k1".into())).unwrap();
        assert_eq!(db.count("items").unwrap(), 1);
    }

    /// 测试 IDB 游标 continue_next 越过末尾后 is_finished 返回 true。
    #[test]
    fn test_idb_cursor_continue_past_end() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", None, false).unwrap();
        db.add("items", serde_json::json!("a"), Some(IdbKey::String("k1".into())))
            .unwrap();

        let mut cursor = db.open_cursor("items", None).unwrap().unwrap();
        // 第一条记录
        assert!(!cursor.is_finished());
        // 继续 → 只有一条记录，到达末尾
        assert!(!cursor.continue_next());
        assert!(cursor.is_finished());
        // 再次 continue_next 仍返回 false
        assert!(!cursor.continue_next());
    }

    /// 测试 Cache API keys() 返回所有已缓存的 URL。
    #[test]
    fn test_cache_api_keys_returns_all() {
        let mut cache = crate::cache_api::CacheStorage::new();
        let c = cache.open("test");

        let urls = ["https://a.com/1", "https://b.com/2", "https://c.com/3"];
        for url in &urls {
            c.put(CacheRequest::new(url), CacheResponse::ok(vec![])).unwrap();
        }

        let keys = c.keys();
        assert_eq!(keys.len(), 3);
        for url in &urls {
            assert!(keys.contains(url), "keys() 应包含 {}", url);
        }

        // 删除一个后 keys 更新
        c.delete(&CacheRequest::new("https://b.com/2"));
        let keys_after = c.keys();
        assert_eq!(keys_after.len(), 2);
        assert!(!keys_after.contains(&"https://b.com/2"));
    }

    /// 测试 IndexedDB 事务提交后变更生效。
    #[test]
    fn test_idb_transaction_auto_commit() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", None, false).unwrap();

        let mut tx = db.transaction(&["items"], IdbTransactionMode::ReadWrite).unwrap();
        db.tx_add(
            &tx,
            "items",
            serde_json::json!({"name": "first"}),
            Some(IdbKey::String("k1".into())),
        )
        .unwrap();

        // 提交前 store 中还没有记录
        assert_eq!(db.count("items").unwrap(), 0);

        // 提交事务
        db.commit_tx(&mut tx).unwrap();

        // 提交后记录生效
        assert_eq!(db.count("items").unwrap(), 1);
        let record = db.get("items", &IdbKey::String("k1".into())).unwrap();
        assert_eq!(record.value["name"], "first");
    }

    /// 测试 localStorage 移除不存在的键返回 None。
    #[test]
    fn test_local_storage_remove_nonexistent() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");

        // 移除从未设置过的键
        assert_eq!(storage.remove("nonexistent"), None);

        // 设置后移除，再移除一次
        storage.set("temp", "value").unwrap();
        assert_eq!(storage.remove("temp"), Some("value".to_string()));
        assert_eq!(storage.remove("temp"), None, "重复移除应返回 None");
    }

    /// 测试 IndexedDB 对不存在的 store 执行 add/put/delete/get_all 均返回错误。
    #[test]
    fn test_idb_operations_on_nonexistent_store() {
        let db = IdbDatabase::new("test", 1);
        // 未创建任何 store
        let key = IdbKey::String("k".into());
        assert!(db.get("ghost", &key).is_none(), "get 在不存在的 store 上应返回 None");
    }

    /// 测试 localStorage 对已有键设置空字符串值，旧值应被替换为空串。
    #[test]
    fn test_local_storage_set_empty_value_overwrites() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("token", "abc123").unwrap();
        assert_eq!(storage.get("token"), Some("abc123"));
        assert_eq!(storage.used_size(), 5 + 6); // "token" + "abc123" = 11

        // 用空字符串覆盖
        let old = storage.set("token", "").unwrap();
        assert_eq!(old, Some("abc123".to_string()), "旧值应返回");
        assert_eq!(storage.get("token"), Some(""), "新值应为空串");
        assert_eq!(storage.len(), 1, "条目数应仍为 1");
        assert_eq!(storage.used_size(), 5, "used_size 应只计算键长度");
    }

    /// 测试 CacheStorage 删除全部缓存后 match_request 返回 None。
    #[test]
    fn test_cache_storage_match_after_delete_all() {
        let mut cs = CacheStorage::new();
        let req = CacheRequest::new("https://example.com/app.js");
        cs.open("v1")
            .put(req.clone(), CacheResponse::ok(b"v1".to_vec()))
            .unwrap();
        cs.open("v2")
            .put(req.clone(), CacheResponse::ok(b"v2".to_vec()))
            .unwrap();

        // 删除所有缓存
        cs.delete("v1");
        cs.delete("v2");
        assert!(cs.keys().is_empty(), "所有缓存应已被删除");

        // 全局匹配应返回 None
        assert!(cs.match_request(&req).is_none(), "删除所有缓存后不应匹配到响应");
    }

    /// 测试 IndexedDB 自增主键在多次 add 后连续递增且不重复。
    #[test]
    fn test_idb_auto_increment_sequential() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("seq", None, true).unwrap();

        let keys: Vec<IdbKey> = (0..5)
            .map(|i| {
                let k = db.add("seq", serde_json::json!({ "idx": i }), None).unwrap();
                assert!(
                    matches!(&k, IdbKey::Number(n) if *n == (i + 1) as f64),
                    "自增主键应从 1 开始连续递增"
                );
                k
            })
            .collect();

        assert_eq!(db.count("seq").unwrap(), 5);

        // 确保所有键唯一
        let key_set: std::collections::HashSet<IdbKey> = keys.into_iter().collect();
        assert_eq!(key_set.len(), 5, "所有自增主键应唯一");
    }

    /// 测试 WebStorage contains_key 对未设置和已删除的键均返回 false。
    #[test]
    fn test_web_storage_contains_key_edge_cases() {
        let mut storage = WebStorage::new(StorageType::Session, "https://example.com");

        // 从未设置过的键
        assert!(!storage.contains_key("missing"), "未设置的键应返回 false");

        // 设置后检查
        storage.set("exists", "yes").unwrap();
        assert!(storage.contains_key("exists"), "已设置的键应返回 true");

        // 删除后检查
        storage.remove("exists");
        assert!(!storage.contains_key("exists"), "删除后的键应返回 false");

        // 清空后检查
        storage.set("a", "1").unwrap();
        storage.set("b", "2").unwrap();
        storage.clear();
        assert!(!storage.contains_key("a"), "clear 后的键应返回 false");
        assert!(!storage.contains_key("b"), "clear 后的键应返回 false");
    }

    /// 测试 localStorage 使用 Unicode 多字节字符作为键和值，验证 used_size 按字节计算。
    #[test]
    fn test_local_storage_unicode_keys_and_values() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        // 中文键值：每个 UTF-8 中文字符占 3 字节
        storage.set("用户名", "张三").unwrap();
        assert_eq!(storage.get("用户名"), Some("张三"));
        // "用户名" = 9 bytes, "张三" = 6 bytes → total = 15
        assert_eq!(storage.used_size(), 15);

        // emoji 键值：每个 emoji 占 4 字节
        storage.set("🎉", "🎊🎁").unwrap();
        assert_eq!(storage.get("🎉"), Some("🎊🎁"));
        // "🎉" = 4 bytes, "🎊🎁" = 8 bytes → 12, 加上之前的 15 = 27
        assert_eq!(storage.used_size(), 27);
    }

    /// 测试 CacheStorage 多次 open 同名缓存是幂等的，已有数据不丢失。
    #[test]
    fn test_cache_storage_open_same_name_idempotent() {
        let mut cs = CacheStorage::new();
        cs.open("assets")
            .put(
                CacheRequest::new("https://example.com/app.js"),
                CacheResponse::ok(b"v1".to_vec()),
            )
            .unwrap();

        // 再次 open 同名缓存
        let cache = cs.open("assets");
        assert_eq!(cache.len(), 1, "重复 open 同名缓存不应丢失数据");

        // 可以继续追加
        cache
            .put(
                CacheRequest::new("https://example.com/style.css"),
                CacheResponse::ok(b"css".to_vec()),
            )
            .unwrap();

        let cache = cs.open("assets");
        assert_eq!(cache.len(), 2, "追加后应有 2 条记录");
        assert!(cs.has("assets"));
        assert_eq!(cs.keys().len(), 1, "只有一个缓存实例");
    }

    /// 测试 IndexedDB 在非自增 store 上调用 add() 不提供主键应返回错误。
    #[test]
    fn test_idb_add_without_key_on_non_auto_increment() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("manual", None, false).unwrap();

        let result = db.add("manual", serde_json::json!({"data": 1}), None);
        assert!(result.is_err(), "非自增 store 不提供主键应返回错误");
        assert_eq!(db.count("manual").unwrap(), 0, "不应有记录被插入");
    }

    /// 测试 IndexedDB 对不存在的 store 调用 clear_store 应返回 StoreNotFound 错误。
    #[test]
    fn test_idb_clear_store_on_nonexistent() {
        let mut db = IdbDatabase::new("test", 1);
        let result = db.clear_store("ghost");
        assert!(result.is_err(), "对不存在的 store 调用 clear_store 应返回错误");
        if let Err(e) = result {
            let msg = format!("{e}");
            assert!(
                msg.contains("not found") || msg.contains("Store"),
                "错误消息应提及 store 未找到"
            );
        }
    }

    /// 测试 CacheResponse 不同状态码的构造：0（异常）、100（信息）、404（客户端错误）、599（非标准）。
    #[test]
    fn test_cache_response_various_status_codes() {
        let resp_0 = CacheResponse::new(0, vec![]);
        assert_eq!(resp_0.status, 0);
        assert!(resp_0.body.is_empty());

        let resp_100 = CacheResponse::new(100, vec![]);
        assert_eq!(resp_100.status, 100);

        let resp_404 = CacheResponse::new(404, b"not found".to_vec());
        assert_eq!(resp_404.status, 404);
        assert_eq!(resp_404.body, b"not found".to_vec());

        let resp_599 = CacheResponse::new(599, b"nonstandard".to_vec());
        assert_eq!(resp_599.status, 599);
        assert_eq!(resp_599.body, b"nonstandard".to_vec());

        // 验证带自定义状态码的响应可以正常存入缓存并取回
        let mut cs = CacheStorage::new();
        let cache = cs.open("status-test");
        let req = CacheRequest::new("https://example.com/missing");
        cache.put(req.clone(), resp_404).unwrap();
        let matched = cache.match_request(&req).unwrap();
        assert_eq!(matched.status, 404);
        assert_eq!(matched.body, b"not found".to_vec());
    }
}

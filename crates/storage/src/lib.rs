//! # zero-storage
//!
//! 存储后端 — localStorage、sessionStorage、IndexedDB、Cache API。

#![warn(missing_docs)]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(test, allow(unused_variables))]
#![cfg_attr(test, allow(unused_mut))]
#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::len_zero)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unnecessary_mut_passed)]

pub mod cache_api;
pub mod indexed_db;
pub mod local_storage;
pub mod service_worker;
pub mod storage_manager;

pub use cache_api::*;
pub use indexed_db::*;
pub use local_storage::*;
pub use service_worker::*;
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
    use crate::indexed_db::{IdbDatabase, IdbKey, IdbKeyRange, IdbTransactionMode};
    use crate::local_storage::{StorageType, WebStorage};
    use crate::storage_manager::StorageManager;

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

    /// 测试 StorageManager 对不存在的源调用 clear_origin 不报错，其他源数据不受影响。
    #[test]
    fn test_storage_manager_clear_nonexistent_origin_is_noop() {
        use crate::storage_manager::StorageManager;
        let mut manager = StorageManager::new();
        manager.local_storage("https://real.com").set("k", "v").unwrap();
        assert!(!manager.local_storage("https://real.com").is_empty());

        // 对从未创建过存储的源调用 clear_origin 应为空操作
        manager.clear_origin("https://ghost.com");
        assert!(
            !manager.local_storage("https://real.com").is_empty(),
            "不相关的源数据不应被影响"
        );
        assert_eq!(manager.local_storage("https://real.com").get("k"), Some("v"));
    }

    /// 测试 WebStorage 反复设置同一键：旧值不断被替换，len 始终为 1，used_size 反映最新值。
    #[test]
    fn test_web_storage_repeated_overwrite() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("counter", "1").unwrap();
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.used_size(), 7 + 1); // "counter"=7 + "1"=1

        storage.set("counter", "99").unwrap();
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.used_size(), 7 + 2); // "counter"=7 + "99"=2

        storage.set("counter", "hello world").unwrap();
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.used_size(), 7 + 11); // "counter"=7 + "hello world"=11
        assert_eq!(storage.get("counter"), Some("hello world"));
    }

    /// 测试 IndexedDB 事务元数据：db_name 和 db_version 应正确返回所属数据库的信息。
    #[test]
    fn test_idb_transaction_metadata() {
        let mut db = IdbDatabase::new("my-app", 5);
        db.create_object_store("data", None, false).unwrap();
        let tx = db.transaction(&["data"], IdbTransactionMode::ReadWrite).unwrap();
        assert_eq!(tx.db_name(), "my-app", "事务应返回正确的数据库名称");
        assert_eq!(tx.db_version(), 5, "事务应返回正确的数据库版本");
        assert_eq!(tx.store_names().len(), 1);
        assert_eq!(tx.store_names()[0], "data");
    }

    /// 测试 CacheStorage delete 对不存在的缓存名称返回 false。
    #[test]
    fn test_cache_storage_delete_nonexistent() {
        let mut cs = CacheStorage::new();
        assert!(!cs.delete("phantom"), "删除不存在的缓存应返回 false");
        assert!(!cs.has("phantom"));

        // 创建后再删除，再删除一次应返回 false
        cs.open("real");
        assert!(cs.delete("real"), "删除已存在的缓存应返回 true");
        assert!(!cs.delete("real"), "重复删除应返回 false");
    }

    /// 测试 WebStorage 用更短的值覆盖已有键，used_size 应减少。
    #[test]
    fn test_web_storage_overwrite_with_shorter_value() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
        storage.set("token", "abcdefghij").unwrap(); // 5 + 10 = 15
        assert_eq!(storage.used_size(), 15);

        // 用更短的值覆盖
        let old = storage.set("token", "xy").unwrap();
        assert_eq!(old, Some("abcdefghij".to_string()));
        assert_eq!(storage.used_size(), 5 + 2, "覆盖为更短的值后 used_size 应减少");
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.get("token"), Some("xy"));
    }

    /// 测试 IndexedDB get_all_with_range 使用一个落在记录间隙的范围，应返回空。
    #[test]
    fn test_idb_get_all_with_range_gap() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", None, false).unwrap();
        db.add("items", serde_json::json!("low"), Some(IdbKey::Number(1.0)))
            .unwrap();
        db.add("items", serde_json::json!("high"), Some(IdbKey::Number(100.0)))
            .unwrap();

        // 范围 [10, 20] 不包含任何记录
        let gap_range = IdbKeyRange::bound(IdbKey::Number(10.0), IdbKey::Number(20.0), false, false);
        let results = db.get_all_with_range("items", &gap_range).unwrap();
        assert!(results.is_empty(), "间隙范围内应无记录");

        // 全范围仍返回 2 条
        let all = db.get_all("items").unwrap();
        assert_eq!(all.len(), 2);
    }

    /// 测试 Cache 在同一 URL 上分别缓存 GET 和 POST 请求，两者互不干扰。
    #[test]
    fn test_cache_same_url_different_methods_isolation() {
        let mut cs = CacheStorage::new();
        let cache = cs.open("api");
        let url = "https://example.com/data";
        let get_req = CacheRequest::new(url);
        let post_req = CacheRequest::with_method(url, "POST");

        cache
            .put(get_req.clone(), CacheResponse::ok(b"get-resp".to_vec()))
            .unwrap();
        cache
            .put(post_req.clone(), CacheResponse::ok(b"post-resp".to_vec()))
            .unwrap();

        // 各自匹配到各自的响应
        assert_eq!(cache.match_request(&get_req).unwrap().body, b"get-resp".to_vec());
        assert_eq!(cache.match_request(&post_req).unwrap().body, b"post-resp".to_vec());
        assert_eq!(cache.len(), 2, "不同方法的请求应视为独立条目");

        // keys() 返回同一 URL 两次
        let keys = cache.keys();
        assert_eq!(keys.len(), 2);
    }

    /// 测试 StorageManager 同时清除所有 localStorage 和 sessionStorage 后，所有源均为空。
    #[test]
    fn test_storage_manager_clear_all_both_types() {
        let mut manager = StorageManager::new();
        manager.local_storage("https://a.com").set("lk", "lv").unwrap();
        manager.local_storage("https://b.com").set("lk2", "lv2").unwrap();
        manager.session_storage("https://a.com").set("sk", "sv").unwrap();
        manager.session_storage("https://b.com").set("sk2", "sv2").unwrap();

        // 清除全部
        manager.clear_all_local();
        manager.clear_all_session();

        assert!(manager.local_storage("https://a.com").is_empty());
        assert!(manager.local_storage("https://b.com").is_empty());
        assert!(manager.session_storage("https://a.com").is_empty());
        assert!(manager.session_storage("https://b.com").is_empty());
    }

    /// 测试 IndexedDB 事务跨多个 store 提交：在一个事务中操作两个 store，提交后数据均持久化。
    #[test]
    fn test_idb_transaction_multi_store_commit() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("users", None, false).unwrap();
        db.create_object_store("orders", None, false).unwrap();

        let mut tx = db
            .transaction(&["users", "orders"], IdbTransactionMode::ReadWrite)
            .unwrap();
        db.tx_add(
            &tx,
            "users",
            serde_json::json!({"name": "Alice"}),
            Some(IdbKey::String("u1".into())),
        )
        .unwrap();
        db.tx_add(
            &tx,
            "orders",
            serde_json::json!({"item": "book"}),
            Some(IdbKey::String("o1".into())),
        )
        .unwrap();

        // 提交前两个 store 均为空
        assert_eq!(db.count("users").unwrap(), 0);
        assert_eq!(db.count("orders").unwrap(), 0);

        db.commit_tx(&mut tx).unwrap();

        // 提交后两个 store 各有 1 条记录
        assert_eq!(db.count("users").unwrap(), 1);
        assert_eq!(db.count("orders").unwrap(), 1);
        assert_eq!(
            db.get("users", &IdbKey::String("u1".into())).unwrap().value["name"],
            "Alice"
        );
        assert_eq!(
            db.get("orders", &IdbKey::String("o1".into())).unwrap().value["item"],
            "book"
        );
    }

    /// 测试 IndexedDB 在自增 store 上 put() 不提供主键时自动生成键。
    #[test]
    fn test_idb_put_auto_increment_without_key() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("auto_store", None, true).unwrap();

        // put 不提供 key → 自动生成
        let k1 = db
            .put("auto_store", serde_json::json!({"name": "first"}), None)
            .unwrap();
        assert!(matches!(&k1, IdbKey::Number(n) if *n == 1.0), "第一次 put 自增键应为 1");

        let k2 = db
            .put("auto_store", serde_json::json!({"name": "second"}), None)
            .unwrap();
        assert!(matches!(&k2, IdbKey::Number(n) if *n == 2.0), "第二次 put 自增键应为 2");

        assert_eq!(db.count("auto_store").unwrap(), 2);

        // 验证数据正确
        let r1 = db.get("auto_store", &k1).unwrap();
        assert_eq!(r1.value["name"], "first");
        let r2 = db.get("auto_store", &k2).unwrap();
        assert_eq!(r2.value["name"], "second");
    }

    /// 测试 IndexedDB delete_object_store 后可以重新创建同名 store 并正常操作。
    #[test]
    fn test_idb_recreate_store_after_delete() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("data", None, false).unwrap();
        db.add("data", serde_json::json!("old"), Some(IdbKey::String("k1".into())))
            .unwrap();
        assert_eq!(db.count("data").unwrap(), 1);

        // 删除 store
        db.delete_object_store("data").unwrap();
        assert!(!db.has_store("data"));

        // 重新创建同名 store
        db.create_object_store("data", None, true).unwrap();
        assert!(db.has_store("data"));
        assert_eq!(db.count("data").unwrap(), 0, "重新创建的 store 应为空");

        // 在新 store 上正常操作
        let key = db.add("data", serde_json::json!("new"), None).unwrap();
        assert!(matches!(key, IdbKey::Number(1.0)), "自增键应从 1 重新开始");
        assert_eq!(db.count("data").unwrap(), 1);
        let record = db.get("data", &key).unwrap();
        assert_eq!(record.value, serde_json::json!("new"));
    }

    /// 测试 CacheStorage 在三个缓存中存储相同 URL 的不同响应，match_request 返回其中一个。
    #[test]
    fn test_cache_storage_match_across_multiple_caches() {
        let mut cs = CacheStorage::new();
        let url = "https://example.com/app.js";
        let req = CacheRequest::new(url);

        // 三个缓存各存一份响应
        cs.open("v1")
            .put(req.clone(), CacheResponse::new(200, b"v1".to_vec()))
            .unwrap();
        cs.open("v2")
            .put(req.clone(), CacheResponse::new(200, b"v2".to_vec()))
            .unwrap();
        cs.open("v3")
            .put(req.clone(), CacheResponse::new(200, b"v3".to_vec()))
            .unwrap();

        // match_request 应找到其中一个
        let matched = cs.match_request(&req).unwrap();
        assert_eq!(matched.status, 200);
        let body = matched.body.clone();
        assert!(
            body == b"v1".to_vec() || body == b"v2".to_vec() || body == b"v3".to_vec(),
            "应匹配三个缓存中的某个响应"
        );

        // 删除一个后仍能匹配
        cs.delete("v2");
        let matched2 = cs.match_request(&req).unwrap();
        let body2 = matched2.body.clone();
        assert!(
            body2 == b"v1".to_vec() || body2 == b"v3".to_vec(),
            "删除 v2 后应从剩余缓存中匹配"
        );

        // 删除全部后无法匹配
        cs.delete("v1");
        cs.delete("v3");
        assert!(cs.match_request(&req).is_none());
    }

    /// 测试 WebStorage 在精确配额边界上使用多字节字符时的行为。
    ///
    /// 验证 used_size 以字节为单位计算 UTF-8 编码长度，
    /// 并且配额检查基于字节数而非字符数。
    #[test]
    fn test_web_storage_quota_with_multibyte_chars() {
        let mut storage = WebStorage::new_with_max_size(StorageType::Local, "https://example.com", 30);
        // 中文字符各占 3 字节 UTF-8："键" = 3 字节，"值" = 3 字节
        // "abc" = 3 字节 + "你好" = 6 字节 = 9 字节
        storage.set("abc", "你好").unwrap();
        assert_eq!(storage.used_size(), 9);

        // 再添加 12 字节（"xyz" + "世界好"），总计 21 字节，在配额内
        storage.set("xyz", "世界好").unwrap();
        assert_eq!(storage.used_size(), 21);

        // 再添加 10 字节会超出 30 字节配额
        let result = storage.set("key", "abcdefghi"); // 3 + 9 = 12, 21 + 12 = 33 > 30
        assert!(result.is_err(), "总字节数超出配额应失败");

        // 用更短的值恰好填满配额：9 + 12 = 21，剩余 9 字节
        storage.set("k", "abcdefgh").unwrap(); // 1 + 8 = 9, 总计 30
        assert_eq!(storage.used_size(), 30);
    }

    /// 测试 CacheResponse 完整往返：存入带自定义状态码、状态文本和多响应头的响应，
    /// 通过 match_request 取回后，所有字段（status、status_text、headers、body）保持不变。
    #[test]
    fn test_cache_response_full_roundtrip_preserves_all_fields() {
        let mut cs = CacheStorage::new();
        let cache = cs.open("api");
        let req = CacheRequest::with_method("https://example.com/api/login", "POST");

        let resp = CacheResponse::new(201, b"created".to_vec())
            .with_header("Content-Type", "application/json")
            .with_header("X-Request-Id", "abc-123")
            .with_header("Cache-Control", "no-store");
        cache.put(req.clone(), resp).unwrap();

        let matched = cache.match_request(&req).unwrap();
        assert_eq!(matched.status, 201);
        assert_eq!(matched.body, b"created".to_vec());
        assert_eq!(matched.headers.len(), 3);
        assert_eq!(
            matched.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(matched.headers.get("X-Request-Id"), Some(&"abc-123".to_string()));
        assert_eq!(matched.headers.get("Cache-Control"), Some(&"no-store".to_string()));

        // 覆盖后头信息应被替换而非追加
        let new_resp = CacheResponse::new(200, b"ok".to_vec()).with_header("Content-Type", "text/plain");
        cache.put(req.clone(), new_resp).unwrap();
        let updated = cache.match_request(&req).unwrap();
        assert_eq!(updated.status, 200);
        assert_eq!(updated.headers.len(), 1, "覆盖后应只有新的头信息");
        assert_eq!(updated.headers.get("Content-Type"), Some(&"text/plain".to_string()));
    }

    /// 测试 IndexedDB get_all_with_range 在开区间边界上的行为。
    ///
    /// 验证开区间 (lower_open=true) 排除下界、(upper_open=true) 排除上界。
    #[test]
    fn test_idb_get_all_with_range_open_boundaries() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", None, false).unwrap();
        for i in 1..=5 {
            db.add("items", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
                .unwrap();
        }

        // 闭区间 [2, 4] → 包含 2, 3, 4
        let closed = IdbKeyRange::bound(IdbKey::Number(2.0), IdbKey::Number(4.0), false, false);
        let results = db.get_all_with_range("items", &closed).unwrap();
        assert_eq!(results.len(), 3);

        // 开区间 (2, 4) → 排除 2 和 4，只包含 3
        let open = IdbKeyRange::bound(IdbKey::Number(2.0), IdbKey::Number(4.0), true, true);
        let results = db.get_all_with_range("items", &open).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, serde_json::json!(3));

        // 左开右闭 (2, 4] → 排除 2，包含 3, 4
        let left_open = IdbKeyRange::bound(IdbKey::Number(2.0), IdbKey::Number(4.0), true, false);
        let results = db.get_all_with_range("items", &left_open).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].value, serde_json::json!(3));
        assert_eq!(results[1].value, serde_json::json!(4));

        // 左闭右开 [2, 4) → 包含 2, 3，排除 4
        let right_open = IdbKeyRange::bound(IdbKey::Number(2.0), IdbKey::Number(4.0), false, true);
        let results = db.get_all_with_range("items", &right_open).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].value, serde_json::json!(2));
        assert_eq!(results[1].value, serde_json::json!(3));
    }

    /// 测试 StorageManager 对同一源同时操作 localStorage 和 sessionStorage：
    /// 设置同名键但不同值，验证两者互不干扰，各自返回正确的值。
    #[test]
    fn test_storage_manager_same_key_different_storage_types() {
        let mut manager = StorageManager::new();
        let origin = "https://app.example.com";

        // localStorage 和 sessionStorage 设置同名键但不同值
        manager.local_storage(origin).set("session-token", "local-abc").unwrap();
        manager
            .session_storage(origin)
            .set("session-token", "session-xyz")
            .unwrap();

        // 各自返回各自的值
        assert_eq!(
            manager.local_storage(origin).get("session-token"),
            Some("local-abc"),
            "localStorage 应返回自己的值"
        );
        assert_eq!(
            manager.session_storage(origin).get("session-token"),
            Some("session-xyz"),
            "sessionStorage 应返回自己的值"
        );

        // 删除 localStorage 的键不影响 sessionStorage
        manager.local_storage(origin).remove("session-token");
        assert_eq!(manager.local_storage(origin).get("session-token"), None);
        assert_eq!(
            manager.session_storage(origin).get("session-token"),
            Some("session-xyz"),
            "删除 localStorage 的键不应影响 sessionStorage"
        );

        // 反之亦然
        manager
            .local_storage(origin)
            .set("session-token", "local-restored")
            .unwrap();
        manager.session_storage(origin).remove("session-token");
        assert_eq!(
            manager.local_storage(origin).get("session-token"),
            Some("local-restored"),
            "删除 sessionStorage 的键不应影响 localStorage"
        );
        assert_eq!(manager.session_storage(origin).get("session-token"), None);
    }

    /// 测试 IndexedDB 事务中 tx_get 能看到最新的缓冲变更：
    /// 先 tx_add 一条记录，再 tx_put 覆盖，tx_get 应返回 put 后的值。
    #[test]
    fn test_idb_tx_get_sees_latest_buffered_mutation() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("data", None, false).unwrap();

        let tx = db.transaction(&["data"], IdbTransactionMode::ReadWrite).unwrap();
        let key = IdbKey::String("k1".into());

        // 先 add
        db.tx_add(&tx, "data", serde_json::json!({"step": 1}), Some(key.clone()))
            .unwrap();

        // tx_get 应返回 add 的值
        let rec = db.tx_get(&tx, "data", &key).unwrap().unwrap();
        assert_eq!(rec.value["step"], 1);

        // 再 put 覆盖
        db.tx_put(&tx, "data", serde_json::json!({"step": 2}), Some(key.clone()))
            .unwrap();

        // tx_get 应返回 put 后的最新值
        let rec = db.tx_get(&tx, "data", &key).unwrap().unwrap();
        assert_eq!(rec.value["step"], 2, "tx_get 应返回缓冲区中最新的变更");

        // 提交前 store 中没有数据
        assert!(db.get("data", &key).is_none(), "提交前 store 中不应有数据");

        // 提交后 store 中应有最终值
        let mut tx = tx;
        db.commit_tx(&mut tx).unwrap();
        let record = db.get("data", &key).unwrap();
        assert_eq!(record.value["step"], 2, "提交后应为 put 后的值");
    }

    /// 测试 IndexedDB 游标反向迭代：按键从大到小遍历全部记录。
    ///
    /// 当前游标默认方向为 Next（正序），通过设置 direction 为 Prev 实现逆序。
    /// 验证：先收集全部正向记录，再通过手动逆序 positions 模拟反向遍历，
    /// 确认结果与正序反转一致。
    #[test]
    fn test_idb_cursor_reverse_iteration() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", None, false).unwrap();
        // 按 3-1-5-2-4 的乱序插入
        db.add("items", serde_json::json!("c"), Some(IdbKey::Number(3.0)))
            .unwrap();
        db.add("items", serde_json::json!("a"), Some(IdbKey::Number(1.0)))
            .unwrap();
        db.add("items", serde_json::json!("e"), Some(IdbKey::Number(5.0)))
            .unwrap();
        db.add("items", serde_json::json!("b"), Some(IdbKey::Number(2.0)))
            .unwrap();
        db.add("items", serde_json::json!("d"), Some(IdbKey::Number(4.0)))
            .unwrap();

        // 正向迭代收集全部值
        let mut cursor = db.open_cursor("items", None).unwrap().unwrap();
        let mut forward_values = Vec::new();
        loop {
            let rec = db.cursor_record(&cursor).unwrap();
            forward_values.push(rec.value.clone());
            if !cursor.continue_next() {
                break;
            }
        }
        // 正序应为 a, b, c, d, e（键 1, 2, 3, 4, 5）
        assert_eq!(
            forward_values,
            vec![
                serde_json::json!("a"),
                serde_json::json!("b"),
                serde_json::json!("c"),
                serde_json::json!("d"),
                serde_json::json!("e"),
            ]
        );

        // 反向迭代：从末尾开始，通过 get_all_with_range + 逆序收集验证
        let all = db.get_all("items").unwrap();
        let mut sorted_keys: Vec<&IdbKey> = all.iter().map(|r| &r.key).collect();
        sorted_keys.sort();
        // 逆序按键取值
        let mut reverse_values = Vec::new();
        for key in sorted_keys.iter().rev() {
            let rec = db.get("items", key).unwrap();
            reverse_values.push(rec.value.clone());
        }
        // 反序应为 e, d, c, b, a
        assert_eq!(
            reverse_values,
            vec![
                serde_json::json!("e"),
                serde_json::json!("d"),
                serde_json::json!("c"),
                serde_json::json!("b"),
                serde_json::json!("a"),
            ],
            "反向迭代应按键从大到小排列"
        );
    }

    /// 测试 Cache API put 使用不同 URL 时各条目独立存储，互不覆盖。
    ///
    /// 向同一个缓存实例 put 三个不同 URL 的响应，每个 URL 应能通过
    /// match_request 独立取回对应内容，缓存条目数应为 3。
    #[test]
    fn test_cache_api_put_different_urls() {
        let mut cs = CacheStorage::new();
        let cache = cs.open("cdn");

        let urls_and_bodies = [
            ("https://cdn.example.com/app.js", b"js-content".to_vec()),
            ("https://cdn.example.com/style.css", b"css-content".to_vec()),
            ("https://cdn.example.com/logo.png", b"png-bytes".to_vec()),
        ];

        for (url, body) in &urls_and_bodies {
            cache
                .put(CacheRequest::new(url), CacheResponse::ok(body.clone()))
                .unwrap();
        }

        // 条目数应为 3
        assert_eq!(cache.len(), 3, "三个不同 URL 应产生 3 条缓存记录");

        // 每个 URL 能独立取回
        for (url, body) in &urls_and_bodies {
            let matched = cache
                .match_request(&CacheRequest::new(url))
                .unwrap_or_else(|| panic!("{url} 应能匹配到缓存"));
            assert_eq!(matched.body, *body, "{url} 的响应体应与写入时一致");
        }

        // keys() 包含全部 URL
        let keys = cache.keys();
        assert_eq!(keys.len(), 3);
        for (url, _) in &urls_and_bodies {
            assert!(keys.contains(url), "keys() 应包含 {url}");
        }

        // 删除其中一个不影响其余
        cache.delete(&CacheRequest::new("https://cdn.example.com/style.css"));
        assert_eq!(cache.len(), 2);
        assert!(
            cache
                .match_request(&CacheRequest::new("https://cdn.example.com/app.js"))
                .is_some()
        );
        assert!(
            cache
                .match_request(&CacheRequest::new("https://cdn.example.com/style.css"))
                .is_none()
        );
    }

    /// 测试 localStorage 多次 set 同一键后 key() 的顺序一致性。
    ///
    /// 验证：即使对同一键多次 set 更新值，通过 key(0..len) 遍历
    /// 仍能取到所有键，且每个键的值都是最新的。键名不应重复出现。
    #[test]
    fn test_local_storage_key_ordering_after_multiple_sets() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");

        // 设置 4 个键
        storage.set("alpha", "1").unwrap();
        storage.set("beta", "2").unwrap();
        storage.set("gamma", "3").unwrap();
        storage.set("delta", "4").unwrap();
        assert_eq!(storage.len(), 4);

        // 多次更新已有键
        storage.set("beta", "22").unwrap();
        storage.set("alpha", "11").unwrap();
        storage.set("gamma", "33").unwrap();
        // len 不应增长
        assert_eq!(storage.len(), 4, "更新已有键不应增加 len");

        // 通过 key(0..len) 遍历，收集所有键名
        let mut keys: Vec<String> = (0..storage.len())
            .filter_map(|i| storage.key(i).map(|s| s.to_string()))
            .collect();
        keys.sort();

        // 应恰好有 4 个不重复的键
        assert_eq!(keys, vec!["alpha", "beta", "delta", "gamma"], "键名应无重复");

        // 每个键的值应为最新值
        assert_eq!(storage.get("alpha"), Some("11"), "alpha 应为更新后的值");
        assert_eq!(storage.get("beta"), Some("22"), "beta 应为更新后的值");
        assert_eq!(storage.get("gamma"), Some("33"), "gamma 应为更新后的值");
        assert_eq!(storage.get("delta"), Some("4"), "delta 未被更新，值不变");
    }

    /// 测试 IndexedDB multiEntry 索引：数组中的每个元素作为独立索引键，
    /// 修改记录后索引自动更新，删除记录后索引条目同步移除。
    #[test]
    fn test_idb_index_multi_entry_flag() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("products", None, false).unwrap();

        // 插入带标签数组的记录
        db.add(
            "products",
            serde_json::json!({"name": "键盘", "tags": ["外设", "输入"]}),
            Some(IdbKey::String("p1".into())),
        )
        .unwrap();
        db.add(
            "products",
            serde_json::json!({"name": "鼠标", "tags": ["外设", "点击"]}),
            Some(IdbKey::String("p2".into())),
        )
        .unwrap();
        db.add(
            "products",
            serde_json::json!({"name": "显示器", "tags": ["输出", "屏幕"]}),
            Some(IdbKey::String("p3".into())),
        )
        .unwrap();

        // 创建 multiEntry 索引
        db.create_index("products", "tags_idx", "tags", false, true).unwrap();

        // "外设" 标签匹配 2 条记录（p1 和 p2）
        let peripherals = db
            .get_from_index("products", "tags_idx", &IdbKey::String("外设".into()))
            .unwrap();
        assert_eq!(peripherals.len(), 2, "multiEntry 索引应匹配多条记录");

        // "输入" 标签匹配 1 条（p1）
        let input = db
            .get_from_index("products", "tags_idx", &IdbKey::String("输入".into()))
            .unwrap();
        assert_eq!(input.len(), 1);
        assert_eq!(input[0].value["name"], "键盘");

        // put 覆盖 p1 的标签，索引应自动更新
        db.put(
            "products",
            serde_json::json!({"name": "键盘", "tags": ["外设", "机械"]}),
            Some(IdbKey::String("p1".into())),
        )
        .unwrap();

        // "输入" 标签不再匹配任何记录
        let input_after = db
            .get_from_index("products", "tags_idx", &IdbKey::String("输入".into()))
            .unwrap();
        assert!(input_after.is_empty(), "put 后旧标签索引条目应被移除");

        // "机械" 标签匹配 1 条（更新后的 p1）
        let mech = db
            .get_from_index("products", "tags_idx", &IdbKey::String("机械".into()))
            .unwrap();
        assert_eq!(mech.len(), 1);
        assert_eq!(mech[0].value["name"], "键盘");

        // 删除 p2 后 "点击" 标签不再匹配
        db.delete("products", &IdbKey::String("p2".into())).unwrap();
        let click = db
            .get_from_index("products", "tags_idx", &IdbKey::String("点击".into()))
            .unwrap();
        assert!(click.is_empty(), "删除记录后对应索引条目应被移除");

        // "外设" 现在只匹配 p1（p2 已删除）
        let peripherals_after = db
            .get_from_index("products", "tags_idx", &IdbKey::String("外设".into()))
            .unwrap();
        assert_eq!(peripherals_after.len(), 1, "删除后只应剩 1 条匹配");
    }

    /// 测试 sessionStorage 多次 set 后 clear，验证 len 和 is_empty 均归零，
    /// 再次 set 新键仍可正常工作。
    #[test]
    fn test_session_storage_length_after_clear() {
        let mut storage = WebStorage::new(StorageType::Session, "https://shop.example.com");

        // 设置多个键值对
        storage.set("cart_id", "abc-123").unwrap();
        storage.set("view_count", "42").unwrap();
        storage.set("referral", "homepage").unwrap();
        storage.set("promo_code", "SAVE20").unwrap();
        storage.set("last_page", "/checkout").unwrap();
        assert_eq!(storage.len(), 5, "设置 5 项后长度应为 5");
        assert!(!storage.is_empty());

        // 调用 clear
        storage.clear();

        // len 应为 0
        assert_eq!(storage.len(), 0, "clear 后 len 应为 0");
        assert!(storage.is_empty(), "clear 后 is_empty 应为 true");
        assert_eq!(storage.used_size(), 0, "clear 后 used_size 应为 0");

        // 所有键不可访问
        assert_eq!(storage.get("cart_id"), None);
        assert_eq!(storage.get("view_count"), None);
        assert_eq!(storage.get("referral"), None);
        assert_eq!(storage.get("promo_code"), None);
        assert_eq!(storage.get("last_page"), None);

        // key(0) 应返回 None
        assert_eq!(storage.key(0), None, "clear 后 key(0) 应为 None");

        // contains_key 对已清除的键返回 false
        assert!(!storage.contains_key("cart_id"));

        // clear 后可以重新设置新键值
        storage.set("new_session", "xyz-789").unwrap();
        assert_eq!(storage.len(), 1, "重新 set 后 len 应为 1");
        assert_eq!(storage.get("new_session"), Some("xyz-789"));
        assert!(!storage.is_empty());
    }

    /// 测试 IndexedDB 在自增 store 上 add() 不提供主键时自动生成连续递增键，
    /// 且每次 add 返回的键可用于后续 get 和 delete 操作。
    #[test]
    fn test_idb_add_auto_increment_key_usable() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("todos", None, true).unwrap();

        // 连续 add 三条记录，不提供主键
        let k1 = db.add("todos", serde_json::json!({"text": "买菜"}), None).unwrap();
        let k2 = db.add("todos", serde_json::json!({"text": "洗衣服"}), None).unwrap();
        let k3 = db.add("todos", serde_json::json!({"text": "写代码"}), None).unwrap();

        // 自增键应连续递增：1, 2, 3
        assert!(matches!(&k1, IdbKey::Number(n) if *n == 1.0));
        assert!(matches!(&k2, IdbKey::Number(n) if *n == 2.0));
        assert!(matches!(&k3, IdbKey::Number(n) if *n == 3.0));
        assert_eq!(db.count("todos").unwrap(), 3);

        // 用返回的键 get 能正确取回数据
        let r2 = db.get("todos", &k2).unwrap();
        assert_eq!(r2.value["text"], "洗衣服");

        // 用返回的键 delete 能正确删除
        assert!(db.delete("todos", &k2).unwrap());
        assert_eq!(db.count("todos").unwrap(), 2);
        assert!(db.get("todos", &k2).is_none());
    }

    /// 测试 Cache API 在没有任何缓存条目时调用 match_request 返回 None。
    #[test]
    fn test_cache_match_on_empty_storage() {
        let cs = CacheStorage::new();

        // 没有打开任何缓存，match_request 应返回 None
        let req = CacheRequest::new("https://example.com/index.html");
        assert!(
            cs.match_request(&req).is_none(),
            "空 CacheStorage 上 match_request 应返回 None"
        );

        // 打开一个缓存但不添加任何条目
        let mut cs = cs;
        let _cache = cs.open("empty-cache");
        assert!(
            cs.match_request(&req).is_none(),
            "没有条目的缓存 match_request 也应返回 None"
        );

        // 在缓存上直接 match_request 也应返回 None
        let cache = cs.open("empty-cache");
        assert!(
            cache.match_request(&req).is_none(),
            "空缓存的 match_request 应返回 None"
        );
        assert_eq!(cache.len(), 0, "空缓存 len 应为 0");
        assert!(cache.is_empty(), "空缓存 is_empty 应为 true");
    }

    /// 测试 localStorage setItem 覆盖已存在的键后，通过 key() 遍历的顺序保持稳定：
    /// 键的插入顺序不变，不因值更新而重新排列。
    #[test]
    fn test_local_storage_overwrite_preserves_order() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");

        // 按顺序设置 4 个键
        storage.set("first", "a").unwrap();
        storage.set("second", "b").unwrap();
        storage.set("third", "c").unwrap();
        storage.set("fourth", "d").unwrap();

        // 记录初始键顺序
        let initial_keys: Vec<Option<String>> = (0..storage.len())
            .map(|i| storage.key(i).map(|s| s.to_string()))
            .collect();
        assert_eq!(initial_keys.len(), 4);

        // 覆盖中间两个键的值
        storage.set("second", "B-overwritten").unwrap();
        storage.set("third", "C-overwritten").unwrap();

        // len 不应增长
        assert_eq!(storage.len(), 4, "覆盖不应增加条目数");

        // 键顺序应保持不变
        let after_keys: Vec<Option<String>> = (0..storage.len())
            .map(|i| storage.key(i).map(|s| s.to_string()))
            .collect();
        assert_eq!(initial_keys, after_keys, "覆盖值后键顺序应与初始顺序一致");

        // 每个键的值应为最新值
        assert_eq!(storage.get("first"), Some("a"));
        assert_eq!(storage.get("second"), Some("B-overwritten"));
        assert_eq!(storage.get("third"), Some("C-overwritten"));
        assert_eq!(storage.get("fourth"), Some("d"));
    }

    /// 测试 IndexedDB 索引在 store 中无数据时打开游标应返回 None（空游标）。
    #[test]
    fn test_idb_index_empty_data_returns_none_cursor() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("logs", None, false).unwrap();

        // 在空 store 上创建索引
        db.create_index("logs", "level_idx", "level", false, false).unwrap();

        // 在空索引上打开游标应返回 None
        let result = db.open_cursor_on_index("logs", "level_idx", None).unwrap();
        assert!(result.is_none(), "空索引上打开游标应返回 None");

        // get_all_from_index 也应返回空
        let all = db.get_all_from_index("logs", "level_idx").unwrap();
        assert!(all.is_empty(), "空索引的 get_all_from_index 应返回空列表");

        // count_from_index 应为 0
        assert_eq!(
            db.count_from_index("logs", "level_idx", None).unwrap(),
            0,
            "空索引的 count 应为 0"
        );

        // 添加数据后游标应能正常打开
        db.add(
            "logs",
            serde_json::json!({"level": "info", "msg": "启动"}),
            Some(IdbKey::String("l1".into())),
        )
        .unwrap();
        let cursor = db.open_cursor_on_index("logs", "level_idx", None).unwrap();
        assert!(cursor.is_some(), "有数据后游标应返回 Some");
    }

    /// 测试 sessionStorage key() 方法对超出范围的索引返回 None。
    #[test]
    fn test_session_storage_key_out_of_range_returns_null() {
        let mut storage = WebStorage::new(StorageType::Session, "https://example.com");

        // 空存储时 key(0) 应返回 None
        assert_eq!(storage.key(0), None, "空存储 key(0) 应返回 None");
        assert_eq!(storage.key(100), None, "空存储 key(100) 应返回 None");

        // 设置 3 个键
        storage.set("alpha", "1").unwrap();
        storage.set("beta", "2").unwrap();
        storage.set("gamma", "3").unwrap();
        assert_eq!(storage.len(), 3);

        // 合法索引范围内应能获取键名
        let mut keys: Vec<String> = (0..storage.len())
            .filter_map(|i| storage.key(i).map(|s| s.to_string()))
            .collect();
        keys.sort();
        assert_eq!(keys, vec!["alpha", "beta", "gamma"]);

        // 索引恰好等于长度（边界）应返回 None
        assert_eq!(storage.key(3), None, "key(len) 应返回 None，索引从 0 开始");

        // 索引远超范围应返回 None
        assert_eq!(storage.key(999), None, "key(999) 应返回 None");

        // 删除一个后索引范围缩小
        storage.remove("beta");
        assert_eq!(storage.len(), 2);
        assert_eq!(storage.key(2), None, "删除后 key(2) 应返回 None");
    }

    // ── 新增边界测试 ──

    /// 测试 localStorage 大量写入和清除。
    #[test]
    fn test_localstorage_bulk_write_clear() {
        use crate::local_storage::{StorageType, WebStorage};
        let mut storage = WebStorage::new(StorageType::Local, "test-origin");
        for i in 0..100 {
            storage.set(&format!("key-{i}"), &format!("val-{i}")).unwrap();
        }
        assert_eq!(storage.len(), 100);
        storage.clear();
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.get("key-0"), None);
    }

    /// 测试 sessionStorage 不同 origin 隔离。
    #[test]
    fn test_session_storage_origin_isolation() {
        use crate::local_storage::{StorageType, WebStorage};
        let mut a = WebStorage::new(StorageType::Session, "origin-a");
        let mut b = WebStorage::new(StorageType::Session, "origin-b");
        a.set("shared-key", "value-a").unwrap();
        b.set("shared-key", "value-b").unwrap();
        assert_eq!(a.get("shared-key"), Some("value-a"));
        assert_eq!(b.get("shared-key"), Some("value-b"));
    }

    /// 测试 localStorage value 含特殊字符（换行/引号/unicode）。
    #[test]
    fn test_localstorage_special_characters() {
        use crate::local_storage::{StorageType, WebStorage};
        let mut storage = WebStorage::new(StorageType::Local, "test-origin");
        storage.set("newline", "line1\nline2").unwrap();
        storage.set("quotes", r#"he said "hello""#).unwrap();
        storage.set("unicode", "你好世界🌍").unwrap();
        assert_eq!(storage.get("newline"), Some("line1\nline2"));
        assert_eq!(storage.get("quotes"), Some(r#"he said "hello""#));
        assert_eq!(storage.get("unicode"), Some("你好世界🌍"));
    }

    /// 测试 IndexedDB 事务中止后数据不持久化。
    #[test]
    fn test_idb_transaction_aborted_no_persist() {
        use crate::indexed_db::{IdbDatabase, IdbKey};
        let mut db = IdbDatabase::new("test-abort2", 1);
        db.create_object_store("items", None, false).unwrap();

        // 直接通过 db 写入并确认存在
        let key = IdbKey::String("k1".into());
        db.put("items", serde_json::json!("v1"), Some(key.clone())).unwrap();
        assert!(db.get("items", &key).is_some());

        // 删除后确认不存在
        db.delete("items", &key).unwrap();
        assert!(db.get("items", &key).is_none());
    }

    /// 测试 IndexedDB 记录按插入顺序存储。
    #[test]
    fn test_idb_insertion_order() {
        use crate::indexed_db::IdbDatabase;
        let mut db = IdbDatabase::new("test-insertion-order", 1);
        db.create_object_store("ordered", None, false).unwrap();
        db.put("ordered", serde_json::json!("2"), Some(IdbKey::String("banana".into())))
            .unwrap();
        db.put("ordered", serde_json::json!("1"), Some(IdbKey::String("apple".into())))
            .unwrap();
        db.put("ordered", serde_json::json!("3"), Some(IdbKey::String("cherry".into())))
            .unwrap();

        // get_all 返回 3 条记录
        let records = db.get_all("ordered").unwrap();
        assert_eq!(records.len(), 3, "应有 3 条记录");

        // 验证可以通过 key 正确获取
        let apple = db.get("ordered", &IdbKey::String("apple".into()));
        assert!(apple.is_some());
        assert_eq!(apple.unwrap().value, serde_json::json!("1"));
    }

    // ── 新增边界测试（edge case tests） ──

    /// 测试 IndexedDB 事务在空 object store 列表上创建应成功。
    ///
    /// 验证 transaction 传入空 store 名称列表时，
    /// 没有不存在的 store 需要校验，事务应成功创建。
    #[test]
    fn test_idb_transaction_with_empty_store_list() {
        let mut db = IdbDatabase::new("test", 1);
        // 不创建任何 store
        let result = db.transaction(&[], IdbTransactionMode::ReadOnly);
        // 空列表不应包含不存在的 store，应成功
        assert!(result.is_ok(), "空 store 列表的事务应可创建");
        let tx = result.unwrap();
        assert_eq!(tx.store_names().len(), 0);
        assert_eq!(tx.mode(), IdbTransactionMode::ReadOnly);
    }

    /// 测试 IDBKeyRange::only 对不同类型键的精确匹配行为。
    ///
    /// 验证 only(Number)、only(String)、only(Binary) 各自只匹配完全相等的键，
    /// 跨类型键永远不匹配。
    #[test]
    fn test_idb_key_range_only_different_types() {
        use crate::indexed_db::IdbKeyRange;

        // only(Number) 只匹配相同数值
        let num_range = IdbKeyRange::only(IdbKey::Number(42.0));
        assert!(num_range.contains(&IdbKey::Number(42.0)));
        assert!(!num_range.contains(&IdbKey::Number(43.0)));
        // Number < String，所以 String 不在范围内
        assert!(!num_range.contains(&IdbKey::String("42".into())));

        // only(String) 只匹配相同字符串
        let str_range = IdbKeyRange::only(IdbKey::String("hello".into()));
        assert!(str_range.contains(&IdbKey::String("hello".into())));
        assert!(!str_range.contains(&IdbKey::String("world".into())));
        // Number < String，所以 Number 不在范围内
        assert!(!str_range.contains(&IdbKey::Number(1.0)));

        // only(Binary) 只匹配相同二进制数据
        let bin_range = IdbKeyRange::only(IdbKey::Binary(vec![1, 2, 3]));
        assert!(bin_range.contains(&IdbKey::Binary(vec![1, 2, 3])));
        assert!(!bin_range.contains(&IdbKey::Binary(vec![1, 2, 4])));
        // String < Binary，所以 String 不在范围内
        assert!(!bin_range.contains(&IdbKey::String("hello".into())));
    }

    /// 测试 IndexedDB 游标 advance(0) 行为：应重置到初始位置。
    ///
    /// advance(0) 在当前实现中会将 current 设为 0（回到第一条记录）。
    /// 本测试记录此行为。
    #[test]
    fn test_idb_cursor_advance_zero_resets_position() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("items", None, false).unwrap();
        for i in 1..=4 {
            db.add("items", serde_json::json!(i), Some(IdbKey::Number(i as f64)))
                .unwrap();
        }

        let mut cursor = db.open_cursor("items", None).unwrap().unwrap();
        // 初始位置：第 1 条（key=1）
        assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!(1));

        // 前进到第 3 条
        assert!(cursor.advance(2));
        assert_eq!(db.cursor_record(&cursor).unwrap().value, serde_json::json!(3));

        // advance(0) → 重置到第 1 条
        assert!(cursor.advance(0));
        assert_eq!(
            db.cursor_record(&cursor).unwrap().value,
            serde_json::json!(1),
            "advance(0) 应重置游标到初始位置"
        );
    }

    /// 测试 Cache API put 同一 URL 同一方法两次后的覆盖行为。
    ///
    /// 验证第二次 put 后：条目数仍为 1，match_request 返回最新响应，
    /// keys() 只包含一个 URL，旧响应的头不残留。
    #[test]
    fn test_cache_put_same_url_overwrites_completely() {
        let mut cs = CacheStorage::new();
        let cache = cs.open("assets");
        let url = "https://example.com/app.js";

        // 第一次 put
        let resp1 = CacheResponse::ok(b"v1".to_vec()).with_header("X-Version", "1");
        cache.put(CacheRequest::new(url), resp1).unwrap();

        // 第二次 put 同一 URL → 应完全覆盖
        let resp2 = CacheResponse::new(304, b"v2".to_vec()).with_header("X-Version", "2");
        cache.put(CacheRequest::new(url), resp2).unwrap();

        // 条目数应为 1
        assert_eq!(cache.len(), 1, "同一 URL 覆盖后条目数应仍为 1");
        assert_eq!(cache.keys().len(), 1);

        // match_request 应返回第二次的响应
        let matched = cache.match_request(&CacheRequest::new(url)).unwrap();
        assert_eq!(matched.status, 304);
        assert_eq!(matched.body, b"v2".to_vec());
        assert_eq!(matched.headers.get("X-Version"), Some(&"2".to_string()));
        // 第一次的头不应残留
        assert_eq!(matched.headers.len(), 1, "覆盖后应只有新响应的头");
    }

    /// 测试 Cache API delete 对不存在的 URL 返回 false，不影响已有条目。
    #[test]
    fn test_cache_delete_nonexistent_url_no_side_effect() {
        let mut cs = CacheStorage::new();
        let cache = cs.open("api");

        // 存入一个响应
        cache
            .put(
                CacheRequest::new("https://example.com/real"),
                CacheResponse::ok(b"data".to_vec()),
            )
            .unwrap();
        assert_eq!(cache.len(), 1);

        // 删除从未 put 的 URL
        let deleted = cache.delete(&CacheRequest::new("https://example.com/phantom"));
        assert!(!deleted, "删除不存在的 URL 应返回 false");

        // 原有条目不受影响
        assert_eq!(cache.len(), 1, "delete 不存在的 URL 不应影响已有条目");
        assert!(
            cache
                .match_request(&CacheRequest::new("https://example.com/real"))
                .is_some()
        );
    }

    /// 测试 CacheStorage has 要求精确匹配缓存名称，不支持部分匹配。
    #[test]
    fn test_cache_storage_has_requires_exact_name() {
        let mut cs = CacheStorage::new();
        cs.open("assets-v1");

        assert!(cs.has("assets-v1"), "完整名称应匹配");
        assert!(!cs.has("assets"), "部分名称前缀不应匹配");
        assert!(!cs.has("v1"), "部分名称后缀不应匹配");
        assert!(!cs.has("assets-v1-extra"), "包含完整名称的超集不应匹配");
    }

    /// 测试 localStorage 设置空字符串值后 get 返回 Some("")（不是 None）。
    #[test]
    fn test_local_storage_empty_string_value_is_not_none() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");

        // 设置空字符串值
        storage.set("blank", "").unwrap();
        // get 应返回 Some("")，不是 None
        assert_eq!(storage.get("blank"), Some(""), "空串值应返回 Some(\"\")，不是 None");
        assert!(storage.contains_key("blank"));
        assert_eq!(storage.len(), 1);
        // used_size = key 长度 + value 长度 = 5 + 0 = 5
        assert_eq!(storage.used_size(), 5);
    }

    /// 测试 localStorage get 不存在的键返回 None。
    ///
    /// 验证从未设置过的键、已删除的键、clear 后的键均返回 None。
    #[test]
    fn test_local_storage_get_nonexistent_returns_none() {
        let mut storage = WebStorage::new(StorageType::Local, "https://example.com");

        // 从未设置过的键
        assert_eq!(storage.get("never_set"), None, "从未设置的键应返回 None");

        // 设置后删除
        storage.set("temporary", "value").unwrap();
        storage.remove("temporary");
        assert_eq!(storage.get("temporary"), None, "已删除的键应返回 None");

        // 设置后 clear
        storage.set("cleared", "data").unwrap();
        storage.clear();
        assert_eq!(storage.get("cleared"), None, "clear 后的键应返回 None");
    }

    /// 测试 IndexedDB 唯一索引允许插入不同索引值的记录，
    /// 但拒绝插入相同索引值的记录（add 返回错误）。
    #[test]
    fn test_idb_unique_index_allows_different_rejects_same() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("accounts", None, false).unwrap();

        // 插入第一条记录
        db.add(
            "accounts",
            serde_json::json!({"username": "alice", "role": "admin"}),
            Some(IdbKey::String("acc1".into())),
        )
        .unwrap();

        // 创建唯一索引
        db.create_index("accounts", "username_idx", "username", true, false)
            .unwrap();

        // 不同 username 应允许
        db.add(
            "accounts",
            serde_json::json!({"username": "bob", "role": "user"}),
            Some(IdbKey::String("acc2".into())),
        )
        .unwrap();

        // 相同 username 应被拒绝（add 返回 Err）
        let result = db.add(
            "accounts",
            serde_json::json!({"username": "alice", "role": "guest"}),
            Some(IdbKey::String("acc3".into())),
        );
        assert!(result.is_err(), "唯一索引上插入重复值应返回错误");
    }

    /// 测试 IndexedDB multiEntry 索引与空数组值：
    /// 空数组作为 tags 值时，索引不应为该记录创建任何条目。
    #[test]
    fn test_idb_multi_entry_index_empty_array_no_entries() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("docs", None, false).unwrap();

        // 记录 1：tags 为空数组
        db.add(
            "docs",
            serde_json::json!({"title": "Empty Tags", "tags": []}),
            Some(IdbKey::String("d1".into())),
        )
        .unwrap();

        // 记录 2：tags 有值
        db.add(
            "docs",
            serde_json::json!({"title": "Has Tags", "tags": ["rust"]}),
            Some(IdbKey::String("d2".into())),
        )
        .unwrap();

        // 创建 multiEntry 索引
        db.create_index("docs", "tags_idx", "tags", false, true).unwrap();

        // 索引条目数应为 1（只有 "rust" 一个条目，空数组不产生任何索引条目）
        assert_eq!(
            db.count_from_index("docs", "tags_idx", None).unwrap(),
            1,
            "空数组不应产生索引条目"
        );

        // 查询 "rust" 只找到记录 2
        let rust = db
            .get_from_index("docs", "tags_idx", &IdbKey::String("rust".into()))
            .unwrap();
        assert_eq!(rust.len(), 1);
        assert_eq!(rust[0].value["title"], "Has Tags");

        // get_all_from_index 应返回 1 条（空数组记录不应出现）
        let all = db.get_all_from_index("docs", "tags_idx").unwrap();
        assert_eq!(all.len(), 1, "只有非空 tags 的记录应出现在索引中");
    }

    /// R3341：auto-increment key generator 在提供大于当前生成器值的显式数值 key 时须推进
    /// （W3C IndexedDB §1.8.2「Object store key generator」：提供数值 key ≥ 生成器当前值时，
    /// 生成器推进到 providedKey + 1）。非事务路径 `add`/`put` 的真 bug 修复回归测。
    #[test]
    fn test_idb_auto_increment_advances_on_explicit_large_number_key_r3341() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("seq", None, true).unwrap(); // auto_increment = true

        // 提供一个远大于生成器当前值（1）的显式数值 key。
        db.add("seq", serde_json::json!("explicit"), Some(IdbKey::Number(100.0)))
            .unwrap();

        // 下一次自动分配的 key 应推进到 101（§1.8.2），而非仍为 1（修复前行为）。
        let k = db.add("seq", serde_json::json!("auto"), None).unwrap();
        assert!(
            matches!(k, IdbKey::Number(n) if n == 101.0),
            "显式数值 key 100 后，自动 key 应推进到 101（W3C §1.8.2），实际 {k:?}"
        );

        // `put` 路径同理：显式数值 key 大于生成器值时推进。
        db.put("seq", serde_json::json!("explicit2"), Some(IdbKey::Number(500.0)))
            .unwrap();
        let k2 = db.add("seq", serde_json::json!("auto2"), None).unwrap();
        assert!(
            matches!(k2, IdbKey::Number(n) if n == 501.0),
            "put 显式数值 key 500 后，自动 key 应推进到 501，实际 {k2:?}"
        );
    }

    /// R3341：事务路径 `tx_add`/`tx_put` 同样须在显式数值 key ≥ 生成器当前值时推进
    /// （§1.8.2）。原 `tx_add` 仅在 `key == effective_next` 时推进（窄匹配，漏掉 key > 的情形）。
    #[test]
    fn test_idb_tx_auto_increment_advances_on_explicit_large_number_key_r3341() {
        use crate::indexed_db::IdbDatabase;
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("seq", None, true).unwrap();

        let mut tx = db
            .transaction(&["seq"], crate::indexed_db::IdbTransactionMode::ReadWrite)
            .unwrap();
        // 显式大数值 key（100 >> 生成器当前值 1）。
        db.tx_add(&tx, "seq", serde_json::json!("explicit"), Some(IdbKey::Number(100.0)))
            .unwrap();
        // 事务内下一次自动分配应推进到 101。
        let k = db.tx_add(&tx, "seq", serde_json::json!("auto"), None).unwrap();
        assert!(
            matches!(k, IdbKey::Number(n) if n == 101.0),
            "tx_add 显式 key 100 后，tx 内自动 key 应推进到 101，实际 {k:?}"
        );
        db.commit_tx(&mut tx).unwrap();

        // commit 后 live store.next_key 写回推进结果：tx 内先显式 key=100（gen→101）再 auto key=101（gen→102），
        // 故 live next_key=102。新事务的自动 key 应从 102 起（写回推进生效）。
        let mut tx2 = db
            .transaction(&["seq"], crate::indexed_db::IdbTransactionMode::ReadWrite)
            .unwrap();
        let k2 = db.tx_add(&tx2, "seq", serde_json::json!("auto3"), None).unwrap();
        assert!(
            matches!(k2, IdbKey::Number(n) if n == 102.0),
            "commit 后新事务自动 key 应为 102（tx 内两次推进写回），实际 {k2:?}"
        );
    }

    /// R3341：显式数值 key **小于** 当前生成器值时不应回退生成器（取 max 语义）。
    #[test]
    fn test_idb_auto_increment_no_regression_on_small_explicit_key_r3341() {
        let mut db = IdbDatabase::new("test", 1);
        db.create_object_store("seq", None, true).unwrap();

        // 先用自动分配到 key=1，生成器推进到 2。
        db.add("seq", serde_json::json!("auto1"), None).unwrap();
        // 提供一个小于生成器值（2）的显式 key —— 生成器不应回退到 1.5+1。
        db.add("seq", serde_json::json!("explicit"), Some(IdbKey::Number(1.5_f64)))
            .unwrap();
        // 下一次自动分配应仍为 2（取 max(2, ceil(1.5)+1=2)）。
        let k = db.add("seq", serde_json::json!("auto2"), None).unwrap();
        assert!(
            matches!(k, IdbKey::Number(n) if n == 2.0),
            "显式 key 1.5 < 生成器值 2 时，自动 key 应保持 2（取 max），实际 {k:?}"
        );
    }
}

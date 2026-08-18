//! 存储管理器 — 管理多个源的 localStorage、sessionStorage 与 IndexedDB。

use std::collections::HashMap;
use std::path::PathBuf;

use crate::StorageError;
use crate::indexed_db::persistence::IndexedDbPersistence;
use crate::indexed_db::{IdbDatabase, IdbTransaction};
use crate::local_storage::{StorageType, WebStorage};

#[cfg(test)]
#[path = "storage_manager/persistence_tests.rs"]
mod persistence_tests;

/// localStorage 默认最大容量（5 MB）。
const DEFAULT_MAX_SIZE: usize = 5 * 1024 * 1024;

/// 默认 IndexedDB 数据目录（`ZERO_STORAGE_DIR` 或平台 data directory）。
pub fn default_indexed_db_dir() -> PathBuf {
    if let Some(path) = zero_runtime_config::optional_path("ZERO_STORAGE_DIR") {
        return path;
    }
    dirs::data_dir()
        .map(|path| path.join("ZeroBrowser").join("Storage").join("IndexedDB"))
        .unwrap_or_else(|| PathBuf::from(".zero-browser-storage").join("IndexedDB"))
}

/// IndexedDB 数据库摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedDbInfo {
    /// 数据库名称。
    pub name: String,
    /// 数据库版本。
    pub version: u64,
}

/// 存储管理器 — 管理多个源的 localStorage、sessionStorage 与 IndexedDB。
pub struct StorageManager {
    /// localStorage 实例（按 origin 分组）。
    local_stores: HashMap<String, WebStorage>,
    /// sessionStorage 实例（按 origin 分组）。
    session_stores: HashMap<String, WebStorage>,
    /// IndexedDB 数据库（按 origin、数据库名分组）。
    indexed_databases: HashMap<String, HashMap<String, IdbDatabase>>,
    /// IndexedDB 持久化 owner；`None` 表示纯内存 manager。
    indexed_db_persistence: Option<IndexedDbPersistence>,
    /// 每个源的最大容量。
    default_max_size: usize,
}

impl StorageManager {
    /// 创建新的存储管理器。
    pub fn new() -> Self {
        Self::with_max_size(DEFAULT_MAX_SIZE)
    }

    /// 创建带自定义最大容量的存储管理器。
    pub fn with_max_size(default_max_size: usize) -> Self {
        Self {
            local_stores: HashMap::new(),
            session_stores: HashMap::new(),
            indexed_databases: HashMap::new(),
            indexed_db_persistence: None,
            default_max_size,
        }
    }

    /// 创建启用 IndexedDB 持久化的 manager，并加载目录中的全部数据库。
    pub fn with_indexed_db_persistence(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let (persistence, indexed_databases) = IndexedDbPersistence::open(root)?;
        Ok(Self {
            local_stores: HashMap::new(),
            session_stores: HashMap::new(),
            indexed_databases,
            indexed_db_persistence: Some(persistence),
            default_max_size: DEFAULT_MAX_SIZE,
        })
    }

    /// 获取指定源的 localStorage（如不存在则创建）。
    pub fn local_storage(&mut self, origin: &str) -> &mut WebStorage {
        self.local_stores
            .entry(origin.to_string())
            .or_insert_with(|| WebStorage::new_with_max_size(StorageType::Local, origin, self.default_max_size))
    }

    /// 获取指定源的 sessionStorage（如不存在则创建）。
    pub fn session_storage(&mut self, origin: &str) -> &mut WebStorage {
        self.session_stores
            .entry(origin.to_string())
            .or_insert_with(|| WebStorage::new_with_max_size(StorageType::Session, origin, self.default_max_size))
    }

    /// 打开指定源的 IndexedDB 数据库。
    ///
    /// 数据库不存在时按 `version` 创建；已存在时拒绝降级，升级版本会保留现有 schema 与数据。
    pub fn open_indexed_db(
        &mut self,
        origin: &str,
        name: &str,
        version: u64,
    ) -> Result<&mut IdbDatabase, StorageError> {
        if version == 0 {
            return Err(StorageError::Database(
                "IndexedDB version must be greater than zero".to_string(),
            ));
        }
        let current = self.indexed_db(origin, name);
        if current.is_some_and(|database| version < database.version) {
            return Err(StorageError::Database(format!(
                "Requested version {version} is lower than current version {}",
                current.map(|database| database.version).unwrap_or_default()
            )));
        }
        let changed = current.is_none_or(|database| version != database.version);
        if changed {
            let mut candidate = current.cloned().unwrap_or_else(|| IdbDatabase::new(name, version));
            candidate.version = version;
            self.persist_database(origin, &candidate)?;
            self.indexed_databases
                .entry(origin.to_string())
                .or_default()
                .insert(name.to_string(), candidate);
        }
        self.indexed_db_mut(origin, name)
            .ok_or_else(|| StorageError::Database("IndexedDB database was not opened".to_string()))
    }

    /// 获取已存在的指定源 IndexedDB 数据库。
    pub fn indexed_db(&self, origin: &str, name: &str) -> Option<&IdbDatabase> {
        self.indexed_databases.get(origin)?.get(name)
    }

    /// 获取已存在的指定源 IndexedDB 数据库的可变引用。
    pub fn indexed_db_mut(&mut self, origin: &str, name: &str) -> Option<&mut IdbDatabase> {
        self.indexed_databases.get_mut(origin)?.get_mut(name)
    }

    /// 在候选副本上修改数据库；持久化成功后才替换 live state。
    pub fn mutate_indexed_db<T>(
        &mut self,
        origin: &str,
        name: &str,
        mutation: impl FnOnce(&mut IdbDatabase) -> Result<T, StorageError>,
    ) -> Result<T, StorageError> {
        let mut candidate = self
            .indexed_db(origin, name)
            .cloned()
            .ok_or_else(|| StorageError::Database(format!("IndexedDB database '{name}' does not exist")))?;
        let result = mutation(&mut candidate)?;
        self.replace_indexed_db(origin, candidate)?;
        Ok(result)
    }

    /// 提交 transaction 到候选数据库；落盘失败时 live database 保持不变。
    pub fn commit_indexed_db_transaction(
        &mut self,
        origin: &str,
        name: &str,
        transaction: &mut IdbTransaction,
    ) -> Result<(), StorageError> {
        self.mutate_indexed_db(origin, name, |database| database.commit_tx(transaction))
    }

    /// 用完整候选数据库替换现有数据库；写盘先于 live state 切换。
    pub fn replace_indexed_db(&mut self, origin: &str, database: IdbDatabase) -> Result<(), StorageError> {
        let name = database.name.clone();
        self.persist_database(origin, &database)?;
        self.indexed_databases
            .entry(origin.to_string())
            .or_default()
            .insert(name, database);
        Ok(())
    }

    /// 删除指定源的 IndexedDB 数据库，并传播持久化错误。
    pub fn try_delete_indexed_db(&mut self, origin: &str, name: &str) -> Result<bool, StorageError> {
        if self.indexed_db(origin, name).is_none() {
            return Ok(false);
        }
        if let Some(persistence) = &self.indexed_db_persistence {
            persistence.delete(origin, name)?;
        }
        Ok(self.remove_indexed_db(origin, name))
    }

    /// 删除指定源的 IndexedDB 数据库，返回数据库是否存在。
    pub fn delete_indexed_db(&mut self, origin: &str, name: &str) -> bool {
        self.try_delete_indexed_db(origin, name).unwrap_or(false)
    }

    fn remove_indexed_db(&mut self, origin: &str, name: &str) -> bool {
        let Some(databases) = self.indexed_databases.get_mut(origin) else {
            return false;
        };
        let removed = databases.remove(name).is_some();
        if databases.is_empty() {
            self.indexed_databases.remove(origin);
        }
        removed
    }

    fn persist_database(&self, origin: &str, database: &IdbDatabase) -> Result<(), StorageError> {
        if let Some(persistence) = &self.indexed_db_persistence {
            persistence.write(origin, database)?;
        }
        Ok(())
    }

    /// 返回指定源的 IndexedDB 数据库名称。
    pub fn indexed_db_names(&self, origin: &str) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .indexed_databases
            .get(origin)
            .map(|databases| databases.keys().map(String::as_str).collect())
            .unwrap_or_default();
        names.sort_unstable();
        names
    }

    /// 返回指定源的 IndexedDB 数据库摘要，按名称排序。
    pub fn indexed_db_info(&self, origin: &str) -> Vec<IndexedDbInfo> {
        let mut info = self
            .indexed_databases
            .get(origin)
            .map(|databases| {
                databases
                    .values()
                    .map(|database| IndexedDbInfo {
                        name: database.name.clone(),
                        version: database.version,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        info.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        info
    }

    /// 清除指定源的所有存储。
    pub fn clear_origin(&mut self, origin: &str) {
        if let Some(store) = self.local_stores.get_mut(origin) {
            store.clear();
        }
        if let Some(store) = self.session_stores.get_mut(origin) {
            store.clear();
        }
        self.indexed_databases.remove(origin);
    }

    /// 清除所有 localStorage。
    pub fn clear_all_local(&mut self) {
        for store in self.local_stores.values_mut() {
            store.clear();
        }
    }

    /// 清除所有 sessionStorage。
    pub fn clear_all_session(&mut self) {
        for store in self.session_stores.values_mut() {
            store.clear();
        }
    }

    /// 删除所有源的 IndexedDB 数据库。
    pub fn clear_all_indexed_db(&mut self) {
        self.indexed_databases.clear();
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexed_db::IdbKey;

    #[test]
    fn test_manager_local_storage() {
        let mut manager = StorageManager::new();
        let storage = manager.local_storage("https://example.com");
        assert_eq!(storage.origin(), "https://example.com");
        assert_eq!(storage.storage_type(), StorageType::Local);
    }

    #[test]
    fn test_manager_session_storage() {
        let mut manager = StorageManager::new();
        let storage = manager.session_storage("https://example.com");
        assert_eq!(storage.origin(), "https://example.com");
        assert_eq!(storage.storage_type(), StorageType::Session);
    }

    #[test]
    fn test_manager_different_origins() {
        let mut manager = StorageManager::new();
        manager.local_storage("https://a.com").set("key", "a").unwrap();
        manager.local_storage("https://b.com").set("key", "b").unwrap();

        assert_eq!(manager.local_storage("https://a.com").get("key"), Some("a"));
        assert_eq!(manager.local_storage("https://b.com").get("key"), Some("b"));
    }

    #[test]
    fn test_manager_clear_origin() {
        let mut manager = StorageManager::new();
        manager.local_storage("https://a.com").set("key", "value").unwrap();
        manager.session_storage("https://a.com").set("sk", "sv").unwrap();
        manager.local_storage("https://b.com").set("key", "value").unwrap();

        manager.clear_origin("https://a.com");

        assert!(manager.local_storage("https://a.com").is_empty());
        assert!(manager.session_storage("https://a.com").is_empty());
        assert!(!manager.local_storage("https://b.com").is_empty());
    }

    #[test]
    fn test_manager_clear_all() {
        let mut manager = StorageManager::new();
        manager.local_storage("https://a.com").set("key", "value").unwrap();
        manager.local_storage("https://b.com").set("key", "value").unwrap();
        manager.session_storage("https://a.com").set("sk", "sv").unwrap();

        manager.clear_all_local();
        assert!(manager.local_storage("https://a.com").is_empty());
        assert!(manager.local_storage("https://b.com").is_empty());

        // sessionStorage unaffected
        assert!(!manager.session_storage("https://a.com").is_empty());

        manager.clear_all_session();
        assert!(manager.session_storage("https://a.com").is_empty());
    }

    // ── 新增测试 ──

    #[test]
    fn test_manager_default() {
        let mut manager = StorageManager::default();
        let storage = manager.local_storage("https://example.com");
        assert_eq!(storage.origin(), "https://example.com");
    }

    #[test]
    fn test_manager_per_origin_isolation() {
        let mut manager = StorageManager::new();
        manager.local_storage("https://a.com").set("x", "1").unwrap();
        manager.local_storage("https://b.com").set("x", "2").unwrap();
        manager.local_storage("https://c.com").set("x", "3").unwrap();

        assert_eq!(manager.local_storage("https://a.com").get("x"), Some("1"));
        assert_eq!(manager.local_storage("https://b.com").get("x"), Some("2"));
        assert_eq!(manager.local_storage("https://c.com").get("x"), Some("3"));

        // Removing from one origin does not affect others
        manager.local_storage("https://b.com").remove("x");
        assert_eq!(manager.local_storage("https://a.com").get("x"), Some("1"));
        assert_eq!(manager.local_storage("https://b.com").get("x"), None);
        assert_eq!(manager.local_storage("https://c.com").get("x"), Some("3"));
    }

    #[test]
    fn test_manager_session_cleared_per_origin() {
        let mut manager = StorageManager::new();
        manager.session_storage("https://a.com").set("s", "v1").unwrap();
        manager.session_storage("https://b.com").set("s", "v2").unwrap();
        manager.clear_origin("https://a.com");
        assert!(manager.session_storage("https://a.com").is_empty());
        assert!(!manager.session_storage("https://b.com").is_empty());
    }

    #[test]
    fn test_manager_local_and_session_independent() {
        let mut manager = StorageManager::new();
        manager.local_storage("https://a.com").set("key", "local-val").unwrap();
        manager
            .session_storage("https://a.com")
            .set("key", "session-val")
            .unwrap();
        assert_eq!(manager.local_storage("https://a.com").get("key"), Some("local-val"));
        assert_eq!(manager.session_storage("https://a.com").get("key"), Some("session-val"));
        // Clear local does not affect session
        manager.clear_all_local();
        assert!(manager.local_storage("https://a.com").is_empty());
        assert_eq!(manager.session_storage("https://a.com").get("key"), Some("session-val"));
    }

    #[test]
    fn test_manager_custom_max_size() {
        let mut manager = StorageManager::with_max_size(50);
        let result = manager.local_storage("https://example.com").set("k", &"x".repeat(50));
        assert!(result.is_err());
    }

    /// 测试 sessionStorage 源隔离：一个源的 sessionStorage 不会泄露到另一个源。
    #[test]
    fn test_session_storage_isolation() {
        let mut manager = StorageManager::new();

        // 两个不同源各自写入同名键
        manager
            .session_storage("https://alpha.com")
            .set("token", "aaa")
            .unwrap();
        manager.session_storage("https://beta.com").set("token", "bbb").unwrap();

        // 各自只能看到自己的数据
        assert_eq!(manager.session_storage("https://alpha.com").get("token"), Some("aaa"));
        assert_eq!(manager.session_storage("https://beta.com").get("token"), Some("bbb"));

        // 清除一个源不影响另一个
        manager.clear_origin("https://alpha.com");
        assert!(manager.session_storage("https://alpha.com").is_empty());
        assert_eq!(manager.session_storage("https://beta.com").get("token"), Some("bbb"));
    }

    /// 测试 StorageManager 对同一源同时存在 localStorage 和 sessionStorage 时，
    /// clear_all_local 只清 localStorage，sessionStorage 不受影响。
    #[test]
    fn test_manager_clear_all_local_preserves_session_data() {
        let mut manager = StorageManager::new();
        let origin = "https://app.example.com";

        manager.local_storage(origin).set("theme", "dark").unwrap();
        manager.local_storage(origin).set("lang", "zh").unwrap();
        manager.session_storage(origin).set("draft", "unsaved").unwrap();
        manager.session_storage(origin).set("tab", "editor").unwrap();

        assert_eq!(manager.local_storage(origin).len(), 2);
        assert_eq!(manager.session_storage(origin).len(), 2);

        // 只清除所有 localStorage
        manager.clear_all_local();

        // localStorage 被清空
        assert!(manager.local_storage(origin).is_empty());
        assert_eq!(manager.local_storage(origin).get("theme"), None);

        // sessionStorage 完好
        assert_eq!(manager.session_storage(origin).len(), 2);
        assert_eq!(manager.session_storage(origin).get("draft"), Some("unsaved"));
        assert_eq!(manager.session_storage(origin).get("tab"), Some("editor"));
    }

    #[test]
    fn test_manager_indexed_db_reopen_version_and_origin_isolation() {
        let mut manager = StorageManager::new();
        let key = IdbKey::String("item-1".to_string());
        {
            let database = manager.open_indexed_db("https://a.example", "app", 1).unwrap();
            database.create_object_store("items", None, false).unwrap();
            database
                .put("items", serde_json::json!({"value": "a"}), Some(key.clone()))
                .unwrap();
        }

        let reopened = manager.open_indexed_db("https://a.example", "app", 1).unwrap();
        assert_eq!(reopened.get("items", &key).unwrap().value["value"], "a");

        let isolated = manager.open_indexed_db("https://b.example", "app", 1).unwrap();
        assert!(!isolated.has_store("items"));

        assert!(manager.open_indexed_db("https://a.example", "app", 0).is_err());
        let upgraded = manager.open_indexed_db("https://a.example", "app", 2).unwrap();
        assert_eq!(upgraded.version, 2);
        assert_eq!(upgraded.get("items", &key).unwrap().value["value"], "a");
        assert!(manager.open_indexed_db("https://a.example", "app", 1).is_err());
    }

    #[test]
    fn indexed_db_versions_support_javascript_safe_integer_limit() {
        const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
        let mut manager = StorageManager::new();
        let database = manager
            .open_indexed_db("https://large-version.example", "app", MAX_SAFE_INTEGER)
            .unwrap();
        assert_eq!(database.version, MAX_SAFE_INTEGER);
        assert_eq!(
            manager.indexed_db_info("https://large-version.example"),
            vec![IndexedDbInfo {
                name: "app".to_string(),
                version: MAX_SAFE_INTEGER,
            }]
        );
    }

    #[test]
    fn test_manager_indexed_db_delete_and_clear_are_origin_scoped() {
        let mut manager = StorageManager::new();
        manager.open_indexed_db("https://a.example", "one", 1).unwrap();
        manager.open_indexed_db("https://a.example", "two", 2).unwrap();
        manager.open_indexed_db("https://b.example", "one", 1).unwrap();

        assert_eq!(manager.indexed_db_names("https://a.example"), vec!["one", "two"]);
        assert_eq!(
            manager.indexed_db_info("https://a.example"),
            vec![
                IndexedDbInfo {
                    name: "one".to_string(),
                    version: 1,
                },
                IndexedDbInfo {
                    name: "two".to_string(),
                    version: 2,
                },
            ]
        );
        assert_eq!(manager.indexed_db("https://a.example", "two").unwrap().version, 2);
        manager
            .indexed_db_mut("https://a.example", "two")
            .unwrap()
            .create_object_store("items", None, false)
            .unwrap();
        assert!(
            manager
                .indexed_db("https://a.example", "two")
                .unwrap()
                .has_store("items")
        );
        assert!(manager.delete_indexed_db("https://a.example", "one"));
        assert!(!manager.delete_indexed_db("https://a.example", "missing"));
        assert_eq!(manager.indexed_db_names("https://a.example"), vec!["two"]);
        assert_eq!(manager.indexed_db_names("https://b.example"), vec!["one"]);

        manager.clear_origin("https://a.example");
        assert!(manager.indexed_db_names("https://a.example").is_empty());
        assert_eq!(manager.indexed_db_names("https://b.example"), vec!["one"]);

        manager.clear_all_indexed_db();
        assert!(manager.indexed_db_names("https://b.example").is_empty());
    }
}

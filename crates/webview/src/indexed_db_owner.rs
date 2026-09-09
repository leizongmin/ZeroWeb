//! Embedded WebView IndexedDB storage ownership.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use zero_storage::StorageManager;

use crate::WebViewError;

/// Shareable IndexedDB storage owner for embedded WebViews.
///
/// Clone one owner into WebViews that belong to the same browsing context.
/// Private browsing contexts should use a separate [`Self::in_memory`] owner.
#[derive(Clone)]
pub struct IndexedDbOwner {
    storage: Arc<Mutex<StorageManager>>,
    /// OPFS per-origin 落盘根；`None` = 纯内存（in-memory owner / kill-switch 回退）。
    opfs_root: Option<PathBuf>,
}

impl IndexedDbOwner {
    /// Create an ephemeral owner that never writes IndexedDB data to disk.
    pub fn in_memory() -> Self {
        Self {
            storage: Arc::new(Mutex::new(StorageManager::new())),
            opfs_root: None,
        }
    }

    /// Create a persistent owner rooted at `path` and load existing databases.
    pub fn persistent(path: impl Into<PathBuf>) -> Result<Self, WebViewError> {
        let path = path.into();
        let storage = StorageManager::with_indexed_db_and_cache_storage_persistence(&path, path.join("CacheStorage"))
            .map_err(|error| WebViewError::Storage(error.to_string()))?;
        Ok(Self {
            storage: Arc::new(Mutex::new(storage)),
            // OPFS per-origin 落盘根随 owner 持久化域走（同根目录 OPFS/ 子目录）。
            opfs_root: Some(path.join("OPFS")),
        })
    }

    pub(crate) fn handler(&self) -> zero_engine::IndexedDbHandler {
        zero_page_runtime::indexed_db_handler(Arc::clone(&self.storage))
    }

    pub(crate) fn cache_storage_handler(&self) -> zero_engine::CacheStorageHandler {
        zero_page_runtime::cache_storage_handler(Arc::clone(&self.storage))
    }

    pub(crate) fn opfs_handler(&self) -> zero_engine::OpfsHandler {
        // OPFS 持久化面独立于 StorageManager（per-origin 树 + 自有落盘格式）；
        // opfs_root 为 None 时纯内存。
        zero_page_runtime::opfs_handler(self.opfs_root.clone())
    }
}

impl Default for IndexedDbOwner {
    fn default() -> Self {
        Self::in_memory()
    }
}

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
}

impl IndexedDbOwner {
    /// Create an ephemeral owner that never writes IndexedDB data to disk.
    pub fn in_memory() -> Self {
        Self {
            storage: Arc::new(Mutex::new(StorageManager::new())),
        }
    }

    /// Create a persistent owner rooted at `path` and load existing databases.
    pub fn persistent(path: impl Into<PathBuf>) -> Result<Self, WebViewError> {
        let storage = StorageManager::with_indexed_db_persistence(path)
            .map_err(|error| WebViewError::Storage(error.to_string()))?;
        Ok(Self {
            storage: Arc::new(Mutex::new(storage)),
        })
    }

    pub(crate) fn handler(&self) -> zero_engine::IndexedDbHandler {
        zero_page_runtime::indexed_db_handler(Arc::clone(&self.storage))
    }

    pub(crate) fn cache_storage_handler(&self) -> zero_engine::CacheStorageHandler {
        zero_page_runtime::cache_storage_handler(Arc::clone(&self.storage))
    }
}

impl Default for IndexedDbOwner {
    fn default() -> Self {
        Self::in_memory()
    }
}

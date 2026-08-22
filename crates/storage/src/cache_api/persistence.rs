use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
#[cfg(unix)]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{Cache, CacheRequest, CacheResponse, CacheStorage};
use crate::StorageError;

const FORMAT_VERSION: u32 = 1;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) type CacheStorageMap = HashMap<String, CacheStorage>;

pub(crate) struct CacheStoragePersistence {
    root: PathBuf,
}

impl CacheStoragePersistence {
    pub(crate) fn open(root: impl Into<PathBuf>) -> Result<(Self, CacheStorageMap), StorageError> {
        let persistence = Self { root: root.into() };
        fs::create_dir_all(&persistence.root).map_err(io_error)?;
        persistence.recover_interrupted_writes()?;
        let cache_storages = persistence.load_cache_storages()?;
        Ok((persistence, cache_storages))
    }

    pub(crate) fn write(&self, origin: &str, cache_storage: &CacheStorage) -> Result<(), StorageError> {
        if cache_storage.is_empty() {
            return self.delete(origin);
        }
        let persisted = PersistedCacheStorage::from_cache_storage(origin, cache_storage);
        let bytes = serde_json::to_vec(&persisted)
            .map_err(|error| StorageError::Serialization(format!("failed to encode CacheStorage: {error}")))?;
        let path = self.cache_storage_path(origin);
        fs::create_dir_all(&self.root).map_err(io_error)?;
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("cache-storage.cache");
        let temporary = self
            .root
            .join(format!(".{file_name}.{}.{}.tmp", std::process::id(), sequence));
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .map_err(io_error)?;
            file.write_all(&bytes).map_err(io_error)?;
            file.sync_all().map_err(io_error)?;
            replace_file(&temporary, &path)?;
            sync_directory(&self.root)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    pub(crate) fn delete(&self, origin: &str) -> Result<(), StorageError> {
        let path = self.cache_storage_path(origin);
        match fs::remove_file(&path) {
            Ok(()) => {
                sync_directory(&self.root)?;
                Ok(())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(io_error(error)),
        }
    }

    fn load_cache_storages(&self) -> Result<CacheStorageMap, StorageError> {
        let mut cache_storages = CacheStorageMap::new();
        for entry in fs::read_dir(&self.root).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            let path = entry.path();
            if !entry.file_type().map_err(io_error)?.is_file()
                || path.extension().and_then(|value| value.to_str()) != Some("cache")
            {
                continue;
            }
            let bytes = fs::read(&path).map_err(io_error)?;
            let persisted: PersistedCacheStorage = serde_json::from_slice(&bytes)
                .map_err(|error| StorageError::Serialization(format!("failed to decode CacheStorage: {error}")))?;
            let expected_path = self.cache_storage_path(&persisted.origin);
            if path != expected_path {
                return Err(StorageError::Serialization(
                    "CacheStorage persistence path does not match stored origin".to_string(),
                ));
            }
            let origin = persisted.origin.clone();
            let cache_storage = persisted.into_cache_storage()?;
            if cache_storages.insert(origin, cache_storage).is_some() {
                return Err(StorageError::Serialization(
                    "duplicate CacheStorage origin in persistence directory".to_string(),
                ));
            }
        }
        Ok(cache_storages)
    }

    fn recover_interrupted_writes(&self) -> Result<(), StorageError> {
        for entry in fs::read_dir(&self.root).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if file_name.ends_with(".tmp") {
                fs::remove_file(path).map_err(io_error)?;
            } else if let Some(target_name) = file_name.strip_suffix(".bak") {
                let target = path.with_file_name(target_name);
                if target.exists() {
                    fs::remove_file(path).map_err(io_error)?;
                } else {
                    fs::rename(path, target).map_err(io_error)?;
                }
            }
        }
        Ok(())
    }

    fn cache_storage_path(&self, origin: &str) -> PathBuf {
        self.root.join(format!("{}.cache", hash_component(origin)))
    }
}

#[derive(Serialize, Deserialize)]
struct PersistedCacheStorage {
    format: u32,
    origin: String,
    caches: Vec<PersistedCache>,
}

#[derive(Serialize, Deserialize)]
struct PersistedCache {
    name: String,
    entries: Vec<PersistedCacheEntry>,
}

#[derive(Serialize, Deserialize)]
struct PersistedCacheEntry {
    request: CacheRequest,
    response: CacheResponse,
}

impl PersistedCacheStorage {
    fn from_cache_storage(origin: &str, cache_storage: &CacheStorage) -> Self {
        let caches = cache_storage
            .iter_caches()
            .map(|(name, cache)| PersistedCache {
                name: name.to_string(),
                entries: cache
                    .entries()
                    .map(|(request, response)| PersistedCacheEntry {
                        request: request.clone(),
                        response: response.clone(),
                    })
                    .collect(),
            })
            .collect();
        Self {
            format: FORMAT_VERSION,
            origin: origin.to_string(),
            caches,
        }
    }

    fn into_cache_storage(self) -> Result<CacheStorage, StorageError> {
        if self.format != FORMAT_VERSION {
            return Err(StorageError::Serialization(format!(
                "unsupported CacheStorage persistence format {}",
                self.format
            )));
        }
        if self.origin.is_empty() {
            return Err(StorageError::Serialization(
                "CacheStorage persistence metadata is invalid".to_string(),
            ));
        }
        let mut names = HashSet::new();
        let mut caches = Vec::with_capacity(self.caches.len());
        for cache in self.caches {
            if !names.insert(cache.name.clone()) {
                return Err(StorageError::Serialization(
                    "duplicate cache in CacheStorage persistence data".to_string(),
                ));
            }
            let entries = cache
                .entries
                .into_iter()
                .map(|entry| (entry.request, entry.response))
                .collect();
            caches.push((cache.name.clone(), Cache::from_entries(&cache.name, entries)?));
        }
        CacheStorage::from_caches(caches)
    }
}

fn hash_component(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn replace_file(temporary: &Path, target: &Path) -> Result<(), StorageError> {
    #[cfg(not(windows))]
    {
        fs::rename(temporary, target).map_err(io_error)
    }
    #[cfg(windows)]
    {
        let backup = target.with_extension("cache.bak");
        if target.exists() {
            let _ = fs::remove_file(&backup);
            fs::rename(target, &backup).map_err(io_error)?;
        }
        if let Err(error) = fs::rename(temporary, target) {
            let _ = fs::rename(&backup, target);
            return Err(io_error(error));
        }
        let _ = fs::remove_file(backup);
        Ok(())
    }
}

fn sync_directory(path: &Path) -> Result<(), StorageError> {
    #[cfg(unix)]
    {
        File::open(path)
            .and_then(|directory| directory.sync_all())
            .map_err(io_error)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

fn io_error(error: std::io::Error) -> StorageError {
    StorageError::Io(error.to_string())
}

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
// File 仅 unix 分支的 sync_directory 使用；不门卫则 Windows clippy 判 unused import
#[cfg(unix)]
use std::fs::File;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{IdbDatabase, IdbIndexKeyPath, IdbKey};
use crate::StorageError;

const FORMAT_VERSION: u32 = 1;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
type DatabaseMap = HashMap<String, HashMap<String, IdbDatabase>>;

pub(crate) struct IndexedDbPersistence {
    root: PathBuf,
}

impl IndexedDbPersistence {
    pub(crate) fn open(root: impl Into<PathBuf>) -> Result<(Self, DatabaseMap), StorageError> {
        let persistence = Self { root: root.into() };
        fs::create_dir_all(&persistence.root).map_err(io_error)?;
        persistence.recover_interrupted_writes()?;
        let databases = persistence.load_databases()?;
        Ok((persistence, databases))
    }

    pub(crate) fn write(&self, origin: &str, database: &IdbDatabase) -> Result<(), StorageError> {
        let persisted = PersistedDatabase::from_database(origin, database)?;
        let bytes = serde_json::to_vec(&persisted)
            .map_err(|error| StorageError::Serialization(format!("failed to encode IndexedDB database: {error}")))?;
        let path = self.database_path(origin, &database.name);
        let parent = path
            .parent()
            .ok_or_else(|| StorageError::Database("IndexedDB persistence path has no parent".to_string()))?;
        fs::create_dir_all(parent).map_err(io_error)?;
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("database.json");
        let temporary = parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), sequence));
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .map_err(io_error)?;
            file.write_all(&bytes).map_err(io_error)?;
            file.sync_all().map_err(io_error)?;
            replace_file(&temporary, &path)?;
            sync_directory(parent)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    pub(crate) fn delete(&self, origin: &str, name: &str) -> Result<(), StorageError> {
        let path = self.database_path(origin, name);
        match fs::remove_file(&path) {
            Ok(()) => {
                if let Some(parent) = path.parent() {
                    sync_directory(parent)?;
                }
                Ok(())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(io_error(error)),
        }
    }

    fn load_databases(&self) -> Result<DatabaseMap, StorageError> {
        let mut databases = DatabaseMap::new();
        for origin_entry in fs::read_dir(&self.root).map_err(io_error)? {
            let origin_entry = origin_entry.map_err(io_error)?;
            if !origin_entry.file_type().map_err(io_error)?.is_dir() {
                continue;
            }
            for database_entry in fs::read_dir(origin_entry.path()).map_err(io_error)? {
                let database_entry = database_entry.map_err(io_error)?;
                let path = database_entry.path();
                if !database_entry.file_type().map_err(io_error)?.is_file()
                    || path.extension().and_then(|value| value.to_str()) != Some("json")
                {
                    continue;
                }
                let bytes = fs::read(&path).map_err(io_error)?;
                let persisted: PersistedDatabase = serde_json::from_slice(&bytes).map_err(|error| {
                    StorageError::Serialization(format!("failed to decode IndexedDB database: {error}"))
                })?;
                let expected_path = self.database_path(&persisted.origin, &persisted.name);
                if path != expected_path {
                    return Err(StorageError::Serialization(
                        "IndexedDB persistence path does not match stored origin and database name".to_string(),
                    ));
                }
                let origin = persisted.origin.clone();
                let name = persisted.name.clone();
                let database = persisted.into_database()?;
                if databases.entry(origin).or_default().insert(name, database).is_some() {
                    return Err(StorageError::Serialization(
                        "duplicate IndexedDB database in persistence directory".to_string(),
                    ));
                }
            }
        }
        Ok(databases)
    }

    fn recover_interrupted_writes(&self) -> Result<(), StorageError> {
        for origin_entry in fs::read_dir(&self.root).map_err(io_error)? {
            let origin_entry = origin_entry.map_err(io_error)?;
            if !origin_entry.file_type().map_err(io_error)?.is_dir() {
                continue;
            }
            for entry in fs::read_dir(origin_entry.path()).map_err(io_error)? {
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
        }
        Ok(())
    }

    fn database_path(&self, origin: &str, name: &str) -> PathBuf {
        self.root
            .join(hash_component(origin))
            .join(format!("{}.json", hash_component(name)))
    }
}

#[derive(Serialize, Deserialize)]
struct PersistedDatabase {
    format: u32,
    origin: String,
    name: String,
    version: u64,
    stores: Vec<PersistedStore>,
}

#[derive(Serialize, Deserialize)]
struct PersistedStore {
    name: String,
    key_path: Option<PersistedObjectStoreKeyPath>,
    auto_increment: bool,
    next_key: u64,
    indexes: Vec<PersistedIndex>,
    records: Vec<PersistedRecord>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum PersistedObjectStoreKeyPath {
    String(String),
    Sequence(Vec<String>),
}

#[derive(Serialize, Deserialize)]
struct PersistedIndex {
    name: String,
    key_path: PersistedIndexKeyPath,
    unique: bool,
    multi_entry: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum PersistedIndexKeyPath {
    String(String),
    Sequence(Vec<String>),
}

#[derive(Serialize, Deserialize)]
struct PersistedRecord {
    key: PersistedKey,
    value: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum PersistedKey {
    Number(String),
    Date(String),
    String(String),
    Binary(Vec<u8>),
    Array(Vec<PersistedKey>),
}

impl PersistedDatabase {
    fn from_database(origin: &str, database: &IdbDatabase) -> Result<Self, StorageError> {
        let stores = database
            .store_info()
            .into_iter()
            .map(|store| {
                let records = database
                    .get_all(&store.name)?
                    .into_iter()
                    .map(|record| PersistedRecord {
                        key: PersistedKey::from(&record.key),
                        value: record.value.clone(),
                    })
                    .collect();
                let indexes = store
                    .indexes
                    .into_iter()
                    .map(|index| PersistedIndex {
                        name: index.name,
                        key_path: index.key_path.into(),
                        unique: index.unique,
                        multi_entry: index.multi_entry,
                    })
                    .collect();
                Ok(PersistedStore {
                    next_key: database.key_generator(&store.name)?,
                    name: store.name,
                    key_path: store.key_path.map(PersistedObjectStoreKeyPath::from),
                    auto_increment: store.auto_increment,
                    indexes,
                    records,
                })
            })
            .collect::<Result<Vec<_>, StorageError>>()?;
        Ok(Self {
            format: FORMAT_VERSION,
            origin: origin.to_string(),
            name: database.name.clone(),
            version: database.version,
            stores,
        })
    }

    fn into_database(self) -> Result<IdbDatabase, StorageError> {
        if self.format != FORMAT_VERSION {
            return Err(StorageError::Serialization(format!(
                "unsupported IndexedDB persistence format {}",
                self.format
            )));
        }
        if self.origin.is_empty() || self.name.is_empty() || self.version == 0 {
            return Err(StorageError::Serialization(
                "IndexedDB persistence metadata is invalid".to_string(),
            ));
        }
        let mut database = IdbDatabase::new(&self.name, self.version);
        let mut store_names = HashSet::new();
        for store in self.stores {
            if !store_names.insert(store.name.clone()) {
                return Err(StorageError::Serialization(
                    "duplicate object store in IndexedDB persistence data".to_string(),
                ));
            }
            database.create_object_store_with_key_path(
                &store.name,
                store.key_path.map(IdbIndexKeyPath::from),
                store.auto_increment,
            )?;
            let mut record_keys = HashSet::new();
            for record in store.records {
                let key = record.key.into_key()?;
                if !record_keys.insert(key.clone()) {
                    return Err(StorageError::Serialization(
                        "duplicate record key in IndexedDB persistence data".to_string(),
                    ));
                }
                database.put(&store.name, record.value, Some(key))?;
            }
            let generated_next = database.key_generator(&store.name)?;
            if store.next_key < generated_next {
                return Err(StorageError::Serialization(
                    "IndexedDB key generator is behind persisted records".to_string(),
                ));
            }
            database.restore_key_generator(&store.name, store.next_key)?;
            let mut index_names = HashSet::new();
            for index in store.indexes {
                if !index_names.insert(index.name.clone()) {
                    return Err(StorageError::Serialization(
                        "duplicate index in IndexedDB persistence data".to_string(),
                    ));
                }
                database.create_index_with_key_path(
                    &store.name,
                    &index.name,
                    index.key_path.into(),
                    index.unique,
                    index.multi_entry,
                )?;
            }
        }
        Ok(database)
    }
}

impl From<&IdbKey> for PersistedKey {
    fn from(key: &IdbKey) -> Self {
        match key {
            IdbKey::Number(value) => Self::Number(value.to_string()),
            IdbKey::Date(value) => Self::Date(value.to_string()),
            IdbKey::String(value) => Self::String(value.clone()),
            IdbKey::Binary(value) => Self::Binary(value.clone()),
            IdbKey::Array(value) => Self::Array(value.iter().map(Self::from).collect()),
        }
    }
}

impl PersistedKey {
    fn into_key(self) -> Result<IdbKey, StorageError> {
        let key = match self {
            Self::Number(value) => IdbKey::Number(parse_f64(&value, false)?),
            Self::Date(value) => IdbKey::Date(parse_f64(&value, true)?),
            Self::String(value) => IdbKey::String(value),
            Self::Binary(value) => IdbKey::Binary(value),
            Self::Array(value) => IdbKey::Array(
                value
                    .into_iter()
                    .map(Self::into_key)
                    .collect::<Result<Vec<_>, StorageError>>()?,
            ),
        };
        if !key.is_valid_key() {
            return Err(StorageError::Serialization(
                "invalid key in IndexedDB persistence data".to_string(),
            ));
        }
        Ok(key)
    }
}

impl From<IdbIndexKeyPath> for PersistedObjectStoreKeyPath {
    fn from(key_path: IdbIndexKeyPath) -> Self {
        match key_path {
            IdbIndexKeyPath::String(value) => Self::String(value),
            IdbIndexKeyPath::Sequence(value) => Self::Sequence(value),
        }
    }
}

impl From<PersistedObjectStoreKeyPath> for IdbIndexKeyPath {
    fn from(key_path: PersistedObjectStoreKeyPath) -> Self {
        match key_path {
            PersistedObjectStoreKeyPath::String(value) => Self::String(value),
            PersistedObjectStoreKeyPath::Sequence(value) => Self::Sequence(value),
        }
    }
}

impl From<IdbIndexKeyPath> for PersistedIndexKeyPath {
    fn from(key_path: IdbIndexKeyPath) -> Self {
        match key_path {
            IdbIndexKeyPath::String(value) => Self::String(value),
            IdbIndexKeyPath::Sequence(value) => Self::Sequence(value),
        }
    }
}

impl From<PersistedIndexKeyPath> for IdbIndexKeyPath {
    fn from(key_path: PersistedIndexKeyPath) -> Self {
        match key_path {
            PersistedIndexKeyPath::String(value) => Self::String(value),
            PersistedIndexKeyPath::Sequence(value) => Self::Sequence(value),
        }
    }
}

fn parse_f64(value: &str, finite: bool) -> Result<f64, StorageError> {
    let parsed = match value {
        "inf" | "Infinity" => f64::INFINITY,
        "-inf" | "-Infinity" => f64::NEG_INFINITY,
        _ => value.parse().map_err(|_| {
            StorageError::Serialization("invalid numeric key in IndexedDB persistence data".to_string())
        })?,
    };
    if parsed.is_nan() || (finite && !parsed.is_finite()) {
        return Err(StorageError::Serialization(
            "invalid numeric key in IndexedDB persistence data".to_string(),
        ));
    }
    Ok(parsed)
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
        let backup = target.with_extension("json.bak");
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

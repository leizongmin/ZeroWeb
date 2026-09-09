//! OPFS per-origin 持久化 — 「JSON + 临时文件 + rename + fsync + 中断恢复」模式。
//!
//! 照 `cache_api/persistence.rs` 既有模式：每 origin 一个 `<sha256(origin)>.opfs` 文件，
//! 写入走 create-new 临时文件 → sync → rename（Windows 走 .bak 换名）→ 目录 sync；
//! open 时清扫残留 `.tmp`/`.bak`（中断恢复）。

use std::collections::BTreeMap;
use std::fmt::Write as _;
#[cfg(unix)]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::OpfsNode;
use crate::StorageError;

/// 持久化文件表（路径 → [内容, 最后修改毫秒]）——落盘与恢复的交换格式。
pub type PersistedFileMap = BTreeMap<Vec<String>, (Vec<u8>, i64)>;

const FORMAT_VERSION: u32 = 1;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// OPFS 持久化 owner（绑定 per-origin 存储根目录）。
pub struct OpfsPersistence {
    root: PathBuf,
}

impl OpfsPersistence {
    /// 打开/创建持久化根目录（中断恢复扫描 + 全量加载）。
    pub fn open(root: impl Into<PathBuf>) -> Result<(Self, BTreeMap<String, PersistedFileMap>), StorageError> {
        let persistence = Self { root: root.into() };
        fs::create_dir_all(&persistence.root).map_err(io_error)?;
        persistence.recover_interrupted_writes()?;
        let filesystems = persistence.load_all()?;
        Ok((persistence, filesystems))
    }

    /// 写入一个 origin 的文件树（空树 → 删除落盘文件）。磁盘错误以 `StorageError` 返回。
    pub fn write(&self, origin: &str, tree: &BTreeMap<Vec<String>, (Vec<u8>, i64)>) -> Result<(), StorageError> {
        if tree.is_empty() {
            return self.delete(origin);
        }
        let persisted = PersistedOpfs {
            format: FORMAT_VERSION,
            origin: origin.to_string(),
            files: tree
                .iter()
                .map(|(path, (bytes, modified))| PersistedOpfsFile {
                    path: path.clone(),
                    data: bytes.clone(),
                    last_modified: *modified,
                })
                .collect(),
        };
        let bytes = serde_json::to_vec(&persisted)
            .map_err(|error| StorageError::Serialization(format!("failed to encode OPFS: {error}")))?;
        let path = self.origin_path(origin);
        fs::create_dir_all(&self.root).map_err(io_error)?;
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("origin.opfs");
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

    /// 删除 origin 的落盘文件。
    pub fn delete(&self, origin: &str) -> Result<(), StorageError> {
        let path = self.origin_path(origin);
        match fs::remove_file(&path) {
            Ok(()) => {
                sync_directory(&self.root)?;
                Ok(())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(io_error(error)),
        }
    }

    /// 从根目录恢复全部 origin 的文件树。
    fn load_all(&self) -> Result<BTreeMap<String, PersistedFileMap>, StorageError> {
        let mut filesystems = BTreeMap::new();
        for entry in fs::read_dir(&self.root).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            let path = entry.path();
            if !entry.file_type().map_err(io_error)?.is_file()
                || path.extension().and_then(|value| value.to_str()) != Some("opfs")
            {
                continue;
            }
            let bytes = fs::read(&path).map_err(io_error)?;
            let persisted: PersistedOpfs = serde_json::from_slice(&bytes)
                .map_err(|error| StorageError::Serialization(format!("failed to decode OPFS: {error}")))?;
            if persisted.format != FORMAT_VERSION {
                return Err(StorageError::Serialization(format!(
                    "unsupported OPFS persistence format {}",
                    persisted.format
                )));
            }
            if persisted.origin.is_empty() {
                return Err(StorageError::Serialization(
                    "OPFS persistence metadata is invalid".to_string(),
                ));
            }
            let expected_path = self.origin_path(&persisted.origin);
            if path != expected_path {
                return Err(StorageError::Serialization(
                    "OPFS persistence path does not match stored origin".to_string(),
                ));
            }
            let mut tree = BTreeMap::new();
            for file in persisted.files {
                tree.insert(file.path, (file.data, file.last_modified));
            }
            filesystems.insert(persisted.origin, tree);
        }
        Ok(filesystems)
    }

    /// 清扫中断写残留（`.tmp` 删除；`.bak` 换回或删除）。
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

    fn origin_path(&self, origin: &str) -> PathBuf {
        self.root.join(format!("{}.opfs", hash_component(origin)))
    }
}

/// 落盘快照：origin + 扁平文件表（路径 → 内容）。
#[derive(Serialize, Deserialize)]
struct PersistedOpfs {
    format: u32,
    origin: String,
    files: Vec<PersistedOpfsFile>,
}

#[derive(Serialize, Deserialize)]
struct PersistedOpfsFile {
    path: Vec<String>,
    data: Vec<u8>,
    last_modified: i64,
}

/// 从 OpfsFileSystem 提取扁平文件表（持久化交换格式）。
pub(crate) fn flatten_filesystem(fs: &super::OpfsFileSystem) -> PersistedFileMap {
    let mut tree = BTreeMap::new();
    fn walk(node: &OpfsNode, path: Vec<String>, tree: &mut PersistedFileMap) {
        match node {
            OpfsNode::Dir(children) => {
                for (name, child) in children {
                    let mut child_path = path.clone();
                    child_path.push(name.clone());
                    walk(child, child_path, tree);
                }
            }
            OpfsNode::File(data) => {
                tree.insert(path, (data.bytes.clone(), data.last_modified));
            }
        }
    }
    walk(&fs.root, Vec::new(), &mut tree);
    tree
}

/// 从扁平文件表重建文件树（加载路径）。
pub(crate) fn tree_from_files(files: &PersistedFileMap) -> OpfsNode {
    let mut root = OpfsNode::new_dir();
    for (path, (bytes, last_modified)) in files {
        let mut current = &mut root;
        for segment in &path[..path.len().saturating_sub(1)] {
            current = match current {
                OpfsNode::Dir(children) => children.entry(segment.clone()).or_insert_with(OpfsNode::new_dir),
                OpfsNode::File(_) => continue,
            };
        }
        if let (Some(name), OpfsNode::Dir(children)) = (path.last(), current) {
            children.insert(
                name.clone(),
                OpfsNode::File(super::OpfsFileData {
                    bytes: bytes.clone(),
                    last_modified: *last_modified,
                }),
            );
        }
    }
    root
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
        let backup = target.with_extension("opfs.bak");
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

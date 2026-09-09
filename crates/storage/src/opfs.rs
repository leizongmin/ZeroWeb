//! OPFS（Origin Private File System）— 目录树、句柄与可写文件流。
//!
//! spec https://fs.spec.whatwg.org/ 。页面侧入口为 `navigator.storage.getDirectory()`
//! （见 engine `js_dom_shim/part02.js` navigator.storage 段），本模块是该面的 Rust
//! 真实实现。句柄以**规范路径**表达（同一路径无论获取多少次是同一条目，
//! `isSameEntry` 为 true）；错误以 [`OpfsError`] 表达（接线层映射为 DOMException，
//! name + legacy code 与 WPT `assert_throws_dom` 断言一致：NotFoundError=8 /
//! TypeMismatchError=17 / NoModificationAllowedError=7 / InvalidModificationError=13 /
//! InvalidStateError=11）。持久化见 [`OpfsPersistence`]（per-origin 落盘）。

use std::collections::BTreeMap;

use crate::StorageError;

pub(crate) mod persistence;

pub use persistence::OpfsPersistence;
pub use persistence::PersistedFileMap;

/// OPFS 节点：目录或文件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OpfsNode {
    /// 目录（子项按名字典序，迭代序与 WPT 断言一致）。
    Dir(BTreeMap<String, OpfsNode>),
    /// 文件内容。
    File(OpfsFileData),
}

impl Default for OpfsNode {
    fn default() -> Self {
        Self::Dir(BTreeMap::new())
    }
}

/// 文件内容 + 最后修改时间（`getFile()` 的 lastModified 语义）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct OpfsFileData {
    /// 内容字节。
    pub(crate) bytes: Vec<u8>,
    /// 最后修改时间（Unix 毫秒；0 = 尚未写入）。
    pub(crate) last_modified: i64,
}

impl OpfsNode {
    /// 新建空目录。
    fn new_dir() -> Self {
        Self::Dir(BTreeMap::new())
    }

    /// 是否为目录。
    fn is_dir(&self) -> bool {
        matches!(self, Self::Dir(_))
    }

    /// 是否为文件。
    fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }
}

/// OPFS 错误 — 接线层按 `name`/`code` 构造 DOMException。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpfsError {
    /// DOMException 名（NotFoundError / TypeMismatchError / NoModificationAllowedError /
    /// InvalidModificationError / InvalidStateError / InvalidAccessError）。
    pub name: String,
    /// legacy code（WebIDL DOMException 表；WPT `assert_throws_dom` 同时断言二者）。
    pub code: u16,
    /// 错误消息。
    pub message: String,
}

impl OpfsError {
    /// NotFoundError（code 8）：路径项不存在。
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            name: "NotFoundError".into(),
            code: 8,
            message: message.into(),
        }
    }

    /// TypeMismatchError（code 17）：期望文件处是目录 / 期望目录处是文件。
    pub fn type_mismatch(message: impl Into<String>) -> Self {
        Self {
            name: "TypeMismatchError".into(),
            code: 17,
            message: message.into(),
        }
    }

    /// NoModificationAllowedError（code 7）：目标（或其子树）有打开中的可写流。
    pub fn no_modification_allowed(message: impl Into<String>) -> Self {
        Self {
            name: "NoModificationAllowedError".into(),
            code: 7,
            message: message.into(),
        }
    }

    /// InvalidModificationError（code 13）：非空目录非 recursive 删除。
    pub fn invalid_modification(message: impl Into<String>) -> Self {
        Self {
            name: "InvalidModificationError".into(),
            code: 13,
            message: message.into(),
        }
    }

    /// InvalidStateError（code 11）：流已关闭/已 abort 后继续操作。
    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self {
            name: "InvalidStateError".into(),
            code: 11,
            message: message.into(),
        }
    }

    /// TypeError 面（spec：名称校验等参数错误走 JS TypeError，非 DOMException——
    /// 由接线层直接按消息判类型；这里借道 Type 编码透传）。
    pub fn type_error(message: impl Into<String>) -> Self {
        Self {
            name: "TypeError".into(),
            code: 0,
            message: message.into(),
        }
    }

    /// 编码为可跨 host 命令边界透传的 `StorageError::Type`（`name|code|message`）。
    pub fn into_storage_error(self) -> StorageError {
        StorageError::Type(format!("{}|{}|{}", self.name, self.code, self.message))
    }
}

/// 从 `StorageError::Type` 还原 [`OpfsError`]（非 OPFS 编码错误返回 None）。
pub fn opfs_error_from_storage(error: &StorageError) -> Option<OpfsError> {
    let StorageError::Type(text) = error else {
        return None;
    };
    let mut parts = text.splitn(3, '|');
    let name = parts.next()?;
    let code = parts.next()?.parse().ok()?;
    let message = parts.next()?.to_string();
    Some(OpfsError {
        name: name.to_string(),
        code,
        message,
    })
}

/// 名称校验（spec FS：空串、`.`、`..`、含 `/` → 无效，接线层抛 TypeError）。
pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('/')
}

/// 目录条目（`entries()` 迭代项）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpfsEntry {
    /// 条目名。
    pub name: String,
    /// 是否为目录。
    pub is_directory: bool,
}

/// 文件元数据（`getFile()` 的 Blob 视图初始化数据）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpfsFileMetadata {
    /// 文件名。
    pub name: String,
    /// 内容字节数。
    pub size: usize,
    /// 最后修改时间（Unix 毫秒）。
    pub last_modified: i64,
    /// 内容快照。
    pub data: Vec<u8>,
}

/// 可写流写命令（spec FS §FileSystemWritableFileStream，对应 JS 侧
/// `write(data)` / `write({type:'write'|'seek'|'truncate', ...})` 与
/// `seek()` / `truncate()` 方法）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpfsWriteCommand {
    /// 在当前/指定位置写入数据；显式 position 时**不**前进指针（spec：仅无 position
    /// 的 write 更新 seek offset）。
    Write {
        /// 指定写入位置（None = 当前指针）。
        position: Option<u64>,
        /// 写入字节。
        data: Vec<u8>,
    },
    /// 移动文件指针。
    Seek {
        /// 新指针位置。
        position: u64,
    },
    /// 截断/零扩展到指定大小；指针超过 size 时钳到 size（WPT cursor-position 断言）。
    Truncate {
        /// 目标大小。
        size: u64,
    },
}

/// 打开中的可写流状态。
#[derive(Debug)]
struct OpfsStream {
    path: Vec<String>,
    /// 写缓冲（`keep_existing_data` 时以原文件内容为初始快照）。
    buffer: Vec<u8>,
    position: u64,
    /// close 成功提交过（spec：仅一个 close 可成功）。
    committed: bool,
    /// abort 过（此后 close reject InvalidStateError）。
    aborted: bool,
}

/// OPFS 文件系统 — 每个站点源（origin）一棵树 + 打开流槽。
///
/// 句柄 = 路径（值语义，可跨 host 命令边界序列化）；节点身份由路径同一性表达。
#[derive(Debug, Default)]
pub struct OpfsFileSystem {
    root: OpfsNode,
    /// 路径 → 打开中的可写流数量（removeEntry 语义守卫）。
    open_writers: BTreeMap<Vec<String>, usize>,
    /// 打开流槽。
    streams: BTreeMap<u64, OpfsStream>,
    next_stream_id: u64,
}

impl OpfsFileSystem {
    /// 创建空文件系统（内存态；持久化由 [`OpfsPersistence`] 负责）。
    pub fn new() -> Self {
        Self::default()
    }

    /// `navigator.storage.getDirectory()` — 根目录句柄路径（恒 `[]`）。
    pub fn root_path(&self) -> Vec<String> {
        Vec::new()
    }

    /// 用量统计（`estimate()` 真实化：全部文件字节和）。
    pub fn usage_bytes(&self) -> u64 {
        fn walk(node: &OpfsNode) -> u64 {
            match node {
                OpfsNode::Dir(children) => children.values().map(walk).sum(),
                OpfsNode::File(data) => data.bytes.len() as u64,
            }
        }
        walk(&self.root)
    }

    /// 从持久化文件表重建（[`OpfsPersistence::open`] 恢复路径）。
    pub fn from_persisted_files(files: &PersistedFileMap) -> Self {
        Self {
            root: persistence::tree_from_files(files),
            ..Self::default()
        }
    }

    /// 提取扁平文件表（[`persistence`] 写盘路径的交换格式）。
    pub fn persisted_files(&self) -> PersistedFileMap {
        persistence::flatten_filesystem(self)
    }

    /// 树是否为空（无任何文件；空树落盘时等价删除）。
    pub fn is_empty(&self) -> bool {
        self.persisted_files().is_empty()
    }

    /// 按路径解析节点。
    fn resolve<'a>(&'a self, path: &[String]) -> Option<&'a OpfsNode> {
        let mut current = &self.root;
        for segment in path {
            let OpfsNode::Dir(children) = current else {
                return None;
            };
            current = children.get(segment)?;
        }
        Some(current)
    }

    /// 下沉到 parent 的可变引用（父链必须已存在）。
    fn dir_mut(&mut self, parent: &[String]) -> &mut BTreeMap<String, OpfsNode> {
        let mut current = &mut self.root;
        for segment in parent {
            current = match current {
                OpfsNode::Dir(children) => children.get_mut(segment).expect("parent path must exist"),
                OpfsNode::File(_) => panic!("parent path resolved through a file"),
            };
        }
        match current {
            OpfsNode::Dir(children) => children,
            OpfsNode::File(_) => panic!("target path resolved through a file"),
        }
    }

    /// `getFileHandle(parent, name, {create})` — 文件句柄（成功返回句柄路径）。
    pub fn get_file_handle(&mut self, parent: &[String], name: &str, create: bool) -> Result<Vec<String>, OpfsError> {
        if !is_valid_name(name) {
            return Err(OpfsError::type_error(format!("无效的文件名 {name:?}")));
        }
        let mut path = parent.to_vec();
        path.push(name.to_string());
        match self.resolve(&path) {
            Some(node) if node.is_dir() => Err(OpfsError::type_mismatch(format!("{name} 是目录"))),
            Some(_) => Ok(path),
            None if !create => Err(OpfsError::not_found(format!("{name} 不存在"))),
            None => {
                self.dir_mut(parent)
                    .insert(name.to_string(), OpfsNode::File(OpfsFileData::default()));
                Ok(path)
            }
        }
    }

    /// `getDirectoryHandle(parent, name, {create})` — 子目录句柄。
    pub fn get_directory_handle(
        &mut self,
        parent: &[String],
        name: &str,
        create: bool,
    ) -> Result<Vec<String>, OpfsError> {
        if !is_valid_name(name) {
            return Err(OpfsError::type_error(format!("无效的目录名 {name:?}")));
        }
        let mut path = parent.to_vec();
        path.push(name.to_string());
        match self.resolve(&path) {
            Some(node) if node.is_file() => Err(OpfsError::type_mismatch(format!("{name} 是文件"))),
            Some(_) => Ok(path),
            None if !create => Err(OpfsError::not_found(format!("{name} 不存在"))),
            None => {
                self.dir_mut(parent)
                    .entry(name.to_string())
                    .or_insert_with(OpfsNode::new_dir);
                Ok(path)
            }
        }
    }

    /// `removeEntry(parent, name, {recursive})` — 删除子项。
    ///
    /// 非空目录非 recursive → InvalidModificationError；目标（或子树）有打开流 →
    /// NoModificationAllowedError；不存在 → NotFoundError。
    pub fn remove_entry(&mut self, parent: &[String], name: &str, recursive: bool) -> Result<(), OpfsError> {
        if !is_valid_name(name) {
            return Err(OpfsError::type_error(format!("无效的名称 {name:?}")));
        }
        let mut path = parent.to_vec();
        path.push(name.to_string());
        let node = self
            .resolve(&path)
            .ok_or_else(|| OpfsError::not_found(format!("{name} 不存在")))?;
        if node.is_dir() {
            let non_empty = matches!(node, OpfsNode::Dir(children) if !children.is_empty());
            if non_empty && !recursive {
                return Err(OpfsError::invalid_modification(format!(
                    "{name} 目录非空（需 recursive）"
                )));
            }
        }
        self.ensure_no_open_writer(&path)?;
        self.dir_mut(parent).remove(name);
        Ok(())
    }

    /// `FileSystemHandle.remove({recursive})` — 句柄自删（文件或目录）。
    pub fn remove(&mut self, path: &[String], recursive: bool) -> Result<(), OpfsError> {
        if path.is_empty() {
            // 根不可删（spec：沙箱根 remove 是 Chromium 扩展；本实现拒绝）。
            return Err(OpfsError::invalid_modification("不能删除根目录"));
        }
        let (parent, name) = path.split_at(path.len() - 1);
        self.remove_entry(parent, &name[0].clone(), recursive)
    }

    /// `resolve(dir_path, child_path)` — child 相对 dir 的路径；非后代 → None。
    pub fn resolve_path(&self, dir: &[String], child: &[String]) -> Option<Vec<String>> {
        if child.len() < dir.len() || !child.starts_with(dir) {
            return None;
        }
        Some(child[dir.len()..].to_vec())
    }

    /// 目录子项名（迭代序 = 字典序）。
    pub fn keys(&self, dir: &[String]) -> Vec<String> {
        match self.resolve(dir) {
            Some(OpfsNode::Dir(children)) => children.keys().cloned().collect(),
            _ => Vec::new(),
        }
    }

    /// 目录条目（[名, kind]）。
    pub fn entries(&self, dir: &[String]) -> Vec<OpfsEntry> {
        match self.resolve(dir) {
            Some(OpfsNode::Dir(children)) => children
                .iter()
                .map(|(name, node)| OpfsEntry {
                    name: name.clone(),
                    is_directory: node.is_dir(),
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    /// `getFile(path)` — 元数据 + 内容快照。
    pub fn get_file(&self, path: &[String]) -> Result<OpfsFileMetadata, OpfsError> {
        let node = self.resolve(path).ok_or_else(|| OpfsError::not_found("文件不存在"))?;
        let OpfsNode::File(data) = node else {
            return Err(OpfsError::type_mismatch("路径是目录"));
        };
        Ok(OpfsFileMetadata {
            name: path.last().cloned().unwrap_or_default(),
            size: data.bytes.len(),
            last_modified: data.last_modified,
            data: data.bytes.clone(),
        })
    }

    /// `createWritable(path, {keepExistingData})` — 打开流槽，返回流 ID。
    pub fn create_writable(&mut self, path: &[String], keep_existing_data: bool) -> Result<u64, OpfsError> {
        let buffer = match self.resolve(path) {
            Some(OpfsNode::File(data)) if keep_existing_data => data.bytes.clone(),
            Some(OpfsNode::File(_)) => Vec::new(),
            Some(OpfsNode::Dir(_)) => return Err(OpfsError::type_mismatch("路径是目录")),
            None => Vec::new(),
        };
        *self.open_writers.entry(path.to_vec()).or_insert(0) += 1;
        let id = self.next_stream_id;
        self.next_stream_id += 1;
        self.streams.insert(
            id,
            OpfsStream {
                path: path.to_vec(),
                buffer,
                position: 0,
                committed: false,
                aborted: false,
            },
        );
        Ok(id)
    }

    /// 流写命令执行。
    pub fn stream_write(&mut self, stream_id: u64, command: OpfsWriteCommand) -> Result<(), OpfsError> {
        let stream = self
            .streams
            .get_mut(&stream_id)
            .ok_or_else(|| OpfsError::invalid_state("stream 不存在"))?;
        if stream.committed || stream.aborted {
            return Err(OpfsError::invalid_state("stream 已关闭"));
        }
        match command {
            OpfsWriteCommand::Write { position, data } => {
                let write_at = position.unwrap_or(stream.position);
                let end = (write_at as usize).saturating_add(data.len());
                if end > stream.buffer.len() {
                    stream.buffer.resize(end, 0);
                }
                stream.buffer[write_at as usize..end].copy_from_slice(&data);
                if position.is_none() {
                    stream.position = write_at + data.len() as u64;
                }
            }
            OpfsWriteCommand::Seek { position } => {
                stream.position = position;
            }
            OpfsWriteCommand::Truncate { size } => {
                stream.buffer.resize(size as usize, 0);
                if stream.position > size {
                    stream.position = size;
                }
            }
        }
        Ok(())
    }

    /// `close()` — 原子提交缓冲到节点（仅一个 close 成功；abort 后 reject）。
    pub fn stream_close(&mut self, stream_id: u64) -> Result<(), OpfsError> {
        let stream = self
            .streams
            .get_mut(&stream_id)
            .ok_or_else(|| OpfsError::invalid_state("stream 不存在"))?;
        if stream.aborted {
            return Err(OpfsError::invalid_state("stream 已 abort"));
        }
        if stream.committed {
            return Err(OpfsError::invalid_state("stream 已关闭"));
        }
        stream.committed = true;
        let (path, buffer) = (stream.path.clone(), std::mem::take(&mut stream.buffer));
        let (parent, name) = path.split_at(path.len() - 1);
        let last_modified = unix_millis();
        self.dir_mut(parent).insert(
            name[0].clone(),
            OpfsNode::File(OpfsFileData {
                bytes: buffer,
                last_modified,
            }),
        );
        self.release_writer(&path);
        self.streams.remove(&stream_id);
        Ok(())
    }

    /// `abort()` — 放弃缓冲（不写回；此后 close reject）。
    pub fn stream_abort(&mut self, stream_id: u64) -> Result<(), OpfsError> {
        let stream = self
            .streams
            .get_mut(&stream_id)
            .ok_or_else(|| OpfsError::invalid_state("stream 不存在"))?;
        if stream.committed {
            return Err(OpfsError::invalid_state("stream 已关闭"));
        }
        stream.aborted = true;
        let path = stream.path.clone();
        self.release_writer(&path);
        self.streams.remove(&stream_id);
        Ok(())
    }

    /// 流是否打开（接线层 guard）。
    pub fn stream_is_open(&self, stream_id: u64) -> bool {
        self.streams.contains_key(&stream_id)
    }

    /// 目标（及其子树）不得有打开流。
    fn ensure_no_open_writer(&self, path: &[String]) -> Result<(), OpfsError> {
        let blocked = self
            .open_writers
            .iter()
            .any(|(writer_path, _)| writer_path == path || writer_path.starts_with(path));
        if blocked {
            let name = path.last().map(String::as_str).unwrap_or("");
            return Err(OpfsError::no_modification_allowed(format!("{name} 有打开中的可写流")));
        }
        Ok(())
    }

    /// 递减路径的打开流计数（归零移除）。
    fn release_writer(&mut self, path: &[String]) {
        if let Some(count) = self.open_writers.get_mut(path) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.open_writers.remove(path);
            }
        }
    }
}

/// 生成 UUID v4 字符串（`getUniqueId()`；RFC 4122）。
pub fn generate_unique_id() -> String {
    let mut bytes = [0u8; 16];
    let _ = getrandom::fill(&mut bytes);
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // variant 10
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// Unix 当前毫秒。
fn unix_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 名称校验（spec：空串/`.`/`..`/含 `/` 无效）。
    #[test]
    fn test_is_valid_name() {
        assert!(is_valid_name("a.txt"));
        assert!(is_valid_name("目录😊"));
        assert!(!is_valid_name(""));
        assert!(!is_valid_name("."));
        assert!(!is_valid_name(".."));
        assert!(!is_valid_name("a/b"));
    }

    /// getFileHandle(create:true) 建空文件；getFile 元数据（name/size/lastModified）。
    #[test]
    fn test_get_file_handle_create_and_get_file() {
        let mut fs = OpfsFileSystem::new();
        let file = fs.get_file_handle(&[], "log.txt", true).unwrap();
        let meta = fs.get_file(&file).unwrap();
        assert_eq!(meta.name, "log.txt");
        assert_eq!(meta.size, 0);
        assert_eq!(meta.last_modified, 0);
    }

    /// getFileHandle(create:false) 不存在 → NotFoundError(8)（WPT 聚类 2）。
    #[test]
    fn test_get_file_handle_not_found() {
        let mut fs = OpfsFileSystem::new();
        let error = fs.get_file_handle(&[], "non-existing-file", false).unwrap_err();
        assert_eq!(error.name, "NotFoundError");
        assert_eq!(error.code, 8);
    }

    /// 同名冲突 → TypeMismatchError(17)（WPT 聚类 2；file/dir 双向）。
    #[test]
    fn test_handle_kind_conflict_type_mismatch() {
        let mut fs = OpfsFileSystem::new();
        // 目录占名 dir-name：期望文件处是目录。
        fs.get_directory_handle(&[], "dir-name", true).unwrap();
        let error = fs.get_file_handle(&[], "dir-name", false).unwrap_err();
        assert_eq!(error.name, "TypeMismatchError");
        assert_eq!(error.code, 17);
        let error = fs.get_file_handle(&[], "dir-name", true).unwrap_err();
        assert_eq!(error.name, "TypeMismatchError");
        // 文件占名 file-name：期望目录处是文件。
        fs.get_file_handle(&[], "file-name", true).unwrap();
        let error = fs.get_directory_handle(&[], "file-name", false).unwrap_err();
        assert_eq!(error.name, "TypeMismatchError");
        assert_eq!(error.code, 17);
    }

    /// 写读往返 + close 后 getFile 内容一致（WPT 聚类 6/7 对照）。
    #[test]
    fn test_writable_write_read_roundtrip() {
        let mut fs = OpfsFileSystem::new();
        let file = fs.get_file_handle(&[], "log.txt", true).unwrap();
        let stream = fs.create_writable(&file, false).unwrap();
        fs.stream_write(
            stream,
            OpfsWriteCommand::Write {
                position: None,
                data: b"hello OPFS".to_vec(),
            },
        )
        .unwrap();
        fs.stream_close(stream).unwrap();
        let meta = fs.get_file(&file).unwrap();
        assert_eq!(meta.data, b"hello OPFS");
        assert_eq!(meta.size, 10);
        assert!(meta.last_modified > 0);
    }

    /// keepExistingData:true → 原内容为初始缓冲；false → 整体替换（WPT「bar」vs「very long string」）。
    #[test]
    fn test_writable_keep_existing_data() {
        let mut fs = OpfsFileSystem::new();
        let file = fs.get_file_handle(&[], "f.txt", true).unwrap();
        let s1 = fs.create_writable(&file, false).unwrap();
        fs.stream_write(
            s1,
            OpfsWriteCommand::Write {
                position: None,
                data: b"barks".to_vec(),
            },
        )
        .unwrap();
        fs.stream_close(s1).unwrap();
        // keepExistingData:false → 写 3 字节后整体替换为 3 字节。
        let s2 = fs.create_writable(&file, false).unwrap();
        fs.stream_write(
            s2,
            OpfsWriteCommand::Write {
                position: None,
                data: b"bar".to_vec(),
            },
        )
        .unwrap();
        fs.stream_close(s2).unwrap();
        assert_eq!(fs.get_file(&file).unwrap().data, b"bar");
        // keepExistingData:true → 覆盖前缀，长度不变（barks → bars）。
        let s3 = fs.create_writable(&file, true).unwrap();
        fs.stream_write(
            s3,
            OpfsWriteCommand::Write {
                position: None,
                data: b"bars".to_vec(),
            },
        )
        .unwrap();
        fs.stream_close(s3).unwrap();
        assert_eq!(fs.get_file(&file).unwrap().data, b"bars");
    }

    /// truncate 钳指针语义（WPT cursor-position 断言）：
    /// truncate(5)+write("abc") → "abc45"；seek(6)+truncate(5)+write("abc") → "12345abc"。
    #[test]
    fn test_writable_truncate_cursor_position() {
        let mut fs = OpfsFileSystem::new();
        let file = fs.get_file_handle(&[], "t.txt", true).unwrap();
        // truncate size > offset：内容 "1234567890"（pos=0）truncate(5) 后 write("abc")
        // 在 0 写 → "abc45"。
        let s = fs.create_writable(&file, true).unwrap();
        fs.stream_write(
            s,
            OpfsWriteCommand::Write {
                position: None,
                data: b"1234567890".to_vec(),
            },
        )
        .unwrap();
        fs.stream_close(s).unwrap();
        let s = fs.create_writable(&file, true).unwrap();
        fs.stream_write(s, OpfsWriteCommand::Truncate { size: 5 }).unwrap();
        fs.stream_write(
            s,
            OpfsWriteCommand::Write {
                position: None,
                data: b"abc".to_vec(),
            },
        )
        .unwrap();
        fs.stream_close(s).unwrap();
        assert_eq!(fs.get_file(&file).unwrap().data, b"abc45");
        // truncate size < offset：重置内容 "1234567890"，seek(6)+truncate(5)（pos 钳到 5）
        // + write("abc") → "12345abc"。
        let s = fs.create_writable(&file, false).unwrap();
        fs.stream_write(
            s,
            OpfsWriteCommand::Write {
                position: None,
                data: b"1234567890".to_vec(),
            },
        )
        .unwrap();
        fs.stream_close(s).unwrap();
        let s = fs.create_writable(&file, true).unwrap();
        fs.stream_write(s, OpfsWriteCommand::Seek { position: 6 }).unwrap();
        fs.stream_write(s, OpfsWriteCommand::Truncate { size: 5 }).unwrap();
        fs.stream_write(
            s,
            OpfsWriteCommand::Write {
                position: None,
                data: b"abc".to_vec(),
            },
        )
        .unwrap();
        fs.stream_close(s).unwrap();
        assert_eq!(fs.get_file(&file).unwrap().data, b"12345abc");
    }

    /// 无 position 的 write 前进指针；显式 position 不前进（spec seek offset 语义）。
    #[test]
    fn test_writable_position_semantics() {
        let mut fs = OpfsFileSystem::new();
        let file = fs.get_file_handle(&[], "p.txt", true).unwrap();
        let s = fs.create_writable(&file, false).unwrap();
        fs.stream_write(
            s,
            OpfsWriteCommand::Write {
                position: None,
                data: b"abc".to_vec(),
            },
        )
        .unwrap();
        // 显式 position 写入不前进指针：在 0 写 "xyz" → 内容 "xyc"。
        fs.stream_write(
            s,
            OpfsWriteCommand::Write {
                position: Some(0),
                data: b"xyz".to_vec(),
            },
        )
        .unwrap();
        // 指针仍在 3：继续写 "def" → "xyzdef"。
        fs.stream_write(
            s,
            OpfsWriteCommand::Write {
                position: None,
                data: b"def".to_vec(),
            },
        )
        .unwrap();
        fs.stream_close(s).unwrap();
        assert_eq!(fs.get_file(&file).unwrap().data, b"xyzdef");
    }

    /// close 后再写 → InvalidStateError(11)；双 close 只一个成功（WPT「only one close」）。
    #[test]
    fn test_writable_close_semantics() {
        let mut fs = OpfsFileSystem::new();
        let file = fs.get_file_handle(&[], "c.txt", true).unwrap();
        let stream = fs.create_writable(&file, false).unwrap();
        fs.stream_close(stream).unwrap();
        let error = fs
            .stream_write(
                stream,
                OpfsWriteCommand::Write {
                    position: None,
                    data: b"x".to_vec(),
                },
            )
            .unwrap_err();
        assert_eq!(error.name, "InvalidStateError");
        assert_eq!(error.code, 11);
        let error = fs.stream_close(stream).unwrap_err();
        assert_eq!(error.name, "InvalidStateError");
    }

    /// abort 后 close reject；缓冲不落盘。
    #[test]
    fn test_writable_abort_discards() {
        let mut fs = OpfsFileSystem::new();
        let file = fs.get_file_handle(&[], "a.txt", true).unwrap();
        let stream = fs.create_writable(&file, false).unwrap();
        fs.stream_write(
            stream,
            OpfsWriteCommand::Write {
                position: None,
                data: b"discard".to_vec(),
            },
        )
        .unwrap();
        fs.stream_abort(stream).unwrap();
        let error = fs.stream_close(stream).unwrap_err();
        assert_eq!(error.name, "InvalidStateError");
        assert_eq!(fs.get_file(&file).unwrap().size, 0);
        // abort 释放打开流计数：removeEntry 不再受阻。
        fs.remove_entry(&[], "a.txt", false).unwrap();
    }

    /// removeEntry：文件删除 + 不存在 → NotFoundError；非空目录非 recursive →
    /// InvalidModificationError(13)；recursive 删除子树（WPT 聚类 2/3 对照）。
    #[test]
    fn test_remove_entry_semantics() {
        let mut fs = OpfsFileSystem::new();
        fs.get_file_handle(&[], "file-to-remove", true).unwrap();
        fs.remove_entry(&[], "file-to-remove", false).unwrap();
        let error = fs.remove_entry(&[], "file-to-remove", false).unwrap_err();
        assert_eq!(error.name, "NotFoundError");
        assert_eq!(error.code, 8);
        // 非空目录。
        let dir = fs.get_directory_handle(&[], "dir", true).unwrap();
        fs.get_file_handle(&dir, "inner.txt", true).unwrap();
        let error = fs.remove_entry(&[], "dir", false).unwrap_err();
        assert_eq!(error.name, "InvalidModificationError");
        assert_eq!(error.code, 13);
        // recursive 删除子树。
        fs.remove_entry(&[], "dir", true).unwrap();
        assert_eq!(fs.entries(&[]).len(), 0);
    }

    /// 有打开流的文件 removeEntry → NoModificationAllowedError(7)（WPT「while open writable fails」）。
    #[test]
    fn test_remove_entry_with_open_writer() {
        let mut fs = OpfsFileSystem::new();
        let file = fs.get_file_handle(&[], "busy.txt", true).unwrap();
        let stream = fs.create_writable(&file, false).unwrap();
        let error = fs.remove_entry(&[], "busy.txt", false).unwrap_err();
        assert_eq!(error.name, "NoModificationAllowedError");
        assert_eq!(error.code, 7);
        fs.stream_close(stream).unwrap();
        fs.remove_entry(&[], "busy.txt", false).unwrap();
    }

    /// 目录迭代：keys/entries 字典序（WPT sort 后比较）+ resolve 路径语义。
    #[test]
    fn test_directory_iteration_and_resolve() {
        let mut fs = OpfsFileSystem::new();
        fs.get_file_handle(&[], "b.txt", true).unwrap();
        fs.get_file_handle(&[], "a.txt", true).unwrap();
        let dir = fs.get_directory_handle(&[], "sub", true).unwrap();
        assert_eq!(
            fs.keys(&[]),
            vec!["a.txt".to_string(), "b.txt".to_string(), "sub".to_string()]
        );
        let entries = fs.entries(&[]);
        assert_eq!(entries.len(), 3);
        assert!(!entries[0].is_directory);
        assert!(entries[2].is_directory);
        // resolve：根 → [sub]；sub → ["sub"]（自）；非后代 → None。
        assert_eq!(fs.resolve_path(&[], &dir), Some(vec!["sub".to_string()]));
        assert_eq!(fs.resolve_path(&dir, &dir), Some(vec![]));
        let file = fs.get_file_handle(&dir, "x.txt", true).unwrap();
        assert_eq!(
            fs.resolve_path(&[], &file),
            Some(vec!["sub".to_string(), "x.txt".to_string()])
        );
        assert_eq!(fs.resolve_path(&file, &dir), None);
    }

    /// 嵌套目录 create：getDirectoryHandle({create:true}) 逐层建立。
    #[test]
    fn test_nested_directory_create() {
        let mut fs = OpfsFileSystem::new();
        let sub = fs.get_directory_handle(&[], "parent", true).unwrap();
        let deeper = fs.get_directory_handle(&sub, "child", true).unwrap();
        let file = fs.get_file_handle(&deeper, "deep.txt", true).unwrap();
        assert_eq!(
            file,
            vec!["parent".to_string(), "child".to_string(), "deep.txt".to_string()]
        );
        assert_eq!(fs.get_file(&file).unwrap().name, "deep.txt");
    }

    /// 持久化往返：写盘 → open 恢复一致（per-origin 隔离 + 空树等价删除）。
    #[test]
    fn test_persistence_roundtrip() {
        let root = std::env::temp_dir().join(format!("zw-opfs-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut tree = BTreeMap::new();
        tree.insert(vec!["a.txt".to_string()], (b"alpha".to_vec(), 42i64));
        tree.insert(vec!["sub".to_string(), "b.txt".to_string()], (b"beta".to_vec(), 7i64));
        {
            let (persistence, loaded) = OpfsPersistence::open(&root).unwrap();
            assert!(loaded.is_empty());
            persistence.write("https://example.com", &tree).unwrap();
        }
        {
            let (persistence, loaded) = OpfsPersistence::open(&root).unwrap();
            let files = loaded.get("https://example.com").unwrap();
            assert_eq!(
                files.get(&vec!["a.txt".to_string()]).unwrap(),
                &(b"alpha".to_vec(), 42i64)
            );
            assert_eq!(
                files.get(&vec!["sub".to_string(), "b.txt".to_string()]).unwrap(),
                &(b"beta".to_vec(), 7i64)
            );
            // 空树写 → 删除落盘文件。
            persistence.write("https://example.com", &BTreeMap::new()).unwrap();
        }
        {
            let (_, loaded) = OpfsPersistence::open(&root).unwrap();
            assert!(loaded.is_empty());
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// 树 ↔ 扁平文件表往返（持久化交换格式）。
    #[test]
    fn test_tree_flatten_roundtrip() {
        let mut fs = OpfsFileSystem::new();
        let dir = fs.get_directory_handle(&[], "d", true).unwrap();
        let file = fs.get_file_handle(&dir, "x.txt", true).unwrap();
        let stream = fs.create_writable(&file, false).unwrap();
        fs.stream_write(
            stream,
            OpfsWriteCommand::Write {
                position: None,
                data: b"data".to_vec(),
            },
        )
        .unwrap();
        fs.stream_close(stream).unwrap();
        let flat = fs.persisted_files();
        assert_eq!(flat.len(), 1);
        let rebuilt = OpfsFileSystem::from_persisted_files(&flat);
        let restored = rebuilt.get_file(&file).unwrap();
        assert_eq!(restored.data, b"data");
        assert_eq!(rebuilt.usage_bytes(), fs.usage_bytes());
    }

    /// estimate 真实化：usage_bytes 按文件字节和统计。
    #[test]
    fn test_usage_bytes() {
        let mut fs = OpfsFileSystem::new();
        assert_eq!(fs.usage_bytes(), 0);
        let file = fs.get_file_handle(&[], "u.txt", true).unwrap();
        let stream = fs.create_writable(&file, false).unwrap();
        fs.stream_write(
            stream,
            OpfsWriteCommand::Write {
                position: None,
                data: vec![0u8; 100],
            },
        )
        .unwrap();
        fs.stream_close(stream).unwrap();
        assert_eq!(fs.usage_bytes(), 100);
    }

    /// getUniqueId：UUID v4 格式（WPT「GUID version 4 format」断言版本/变体位）。
    #[test]
    fn test_generate_unique_id_format() {
        let id = generate_unique_id();
        assert_eq!(id.len(), 36);
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(parts.iter().map(|p| p.len()).collect::<Vec<_>>(), vec![8, 4, 4, 4, 12]);
        assert!(parts[2].starts_with('4')); // version 4
        assert!(matches!(parts[3].chars().next(), Some('8' | '9' | 'a' | 'b'))); // variant 10
    }

    /// 错误编码往返：OpfsError ↔ StorageError::Type（host 命令边界透传）。
    #[test]
    fn test_error_storage_encoding_roundtrip() {
        let error = OpfsError::not_found("missing");
        let storage = error.clone().into_storage_error();
        let restored = opfs_error_from_storage(&storage).unwrap();
        assert_eq!(restored, error);
        assert!(opfs_error_from_storage(&StorageError::Io("plain io".into())).is_none());
    }
}

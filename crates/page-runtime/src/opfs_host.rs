//! OPFS 页面宿主 — `__zw_opfs` wire 请求的 dispatch（基于 zero-storage opfs 模块）。
//!
//! wire 契约（JSON）：
//! - 请求：`{"op": "...", ...}`，各 op 字段见 [`OpfsRequest`]；路径为段数组；
//!   文件字节为字节数组（IndexedDB binary key 同款 wire 约定）
//! - 响应：各 op 的 JSON 对象；流式写经 `streamId` 关联
//! - 错误：`Err("Name|code|message")`（zero-storage `OpfsError` 编码；接线 JS 侧
//!   映射为 DOMException；`TypeError|0|...` 映射 JS TypeError）

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use serde_json::json;

use zero_engine::OpfsHandler;
use zero_storage::{OpfsError, OpfsFileSystem, OpfsPersistence, OpfsWriteCommand, PersistedFileMap, StorageError};

/// 构造由页面运行路径共享的 OPFS handler。
///
/// `root` 为 `Some` 时 per-origin 落盘（打开时恢复 + 每次提交后写回）；
/// `None` 为纯内存（kill-switch 回退路径 / 私密浏览）。
pub fn opfs_handler(root: Option<std::path::PathBuf>) -> OpfsHandler {
    let state = match root {
        Some(path) => match OpfsPersistence::open(&path) {
            Ok((persistence, filesystems)) => Arc::new(Mutex::new(OpfsHostState::persistent(persistence, filesystems))),
            Err(_error) => {
                // 持久化层不可用 → 磁盘错误不 panic，降级内存态（照 cache_api 模式）。
                // 持久化不可用为可预期环境状态（只读盘/权限）；静默降级内存态（wire 层照常工作）。
                Arc::new(Mutex::new(OpfsHostState::in_memory()))
            }
        },
        None => Arc::new(Mutex::new(OpfsHostState::in_memory())),
    };
    Arc::new(move |origin, request| handle_request(&state, origin, request))
}

/// 单个 origin 的 OPFS 运行态：文件系统 + 持久化 owner（可选）。
struct OpfsOriginState {
    filesystem: OpfsFileSystem,
    dirty: bool,
}

/// 宿主全局态：per-origin 文件系统 + 持久化 owner。
struct OpfsHostState {
    origins: BTreeMap<String, OpfsOriginState>,
    persistence: Option<OpfsPersistence>,
}

impl OpfsHostState {
    fn in_memory() -> Self {
        Self {
            origins: BTreeMap::new(),
            persistence: None,
        }
    }

    fn persistent(persistence: OpfsPersistence, filesystems: BTreeMap<String, PersistedFileMap>) -> Self {
        let origins = filesystems
            .into_iter()
            .map(|(origin, files)| {
                (
                    origin,
                    OpfsOriginState {
                        filesystem: OpfsFileSystem::from_persisted_files(&files),
                        dirty: false,
                    },
                )
            })
            .collect();
        Self {
            origins,
            persistence: Some(persistence),
        }
    }

    fn origin_mut(&mut self, origin: &str) -> &mut OpfsOriginState {
        self.origins
            .entry(origin.to_string())
            .or_insert_with(|| OpfsOriginState {
                filesystem: OpfsFileSystem::new(),
                dirty: false,
            })
    }

    /// 提交脏 origin 到持久化层（磁盘错误不 panic，返回 Err 由 wire 层上报）。
    fn flush(&mut self, origin: &str) -> Result<(), OpfsError> {
        let Some(persistence) = &self.persistence else {
            return Ok(());
        };
        let Some(state) = self.origins.get(origin) else {
            return Ok(());
        };
        if !state.dirty {
            return Ok(());
        }
        let tree = state.filesystem.persisted_files();
        persistence.write(origin, &tree).map_err(persistence_error)?;
        if let Some(state) = self.origins.get_mut(origin) {
            state.dirty = false;
        }
        Ok(())
    }
}

/// OPFS wire 请求。
#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
enum OpfsRequest {
    /// `getDirectory()` — 根目录句柄。
    GetDirectory,
    /// `getFileHandle(parent, name, {create})`。
    #[serde(rename_all = "camelCase")]
    GetFileHandle {
        parent: Vec<String>,
        name: String,
        create: bool,
    },
    /// `getDirectoryHandle(parent, name, {create})`。
    #[serde(rename_all = "camelCase")]
    GetDirectoryHandle {
        parent: Vec<String>,
        name: String,
        create: bool,
    },
    /// `removeEntry(parent, name, {recursive})`。
    #[serde(rename_all = "camelCase")]
    RemoveEntry {
        parent: Vec<String>,
        name: String,
        recursive: bool,
    },
    /// `FileSystemHandle.remove({recursive})`（句柄自删）。
    #[serde(rename_all = "camelCase")]
    Remove { path: Vec<String>, recursive: bool },
    /// `resolve(dir, child)`。
    #[serde(rename_all = "camelCase")]
    Resolve { dir: Vec<String>, child: Vec<String> },
    /// `keys(dir)` / `entries(dir)`。
    #[serde(rename_all = "camelCase")]
    ListEntries { dir: Vec<String> },
    /// `getFile(path)`。
    #[serde(rename_all = "camelCase")]
    GetFile { path: Vec<String> },
    /// `createWritable(path, {keepExistingData})` → streamId。
    #[serde(rename_all = "camelCase")]
    CreateWritable {
        path: Vec<String>,
        keep_existing_data: bool,
    },
    /// 流写命令（write/seek/truncate）。
    #[serde(rename_all = "camelCase")]
    StreamWrite { stream_id: u64, command: StreamCommandWire },
    /// 流 close（提交）。
    #[serde(rename_all = "camelCase")]
    StreamClose { stream_id: u64 },
    /// 流 abort（弃缓冲）。
    #[serde(rename_all = "camelCase")]
    StreamAbort { stream_id: u64 },
    /// `estimate()` 用量。
    Estimate,
    /// `getUniqueId(path)` — 句柄稳定 ID。
    GetUniqueId { path: Vec<String> },
}

/// 流写命令 wire 形态。
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum StreamCommandWire {
    /// 写入（position 缺省 = 当前指针）。
    Write { position: Option<u64>, data: Vec<u8> },
    /// 移动指针。
    Seek { position: u64 },
    /// 截断/零扩展。
    Truncate { size: u64 },
}

fn handle_request(state: &Arc<Mutex<OpfsHostState>>, origin: &str, request: &str) -> Result<String, String> {
    if origin == "null" {
        return Err("SecurityError|0|OPFS is unavailable for opaque origins".to_string());
    }
    let request: OpfsRequest =
        serde_json::from_str(request).map_err(|error| format!("TypeError|0|invalid OPFS request: {error}"))?;
    let mut state = state
        .lock()
        .map_err(|_| "UnknownError|0|OPFS state lock is poisoned".to_string())?;
    let response = dispatch_request(&mut state, origin, request)?;
    state.flush(origin).map_err(opfs_error_string)?;
    serde_json::to_string(&response).map_err(|error| format!("UnknownError|0|failed to serialize response: {error}"))
}

fn dispatch_request(
    state: &mut OpfsHostState,
    origin: &str,
    request: OpfsRequest,
) -> Result<serde_json::Value, String> {
    // 写 op 清单：成功后置 dirty（下一次 flush 落盘）。GetDirectory/读面不置脏。
    let is_write = matches!(
        request,
        OpfsRequest::GetFileHandle { create: true, .. }
            | OpfsRequest::GetDirectoryHandle { create: true, .. }
            | OpfsRequest::RemoveEntry { .. }
            | OpfsRequest::Remove { .. }
            | OpfsRequest::StreamClose { .. }
            | OpfsRequest::StreamAbort { .. }
    );
    let filesystem = &mut state.origin_mut(origin).filesystem;
    let result = match request {
        OpfsRequest::GetDirectory => Ok(json!({ "path": filesystem.root_path() })),
        OpfsRequest::GetFileHandle { parent, name, create } => {
            let path = filesystem
                .get_file_handle(&parent, &name, create)
                .map_err(opfs_error_string)?;
            Ok(json!({ "path": path, "kind": "file" }))
        }
        OpfsRequest::GetDirectoryHandle { parent, name, create } => {
            let path = filesystem
                .get_directory_handle(&parent, &name, create)
                .map_err(opfs_error_string)?;
            Ok(json!({ "path": path, "kind": "directory" }))
        }
        OpfsRequest::RemoveEntry {
            parent,
            name,
            recursive,
        } => {
            filesystem
                .remove_entry(&parent, &name, recursive)
                .map_err(opfs_error_string)?;
            Ok(json!({ "ok": true }))
        }
        OpfsRequest::Remove { path, recursive } => {
            filesystem.remove(&path, recursive).map_err(opfs_error_string)?;
            Ok(json!({ "ok": true }))
        }
        OpfsRequest::Resolve { dir, child } => {
            let resolved = filesystem.resolve_path(&dir, &child);
            Ok(json!({ "path": resolved }))
        }
        OpfsRequest::ListEntries { dir } => {
            // 已删除/不存在的目录迭代 → NotFoundError（WPT 对被移除句柄继续
            // getSortedDirectoryEntries 断言 NotFoundError；根路径恒合法）。
            if !dir.is_empty() && !filesystem.is_directory(&dir) && filesystem.get_file(&dir).is_err() {
                return Err(opfs_error_string(OpfsError::not_found("目录不存在")));
            }
            let entries = filesystem.entries(&dir);
            Ok(json!({
                "keys": filesystem.keys(&dir),
                "entries": entries
                    .into_iter()
                    .map(|entry| json!({"name": entry.name, "kind": if entry.is_directory { "directory" } else { "file" }}))
                    .collect::<Vec<_>>(),
            }))
        }
        OpfsRequest::GetFile { path } => {
            let file = filesystem.get_file(&path).map_err(opfs_error_string)?;
            Ok(json!({
                "name": file.name,
                "size": file.size,
                "lastModified": file.last_modified,
                "data": file.data,
            }))
        }
        OpfsRequest::CreateWritable {
            path,
            keep_existing_data,
        } => {
            let stream_id = filesystem
                .create_writable(&path, keep_existing_data)
                .map_err(opfs_error_string)?;
            Ok(json!({ "streamId": stream_id }))
        }
        OpfsRequest::StreamWrite { stream_id, command } => {
            let command = match command {
                StreamCommandWire::Write { position, data } => OpfsWriteCommand::Write { position, data },
                StreamCommandWire::Seek { position } => OpfsWriteCommand::Seek { position },
                StreamCommandWire::Truncate { size } => OpfsWriteCommand::Truncate { size },
            };
            filesystem.stream_write(stream_id, command).map_err(opfs_error_string)?;
            Ok(json!({ "ok": true }))
        }
        OpfsRequest::StreamClose { stream_id } => {
            filesystem.stream_close(stream_id).map_err(opfs_error_string)?;
            Ok(json!({ "ok": true }))
        }
        OpfsRequest::StreamAbort { stream_id } => {
            filesystem.stream_abort(stream_id).map_err(opfs_error_string)?;
            Ok(json!({ "ok": true }))
        }
        OpfsRequest::Estimate => Ok(json!({ "usage": filesystem.usage_bytes() })),
        OpfsRequest::GetUniqueId { path } => {
            // 句柄唯一 ID：Rust 侧仅生成新 GUID；「同路径同 ID / 写后不变」由调用方
            // JS 按路径键控缓存。存在性 = 文件节点，或目录节点（含空目录/根）。
            let exists = filesystem.get_file(&path).is_ok() || filesystem.is_directory(&path);
            if !exists {
                return Err(opfs_error_string(OpfsError::not_found("路径不存在")));
            }
            Ok(json!({ "uniqueId": zero_storage::generate_unique_id() }))
        }
    };
    if is_write && result.is_ok() {
        state.origin_mut(origin).dirty = true;
    }
    result
}

/// 持久化层 I/O 错误 → UnknownError wire 形态。
fn persistence_error(error: StorageError) -> OpfsError {
    OpfsError {
        name: "UnknownError".to_string(),
        code: 0,
        message: format!("OPFS persistence failed: {error}"),
    }
}

/// [`OpfsError`] → wire 错误串（`Name|code|message`）。
fn opfs_error_string(error: OpfsError) -> String {
    format!("{}|{}|{}", error.name, error.code, error.message)
}

/// 从 wire 错误串还原 [`OpfsError`]（opfs_error_from_storage 的 wire 对偶；测试用）。
#[allow(dead_code)]
fn opfs_error_from_wire(text: &str) -> Option<OpfsError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn call(handler: &OpfsHandler, origin: &str, request: &str) -> Result<serde_json::Value, String> {
        let text = handler(origin, request)?;
        serde_json::from_str(&text).map_err(|error| format!("invalid response JSON: {error}"))
    }

    #[test]
    fn opaque_origin_is_rejected() {
        let handler = opfs_handler(None);
        let error = handler("null", r#"{"op":"getDirectory"}"#).unwrap_err();
        assert!(error.starts_with("SecurityError"), "{error}");
    }

    #[test]
    fn get_directory_returns_root_path() {
        let handler = opfs_handler(None);
        let response = call(&handler, "https://example.com", r#"{"op":"getDirectory"}"#).unwrap();
        assert_eq!(response["path"], serde_json::json!([]));
    }

    #[test]
    fn file_roundtrip_in_memory() {
        let handler = opfs_handler(None);
        let origin = "https://example.com";
        let file = call(
            &handler,
            origin,
            r#"{"op":"getFileHandle","parent":[],"name":"log.txt","create":true}"#,
        )
        .unwrap();
        assert_eq!(file["kind"], "file");
        let stream = call(
            &handler,
            origin,
            r#"{"op":"createWritable","path":["log.txt"],"keepExistingData":false}"#,
        )
        .unwrap();
        let stream_id = stream["streamId"].as_u64().unwrap();
        call(
            &handler,
            origin,
            &format!(r#"{{"op":"streamWrite","streamId":{stream_id},"command":{{"type":"write","data":[104,105]}}}}"#),
        )
        .unwrap();
        call(
            &handler,
            origin,
            &format!(r#"{{"op":"streamClose","streamId":{stream_id}}}"#),
        )
        .unwrap();
        let file = call(&handler, origin, r#"{"op":"getFile","path":["log.txt"]}"#).unwrap();
        assert_eq!(file["data"], serde_json::json!([104, 105]));
        assert_eq!(file["size"], 2);
    }

    #[test]
    fn error_wire_carries_name_and_code() {
        let handler = opfs_handler(None);
        let error = handler(
            "https://example.com",
            r#"{"op":"getFileHandle","parent":[],"name":"missing.txt","create":false}"#,
        )
        .unwrap_err();
        let restored = opfs_error_from_wire(&error).unwrap();
        assert_eq!(restored.name, "NotFoundError");
        assert_eq!(restored.code, 8);
    }

    #[test]
    fn persistence_survives_reopen() {
        let root = std::env::temp_dir().join(format!("zw-opfs-host-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let origin = "https://persist.example";
        {
            let handler = opfs_handler(Some(root.clone()));
            call(
                &handler,
                origin,
                r#"{"op":"getFileHandle","parent":[],"name":"f.txt","create":true}"#,
            )
            .unwrap();
            let stream = call(
                &handler,
                origin,
                r#"{"op":"createWritable","path":["f.txt"],"keepExistingData":false}"#,
            )
            .unwrap();
            let stream_id = stream["streamId"].as_u64().unwrap();
            call(
                &handler,
                origin,
                &format!(
                    r#"{{"op":"streamWrite","streamId":{stream_id},"command":{{"type":"write","data":[1,2,3]}}}}"#
                ),
            )
            .unwrap();
            call(
                &handler,
                origin,
                &format!(r#"{{"op":"streamClose","streamId":{stream_id}}}"#),
            )
            .unwrap();
        }
        {
            let handler = opfs_handler(Some(root.clone()));
            let file = call(&handler, origin, r#"{"op":"getFile","path":["f.txt"]}"#).unwrap();
            assert_eq!(file["data"], serde_json::json!([1, 2, 3]));
            // 其他 origin 不受影响。
            let response = call(
                &handler,
                "https://other.example",
                r#"{"op":"getFile","path":["f.txt"]}"#,
            );
            assert!(response.is_err());
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn remove_entry_semantics_cross_wire() {
        let handler = opfs_handler(None);
        let origin = "https://example.com";
        call(
            &handler,
            origin,
            r#"{"op":"getDirectoryHandle","parent":[],"name":"d","create":true}"#,
        )
        .unwrap();
        call(
            &handler,
            origin,
            r#"{"op":"getFileHandle","parent":["d"],"name":"inner.txt","create":true}"#,
        )
        .unwrap();
        // 非空目录非 recursive → InvalidModificationError(13)。
        let error = handler(
            origin,
            r#"{"op":"removeEntry","parent":[],"name":"d","recursive":false}"#,
        )
        .unwrap_err();
        assert!(error.starts_with("InvalidModificationError|13|"), "{error}");
        // recursive → 成功。
        let ok = call(
            &handler,
            origin,
            r#"{"op":"removeEntry","parent":[],"name":"d","recursive":true}"#,
        )
        .unwrap();
        assert_eq!(ok["ok"], true);
    }
}

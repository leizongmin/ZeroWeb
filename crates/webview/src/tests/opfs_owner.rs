//! WebView OPFS 存储所有权 e2e（storage-opfs goal M3 / DC-3）。
//!
//! 覆盖：per-origin 落盘 + 跨会话 e2e（写入 → 重建 WebView/owner → 读回一致）+
//! in-memory owner 隔离（kill-switch 回退路径不落盘）。

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{IndexedDbOwner, WebView, WebViewConfig};

static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("zeroweb-webview-opfs-{}-{sequence}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// 写入 OPFS 文件（`navigator.storage.getDirectory()` 全页面 API 面，经 __zw_opfs 宿主桥）。
fn write_opfs_file(webview: &mut WebView, name: &str, file_content: &str) {
    let script = format!(
        r#"await navigator.storage.getDirectory()
             .then(root => root.getFileHandle({name:?}, {{ create: true }}))
             .then(handle => handle.createWritable())
             .then(w => w.write({file_content:?}).then(() => w))
             .then(w => w.close())
             .then(() => "ok")"#
    );
    let result = run_async_script(webview, &script);
    assert_eq!(result, "ok", "OPFS 写入应成功（{name}）");
}

/// 读回 OPFS 文件文本。
fn read_opfs_file(webview: &mut WebView, name: &str) -> String {
    let script = format!(
        r#"await navigator.storage.getDirectory()
             .then(root => root.getFileHandle({name:?}))
             .then(handle => handle.getFile())
             .then(file => file.text())"#
    );
    run_async_script(webview, &script)
}

fn webview_with_owner(owner: IndexedDbOwner, origin: &str) -> WebView {
    let mut webview = WebView::new_with_indexed_db_owner(WebViewConfig::default(), owner);
    webview.prepare_document_state(origin);
    webview.execute_script("0").unwrap();
    webview
}

/// 排空 microtask 队列（OPFS 全 Promise 链，多轮 execute 触发 drain；照 cache_storage 模式）。
fn pump_microtasks(webview: &mut WebView) {
    for i in 0..8 {
        webview
            .execute_script(&format!("globalThis.__opfsPump = {i};"))
            .unwrap();
    }
}

/// 执行 async IIFE 并等待结果落至 `globalThis.__opfsResult`。
fn run_async_script(webview: &mut WebView, body: &str) -> String {
    let script = format!(
        r#"(async () => {{ try {{ globalThis.__opfsResult = await (async () => {{ return {body} }})(); }} catch (e) {{ globalThis.__opfsResult = "error:" + (e && e.message ? e.message : e); }} }})()"#
    );
    webview.execute_script(&script).unwrap();
    pump_microtasks(webview);
    webview.execute_script("globalThis.__opfsResult").unwrap()
}

/// DC-3：per-origin 落盘，跨会话 e2e——写入 → 重建 WebView（同 root 新 owner）→ 读回一致。
#[test]
fn persistent_owner_opfs_file_survives_webview_rebuild() {
    let directory = TestDirectory::new();
    let origin = "https://opfs-persistent.example/page";
    {
        let owner = IndexedDbOwner::persistent(directory.path()).unwrap();
        let mut webview = webview_with_owner(owner, origin);
        write_opfs_file(&mut webview, "log.txt", "persisted content");
    }

    let owner = IndexedDbOwner::persistent(directory.path()).unwrap();
    let mut restored = webview_with_owner(owner, origin);
    assert_eq!(
        read_opfs_file(&mut restored, "log.txt"),
        "persisted content",
        "跨会话读回应与写入一致"
    );
}

/// per-origin 隔离：不同 origin 互不可见（同 owner 两个 origin 各自独立）。
#[test]
fn opfs_files_are_origin_scoped() {
    let directory = TestDirectory::new();
    let owner_a = IndexedDbOwner::persistent(directory.path()).unwrap();
    let mut webview_a = webview_with_owner(owner_a, "https://a.example/page");
    write_opfs_file(&mut webview_a, "shared-name.txt", "from-a");

    let owner_b = IndexedDbOwner::persistent(directory.path()).unwrap();
    let mut webview_b = webview_with_owner(owner_b, "https://b.example/page");
    write_opfs_file(&mut webview_b, "shared-name.txt", "from-b");

    // 各自读回各自的（origin 域隔离）。
    let owner_a2 = IndexedDbOwner::persistent(directory.path()).unwrap();
    let mut restored_a = webview_with_owner(owner_a2, "https://a.example/other");
    assert_eq!(read_opfs_file(&mut restored_a, "shared-name.txt"), "from-a");
    let owner_b2 = IndexedDbOwner::persistent(directory.path()).unwrap();
    let mut restored_b = webview_with_owner(owner_b2, "https://b.example/other");
    assert_eq!(read_opfs_file(&mut restored_b, "shared-name.txt"), "from-b");
}

/// in-memory owner（kill-switch 回退路径 / 私密浏览）：会话内可读，重建后不恢复（无落盘）。
#[test]
fn in_memory_owner_opfs_does_not_persist() {
    let directory = TestDirectory::new();
    let origin = "https://opfs-ephemeral.example/page";
    {
        let mut webview = webview_with_owner(IndexedDbOwner::in_memory(), origin);
        write_opfs_file(&mut webview, "temp.txt", "ephemeral");
        // 同会话可读。
        assert_eq!(read_opfs_file(&mut webview, "temp.txt"), "ephemeral");
    }

    let mut fresh = webview_with_owner(IndexedDbOwner::in_memory(), origin);
    let result = read_opfs_file(&mut fresh, "temp.txt");
    assert!(
        result.starts_with("error:"),
        "in-memory owner 重建后不应恢复文件（got {result:?}）"
    );
}

/// 目录结构（嵌套子目录 + 多文件）落盘往返。
#[test]
fn persistent_owner_opfs_directory_tree_roundtrip() {
    let directory = TestDirectory::new();
    let origin = "https://opfs-tree.example/page";
    {
        let owner = IndexedDbOwner::persistent(directory.path()).unwrap();
        let mut webview = webview_with_owner(owner, origin);
        let script = r#"await (async () => {
            const root = await navigator.storage.getDirectory();
            const docs = await root.getDirectoryHandle('docs', { create: true });
            const sub = await docs.getDirectoryHandle('sub', { create: true });
            const write = async (dir, name, text) => {
              const h = await dir.getFileHandle(name, { create: true });
              const w = await h.createWritable();
              await w.write(text);
              await w.close();
            };
            await write(docs, 'a.txt', 'alpha');
            await write(sub, 'b.txt', 'beta');
            return "ok";
          })()"#;
        assert_eq!(run_async_script(&mut webview, script), "ok");
    }

    let owner = IndexedDbOwner::persistent(directory.path()).unwrap();
    let mut restored = webview_with_owner(owner, origin);
    let script = r#"await (async () => {
        const root = await navigator.storage.getDirectory();
        const docs = await root.getDirectoryHandle('docs');
        const sub = await docs.getDirectoryHandle('sub');
        const a = await (await docs.getFileHandle('a.txt')).getFile();
        const b = await (await sub.getFileHandle('b.txt')).getFile();
        return (await a.text()) + "/" + (await b.text());
      })()"#;
    assert_eq!(run_async_script(&mut restored, script), "alpha/beta");
}

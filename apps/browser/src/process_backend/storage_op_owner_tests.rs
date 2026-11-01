//! StorageOp 特权边界回归（renderer-trust-boundary IPC-02）。
//!
//! 被攻破 renderer 可构造任意 `StorageOp` 消息；存储分区 origin 必须由
//! browser 侧已提交导航推导，不采信消息自带 origin（同 IndexedDB owner 模型）。

use std::collections::HashMap;
use std::path::PathBuf;

use super::*;

fn lock_multiprocess_tests() -> std::sync::MutexGuard<'static, ()> {
    crate::tests::MULTIPROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

fn commit_navigation(
    backend: &mut ProcessTabBackend,
    tab_id: TabId,
    renderer_id: u64,
    url: &str,
    navigation_epoch: u64,
) {
    backend.tab_to_renderer.insert(tab_id, renderer_id);
    backend.stage_indexed_db_navigation(renderer_id, url, navigation_epoch);
    backend.handle_navigation_committed(
        tab_id,
        renderer_id,
        NavigationCommittedParams {
            url: url.to_string(),
            navigation_epoch,
        },
    );
}

fn storage_op(origin: &str, operation: StorageOperation, key: &str, value: Option<&str>) -> StorageOpParams {
    StorageOpParams {
        storage_type: StorageType::Local,
        operation,
        key: key.to_string(),
        value: value.map(str::to_string),
        origin: origin.to_string(),
    }
}

#[test]
fn storage_op_binds_to_committed_origin_not_message_field() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(801);
    commit_navigation(&mut backend, tab_id, 101, "https://committed.example/page", 1);

    // 被攻破 renderer 自称 evil.example，值必须落在服务端绑定的 committed.example。
    backend.handle_storage_op(
        tab_id,
        101,
        storage_op(
            "https://evil.example",
            StorageOperation::Set,
            "session-key",
            Some("injected"),
        ),
    );

    let mut storage = backend.storage.lock().unwrap();
    assert_eq!(
        storage
            .local_storage("https://committed.example")
            .get("session-key")
            .map(str::to_string),
        Some("injected".to_string())
    );
    assert!(
        storage
            .local_storage("https://evil.example")
            .get("session-key")
            .is_none(),
        "消息自带 origin 不得决定存储分区"
    );
}

#[test]
fn storage_op_rejected_before_navigation_commit() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(802);

    // 未提交任何导航的 renderer 没有服务端绑定 origin，操作整体拒绝。
    backend.handle_storage_op(
        tab_id,
        102,
        storage_op("https://evil.example", StorageOperation::Set, "k", Some("v")),
    );

    let mut storage = backend.storage.lock().unwrap();
    assert!(storage.local_storage("https://evil.example").is_empty());
}

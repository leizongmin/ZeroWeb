#[cfg(test)]
use zero_protocol::{
    IpcMessage, IpcMessageKind, StorageOpParams, StorageOperation, StorageType, deserialize, serialize,
};
use zero_storage::StorageManager;

/// localStorage CRUD + IPC 序列化
#[test]
fn test_local_storage_crud_and_ipc() {
    let mut mgr = StorageManager::new();
    let store = mgr.local_storage("https://example.com");

    // CRUD 操作
    assert!(store.get("key").is_none());
    let old = store.set("key", "value").expect("set");
    assert!(old.is_none());
    assert_eq!(store.get("key"), Some("value"));

    let old = store.set("key", "updated").expect("set");
    assert_eq!(old, Some("value".to_string()));
    assert_eq!(store.get("key"), Some("updated"));

    // 通过 IPC 消息序列化传输存储操作
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Set,
            key: "key".to_string(),
            value: Some("updated".to_string()),
            origin: "https://example.com".to_string(),
        }),
    };
    let bytes = serialize(&msg).expect("serialize");
    let decoded = deserialize(&bytes).expect("deserialize");
    if let IpcMessageKind::StorageOp(p) = decoded.kind {
        assert_eq!(p.key, "key");
        assert_eq!(p.value, Some("updated".to_string()));
    } else {
        panic!("expected StorageOp");
    }
}

/// sessionStorage 隔离
#[test]
fn test_session_storage_isolation() {
    let mut mgr = StorageManager::new();

    let local = mgr.local_storage("https://example.com");
    local.set("shared_key", "local_value").unwrap();

    let session = mgr.session_storage("https://example.com");
    session.set("shared_key", "session_value").unwrap();

    assert_eq!(
        mgr.local_storage("https://example.com").get("shared_key"),
        Some("local_value")
    );
    assert_eq!(
        mgr.session_storage("https://example.com").get("shared_key"),
        Some("session_value")
    );
}

/// 不同源的存储隔离
#[test]
fn test_storage_origin_isolation() {
    let mut mgr = StorageManager::new();

    let store_a = mgr.local_storage("https://a.com");
    store_a.set("key", "value_a").unwrap();

    let store_b = mgr.local_storage("https://b.com");
    store_b.set("key", "value_b").unwrap();

    assert_eq!(mgr.local_storage("https://a.com").get("key"), Some("value_a"));
    assert_eq!(mgr.local_storage("https://b.com").get("key"), Some("value_b"));
}

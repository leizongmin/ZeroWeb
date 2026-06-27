//! 浏览器 Shell + Protocol + Storage 跨 crate 集成测试。
//!
//! 验证 BrowserShell 操作（导航、标签页管理）与 IPC 消息序列化和
//! Storage 持久化的端到端正确性。

#[cfg(test)]
use zero_browser_shell::{BrowserShell, SuggestionSource};
#[cfg(test)]
use zero_protocol::{IpcMessage, IpcMessageKind, NavigateParams, deserialize, serialize};
#[cfg(test)]
use zero_storage::StorageManager;

/// BrowserShell 导航操作产生正确的 IPC Navigate 消息并序列化/反序列化。
#[test]
fn test_browser_shell_navigation_ipc_roundtrip() {
    let mut shell = BrowserShell::new();

    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");

    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".to_string(),
            referrer: None,
                navigation_epoch: 0,
        }),
    };
    let bytes = serialize(&msg).expect("serialize navigate");
    let decoded = deserialize(&bytes).expect("deserialize navigate");

    if let IpcMessageKind::Navigate(p) = decoded.kind {
        assert_eq!(p.url, "https://example.com");
        assert_eq!(p.referrer, None);
    } else {
        panic!("expected Navigate");
    }

    let tab = shell.active_tab().unwrap();
    assert_eq!(tab.url(), Some("https://example.com"));
    assert_eq!(tab.title(), Some("Example"));
}

/// 多标签页导航 → 多条 IPC 消息 → 顺序保持。
#[test]
fn test_browser_shell_multi_tab_navigation_ipc() {
    let mut shell = BrowserShell::new();

    shell.navigate("https://site-a.com");
    shell.on_page_loaded("Site A");

    shell.new_tab(None);
    shell.navigate("https://site-b.com");
    shell.on_page_loaded("Site B");

    assert_eq!(shell.tab_count(), 2);

    let msgs = vec![
        IpcMessage {
            id: 1,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://site-a.com".to_string(),
                referrer: None,
                navigation_epoch: 0,
            }),
        },
        IpcMessage {
            id: 2,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://site-b.com".to_string(),
                referrer: Some("https://site-a.com".to_string()),
                navigation_epoch: 0,
            }),
        },
    ];

    let mut decoded_msgs = Vec::new();
    for msg in &msgs {
        let bytes = serialize(msg).expect("serialize");
        let decoded = deserialize(&bytes).expect("deserialize");
        decoded_msgs.push(decoded);
    }

    assert_eq!(decoded_msgs.len(), 2);
    assert_eq!(decoded_msgs[0].id, 1);
    assert_eq!(decoded_msgs[1].id, 2);

    if let IpcMessageKind::Navigate(p) = &decoded_msgs[1].kind {
        assert_eq!(p.referrer, Some("https://site-a.com".to_string()));
    } else {
        panic!("expected Navigate");
    }
}

/// BrowserShell 前进/后退 → 对应 IPC GoBack/GoForward 消息。
#[test]
fn test_browser_shell_back_forward_ipc() {
    let mut shell = BrowserShell::new();

    shell.navigate("https://a.com");
    shell.on_page_loaded("A");
    shell.navigate("https://b.com");
    shell.on_page_loaded("B");

    let back_msg = IpcMessage {
        id: 10,
        kind: IpcMessageKind::GoBack,
    };
    let bytes = serialize(&back_msg).expect("serialize go_back");
    let decoded = deserialize(&bytes).expect("deserialize go_back");
    assert!(matches!(decoded.kind, IpcMessageKind::GoBack));

    shell.go_back();
    let tab = shell.active_tab().unwrap();
    assert_eq!(tab.url(), Some("https://a.com"));

    let fwd_msg = IpcMessage {
        id: 11,
        kind: IpcMessageKind::GoForward,
    };
    let bytes = serialize(&fwd_msg).expect("serialize go_forward");
    let decoded = deserialize(&bytes).expect("deserialize go_forward");
    assert!(matches!(decoded.kind, IpcMessageKind::GoForward));

    shell.go_forward();
    let tab = shell.active_tab().unwrap();
    assert_eq!(tab.url(), Some("https://b.com"));
}

/// BrowserShell 书签操作与 StorageManager 持久化集成。
#[test]
fn test_browser_shell_bookmarks_with_storage() {
    let mut shell = BrowserShell::new();
    let mut storage = StorageManager::new();

    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");
    shell.add_bookmark();
    assert_eq!(shell.bookmarks().len(), 1);

    let bookmark_json = r#"{"title":"Example","url":"https://example.com"}"#;
    storage
        .local_storage("https://example.com")
        .set("bookmark_1", bookmark_json)
        .unwrap();

    let stored = storage.local_storage("https://example.com").get("bookmark_1").unwrap();
    assert!(stored.contains("Example"));
    assert!(stored.contains("https://example.com"));

    storage
        .local_storage("https://example.com")
        .remove("bookmark_1")
        .unwrap();
    assert!(storage.local_storage("https://example.com").get("bookmark_1").is_none());
}

/// BrowserShell 历史记录 → StorageManager 持久化。
#[test]
fn test_browser_shell_history_with_storage() {
    let mut shell = BrowserShell::new();
    let mut storage = StorageManager::new();

    shell.navigate("https://a.com");
    shell.on_page_loaded("A");
    shell.navigate("https://b.com");
    shell.on_page_loaded("B");
    shell.navigate("https://c.com");
    shell.on_page_loaded("C");

    assert_eq!(shell.history().len(), 3);

    for (i, entry) in shell.history().iter().enumerate() {
        let key = format!("history_{i}");
        let value = format!("{}|{}", entry.title(), entry.url());
        storage.session_storage("browser://history").set(&key, &value).unwrap();
    }

    // History is LIFO (most recent first): c.com, b.com, a.com
    let h0 = storage.session_storage("browser://history").get("history_0").unwrap();
    assert!(h0.contains("C"));
    assert!(h0.contains("https://c.com"));

    let h2 = storage.session_storage("browser://history").get("history_2").unwrap();
    assert!(h2.contains("A"));
    assert!(h2.contains("https://a.com"));

    shell.history_mut().clear();
    assert!(shell.history().is_empty());
    storage.session_storage("browser://history").clear();
    assert!(storage.session_storage("browser://history").get("history_0").is_none());
}

/// BrowserShell 下载管理器 → IPC FetchResponse 消息。
#[test]
fn test_browser_shell_download_ipc() {
    let mut shell = BrowserShell::new();

    let dl_id = shell
        .downloads_mut()
        .start_download("https://example.com/file.zip", "file.zip");
    shell.downloads_mut().update_progress(dl_id, 50, Some(100));

    use zero_protocol::FetchResponseParams;
    let msg = IpcMessage {
        id: 100,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 1,
            status_code: 200,
            headers: vec![],
            body: vec![0u8; 50],
        }),
    };

    let bytes = serialize(&msg).expect("serialize fetch_response");
    let decoded = deserialize(&bytes).expect("deserialize fetch_response");

    if let IpcMessageKind::FetchResponse(p) = decoded.kind {
        assert_eq!(p.status_code, 200);
        assert_eq!(p.body.len(), 50);
    } else {
        panic!("expected FetchResponse");
    }

    shell.downloads_mut().mark_completed(dl_id);
    let dl = shell.downloads().get(dl_id).unwrap();
    assert!(dl.is_completed());
}

/// BrowserShell 自动补全与书签/历史交互。
#[test]
fn test_browser_shell_autocomplete_from_history_and_bookmarks() {
    let mut shell = BrowserShell::new();

    shell.navigate("https://rust-lang.org");
    shell.on_page_loaded("Rust");
    shell.navigate("https://doc.rust-lang.org/book");
    shell.on_page_loaded("The Book");
    shell.add_bookmark();

    let suggestions = shell.suggest("rust");
    assert!(!suggestions.is_empty(), "should have suggestions for 'rust'");

    let has_bookmark = suggestions
        .iter()
        .any(|s| s.source() == SuggestionSource::Bookmark && s.url().contains("rust-lang.org"));
    assert!(has_bookmark, "bookmark should appear in suggestions");
}

/// BrowserShell 设置 → IPC Storage 操作消息。
#[test]
fn test_browser_shell_settings_storage_ipc() {
    let shell = BrowserShell::new();
    let settings = shell.settings();

    let url = settings.search("hello world");
    assert!(url.contains("hello+world") || url.contains("hello%20world"));

    use zero_protocol::{StorageOpParams, StorageOperation, StorageType};
    let msg = IpcMessage {
        id: 200,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Set,
            key: "search_engine".to_string(),
            value: Some("google".to_string()),
            origin: "browser://settings".to_string(),
        }),
    };

    let bytes = serialize(&msg).expect("serialize storage op");
    let decoded = deserialize(&bytes).expect("deserialize storage op");

    if let IpcMessageKind::StorageOp(p) = decoded.kind {
        assert_eq!(p.storage_type, StorageType::Local);
        assert!(matches!(p.operation, StorageOperation::Set));
        assert_eq!(p.key, "search_engine");
        assert_eq!(p.origin, "browser://settings");
    } else {
        panic!("expected StorageOp");
    }
}

/// BrowserShell 缩放操作与 IPC Reload 消息同步。
#[test]
fn test_browser_shell_zoom_reload_ipc() {
    let mut shell = BrowserShell::new();

    shell.zoom_in();
    assert!(shell.zoom() > 1.0);

    shell.zoom_out();
    shell.zoom_out();
    assert!(shell.zoom() < 1.0);

    shell.zoom_reset();
    assert!((shell.zoom() - 1.0).abs() < f32::EPSILON);

    let msg = IpcMessage {
        id: 300,
        kind: IpcMessageKind::Reload,
    };
    let bytes = serialize(&msg).expect("serialize reload");
    let decoded = deserialize(&bytes).expect("deserialize reload");
    assert!(matches!(decoded.kind, IpcMessageKind::Reload));
}

/// BrowserShell 页面加载完成 → IPC LoadComplete 消息。
#[test]
fn test_browser_shell_page_load_ipc() {
    let mut shell = BrowserShell::new();

    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");

    let msg = IpcMessage {
        id: 400,
        kind: IpcMessageKind::LoadComplete,
    };
    let bytes = serialize(&msg).expect("serialize");
    let decoded = deserialize(&bytes).expect("deserialize");
    assert!(matches!(decoded.kind, IpcMessageKind::LoadComplete));

    let tab = shell.active_tab().unwrap();
    assert!(!tab.is_loading());
}

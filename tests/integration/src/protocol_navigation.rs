#[cfg(test)]
use zero_net::navigation::NavigationHistory;
use zero_protocol::{IpcMessage, IpcMessageKind, NavigateParams, deserialize, serialize};

/// 导航历史操作 → IPC 消息序列化
#[test]
fn test_navigation_ipc_roundtrip() {
    let mut nav = NavigationHistory::new(50);
    nav.navigate("https://example.com", Some("Home".to_string()));
    nav.navigate("https://example.com/about", Some("About".to_string()));

    // 序列化导航命令
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com/about".to_string(),
            referrer: Some("https://example.com".to_string()),
        }),
    };
    let bytes = serialize(&msg).expect("serialize");
    let decoded = deserialize(&bytes).expect("deserialize");

    if let IpcMessageKind::Navigate(p) = decoded.kind {
        assert_eq!(p.url, "https://example.com/about");
        assert_eq!(p.referrer, Some("https://example.com".to_string()));
    } else {
        panic!("expected Navigate");
    }

    // 验证导航历史状态
    nav.go_back();
    assert_eq!(nav.current().unwrap().url, "https://example.com");
}

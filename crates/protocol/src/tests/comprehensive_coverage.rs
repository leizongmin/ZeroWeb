//! 额外测试：提升覆盖率至 100%

use super::*;

#[test]
fn test_serialize_deserialize_stop_loading() {
    let msg = IpcMessage {
        id: 100,
        kind: IpcMessageKind::StopLoading,
    };
    let out = roundtrip(msg);
    assert!(matches!(out.kind, IpcMessageKind::StopLoading));
}

#[test]
fn test_navigate_params_edge_cases() {
    // 测试 NavigateParams 的所有边界情况
    let msg1 = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: None,
            navigation_epoch: 0,
        }),
    };
    let out1 = roundtrip(msg1);
    if let IpcMessageKind::Navigate(p) = out1.kind {
        assert_eq!("https://example.com", p.url);
        assert!(p.referrer.is_none());
    } else {
        panic!("expected Navigate");
    }

    let msg2 = IpcMessage {
        id: 2,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://test.com/path?query=value#frag".into(),
            referrer: Some("https://referrer.com".into()),
            navigation_epoch: 0,
        }),
    };
    let out2 = roundtrip(msg2);
    if let IpcMessageKind::Navigate(p) = out2.kind {
        assert_eq!("https://test.com/path?query=value#frag", p.url);
        assert_eq!(Some("https://referrer.com".into()), p.referrer);
    } else {
        panic!("expected Navigate");
    }
}

#[test]
fn test_load_complete_and_load_failed() {
    // 测试 LoadComplete 变体
    let msg1 = IpcMessage {
        id: 1,
        kind: IpcMessageKind::LoadComplete,
    };
    let out1 = roundtrip(msg1);
    assert!(matches!(out1.kind, IpcMessageKind::LoadComplete));

    // 测试 LoadFailed 变体
    let msg2 = IpcMessage {
        id: 2,
        kind: IpcMessageKind::LoadFailed("network error".into()),
    };
    let out2 = roundtrip(msg2);
    if let IpcMessageKind::LoadFailed(reason) = out2.kind {
        assert_eq!("network error", reason);
    } else {
        panic!("expected LoadFailed");
    }
}

#[test]
fn test_scroll_event_boundary_values() {
    // 测试滚动事件的最大和最小值
    let msg1 = IpcMessage {
        id: 1,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: f32::MAX,
            delta_y: f32::MIN,
            ..Default::default()
        }),
    };
    let out1 = roundtrip(msg1);
    if let IpcMessageKind::ScrollEvent(p) = out1.kind {
        assert_eq!(f32::MAX, p.delta_x);
        assert_eq!(f32::MIN, p.delta_y);
    } else {
        panic!("expected ScrollEvent");
    }

    let msg2 = IpcMessage {
        id: 2,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: -0.0,
            delta_y: 0.0,
            ..Default::default()
        }),
    };
    let out2 = roundtrip(msg2);
    if let IpcMessageKind::ScrollEvent(p) = out2.kind {
        assert_eq!(-0.0, p.delta_x);
        assert_eq!(0.0, p.delta_y);
    } else {
        panic!("expected ScrollEvent");
    }
}

#[test]
fn test_storage_op_clear_and_key_operations() {
    // 测试 Clear 操作（key 通常为空字符串）
    let msg1 = IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Clear,
            key: String::new(),
            value: None,
            origin: "https://example.com".into(),
        }),
    };
    let out1 = roundtrip(msg1);
    if let IpcMessageKind::StorageOp(p) = out1.kind {
        assert_eq!(StorageOperation::Clear, p.operation);
        assert!(p.key.is_empty());
        assert!(p.value.is_none());
    } else {
        panic!("expected StorageOp");
    }

    // 测试 Key 操作（按索引获取键名）
    let msg2 = IpcMessage {
        id: 2,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Session,
            operation: StorageOperation::Key,
            key: "5".into(), // 获取索引为 5 的键名
            value: None,
            origin: "https://app.example.com".into(),
        }),
    };
    let out2 = roundtrip(msg2);
    if let IpcMessageKind::StorageOp(p) = out2.kind {
        assert_eq!(StorageOperation::Key, p.operation);
        assert_eq!("5", p.key);
    } else {
        panic!("expected StorageOp");
    }
}

#[test]
fn test_process_role_debug_and_clone_comprehensive() {
    // 测试 ProcessRole 的 Debug 输出
    let debug_browser = format!("{:?}", ProcessRole::Browser);
    assert!(debug_browser.contains("Browser"));
    assert!(!debug_browser.is_empty());

    let debug_renderer = format!("{:?}", ProcessRole::Renderer);
    assert!(debug_renderer.contains("Renderer"));
    assert!(!debug_renderer.is_empty());
    assert_ne!(debug_browser, debug_renderer);

    // 测试 Copy 和 Clone 语义
    let role1 = ProcessRole::Browser;
    let role2 = role1; // Copy
    let role3 = role1; // Clone (Copy implies Clone)
    assert_eq!(role1, role2);
    assert_eq!(role1, role3);
    assert_ne!(role1, ProcessRole::Renderer);
}

#[test]
fn test_protocol_error_display_edge_cases() {
    // 测试 ProtocolError 的 Display 输出
    let errors = vec![
        ProtocolError::Serialization("".into()),
        ProtocolError::Deserialization("error\nwith\tcontrol\rchars".into()),
        ProtocolError::Channel("pipe broken".into()),
        ProtocolError::Process("💥 crash".into()),
    ];

    for err in errors {
        let display = format!("{err}");
        assert!(!display.is_empty(), "Display output should not be empty");

        // 验证每种错误类型都有特定的前缀
        match err {
            ProtocolError::Serialization(_) => {
                assert!(display.contains("Serialization error"));
            }
            ProtocolError::Deserialization(_) => {
                assert!(display.contains("Deserialization error"));
            }
            ProtocolError::Channel(_) => {
                assert!(display.contains("Channel error"));
            }
            ProtocolError::Process(_) => {
                assert!(display.contains("Process error"));
            }
        }
    }
}

#[test]
fn test_fetch_response_max_status_code() {
    // 测试 FetchResponse 的最大状态码
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 999,
            status_code: u16::MAX,
            headers: vec![("X-Max".into(), "true".into())],
            body: vec![0xFF, 0xFF],
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchResponse(p) = out.kind {
        assert_eq!(u16::MAX, p.status_code);
        assert_eq!(1, p.headers.len());
        assert_eq!(2, p.body.len());
    } else {
        panic!("expected FetchResponse");
    }
}

#[test]
fn test_ipc_message_id_max() {
    // 测试 IpcMessage 的最大 ID
    let msg = IpcMessage {
        id: u64::MAX,
        kind: IpcMessageKind::Ok,
    };
    let out = roundtrip(msg);
    assert_eq!(u64::MAX, out.id);
    assert!(matches!(out.kind, IpcMessageKind::Ok));
}

#[test]
fn test_mouse_event_button_zero() {
    // 测试 MouseEvent 的 button = 0（左键）
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::MouseEvent(MouseEventParams {
            x: 100.0,
            y: 200.0,
            button: 0,
            event_type: MouseEventType::Click,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::MouseEvent(p) = out.kind {
        assert_eq!(100.0, p.x);
        assert_eq!(200.0, p.y);
        assert_eq!(0, p.button);
        assert_eq!(MouseEventType::Click, p.event_type);
    } else {
        panic!("expected MouseEvent");
    }
}

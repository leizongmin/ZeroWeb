//! 序列化/反序列化路径覆盖率补充测试 — 确保两个函数的成功路径各被直接调用。

use super::*;

/// 直接调用 serialize() 并验证输出非空。
#[test]
fn test_serialize_produces_nonempty_bytes() {
    let msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::Ok,
    };
    let bytes = serialize(&msg).expect("serialize should succeed");
    assert!(!bytes.is_empty(), "serialized bytes should not be empty");
}

/// 直接调用 deserialize() 并验证输出。
#[test]
fn test_deserialize_valid_bytes_succeeds() {
    let msg = IpcMessage {
        id: 99,
        kind: IpcMessageKind::Heartbeat,
    };
    let bytes = serialize(&msg).expect("serialize");
    let out = deserialize(&bytes).expect("deserialize should succeed");
    assert_eq!(out.id, 99);
    assert!(matches!(out.kind, IpcMessageKind::Heartbeat));
}

/// 序列化所有 IpcMessageKind 变体并验证 roundtrip 正确。
#[test]
fn test_roundtrip_all_message_kinds() {
    let messages: Vec<IpcMessage> = vec![
        IpcMessage {
            id: 1,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com".into(),
                referrer: None,
                navigation_epoch: 0,
            }),
        },
        IpcMessage {
            id: 2,
            kind: IpcMessageKind::GoBack,
        },
        IpcMessage {
            id: 3,
            kind: IpcMessageKind::GoForward,
        },
        IpcMessage {
            id: 4,
            kind: IpcMessageKind::StopLoading,
        },
        IpcMessage {
            id: 5,
            kind: IpcMessageKind::Reload,
        },
        IpcMessage {
            id: 6,
            kind: IpcMessageKind::LoadHtml(LoadHtmlParams {
                html: "<body>hi</body>".into(),
                css: Some("body { margin: 0; }".into()),
                url: Some("zero://settings".into()),
                navigation_epoch: 0,
            }),
        },
        IpcMessage {
            id: 7,
            kind: IpcMessageKind::SetViewport(SetViewportParams {
                width: 800,
                height: 600,
                device_scale_factor: 1.0,
            }),
        },
        IpcMessage {
            id: 8,
            kind: IpcMessageKind::SetColorScheme(SetColorSchemeParams {
                scheme: IpcColorScheme::Light,
            }),
        },
        IpcMessage {
            id: 9,
            kind: IpcMessageKind::TitleChanged("Test Page".into()),
        },
        IpcMessage {
            id: 10,
            kind: IpcMessageKind::UrlChanged("https://new.url".into()),
        },
        IpcMessage {
            id: 11,
            kind: IpcMessageKind::LoadComplete,
        },
        IpcMessage {
            id: 12,
            kind: IpcMessageKind::LoadFailed("timeout".into()),
        },
        IpcMessage {
            id: 13,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 1,
                url: "https://api.example.com".into(),
                method: "POST".into(),
                headers: vec![("Content-Type".into(), "application/json".into())],
                body: Some(b"{}".to_vec()),
            }),
        },
        IpcMessage {
            id: 14,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 1,
                status_code: 201,
                headers: vec![],
                body: b"created".to_vec(),
            }),
        },
        IpcMessage {
            id: 15,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Set,
                key: "foo".into(),
                value: Some("bar".into()),
                origin: "https://example.com".into(),
            }),
        },
        IpcMessage {
            id: 16,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 100.0,
                y: 200.0,
                button: 0,
                event_type: MouseEventType::Click,
            }),
        },
        IpcMessage {
            id: 17,
            kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: "A".into(),
                code: "KeyA".into(),
                ctrl: false,
                shift: false,
                alt: false,
                meta: false,
                event_type: KeyboardEventType::Down,
            }),
        },
        IpcMessage {
            id: 18,
            kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
                delta_x: 10.0,
                delta_y: -5.0,
                ..Default::default()
            }),
        },
        IpcMessage {
            id: 19,
            kind: IpcMessageKind::HitTestLink(HitTestLinkParams { x: 1.0, y: 2.0 }),
        },
        IpcMessage {
            id: 20,
            kind: IpcMessageKind::HitTestLinkResult(HitTestLinkResultParams { href: None }),
        },
        IpcMessage {
            id: 21,
            kind: IpcMessageKind::Heartbeat,
        },
        IpcMessage {
            id: 25,
            kind: IpcMessageKind::HitTestImage(HitTestLinkParams { x: 3.0, y: 4.0 }),
        },
        IpcMessage {
            id: 26,
            kind: IpcMessageKind::HitTestImageResult(HitTestLinkResultParams {
                href: Some("https://example.com/img.png".into()),
            }),
        },
        IpcMessage {
            id: 22,
            kind: IpcMessageKind::CrashNotification("segfault".into()),
        },
        IpcMessage {
            id: 23,
            kind: IpcMessageKind::Ok,
        },
        IpcMessage {
            id: 24,
            kind: IpcMessageKind::Error("something went wrong".into()),
        },
        IpcMessage {
            id: 27,
            kind: IpcMessageKind::AutomationRequest(AutomationRequest {
                operation: AutomationOperation::GetActiveElement,
            }),
        },
        IpcMessage {
            id: 28,
            kind: IpcMessageKind::AutomationResponse(AutomationResponse {
                navigation_epoch: 1,
                document_generation: 2,
                result: Ok(AutomationResult::Empty),
            }),
        },
    ];

    for msg in &messages {
        let bytes = serialize(msg).unwrap_or_else(|e| panic!("serialize failed for id={}: {e:?}", msg.id));
        let out = deserialize(&bytes).unwrap_or_else(|e| panic!("deserialize failed for id={}: {e:?}", msg.id));
        assert_eq!(out.id, msg.id, "id mismatch");
    }

    assert_eq!(messages.len(), 28, "should test all listed message kinds");
}

/// 测试 StorageType::Session 变体也能序列化。
#[test]
fn test_storage_op_session_type() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Session,
            operation: StorageOperation::Get,
            key: "session_key".into(),
            value: None,
            origin: "https://example.com".into(),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::StorageOp(p) = out.kind {
        assert_eq!(p.storage_type, StorageType::Session);
        assert_eq!(p.operation, StorageOperation::Get);
    } else {
        panic!("expected StorageOp");
    }
}

/// 测试所有 StorageOperation 变体。
#[test]
fn test_all_storage_operations() {
    let ops = vec![
        StorageOperation::Get,
        StorageOperation::Set,
        StorageOperation::Remove,
        StorageOperation::Clear,
        StorageOperation::Length,
        StorageOperation::Key,
    ];
    for op in ops {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: op.clone(),
                key: "k".into(),
                value: None,
                origin: "https://example.com".into(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::StorageOp(p) = out.kind {
            assert_eq!(p.operation, op);
        } else {
            panic!("expected StorageOp");
        }
    }
}

/// 测试所有 MouseEventType 变体。
#[test]
fn test_all_mouse_event_types() {
    let types = vec![
        MouseEventType::Down,
        MouseEventType::Up,
        MouseEventType::Move,
        MouseEventType::Click,
        MouseEventType::DblClick,
    ];
    for et in &types {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 0.0,
                y: 0.0,
                button: 0,
                event_type: et.clone(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::MouseEvent(p) = out.kind {
            assert_eq!(p.event_type, *et);
        } else {
            panic!("expected MouseEvent");
        }
    }
}

/// 测试所有 KeyboardEventType 变体。
#[test]
fn test_all_keyboard_event_types() {
    let types = vec![KeyboardEventType::Down, KeyboardEventType::Up, KeyboardEventType::Press];
    for et in &types {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: "X".into(),
                code: "KeyX".into(),
                ctrl: false,
                shift: false,
                alt: false,
                meta: false,
                event_type: et.clone(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::KeyboardEvent(p) = out.kind {
            assert_eq!(p.event_type, *et);
        } else {
            panic!("expected KeyboardEvent");
        }
    }
}

/// 测试 Navigate 无 referrer。
#[test]
fn test_navigate_no_referrer() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: None,
            navigation_epoch: 0,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::Navigate(p) = out.kind {
        assert_eq!(p.url, "https://example.com");
        assert!(p.referrer.is_none());
    } else {
        panic!("expected Navigate");
    }
}

/// 测试 FetchRequest 无 body。
#[test]
fn test_fetch_request_no_body() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 1,
            url: "https://example.com".into(),
            method: "GET".into(),
            headers: vec![],
            body: None,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchRequest(p) = out.kind {
        assert!(p.body.is_none());
    } else {
        panic!("expected FetchRequest");
    }
}

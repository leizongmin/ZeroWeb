//! IPC 序列化覆盖率提升测试。
//!
//! 专注于测试序列化/反序列化的各种边界条件和错误恢复场景。

use super::*;

#[test]
fn test_serialize_deserialize_empty_message() {
    // 创建一个最小消息
    let msg = IpcMessage {
        id: 0,
        kind: IpcMessageKind::Ok,
    };
    let out = roundtrip(msg);
    assert_eq!(0, out.id);
    assert!(matches!(out.kind, IpcMessageKind::Ok));
}

#[test]
fn test_serialize_deserialize_all_error_variants() {
    // 测试所有错误类型的序列化/反序列化
    let error_msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::Error("test error".to_string()),
    };
    let out = roundtrip(error_msg);
    if let IpcMessageKind::Error(err) = out.kind {
        assert_eq!("test error", err);
    } else {
        panic!("expected Error variant");
    }
}

#[test]
fn test_serialize_deserialize_max_u64_id() {
    // 测试最大消息 ID
    let msg = IpcMessage {
        id: u64::MAX,
        kind: IpcMessageKind::Heartbeat,
    };
    let out = roundtrip(msg);
    assert_eq!(u64::MAX, out.id);
    assert!(matches!(out.kind, IpcMessageKind::Heartbeat));
}

#[test]
fn test_serialize_deserialize_unicode_strings() {
    // 测试包含 Unicode 字符串的各种消息
    let messages = vec![
        IpcMessage {
            id: 1,
            kind: IpcMessageKind::TitleChanged("你好世界 🌍".to_string()),
        },
        IpcMessage {
            id: 2,
            kind: IpcMessageKind::UrlChanged("https://例子.com/测试".to_string()),
        },
        IpcMessage {
            id: 3,
            kind: IpcMessageKind::LoadFailed("连接超时: 500ms".to_string()),
        },
        IpcMessage {
            id: 4,
            kind: IpcMessageKind::CrashNotification("Segmentation fault in module: ñoño_café".to_string()),
        },
    ];

    for msg in messages {
        let out = roundtrip(msg.clone());
        assert_eq!(msg.id, out.id);

        match (msg.kind, out.kind) {
            (IpcMessageKind::TitleChanged(s1), IpcMessageKind::TitleChanged(s2)) => {
                assert_eq!(s1, s2);
            }
            (IpcMessageKind::UrlChanged(s1), IpcMessageKind::UrlChanged(s2)) => {
                assert_eq!(s1, s2);
            }
            (IpcMessageKind::LoadFailed(s1), IpcMessageKind::LoadFailed(s2)) => {
                assert_eq!(s1, s2);
            }
            (IpcMessageKind::CrashNotification(s1), IpcMessageKind::CrashNotification(s2)) => {
                assert_eq!(s1, s2);
            }
            _ => panic!("Kind mismatch"),
        }
    }
}

#[test]
fn test_serialize_deserialize_large_payload() {
    // 测试大负载数据
    let large_body: Vec<u8> = (0..10_000).map(|i| (i % 256) as u8).collect();
    let msg = IpcMessage {
        id: 999,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 1,
            status_code: 200,
            headers: vec![
                ("Content-Type".to_string(), "application/octet-stream".to_string()),
                ("Content-Length".to_string(), "10000".to_string()),
            ],
            body: large_body.clone(),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchResponse(p) = out.kind {
        assert_eq!(10000, p.body.len());
        assert_eq!(large_body, p.body);
        assert_eq!(2, p.headers.len());
        assert_eq!("application/octet-stream", p.headers[1].1);
    } else {
        panic!("expected FetchResponse");
    }
}

#[test]
fn test_serialize_deserialize_many_headers() {
    // 测试大量 HTTP 头
    let mut headers = Vec::new();
    for i in 0..100 {
        headers.push((format!("X-Custom-{}", i), format!("value-{}", i)));
    }
    headers.push(("Content-Type".to_string(), "application/json".to_string()));

    let msg = IpcMessage {
        id: 100,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 42,
            url: "https://api.example.com/large".to_string(),
            method: "POST".to_string(),
            headers,
            body: Some(b"{\"data\": \"very large payload\"}".to_vec()),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchRequest(p) = out.kind {
        assert_eq!(102, p.headers.len());  // 100 custom + 2 built-in
        assert_eq!(Some(b"{\"data\": \"very large payload\"}".to_vec()), p.body);
    } else {
        panic!("expected FetchRequest");
    }
}

#[test]
fn test_serialize_deserialize_special_characters() {
    // 测试特殊字符在各种字符串字段中
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com/path?q=value with spaces&id=123&special=<>&\"'".to_string(),
            referrer: Some("https://referrer.com/path;parameters?with=special&chars".to_string()),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::Navigate(p) = out.kind {
        assert_eq!(p.url, "https://example.com/path?q=value with spaces&id=123&special=<>&\"'");
        assert_eq!(p.referrer, Some("https://referrer.com/path;parameters?with=special&chars"));
    } else {
        panic!("expected Navigate");
    }
}

#[test]
fn test_serialize_deserialize_empty_values() {
    // 测试各种空值情况
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Set,
            key: "".to_string(),
            value: Some("".to_string()),
            origin: "https://example.com".to_string(),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::StorageOp(p) = out.kind {
        assert_eq!(p.key, "");
        assert_eq!(p.value, Some("".to_string()));
        assert_eq!(p.origin, "https://example.com");
    } else {
        panic!("expected StorageOp");
    }
}

#[test]
fn test_serialize_deserialize_empty_body() {
    // 测试空的请求体
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 42,
            url: "https://api.example.com/empty".to_string(),
            method: "GET".to_string(),
            headers: vec![],
            body: None,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchRequest(p) = out.kind {
        assert_eq!(p.body, None);
    } else {
        panic!("expected FetchRequest");
    }
}

#[test]
fn test_serialize_deserialize_zero_byte_body() {
    // 测试零字节体
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 42,
            status_code: 204,
            headers: vec![],
            body: vec![],
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchResponse(p) = out.kind {
        assert_eq!(p.body, vec![]);
        assert_eq!(204, p.status_code);
    } else {
        panic!("expected FetchResponse");
    }
}

#[test]
fn test_serialize_deserialize_boundary_status_codes() {
    // 测试各种 HTTP 状态码（包括边界值）
    let test_cases = vec![
        (0, "Empty status"),
        (100, "Continue"),
        (200, "OK"),
        (301, "Moved Permanently"),
        (404, "Not Found"),
        (500, "Internal Server Error"),
        (599, "Non-standard"),
        (65535, "Max u16"),
    ];

    for (status_code, desc) in test_cases {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 1,
                status_code,
                headers: vec![],
                body: vec![],
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchResponse(p) = out.kind {
            assert_eq!(status_code, p.status_code);
        } else {
            panic!("expected FetchResponse for {}", desc);
        }
    }
}

#[test]
fn test_serialize_deserialize_all_keyboard_event_types() {
    // 测试所有键盘事件类型
    let events = vec![
        (KeyboardEventType::Down, "keydown"),
        (KeyboardEventType::Up, "keyup"),
        (KeyboardEventType::Press, "keypress"),
    ];

    for (event_type, desc) in events {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: "a".to_string(),
                code: "KeyA".to_string(),
                ctrl: true,
                shift: false,
                alt: true,
                meta: false,
                event_type,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::KeyboardEvent(p) = out.kind {
            assert_eq!(event_type, p.event_type);
            assert_eq!(true, p.ctrl);
            assert_eq!(true, p.alt);
        } else {
            panic!("expected KeyboardEvent for {}", desc);
        }
    }
}

#[test]
fn test_serialize_deserialize_all_mouse_event_types() {
    // 测试所有鼠标事件类型
    let events = vec![
        (MouseEventType::Click, "click"),
        (MouseEventType::Down, "mousedown"),
        (MouseEventType::Up, "mouseup"),
        (MouseEventType::Move, "mousemove"),
        (MouseEventType::DblClick, "dblclick"),
    ];

    for (event_type, desc) in events {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 100.0,
                y: 200.0,
                button: 2,
                event_type,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::MouseEvent(p) = out.kind {
            assert_eq!(event_type, p.event_type);
            assert_eq!(100.0, p.x);
            assert_eq!(200.0, p.y);
            assert_eq!(2, p.button);
        } else {
            panic!("expected MouseEvent for {}", desc);
        }
    }
}

#[test]
fn test_serialize_deserialize_empty_headers() {
    // 测试空的头列表
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 1,
            status_code: 200,
            headers: vec![],
            body: b"response body".to_vec(),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchResponse(p) = out.kind {
        assert_eq!(0, p.headers.len());
        assert_eq!(b"response body", p.body.as_slice());
    } else {
        panic!("expected FetchResponse");
    }
}

#[test]
fn test_serialize_deserialize_duplicate_headers() {
    // 测试重复的头字段（这在 HTTP 中是允许的）
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 1,
            url: "https://example.com".to_string(),
            method: "GET".to_string(),
            headers: vec![
                ("Cookie".to_string(), "session=abc123".to_string()),
                ("Cookie".to_string(), "user=def456".to_string()),
                ("X-Custom".to_string(), "value1".to_string()),
                ("X-Custom".to_string(), "value2".to_string()),
            ],
            body: None,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchRequest(p) = out.kind {
        assert_eq!(4, p.headers.len());
        // 应该保留重复的头
        let cookies: Vec<_> = p.headers.iter().filter(|(k, _)| k == "Cookie").collect();
        assert_eq!(2, cookies.len());
    } else {
        panic!("expected FetchRequest");
    }
}

#[test]
fn test_serialize_deserialize_whitespace_in_strings() {
    // 测试字符串中的空白字符
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "  https://example.com/path  ".to_string(),
            referrer: Some("  https://referrer.com  ".to_string()),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::Navigate(p) = out.kind {
        assert_eq!("  https://example.com/path  ", p.url);
        assert_eq!(Some("  https://referrer.com  ".to_string()), p.referrer);
    } else {
        panic!("expected Navigate");
    }
}

#[test]
fn test_serialize_deserialize_special_bytes_in_body() {
    // 测试包含特殊字节的消息体
    let special_bytes = vec![
        0x00, 0x01, 0x7F,  // ASCII 控制字符
        0x80, 0xFF,        // 非法 UTF-8 序列的开始
        0xC0, 0x80,        // Overlong encoding
        0x4A, 0x6F, 0x68, 0x6E, // UTF-8 for "John"
    ];
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 1,
            status_code: 200,
            headers: vec![],
            body: special_bytes.clone(),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchResponse(p) = out.kind {
        assert_eq!(special_bytes, p.body);
    } else {
        panic!("expected FetchResponse");
    }
}

#[test]
fn test_serialize_deserialize_all_storage_operations() {
    // 测试所有存储操作类型
    let operations = vec![
        (StorageOperation::Get, "get"),
        (StorageOperation::Set, "set"),
        (StorageOperation::Remove, "remove"),
        (StorageOperation::Clear, "clear"),
        (StorageOperation::Length, "length"),
        (StorageOperation::Key, "key"),
    ];

    for (operation, desc) in operations {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation,
                key: "test_key".to_string(),
                value: match operation {
                    StorageOperation::Get | StorageOperation::Remove | StorageOperation::Length | StorageOperation::Key => None,
                    StorageOperation::Set | StorageOperation::Clear => Some("test_value".to_string()),
                },
                origin: "https://example.com".to_string(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::StorageOp(p) = out.kind {
            assert_eq!(operation, p.operation);
            match operation {
                StorageOperation::Get | StorageOperation::Remove | StorageOperation::Length | StorageOperation::Key => {
                    assert!(p.value.is_none());
                }
                StorageOperation::Set | StorageOperation::Clear => {
                    assert_eq!(Some("test_value".to_string()), p.value);
                }
            }
        } else {
            panic!("expected StorageOp for {}", desc);
        }
    }
}

#[test]
fn test_serialize_deserialize_all_storage_types() {
    // 测试所有存储类型
    let storage_types = vec![
        (StorageType::Local, "local"),
        (StorageType::Session, "session"),
    ];

    for (storage_type, desc) in storage_types {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type,
                operation: StorageOperation::Get,
                key: "key".to_string(),
                value: None,
                origin: "https://example.com".to_string(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::StorageOp(p) = out.kind {
            assert_eq!(storage_type, p.storage_type);
        } else {
            panic!("expected StorageOp for {}", desc);
        }
    }
}

#[test]
fn test_serialize_deserialize_inconsistent_data_handling() {
    // 测试当数据无法反序列化时的情况
    // 这里主要确保 deserialize 函数能优雅地处理各种错误

    // 测试无效的魔术字节
    let invalid_data = vec![0x00, 0x00, 0x00, 0x00];
    let result = deserialize(&invalid_data);
    assert!(result.is_err());

    // 测试截断的数据
    let valid_msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Ok,
    };
    let valid_bytes = serialize(&valid_msg).unwrap();
    let truncated_data = &valid_bytes[..valid_bytes.len() / 2];
    let result = deserialize(truncated_data);
    assert!(result.is_err());
}

#[test]
fn test_serialize_consistency() {
    // 测试序列化的一致性：相同消息应产生相同的字节
    let msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 1,
            status_code: 200,
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: b"{\"test\": true}".to_vec(),
        }),
    };

    let bytes1 = serialize(&msg).unwrap();
    let bytes2 = serialize(&msg).unwrap();
    assert_eq!(bytes1, bytes2);

    // 反序列化也应产生相同的消息
    let msg1: IpcMessage = deserialize(&bytes1).unwrap();
    let msg2: IpcMessage = deserialize(&bytes2).unwrap();
    assert_eq!(msg1.id, msg2.id);
    assert!(matches!(&msg1.kind, &IpcMessageKind::FetchResponse(_)));
    assert!(matches!(&msg2.kind, &IpcMessageKind::FetchResponse(_)));
}

#[test]
fn test_serialize_deserialize_zero_values() {
    // 测试包含零值的场景
    let msg = IpcMessage {
        id: 0,
        kind: IpcMessageKind::MouseEvent(MouseEventParams {
            x: 0.0,
            y: 0.0,
            button: 0,
            event_type: MouseEventType::Click,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::MouseEvent(p) = out.kind {
        assert_eq!(0.0, p.x);
        assert_eq!(0.0, p.y);
        assert_eq!(0, p.button);
    } else {
        panic!("expected MouseEvent");
    }
}
//! IpcChannel 契约测试、序列化确定性、对抗性反序列化与高级场景测试。

use super::*;

// ══════════════════════════════════════════════════════════
//  新增测试：IpcChannel 契约、序列化确定性、对抗性反序列化
// ══════════════════════════════════════════════════════════

#[test]
fn test_ipc_channel_mock_send_recv() {
    let mut ch = MockChannel::new();

    let msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: Some("https://ref.com".into()),
            navigation_epoch: 0,
        }),
    };

    // send → recv：消息保持不变
    ch.send(msg.clone()).expect("send");
    let out = ch.recv().expect("recv");
    assert_eq!(msg.id, out.id);
    if let IpcMessageKind::Navigate(p) = out.kind {
        assert_eq!("https://example.com", p.url);
        assert_eq!(Some("https://ref.com".into()), p.referrer);
    } else {
        panic!("expected Navigate");
    }

    // 空通道 recv 应返回错误
    assert!(ch.recv().is_err());
}

#[test]
fn test_ipc_channel_mock_try_recv() {
    let mut ch = MockChannel::new();

    // 空通道 try_recv 返回 Ok(None)
    assert!(ch.try_recv().expect("try_recv empty").is_none());

    ch.send(IpcMessage {
        id: 1,
        kind: IpcMessageKind::Heartbeat,
    })
    .expect("send");

    // 有消息时 try_recv 返回 Ok(Some(..))
    let opt = ch.try_recv().expect("try_recv");
    assert!(opt.is_some());
    assert_eq!(1, opt.unwrap().id);

    // 再取一次应为空
    assert!(ch.try_recv().expect("try_recv again").is_none());
}

#[test]
fn test_ipc_channel_mock_close() {
    let mut ch = MockChannel::new();

    ch.send(IpcMessage {
        id: 1,
        kind: IpcMessageKind::Ok,
    })
    .expect("send before close");

    ch.close();

    // 关闭后 send 应失败
    let res = ch.send(IpcMessage {
        id: 2,
        kind: IpcMessageKind::Ok,
    });
    assert!(res.is_err());

    // 关闭后 recv 也应失败（即使队列有消息）
    assert!(ch.recv().is_err());
    assert!(ch.try_recv().is_err());
}

#[test]
fn test_ipc_channel_mock_fifo_order() {
    let mut ch = MockChannel::new();

    for i in 0u64..5 {
        ch.send(IpcMessage {
            id: i,
            kind: IpcMessageKind::Heartbeat,
        })
        .expect("send");
    }

    for i in 0u64..5 {
        let msg = ch.recv().expect("recv");
        assert_eq!(i, msg.id, "FIFO order violated");
    }
}

#[test]
fn test_serialization_deterministic() {
    let msgs = vec![
        IpcMessage {
            id: 100,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com".into(),
                referrer: None,
                navigation_epoch: 0,
            }),
        },
        IpcMessage {
            id: 200,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 1,
                url: "https://api.com".into(),
                method: "POST".into(),
                headers: vec![("X-Foo".into(), "bar".into())],
                body: Some(vec![1, 2, 3]),
            }),
        },
        IpcMessage {
            id: 300,
            kind: IpcMessageKind::Heartbeat,
        },
    ];

    for msg in &msgs {
        let b1 = serialize(msg).expect("serialize 1");
        let b2 = serialize(msg).expect("serialize 2");
        assert_eq!(b1, b2, "deterministic encoding violated for message id={}", msg.id);
    }
}

#[test]
fn test_deserialization_trailing_bytes() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Ok,
    };
    let mut bytes = serialize(&msg).expect("serialize");

    // 在有效载荷末尾追加额外字节
    bytes.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

    // bincode 1.x 默认配置不拒绝尾部多余数据，会成功解析并忽略尾部。
    // 记录此行为：反序列化成功，消息内容不变。
    let result = deserialize(&bytes);
    assert!(
        result.is_ok(),
        "bincode 1.x default config accepts trailing bytes — document this behavior"
    );
    let out = result.expect("deserialize");
    assert_eq!(1, out.id);
    assert!(matches!(out.kind, IpcMessageKind::Ok));
}

#[test]
fn test_deserialization_random_bytes() {
    // 各种长度的随机/对抗性字节，反序列化均应返回错误
    let cases: Vec<Vec<u8>> = vec![
        vec![],
        vec![0x00],
        vec![0xFF],
        vec![0x01, 0x02, 0x03],
        vec![0xDE, 0xAD, 0xBE, 0xEF],
        vec![0xFF; 8],
        vec![0xFF; 16],
        vec![0xFF; 32],
        vec![0xFF; 64],
        vec![0xFF; 256],
        // 看起来像合法 bincode 头但内容不完整
        vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    ];

    for (i, bytes) in cases.iter().enumerate() {
        let result = deserialize(bytes);
        assert!(
            result.is_err(),
            "random bytes case {i} (len={}) should fail to deserialize",
            bytes.len()
        );
    }
}

#[test]
fn test_different_message_types_interleaved() {
    let msgs: Vec<IpcMessage> = vec![
        IpcMessage {
            id: 0,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com".into(),
                referrer: None,
                navigation_epoch: 0,
            }),
        },
        IpcMessage {
            id: 1,
            kind: IpcMessageKind::Heartbeat,
        },
        IpcMessage {
            id: 2,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 1,
                url: "https://api.com".into(),
                method: "GET".into(),
                headers: vec![],
                body: None,
            }),
        },
        IpcMessage {
            id: 3,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Set,
                key: "k".into(),
                value: Some("v".into()),
                origin: "https://example.com".into(),
            }),
        },
        IpcMessage {
            id: 4,
            kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: "a".into(),
                code: "KeyA".into(),
                ctrl: false,
                shift: false,
                alt: false,
                meta: false,
                event_type: KeyboardEventType::Down,
            }),
        },
        IpcMessage {
            id: 5,
            kind: IpcMessageKind::Ok,
        },
    ];

    // Serialize all, then deserialize all — interleaved types must not corrupt each other
    let encoded: Vec<Vec<u8>> = msgs.iter().map(|m| serialize(m).expect("s")).collect();
    for (i, bytes) in encoded.iter().enumerate() {
        let out = deserialize(bytes).expect("d");
        assert_eq!(i as u64, out.id);
    }
}

// ══════════════════════════════════════════════════════════
//  IPC 消息边界条件测试
// ══════════════════════════════════════════════════════════

/// 测试大载荷（100KB 字符串）消息的序列化/反序列化往返正确性。
#[test]
fn test_ipc_message_large_payload() {
    let large_string: String = "A".repeat(100 * 1024); // 100KB
    let msg = IpcMessage {
        id: 999,
        kind: IpcMessageKind::TitleChanged(large_string.clone()),
    };
    let out = roundtrip(msg);
    assert_eq!(999, out.id);
    if let IpcMessageKind::TitleChanged(t) = out.kind {
        assert_eq!(100 * 1024, t.len());
        assert_eq!(large_string, t);
    } else {
        panic!("expected TitleChanged");
    }
}

/// 测试空字符串和零值字段的消息序列化/反序列化。
#[test]
fn test_ipc_message_empty_fields() {
    let msg = IpcMessage {
        id: 0,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Clear,
            key: String::new(),
            value: Some(String::new()),
            origin: String::new(),
        }),
    };
    let out = roundtrip(msg);
    assert_eq!(0, out.id);
    if let IpcMessageKind::StorageOp(p) = out.kind {
        assert!(p.key.is_empty());
        assert_eq!(Some(String::new()), p.value);
        assert!(p.origin.is_empty());
        assert_eq!(StorageOperation::Clear, p.operation);
    } else {
        panic!("expected StorageOp");
    }
}

/// 测试连续序列化多条不同消息后逐个反序列化，验证每条消息还原正确。
#[test]
fn test_ipc_message_concurrent_serialization() {
    let messages: Vec<IpcMessage> = vec![
        IpcMessage {
            id: 10,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com".into(),
                referrer: None,
                navigation_epoch: 0,
            }),
        },
        IpcMessage {
            id: 20,
            kind: IpcMessageKind::TitleChanged("测试标题".into()),
        },
        IpcMessage {
            id: 30,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 42.0,
                y: 99.5,
                button: 2,
                event_type: MouseEventType::DblClick,
            }),
        },
        IpcMessage {
            id: 40,
            kind: IpcMessageKind::Error("未知错误".into()),
        },
        IpcMessage {
            id: 50,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 7,
                url: "https://api.example.com/data".into(),
                method: "POST".into(),
                headers: vec![("Content-Type".into(), "application/json".into())],
                body: Some(b"{\"key\":\"value\"}".to_vec()),
            }),
        },
    ];

    // 连续序列化所有消息
    let serialized: Vec<Vec<u8>> = messages.iter().map(|m| serialize(m).expect("serialize")).collect();

    // 逐个反序列化并验证
    for (i, bytes) in serialized.iter().enumerate() {
        let out = deserialize(bytes).expect("deserialize");
        assert_eq!(messages[i].id, out.id, "消息 id 不匹配：索引 {i}");

        // 对每条消息按类型验证字段
        match (&messages[i].kind, &out.kind) {
            (IpcMessageKind::Navigate(a), IpcMessageKind::Navigate(b)) => {
                assert_eq!(a.url, b.url);
                assert_eq!(a.referrer, b.referrer);
            }
            (IpcMessageKind::TitleChanged(a), IpcMessageKind::TitleChanged(b)) => {
                assert_eq!(a, b);
            }
            (IpcMessageKind::MouseEvent(a), IpcMessageKind::MouseEvent(b)) => {
                assert_eq!(a.x, b.x);
                assert_eq!(a.y, b.y);
                assert_eq!(a.button, b.button);
                assert_eq!(a.event_type, b.event_type);
            }
            (IpcMessageKind::Error(a), IpcMessageKind::Error(b)) => {
                assert_eq!(a, b);
            }
            (IpcMessageKind::FetchRequest(a), IpcMessageKind::FetchRequest(b)) => {
                assert_eq!(a.request_id, b.request_id);
                assert_eq!(a.url, b.url);
                assert_eq!(a.method, b.method);
                assert_eq!(a.headers, b.headers);
                assert_eq!(a.body, b.body);
            }
            _ => panic!("消息类型不匹配：索引 {i}"),
        }
    }
}

/// 测试通过 mock 通道发送多条消息后按 FIFO 顺序接收。
#[test]
fn test_ipc_channel_ordering() {
    let mut ch = MockChannel::new();

    let messages: Vec<IpcMessage> = vec![
        IpcMessage {
            id: 1,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://a.com".into(),
                referrer: None,
                navigation_epoch: 0,
            }),
        },
        IpcMessage {
            id: 2,
            kind: IpcMessageKind::LoadComplete,
        },
        IpcMessage {
            id: 3,
            kind: IpcMessageKind::TitleChanged("页面标题".into()),
        },
        IpcMessage {
            id: 4,
            kind: IpcMessageKind::Heartbeat,
        },
        IpcMessage {
            id: 5,
            kind: IpcMessageKind::Error("超时".into()),
        },
    ];

    // 按顺序发送所有消息
    for msg in &messages {
        ch.send(msg.clone()).expect("send");
    }

    // 按 FIFO 顺序接收并验证
    for (i, expected) in messages.iter().enumerate() {
        let received = ch.recv().expect("recv");
        assert_eq!(
            expected.id, received.id,
            "FIFO 顺序违反：期望 id={}，实际 id={}（索引 {}）",
            expected.id, received.id, i
        );

        // 验证消息类型匹配
        match (&expected.kind, &received.kind) {
            (IpcMessageKind::Navigate(a), IpcMessageKind::Navigate(b)) => {
                assert_eq!(a.url, b.url);
            }
            (IpcMessageKind::TitleChanged(a), IpcMessageKind::TitleChanged(b)) => {
                assert_eq!(a, b);
            }
            (IpcMessageKind::Error(a), IpcMessageKind::Error(b)) => {
                assert_eq!(a, b);
            }
            (IpcMessageKind::LoadComplete, IpcMessageKind::LoadComplete)
            | (IpcMessageKind::Heartbeat, IpcMessageKind::Heartbeat) => {}
            _ => panic!("消息类型不匹配：索引 {i}"),
        }
    }

    // 所有消息已消费，再次 recv 应返回错误
    assert!(ch.recv().is_err());
}

// ══════════════════════════════════════════════════════════
//  IPC 压力测试与剩余边界条件测试
// ══════════════════════════════════════════════════════════

/// 测试超大消息（1MB+ 载荷）的序列化/反序列化往返正确性。
/// 验证 bincode 在大载荷下不会截断或损坏数据。
#[test]
fn test_large_message_serialization() {
    // 构造一个 1MB+ 的二进制 body（恰好 1MB + 1 字节以确保超过 1MB）
    let large_body: Vec<u8> = (0..1_048_577).map(|i| (i % 256) as u8).collect();
    assert!(large_body.len() > 1_048_576, "载荷必须超过 1MB");

    let msg = IpcMessage {
        id: u64::MAX,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: u64::MAX,
            status_code: 200,
            headers: vec![("Content-Length".into(), format!("{}", large_body.len()))],
            body: large_body.clone(),
        }),
    };

    let bytes = serialize(&msg).expect("序列化 1MB+ 消息应成功");
    // 序列化后的字节长度应明显大于原始 body
    assert!(
        bytes.len() > large_body.len(),
        "序列化结果应包含完整载荷，实际 {} 字节",
        bytes.len()
    );

    let out = deserialize(&bytes).expect("反序列化 1MB+ 消息应成功");
    assert_eq!(u64::MAX, out.id);
    if let IpcMessageKind::FetchResponse(p) = out.kind {
        assert_eq!(u64::MAX, p.request_id);
        assert_eq!(200, p.status_code);
        assert_eq!(large_body.len(), p.body.len());
        assert_eq!(large_body, p.body, "大载荷往返后内容应完全一致");
    } else {
        panic!("期望 FetchResponse");
    }
}

/// 测试多条不同类型消息的序列化/反序列化顺序保持不变。
/// 模拟高并发场景：连续序列化 100 条消息，反序列化后 ID 和类型顺序必须一致。
#[test]
fn test_concurrent_message_ordering() {
    let messages: Vec<IpcMessage> = (0..100)
        .map(|i| {
            let kind = match i % 5 {
                0 => IpcMessageKind::Navigate(NavigateParams {
                    url: format!("https://example.com/page/{i}"),
                    referrer: if i % 2 == 0 {
                        Some("https://referrer.com".into())
                    } else {
                        None
                    },
                    navigation_epoch: 0,
                }),
                1 => IpcMessageKind::FetchRequest(FetchParams {
                    request_id: i as u64,
                    url: format!("https://api.example.com/{i}"),
                    method: "GET".into(),
                    headers: vec![],
                    body: None,
                }),
                2 => IpcMessageKind::MouseEvent(MouseEventParams {
                    x: i as f32 * 1.5,
                    y: i as f32 * 2.5,
                    button: (i % 3) as u8,
                    event_type: MouseEventType::Click,
                }),
                3 => IpcMessageKind::TitleChanged(format!("页面标题 #{i}")),
                _ => IpcMessageKind::Heartbeat,
            };
            IpcMessage { id: i as u64, kind }
        })
        .collect();

    // 连续序列化所有消息
    let serialized: Vec<Vec<u8>> = messages.iter().map(|m| serialize(m).expect("序列化应成功")).collect();

    assert_eq!(messages.len(), serialized.len());

    // 逐个反序列化，验证顺序和内容
    for (i, bytes) in serialized.iter().enumerate() {
        let out = deserialize(bytes).expect("反序列化应成功");
        assert_eq!(i as u64, out.id, "消息顺序不一致：期望 id={}，实际 id={}", i, out.id);

        // 按消息类型验证关键字段
        match (&messages[i].kind, &out.kind) {
            (IpcMessageKind::Navigate(a), IpcMessageKind::Navigate(b)) => {
                assert_eq!(a.url, b.url, "Navigate url 不匹配，索引 {i}");
                assert_eq!(a.referrer, b.referrer, "Navigate referrer 不匹配，索引 {i}");
            }
            (IpcMessageKind::FetchRequest(a), IpcMessageKind::FetchRequest(b)) => {
                assert_eq!(a.request_id, b.request_id, "FetchRequest request_id 不匹配，索引 {i}");
                assert_eq!(a.url, b.url, "FetchRequest url 不匹配，索引 {i}");
            }
            (IpcMessageKind::MouseEvent(a), IpcMessageKind::MouseEvent(b)) => {
                assert_eq!(a.x, b.x, "MouseEvent x 不匹配，索引 {i}");
                assert_eq!(a.y, b.y, "MouseEvent y 不匹配，索引 {i}");
                assert_eq!(a.button, b.button, "MouseEvent button 不匹配，索引 {i}");
            }
            (IpcMessageKind::TitleChanged(a), IpcMessageKind::TitleChanged(b)) => {
                assert_eq!(a, b, "TitleChanged 不匹配，索引 {i}");
            }
            (IpcMessageKind::Heartbeat, IpcMessageKind::Heartbeat) => {}
            _ => panic!("消息类型不匹配：索引 {i}"),
        }
    }
}

/// 测试所有可选字段均为空值时的消息序列化/反序列化。
/// 覆盖 None、空字符串、空 Vec 等边界情况。
#[test]
fn test_message_with_empty_fields() {
    // Navigate：url 为空字符串，referrer 为 None
    let msg1 = IpcMessage {
        id: 0,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: String::new(),
            referrer: None,
            navigation_epoch: 0,
        }),
    };
    let out1 = roundtrip(msg1);
    if let IpcMessageKind::Navigate(p) = out1.kind {
        assert!(p.url.is_empty(), "url 应为空字符串");
        assert!(p.referrer.is_none(), "referrer 应为 None");
    } else {
        panic!("期望 Navigate");
    }

    // FetchRequest：空 headers，body 为 None
    let msg2 = IpcMessage {
        id: 0,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 0,
            url: String::new(),
            method: String::new(),
            headers: vec![],
            body: None,
        }),
    };
    let out2 = roundtrip(msg2);
    if let IpcMessageKind::FetchRequest(p) = out2.kind {
        assert_eq!(0, p.request_id);
        assert!(p.url.is_empty());
        assert!(p.method.is_empty());
        assert!(p.headers.is_empty());
        assert!(p.body.is_none());
    } else {
        panic!("期望 FetchRequest");
    }

    // FetchResponse：空 headers，空 body
    let msg3 = IpcMessage {
        id: 0,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 0,
            status_code: 0,
            headers: vec![],
            body: vec![],
        }),
    };
    let out3 = roundtrip(msg3);
    if let IpcMessageKind::FetchResponse(p) = out3.kind {
        assert_eq!(0, p.status_code);
        assert!(p.headers.is_empty());
        assert!(p.body.is_empty());
    } else {
        panic!("期望 FetchResponse");
    }

    // StorageOp：value 为 None，key 和 origin 为空字符串
    let msg4 = IpcMessage {
        id: 0,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Get,
            key: String::new(),
            value: None,
            origin: String::new(),
        }),
    };
    let out4 = roundtrip(msg4);
    if let IpcMessageKind::StorageOp(p) = out4.kind {
        assert!(p.key.is_empty());
        assert!(p.value.is_none());
        assert!(p.origin.is_empty());
    } else {
        panic!("期望 StorageOp");
    }

    // MouseEvent 和 KeyboardEvent：零值坐标/按键
    let msg5 = IpcMessage {
        id: 0,
        kind: IpcMessageKind::MouseEvent(MouseEventParams {
            x: 0.0,
            y: 0.0,
            button: 0,
            event_type: MouseEventType::Click,
        }),
    };
    let out5 = roundtrip(msg5);
    if let IpcMessageKind::MouseEvent(p) = out5.kind {
        assert_eq!(0.0, p.x);
        assert_eq!(0.0, p.y);
        assert_eq!(0, p.button);
    } else {
        panic!("期望 MouseEvent");
    }
}

/// 测试包含 Unicode 字符串的消息序列化/反序列化往返正确性。
/// 覆盖中日韩文字、emoji、组合字符、特殊 Unicode 等。
#[test]
fn test_message_with_unicode_payload() {
    let long_ascii = "x".repeat(1000);
    let unicode_cases: Vec<&str> = vec![
        // 中文
        "这是一个中文标题，包含标点符号：【】《》、。，！？",
        // 日文
        "こんにちは世界 🌍 日本語テスト",
        // 韩文
        "안녕하세요 세계",
        // emoji 丰富文本
        "🎉🚀💻🔒 ✓ — Unicode 测试 © 2024",
        // 混合 RTL/LTR
        "Hello مرحبا العالم שלום",
        // 组合字符（é 可以是 e + ́ ）
        "caf\u{0065}\u{0301} = café",
        // 零宽字符
        "abc\u{200B}\u{200C}\u{200D}def",
        // 四字节 emoji
        "👨‍👩‍👧‍👦 family emoji",
        // 空字符串
        "",
        // 纯 ASCII 但很长的字符串
        &long_ascii,
    ];

    for (i, text) in unicode_cases.iter().enumerate() {
        // TitleChanged
        let msg = IpcMessage {
            id: i as u64,
            kind: IpcMessageKind::TitleChanged(text.to_string()),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::TitleChanged(t) = out.kind {
            assert_eq!(text, &t, "TitleChanged Unicode 往返失败：索引 {i}");
        } else {
            panic!("期望 TitleChanged，索引 {i}");
        }

        // UrlChanged
        let msg2 = IpcMessage {
            id: i as u64 + 100,
            kind: IpcMessageKind::UrlChanged(text.to_string()),
        };
        let out2 = roundtrip(msg2);
        if let IpcMessageKind::UrlChanged(u) = out2.kind {
            assert_eq!(text, &u, "UrlChanged Unicode 往返失败：索引 {i}");
        } else {
            panic!("期望 UrlChanged，索引 {i}");
        }

        // CrashNotification
        let msg3 = IpcMessage {
            id: i as u64 + 200,
            kind: IpcMessageKind::CrashNotification(text.to_string()),
        };
        let out3 = roundtrip(msg3);
        if let IpcMessageKind::CrashNotification(r) = out3.kind {
            assert_eq!(text, &r, "CrashNotification Unicode 往返失败：索引 {i}");
        } else {
            panic!("期望 CrashNotification，索引 {i}");
        }

        // Error
        let msg4 = IpcMessage {
            id: i as u64 + 300,
            kind: IpcMessageKind::Error(text.to_string()),
        };
        let out4 = roundtrip(msg4);
        if let IpcMessageKind::Error(e) = out4.kind {
            assert_eq!(text, &e, "Error Unicode 往返失败：索引 {i}");
        } else {
            panic!("期望 Error，索引 {i}");
        }

        // Navigate url 字段
        let msg5 = IpcMessage {
            id: i as u64 + 400,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: text.to_string(),
                referrer: Some(text.to_string()),
                navigation_epoch: 0,
            }),
        };
        let out5 = roundtrip(msg5);
        if let IpcMessageKind::Navigate(p) = out5.kind {
            assert_eq!(text, &p.url, "Navigate url Unicode 往返失败：索引 {i}");
            assert_eq!(
                &Some(text.to_string()),
                &p.referrer,
                "Navigate referrer Unicode 往返失败：索引 {i}"
            );
        } else {
            panic!("期望 Navigate，索引 {i}");
        }

        // FetchParams headers 中的 Unicode 键值对
        let msg6 = IpcMessage {
            id: i as u64 + 500,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: i as u64,
                url: text.to_string(),
                method: text.to_string(),
                headers: vec![
                    (text.to_string(), text.to_string()),
                    ("ASCII-Key".into(), text.to_string()),
                ],
                body: Some(text.as_bytes().to_vec()),
            }),
        };
        let out6 = roundtrip(msg6);
        if let IpcMessageKind::FetchRequest(p) = out6.kind {
            assert_eq!(text, &p.url, "FetchParams url Unicode 往返失败：索引 {i}");
            assert_eq!(text, &p.method, "FetchParams method Unicode 往返失败：索引 {i}");
            assert_eq!(2, p.headers.len(), "FetchParams headers 长度不匹配：索引 {i}");
            assert_eq!(
                text, &p.headers[0].0,
                "FetchParams header key Unicode 往返失败：索引 {i}"
            );
            assert_eq!(
                text, &p.headers[0].1,
                "FetchParams header value Unicode 往返失败：索引 {i}"
            );
            assert_eq!(
                Some(text.as_bytes().to_vec()),
                p.body,
                "FetchParams body Unicode 往返失败：索引 {i}"
            );
        } else {
            panic!("期望 FetchRequest，索引 {i}");
        }

        // StorageOp key/value/origin 全部 Unicode
        let msg7 = IpcMessage {
            id: i as u64 + 600,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Session,
                operation: StorageOperation::Set,
                key: text.to_string(),
                value: Some(text.to_string()),
                origin: text.to_string(),
            }),
        };
        let out7 = roundtrip(msg7);
        if let IpcMessageKind::StorageOp(p) = out7.kind {
            assert_eq!(text, &p.key, "StorageOp key Unicode 往返失败：索引 {i}");
            assert_eq!(
                &Some(text.to_string()),
                &p.value,
                "StorageOp value Unicode 往返失败：索引 {i}"
            );
            assert_eq!(text, &p.origin, "StorageOp origin Unicode 往返失败：索引 {i}");
        } else {
            panic!("期望 StorageOp，索引 {i}");
        }
    }
}

/// 测试包含多层嵌套结构的消息序列化/反序列化往返。
/// IpcMessage 本身包含嵌套的 params 结构体（含 Vec<(String, String)> 等），
/// 验证这些嵌套层级在 bincode 编码/解码后完整还原。
#[test]
fn test_nested_message_round_trip() {
    // 构造具有多层嵌套的消息：
    // IpcMessage -> FetchRequest -> headers: Vec<(String, String)> (多层嵌套)
    //                                  body: Option<Vec<u8>>
    let nested_headers: Vec<(String, String)> = (0..20)
        .map(|i| {
            let nested_value = format!(
                "level0={i}; level1={{nested_{i}: {{deep: true}}}}; arr=[{},{},{}]",
                i,
                i + 1,
                i + 2
            );
            (format!("X-Nested-Key-{i}"), nested_value)
        })
        .collect();

    // 嵌套的二进制 body，内部包含一个模拟的 JSON 结构
    let nested_body: Vec<u8> = format!(
        "{{\"outer\":{{\"inner\":[{}]}}}}",
        (0..10)
            .map(|i| format!("{{\"id\":{i},\"value\":\"nested_{i}\"}}"))
            .collect::<Vec<_>>()
            .join(",")
    )
    .into_bytes();

    let msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 12345,
            url: "https://example.com/api/v1/deeply/nested/endpoint".into(),
            method: "POST".into(),
            headers: nested_headers.clone(),
            body: Some(nested_body.clone()),
        }),
    };

    let out = roundtrip(msg);
    if let IpcMessageKind::FetchRequest(p) = out.kind {
        assert_eq!(12345, p.request_id);
        assert_eq!("POST", p.method);
        assert_eq!(20, p.headers.len(), "嵌套 headers 长度应保持一致");
        assert_eq!(nested_headers, p.headers, "嵌套 headers 内容应完全一致");
        assert_eq!(Some(nested_body), p.body, "嵌套 body 内容应完全一致");
    } else {
        panic!("期望 FetchRequest");
    }

    // 测试更深层的嵌套：StorageOp 中 value 包含嵌套 JSON 字符串
    // 手动构造嵌套 JSON 字符串（不依赖 serde_json，直接拼字符串）
    let deeply_nested_value = format!(
        "{{\"users\":[{}]}}",
        (0..5)
            .map(|i| format!(
                "{{\"name\":\"user_{i}\",\"settings\":{{\"theme\":\"dark\",\"lang\":\"zh\",\"tags\":[\"a\",\"b\",\"c\"]}}}}"
            ))
            .collect::<Vec<_>>()
            .join(",")
    );

    let msg2 = IpcMessage {
        id: 43,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Set,
            key: "deeply_nested_key".into(),
            value: Some(deeply_nested_value.clone()),
            origin: "https://example.com".into(),
        }),
    };

    let out2 = roundtrip(msg2);
    if let IpcMessageKind::StorageOp(p) = out2.kind {
        assert_eq!(Some(deeply_nested_value), p.value, "深层嵌套 value 往返应一致");
    } else {
        panic!("期望 StorageOp");
    }

    // 测试 FetchResponse 中 headers 嵌套元组的往返
    let response_headers: Vec<(String, String)> = vec![
        ("Content-Type".into(), "text/html; charset=utf-8".into()),
        ("Set-Cookie".into(), "session=abc123; Path=/; HttpOnly; Secure".into()),
        ("X-Forwarded-For".into(), "10.0.0.1, 172.16.0.1, 192.168.1.1".into()),
        ("Link".into(), "</api/page/2>; rel=\"next\"; title=\"下一页\"".into()),
    ];

    let msg3 = IpcMessage {
        id: 44,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 99999,
            status_code: 206,
            headers: response_headers.clone(),
            body: vec![0u8; 1024], // 1KB 零填充 body
        }),
    };

    let out3 = roundtrip(msg3);
    if let IpcMessageKind::FetchResponse(p) = out3.kind {
        assert_eq!(99999, p.request_id);
        assert_eq!(206, p.status_code);
        assert_eq!(response_headers, p.headers, "嵌套响应 headers 往返应一致");
        assert_eq!(1024, p.body.len());
        assert!(p.body.iter().all(|&b| b == 0), "body 应全为零");
    } else {
        panic!("期望 FetchResponse");
    }
}

/// 测试使用每种支持的字段类型的 IPC 消息的序列化/反序列化。
///
/// 构造一条消息，使用 u64、String、Option、Vec、enum variants、
/// bool、u8、f32 以及嵌套结构体——验证完整的往返行程。
#[test]
fn test_ipc_message_with_all_field_types() {
    // 消息，该消息使用了每一种支持的字段类型
    let msg = IpcMessage {
        id: u64::MAX, // u64
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 42,                                           // u64
            url: "https://example.com/api?v=1&lang=中文#frag".into(), // 带有 unicode 的 String
            method: "POST".into(),                                    // String
            headers: vec![
                // Vec<(String, String)>
                ("Content-Type".into(), "application/json".into()),
                ("X-Empty".into(), String::new()), // 空字符串
                ("X-Unicode".into(), "Ñoño café ☕".into()),
            ],
            body: Some(vec![0u8, 127, 255, 0xDE, 0xAD]), // Option<Vec<u8>>
        }),
    };
    let bytes = serialize(&msg).expect("serialize");
    let out = deserialize(&bytes).expect("deserialize");

    assert_eq!(u64::MAX, out.id);
    if let IpcMessageKind::FetchRequest(p) = out.kind {
        assert_eq!(42, p.request_id);
        assert_eq!("https://example.com/api?v=1&lang=中文#frag", p.url);
        assert_eq!("POST", p.method);
        assert_eq!(3, p.headers.len());
        assert_eq!("application/json", p.headers[0].1);
        assert!(p.headers[1].1.is_empty());
        assert_eq!("Ñoño café ☕", p.headers[2].1);
        assert_eq!(Some(vec![0u8, 127, 255, 0xDE, 0xAD]), p.body);
    } else {
        panic!("expected FetchRequest");
    }

    // 测试鼠标事件字段：f32 x/y, u8 按钮, 枚举 event_type
    let mouse_msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::MouseEvent(MouseEventParams {
            x: -100.5,
            y: 0.0,
            button: u8::MAX,
            event_type: MouseEventType::Move,
        }),
    };
    let out2 = roundtrip(mouse_msg);
    if let IpcMessageKind::MouseEvent(p) = out2.kind {
        assert_eq!(-100.5, p.x);
        assert_eq!(0.0, p.y);
        assert_eq!(u8::MAX, p.button);
        assert_eq!(MouseEventType::Move, p.event_type);
    } else {
        panic!("expected MouseEvent");
    }

    // 测试键盘事件字段：bool 修饰键
    let kb_msg = IpcMessage {
        id: 2,
        kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
            key: String::new(),
            code: String::new(),
            ctrl: true,
            shift: true,
            alt: true,
            meta: true,
            event_type: KeyboardEventType::Up,
        }),
    };
    let out3 = roundtrip(kb_msg);
    if let IpcMessageKind::KeyboardEvent(p) = out3.kind {
        assert!(p.ctrl && p.shift && p.alt && p.meta);
        assert!(p.key.is_empty() && p.code.is_empty());
    } else {
        panic!("expected KeyboardEvent");
    }

    // 测试存储操作字段：枚举存储类型 + 操作
    let store_msg = IpcMessage {
        id: 3,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Session,
            operation: StorageOperation::Remove,
            key: "k".into(),
            value: None,
            origin: "https://example.com".into(),
        }),
    };
    let out4 = roundtrip(store_msg);
    if let IpcMessageKind::StorageOp(p) = out4.kind {
        assert_eq!(StorageType::Session, p.storage_type);
        assert_eq!(StorageOperation::Remove, p.operation);
        assert!(p.value.is_none());
    } else {
        panic!("expected StorageOp");
    }

    // 测试滚动事件字段：f32 增量
    let scroll_msg = IpcMessage {
        id: 4,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: f32::MAX,
            delta_y: f32::MIN,
            ..Default::default()
        }),
    };
    let out5 = roundtrip(scroll_msg);
    if let IpcMessageKind::ScrollEvent(p) = out5.kind {
        assert_eq!(f32::MAX, p.delta_x);
        assert_eq!(f32::MIN, p.delta_y);
    } else {
        panic!("expected ScrollEvent");
    }

    // 测试 FetchResponse 字段：u16 状态码, Vec<u8> 主体
    let resp_msg = IpcMessage {
        id: 5,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 0,
            status_code: 418,
            headers: vec![],
            body: vec![],
        }),
    };
    let out6 = roundtrip(resp_msg);
    if let IpcMessageKind::FetchResponse(p) = out6.kind {
        assert_eq!(418, p.status_code);
        assert!(p.body.is_empty() && p.headers.is_empty());
    } else {
        panic!("expected FetchResponse");
    }
}

/// 测试 IPC 向后兼容性：添加新变体不会破坏旧的反序列化。
///
/// 在 `IpcMessageKind` 中添加新变体不会改变现有变体的序列化形式。
/// 序列化一条旧样式的消息，验证它仍然能正确反序列化。
#[test]
fn test_ipc_backward_compatibility() {
    // 序列化一条包含所有当前变体的消息
    let old_kinds: Vec<IpcMessageKind> = vec![
        IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: None,
            navigation_epoch: 0,
        }),
        IpcMessageKind::GoBack,
        IpcMessageKind::GoForward,
        IpcMessageKind::StopLoading,
        IpcMessageKind::Reload,
        IpcMessageKind::TitleChanged("Test".into()),
        IpcMessageKind::UrlChanged("https://example.com".into()),
        IpcMessageKind::LoadComplete,
        IpcMessageKind::LoadFailed("timeout".into()),
        IpcMessageKind::Heartbeat,
        IpcMessageKind::CrashNotification("segfault".into()),
        IpcMessageKind::Ok,
        IpcMessageKind::Error("failure".into()),
    ];

    // 序列化所有旧的消息并存储字节
    let serialized: Vec<(u64, Vec<u8>, IpcMessageKind)> = old_kinds
        .into_iter()
        .enumerate()
        .map(|(i, kind)| {
            let msg = IpcMessage {
                id: i as u64,
                kind: kind.clone(),
            };
            let bytes = serialize(&msg).expect("serialize");
            (i as u64, bytes, kind)
        })
        .collect();

    // 反序列化每个消息并验证所有字段完好无损
    for (id, bytes, original_kind) in &serialized {
        let out: IpcMessage = deserialize(bytes).expect("deserialize");
        assert_eq!(*id, out.id, "id mismatch for message {id}");

        // 按类型验证原始内容与往返行程完全匹配
        match (original_kind, &out.kind) {
            (IpcMessageKind::Navigate(a), IpcMessageKind::Navigate(b)) => {
                assert_eq!(a.url, b.url);
                assert_eq!(a.referrer, b.referrer);
            }
            (IpcMessageKind::TitleChanged(a), IpcMessageKind::TitleChanged(b)) => {
                assert_eq!(a, b);
            }
            (IpcMessageKind::UrlChanged(a), IpcMessageKind::UrlChanged(b)) => {
                assert_eq!(a, b);
            }
            (IpcMessageKind::LoadFailed(a), IpcMessageKind::LoadFailed(b)) => {
                assert_eq!(a, b);
            }
            (IpcMessageKind::CrashNotification(a), IpcMessageKind::CrashNotification(b)) => {
                assert_eq!(a, b);
            }
            (IpcMessageKind::Error(a), IpcMessageKind::Error(b)) => {
                assert_eq!(a, b);
            }
            (IpcMessageKind::GoBack, IpcMessageKind::GoBack)
            | (IpcMessageKind::GoForward, IpcMessageKind::GoForward)
            | (IpcMessageKind::StopLoading, IpcMessageKind::StopLoading)
            | (IpcMessageKind::Reload, IpcMessageKind::Reload)
            | (IpcMessageKind::LoadComplete, IpcMessageKind::LoadComplete)
            | (IpcMessageKind::Heartbeat, IpcMessageKind::Heartbeat)
            | (IpcMessageKind::Ok, IpcMessageKind::Ok) => {}
            _ => panic!("kind mismatch for message {id}"),
        }

        // 重新序列化并验证字节是相同的（往返行程的确定性）
        let re_bytes = serialize(&out).expect("re-serialize");
        assert_eq!(bytes, &re_bytes, "byte mismatch for message {id}");
    }
}

/// 测试 IPC 消息空字符串载荷的序列化/反序列化往返。
#[test]
fn test_ipc_message_empty_payload() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::TitleChanged(String::new()),
    };
    let out = roundtrip(msg);
    assert_eq!(1, out.id);
    if let IpcMessageKind::TitleChanged(t) = out.kind {
        assert_eq!("", t);
    } else {
        panic!("expected TitleChanged");
    }
}

/// 测试 IPC 消息 Unicode 载荷（含中文和 emoji）的序列化/反序列化往返。
#[test]
fn test_ipc_message_unicode_payload() {
    let payload = "你好世界🌍";
    let msg = IpcMessage {
        id: 2,
        kind: IpcMessageKind::TitleChanged(payload.into()),
    };
    let out = roundtrip(msg);
    assert_eq!(2, out.id);
    if let IpcMessageKind::TitleChanged(t) = out.kind {
        assert_eq!(payload, t);
    } else {
        panic!("expected TitleChanged");
    }
}

/// 测试连续序列化 3 条消息后逐个反序列化，验证消息顺序保持不变。
#[test]
fn test_ipc_message_order_preserved() {
    let msgs = [
        IpcMessage {
            id: 10,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://a.com".into(),
                referrer: None,
                navigation_epoch: 0,
            }),
        },
        IpcMessage {
            id: 20,
            kind: IpcMessageKind::TitleChanged("second".into()),
        },
        IpcMessage {
            id: 30,
            kind: IpcMessageKind::Heartbeat,
        },
    ];
    let serialized: Vec<Vec<u8>> = msgs.iter().map(|m| serialize(m).expect("serialize")).collect();
    let deserialized: Vec<IpcMessage> = serialized
        .iter()
        .map(|b| deserialize(b).expect("deserialize"))
        .collect();
    assert_eq!(3, deserialized.len());
    assert_eq!(10, deserialized[0].id);
    assert_eq!(20, deserialized[1].id);
    assert_eq!(30, deserialized[2].id);
}

/// 测试同一消息序列化两次产生完全相同的二进制输出。
#[test]
fn test_ipc_deterministic_encoding() {
    let msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: Some("https://ref.com".into()),
            navigation_epoch: 0,
        }),
    };
    let bytes1 = serialize(&msg).expect("first serialize");
    let bytes2 = serialize(&msg).expect("second serialize");
    assert_eq!(bytes1, bytes2, "same message must produce identical byte output");
}

/// 测试 10KB 载荷消息的序列化/反序列化往返。
#[test]
fn test_ipc_large_payload() {
    let payload: String = "X".repeat(10_240); // 10KB
    let msg = IpcMessage {
        id: 99,
        kind: IpcMessageKind::TitleChanged(payload.clone()),
    };
    let out = roundtrip(msg);
    assert_eq!(99, out.id);
    if let IpcMessageKind::TitleChanged(t) = out.kind {
        assert_eq!(10_240, t.len());
        assert_eq!(payload, t);
    } else {
        panic!("expected TitleChanged");
    }
}

/// 测试同一消息多次序列化产生完全相同的二进制输出（bincode 确定性）。
/// 对于 IPC 协议的可靠性和幂等性至关重要。
#[test]
fn test_binary_round_trip_determinism() {
    let test_messages: Vec<IpcMessage> = vec![
        // 简单变体
        IpcMessage {
            id: 1,
            kind: IpcMessageKind::Ok,
        },
        IpcMessage {
            id: 2,
            kind: IpcMessageKind::Heartbeat,
        },
        // 带字符串的变体
        IpcMessage {
            id: 3,
            kind: IpcMessageKind::TitleChanged("确定性测试标题".into()),
        },
        IpcMessage {
            id: 4,
            kind: IpcMessageKind::Error("错误信息 123!@#".into()),
        },
        // 带可选字段的变体
        IpcMessage {
            id: 5,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com/path?q=hello&lang=zh".into(),
                referrer: Some("https://google.com".into()),
                navigation_epoch: 0,
            }),
        },
        IpcMessage {
            id: 6,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com".into(),
                referrer: None,
                navigation_epoch: 0,
            }),
        },
        // 带复杂数据结构的变体
        IpcMessage {
            id: 7,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 42,
                url: "https://api.example.com/v1/data".into(),
                method: "POST".into(),
                headers: vec![
                    ("Content-Type".into(), "application/json".into()),
                    ("Authorization".into(), "Bearer token123".into()),
                    ("X-Request-Id".into(), "abc-def-ghi".into()),
                ],
                body: Some(b"{\"key\":\"value\",\"nested\":{\"a\":1}}".to_vec()),
            }),
        },
        // 带枚举字段的变体
        IpcMessage {
            id: 8,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Session,
                operation: StorageOperation::Set,
                key: "test_key".into(),
                value: Some("test_value".into()),
                origin: "https://example.com".into(),
            }),
        },
        // 带浮点数的变体
        IpcMessage {
            id: 9,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 123.456,
                y: 789.012,
                button: 2,
                event_type: MouseEventType::DblClick,
            }),
        },
        IpcMessage {
            id: 10,
            kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
                delta_x: -3.14,
                delta_y: 2.718,
                ..Default::default()
            }),
        },
    ];

    for msg in &test_messages {
        // 序列化 3 次，验证每次输出完全一致
        let bytes1 = serialize(msg).expect("第一次序列化应成功");
        let bytes2 = serialize(msg).expect("第二次序列化应成功");
        let bytes3 = serialize(msg).expect("第三次序列化应成功");

        assert_eq!(
            bytes1, bytes2,
            "确定性违反：同一消息 (id={}) 的两次序列化结果不同",
            msg.id
        );
        assert_eq!(
            bytes2, bytes3,
            "确定性违反：同一消息 (id={}) 的第三次序列化与前两次不同",
            msg.id
        );

        // 反序列化后重新序列化，验证也产生相同字节（完整的往返确定性）
        let roundtrip_bytes = {
            let deserialized = deserialize(&bytes1).expect("反序列化应成功");
            serialize(&deserialized).expect("重新序列化应成功")
        };
        assert_eq!(
            bytes1, roundtrip_bytes,
            "往返确定性违反：消息 (id={}) 反序列化后重新序列化的结果与原始不同",
            msg.id
        );
    }
}

// ══════════════════════════════════════════════════════════
// 边界条件测试：所有枚举变体序列化、可选字段、Vec 字段
// ══════════════════════════════════════════════════════════

/// 测试所有 IpcMessageKind 枚举变体都能正确序列化和反序列化。
/// 构造每种变体的消息，序列化后反序列化，验证类型和 ID 保持一致。
#[test]
fn test_ipc_enum_variants() {
    let variants: Vec<(u64, IpcMessageKind)> = vec![
        (
            1,
            IpcMessageKind::Navigate(NavigateParams {
                url: "https://a.com".into(),
                referrer: Some("https://b.com".into()),
                navigation_epoch: 0,
            }),
        ),
        (2, IpcMessageKind::GoBack),
        (3, IpcMessageKind::GoForward),
        (4, IpcMessageKind::StopLoading),
        (5, IpcMessageKind::Reload),
        (6, IpcMessageKind::TitleChanged("title".into())),
        (7, IpcMessageKind::UrlChanged("https://c.com".into())),
        (8, IpcMessageKind::LoadComplete),
        (9, IpcMessageKind::LoadFailed("timeout".into())),
        (
            10,
            IpcMessageKind::FetchRequest(FetchParams {
                request_id: 1,
                url: "https://d.com".into(),
                method: "GET".into(),
                headers: vec![],
                body: None,
            }),
        ),
        (
            11,
            IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 1,
                status_code: 200,
                headers: vec![],
                body: vec![42],
            }),
        ),
        (
            12,
            IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Get,
                key: "k".into(),
                value: None,
                origin: "o".into(),
            }),
        ),
        (
            13,
            IpcMessageKind::MouseEvent(MouseEventParams {
                x: 1.0,
                y: 2.0,
                button: 0,
                event_type: MouseEventType::Click,
            }),
        ),
        (
            14,
            IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: "a".into(),
                code: "KeyA".into(),
                ctrl: false,
                shift: false,
                alt: false,
                meta: false,
                event_type: KeyboardEventType::Down,
            }),
        ),
        (
            15,
            IpcMessageKind::ScrollEvent(ScrollEventParams {
                delta_x: 1.0,
                delta_y: -1.0,
                ..Default::default()
            }),
        ),
        (16, IpcMessageKind::Heartbeat),
        (17, IpcMessageKind::CrashNotification("segfault".into())),
        (18, IpcMessageKind::Ok),
        (19, IpcMessageKind::Error("failure".into())),
    ];
    for (id, kind) in variants {
        let msg = IpcMessage { id, kind };
        let bytes = serialize(&msg).expect("serialize");
        let out = deserialize(&bytes).expect("deserialize");
        assert_eq!(id, out.id, "variant id={} should round-trip correctly", id);
        // Re-serialize to verify kind preserved
        let bytes2 = serialize(&out).expect("re-serialize");
        assert_eq!(
            bytes, bytes2,
            "variant id={} bytes should be identical after round-trip",
            id
        );
    }
}

/// 测试 IPC 消息中的可选字段（Option<T>）在 None 和 Some 两种情况下正确序列化。
#[test]
fn test_ipc_optional_fields() {
    // NavigateParams.referrer: None
    let msg_none = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: None,
            navigation_epoch: 0,
        }),
    };
    let out_none = roundtrip(msg_none);
    if let IpcMessageKind::Navigate(p) = out_none.kind {
        assert!(p.referrer.is_none(), "referrer 应为 None");
    } else {
        panic!("期望 Navigate");
    }

    // NavigateParams.referrer: Some
    let msg_some = IpcMessage {
        id: 2,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: Some("https://referrer.com".into()),
            navigation_epoch: 0,
        }),
    };
    let out_some = roundtrip(msg_some);
    if let IpcMessageKind::Navigate(p) = out_some.kind {
        assert_eq!(Some("https://referrer.com".into()), p.referrer, "referrer 应为 Some");
    } else {
        panic!("期望 Navigate");
    }

    // FetchParams.body: None vs Some
    let msg_body_none = IpcMessage {
        id: 3,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 1,
            url: "https://api.com".into(),
            method: "GET".into(),
            headers: vec![],
            body: None,
        }),
    };
    let out_body_none = roundtrip(msg_body_none);
    if let IpcMessageKind::FetchRequest(p) = out_body_none.kind {
        assert!(p.body.is_none(), "body 应为 None");
    } else {
        panic!("期望 FetchRequest");
    }

    let msg_body_some = IpcMessage {
        id: 4,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 2,
            url: "https://api.com".into(),
            method: "POST".into(),
            headers: vec![],
            body: Some(vec![1, 2, 3]),
        }),
    };
    let out_body_some = roundtrip(msg_body_some);
    if let IpcMessageKind::FetchRequest(p) = out_body_some.kind {
        assert_eq!(Some(vec![1, 2, 3]), p.body, "body 应为 Some([1,2,3])");
    } else {
        panic!("期望 FetchRequest");
    }

    // StorageOpParams.value: None vs Some
    let msg_val_none = IpcMessage {
        id: 5,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Get,
            key: "k".into(),
            value: None,
            origin: "https://example.com".into(),
        }),
    };
    let out_val_none = roundtrip(msg_val_none);
    if let IpcMessageKind::StorageOp(p) = out_val_none.kind {
        assert!(p.value.is_none(), "value 应为 None");
    } else {
        panic!("期望 StorageOp");
    }
}

/// 测试 IPC 消息中的 Vec 字段（headers、body）正确序列化。
#[test]
fn test_ipc_vec_field() {
    // FetchParams.headers: 多个键值对
    let headers: Vec<(String, String)> = vec![
        ("Accept".into(), "text/html".into()),
        ("Accept-Language".into(), "en-US".into()),
        ("Authorization".into(), "Bearer token".into()),
    ];
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 42,
            url: "https://example.com".into(),
            method: "GET".into(),
            headers: headers.clone(),
            body: Some(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchRequest(p) = out.kind {
        assert_eq!(p.headers.len(), 3, "headers 应有 3 个元素");
        assert_eq!(p.headers, headers, "headers 内容应完全一致");
        assert_eq!(p.body, Some(vec![0xDE, 0xAD, 0xBE, 0xEF]), "body 应完全一致");
    } else {
        panic!("期望 FetchRequest");
    }

    // FetchResponse.headers: 空列表 vs 多个
    let msg_empty = IpcMessage {
        id: 2,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 1,
            status_code: 200,
            headers: vec![],
            body: vec![],
        }),
    };
    let out_empty = roundtrip(msg_empty);
    if let IpcMessageKind::FetchResponse(p) = out_empty.kind {
        assert!(p.headers.is_empty(), "空 headers 应保持为空");
        assert!(p.body.is_empty(), "空 body 应保持为空");
    } else {
        panic!("期望 FetchResponse");
    }

    // FetchResponse.body: 大体积二进制
    let large_body: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    let msg_large = IpcMessage {
        id: 3,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 1,
            status_code: 200,
            headers: vec![("Content-Length".into(), "1024".into())],
            body: large_body.clone(),
        }),
    };
    let out_large = roundtrip(msg_large);
    if let IpcMessageKind::FetchResponse(p) = out_large.kind {
        assert_eq!(p.body.len(), 1024, "body 应有 1024 字节");
        assert_eq!(p.body, large_body, "大体积 body 应完全一致");
    } else {
        panic!("期望 FetchResponse");
    }
}

// ══════════════════════════════════════════════════════════
//  更多边界条件测试
// ══════════════════════════════════════════════════════════

/// 测试 f32 特殊浮点值（NaN、Infinity、负 Infinity）在鼠标和滚动事件中的往返正确性。
/// 验证 bincode 能正确编码/解码 IEEE 754 特殊值。
#[test]
fn test_mouse_and_scroll_special_float_values() {
    // 鼠标事件使用 NaN 和 Infinity
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::MouseEvent(MouseEventParams {
            x: f32::NAN,
            y: f32::INFINITY,
            button: 0,
            event_type: MouseEventType::Move,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::MouseEvent(p) = out.kind {
        assert!(p.x.is_nan(), "x 应为 NaN");
        assert!(p.x.is_sign_positive(), "NaN 应为正号");
        assert_eq!(f32::INFINITY, p.y);
    } else {
        panic!("期望 MouseEvent");
    }

    // 滚动事件使用负 Infinity
    let msg2 = IpcMessage {
        id: 2,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: f32::NEG_INFINITY,
            delta_y: f32::NAN,
            ..Default::default()
        }),
    };
    let out2 = roundtrip(msg2);
    if let IpcMessageKind::ScrollEvent(p) = out2.kind {
        assert_eq!(f32::NEG_INFINITY, p.delta_x);
        assert!(p.delta_y.is_nan(), "delta_y 应为 NaN");
    } else {
        panic!("期望 ScrollEvent");
    }
}

/// 测试 NavigateParams 中 referrer 为 Some("")（空字符串）的情况。
/// 验证 Some 包裹空字符串与 None 在序列化后不混淆。
#[test]
fn test_navigate_referrer_some_empty_string() {
    let msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: Some(String::new()),
            navigation_epoch: 0,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::Navigate(p) = out.kind {
        assert_eq!("https://example.com", p.url);
        // referrer 必须是 Some("")，不能变成 None
        assert!(p.referrer.is_some(), "referrer 不应为 None");
        assert_eq!("", p.referrer.unwrap(), "referrer 应为空字符串");
    } else {
        panic!("期望 Navigate");
    }
}

/// 测试 FetchRequest 中 request_id 为 0（最小值）且 HTTP 方法含特殊字符时的序列化正确性。
#[test]
fn test_fetch_request_zero_request_id_special_method() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 0,
            url: "https://example.com/api".into(),
            method: "PATCH".into(),
            headers: vec![
                ("X-Custom".into(), "a=b&c=d".into()),
                ("Content-Type".into(), "application/x-www-form-urlencoded".into()),
            ],
            body: Some(vec![]), // 空但非 None 的 body
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchRequest(p) = out.kind {
        assert_eq!(0, p.request_id, "request_id 应为 0");
        assert_eq!("PATCH", p.method);
        assert_eq!(2, p.headers.len());
        assert_eq!("a=b&c=d", p.headers[0].1);
        assert!(p.body.is_some(), "body 应为 Some");
        assert!(p.body.as_ref().unwrap().is_empty(), "body 内容应为空 Vec");
    } else {
        panic!("期望 FetchRequest");
    }
}

/// 测试 StorageOp 中 value 为 Some("")（空字符串）与 None 的区分。
/// 验证 Option<String> 在 None 和 Some("") 两种状态下序列化后不会混淆。
#[test]
fn test_storage_op_value_some_empty_vs_none() {
    // value = Some("")
    let msg_some_empty = IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Session,
            operation: StorageOperation::Set,
            key: "empty_key".into(),
            value: Some(String::new()),
            origin: "https://example.com".into(),
        }),
    };
    let out_some_empty = roundtrip(msg_some_empty);
    if let IpcMessageKind::StorageOp(p) = out_some_empty.kind {
        assert!(p.value.is_some(), "value 不应为 None");
        assert_eq!("", p.value.unwrap(), "value 应为空字符串");
    } else {
        panic!("期望 StorageOp");
    }

    // value = None
    let msg_none = IpcMessage {
        id: 2,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Session,
            operation: StorageOperation::Get,
            key: "empty_key".into(),
            value: None,
            origin: "https://example.com".into(),
        }),
    };
    let out_none = roundtrip(msg_none);
    if let IpcMessageKind::StorageOp(p) = out_none.kind {
        assert!(p.value.is_none(), "value 应为 None");
    } else {
        panic!("期望 StorageOp");
    }

    // 两条消息的序列化结果应不同（Some("") 和 None 编码不同）
    let bytes_some_empty = serialize(&IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Session,
            operation: StorageOperation::Set,
            key: "k".into(),
            value: Some(String::new()),
            origin: "o".into(),
        }),
    })
    .expect("serialize");
    let bytes_none = serialize(&IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Session,
            operation: StorageOperation::Set,
            key: "k".into(),
            value: None,
            origin: "o".into(),
        }),
    })
    .expect("serialize");
    assert_ne!(bytes_some_empty, bytes_none, "Some(\"\") 和 None 的序列化结果应不同");
}

// ══════════════════════════════════════════════════════════
//  新增边界条件测试
// ══════════════════════════════════════════════════════════

/// 测试 IpcChannel trait 的对象安全性（object safety）。
/// 验证 `Box<dyn IpcChannel>` 可以正常使用 send/recv/try_recv/close 方法，
/// 确保 trait 定义满足动态派发要求。
#[test]
fn test_ipc_channel_trait_object_dynamic_dispatch() {
    let mut ch: Box<dyn crate::IpcChannel> = Box::new(MockChannel::new());

    // 通过 trait object 发送消息
    ch.send(IpcMessage {
        id: 1,
        kind: IpcMessageKind::Heartbeat,
    })
    .expect("trait object send 应成功");
    ch.send(IpcMessage {
        id: 2,
        kind: IpcMessageKind::Ok,
    })
    .expect("trait object send 应成功");

    // 通过 trait object 接收消息（FIFO 顺序）
    let msg1 = ch.recv().expect("trait object recv 应成功");
    assert_eq!(1, msg1.id);
    assert!(matches!(msg1.kind, IpcMessageKind::Heartbeat));

    let msg2 = ch.recv().expect("trait object recv 应成功");
    assert_eq!(2, msg2.id);
    assert!(matches!(msg2.kind, IpcMessageKind::Ok));

    // 空通道 recv 应返回错误
    assert!(ch.recv().is_err());

    // 空通道 try_recv 应返回 Ok(None)
    assert!(ch.try_recv().expect("try_recv").is_none());

    // 关闭后操作应失败
    ch.close();
    assert!(
        ch.send(IpcMessage {
            id: 3,
            kind: IpcMessageKind::Ok
        })
        .is_err()
    );
    assert!(ch.recv().is_err());
    assert!(ch.try_recv().is_err());
}

/// 测试 ProtocolError 的 std::error::Error trait 实现。
/// 验证 thiserror derive 生成的 source() 方法能够正确返回内部错误源，
/// 以及 Display 格式包含关键信息。
#[test]
fn test_protocol_error_std_error_integration() {
    use std::error::Error;

    let err = ProtocolError::Serialization("frame overflow".into());
    // 验证 Display 输出
    let display = format!("{err}");
    assert!(display.contains("Serialization error"), "Display 应包含错误类型");
    assert!(display.contains("frame overflow"), "Display 应包含错误详情");

    // 验证 source() — thiserror 简单变体无内部 source，应返回 None
    assert!(err.source().is_none(), "ProtocolError 无嵌套 source，应为 None");

    // 验证 Debug 输出
    let debug = format!("{err:?}");
    assert!(debug.contains("Serialization"), "Debug 应包含变体名");

    // 验证每种变体的 Display 格式
    let channel_err = ProtocolError::Channel("pipe broken".into());
    assert!(format!("{channel_err}").contains("Channel error"));

    let process_err = ProtocolError::Process("spawn failed".into());
    assert!(format!("{process_err}").contains("Process error"));

    let deser_err = ProtocolError::Deserialization("unexpected EOF".into());
    assert!(format!("{deser_err}").contains("Deserialization error"));
}

/// 测试 KeyboardEvent 中 key 和 code 字段包含控制字符时的序列化/反序列化。
/// 控制字符（null、tab、换行、回车）在 IPC 传输中不应被丢失或损坏。
#[test]
fn test_keyboard_event_control_characters_in_key_and_code() {
    // key 包含 null、tab、换行、回车等控制字符
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
            key: "a\tb\nc\rd\u{0000}e".into(),
            code: "Key\u{0009}Code\u{000A}".into(),
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
            event_type: KeyboardEventType::Down,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::KeyboardEvent(p) = out.kind {
        assert_eq!("a\tb\nc\rd\u{0000}e", p.key, "含控制字符的 key 往返后应完全一致");
        assert_eq!("Key\u{0009}Code\u{000A}", p.code, "含控制字符的 code 往返后应完全一致");
    } else {
        panic!("期望 KeyboardEvent");
    }

    // key 和 code 均为纯控制字符的极端情况
    let msg2 = IpcMessage {
        id: 2,
        kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
            key: "\u{0000}\u{0001}\u{001F}".into(),
            code: "\t\n\r".into(),
            ctrl: true,
            shift: true,
            alt: true,
            meta: true,
            event_type: KeyboardEventType::Press,
        }),
    };
    let out2 = roundtrip(msg2);
    if let IpcMessageKind::KeyboardEvent(p) = out2.kind {
        assert_eq!("\u{0000}\u{0001}\u{001F}", p.key);
        assert_eq!("\t\n\r", p.code);
        assert!(p.ctrl && p.shift && p.alt && p.meta);
    } else {
        panic!("期望 KeyboardEvent");
    }
}

/// 测试反序列化错误恢复：无效数据反序列化失败后，
/// 紧接着对有效数据反序列化应不受影响。
/// 验证 deserialize 函数是无状态的，不会因前一次失败而污染后续调用。
#[test]
fn test_deserialization_error_recovery() {
    // 1. 先反序列化一条有效消息
    let valid_msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::Heartbeat,
    };
    let valid_bytes = serialize(&valid_msg).expect("序列化应成功");
    let out = deserialize(&valid_bytes).expect("有效数据反序列化应成功");
    assert_eq!(42, out.id);

    // 2. 用无效数据反序列化（应失败）
    let garbage = vec![0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8];
    assert!(deserialize(&garbage).is_err(), "无效数据应反序列化失败");

    // 3. 紧接着再次反序列化有效数据（应成功，不受前一次失败影响）
    let valid_msg2 = IpcMessage {
        id: 99,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://recovery.test".into(),
            referrer: Some("https://referrer.test".into()),
            navigation_epoch: 0,
        }),
    };
    let valid_bytes2 = serialize(&valid_msg2).expect("序列化应成功");
    let out2 = deserialize(&valid_bytes2).expect("错误恢复后有效数据应能正确反序列化");
    assert_eq!(99, out2.id);
    if let IpcMessageKind::Navigate(p) = out2.kind {
        assert_eq!("https://recovery.test", p.url);
        assert_eq!(Some("https://referrer.test".into()), p.referrer);
    } else {
        panic!("期望 Navigate");
    }

    // 4. 连续多次无效数据后再验证有效数据
    for _ in 0..5 {
        let _ = deserialize(&[0xDE, 0xAD, 0xBE, 0xEF]);
    }
    let valid_msg3 = IpcMessage {
        id: u64::MAX,
        kind: IpcMessageKind::Error("recovery test".into()),
    };
    let valid_bytes3 = serialize(&valid_msg3).expect("序列化应成功");
    let out3 = deserialize(&valid_bytes3).expect("多次失败后应仍能反序列化有效数据");
    assert_eq!(u64::MAX, out3.id);
    if let IpcMessageKind::Error(e) = out3.kind {
        assert_eq!("recovery test", e);
    } else {
        panic!("期望 Error");
    }
}

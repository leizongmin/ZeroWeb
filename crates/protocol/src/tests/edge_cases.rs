//! 边界条件测试：单字节 body、特殊浮点值、负零、组合边界值等。

use super::*;

/// 测试 FetchRequest 的 body 为恰好 1 字节的 Vec<u8> 时的序列化/反序列化。
/// 单字节 body 是 Option<Vec<u8>> 的最小非空边界，确保不会与 body=None 混淆。
#[test]
fn test_fetch_request_single_byte_body() {
    // body = Some(vec![0x00]) — 最小非空 body，内容为零字节
    let msg1 = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 1,
            url: "https://example.com".into(),
            method: "POST".into(),
            headers: vec![],
            body: Some(vec![0x00]),
        }),
    };
    let out1 = roundtrip(msg1);
    if let IpcMessageKind::FetchRequest(p) = out1.kind {
        assert_eq!(Some(vec![0x00]), p.body, "单字节零值 body 往返应一致");
    } else {
        panic!("期望 FetchRequest");
    }

    // body = Some(vec![0xFF]) — 单字节最大值
    let msg2 = IpcMessage {
        id: 2,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 2,
            url: "https://example.com".into(),
            method: "PUT".into(),
            headers: vec![],
            body: Some(vec![0xFF]),
        }),
    };
    let out2 = roundtrip(msg2);
    if let IpcMessageKind::FetchRequest(p) = out2.kind {
        assert_eq!(Some(vec![0xFF]), p.body, "单字节 0xFF body 往返应一致");
    } else {
        panic!("期望 FetchRequest");
    }

    // 对比：body = None 的序列化结果必须与 Some(vec![0x00]) 不同
    let bytes_single = serialize(&IpcMessage {
        id: 3,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 3,
            url: "https://example.com".into(),
            method: "POST".into(),
            headers: vec![],
            body: Some(vec![0x00]),
        }),
    })
    .expect("serialize");
    let bytes_none = serialize(&IpcMessage {
        id: 3,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 3,
            url: "https://example.com".into(),
            method: "POST".into(),
            headers: vec![],
            body: None,
        }),
    })
    .expect("serialize");
    assert_ne!(
        bytes_single, bytes_none,
        "Some(vec![0x00]) 和 None 的序列化结果必须不同"
    );

    // 对比：FetchResponse 单字节 body 也应正确往返
    let msg_resp = IpcMessage {
        id: 4,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 4,
            status_code: 200,
            headers: vec![],
            body: vec![0x42],
        }),
    };
    let out_resp = roundtrip(msg_resp);
    if let IpcMessageKind::FetchResponse(p) = out_resp.kind {
        assert_eq!(vec![0x42], p.body, "FetchResponse 单字节 body 往返应一致");
    } else {
        panic!("期望 FetchResponse");
    }
}

// ══════════════════════════════════════════════════════════
//  新增边界条件测试（第 2 批）
// ══════════════════════════════════════════════════════════

/// 测试 f32 负零（-0.0）在鼠标和滚动事件中的序列化/反序列化。
/// IEEE 754 中 +0.0 和 -0.0 的位模式不同，验证序列化能保留符号位。
#[test]
fn test_float_negative_zero_preserved() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::MouseEvent(MouseEventParams {
            x: -0.0,
            y: 0.0,
            button: 0,
            event_type: MouseEventType::Move,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::MouseEvent(p) = out.kind {
        // 验证符号位被保留：-0.0 应为负号
        assert!(p.x.is_sign_negative(), "x=-0.0 往返后应保持负号");
        assert!(p.y.is_sign_positive(), "y=+0.0 往返后应保持正号");
        // -0.0 == 0.0 在 IEEE 754 中为 true，但位模式不同
        assert_eq!(0.0, p.x);
    } else {
        panic!("期望 MouseEvent");
    }

    // 滚动事件中的负零
    let msg2 = IpcMessage {
        id: 2,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: -0.0,
            delta_y: -0.0,
            ..Default::default()
        }),
    };
    let out2 = roundtrip(msg2);
    if let IpcMessageKind::ScrollEvent(p) = out2.kind {
        assert!(p.delta_x.is_sign_negative(), "delta_x=-0.0 应保持负号");
        assert!(p.delta_y.is_sign_negative(), "delta_y=-0.0 应保持负号");
    } else {
        panic!("期望 ScrollEvent");
    }
}

/// 测试 FetchResponse 同时使用 request_id=u64::MAX 和 status_code=u16::MAX 的组合边界值。
/// 验证多个字段同时为最大值时序列化不会溢出或截断。
#[test]
fn test_fetch_response_combined_max_boundary() {
    let msg = IpcMessage {
        id: u64::MAX,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: u64::MAX,
            status_code: u16::MAX,
            headers: vec![
                ("X-Max-Id".into(), format!("{}", u64::MAX)),
                ("X-Status".into(), format!("{}", u16::MAX)),
            ],
            body: vec![0xFF; 16],
        }),
    };
    let out = roundtrip(msg);
    assert_eq!(u64::MAX, out.id, "消息 id 应为 u64::MAX");
    if let IpcMessageKind::FetchResponse(p) = out.kind {
        assert_eq!(u64::MAX, p.request_id, "request_id 应为 u64::MAX");
        assert_eq!(u16::MAX, p.status_code, "status_code 应为 u16::MAX");
        assert_eq!(2, p.headers.len());
        assert_eq!(format!("{}", u64::MAX), p.headers[0].1);
        assert_eq!(format!("{}", u16::MAX), p.headers[1].1);
        assert_eq!(vec![0xFF; 16], p.body);
    } else {
        panic!("期望 FetchResponse");
    }
}

/// 测试 FetchRequest 的 headers 中键和/或值为空字符串时的序列化/反序列化。
/// 空字符串键值对不应被序列化器丢弃或与无 header 混淆。
#[test]
fn test_headers_with_empty_key_or_value() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 1,
            url: "https://example.com".into(),
            method: "GET".into(),
            headers: vec![
                (String::new(), "value-with-empty-key".into()), // 空 key
                ("key-with-empty-value".into(), String::new()), // 空 value
                (String::new(), String::new()),                 // 两者都为空
                ("normal".into(), "header".into()),             // 正常键值对
            ],
            body: None,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchRequest(p) = out.kind {
        assert_eq!(4, p.headers.len(), "应保留所有 4 个 header");
        assert_eq!("", p.headers[0].0, "第一个 header 的 key 应为空字符串");
        assert_eq!("value-with-empty-key", p.headers[0].1);
        assert_eq!("key-with-empty-value", p.headers[1].0);
        assert_eq!("", p.headers[1].1, "第二个 header 的 value 应为空字符串");
        assert_eq!("", p.headers[2].0, "第三个 header 的 key 应为空字符串");
        assert_eq!("", p.headers[2].1, "第三个 header 的 value 应为空字符串");
        assert_eq!("normal", p.headers[3].0);
        assert_eq!("header", p.headers[3].1);
    } else {
        panic!("期望 FetchRequest");
    }

    // 验证含空键值对的 headers 序列化结果与空 headers 不同
    let bytes_with_headers = serialize(&IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 1,
            url: "https://example.com".into(),
            method: "GET".into(),
            headers: vec![(String::new(), String::new())],
            body: None,
        }),
    })
    .expect("serialize");
    let bytes_no_headers = serialize(&IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 1,
            url: "https://example.com".into(),
            method: "GET".into(),
            headers: vec![],
            body: None,
        }),
    })
    .expect("serialize");
    assert_ne!(
        bytes_with_headers, bytes_no_headers,
        "含一个空键值对 header 的序列化结果应与无 header 不同"
    );
}

/// 测试两条不同消息序列化后拼接字节，分别取前缀和后缀反序列化互不干扰。
/// 验证 deserialize 只消费精确的字节数，不会越界读取。
#[test]
fn test_serialized_bytes_no_cross_contamination() {
    let msg_a = IpcMessage {
        id: 111,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://a.com".into(),
            referrer: Some("https://ref.com".into()),
            navigation_epoch: 0,
        }),
    };
    let msg_b = IpcMessage {
        id: 222,
        kind: IpcMessageKind::TitleChanged("消息 B".into()),
    };

    let bytes_a = serialize(&msg_a).expect("serialize A");
    let bytes_b = serialize(&msg_b).expect("serialize B");
    let len_a = bytes_a.len();

    // 拼接两条消息的字节
    let mut concatenated = bytes_a.clone();
    concatenated.extend_from_slice(&bytes_b);

    // 从拼接缓冲区的前 len_a 字节反序列化应得到 msg_a
    let out_a: IpcMessage = deserialize(&concatenated[..len_a]).expect("反序列化 A 应成功");
    assert_eq!(111, out_a.id, "应为消息 A 的 id");
    if let IpcMessageKind::Navigate(p) = out_a.kind {
        assert_eq!("https://a.com", p.url);
    } else {
        panic!("期望 Navigate");
    }

    // 从拼接缓冲区的后段字节反序列化应得到 msg_b
    let out_b: IpcMessage = deserialize(&concatenated[len_a..]).expect("反序列化 B 应成功");
    assert_eq!(222, out_b.id, "应为消息 B 的 id");
    if let IpcMessageKind::TitleChanged(t) = out_b.kind {
        assert_eq!("消息 B", t);
    } else {
        panic!("期望 TitleChanged");
    }
}

/// 测试通过 Box<dyn IpcChannel> trait object 发送 50 条混合类型消息后按 FIFO 顺序接收。
/// 验证大量消息下 trait object 动态派发的正确性和顺序保证。
#[test]
fn test_ipc_channel_trait_object_stress_50_messages() {
    let mut ch: Box<dyn crate::IpcChannel> = Box::new(MockChannel::new());

    // 构造 50 条混合类型消息
    let messages: Vec<IpcMessage> = (0..50)
        .map(|i| {
            let kind = match i % 10 {
                0 => IpcMessageKind::Navigate(NavigateParams {
                    url: format!("https://example.com/page/{i}"),
                    referrer: if i % 3 == 0 {
                        Some("https://ref.com".into())
                    } else {
                        None
                    },
                    navigation_epoch: 0,
                }),
                1 => IpcMessageKind::TitleChanged(format!("标题 #{i}")),
                2 => IpcMessageKind::FetchRequest(FetchParams {
                    request_id: i as u64,
                    url: format!("https://api.com/{i}"),
                    method: "GET".into(),
                    headers: vec![(format!("X-Id-{i}"), format!("value-{i}"))],
                    body: if i % 2 == 0 { Some(vec![i as u8; 4]) } else { None },
                }),
                3 => IpcMessageKind::FetchResponse(FetchResponseParams {
                    request_id: i as u64,
                    status_code: (200 + (i % 5) as u16),
                    headers: vec![],
                    body: vec![i as u8],
                }),
                4 => IpcMessageKind::StorageOp(StorageOpParams {
                    storage_type: if i % 2 == 0 {
                        StorageType::Local
                    } else {
                        StorageType::Session
                    },
                    operation: StorageOperation::Set,
                    key: format!("key_{i}"),
                    value: Some(format!("val_{i}")),
                    origin: "https://example.com".into(),
                }),
                5 => IpcMessageKind::MouseEvent(MouseEventParams {
                    x: i as f32 * 10.0,
                    y: i as f32 * 20.0,
                    button: (i % 4) as u8,
                    event_type: MouseEventType::Click,
                }),
                6 => IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                    key: format!("Key{i}"),
                    code: format!("Code{i}"),
                    ctrl: i % 2 == 0,
                    shift: i % 3 == 0,
                    alt: false,
                    meta: false,
                    event_type: KeyboardEventType::Down,
                }),
                7 => IpcMessageKind::ScrollEvent(ScrollEventParams {
                    delta_x: i as f32,
                    delta_y: -(i as f32),
                    ..Default::default()
                }),
                8 => IpcMessageKind::Heartbeat,
                _ => IpcMessageKind::Error(format!("错误 #{i}")),
            };
            IpcMessage { id: i as u64, kind }
        })
        .collect();

    // 通过 trait object 发送全部消息
    for msg in &messages {
        ch.send(msg.clone())
            .unwrap_or_else(|_| panic!("发送消息 id={} 应成功", msg.id));
    }

    // 按 FIFO 顺序接收并逐一验证
    for (i, expected) in messages.iter().enumerate() {
        let received = ch.recv().unwrap_or_else(|_| panic!("接收第 {i} 条消息应成功"));
        assert_eq!(
            expected.id, received.id,
            "FIFO 顺序违反：索引 {i}，期望 id={}，实际 id={}",
            expected.id, received.id
        );

        // 验证消息类型匹配
        match (&expected.kind, &received.kind) {
            (IpcMessageKind::Navigate(a), IpcMessageKind::Navigate(b)) => {
                assert_eq!(a.url, b.url, "索引 {i}: Navigate url 不匹配");
            }
            (IpcMessageKind::TitleChanged(a), IpcMessageKind::TitleChanged(b)) => {
                assert_eq!(a, b, "索引 {i}: TitleChanged 不匹配");
            }
            (IpcMessageKind::FetchRequest(a), IpcMessageKind::FetchRequest(b)) => {
                assert_eq!(a.request_id, b.request_id, "索引 {i}: FetchRequest request_id 不匹配");
            }
            (IpcMessageKind::FetchResponse(a), IpcMessageKind::FetchResponse(b)) => {
                assert_eq!(
                    a.status_code, b.status_code,
                    "索引 {i}: FetchResponse status_code 不匹配"
                );
            }
            (IpcMessageKind::StorageOp(a), IpcMessageKind::StorageOp(b)) => {
                assert_eq!(a.key, b.key, "索引 {i}: StorageOp key 不匹配");
            }
            (IpcMessageKind::MouseEvent(a), IpcMessageKind::MouseEvent(b)) => {
                assert_eq!(a.x, b.x, "索引 {i}: MouseEvent x 不匹配");
            }
            (IpcMessageKind::KeyboardEvent(a), IpcMessageKind::KeyboardEvent(b)) => {
                assert_eq!(a.key, b.key, "索引 {i}: KeyboardEvent key 不匹配");
            }
            (IpcMessageKind::ScrollEvent(a), IpcMessageKind::ScrollEvent(b)) => {
                assert_eq!(a.delta_x, b.delta_x, "索引 {i}: ScrollEvent delta_x 不匹配");
            }
            (IpcMessageKind::Heartbeat, IpcMessageKind::Heartbeat) => {}
            (IpcMessageKind::Error(a), IpcMessageKind::Error(b)) => {
                assert_eq!(a, b, "索引 {i}: Error 不匹配");
            }
            _ => panic!("索引 {i}: 消息类型不匹配"),
        }
    }

    // 所有消息已消费，再次 recv 应返回错误
    assert!(ch.recv().is_err(), "消费全部消息后 recv 应返回错误");

    // try_recv 应返回 Ok(None)
    assert!(ch.try_recv().expect("try_recv").is_none());
}

// ══════════════════════════════════════════════════════════
//  新增边界条件测试（第 3 批）
// ══════════════════════════════════════════════════════════

/// 测试 FetchRequest 中 headers 包含重复键名时的序列化/反序列化。
/// HTTP 协议允许多个同名 header（如 Set-Cookie、Accept），
/// 验证 IPC 序列化不会去重或合并同名键。
#[test]
fn test_headers_duplicate_keys_preserved() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 1,
            url: "https://example.com".into(),
            method: "GET".into(),
            headers: vec![
                ("Accept".into(), "text/html".into()),
                ("Accept".into(), "application/json".into()),
                ("Accept".into(), "*/*".into()),
                ("Set-Cookie".into(), "a=1".into()),
                ("Set-Cookie".into(), "b=2; Path=/".into()),
            ],
            body: None,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchRequest(p) = out.kind {
        assert_eq!(5, p.headers.len(), "重复键名的 header 不应被去重");
        // 验证所有 Accept header 都独立保留
        let accept_values: Vec<&str> = p
            .headers
            .iter()
            .filter(|(k, _)| k == "Accept")
            .map(|(_, v)| v.as_str())
            .collect();
        assert_eq!(
            vec!["text/html", "application/json", "*/*"],
            accept_values,
            "重复的 Accept 键应各自独立保留"
        );
        // 验证 Set-Cookie header 也独立保留
        let cookie_values: Vec<&str> = p
            .headers
            .iter()
            .filter(|(k, _)| k == "Set-Cookie")
            .map(|(_, v)| v.as_str())
            .collect();
        assert_eq!(vec!["a=1", "b=2; Path=/"], cookie_values);
    } else {
        panic!("期望 FetchRequest");
    }
}

/// 测试 IpcMessage 克隆后两条消息独立序列化产生完全相同的字节。
/// 验证 Clone derive 生成的克隆体在语义上与原始消息完全一致，
/// 且修改克隆体不影响原始消息的序列化结果。
#[test]
fn test_message_clone_independence() {
    let original = IpcMessage {
        id: 42,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: Some("https://ref.com".into()),
            navigation_epoch: 0,
        }),
    };

    // 克隆后两条消息序列化应产生完全相同的字节
    let cloned = original.clone();
    let bytes_original = serialize(&original).expect("序列化原始消息应成功");
    let bytes_cloned = serialize(&cloned).expect("序列化克隆消息应成功");
    assert_eq!(bytes_original, bytes_cloned, "克隆消息的序列化结果应与原始消息完全一致");

    // 修改克隆体后，原始消息的序列化结果应保持不变
    let mut modified = original.clone();
    modified.id = 99;
    let bytes_original_after = serialize(&original).expect("序列化原始消息应成功");
    assert_eq!(
        bytes_original, bytes_original_after,
        "修改克隆体后原始消息的序列化结果不应改变"
    );
    // 修改后的消息序列化结果应与原始不同
    let bytes_modified = serialize(&modified).expect("序列化修改后消息应成功");
    assert_ne!(bytes_original, bytes_modified, "修改后的消息序列化结果应与原始不同");
}

/// 测试 LoadFailed 和 CrashNotification 中包含多行错误信息的序列化/反序列化。
/// 实际运行时错误通常包含堆栈跟踪等多行文本，验证换行符在 IPC 传输中完整保留。
#[test]
fn test_multiline_error_messages_roundtrip() {
    let multiline_error = "line 1: connection refused\nline 2: retrying...\nline 3: timeout\nline 4: giving up";

    // LoadFailed 含多行错误
    let msg1 = IpcMessage {
        id: 1,
        kind: IpcMessageKind::LoadFailed(multiline_error.into()),
    };
    let out1 = roundtrip(msg1);
    if let IpcMessageKind::LoadFailed(e) = out1.kind {
        assert_eq!(multiline_error, e, "LoadFailed 多行错误信息往返应完全一致");
        assert_eq!(4, e.lines().count(), "多行错误应保持 4 行");
    } else {
        panic!("期望 LoadFailed");
    }

    // CrashNotification 含多行堆栈跟踪
    let stack_trace = "Segmentation fault\n  at main.rs:42\n  at lib.rs:108\n  caused by: null pointer dereference";
    let msg2 = IpcMessage {
        id: 2,
        kind: IpcMessageKind::CrashNotification(stack_trace.into()),
    };
    let out2 = roundtrip(msg2);
    if let IpcMessageKind::CrashNotification(r) = out2.kind {
        assert_eq!(stack_trace, r, "CrashNotification 多行信息往返应完全一致");
        assert_eq!(4, r.lines().count());
    } else {
        panic!("期望 CrashNotification");
    }

    // Error 响应含多行错误
    let msg3 = IpcMessage {
        id: 3,
        kind: IpcMessageKind::Error("error:\n  detail 1\n  detail 2\n  detail 3".into()),
    };
    let out3 = roundtrip(msg3);
    if let IpcMessageKind::Error(e) = out3.kind {
        assert_eq!(4, e.lines().count(), "Error 多行信息行数应保持一致");
    } else {
        panic!("期望 Error");
    }
}

/// 测试 MouseEvent 和 KeyboardEvent 中所有事件类型（Down/Up/Move/Click/DblClick 和 Down/Up/Press）
/// 与零值/极端值坐标和修饰键的组合，确保每种事件类型在边界条件下序列化不丢失字段。
#[test]
fn test_input_events_all_types_with_boundary_values() {
    // 鼠标事件：每种类型均使用负坐标和最大 button 值
    let mouse_types = vec![
        MouseEventType::Down,
        MouseEventType::Up,
        MouseEventType::Move,
        MouseEventType::Click,
        MouseEventType::DblClick,
    ];
    for (i, etype) in mouse_types.into_iter().enumerate() {
        let msg = IpcMessage {
            id: i as u64,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: -999.99,
                y: -0.001,
                button: u8::MAX,
                event_type: etype.clone(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::MouseEvent(p) = out.kind {
            assert_eq!(-999.99, p.x, "MouseEvent x 往返失败，类型 {:?}", etype);
            assert_eq!(-0.001, p.y, "MouseEvent y 往返失败，类型 {:?}", etype);
            assert_eq!(u8::MAX, p.button);
            assert_eq!(etype, p.event_type, "MouseEvent event_type 往返失败");
        } else {
            panic!("期望 MouseEvent，索引 {i}");
        }
    }

    // 键盘事件：每种类型均使用空 key 和所有修饰键按下
    let kb_types = vec![KeyboardEventType::Down, KeyboardEventType::Up, KeyboardEventType::Press];
    for (i, etype) in kb_types.into_iter().enumerate() {
        let msg = IpcMessage {
            id: (i + 10) as u64,
            kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: String::new(),
                code: "Space".into(),
                ctrl: true,
                shift: true,
                alt: true,
                meta: true,
                event_type: etype.clone(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::KeyboardEvent(p) = out.kind {
            assert!(p.key.is_empty(), "key 应为空字符串");
            assert_eq!("Space", p.code);
            assert!(p.ctrl && p.shift && p.alt && p.meta, "所有修饰键应为 true");
            assert_eq!(etype, p.event_type, "KeyboardEvent event_type 往返失败");
        } else {
            panic!("期望 KeyboardEvent，索引 {i}");
        }
    }
}

/// 测试 NavigateParams 中 referrer 包含 URL 编码特殊字符时的序列化/反序列化。
/// URL 中的百分号编码、查询参数、锚点、认证信息等特殊字符在 IPC 传输中应完整保留。
#[test]
fn test_navigate_with_url_encoded_referrer() {
    let referrer = "https://example.com/path?q=%E4%B8%AD%E6%96%87&lang=zh#frag%20ment";
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://target.com/page".into(),
            referrer: Some(referrer.into()),
            navigation_epoch: 0,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::Navigate(p) = out.kind {
        assert_eq!("https://target.com/page", p.url);
        assert_eq!(
            Some(referrer.to_string()),
            p.referrer,
            "含 URL 编码的 referrer 往返应完全一致"
        );
    } else {
        panic!("期望 Navigate");
    }
}

/// 测试 KeyboardEvent 中 key 和 code 为超长字符串时的序列化/反序列化。
/// 验证大体积字符串字段不会在 bincode 编码中被截断。
#[test]
fn test_keyboard_event_long_key_and_code() {
    let long_key = "X".repeat(10_000);
    let long_code = "Y".repeat(10_000);
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
            key: long_key.clone(),
            code: long_code.clone(),
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
            event_type: KeyboardEventType::Press,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::KeyboardEvent(p) = out.kind {
        assert_eq!(10_000, p.key.len(), "key 长度应为 10000");
        assert_eq!(10_000, p.code.len(), "code 长度应为 10000");
        assert_eq!(long_key, p.key, "超长 key 往返应完全一致");
        assert_eq!(long_code, p.code, "超长 code 往返应完全一致");
    } else {
        panic!("期望 KeyboardEvent");
    }
}

/// 测试 StorageOp 中所有 StorageType 与 StorageOperation 的笛卡尔积组合。
/// 确保每种存储类型与操作的搭配都能正确序列化/反序列化，不会因枚举组合产生编码冲突。
#[test]
fn test_storage_op_all_type_operation_combinations() {
    let storage_types = vec![StorageType::Local, StorageType::Session];
    let operations = vec![
        StorageOperation::Get,
        StorageOperation::Set,
        StorageOperation::Remove,
        StorageOperation::Clear,
        StorageOperation::Length,
        StorageOperation::Key,
    ];

    let mut id = 0u64;
    for st in &storage_types {
        for op in &operations {
            id += 1;
            let msg = IpcMessage {
                id,
                kind: IpcMessageKind::StorageOp(StorageOpParams {
                    storage_type: st.clone(),
                    operation: op.clone(),
                    key: format!("key_{id}"),
                    value: if matches!(op, StorageOperation::Set) {
                        Some(format!("val_{id}"))
                    } else {
                        None
                    },
                    origin: "https://example.com".into(),
                }),
            };
            let out = roundtrip(msg);
            if let IpcMessageKind::StorageOp(p) = out.kind {
                assert_eq!(*st, p.storage_type, "StorageType 不匹配，id={id}");
                assert_eq!(*op, p.operation, "StorageOperation 不匹配，id={id}");
                assert_eq!(format!("key_{id}"), p.key, "key 不匹配，id={id}");
            } else {
                panic!("期望 StorageOp，id={id}");
            }
        }
    }
    assert_eq!(12, id, "应测试 2x6=12 种组合");
}

/// 测试 FetchResponse 的 body 仅包含 0xFF 字节（所有位置 1）时的序列化/反序列化。
/// 验证二进制传输不会将 0xFF 误判为填充或特殊标记而截断。
#[test]
fn test_fetch_response_body_all_0xff_bytes() {
    let body: Vec<u8> = vec![0xFF; 256];
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 1,
            status_code: 200,
            headers: vec![("Content-Type".into(), "application/octet-stream".into())],
            body: body.clone(),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchResponse(p) = out.kind {
        assert_eq!(256, p.body.len(), "body 长度应为 256");
        assert!(p.body.iter().all(|&b| b == 0xFF), "所有字节应为 0xFF");
        assert_eq!(body, p.body, "全 0xFF body 往返应完全一致");
    } else {
        panic!("期望 FetchResponse");
    }
}

/// 测试通过 MockChannel 突发发送 20 条消息后一次性全部接收。
/// 验证通道在突发写入场景下不丢消息、顺序不乱。
#[test]
fn test_channel_burst_send_then_receive_all() {
    let mut ch = MockChannel::new();

    // 突发发送 20 条混合类型消息
    for i in 0..20u64 {
        let kind = match i % 4 {
            0 => IpcMessageKind::Navigate(NavigateParams {
                url: format!("https://example.com/{i}"),
                referrer: None,
                navigation_epoch: 0,
            }),
            1 => IpcMessageKind::TitleChanged(format!("标题 {i}")),
            2 => IpcMessageKind::Heartbeat,
            _ => IpcMessageKind::Error(format!("错误 {i}")),
        };
        ch.send(IpcMessage { id: i, kind }).expect("发送应成功");
    }

    // 一次性全部接收，验证 FIFO 顺序
    for i in 0..20u64 {
        let msg = ch.recv().unwrap_or_else(|_| panic!("接收第 {i} 条应成功"));
        assert_eq!(i, msg.id, "FIFO 顺序违反：期望 id={i}，实际 id={}", msg.id);
    }

    // 通道应为空
    assert!(ch.recv().is_err(), "所有消息已消费，recv 应返回错误");
}

/// 测试相同 id 但不同 IpcMessageKind 的消息序列化结果必须不同。
/// 验证 bincode 编码中枚举变体标签确实影响了输出字节，
/// 防止不同变体因编码缺陷产生相同的二进制表示。
#[test]
fn test_different_kinds_produce_different_bytes() {
    let id = 42u64;
    let msg_ok = IpcMessage {
        id,
        kind: IpcMessageKind::Ok,
    };
    let msg_heartbeat = IpcMessage {
        id,
        kind: IpcMessageKind::Heartbeat,
    };
    let msg_load_complete = IpcMessage {
        id,
        kind: IpcMessageKind::LoadComplete,
    };
    let msg_go_back = IpcMessage {
        id,
        kind: IpcMessageKind::GoBack,
    };
    let msg_go_forward = IpcMessage {
        id,
        kind: IpcMessageKind::GoForward,
    };
    let msg_reload = IpcMessage {
        id,
        kind: IpcMessageKind::Reload,
    };
    let msg_stop = IpcMessage {
        id,
        kind: IpcMessageKind::StopLoading,
    };

    let bytes = [
        serialize(&msg_ok).expect("s"),
        serialize(&msg_heartbeat).expect("s"),
        serialize(&msg_load_complete).expect("s"),
        serialize(&msg_go_back).expect("s"),
        serialize(&msg_go_forward).expect("s"),
        serialize(&msg_reload).expect("s"),
        serialize(&msg_stop).expect("s"),
    ];

    // 任意两条不同变体的序列化结果应不同
    for i in 0..bytes.len() {
        for j in (i + 1)..bytes.len() {
            assert_ne!(
                bytes[i], bytes[j],
                "不同 IpcMessageKind 变体（索引 {i} vs {j}）的序列化结果必须不同"
            );
        }
    }
}

// ══════════════════════════════════════════════════════════
//  新增边界条件测试（第 4 批）
// ══════════════════════════════════════════════════════════

/// 测试 MockChannel 发送 3 条消息后接收 2 条，再发送 2 条，验证 FIFO 顺序跨越
/// send-recv-send 周期保持不变：先收到的应是前两轮发送的，最后收到的是第二轮发送的。
#[test]
fn test_mock_channel_send_recv_send_fifo_order() {
    let mut ch = MockChannel::new();

    // 第一轮：发送 3 条消息 (id=1,2,3)
    for i in 1u64..=3 {
        ch.send(IpcMessage {
            id: i,
            kind: IpcMessageKind::Heartbeat,
        })
        .expect("send");
    }

    // 接收 2 条 (id=1,2)
    assert_eq!(1, ch.recv().expect("recv").id);
    assert_eq!(2, ch.recv().expect("recv").id);

    // 第二轮：发送 2 条消息 (id=4,5)
    for i in 4u64..=5 {
        ch.send(IpcMessage {
            id: i,
            kind: IpcMessageKind::Ok,
        })
        .expect("send");
    }

    // 按 FIFO 顺序接收剩余消息：应先收到第一轮的 id=3，再收到第二轮的 id=4,5
    assert_eq!(3, ch.recv().expect("recv").id, "FIFO：第一轮剩余消息应先于第二轮");
    assert_eq!(4, ch.recv().expect("recv").id, "FIFO：第二轮第一条消息");
    assert_eq!(5, ch.recv().expect("recv").id, "FIFO：第二轮第二条消息");

    // 通道应为空
    assert!(ch.recv().is_err(), "所有消息已消费，recv 应返回错误");
}

/// 测试 StorageOp 使用 StorageType::Session 序列化/反序列化后 storage_type 正确保留。
/// 验证 Session 枚举值在 bincode 编码/解码过程中不会被错误映射为 Local。
#[test]
fn test_storage_op_session_type_preserved() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Session,
            operation: StorageOperation::Set,
            key: "session_key".into(),
            value: Some("session_value".into()),
            origin: "https://example.com".into(),
        }),
    };
    let bytes = serialize(&msg).expect("序列化应成功");
    let out: IpcMessage = deserialize(&bytes).expect("反序列化应成功");
    if let IpcMessageKind::StorageOp(p) = out.kind {
        assert_eq!(
            StorageType::Session,
            p.storage_type,
            "StorageType::Session 经序列化往返后应保持为 Session"
        );
        assert_eq!("session_key", p.key);
        assert_eq!(Some("session_value".into()), p.value);
    } else {
        panic!("期望 StorageOp");
    }

    // 验证 Session 和 Local 的序列化结果不同
    let msg_local = IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Set,
            key: "session_key".into(),
            value: Some("session_value".into()),
            origin: "https://example.com".into(),
        }),
    };
    let bytes_local = serialize(&msg_local).expect("序列化 Local 应成功");
    assert_ne!(
        bytes, bytes_local,
        "StorageType::Session 和 StorageType::Local 的序列化结果应不同"
    );
}

/// 测试 IpcMessage 中 request_id = 0 且 id = 0 时，零值在序列化/反序列化后完整保留。
/// 零值是 u64 的最小值，验证不会与 Option::None 或缺省值混淆。
#[test]
fn test_ipc_message_zero_id_and_request_id() {
    let msg = IpcMessage {
        id: 0,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 0,
            url: "https://example.com".into(),
            method: "GET".into(),
            headers: vec![],
            body: None,
        }),
    };
    let bytes = serialize(&msg).expect("序列化应成功");
    let out: IpcMessage = deserialize(&bytes).expect("反序列化应成功");
    assert_eq!(0, out.id, "消息 id=0 经往返后应保持为 0");
    if let IpcMessageKind::FetchRequest(p) = out.kind {
        assert_eq!(0, p.request_id, "request_id=0 经往返后应保持为 0");
    } else {
        panic!("期望 FetchRequest");
    }

    // 验证 id=0 和 id=1 的序列化结果不同，确认零值不会被误解为缺失
    let msg_id1 = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 0,
            url: "https://example.com".into(),
            method: "GET".into(),
            headers: vec![],
            body: None,
        }),
    };
    let bytes_id1 = serialize(&msg_id1).expect("序列化应成功");
    assert_ne!(
        bytes, bytes_id1,
        "id=0 和 id=1 的序列化结果应不同，零值不应被误解为缺失"
    );
}

/// 测试 FetchParams 的 body 包含 256 字节（0x00 到 0xFF）时的序列化/反序列化。
/// 验证所有 256 种可能的字节值在 IPC 传输中逐字节完整保留，无一丢失或损坏。
#[test]
fn test_fetch_params_body_full_byte_range() {
    let body: Vec<u8> = (0u8..=255).collect();
    assert_eq!(256, body.len(), "body 应恰好包含 256 字节");

    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 42,
            url: "https://example.com/upload".into(),
            method: "POST".into(),
            headers: vec![("Content-Type".into(), "application/octet-stream".into())],
            body: Some(body.clone()),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchRequest(p) = out.kind {
        assert_eq!(Some(body), p.body, "256 字节 body（0x00-0xFF）逐字节往返应完全一致");
    } else {
        panic!("期望 FetchRequest");
    }
}

/// 测试 ProtocolError 的 Display 格式化对每个变体都产生非空字符串。
/// 确保所有错误类型在日志输出或用户展示时不会输出空白内容。
#[test]
fn test_protocol_error_display_non_empty_for_all_variants() {
    let variants: Vec<ProtocolError> = vec![
        ProtocolError::Serialization("序列化失败".into()),
        ProtocolError::Deserialization("反序列化失败".into()),
        ProtocolError::Channel("通道已关闭".into()),
        ProtocolError::Process("进程崩溃".into()),
    ];

    for (i, err) in variants.iter().enumerate() {
        let display = format!("{err}");
        assert!(
            !display.is_empty(),
            "ProtocolError 变体索引 {i} 的 Display 输出不应为空"
        );
    }
}

// ── 新增边界测试 ──

/// 测试 NavigateParams referrer 为 None 时序列化往返正确。
#[test]
fn test_navigate_params_none_referrer_roundtrip() {
    let msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: None,
            navigation_epoch: 0,
        }),
    };
    let rt = roundtrip(msg);
    match rt.kind {
        IpcMessageKind::Navigate(p) => {
            assert_eq!(p.url, "https://example.com");
            assert!(p.referrer.is_none(), "referrer 应为 None");
        }
        _ => panic!("期望 Navigate"),
    }
}

/// 测试 KeyboardEventParams 所有修饰键同时为 true 的往返。
#[test]
fn test_keyboard_event_all_modifiers_true_roundtrip() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
            key: "a".into(),
            code: "KeyA".into(),
            ctrl: true,
            shift: true,
            alt: true,
            meta: true,
            event_type: KeyboardEventType::Press,
        }),
    };
    let rt = roundtrip(msg);
    match rt.kind {
        IpcMessageKind::KeyboardEvent(p) => {
            assert!(p.ctrl && p.shift && p.alt && p.meta);
            assert_eq!(p.event_type, KeyboardEventType::Press);
        }
        _ => panic!("期望 KeyboardEvent"),
    }
}

/// 测试 MouseEventType 所有变体序列化后不混淆。
#[test]
fn test_mouse_event_type_all_variants_distinct() {
    let variants = [
        MouseEventType::Down,
        MouseEventType::Up,
        MouseEventType::Move,
        MouseEventType::Click,
        MouseEventType::DblClick,
    ];
    let mut bytes_set = std::collections::HashSet::new();
    for v in &variants {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 0.0,
                y: 0.0,
                button: 0,
                event_type: v.clone(),
            }),
        };
        let serialized = serialize(&msg).expect("serialize");
        bytes_set.insert(serialized);
    }
    assert_eq!(
        bytes_set.len(),
        variants.len(),
        "每个 MouseEventType 变体应产生不同的字节"
    );
}

/// 测试 ScrollEventParams 负 delta 值序列化往返。
#[test]
fn test_scroll_event_negative_deltas_roundtrip() {
    let msg = IpcMessage {
        id: 7,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: -50.5,
            delta_y: -100.0,
            ..Default::default()
        }),
    };
    let rt = roundtrip(msg);
    match rt.kind {
        IpcMessageKind::ScrollEvent(p) => {
            assert!((p.delta_x - (-50.5)).abs() < 0.001);
            assert!((p.delta_y - (-100.0)).abs() < 0.001);
        }
        _ => panic!("期望 ScrollEvent"),
    }
}

/// 测试 GoBack/GoForward 消息序列化后字节不同。
#[test]
fn test_go_back_forward_different_bytes() {
    let back = IpcMessage {
        id: 1,
        kind: IpcMessageKind::GoBack,
    };
    let forward = IpcMessage {
        id: 1,
        kind: IpcMessageKind::GoForward,
    };
    let b1 = serialize(&back).expect("serialize back");
    let b2 = serialize(&forward).expect("serialize forward");
    assert_ne!(b1, b2, "GoBack 和 GoForward 应产生不同的字节");
}

// ── message.rs 类型序列化 round-trip 测试 ──

/// 测试 NavigateParams 带 referrer 的 round-trip。
#[test]
fn test_navigate_params_with_referrer() {
    let params = NavigateParams {
        url: "https://example.com/page".to_string(),
        referrer: Some("https://google.com".to_string()),
        navigation_epoch: 0,
    };
    let bytes = bincode::serialize(&params).expect("serialize");
    let rt: NavigateParams = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(rt.url, "https://example.com/page");
    assert_eq!(rt.referrer, Some("https://google.com".to_string()));
}

/// 测试 NavigateParams 无 referrer 的 round-trip。
#[test]
fn test_navigate_params_no_referrer() {
    let params = NavigateParams {
        url: "https://example.com".to_string(),
        referrer: None,
        navigation_epoch: 0,
    };
    let bytes = bincode::serialize(&params).expect("serialize");
    let rt: NavigateParams = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(rt.url, "https://example.com");
    assert_eq!(rt.referrer, None);
}

/// 测试 FetchParams 带 body 的 round-trip。
#[test]
fn test_message_fetch_params_with_body() {
    let params = FetchParams {
        request_id: 42,
        url: "https://api.example.com/data".to_string(),
        method: "POST".to_string(),
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Authorization".to_string(), "Bearer token123".to_string()),
        ],
        body: Some(b"{\"key\": \"value\"}".to_vec()),
    };
    let bytes = bincode::serialize(&params).expect("serialize");
    let rt: FetchParams = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(rt.request_id, 42);
    assert_eq!(rt.method, "POST");
    assert_eq!(rt.headers.len(), 2);
    assert_eq!(rt.body.as_ref().unwrap().len(), 16);
}

/// 测试 FetchParams 无 body 的 round-trip。
#[test]
fn test_fetch_params_no_body() {
    let params = FetchParams {
        request_id: 1,
        url: "https://example.com".to_string(),
        method: "GET".to_string(),
        headers: vec![],
        body: None,
    };
    let bytes = bincode::serialize(&params).expect("serialize");
    let rt: FetchParams = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(rt.method, "GET");
    assert!(rt.body.is_none());
}

/// 测试 FetchResponseParams round-trip。
#[test]
fn test_fetch_response_params() {
    let params = FetchResponseParams {
        request_id: 42,
        status_code: 200,
        headers: vec![("content-type".to_string(), "text/html".to_string())],
        body: b"<html></html>".to_vec(),
    };
    let bytes = bincode::serialize(&params).expect("serialize");
    let rt: FetchResponseParams = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(rt.status_code, 200);
    assert_eq!(rt.body, b"<html></html>".to_vec());
}

/// 测试 FetchResponseParams 非 200 状态码。
#[test]
fn test_fetch_response_error_status() {
    let params = FetchResponseParams {
        request_id: 5,
        status_code: 404,
        headers: vec![],
        body: b"Not Found".to_vec(),
    };
    let bytes = bincode::serialize(&params).expect("serialize");
    let rt: FetchResponseParams = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(rt.status_code, 404);
}

/// 测试 StorageOpParams 全类型 round-trip。
#[test]
fn test_storage_op_params_get() {
    let params = StorageOpParams {
        storage_type: StorageType::Local,
        operation: StorageOperation::Get,
        key: "user_token".to_string(),
        value: None,
        origin: "https://example.com".to_string(),
    };
    let bytes = bincode::serialize(&params).expect("serialize");
    let rt: StorageOpParams = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(rt.storage_type, StorageType::Local);
    assert_eq!(rt.operation, StorageOperation::Get);
    assert_eq!(rt.key, "user_token");
    assert!(rt.value.is_none());
}

/// 测试 StorageOpParams Set 操作 round-trip。
#[test]
fn test_storage_op_params_set() {
    let params = StorageOpParams {
        storage_type: StorageType::Session,
        operation: StorageOperation::Set,
        key: "session_id".to_string(),
        value: Some("abc123".to_string()),
        origin: "https://app.example.com".to_string(),
    };
    let bytes = bincode::serialize(&params).expect("serialize");
    let rt: StorageOpParams = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(rt.storage_type, StorageType::Session);
    assert_eq!(rt.operation, StorageOperation::Set);
    assert_eq!(rt.value, Some("abc123".to_string()));
}

/// 测试 StorageOperation 所有变体可序列化。
#[test]
fn test_storage_operation_all_variants() {
    let ops = [
        StorageOperation::Get,
        StorageOperation::Set,
        StorageOperation::Remove,
        StorageOperation::Clear,
        StorageOperation::Length,
        StorageOperation::Key,
    ];
    for op in &ops {
        let bytes = bincode::serialize(op).expect("serialize");
        let rt: StorageOperation = bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(&rt, op);
    }
}

/// 测试 StorageType 两种变体。
#[test]
fn test_storage_type_variants() {
    let local = bincode::serialize(&StorageType::Local).expect("s");
    let session = bincode::serialize(&StorageType::Session).expect("s");
    assert_ne!(local, session);
    let rt_l: StorageType = bincode::deserialize(&local).expect("d");
    let rt_s: StorageType = bincode::deserialize(&session).expect("d");
    assert_eq!(rt_l, StorageType::Local);
    assert_eq!(rt_s, StorageType::Session);
}

/// 测试 MouseEventParams round-trip。
#[test]
fn test_mouse_event_params_roundtrip() {
    let params = MouseEventParams {
        x: 150.5,
        y: 200.75,
        button: 1,
        event_type: MouseEventType::DblClick,
    };
    let bytes = bincode::serialize(&params).expect("serialize");
    let rt: MouseEventParams = bincode::deserialize(&bytes).expect("deserialize");
    assert!((rt.x - 150.5).abs() < 0.01);
    assert!((rt.y - 200.75).abs() < 0.01);
    assert_eq!(rt.button, 1);
    assert_eq!(rt.event_type, MouseEventType::DblClick);
}

/// 测试 MouseEventType 所有变体。
#[test]
fn test_mouse_event_type_all_variants() {
    let types = [
        MouseEventType::Down,
        MouseEventType::Up,
        MouseEventType::Move,
        MouseEventType::Click,
        MouseEventType::DblClick,
    ];
    for t in &types {
        let bytes = bincode::serialize(t).expect("serialize");
        let rt: MouseEventType = bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(&rt, t);
    }
}

/// 测试 KeyboardEventParams 全修饰键 round-trip。
#[test]
fn test_keyboard_event_all_modifiers() {
    let params = KeyboardEventParams {
        key: "c".to_string(),
        code: "KeyC".to_string(),
        ctrl: true,
        shift: true,
        alt: true,
        meta: true,
        event_type: KeyboardEventType::Down,
    };
    let bytes = bincode::serialize(&params).expect("serialize");
    let rt: KeyboardEventParams = bincode::deserialize(&bytes).expect("deserialize");
    assert!(rt.ctrl && rt.shift && rt.alt && rt.meta);
    assert_eq!(rt.key, "c");
}

/// 测试 KeyboardEventParams 无修饰键。
#[test]
fn test_keyboard_event_no_modifiers() {
    let params = KeyboardEventParams {
        key: "a".to_string(),
        code: "KeyA".to_string(),
        ctrl: false,
        shift: false,
        alt: false,
        meta: false,
        event_type: KeyboardEventType::Press,
    };
    let bytes = bincode::serialize(&params).expect("serialize");
    let rt: KeyboardEventParams = bincode::deserialize(&bytes).expect("deserialize");
    assert!(!rt.ctrl && !rt.shift && !rt.alt && !rt.meta);
}

/// 测试 KeyboardEventType 所有变体。
#[test]
fn test_keyboard_event_type_all_variants() {
    let types = [KeyboardEventType::Down, KeyboardEventType::Up, KeyboardEventType::Press];
    for t in &types {
        let bytes = bincode::serialize(t).expect("serialize");
        let rt: KeyboardEventType = bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(&rt, t);
    }
}

/// 测试 ScrollEventParams 负值 round-trip。
#[test]
fn test_scroll_event_negative_values() {
    let params = ScrollEventParams {
        delta_x: -3.1416,
        delta_y: -159.265,
        ..Default::default()
    };
    let bytes = bincode::serialize(&params).expect("serialize");
    let rt: ScrollEventParams = bincode::deserialize(&bytes).expect("deserialize");
    assert!((rt.delta_x - (-3.1416)).abs() < 0.001);
    assert!((rt.delta_y - (-159.265)).abs() < 0.001);
}

/// 测试 ScrollEventParams 零值 round-trip。
#[test]
fn test_scroll_event_zero_values() {
    let params = ScrollEventParams {
        delta_x: 0.0,
        delta_y: 0.0,
        ..Default::default()
    };
    let bytes = bincode::serialize(&params).expect("serialize");
    let rt: ScrollEventParams = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(rt.delta_x, 0.0);
    assert_eq!(rt.delta_y, 0.0);
}

/// 测试 IpcMessageKind Heartbeat/CrashNotification round-trip。
#[test]
fn test_heartbeat_and_crash_roundtrip() {
    let hb = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Heartbeat,
    };
    let rt = roundtrip(hb);
    assert!(matches!(rt.kind, IpcMessageKind::Heartbeat));

    let crash = IpcMessage {
        id: 2,
        kind: IpcMessageKind::CrashNotification("segfault at 0xdead".to_string()),
    };
    let rt2 = roundtrip(crash);
    match rt2.kind {
        IpcMessageKind::CrashNotification(msg) => {
            assert_eq!(msg, "segfault at 0xdead");
        }
        _ => panic!("Expected CrashNotification"),
    }
}

/// 测试 IpcMessageKind Ok/Error round-trip。
#[test]
fn test_ok_error_roundtrip() {
    let ok_msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Ok,
    };
    let rt = roundtrip(ok_msg);
    assert!(matches!(rt.kind, IpcMessageKind::Ok));

    let err_msg = IpcMessage {
        id: 2,
        kind: IpcMessageKind::Error("permission denied".to_string()),
    };
    let rt2 = roundtrip(err_msg);
    match rt2.kind {
        IpcMessageKind::Error(msg) => assert_eq!(msg, "permission denied"),
        _ => panic!("Expected Error"),
    }
}

/// 测试 ProcessRole 枚举值相等性（message 模块验证）。
#[test]
fn test_process_role_message_layer() {
    assert_eq!(ProcessRole::Browser, ProcessRole::Browser);
    assert_ne!(ProcessRole::Browser, ProcessRole::Renderer);
    // Copy + Clone 特性验证
    let r = ProcessRole::Renderer;
    let cloned = r;
    assert_eq!(r, cloned);
}

/// 测试空载荷消息的确定性编码。
#[test]
fn test_deterministic_encoding_empty_messages() {
    let msg1 = IpcMessage {
        id: 42,
        kind: IpcMessageKind::LoadComplete,
    };
    let msg2 = IpcMessage {
        id: 42,
        kind: IpcMessageKind::LoadComplete,
    };
    let b1 = serialize(&msg1).expect("s1");
    let b2 = serialize(&msg2).expect("s2");
    assert_eq!(b1, b2, "相同消息应产生确定性编码");
}

// ══════════════════════════════════════════════════════════
//  边界条件测试（第 5 批）：专项 edge case 覆盖
// ══════════════════════════════════════════════════════════

/// 测试 FetchParams method 字段大小写敏感性。
/// 验证小写 "get"、大写 "PATCH"、混合大小写 "delete" 在序列化往返后原样保留。
#[test]
fn test_fetch_params_method_case_sensitivity() {
    for (i, method) in ["get", "PATCH", "delete"].iter().enumerate() {
        let msg = IpcMessage {
            id: i as u64,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: i as u64,
                url: "https://example.com".into(),
                method: method.to_string(),
                headers: vec![],
                body: None,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchRequest(p) = out.kind {
            assert_eq!(
                method, &p.method,
                "method 字段大小写应原样保留，期望 {:?}，实际 {:?}",
                method, p.method
            );
        } else {
            panic!("期望 FetchRequest");
        }
    }
}

/// 测试 NavigateParams 自引用 referrer：url 和 referrer 设为相同值。
/// 验证序列化/反序列化后两个字段均保持原值，不互相干扰。
#[test]
fn test_navigate_self_referential_referrer() {
    let same_url = "https://example.com/page";
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: same_url.into(),
            referrer: Some(same_url.into()),
            navigation_epoch: 0,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::Navigate(p) = out.kind {
        assert_eq!(same_url, p.url, "url 应保持原值");
        assert_eq!(Some(same_url.to_string()), p.referrer, "referrer 应与 url 相同");
    } else {
        panic!("期望 Navigate");
    }
}

/// 测试 IpcMessageKind::Ok 与 Error("") 的序列化字节必须不同。
/// Ok 是无数据变体，Error("") 是空字符串变体，两者编码应有区别。
#[test]
fn test_ok_vs_error_empty_string_byte_distinctness() {
    let msg_ok = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Ok,
    };
    let msg_err = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Error(String::new()),
    };
    let bytes_ok = serialize(&msg_ok).expect("序列化 Ok 应成功");
    let bytes_err = serialize(&msg_err).expect("序列化 Error(\"\") 应成功");
    assert_ne!(bytes_ok, bytes_err, "Ok 和 Error(\"\") 的序列化字节必须不同");
}

/// 测试 ProcessRole 的 Debug 输出包含预期的变体名称。
#[test]
fn test_process_role_debug_output() {
    let browser_debug = format!("{:?}", ProcessRole::Browser);
    assert!(browser_debug.contains("Browser"), "Browser Debug 输出应包含 'Browser'");
    let renderer_debug = format!("{:?}", ProcessRole::Renderer);
    assert!(
        renderer_debug.contains("Renderer"),
        "Renderer Debug 输出应包含 'Renderer'"
    );
}

/// 测试 FetchResponseParams 常见 HTTP 状态码（100, 204, 304, 500, 503）的往返正确性。
#[test]
fn test_fetch_response_common_http_status_codes() {
    for (i, code) in [100u16, 204, 304, 500, 503].iter().enumerate() {
        let msg = IpcMessage {
            id: i as u64,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: i as u64,
                status_code: *code,
                headers: vec![],
                body: vec![],
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchResponse(p) = out.kind {
            assert_eq!(*code, p.status_code, "状态码 {} 往返后应保持不变", code);
        } else {
            panic!("期望 FetchResponse，状态码 {}", code);
        }
    }
}

/// 测试 FetchParams headers 中包含非 ASCII（UTF-8）值时的往返正确性。
/// 某些 HTTP header 值可能包含国际化文本，验证 IPC 传输不会丢失或损坏 UTF-8 编码。
#[test]
fn test_fetch_params_non_ascii_header_values() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 1,
            url: "https://example.com".into(),
            method: "GET".into(),
            headers: vec![
                ("X-Message".into(), "こんにちは世界".into()),
                ("X-Emoji".into(), "🎉🚀💻".into()),
                ("X-Accented".into(), "Ñoño café ☕".into()),
            ],
            body: None,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchRequest(p) = out.kind {
        assert_eq!(3, p.headers.len(), "应保留所有 3 个 header");
        assert_eq!("こんにちは世界", p.headers[0].1);
        assert_eq!("🎉🚀💻", p.headers[1].1);
        assert_eq!("Ñoño café ☕", p.headers[2].1);
    } else {
        panic!("期望 FetchRequest");
    }
}

/// 测试 StorageOpParams 在非 Set 操作（Get、Remove）时 value=Some("data") 的保留。
/// 虽然 Get/Remove 通常不需要 value，但协议层应允许 value 字段携带任意值而不丢失。
#[test]
fn test_storage_op_value_with_non_set_operations() {
    // Get 操作带 value
    let msg_get = IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Get,
            key: "my_key".into(),
            value: Some("data".into()),
            origin: "https://example.com".into(),
        }),
    };
    let out_get = roundtrip(msg_get);
    if let IpcMessageKind::StorageOp(p) = out_get.kind {
        assert_eq!(StorageOperation::Get, p.operation);
        assert_eq!(Some("data".into()), p.value, "Get 操作的 value 应保留");
    } else {
        panic!("期望 StorageOp (Get)");
    }

    // Remove 操作带 value
    let msg_remove = IpcMessage {
        id: 2,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Session,
            operation: StorageOperation::Remove,
            key: "my_key".into(),
            value: Some("data".into()),
            origin: "https://example.com".into(),
        }),
    };
    let out_remove = roundtrip(msg_remove);
    if let IpcMessageKind::StorageOp(p) = out_remove.kind {
        assert_eq!(StorageOperation::Remove, p.operation);
        assert_eq!(Some("data".into()), p.value, "Remove 操作的 value 应保留");
    } else {
        panic!("期望 StorageOp (Remove)");
    }
}

/// 测试 MockChannel 交替 send/recv 操作：发送一条、接收一条，循环 5 次。
/// 验证交替读写不会导致消息丢失或顺序混乱。
#[test]
fn test_mock_channel_interleaved_send_recv() {
    let mut ch = MockChannel::new();

    for i in 0u64..5 {
        ch.send(IpcMessage {
            id: i,
            kind: IpcMessageKind::Heartbeat,
        })
        .expect("发送应成功");
        let out = ch.recv().expect("接收应成功");
        assert_eq!(i, out.id, "交替 send/recv：期望 id={}，实际 id={}", i, out.id);
    }

    // 通道应为空
    assert!(ch.recv().is_err(), "交替操作后通道应为空");
}

/// 测试 MockChannel 满足 Send + Sync trait 约束（编译时检查）。
/// 验证 MockChannel 可以安全地跨线程使用。
#[test]
fn test_mock_channel_send_sync_compile_check() {
    fn assert_send_sync<T: Send + Sync>(_: &T) {}
    let ch = MockChannel::new();
    assert_send_sync(&ch);
}

/// 测试 FetchParams headers 为空 Vec 时的序列化/反序列化往返。
/// 空 Vec 与包含元素的 Vec 在编码上有区别，验证空 Vec 不会变成 None 或其他值。
#[test]
fn test_fetch_params_empty_headers_vec() {
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
        assert!(p.headers.is_empty(), "空 headers Vec 往返后应保持为空 Vec");
        assert_eq!(0, p.headers.len(), "headers 长度应为 0");
    } else {
        panic!("期望 FetchRequest");
    }
}

/// 测试 StorageOpParams key 为空字符串时的序列化/反序列化往返。
/// 空 key 是合法值，不应被序列化器丢弃或转换为 None。
#[test]
fn test_storage_op_empty_key_roundtrip() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Get,
            key: String::new(),
            value: None,
            origin: "https://example.com".into(),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::StorageOp(p) = out.kind {
        assert_eq!("", p.key, "空 key 往返后应保持为空字符串");
        assert!(p.key.is_empty());
    } else {
        panic!("期望 StorageOp");
    }
}

/// 测试 MouseEventParams 负坐标的序列化/反序列化往返。
/// 鼠标坐标在某些场景下可能为负值（如相对于子元素的坐标），验证 f32 负值完整保留。
#[test]
fn test_mouse_params_negative_coordinates() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::MouseEvent(MouseEventParams {
            x: -100.5,
            y: -200.75,
            button: 0,
            event_type: MouseEventType::Move,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::MouseEvent(p) = out.kind {
        assert_eq!(-100.5, p.x, "负 x 坐标应完整保留");
        assert_eq!(-200.75, p.y, "负 y 坐标应完整保留");
    } else {
        panic!("期望 MouseEvent");
    }
}

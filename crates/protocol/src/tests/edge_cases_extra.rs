//! 额外边界条件测试：聚焦未覆盖的 edge case 路径。

use super::*;

// ══════════════════════════════════════════════════════════
//  1. NavigateParams referrer 字段边界条件
// ══════════════════════════════════════════════════════════

/// 测试 referrer 包含换行符、制表符等空白字符时的序列化/反序列化。
/// 验证 referrer 中的特殊空白字符在 IPC 传输中不被丢弃或转换。
#[test]
fn test_navigate_referrer_with_whitespace_chars() {
    let referrer = "https://example.com/page\t?q=1\n#frag\r\n";
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: Some(referrer.into()),
            navigation_epoch: 0,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::Navigate(p) = out.kind {
        assert_eq!(
            Some(referrer.to_string()),
            p.referrer,
            "含空白字符的 referrer 往返应完全一致"
        );
    } else {
        panic!("期望 Navigate");
    }
}

/// 测试 referrer 为仅含空格的字符串时的序列化/反序列化。
/// 验证 Some("   ") 不会被规范化为 Some("") 或 None。
#[test]
fn test_navigate_referrer_whitespace_only() {
    let msg = IpcMessage {
        id: 2,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: Some("   ".into()),
            navigation_epoch: 0,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::Navigate(p) = out.kind {
        assert_eq!(Some("   ".to_string()), p.referrer, "纯空格 referrer 不应被规范化");
    } else {
        panic!("期望 Navigate");
    }
}

/// 测试 NavigateParams url 和 referrer 同时为空字符串。
#[test]
fn test_navigate_both_url_and_referrer_empty() {
    let msg = IpcMessage {
        id: 3,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: String::new(),
            referrer: Some(String::new()),
            navigation_epoch: 0,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::Navigate(p) = out.kind {
        assert!(p.url.is_empty(), "url 应为空字符串");
        assert!(p.referrer.is_some(), "referrer 应为 Some");
        assert!(p.referrer.unwrap().is_empty(), "referrer 内容应为空字符串");
    } else {
        panic!("期望 Navigate");
    }
}

/// 测试 referrer 包含 NUL 字节（\0）时的序列化/反序列化。
/// Rust String 允许包含 NUL 字节，验证 IPC 传输不会在 NUL 处截断。
#[test]
fn test_navigate_referrer_with_nul_byte() {
    let referrer = "https://example.com\0hidden";
    let msg = IpcMessage {
        id: 4,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: Some(referrer.into()),
            navigation_epoch: 0,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::Navigate(p) = out.kind {
        assert_eq!(
            Some(referrer.to_string()),
            p.referrer,
            "含 NUL 字节的 referrer 不应被截断"
        );
    } else {
        panic!("期望 Navigate");
    }
}

// ══════════════════════════════════════════════════════════
//  2. KeyboardEvent 修饰键组合边界
// ══════════════════════════════════════════════════════════

/// 测试 KeyboardEvent 仅 ctrl=true，其他修饰键为 false。
/// 验证单个修饰键独立往返，不会因其他修饰键为 false 而丢失。
#[test]
fn test_keyboard_event_ctrl_only() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
            key: "c".into(),
            code: "KeyC".into(),
            ctrl: true,
            shift: false,
            alt: false,
            meta: false,
            event_type: KeyboardEventType::Down,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::KeyboardEvent(p) = out.kind {
        assert!(p.ctrl, "ctrl 应为 true");
        assert!(!p.shift, "shift 应为 false");
        assert!(!p.alt, "alt 应为 false");
        assert!(!p.meta, "meta 应为 false");
    } else {
        panic!("期望 KeyboardEvent");
    }
}

/// 测试 KeyboardEvent 仅 alt=true，验证 alt 修饰键独立正确序列化。
#[test]
fn test_keyboard_event_alt_only() {
    let msg = IpcMessage {
        id: 2,
        kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
            key: "a".into(),
            code: "KeyA".into(),
            ctrl: false,
            shift: false,
            alt: true,
            meta: false,
            event_type: KeyboardEventType::Press,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::KeyboardEvent(p) = out.kind {
        assert!(!p.ctrl);
        assert!(!p.shift);
        assert!(p.alt, "alt 应为 true");
        assert!(!p.meta);
        assert_eq!(KeyboardEventType::Press, p.event_type);
    } else {
        panic!("期望 KeyboardEvent");
    }
}

/// 测试 KeyboardEvent ctrl+shift 组合（无 alt/meta），模拟 Ctrl+Shift+I 等常见快捷键。
#[test]
fn test_keyboard_event_ctrl_shift_combo() {
    let msg = IpcMessage {
        id: 3,
        kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
            key: "I".into(),
            code: "KeyI".into(),
            ctrl: true,
            shift: true,
            alt: false,
            meta: false,
            event_type: KeyboardEventType::Down,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::KeyboardEvent(p) = out.kind {
        assert!(p.ctrl && p.shift, "ctrl+shift 应同时为 true");
        assert!(!p.alt && !p.meta, "alt 和 meta 应为 false");
    } else {
        panic!("期望 KeyboardEvent");
    }
}

/// 测试 KeyboardEvent ctrl+alt+meta 组合（无 shift），模拟三键组合。
#[test]
fn test_keyboard_event_ctrl_alt_meta_combo() {
    let msg = IpcMessage {
        id: 4,
        kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
            key: "Delete".into(),
            code: "Delete".into(),
            ctrl: true,
            shift: false,
            alt: true,
            meta: true,
            event_type: KeyboardEventType::Up,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::KeyboardEvent(p) = out.kind {
        assert!(p.ctrl && p.alt && p.meta, "ctrl+alt+meta 应同时为 true");
        assert!(!p.shift, "shift 应为 false");
        assert_eq!(KeyboardEventType::Up, p.event_type);
    } else {
        panic!("期望 KeyboardEvent");
    }
}

// ══════════════════════════════════════════════════════════
//  3. MouseEvent button 类型变体
// ══════════════════════════════════════════════════════════

/// 测试 MouseEvent button=1（中键）在所有事件类型中的往返。
#[test]
fn test_mouse_event_middle_button_all_types() {
    let types = [
        MouseEventType::Down,
        MouseEventType::Up,
        MouseEventType::Click,
        MouseEventType::Move,
        MouseEventType::DblClick,
    ];
    for (i, etype) in types.iter().enumerate() {
        let msg = IpcMessage {
            id: i as u64,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 100.0,
                y: 200.0,
                button: 1,
                event_type: etype.clone(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::MouseEvent(p) = out.kind {
            assert_eq!(1, p.button, "button 应为 1（中键），类型 {:?}", etype);
            assert_eq!(*etype, p.event_type);
        } else {
            panic!("期望 MouseEvent，索引 {i}");
        }
    }
}

/// 测试 MouseEvent button=2（右键）的 Click 和 Down 事件往返。
#[test]
fn test_mouse_event_right_button_click_and_down() {
    // 右键 Down
    let msg_down = IpcMessage {
        id: 1,
        kind: IpcMessageKind::MouseEvent(MouseEventParams {
            x: 50.0,
            y: 75.0,
            button: 2,
            event_type: MouseEventType::Down,
        }),
    };
    let out_down = roundtrip(msg_down);
    if let IpcMessageKind::MouseEvent(p) = out_down.kind {
        assert_eq!(2, p.button, "button 应为 2（右键）");
        assert_eq!(MouseEventType::Down, p.event_type);
    } else {
        panic!("期望 MouseEvent");
    }

    // 右键 Click
    let msg_click = IpcMessage {
        id: 2,
        kind: IpcMessageKind::MouseEvent(MouseEventParams {
            x: 50.0,
            y: 75.0,
            button: 2,
            event_type: MouseEventType::Click,
        }),
    };
    let out_click = roundtrip(msg_click);
    if let IpcMessageKind::MouseEvent(p) = out_click.kind {
        assert_eq!(2, p.button, "button 应为 2（右键）");
        assert_eq!(MouseEventType::Click, p.event_type);
    } else {
        panic!("期望 MouseEvent");
    }
}

/// 测试 MouseEvent button=3（后退键）和 button=4（前进键）的序列化往返。
/// 部分鼠标有额外的侧键，button 值大于 2 是合法的。
#[test]
fn test_mouse_event_extended_buttons() {
    for btn in [3u8, 4, 5, 10] {
        let msg = IpcMessage {
            id: btn as u64,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 0.0,
                y: 0.0,
                button: btn,
                event_type: MouseEventType::Down,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::MouseEvent(p) = out.kind {
            assert_eq!(btn, p.button, "button={} 往返应保持不变", btn);
        } else {
            panic!("期望 MouseEvent，button={}", btn);
        }
    }
}

// ══════════════════════════════════════════════════════════
//  4. ScrollEvent delta 边界值
// ══════════════════════════════════════════════════════════

/// 测试 ScrollEvent 一个轴为零、另一个轴为非零值的往返。
/// 模拟纯水平滚动（delta_x > 0, delta_y = 0）或纯垂直滚动。
#[test]
fn test_scroll_event_single_axis_scroll() {
    // 纯水平滚动
    let msg_h = IpcMessage {
        id: 1,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: 120.0,
            delta_y: 0.0,
            ..Default::default()
        }),
    };
    let out_h = roundtrip(msg_h);
    if let IpcMessageKind::ScrollEvent(p) = out_h.kind {
        assert_eq!(120.0, p.delta_x);
        assert_eq!(0.0, p.delta_y);
    } else {
        panic!("期望 ScrollEvent");
    }

    // 纯垂直滚动
    let msg_v = IpcMessage {
        id: 2,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: 0.0,
            delta_y: -45.5,
            ..Default::default()
        }),
    };
    let out_v = roundtrip(msg_v);
    if let IpcMessageKind::ScrollEvent(p) = out_v.kind {
        assert_eq!(0.0, p.delta_x);
        assert_eq!(-45.5, p.delta_y);
    } else {
        panic!("期望 ScrollEvent");
    }
}

/// 测试 ScrollEvent 使用 f32 极小非零值（次正规数）和极大值时的往返。
#[test]
fn test_scroll_event_extreme_float_values() {
    // 极小非零值（次正规数）
    let tiny = f32::from_bits(1); // 最小正次正规数
    let msg_tiny = IpcMessage {
        id: 1,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: tiny,
            delta_y: -tiny,
            ..Default::default()
        }),
    };
    let out_tiny = roundtrip(msg_tiny);
    if let IpcMessageKind::ScrollEvent(p) = out_tiny.kind {
        assert_eq!(
            tiny.to_bits(),
            p.delta_x.to_bits(),
            "极小正次正规数 delta_x 位模式应保留"
        );
        assert_eq!(
            (-tiny).to_bits(),
            p.delta_y.to_bits(),
            "极小负次正规数 delta_y 位模式应保留"
        );
    } else {
        panic!("期望 ScrollEvent");
    }

    // f32 最大有限值
    let msg_max = IpcMessage {
        id: 2,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: f32::MAX,
            delta_y: f32::MIN,
            ..Default::default()
        }),
    };
    let out_max = roundtrip(msg_max);
    if let IpcMessageKind::ScrollEvent(p) = out_max.kind {
        assert_eq!(f32::MAX, p.delta_x, "delta_x=f32::MAX 应完整保留");
        assert_eq!(f32::MIN, p.delta_y, "delta_y=f32::MIN 应完整保留");
    } else {
        panic!("期望 ScrollEvent");
    }
}

/// 测试 ScrollEvent 正负 delta 组合的序列化字节互不相同。
/// 验证 (delta_x=1.0, delta_y=2.0) 和 (delta_x=-1.0, delta_y=-2.0) 编码不同。
#[test]
fn test_scroll_event_positive_negative_deltas_distinct_bytes() {
    let msg_pos = IpcMessage {
        id: 1,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: 100.0,
            delta_y: 200.0,
            ..Default::default()
        }),
    };
    let msg_neg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: -100.0,
            delta_y: -200.0,
            ..Default::default()
        }),
    };
    let bytes_pos = serialize(&msg_pos).expect("serialize positive");
    let bytes_neg = serialize(&msg_neg).expect("serialize negative");
    assert_ne!(bytes_pos, bytes_neg, "正负 delta 的序列化字节应不同");
}

/// R3298（元素滚动 RFC S1）：`ScrollEventParams` 新增 `cursor_x`/`cursor_y`（滚轮视口坐标）
/// 字段的往返 + 向后兼容性。
///
/// - 字段显式赋值时序列化→反序列化完整保留（4 字段全往返）。
/// - 旧式构造（仅 delta_x/delta_y + `..Default::default()`）cursor 退化为 0.0（向后兼容旧发送端）。
/// - 不同 cursor 坐标的序列化字节不同（保证 cursor 真入 wire-format，非被序列化器忽略）。
#[test]
fn test_scroll_event_cursor_fields_roundtrip_r3298() {
    // 4 字段显式构造：delta + cursor 全往返
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: 0.0,
            delta_y: 120.0,
            cursor_x: 320.0,
            cursor_y: 480.0,
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::ScrollEvent(p) = out.kind {
        assert_eq!(0.0, p.delta_x);
        assert_eq!(120.0, p.delta_y);
        assert_eq!(320.0, p.cursor_x, "cursor_x 应完整往返");
        assert_eq!(480.0, p.cursor_y, "cursor_y 应完整往返");
    } else {
        panic!("期望 ScrollEvent");
    }

    // 向后兼容：旧式构造 cursor 默认 0.0
    let msg_legacy = IpcMessage {
        id: 2,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: 0.0,
            delta_y: 50.0,
            ..Default::default()
        }),
    };
    let out_legacy = roundtrip(msg_legacy);
    if let IpcMessageKind::ScrollEvent(p) = out_legacy.kind {
        assert_eq!(0.0, p.cursor_x, "旧式构造 cursor_x 应默认 0.0");
        assert_eq!(0.0, p.cursor_y, "旧式构造 cursor_y 应默认 0.0");
    } else {
        panic!("期望 ScrollEvent");
    }

    // cursor 真入 wire-format：不同 cursor 的序列化字节不同
    let msg_a = IpcMessage {
        id: 3,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: 0.0,
            delta_y: 50.0,
            cursor_x: 10.0,
            cursor_y: 20.0,
        }),
    };
    let msg_b = IpcMessage {
        id: 3,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: 0.0,
            delta_y: 50.0,
            cursor_x: 999.0,
            cursor_y: 888.0,
        }),
    };
    let bytes_a = serialize(&msg_a).expect("serialize cursor A");
    let bytes_b = serialize(&msg_b).expect("serialize cursor B");
    assert_ne!(
        bytes_a, bytes_b,
        "不同 cursor 坐标的序列化字节应不同（cursor 真入 wire-format）"
    );
}

// ══════════════════════════════════════════════════════════
//  5. StorageOp 操作类型边界条件
// ══════════════════════════════════════════════════════════

/// 测试 StorageOp Clear 操作的 value=Some("data") 往返。
/// Clear 操作通常不需要 value，但协议层应允许携带。
#[test]
fn test_storage_op_clear_with_value() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Clear,
            key: String::new(),
            value: Some("unexpected".into()),
            origin: "https://example.com".into(),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::StorageOp(p) = out.kind {
        assert_eq!(StorageOperation::Clear, p.operation);
        assert_eq!(Some("unexpected".into()), p.value, "Clear 操作的 value 应保留");
    } else {
        panic!("期望 StorageOp");
    }
}

/// 测试 StorageOp Length 操作的完整字段往返。
#[test]
fn test_storage_op_length_all_fields() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Session,
            operation: StorageOperation::Length,
            key: String::new(),
            value: None,
            origin: "https://app.example.com".into(),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::StorageOp(p) = out.kind {
        assert_eq!(StorageType::Session, p.storage_type);
        assert_eq!(StorageOperation::Length, p.operation);
        assert!(p.key.is_empty());
        assert!(p.value.is_none());
        assert_eq!("https://app.example.com", p.origin);
    } else {
        panic!("期望 StorageOp");
    }
}

/// 测试 StorageOp Key 操作（按索引获取键名）的序列化往返。
#[test]
fn test_storage_op_key_by_index() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Key,
            key: "0".into(),
            value: None,
            origin: "https://example.com".into(),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::StorageOp(p) = out.kind {
        assert_eq!(StorageOperation::Key, p.operation);
        assert_eq!("0", p.key, "key='0' 表示获取第一个键名");
    } else {
        panic!("期望 StorageOp");
    }
}

/// 测试 StorageOp origin 为空字符串时的序列化/反序列化。
#[test]
fn test_storage_op_empty_origin() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Get,
            key: "test_key".into(),
            value: None,
            origin: String::new(),
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::StorageOp(p) = out.kind {
        assert!(p.origin.is_empty(), "空 origin 往返后应保持为空字符串");
    } else {
        panic!("期望 StorageOp");
    }
}

// ══════════════════════════════════════════════════════════
//  6. ProcessRole Debug trait 完整验证
// ══════════════════════════════════════════════════════════

/// 测试 ProcessRole Debug 格式化输出的精确格式。
/// 验证 Debug 输出不仅包含变体名，且格式符合 `VariantName` 或 `EnumName::VariantName` 模式。
#[test]
fn test_process_role_debug_format_precision() {
    let browser = format!("{:?}", ProcessRole::Browser);
    let renderer = format!("{:?}", ProcessRole::Renderer);

    // 两个 Debug 输出不应相同
    assert_ne!(browser, renderer, "Browser 和 Renderer 的 Debug 输出应不同");
    // 各自应包含自身变体名
    assert!(browser.contains("Browser"), "Browser Debug 应包含 'Browser'");
    assert!(renderer.contains("Renderer"), "Renderer Debug 应包含 'Renderer'");
    // Debug 输出不应为空
    assert!(!browser.is_empty());
    assert!(!renderer.is_empty());
}

// ══════════════════════════════════════════════════════════
//  7. ProtocolError Display trait 格式化边界条件
// ══════════════════════════════════════════════════════════

/// 测试 ProtocolError Display 在错误消息为空字符串时的输出。
/// 验证空错误消息不会导致 Display 输出为空。
#[test]
fn test_protocol_error_display_empty_message() {
    let err = ProtocolError::Serialization(String::new());
    let display = format!("{err}");
    assert!(!display.is_empty(), "即使错误消息为空，Display 输出也不应为空");
    assert!(display.contains("Serialization error"), "应包含错误类型前缀");
}

/// 测试 ProtocolError Display 在错误消息包含特殊字符时的输出。
#[test]
fn test_protocol_error_display_special_chars() {
    let err = ProtocolError::Channel("error: \n\t\r\0\"'<>".into());
    let display = format!("{err}");
    assert!(display.contains("Channel error"), "应包含错误类型前缀");
    // 特殊字符应被保留在 Display 输出中
    assert!(display.contains('\n'), "换行符应被保留");
    assert!(display.contains('\t'), "制表符应被保留");
}

/// 测试 ProtocolError Display 在错误消息包含多语言文本时的输出。
#[test]
fn test_protocol_error_display_unicode_message() {
    let err = ProtocolError::Process("进程崩溃：内存溢出 💥".into());
    let display = format!("{err}");
    assert!(display.contains("Process error"), "应包含错误类型前缀");
    assert!(
        display.contains("进程崩溃：内存溢出 💥"),
        "Unicode 错误消息应被完整保留"
    );
}

// ══════════════════════════════════════════════════════════
//  8. IPC 消息序列化附加边界条件
// ══════════════════════════════════════════════════════════

/// 测试 FetchRequest body 为 Some(vec![])（空 Vec）与 None 的序列化字节不同。
/// 空 Vec 和 None 在语义上不同，序列化编码也应不同。
#[test]
fn test_fetch_request_empty_vec_body_vs_none() {
    let msg_empty_vec = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 1,
            url: "https://example.com".into(),
            method: "POST".into(),
            headers: vec![],
            body: Some(vec![]),
        }),
    };
    let msg_none = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 1,
            url: "https://example.com".into(),
            method: "POST".into(),
            headers: vec![],
            body: None,
        }),
    };
    let bytes_empty = serialize(&msg_empty_vec).expect("serialize empty vec");
    let bytes_none = serialize(&msg_none).expect("serialize none");
    assert_ne!(bytes_empty, bytes_none, "Some(vec![]) 和 None 的序列化字节应不同");

    // 验证往返后 body 类型保持
    let out_empty = roundtrip(msg_empty_vec);
    if let IpcMessageKind::FetchRequest(p) = out_empty.kind {
        assert!(p.body.is_some(), "body 应为 Some");
        assert!(p.body.unwrap().is_empty(), "body 内容应为空 Vec");
    } else {
        panic!("期望 FetchRequest");
    }

    let out_none = roundtrip(msg_none);
    if let IpcMessageKind::FetchRequest(p) = out_none.kind {
        assert!(p.body.is_none(), "body 应为 None");
    } else {
        panic!("期望 FetchRequest");
    }
}

/// 测试 FetchResponse 状态码 0（非标准 HTTP 状态码）的往返。
/// 验证 u16 最小值在 status_code 字段中正确保留。
#[test]
fn test_fetch_response_status_code_zero() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 1,
            status_code: 0,
            headers: vec![],
            body: vec![],
        }),
    };
    let out = roundtrip(msg);
    if let IpcMessageKind::FetchResponse(p) = out.kind {
        assert_eq!(0u16, p.status_code, "status_code=0 应完整保留");
    } else {
        panic!("期望 FetchResponse");
    }
}

/// 测试 IpcMessage id 为 1（最小非零值）时的往返。
/// 验证 id=1 不会与 id=0 或缺失混淆。
#[test]
fn test_ipc_message_id_one() {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Ok,
    };
    let bytes = serialize(&msg).expect("serialize");
    let msg0 = IpcMessage {
        id: 0,
        kind: IpcMessageKind::Ok,
    };
    let bytes0 = serialize(&msg0).expect("serialize");
    assert_ne!(bytes, bytes0, "id=1 和 id=0 的序列化字节应不同");
}

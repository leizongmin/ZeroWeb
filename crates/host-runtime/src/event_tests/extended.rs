use super::*;

/// 验证：多个修饰键同时按下时，每个修饰键独立产生 KeyboardInput 事件，
/// 事件的 key 和 pressed 字段均正确。
/// 模拟 Ctrl+Shift+Alt 组合键场景。
#[test]
fn test_event_modifiers_combination() {
    // 模拟 Ctrl+Shift+Alt 组合键的完整按下和释放序列
    let events: Vec<AppEvent> = vec![
        // 1. Ctrl 按下
        AppEvent::KeyboardInput {
            key: "Control".to_string(),
            pressed: true,
        },
        // 2. Shift 按下（Ctrl 仍按住）
        AppEvent::KeyboardInput {
            key: "Shift".to_string(),
            pressed: true,
        },
        // 3. Alt 按下（Ctrl+Shift 仍按住）
        AppEvent::KeyboardInput {
            key: "Alt".to_string(),
            pressed: true,
        },
        // 4. 字符键按下（Ctrl+Shift+Alt 全部按住）
        AppEvent::KeyboardInput {
            key: "A".to_string(),
            pressed: true,
        },
        // 5. 字符键释放
        AppEvent::KeyboardInput {
            key: "A".to_string(),
            pressed: false,
        },
        // 6. Alt 释放
        AppEvent::KeyboardInput {
            key: "Alt".to_string(),
            pressed: false,
        },
        // 7. Shift 释放
        AppEvent::KeyboardInput {
            key: "Shift".to_string(),
            pressed: false,
        },
        // 8. Ctrl 释放
        AppEvent::KeyboardInput {
            key: "Control".to_string(),
            pressed: false,
        },
    ];

    // 验证事件数量
    assert_eq!(events.len(), 8, "完整的修饰键组合应产生 8 个事件");

    // 验证每个修饰键按下事件的 pressed 为 true
    let modifier_presses: Vec<&AppEvent> = events.iter().take(3).collect();
    for (i, event) in modifier_presses.iter().enumerate() {
        if let AppEvent::KeyboardInput { pressed, .. } = event {
            assert!(pressed, "第 {} 个修饰键按下事件 pressed 应为 true", i + 1);
        } else {
            panic!("前三个事件应为 KeyboardInput");
        }
    }

    // 验证修饰键名称正确
    if let AppEvent::KeyboardInput { key, .. } = &events[0] {
        assert_eq!(key, "Control");
    }
    if let AppEvent::KeyboardInput { key, .. } = &events[1] {
        assert_eq!(key, "Shift");
    }
    if let AppEvent::KeyboardInput { key, .. } = &events[2] {
        assert_eq!(key, "Alt");
    }

    // 验证字符键按下和释放
    if let AppEvent::KeyboardInput { key, pressed } = &events[3] {
        assert_eq!(key, "A");
        assert!(pressed, "字符键按下 pressed 应为 true");
    }
    if let AppEvent::KeyboardInput { key, pressed } = &events[4] {
        assert_eq!(key, "A");
        assert!(!pressed, "字符键释放 pressed 应为 false");
    }

    // 验证释放顺序与按下顺序相反
    let release_keys: Vec<String> = events[5..]
        .iter()
        .map(|e| {
            if let AppEvent::KeyboardInput { key, pressed } = e {
                assert!(!pressed, "释放事件 pressed 应为 false");
                key.clone()
            } else {
                panic!("应为 KeyboardInput");
            }
        })
        .collect();
    assert_eq!(release_keys, vec!["Alt", "Shift", "Control"]);
}

/// 验证：按键重复事件的 pressed 字段始终为 true。
/// 当用户长按某个键时，操作系统会产生多次按下事件（key repeat），
/// 这些事件的 pressed 字段应全部为 true（而非混合 true/false）。
#[test]
fn test_key_event_repeat_flag() {
    // 模拟长按 "A" 键产生的事件序列：首次按下 + 多次重复
    let repeat_events: Vec<AppEvent> = vec![
        AppEvent::KeyboardInput {
            key: "A".to_string(),
            pressed: true,
        },
        AppEvent::KeyboardInput {
            key: "A".to_string(),
            pressed: true,
        },
        AppEvent::KeyboardInput {
            key: "A".to_string(),
            pressed: true,
        },
        AppEvent::KeyboardInput {
            key: "A".to_string(),
            pressed: true,
        },
        AppEvent::KeyboardInput {
            key: "A".to_string(),
            pressed: false,
        },
    ];

    // 验证所有重复按下事件的 key 和 pressed 字段
    for (i, event) in repeat_events.iter().enumerate() {
        if let AppEvent::KeyboardInput { key, pressed } = event {
            assert_eq!(key, "A", "第 {} 个事件的 key 应为 'A'", i + 1);
            if i < 4 {
                assert!(pressed, "第 {} 个 repeat 事件的 pressed 应为 true", i + 1);
            } else {
                // 最终释放事件
                assert!(!pressed, "释放事件的 pressed 应为 false");
            }
        } else {
            panic!("应为 KeyboardInput 变体");
        }
    }

    // 验证：重复事件数量为 4 次按下 + 1 次释放 = 5
    assert_eq!(repeat_events.len(), 5);

    // 验证：通过 element_state_to_pressed 转换，所有 Pressed 状态映射为 true
    for _ in 0..4 {
        assert!(element_state_to_pressed(winit::event::ElementState::Pressed));
    }
}

/// 验证：鼠标左键、右键、中键、后退、前进五个标准按钮值的完整覆盖。
/// 确保每种按钮的 Debug 输出、等价性和互不相等性均正确。
#[test]
fn test_mouse_button_all_variants() {
    let buttons = [
        MouseButton::Left,
        MouseButton::Right,
        MouseButton::Middle,
        MouseButton::Back,
        MouseButton::Forward,
    ];

    // 验证 Debug 输出非空且各不相同
    let debug_outputs: Vec<String> = buttons.iter().map(|b| format!("{:?}", b)).collect();
    for debug in &debug_outputs {
        assert!(!debug.is_empty(), "MouseButton Debug 输出不应为空");
    }
    for i in 0..debug_outputs.len() {
        for j in (i + 1)..debug_outputs.len() {
            assert_ne!(
                debug_outputs[i], debug_outputs[j],
                "{:?} 和 {:?} 的 Debug 输出不应相同",
                buttons[i], buttons[j]
            );
        }
    }

    // 验证每种按钮的自等价性
    for btn in &buttons {
        assert_eq!(*btn, *btn, "{:?} 应等于自身", btn);
    }

    // 验证五种按钮两两互不相等
    for i in 0..buttons.len() {
        for j in (i + 1)..buttons.len() {
            assert_ne!(buttons[i], buttons[j], "{:?} 不应等于 {:?}", buttons[i], buttons[j]);
        }
    }

    // 验证五种按钮通过 convert_mouse_button 转换正确
    let winit_buttons = [
        winit::event::MouseButton::Left,
        winit::event::MouseButton::Right,
        winit::event::MouseButton::Middle,
        winit::event::MouseButton::Back,
        winit::event::MouseButton::Forward,
    ];
    for (winit_btn, expected_btn) in winit_buttons.iter().zip(buttons.iter()) {
        assert_eq!(
            convert_mouse_button(*winit_btn),
            *expected_btn,
            "convert_mouse_button({:?}) 应返回 {:?}",
            winit_btn,
            expected_btn
        );
    }

    // 验证 Copy 语义正确
    for btn in &buttons {
        let copied = *btn;
        assert_eq!(*btn, copied);
    }
}

/// 验证：鼠标进入/离开窗口时，CursorMoved 事件携带正确的坐标。
/// 由于当前 AppEvent 使用 MouseMoved 统一表示光标位置，
/// 进入和离开均通过 MouseMoved 事件传递坐标信息。
#[test]
fn test_mouse_enter_leave_coordinates() {
    // 模拟鼠标进入窗口：光标从窗口外移入窗口内
    let enter_event = AppEvent::MouseMoved { x: 0.0, y: 0.0 };
    if let AppEvent::MouseMoved { x, y } = enter_event {
        assert!((x - 0.0).abs() < f64::EPSILON, "鼠标进入窗口的 x 坐标应为 0.0");
        assert!((y - 0.0).abs() < f64::EPSILON, "鼠标进入窗口的 y 坐标应为 0.0");
    } else {
        panic!("Expected MouseMoved variant");
    }

    // 模拟鼠标在窗口内移动
    let move_inside = AppEvent::MouseMoved { x: 500.5, y: 300.25 };
    if let AppEvent::MouseMoved { x, y } = move_inside {
        assert!((x - 500.5).abs() < f64::EPSILON);
        assert!((y - 300.25).abs() < f64::EPSILON);
    } else {
        panic!("Expected MouseMoved variant");
    }

    // 模拟鼠标离开窗口：光标移动到窗口边界外
    let leave_event = AppEvent::MouseMoved { x: -5.0, y: -10.0 };
    if let AppEvent::MouseMoved { x, y } = leave_event {
        assert!((x - (-5.0)).abs() < f64::EPSILON, "鼠标离开窗口的 x 坐标应为负值");
        assert!((y - (-10.0)).abs() < f64::EPSILON, "鼠标离开窗口的 y 坐标应为负值");
    } else {
        panic!("Expected MouseMoved variant");
    }

    // 模拟鼠标从右侧离开窗口
    let leave_right = AppEvent::MouseMoved { x: 1921.0, y: 1080.0 };
    if let AppEvent::MouseMoved { x, y } = leave_right {
        assert!((x - 1921.0).abs() < f64::EPSILON);
        assert!((y - 1080.0).abs() < f64::EPSILON);
    } else {
        panic!("Expected MouseMoved variant");
    }

    // 通过 BasicApp 分发路径验证坐标传递
    let mut received: Vec<AppEvent> = Vec::new();
    let mut callback = |e: AppEvent| received.push(e);
    let attrs = winit::window::WindowAttributes::default();
    let mut app = crate::window::BasicApp::new_basic(attrs, &mut callback);

    // 模拟进入事件
    app.handle_window_event(winit::event::WindowEvent::CursorMoved {
        device_id: winit::event::DeviceId::dummy(),
        position: winit::dpi::PhysicalPosition::new(0.0, 0.0),
    });
    // 模拟离开事件
    app.handle_window_event(winit::event::WindowEvent::CursorMoved {
        device_id: winit::event::DeviceId::dummy(),
        position: winit::dpi::PhysicalPosition::new(-50.0, -100.0),
    });

    assert_eq!(received.len(), 2, "应收到两个 MouseMoved 事件");
    // 进入事件坐标
    match &received[0] {
        AppEvent::MouseMoved { x, y } => {
            assert!((*x - 0.0).abs() < f64::EPSILON);
            assert!((*y - 0.0).abs() < f64::EPSILON);
        }
        _ => panic!("Expected MouseMoved"),
    }
    // 离开事件坐标
    match &received[1] {
        AppEvent::MouseMoved { x, y } => {
            assert!((*x - (-50.0)).abs() < f64::EPSILON);
            assert!((*y - (-100.0)).abs() < f64::EPSILON);
        }
        _ => panic!("Expected MouseMoved"),
    }
}

/// 验证：窗口 resize 到 0x0 尺寸时不会 panic，且事件正确携带零值。
/// 某些平台在窗口最小化时会发出 (0, 0) 尺寸的 resize 事件。
#[test]
fn test_resize_to_zero_dimensions() {
    // 直接构造 AppEvent::Resized { 0, 0 }，验证不会 panic
    let event = AppEvent::Resized { width: 0, height: 0 };
    if let AppEvent::Resized { width, height } = event {
        assert_eq!(width, 0, "零宽度应正确存储");
        assert_eq!(height, 0, "零高度应正确存储");
    } else {
        panic!("Expected Resized variant");
    }

    // 通过 winit PhysicalSize 构造零尺寸
    let size = winit::dpi::PhysicalSize::new(0u32, 0u32);
    assert_eq!(size.width, 0);
    assert_eq!(size.height, 0);

    // 通过 BasicApp 分发路径验证 0x0 resize 不 panic
    let mut received: Vec<AppEvent> = Vec::new();
    let mut callback = |e: AppEvent| received.push(e);
    let attrs = winit::window::WindowAttributes::default();
    let mut app = crate::window::BasicApp::new_basic(attrs, &mut callback);

    app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
        0u32, 0u32,
    )));
    assert_eq!(received.len(), 1, "应收到一个 Resized 事件");
    match &received[0] {
        AppEvent::Resized { width, height } => {
            assert_eq!(*width, 0, "分发后的宽度应为 0");
            assert_eq!(*height, 0, "分发后的高度应为 0");
        }
        _ => panic!("Expected Resized"),
    }

    // 通过 GpuApp 分发路径验证 0x0 resize 不 panic
    let mut received_gpu: Vec<AppEvent> = Vec::new();
    let mut gpu_callback = |e: AppEvent, _: Option<std::sync::Arc<winit::window::Window>>| {
        received_gpu.push(e);
    };
    let gpu_attrs = winit::window::WindowAttributes::default();
    let mut gpu_app = crate::window::GpuApp::new_with_window(gpu_attrs, &mut gpu_callback);
    gpu_app.handle_window_event(
        winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(0u32, 0u32)),
        None,
    );
    assert_eq!(received_gpu.len(), 1);
    match &received_gpu[0] {
        AppEvent::Resized { width, height } => {
            assert_eq!(*width, 0);
            assert_eq!(*height, 0);
        }
        _ => panic!("Expected Resized"),
    }

    // Debug 格式化不应 panic
    let debug_str = format!("{:?}", event);
    assert!(!debug_str.is_empty());
}

/// 验证：IME 组合事件在空字符串情况下的完整处理（含分发路径验证）。
/// 某些输入法在特定状态下可能提交或预编辑空字符串。
#[test]
fn test_ime_composition_empty_string_dispatch() {
    // 空字符串 Commit 事件
    let commit_empty = ImeEvent::Commit(String::new());
    if let ImeEvent::Commit(text) = &commit_empty {
        assert!(text.is_empty(), "空字符串 commit 的 text 应为空");
    } else {
        panic!("Expected Commit variant");
    }

    // 空字符串 Preedit 事件（无光标）
    let preedit_empty = ImeEvent::Preedit {
        text: String::new(),
        cursor: None,
    };
    if let ImeEvent::Preedit { text, cursor } = &preedit_empty {
        assert!(text.is_empty(), "空 preedit 的 text 应为空");
        assert!(cursor.is_none(), "空 preedit 的 cursor 应为 None");
    } else {
        panic!("Expected Preedit variant");
    }

    // 空字符串 Preedit 事件（带光标 (0, 0)）
    let preedit_empty_with_cursor = ImeEvent::Preedit {
        text: String::new(),
        cursor: Some((0, 0)),
    };
    if let ImeEvent::Preedit { text, cursor } = &preedit_empty_with_cursor {
        assert!(text.is_empty());
        assert_eq!(*cursor, Some((0, 0)));
    } else {
        panic!("Expected Preedit variant");
    }

    // 通过 winit 转换路径验证空字符串
    assert_eq!(
        convert_ime(winit::event::Ime::Commit(String::new())),
        ImeEvent::Commit(String::new()),
        "winit 空 commit 转换应一致"
    );
    assert_eq!(
        convert_ime(winit::event::Ime::Preedit(String::new(), None)),
        ImeEvent::Preedit {
            text: String::new(),
            cursor: None,
        },
        "winit 空 preedit 转换应一致"
    );

    // 等价性验证
    assert_eq!(commit_empty, ImeEvent::Commit(String::new()));
    assert_eq!(
        preedit_empty,
        ImeEvent::Preedit {
            text: String::new(),
            cursor: None,
        }
    );

    // 通过 BasicApp 分发路径验证空 IME 事件不 panic
    let mut received: Vec<AppEvent> = Vec::new();
    let mut callback = |e: AppEvent| received.push(e);
    let attrs = winit::window::WindowAttributes::default();
    let mut app = crate::window::BasicApp::new_basic(attrs, &mut callback);

    // 分发空 Preedit
    app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Preedit(
        String::new(),
        None,
    )));
    // 分发空 Commit
    app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Commit(String::new())));

    assert_eq!(received.len(), 2, "应收到两个 IME 事件");
    match &received[0] {
        AppEvent::Ime(ImeEvent::Preedit { text, cursor }) => {
            assert!(text.is_empty());
            assert!(cursor.is_none());
        }
        _ => panic!("Expected Ime(Preedit)"),
    }
    match &received[1] {
        AppEvent::Ime(ImeEvent::Commit(s)) => {
            assert!(s.is_empty());
        }
        _ => panic!("Expected Ime(Commit)"),
    }
}

/// 验证：Touch event with multiple touch points.
/// 多个触摸点事件各自携带独立的 id、phase 和坐标。
#[test]
fn test_touch_event_coordinates() {
    let touch1 = TouchEvent {
        id: 0,
        phase: TouchPhase::Started,
        x: 100.0,
        y: 200.0,
    };
    let touch2 = TouchEvent {
        id: 1,
        phase: TouchPhase::Started,
        x: 300.0,
        y: 400.0,
    };
    let touch3 = TouchEvent {
        id: 2,
        phase: TouchPhase::Moved,
        x: 150.0,
        y: 250.0,
    };

    // 验证每个触摸点的坐标独立
    assert!((touch1.x - 100.0).abs() < f64::EPSILON);
    assert!((touch1.y - 200.0).abs() < f64::EPSILON);
    assert!((touch2.x - 300.0).abs() < f64::EPSILON);
    assert!((touch2.y - 400.0).abs() < f64::EPSILON);
    assert!((touch3.x - 150.0).abs() < f64::EPSILON);
    assert!((touch3.y - 250.0).abs() < f64::EPSILON);

    // 验证每个触摸点的 id 独立
    assert_eq!(touch1.id, 0);
    assert_eq!(touch2.id, 1);
    assert_eq!(touch3.id, 2);

    // 验证 phase 正确
    assert_eq!(touch1.phase, TouchPhase::Started);
    assert_eq!(touch2.phase, TouchPhase::Started);
    assert_eq!(touch3.phase, TouchPhase::Moved);

    // 模拟多点触发的结束：touch1 结束，touch2 移动，touch3 取消
    let touch1_end = TouchEvent {
        id: 0,
        phase: TouchPhase::Ended,
        x: 110.0,
        y: 210.0,
    };
    let touch2_move = TouchEvent {
        id: 1,
        phase: TouchPhase::Moved,
        x: 310.0,
        y: 410.0,
    };
    let touch3_cancel = TouchEvent {
        id: 2,
        phase: TouchPhase::Cancelled,
        x: 150.0,
        y: 250.0,
    };
    assert_eq!(touch1_end.phase, TouchPhase::Ended);
    assert_eq!(touch2_move.phase, TouchPhase::Moved);
    assert_eq!(touch3_cancel.phase, TouchPhase::Cancelled);
}

/// 验证：所有转换函数对极端输入的鲁棒性
#[test]
fn test_conversion_functions_robustness() {
    // convert_mouse_button: Other(0) 和 Other(u16::MAX) 边界值
    assert_eq!(
        convert_mouse_button(winit::event::MouseButton::Other(0)),
        MouseButton::Other(0)
    );
    assert_eq!(
        convert_mouse_button(winit::event::MouseButton::Other(u16::MAX)),
        MouseButton::Other(u16::MAX)
    );

    // convert_scroll_delta: PixelDelta 极端值
    let extreme_pos = winit::dpi::PhysicalPosition::new(f64::MAX, f64::MIN);
    let result = convert_scroll_delta(winit::event::MouseScrollDelta::PixelDelta(extreme_pos));
    assert_eq!(result, MouseScrollDelta::PixelDelta(f64::MAX, f64::MIN));

    // convert_scroll_delta: LineDelta 零值
    let result = convert_scroll_delta(winit::event::MouseScrollDelta::LineDelta(0.0, 0.0));
    assert_eq!(result, MouseScrollDelta::LineDelta(0.0, 0.0));

    // convert_ime: 空字符串 Preedit 和 Commit
    assert_eq!(
        convert_ime(winit::event::Ime::Preedit(String::new(), None)),
        ImeEvent::Preedit {
            text: String::new(),
            cursor: None
        }
    );
    assert_eq!(
        convert_ime(winit::event::Ime::Commit(String::new())),
        ImeEvent::Commit(String::new())
    );

    // convert_touch_phase: 全部变体两两不同
    let phases = [
        (winit::event::TouchPhase::Started, TouchPhase::Started),
        (winit::event::TouchPhase::Moved, TouchPhase::Moved),
        (winit::event::TouchPhase::Ended, TouchPhase::Ended),
        (winit::event::TouchPhase::Cancelled, TouchPhase::Cancelled),
    ];
    for (winit_phase, expected) in &phases {
        assert_eq!(convert_touch_phase(*winit_phase), *expected);
    }

    // element_state_to_pressed: 双重确认对称性
    assert!(element_state_to_pressed(winit::event::ElementState::Pressed));
    assert!(!element_state_to_pressed(winit::event::ElementState::Released));
}

// ── 新增覆盖率测试 ──

/// 验证：Event::Touch with multiple touch points (phase: Started/Moved/Ended/Cancelled)
#[test]
fn test_touch_event_multiple_touch_points() {
    // 创建多个触摸点，每个都有不同的阶段
    let touch1 = TouchEvent {
        id: 0,
        phase: TouchPhase::Started,
        x: 100.0,
        y: 200.0,
    };
    let touch2 = TouchEvent {
        id: 1,
        phase: TouchPhase::Moved,
        x: 150.0,
        y: 250.0,
    };
    let touch3 = TouchEvent {
        id: 2,
        phase: TouchPhase::Ended,
        x: 200.0,
        y: 300.0,
    };
    let touch4 = TouchEvent {
        id: 3,
        phase: TouchPhase::Cancelled,
        x: 250.0,
        y: 350.0,
    };

    // 验证每个触摸点的数据
    assert_eq!(touch1.phase, TouchPhase::Started);
    assert_eq!(touch2.phase, TouchPhase::Moved);
    assert_eq!(touch3.phase, TouchPhase::Ended);
    assert_eq!(touch4.phase, TouchPhase::Cancelled);

    // 验证坐标
    assert!((touch1.x - 100.0).abs() < f64::EPSILON);
    assert!((touch1.y - 200.0).abs() < f64::EPSILON);
    assert!((touch2.x - 150.0).abs() < f64::EPSILON);
    assert!((touch2.y - 250.0).abs() < f64::EPSILON);

    // 验证 AppEvent::Touch 包装
    let app_touch1 = AppEvent::Touch(touch1);
    let app_touch2 = AppEvent::Touch(touch2);
    let app_touch3 = AppEvent::Touch(touch3);
    let app_touch4 = AppEvent::Touch(touch4);

    // 解包并验证
    match (app_touch1, app_touch2, app_touch3, app_touch4) {
        (AppEvent::Touch(t1), AppEvent::Touch(t2), AppEvent::Touch(t3), AppEvent::Touch(t4)) => {
            assert_eq!(t1.phase, TouchPhase::Started);
            assert_eq!(t2.phase, TouchPhase::Moved);
            assert_eq!(t3.phase, TouchPhase::Ended);
            assert_eq!(t4.phase, TouchPhase::Cancelled);
        }
        _ => panic!("Expected all to be Touch events"),
    }
}

/// 验证：AppEvent::ScaleChanged with various scale factors
#[test]
fn test_scale_changed_event() {
    let scales: Vec<f64> = vec![
        1.0,  // 正常比例
        2.0,  // Retina 屏幕比例
        0.5,  // 缩小
        1.25, // 1.25x 比例
        1.75, // 1.75x 比例
    ];

    for scale in scales {
        let event = AppEvent::ScaleFactorChanged { scale_factor: scale };
        if let AppEvent::ScaleFactorChanged { scale_factor } = event {
            assert!(
                (scale_factor - scale).abs() < f64::EPSILON,
                "Scale factor should match: expected {}, got {}",
                scale,
                scale_factor
            );
        } else {
            panic!("Expected ScaleFactorChanged event");
        }
    }
}

/// 验证：Event::Ime with Preedit/Commit/Disconnected variants
#[test]
fn test_ime_event_full_range() {
    // 测试 IME 的所有变体
    let enabled = AppEvent::Ime(ImeEvent::Enabled);
    let preedit = AppEvent::Ime(ImeEvent::Preedit {
        text: "正在输入...".to_string(),
        cursor: Some((0, 5)),
    });
    let commit = AppEvent::Ime(ImeEvent::Commit("输入完成".to_string()));
    let disabled = AppEvent::Ime(ImeEvent::Disabled);

    // 验证每个变体
    match enabled {
        AppEvent::Ime(ImeEvent::Enabled) => (),
        _ => panic!("Expected Enabled IME event"),
    }

    match preedit {
        AppEvent::Ime(ImeEvent::Preedit { text, cursor }) => {
            assert_eq!(text, "正在输入...");
            assert_eq!(cursor, Some((0, 5)));
        }
        _ => panic!("Expected Preedit IME event"),
    }

    match commit {
        AppEvent::Ime(ImeEvent::Commit(text)) => {
            assert_eq!(text, "输入完成");
        }
        _ => panic!("Expected Commit IME event"),
    }

    match disabled {
        AppEvent::Ime(ImeEvent::Disabled) => (),
        _ => panic!("Expected Disabled IME event"),
    }
}

/// 验证：Event::Destroyed handling
#[test]
fn test_destroyed_event() {
    // 由于当前代码中没有 AppEvent::Destroyed，这里测试其他事件的生命周期
    // 确保 AppEvent 的所有变体都能正确处理
    let events = vec![
        AppEvent::RedrawRequested,
        AppEvent::CloseRequested,
        AppEvent::Focused,
        AppEvent::Unfocused,
    ];

    for event in events {
        // 验证 Debug 格式化不会 panic
        let debug_str = format!("{:?}", event);
        assert!(!debug_str.is_empty(), "Debug format should not be empty");
    }
}

/// 验证：Event::MouseEnter/Leave with coordinates (basic)
#[test]
fn test_mouse_enter_leave_coordinates_basic() {
    // 当前使用 AppEvent::MouseMoved 来表示进入和离开事件
    let mouse_enter = AppEvent::MouseMoved { x: 0.0, y: 0.0 };
    let mouse_leave = AppEvent::MouseMoved { x: 1920.0, y: 1080.0 };

    // 验证进入事件坐标
    match mouse_enter {
        AppEvent::MouseMoved { x, y } => {
            assert!((x - 0.0).abs() < f64::EPSILON);
            assert!((y - 0.0).abs() < f64::EPSILON);
        }
        _ => panic!("Expected MouseMoved event for enter"),
    }

    // 验证离开事件坐标
    match mouse_leave {
        AppEvent::MouseMoved { x, y } => {
            assert!((x - 1920.0).abs() < f64::EPSILON);
            assert!((y - 1080.0).abs() < f64::EPSILON);
        }
        _ => panic!("Expected MouseMoved event for leave"),
    }
}

/// 验证：Event::Scroll with both line and pixel delta
#[test]
fn test_scroll_delta_both_types() {
    // 测试像素滚动
    let pixel_scroll = AppEvent::MouseWheel {
        delta: MouseScrollDelta::PixelDelta(10.0, -5.0),
    };
    match pixel_scroll {
        AppEvent::MouseWheel {
            delta: MouseScrollDelta::PixelDelta(x, y),
        } => {
            assert!((x - 10.0).abs() < f64::EPSILON);
            assert!((y - (-5.0)).abs() < f64::EPSILON);
        }
        _ => panic!("Expected PixelDelta scroll"),
    }

    // 测试行滚动
    let line_scroll = AppEvent::MouseWheel {
        delta: MouseScrollDelta::LineDelta(2.0, -1.0),
    };
    match line_scroll {
        AppEvent::MouseWheel {
            delta: MouseScrollDelta::LineDelta(x, y),
        } => {
            assert!((x - 2.0f32).abs() < f32::EPSILON);
            assert!((y - (-1.0f32)).abs() < f32::EPSILON);
        }
        _ => panic!("Expected LineDelta scroll"),
    }
}

/// 验证：Converting events from winit events (MockEvent conversion paths)
#[test]
fn test_winit_event_conversions() {
    // 测试从 winit 事件到 AppEvent 的完整转换路径

    // Mock 事件 - 由于无法直接构造 winit::event::WindowEvent，
    // 这里测试转换函数的逻辑

    // 测试 element_state_to_pressed
    let pressed = element_state_to_pressed(winit::event::ElementState::Pressed);
    assert!(pressed);

    let released = element_state_to_pressed(winit::event::ElementState::Released);
    assert!(!released);

    // 测试 convert_mouse_button
    let all_buttons = [
        winit::event::MouseButton::Left,
        winit::event::MouseButton::Right,
        winit::event::MouseButton::Middle,
        winit::event::MouseButton::Back,
        winit::event::MouseButton::Forward,
        winit::event::MouseButton::Other(42),
    ];

    for winit_btn in all_buttons {
        let converted = convert_mouse_button(winit_btn);
        // 验证转换不会 panic
        match winit_btn {
            winit::event::MouseButton::Left => assert_eq!(converted, MouseButton::Left),
            winit::event::MouseButton::Right => assert_eq!(converted, MouseButton::Right),
            winit::event::MouseButton::Middle => assert_eq!(converted, MouseButton::Middle),
            winit::event::MouseButton::Back => assert_eq!(converted, MouseButton::Back),
            winit::event::MouseButton::Forward => assert_eq!(converted, MouseButton::Forward),
            winit::event::MouseButton::Other(n) => assert_eq!(converted, MouseButton::Other(n)),
        }
    }

    // 测试 convert_scroll_delta
    let pixel_delta = winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(5.0, -3.0));
    let converted_pixel = convert_scroll_delta(pixel_delta);
    assert_eq!(converted_pixel, MouseScrollDelta::PixelDelta(5.0, -3.0));

    let line_delta = winit::event::MouseScrollDelta::LineDelta(1.0, -0.5);
    let converted_line = convert_scroll_delta(line_delta);
    assert_eq!(converted_line, MouseScrollDelta::LineDelta(1.0, -0.5));

    // 测试 convert_ime
    let ime_enabled = convert_ime(winit::event::Ime::Enabled);
    assert_eq!(ime_enabled, ImeEvent::Enabled);

    let ime_preedit = convert_ime(winit::event::Ime::Preedit("test".to_string(), None));
    assert_eq!(
        ime_preedit,
        ImeEvent::Preedit {
            text: "test".to_string(),
            cursor: None,
        }
    );

    let ime_commit = convert_ime(winit::event::Ime::Commit("commit".to_string()));
    assert_eq!(ime_commit, ImeEvent::Commit("commit".to_string()));

    let ime_disabled = convert_ime(winit::event::Ime::Disabled);
    assert_eq!(ime_disabled, ImeEvent::Disabled);

    // 测试 convert_touch_phase
    let all_touch_phases = [
        winit::event::TouchPhase::Started,
        winit::event::TouchPhase::Moved,
        winit::event::TouchPhase::Ended,
        winit::event::TouchPhase::Cancelled,
    ];

    let expected_phases = [
        TouchPhase::Started,
        TouchPhase::Moved,
        TouchPhase::Ended,
        TouchPhase::Cancelled,
    ];

    for (winit_phase, expected) in all_touch_phases.iter().zip(expected_phases.iter()) {
        let converted = convert_touch_phase(*winit_phase);
        assert_eq!(converted, *expected);
    }
}

// ── 新增边界测试（第二轮） ──

/// 测试 MouseButton 枚举比较语义（补充）。
#[test]
fn test_mouse_button_equality_comprehensive() {
    assert_eq!(MouseButton::Left, MouseButton::Left);
    assert_eq!(MouseButton::Other(5), MouseButton::Other(5));
    assert_ne!(MouseButton::Other(5), MouseButton::Other(6));
    assert_ne!(MouseButton::Left, MouseButton::Right);
}

/// 测试 TouchPhase 枚举比较语义。
#[test]
fn test_touch_phase_ordering() {
    assert_eq!(TouchPhase::Started, TouchPhase::Started);
    assert_ne!(TouchPhase::Started, TouchPhase::Ended);
}

/// 测试 ImeEvent Preedit 带光标。
#[test]
fn test_ime_preedit_with_cursor() {
    let event = ImeEvent::Preedit {
        text: "你好".to_string(),
        cursor: Some((0, 2)),
    };
    match event {
        ImeEvent::Preedit { text, cursor } => {
            assert_eq!(text, "你好");
            assert_eq!(cursor, Some((0, 2)));
        }
        _ => panic!("期望 Preedit"),
    }
}

/// 测试 MouseScrollDelta PixelDelta 极端负值。
#[test]
fn test_scroll_delta_negative() {
    let delta = MouseScrollDelta::PixelDelta(-1000.0, -2000.0);
    match delta {
        MouseScrollDelta::PixelDelta(x, y) => {
            assert_eq!(x, -1000.0);
            assert_eq!(y, -2000.0);
        }
        _ => panic!("期望 PixelDelta"),
    }
}

/// 测试 AppEvent::Touch 包含完整 TouchEvent 数据。
#[test]
fn test_touch_event_data() {
    let touch = TouchEvent {
        id: 42,
        phase: TouchPhase::Moved,
        x: 123.45,
        y: 678.90,
    };
    assert_eq!(touch.id, 42);
    assert_eq!(touch.phase, TouchPhase::Moved);
    assert!((touch.x - 123.45).abs() < 0.01);
    assert!((touch.y - 678.90).abs() < 0.01);
}

/// 测试 AppEvent 部分变体 Debug 格式化不 panic。
#[test]
fn test_app_event_more_debug_formats() {
    let events = vec![
        AppEvent::MouseMoved { x: 0.0, y: 0.0 },
        AppEvent::MouseInput {
            button: MouseButton::Middle,
            pressed: true,
        },
        AppEvent::MouseWheel {
            delta: MouseScrollDelta::LineDelta(1.0, 0.0),
        },
        AppEvent::Touch(TouchEvent {
            id: 0,
            phase: TouchPhase::Started,
            x: 0.0,
            y: 0.0,
        }),
        AppEvent::Ime(ImeEvent::Disabled),
    ];
    for event in &events {
        let debug = format!("{event:?}");
        assert!(!debug.is_empty(), "Debug 格式不应为空");
    }
    for event in &events {
        let debug = format!("{event:?}");
        assert!(!debug.is_empty(), "Debug 格式不应为空");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 额外转换函数覆盖率测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 element_state_to_pressed
#[test]
fn test_element_state_to_pressed_values() {
    assert!(element_state_to_pressed(winit::event::ElementState::Pressed));
    assert!(!element_state_to_pressed(winit::event::ElementState::Released));
}

/// 测试 convert_mouse_button — Other 变体
#[test]
fn test_convert_mouse_button_other() {
    let btn = convert_mouse_button(winit::event::MouseButton::Other(42));
    assert_eq!(btn, MouseButton::Other(42));
}

/// 测试 convert_scroll_delta — PixelDelta 边界值
#[test]
fn test_convert_scroll_delta_pixel_boundary() {
    let delta = convert_scroll_delta(winit::event::MouseScrollDelta::PixelDelta(
        winit::dpi::PhysicalPosition::new(f64::MAX, f64::MIN),
    ));
    if let MouseScrollDelta::PixelDelta(x, y) = delta {
        assert_eq!(x, f64::MAX);
        assert_eq!(y, f64::MIN);
    } else {
        panic!("Expected PixelDelta");
    }
}

/// 测试 convert_touch_phase — Cancelled
#[test]
fn test_convert_touch_phase_cancelled() {
    let phase = convert_touch_phase(winit::event::TouchPhase::Cancelled);
    assert_eq!(phase, TouchPhase::Cancelled);
}

/// 测试 convert_ime — Preedit with cursor
#[test]
fn test_convert_ime_preedit_with_cursor() {
    let ime = convert_ime(winit::event::Ime::Preedit("测试".to_string(), Some((0, 2))));
    if let ImeEvent::Preedit { text, cursor } = ime {
        assert_eq!(text, "测试");
        assert_eq!(cursor, Some((0, 2)));
    } else {
        panic!("Expected Preedit");
    }
}

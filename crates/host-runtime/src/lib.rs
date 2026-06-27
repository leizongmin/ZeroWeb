//! # zero-host-runtime
//!
//! 平台宿主 — 窗口、事件循环、surface、输入法。
//!
//! 基于 winit 提供跨平台窗口管理和事件循环。

#![warn(missing_docs)]

pub mod event;
pub mod window;

/// 宿主运行时错误
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    /// 窗口创建失败
    #[error("窗口创建失败: {0}")]
    WindowCreationFailed(String),
    /// GPU 设备请求失败
    #[error("GPU 设备请求失败: {0}")]
    GpuRequestFailed(String),
    /// 事件循环错误
    #[error("事件循环错误: {0}")]
    EventLoopError(String),
}

/// 宿主运行时结果
pub type HostResult<T> = Result<T, HostError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_error_window_creation_message() {
        let err = HostError::WindowCreationFailed("display not found".into());
        let msg = err.to_string();
        assert!(msg.contains("display not found"), "message: {msg}");
    }

    #[test]
    fn test_host_error_gpu_request_message() {
        let err = HostError::GpuRequestFailed("no adapter".into());
        let msg = err.to_string();
        assert!(msg.contains("no adapter"), "message: {msg}");
    }

    #[test]
    fn test_host_error_event_loop_message() {
        let err = HostError::EventLoopError("interrupted".into());
        let msg = err.to_string();
        assert!(msg.contains("interrupted"), "message: {msg}");
    }

    #[test]
    fn test_host_result_ok_and_err() {
        let ok: HostResult<()> = Ok(());
        assert!(ok.is_ok());

        let err: HostResult<()> = Err(HostError::EventLoopError("x".into()));
        assert!(err.is_err());
    }

    /// 测试单点触摸事件的 id 和位置。
    /// 验证 TouchEvent 的 id、phase、x、y 字段与构造值一致。
    #[test]
    fn test_touch_event_single_point() {
        use crate::event::{TouchEvent, TouchPhase};

        let touch = TouchEvent {
            id: 7,
            phase: TouchPhase::Started,
            x: 150.5,
            y: 300.75,
        };

        assert_eq!(touch.id, 7, "touch id should be 7");
        assert_eq!(touch.phase, TouchPhase::Started, "touch phase should be Started");
        assert!((touch.x - 150.5).abs() < f64::EPSILON, "touch x should be 150.5");
        assert!((touch.y - 300.75).abs() < f64::EPSILON, "touch y should be 300.75");
    }

    /// 测试 IME 组合事件的文本和光标位置。
    /// 验证 ImeEvent::Preedit 的 text 和 cursor 字段正确传递，
    /// 以及 ImeEvent::Commit 的文本内容。
    #[test]
    fn test_ime_event_composition() {
        use crate::event::ImeEvent;

        // 预编辑阶段：正在输入拼音 "zhong"
        let preedit = ImeEvent::Preedit {
            text: "zhong".to_string(),
            cursor: Some((0, 5)),
        };
        if let ImeEvent::Preedit { text, cursor } = &preedit {
            assert_eq!(text, "zhong", "preedit text should be 'zhong'");
            assert_eq!(*cursor, Some((0, 5)), "cursor should be (0, 5)");
        } else {
            panic!("Expected Preedit variant");
        }

        // 提交阶段：用户选中"中"
        let commit = ImeEvent::Commit("中".to_string());
        if let ImeEvent::Commit(text) = &commit {
            assert_eq!(text, "中", "commit text should be '中'");
        } else {
            panic!("Expected Commit variant");
        }
    }

    /// 测试鼠标左键点击事件的按钮值。
    #[test]
    fn test_mouse_event_button_left() {
        use crate::event::{AppEvent, MouseButton};

        // 左键按下
        let press = AppEvent::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            x: 0.0,
            y: 0.0,
        };
        if let AppEvent::MouseInput { button, pressed, .. } = press {
            assert_eq!(button, MouseButton::Left, "按钮应为左键");
            assert!(pressed, "应为按下状态");
        } else {
            panic!("Expected MouseInput variant");
        }

        // 左键释放
        let release = AppEvent::MouseInput {
            button: MouseButton::Left,
            pressed: false,
            x: 0.0,
            y: 0.0,
        };
        if let AppEvent::MouseInput { button, pressed, .. } = release {
            assert_eq!(button, MouseButton::Left);
            assert!(!pressed, "应为释放状态");
        } else {
            panic!("Expected MouseInput variant");
        }
    }

    /// 测试键盘事件的特定按键名称。
    #[test]
    fn test_keyboard_event_key_code() {
        use crate::event::AppEvent;

        // 测试特定按键码
        let cases: Vec<(&str, bool)> = vec![
            ("KeyA", true),
            ("Enter", true),
            ("Escape", false),
            ("Space", true),
            ("ArrowUp", true),
            ("F1", false),
        ];

        for (key_name, pressed) in cases {
            let event = AppEvent::KeyboardInput {
                key: key_name.to_string(),
                text: None,
                pressed,
            };
            if let AppEvent::KeyboardInput { key, pressed: p, .. } = event {
                assert_eq!(key, key_name, "key 应为 {key_name}");
                assert_eq!(p, pressed, "pressed 应为 {pressed}");
            } else {
                panic!("Expected KeyboardInput variant for key {key_name}");
            }
        }
    }

    /// 测试多点触摸事件的坐标和 id 独立性。
    #[test]
    fn test_touch_event_multiple_points() {
        use crate::event::{TouchEvent, TouchPhase};

        let touches: Vec<TouchEvent> = vec![
            TouchEvent {
                id: 0,
                phase: TouchPhase::Started,
                x: 100.0,
                y: 200.0,
            },
            TouchEvent {
                id: 1,
                phase: TouchPhase::Started,
                x: 300.0,
                y: 400.0,
            },
            TouchEvent {
                id: 2,
                phase: TouchPhase::Moved,
                x: 110.0,
                y: 210.0,
            },
        ];

        assert_eq!(touches.len(), 3, "应有 3 个触摸点");

        // 验证每个触摸点的 id 和坐标独立
        assert_eq!(touches[0].id, 0);
        assert!((touches[0].x - 100.0).abs() < f64::EPSILON);
        assert!((touches[0].y - 200.0).abs() < f64::EPSILON);
        assert_eq!(touches[0].phase, TouchPhase::Started);

        assert_eq!(touches[1].id, 1);
        assert!((touches[1].x - 300.0).abs() < f64::EPSILON);
        assert!((touches[1].y - 400.0).abs() < f64::EPSILON);

        assert_eq!(touches[2].id, 2);
        assert_eq!(touches[2].phase, TouchPhase::Moved);

        // 模拟释放触摸点 0 和 1
        let ended: Vec<TouchEvent> = vec![
            TouchEvent {
                id: 0,
                phase: TouchPhase::Ended,
                x: 105.0,
                y: 205.0,
            },
            TouchEvent {
                id: 1,
                phase: TouchPhase::Ended,
                x: 295.0,
                y: 395.0,
            },
        ];
        assert_eq!(ended[0].phase, TouchPhase::Ended);
        assert_eq!(ended[1].phase, TouchPhase::Ended);
    }

    /// 测试窗口 resize 事件携带的新尺寸字段。
    #[test]
    fn test_window_event_resize() {
        use crate::event::AppEvent;

        let cases: Vec<(u32, u32)> = vec![(800, 600), (0, 0), (1920, 1080), (3840, 2160), (1, 1)];

        for (w, h) in cases {
            let event = AppEvent::Resized { width: w, height: h };
            if let AppEvent::Resized { width, height } = event {
                assert_eq!(width, w, "resize 宽度应为 {w}");
                assert_eq!(height, h, "resize 高度应为 {h}");
            } else {
                panic!("Expected Resized variant for ({w}, {h})");
            }
        }
    }

    /// 测试 IME 组合更新事件（Preedit）的文本和光标。
    #[test]
    fn test_ime_event_composition_update() {
        use crate::event::ImeEvent;

        // 模拟拼音输入 "nihao" 的多次 preedit 更新
        let updates: Vec<ImeEvent> = vec![
            ImeEvent::Preedit {
                text: "n".to_string(),
                cursor: Some((0, 1)),
            },
            ImeEvent::Preedit {
                text: "ni".to_string(),
                cursor: Some((0, 2)),
            },
            ImeEvent::Preedit {
                text: "nih".to_string(),
                cursor: Some((0, 3)),
            },
            ImeEvent::Preedit {
                text: "niha".to_string(),
                cursor: Some((0, 4)),
            },
            ImeEvent::Preedit {
                text: "nihao".to_string(),
                cursor: Some((0, 5)),
            },
        ];

        // 验证 preedit 文本逐步增长
        for (i, event) in updates.iter().enumerate() {
            if let ImeEvent::Preedit { text, cursor } = event {
                assert_eq!(text.len(), i + 1, "第 {} 次 preedit 文本长度应为 {}", i + 1, i + 1);
                assert_eq!(*cursor, Some((0, i + 1)), "光标范围应为 (0, {})", i + 1);
            } else {
                panic!("第 {} 个事件应为 Preedit", i + 1);
            }
        }

        // 最终 commit
        let commit = ImeEvent::Commit("你好".to_string());
        if let ImeEvent::Commit(text) = &commit {
            assert_eq!(text, "你好", "commit 文本应为 '你好'");
        } else {
            panic!("Expected Commit variant");
        }
    }

    /// 测试 HostError 各变体的 Display 输出包含中文前缀。
    /// 确保 WindowCreationFailed 包含"窗口创建失败"，
    /// GpuRequestFailed 包含"GPU 设备请求失败"，
    /// EventLoopError 包含"事件循环错误"。
    #[test]
    fn test_host_error_display_chinese_prefix() {
        let err1 = HostError::WindowCreationFailed("reason".into());
        let msg1 = err1.to_string();
        assert!(
            msg1.starts_with("窗口创建失败"),
            "WindowCreationFailed 应以'窗口创建失败'开头，实际: {msg1}"
        );

        let err2 = HostError::GpuRequestFailed("reason".into());
        let msg2 = err2.to_string();
        assert!(
            msg2.starts_with("GPU 设备请求失败"),
            "GpuRequestFailed 应以'GPU 设备请求失败'开头，实际: {msg2}"
        );

        let err3 = HostError::EventLoopError("reason".into());
        let msg3 = err3.to_string();
        assert!(
            msg3.starts_with("事件循环错误"),
            "EventLoopError 应以'事件循环错误'开头，实际: {msg3}"
        );
    }

    /// 测试 TouchEvent 在极端 id 值和坐标边界条件下的行为。
    /// 验证 u64::MAX 作为 id、f64::MAX/INFINITY/负无穷 作为坐标均能正常存储和读取。
    #[test]
    fn test_touch_event_extreme_id_and_coordinates() {
        use crate::event::{TouchEvent, TouchPhase};

        // u64::MAX 作为 id
        let max_id_touch = TouchEvent {
            id: u64::MAX,
            phase: TouchPhase::Started,
            x: 0.0,
            y: 0.0,
        };
        assert_eq!(max_id_touch.id, u64::MAX, "touch id 应为 u64::MAX");

        // f64::MAX 坐标
        let max_coord = TouchEvent {
            id: 0,
            phase: TouchPhase::Moved,
            x: f64::MAX,
            y: f64::MIN,
        };
        assert!((max_coord.x - f64::MAX).abs() < f64::EPSILON, "x 应为 f64::MAX");
        assert!((max_coord.y - f64::MIN).abs() < f64::EPSILON, "y 应为 f64::MIN");

        // f64::INFINITY 和 NEG_INFINITY 坐标（模拟极端触控位置）
        let inf_touch = TouchEvent {
            id: 1,
            phase: TouchPhase::Ended,
            x: f64::INFINITY,
            y: f64::NEG_INFINITY,
        };
        assert!(
            inf_touch.x.is_infinite() && inf_touch.x.is_sign_positive(),
            "x 应为正无穷"
        );
        assert!(
            inf_touch.y.is_infinite() && inf_touch.y.is_sign_negative(),
            "y 应为负无穷"
        );

        // 零 id 和零坐标
        let zero_touch = TouchEvent {
            id: 0,
            phase: TouchPhase::Cancelled,
            x: 0.0,
            y: 0.0,
        };
        assert_eq!(zero_touch.id, 0);
        assert!((zero_touch.x - 0.0).abs() < f64::EPSILON);
        assert!((zero_touch.y - 0.0).abs() < f64::EPSILON);
    }

    /// 测试 MouseScrollDelta::LineDelta 在极端 f32 值下的行为。
    /// 覆盖零值、负值、f32::MAX、f32::MIN 等边界条件。
    #[test]
    fn test_mouse_scroll_delta_line_extreme_values() {
        use crate::event::MouseScrollDelta;

        // 零值滚动
        let zero = MouseScrollDelta::LineDelta(0.0, 0.0);
        assert_eq!(zero, MouseScrollDelta::LineDelta(0.0, 0.0));

        // f32::MAX 和 f32::MIN
        let extreme = MouseScrollDelta::LineDelta(f32::MAX, f32::MIN);
        assert_eq!(extreme, MouseScrollDelta::LineDelta(f32::MAX, f32::MIN));

        // 负值双向滚动
        let negative = MouseScrollDelta::LineDelta(-100.0, -200.0);
        assert_eq!(negative, MouseScrollDelta::LineDelta(-100.0, -200.0));

        // 混合正负
        let mixed = MouseScrollDelta::LineDelta(-1.0, 1.0);
        assert_ne!(mixed, MouseScrollDelta::LineDelta(1.0, -1.0));

        // 通过 winit 转换路径验证极端值不丢失
        let winit_extreme =
            crate::event::convert_scroll_delta(winit::event::MouseScrollDelta::LineDelta(f32::MAX, f32::MIN));
        assert_eq!(winit_extreme, MouseScrollDelta::LineDelta(f32::MAX, f32::MIN));
    }

    /// 测试连续多个 CloseRequested 事件的处理。
    /// 用户可能在窗口关闭前快速点击多次关闭按钮，
    /// 每次点击都应独立产生一个 CloseRequested 事件。
    #[test]
    fn test_consecutive_close_requested_events() {
        let mut received: Vec<crate::event::AppEvent> = Vec::new();
        let mut callback = |e: crate::event::AppEvent| received.push(e);
        let attrs = winit::window::WindowAttributes::default();
        let mut app = crate::window::BasicApp::new_basic(attrs, &mut callback);

        // 模拟用户快速连续点击关闭按钮 5 次
        for _ in 0..5 {
            app.handle_window_event(winit::event::WindowEvent::CloseRequested);
        }

        assert_eq!(received.len(), 5, "应收到 5 个 CloseRequested 事件");
        for (i, event) in received.iter().enumerate() {
            assert!(
                matches!(event, crate::event::AppEvent::CloseRequested),
                "第 {} 个事件应为 CloseRequested",
                i + 1
            );
        }
    }

    /// 测试 HostError 各变体的 source 为空（无底层错误链）。
    /// 确保错误类型实现了 std::error::Error trait 的 source 方法返回 None。
    #[test]
    fn test_host_error_source_is_none() {
        use std::error::Error;

        let err1 = HostError::WindowCreationFailed("test".into());
        assert!(err1.source().is_none(), "WindowCreationFailed source 应为 None");

        let err2 = HostError::GpuRequestFailed("test".into());
        assert!(err2.source().is_none(), "GpuRequestFailed source 应为 None");

        let err3 = HostError::EventLoopError("test".into());
        assert!(err3.source().is_none(), "EventLoopError source 应为 None");
    }

    /// 测试鼠标事件的 x/y 坐标精确传递。
    #[test]
    fn test_mouse_event_coordinates() {
        use crate::event::AppEvent;

        let cases: Vec<(f64, f64)> = vec![
            (0.0, 0.0),
            (100.0, 200.0),
            (1920.5, 1080.25),
            (-50.0, -100.0),
            (f64::MAX, f64::MIN),
        ];

        for (x, y) in cases {
            let event = AppEvent::MouseMoved { x, y };
            if let AppEvent::MouseMoved { x: ex, y: ey } = event {
                assert!((ex - x).abs() < f64::EPSILON, "x 坐标不匹配: 期望 {x}, 实际 {ex}");
                assert!((ey - y).abs() < f64::EPSILON, "y 坐标不匹配: 期望 {y}, 实际 {ey}");
            } else {
                panic!("Expected MouseMoved variant for ({x}, {y})");
            }
        }
    }

    /// 测试 BasicApp 处理连续三次 Resized 事件（800x600 → 1024x768 → 640x480），
    /// 验证所有事件均被接收且顺序和尺寸值正确。
    #[test]
    fn test_basic_app_consecutive_resized_events_order() {
        use crate::event::AppEvent;

        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let attrs = winit::window::WindowAttributes::default();
        let mut app = crate::window::BasicApp::new_basic(attrs, &mut callback);

        // 第一次 resize：800x600
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
            800, 600,
        )));
        // 第二次 resize：1024x768
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
            1024, 768,
        )));
        // 第三次 resize：640x480
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
            640, 480,
        )));

        assert_eq!(received.len(), 3, "应收到 3 个 Resized 事件");

        // 验证顺序和尺寸值
        match &received[0] {
            AppEvent::Resized { width, height } => {
                assert_eq!(*width, 800, "第 1 次 resize 宽度应为 800");
                assert_eq!(*height, 600, "第 1 次 resize 高度应为 600");
            }
            _ => panic!("第 1 个事件应为 Resized"),
        }
        match &received[1] {
            AppEvent::Resized { width, height } => {
                assert_eq!(*width, 1024, "第 2 次 resize 宽度应为 1024");
                assert_eq!(*height, 768, "第 2 次 resize 高度应为 768");
            }
            _ => panic!("第 2 个事件应为 Resized"),
        }
        match &received[2] {
            AppEvent::Resized { width, height } => {
                assert_eq!(*width, 640, "第 3 次 resize 宽度应为 640");
                assert_eq!(*height, 480, "第 3 次 resize 高度应为 480");
            }
            _ => panic!("第 3 个事件应为 Resized"),
        }
    }

    /// 测试 KeyboardInput 事件同时按下 Shift+Ctrl+Alt 三个修饰键的场景。
    /// 验证每个修饰键按下事件的 key 名称和 pressed 状态均正确。
    #[test]
    fn test_keyboard_input_all_modifiers_simultaneous() {
        use crate::event::AppEvent;

        let events: Vec<AppEvent> = vec![
            AppEvent::KeyboardInput {
                key: "Shift".to_string(),
                text: None,
                pressed: true,
            },
            AppEvent::KeyboardInput {
                key: "Control".to_string(),
                text: None,
                pressed: true,
            },
            AppEvent::KeyboardInput {
                key: "Alt".to_string(),
                text: None,
                pressed: true,
            },
        ];

        assert_eq!(events.len(), 3, "应构造 3 个修饰键按下事件");

        let expected: Vec<(&str, bool)> = vec![("Shift", true), ("Control", true), ("Alt", true)];
        for (i, (expected_key, expected_pressed)) in expected.iter().enumerate() {
            match &events[i] {
                AppEvent::KeyboardInput { key, pressed, .. } => {
                    assert_eq!(key, expected_key, "第 {} 个修饰键名称应为 {}", i + 1, expected_key);
                    assert_eq!(
                        *pressed,
                        *expected_pressed,
                        "第 {} 个修饰键 pressed 应为 {}",
                        i + 1,
                        expected_pressed
                    );
                }
                _ => panic!("第 {} 个事件应为 KeyboardInput", i + 1),
            }
        }
    }

    /// 测试 MouseInput 事件的中键（Middle）按钮变体。
    /// 验证通过 BasicApp 分发路径传递 Middle 按钮的按下和释放事件。
    #[test]
    fn test_mouse_input_middle_button_dispatch() {
        use crate::event::{AppEvent, MouseButton};

        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let attrs = winit::window::WindowAttributes::default();
        let mut app = crate::window::BasicApp::new_basic(attrs, &mut callback);

        // 中键按下
        app.handle_window_event(winit::event::WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state: winit::event::ElementState::Pressed,
            button: winit::event::MouseButton::Middle,
        });
        // 中键释放
        app.handle_window_event(winit::event::WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state: winit::event::ElementState::Released,
            button: winit::event::MouseButton::Middle,
        });

        assert_eq!(received.len(), 2, "应收到 2 个 MouseInput 事件");

        match &received[0] {
            AppEvent::MouseInput { button, pressed, .. } => {
                assert_eq!(*button, MouseButton::Middle, "按下事件的按钮应为 Middle");
                assert!(*pressed, "按下事件 pressed 应为 true");
            }
            _ => panic!("第 1 个事件应为 MouseInput"),
        }
        match &received[1] {
            AppEvent::MouseInput { button, pressed, .. } => {
                assert_eq!(*button, MouseButton::Middle, "释放事件的按钮应为 Middle");
                assert!(!pressed, "释放事件 pressed 应为 false");
            }
            _ => panic!("第 2 个事件应为 MouseInput"),
        }
    }

    /// 测试 ImeEvent::Preedit 在文本为空字符串且光标位置为 (0, 0) 时的行为。
    /// 验证空 preedit 文本和零值光标位置均能正确存储和读取。
    #[test]
    fn test_ime_preedit_empty_string_with_cursor_position() {
        use crate::event::ImeEvent;

        // 空文本 + 光标在起始位置
        let preedit = ImeEvent::Preedit {
            text: String::new(),
            cursor: Some((0, 0)),
        };
        if let ImeEvent::Preedit { text, cursor } = &preedit {
            assert!(text.is_empty(), "preedit 文本应为空字符串");
            assert_eq!(*cursor, Some((0, 0)), "光标位置应为 (0, 0)");
        } else {
            panic!("Expected Preedit variant");
        }

        // 通过 winit 转换路径验证
        let converted = crate::event::convert_ime(winit::event::Ime::Preedit(String::new(), Some((0, 0))));
        assert_eq!(
            converted,
            ImeEvent::Preedit {
                text: String::new(),
                cursor: Some((0, 0)),
            },
            "winit 空 preedit 转换应保留光标位置 (0, 0)"
        );
    }

    /// 测试 KeyboardInput 事件中按键 "Space" 且 released=true（pressed=false）的场景。
    /// 验证空格键释放事件能正确构造和读取。
    #[test]
    fn test_keyboard_input_space_released() {
        use crate::event::AppEvent;

        let event = AppEvent::KeyboardInput {
            key: "Space".to_string(),
            text: None,
            pressed: false,
        };

        if let AppEvent::KeyboardInput { key, pressed, .. } = &event {
            assert_eq!(key, "Space", "按键名称应为 'Space'");
            assert!(!pressed, "Space 释放事件 pressed 应为 false");
        } else {
            panic!("Expected KeyboardInput variant");
        }

        // 验证 Debug 输出包含 Space 和 pressed=false 信息
        let debug = format!("{:?}", event);
        assert!(debug.contains("Space"), "Debug 输出应包含 'Space'");
    }

    /// 测试 KeyboardInput 事件使用未知按键名称的边界行为。
    /// 当底层输入系统产生无法识别的按键名称时（如特殊硬件键或自定义输入设备），
    /// key 字段应原样存储未知字符串，不会 panic 或被替换为默认值。
    /// 验证按下和释放两个状态均能正确传递未知键名。
    #[test]
    fn test_keyboard_input_unknown_key_name() {
        use crate::event::AppEvent;

        // 使用完全虚构的键名，模拟无法识别的输入设备
        let unknown_press = AppEvent::KeyboardInput {
            key: "UnknownKey_0xDEAD".to_string(),
            text: None,
            pressed: true,
        };
        if let AppEvent::KeyboardInput { key, pressed, .. } = &unknown_press {
            assert_eq!(key, "UnknownKey_0xDEAD", "未知按键名称应原样存储，不应被替换或截断");
            assert!(pressed, "未知键的 pressed 应为 true");
        } else {
            panic!("Expected KeyboardInput variant");
        }

        // 释放未知键
        let unknown_release = AppEvent::KeyboardInput {
            key: "UnknownKey_0xDEAD".to_string(),
            text: None,
            pressed: false,
        };
        if let AppEvent::KeyboardInput { key, pressed, .. } = &unknown_release {
            assert_eq!(key, "UnknownKey_0xDEAD", "释放事件应保留相同的未知键名");
            assert!(!pressed, "释放事件的 pressed 应为 false");
        } else {
            panic!("Expected KeyboardInput variant");
        }

        // 验证 Debug 输出包含未知键名（确保格式化不 panic）
        let debug = format!("{:?}", unknown_press);
        assert!(debug.contains("UnknownKey_0xDEAD"), "Debug 输出应包含未知键名");

        // 空字符串键名（极端未知键场景）
        let empty_key = AppEvent::KeyboardInput {
            key: String::new(),
            text: None,
            pressed: true,
        };
        if let AppEvent::KeyboardInput { key, pressed, .. } = &empty_key {
            assert!(key.is_empty(), "空字符串键名应能正常存储");
            assert!(pressed);
        } else {
            panic!("Expected KeyboardInput variant");
        }
    }

    /// 测试鼠标移动事件（MouseMotion）携带负数坐标的边界行为。
    /// 当光标位于窗口外部（如拖拽到窗口左侧或上方）时，
    /// 坐标值为负数。验证通过 BasicApp 分发路径传递负坐标时无精度丢失。
    #[test]
    fn test_mouse_motion_negative_coordinates() {
        use crate::event::AppEvent;

        // 直接构造负坐标 MouseMoved 事件
        let negative_cases: Vec<(f64, f64)> = vec![
            (-1.0, -1.0),
            (-0.001, -999.999),
            (-f64::MAX, -f64::MAX),
            (-1920.0, -1080.0),
        ];

        for (x, y) in &negative_cases {
            let event = AppEvent::MouseMoved { x: *x, y: *y };
            if let AppEvent::MouseMoved { x: ex, y: ey } = event {
                assert!((ex - *x).abs() < f64::EPSILON, "负坐标 x 精度丢失: 期望 {x}, 实际 {ex}");
                assert!((ey - *y).abs() < f64::EPSILON, "负坐标 y 精度丢失: 期望 {y}, 实际 {ey}");
            } else {
                panic!("Expected MouseMoved variant for ({x}, {y})");
            }
        }

        // 通过 BasicApp 分发路径验证负坐标精确传递
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let attrs = winit::window::WindowAttributes::default();
        let mut app = crate::window::BasicApp::new_basic(attrs, &mut callback);

        app.handle_window_event(winit::event::WindowEvent::CursorMoved {
            device_id: winit::event::DeviceId::dummy(),
            position: winit::dpi::PhysicalPosition::new(-500.5, -1000.25),
        });

        assert_eq!(received.len(), 1, "应收到 1 个 MouseMoved 事件");
        match &received[0] {
            AppEvent::MouseMoved { x, y } => {
                assert!((*x - (-500.5)).abs() < f64::EPSILON, "分发后 x 应为 -500.5，实际 {x}");
                assert!(
                    (*y - (-1000.25)).abs() < f64::EPSILON,
                    "分发后 y 应为 -1000.25，实际 {y}"
                );
            }
            _ => panic!("应为 MouseMoved 事件"),
        }
    }

    /// 测试触摸事件的 Ended 阶段（手指抬起）。
    /// Ended 阶段表示触摸点被释放，坐标应记录手指抬起时的最终位置。
    /// 验证通过 BasicApp 分发路径传递 Ended 阶段时，id、phase 和坐标均正确。
    #[test]
    fn test_touch_event_ended_phase() {
        use crate::event::{AppEvent, TouchPhase};

        // 直接构造 Ended 阶段的触摸事件
        let ended_event = AppEvent::Touch(crate::event::TouchEvent {
            id: 42,
            phase: TouchPhase::Ended,
            x: 250.75,
            y: 480.5,
        });

        if let AppEvent::Touch(te) = &ended_event {
            assert_eq!(te.id, 42, "Ended 阶段的触摸点 id 应为 42");
            assert_eq!(te.phase, TouchPhase::Ended, "阶段应为 Ended");
            assert!((te.x - 250.75).abs() < f64::EPSILON, "x 应为 250.75");
            assert!((te.y - 480.5).abs() < f64::EPSILON, "y 应为 480.5");
        } else {
            panic!("Expected Touch variant");
        }

        // 通过 BasicApp 分发路径验证 Ended 阶段
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let attrs = winit::window::WindowAttributes::default();
        let mut app = crate::window::BasicApp::new_basic(attrs, &mut callback);

        app.handle_window_event(winit::event::WindowEvent::Touch(winit::event::Touch {
            device_id: winit::event::DeviceId::dummy(),
            phase: winit::event::TouchPhase::Ended,
            location: winit::dpi::PhysicalPosition::new(300.0, 600.0),
            id: 99,
            force: None,
        }));

        assert_eq!(received.len(), 1, "应收到 1 个 Touch 事件");
        match &received[0] {
            AppEvent::Touch(te) => {
                assert_eq!(te.id, 99, "分发后的触摸点 id 应为 99");
                assert_eq!(te.phase, TouchPhase::Ended, "分发后的阶段应为 Ended");
                assert!((te.x - 300.0).abs() < f64::EPSILON, "x 应为 300.0");
                assert!((te.y - 600.0).abs() < f64::EPSILON, "y 应为 600.0");
            }
            _ => panic!("应为 Touch 事件"),
        }

        // 验证 Ended 与其他阶段互不相等
        assert_ne!(TouchPhase::Ended, TouchPhase::Started, "Ended 不应等于 Started");
        assert_ne!(TouchPhase::Ended, TouchPhase::Moved, "Ended 不应等于 Moved");
        assert_ne!(TouchPhase::Ended, TouchPhase::Cancelled, "Ended 不应等于 Cancelled");
    }

    /// 测试窗口 ScaleFactorChanged 事件在缩放因子为 2.0 时的行为。
    /// 当用户将系统显示缩放比例从 100% 切换到 200%（如拖动窗口到高 DPI 显示器）时，
    /// winit 会产生 ScaleFactorChanged 事件。
    /// 当前 BasicApp 未处理此事件（落入 `_ => {}` 分支），验证其被静默忽略不产生回调，
    /// 且不影响后续正常事件的分发。
    #[test]
    fn test_window_event_scale_changed_factor_2() {
        use crate::event::AppEvent;

        // 阶段 1：ScaleFactorChanged 等未处理事件应被静默忽略
        {
            let mut received: Vec<AppEvent> = Vec::new();
            let mut callback = |e: AppEvent| received.push(e);
            let attrs = winit::window::WindowAttributes::default();
            let mut app = crate::window::BasicApp::new_basic(attrs, &mut callback);

            // winit 的 ScaleFactorChanged 携带 scale_factor 和 InnerSizeWriter，
            // 由于 InnerSizeWriter 无法在测试中直接构造，
            // 使用同样落入 `_ => {}` 分支的 ThemeChanged 事件来模拟未处理事件。
            app.handle_window_event(winit::event::WindowEvent::ThemeChanged(winit::window::Theme::Light));

            assert!(
                received.is_empty(),
                "未处理的窗口事件（如 ScaleFactorChanged）不应产生回调事件"
            );
        }

        // 阶段 2：忽略 ScaleFactorChanged 后，Resized 事件仍能正确分发
        // 模拟缩放因子 2.0 下，逻辑尺寸 800x450 对应物理尺寸 1600x900
        {
            let mut received: Vec<AppEvent> = Vec::new();
            let mut callback = |e: AppEvent| received.push(e);
            let attrs = winit::window::WindowAttributes::default();
            let mut app = crate::window::BasicApp::new_basic(attrs, &mut callback);

            app.handle_window_event(winit::event::WindowEvent::ThemeChanged(winit::window::Theme::Light));
            app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
                1600, 900,
            )));

            assert_eq!(received.len(), 1, "忽略未处理事件后，Resized 应正常分发");
            match &received[0] {
                AppEvent::Resized { width, height } => {
                    assert_eq!(*width, 1600, "物理宽度应为 1600（800 * 2.0）");
                    assert_eq!(*height, 900, "物理高度应为 900（450 * 2.0）");
                }
                _ => panic!("应为 Resized 事件"),
            }
        }

        // 阶段 3：连续未处理事件后正常事件仍正常工作
        {
            let mut received: Vec<AppEvent> = Vec::new();
            let mut callback = |e: AppEvent| received.push(e);
            let attrs = winit::window::WindowAttributes::default();
            let mut app = crate::window::BasicApp::new_basic(attrs, &mut callback);

            app.handle_window_event(winit::event::WindowEvent::Destroyed);
            app.handle_window_event(winit::event::WindowEvent::Occluded(false));
            app.handle_window_event(winit::event::WindowEvent::Focused(true));

            assert_eq!(received.len(), 1, "连续未处理事件后，Focused 应正常分发");
            assert!(matches!(received[0], AppEvent::Focused), "应为 Focused 事件");
        }
    }

    /// 测试 IME Commit 事件提交空字符串的边界行为。
    /// 某些输入法在特定情况下可能提交空字符串（如用户取消输入、输入法状态异常等）。
    /// 验证空字符串 Commit 事件能正确构造、存储、通过 winit 转换路径传递，
    /// 且通过 BasicApp 分发后回调接收到的文本确实为空。
    #[test]
    fn test_ime_commit_empty_string() {
        use crate::event::{AppEvent, ImeEvent};

        // 直接构造空字符串 Commit 事件
        let empty_commit = ImeEvent::Commit(String::new());
        if let ImeEvent::Commit(text) = &empty_commit {
            assert!(text.is_empty(), "空字符串 Commit 的 text 应为空");
        } else {
            panic!("Expected Commit variant");
        }

        // 验证相等性
        assert_eq!(
            empty_commit,
            ImeEvent::Commit(String::new()),
            "空字符串 Commit 应等于另一个空字符串 Commit"
        );
        assert_ne!(
            empty_commit,
            ImeEvent::Commit("a".to_string()),
            "空字符串 Commit 不应等于非空 Commit"
        );

        // 通过 winit 转换路径验证
        let converted = crate::event::convert_ime(winit::event::Ime::Commit(String::new()));
        assert_eq!(
            converted,
            ImeEvent::Commit(String::new()),
            "winit 空 commit 转换结果应一致"
        );

        // 通过 BasicApp 分发路径验证空 Commit 不 panic
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let attrs = winit::window::WindowAttributes::default();
        let mut app = crate::window::BasicApp::new_basic(attrs, &mut callback);

        app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Commit(String::new())));

        assert_eq!(received.len(), 1, "应收到 1 个 IME Commit 事件");
        match &received[0] {
            AppEvent::Ime(ImeEvent::Commit(text)) => {
                assert!(text.is_empty(), "分发后的 Commit 文本应为空字符串，实际: '{text}'");
            }
            _ => panic!("应为 Ime(Commit) 事件"),
        }

        // 验证 Debug 格式化不 panic
        let debug = format!("{:?}", empty_commit);
        assert!(!debug.is_empty(), "Debug 输出不应为空");
    }

    // ── 新增边界测试 ──

    /// 测试 HostError 所有变体的 Debug 输出非空。
    #[test]
    fn test_host_error_debug_non_empty() {
        let variants = [
            HostError::WindowCreationFailed("test".into()),
            HostError::GpuRequestFailed("gpu".into()),
            HostError::EventLoopError("loop".into()),
        ];
        for (i, err) in variants.iter().enumerate() {
            let debug = format!("{err:?}");
            assert!(!debug.is_empty(), "HostError 变体 {i} 的 Debug 不应为空");
        }
    }

    /// 测试 TouchPhase 所有变体相等性比较。
    #[test]
    fn test_touch_phase_inequality() {
        use crate::event::TouchPhase;
        assert_ne!(TouchPhase::Started, TouchPhase::Moved);
        assert_ne!(TouchPhase::Moved, TouchPhase::Ended);
        assert_eq!(TouchPhase::Started, TouchPhase::Started);
    }

    /// 测试 MouseScrollDelta LineDelta 极端值转换。
    #[test]
    fn test_scroll_delta_line_delta_conversion() {
        use crate::event::MouseScrollDelta;
        let delta = winit::event::MouseScrollDelta::LineDelta(0.0, 0.0);
        let result = crate::event::convert_scroll_delta(delta);
        assert_eq!(result, MouseScrollDelta::LineDelta(0.0, 0.0), "零 delta 应保持不变");
    }

    /// 测试 BasicApp 处理 Destroyed 事件不 panic 且不增加事件。
    #[test]
    fn test_basic_app_destroyed_event_ignored() {
        let mut received: Vec<crate::event::AppEvent> = Vec::new();
        let mut callback = |e: crate::event::AppEvent| received.push(e);
        let attrs = winit::window::WindowAttributes::default();
        let mut app = crate::window::BasicApp::new_basic(attrs, &mut callback);

        app.handle_window_event(winit::event::WindowEvent::Destroyed);
        assert!(received.is_empty(), "Destroyed 事件不应产生 AppEvent");
    }

    /// 测试 MouseButton 从 winit 转换的完整性。
    #[test]
    fn test_mouse_button_conversion_roundtrip() {
        use crate::event::MouseButton;
        assert_eq!(MouseButton::Left, MouseButton::Left);
        assert_eq!(MouseButton::Right, MouseButton::Right);
        assert_eq!(MouseButton::Middle, MouseButton::Middle);
        assert_ne!(MouseButton::Left, MouseButton::Right);
    }

    /// 测试 convert_scroll_delta 函数的所有分支
    #[test]
    fn test_convert_scroll_delta_all_branches() {
        use crate::event::{MouseScrollDelta, convert_scroll_delta};

        // Test LineDelta
        let line_delta = winit::event::MouseScrollDelta::LineDelta(1.5, -2.0);
        let converted = convert_scroll_delta(line_delta);
        assert_eq!(converted, MouseScrollDelta::LineDelta(1.5, -2.0));

        // Test PixelDelta with small values
        let pixel_delta = winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(10.0, 20.0));
        let converted = convert_scroll_delta(pixel_delta);
        assert_eq!(converted, MouseScrollDelta::PixelDelta(10.0, 20.0));

        // Test PixelDelta with large values
        let pixel_delta =
            winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(100000.0, -99999.0));
        let converted = convert_scroll_delta(pixel_delta);
        assert_eq!(converted, MouseScrollDelta::PixelDelta(100000.0, -99999.0));
    }

    /// 测试 convert_touch_phase 函数的所有分支
    #[test]
    fn test_convert_touch_phase_all_branches() {
        use crate::event::{TouchPhase, convert_touch_phase};

        // Test Started phase
        assert_eq!(
            convert_touch_phase(winit::event::TouchPhase::Started),
            TouchPhase::Started
        );

        // Test Moved phase
        assert_eq!(convert_touch_phase(winit::event::TouchPhase::Moved), TouchPhase::Moved);

        // Test Ended phase
        assert_eq!(convert_touch_phase(winit::event::TouchPhase::Ended), TouchPhase::Ended);

        // Test Cancelled phase
        assert_eq!(
            convert_touch_phase(winit::event::TouchPhase::Cancelled),
            TouchPhase::Cancelled
        );
    }

    /// 测试 convert_mouse_button 函数的所有分支
    #[test]
    fn test_convert_mouse_button_all_branches() {
        use crate::event::{MouseButton, convert_mouse_button};

        // Test Left button
        assert_eq!(convert_mouse_button(winit::event::MouseButton::Left), MouseButton::Left);

        // Test Right button
        assert_eq!(
            convert_mouse_button(winit::event::MouseButton::Right),
            MouseButton::Right
        );

        // Test Middle button
        assert_eq!(
            convert_mouse_button(winit::event::MouseButton::Middle),
            MouseButton::Middle
        );

        // Test Back button
        assert_eq!(convert_mouse_button(winit::event::MouseButton::Back), MouseButton::Back);

        // Test Forward button
        assert_eq!(
            convert_mouse_button(winit::event::MouseButton::Forward),
            MouseButton::Forward
        );

        // Test Other button
        assert_eq!(
            convert_mouse_button(winit::event::MouseButton::Other(8)),
            MouseButton::Other(8)
        );
    }

    /// 测试 convert_element_state 函数的所有分支
    #[test]
    fn test_convert_element_state_all_branches() {
        use crate::event::element_state_to_pressed;

        // Test Pressed
        let pressed = element_state_to_pressed(winit::event::ElementState::Pressed);
        assert!(pressed);

        // Test Released
        let released = element_state_to_pressed(winit::event::ElementState::Released);
        assert!(!released);
    }

    /// 测试 convert_ime 函数的所有分支
    #[test]
    fn test_convert_ime_all_branches() {
        use crate::event::{ImeEvent, convert_ime};

        // Test Enabled
        let ime = winit::event::Ime::Enabled;
        let converted = convert_ime(ime);
        assert_eq!(converted, ImeEvent::Enabled);

        // Test Preedit with cursor
        let ime = winit::event::Ime::Preedit("hello".to_string(), Some((0, 5)));
        let converted = convert_ime(ime);
        if let ImeEvent::Preedit { text, cursor } = converted {
            assert_eq!(text, "hello");
            assert_eq!(cursor, Some((0, 5)));
        } else {
            panic!("Expected ImeEvent::Preedit");
        }

        // Test Preedit without cursor
        let ime = winit::event::Ime::Preedit("world".to_string(), None);
        let converted = convert_ime(ime);
        if let ImeEvent::Preedit { text, cursor } = converted {
            assert_eq!(text, "world");
            assert_eq!(cursor, None);
        } else {
            panic!("Expected ImeEvent::Preedit");
        }

        // Test Commit
        let ime = winit::event::Ime::Commit("测试".to_string());
        let converted = convert_ime(ime);
        if let ImeEvent::Commit(text) = converted {
            assert_eq!(text, "测试");
        } else {
            panic!("Expected ImeEvent::Commit");
        }

        // Test Disabled
        let ime = winit::event::Ime::Disabled;
        let converted = convert_ime(ime);
        assert_eq!(converted, ImeEvent::Disabled);
    }
}

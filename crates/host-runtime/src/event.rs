//! 事件类型 — 窗口事件和输入事件

/// 鼠标按钮
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// 左键
    Left,
    /// 右键
    Right,
    /// 中键
    Middle,
    /// 后退键
    Back,
    /// 前进键
    Forward,
    /// 其他按钮
    Other(u16),
}

/// 鼠标滚轮滚动增量
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseScrollDelta {
    /// 像素增量
    PixelDelta(f64, f64),
    /// 行增量
    LineDelta(f32, f32),
}

/// IME 输入事件
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImeEvent {
    /// IME 已启用
    Enabled,
    /// 预编辑文本（正在输入的文本，以及光标范围）
    Preedit {
        /// 预编辑文本
        text: String,
        /// 光标范围 (start, end)，None 表示无光标
        cursor: Option<(usize, usize)>,
    },
    /// 提交文本（用户确认输入）
    Commit(String),
    /// IME 已禁用
    Disabled,
}

/// 触摸事件阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchPhase {
    /// 手指按下
    Started,
    /// 手指移动
    Moved,
    /// 手指抬起
    Ended,
    /// 触摸取消
    Cancelled,
}

/// 触摸事件
#[derive(Debug, Clone)]
pub struct TouchEvent {
    /// 触摸点 ID
    pub id: u64,
    /// 触摸阶段
    pub phase: TouchPhase,
    /// 触摸位置（相对于窗口左上角，物理像素）
    pub x: f64,
    /// 触摸位置 Y
    pub y: f64,
}

/// 应用生命周期事件
#[derive(Debug)]
pub enum AppEvent {
    /// 窗口需要重绘
    RedrawRequested,
    /// 窗口大小变更
    Resized {
        /// 新宽度
        width: u32,
        /// 新高度
        height: u32,
    },
    /// 窗口关闭请求
    CloseRequested,
    /// 窗口获得焦点
    Focused,
    /// 窗口失去焦点
    Unfocused,
    /// 键盘输入
    KeyboardInput {
        /// 按键名称（逻辑键名）
        key: String,
        /// 是否按下（true = 按下, false = 释放）
        pressed: bool,
    },
    /// 鼠标移动
    MouseMoved {
        /// X 坐标（物理像素，相对于窗口左上角）
        x: f64,
        /// Y 坐标（物理像素，相对于窗口左上角）
        y: f64,
    },
    /// 鼠标按钮按下/释放
    MouseInput {
        /// 鼠标按钮
        button: MouseButton,
        /// 是否按下（true = 按下, false = 释放）
        pressed: bool,
    },
    /// 鼠标滚轮滚动
    MouseWheel {
        /// 滚动增量
        delta: MouseScrollDelta,
    },
    /// 触摸事件
    Touch(TouchEvent),
    /// IME 输入事件（用于中文、日文、韩文等输入法）
    Ime(ImeEvent),
}

/// 将 winit 的 ElementState 转换为布尔值（pressed）
pub(crate) fn element_state_to_pressed(state: winit::event::ElementState) -> bool {
    match state {
        winit::event::ElementState::Pressed => true,
        winit::event::ElementState::Released => false,
    }
}

/// 将 winit 的 MouseButton 转换为自定义 MouseButton
pub(crate) fn convert_mouse_button(btn: winit::event::MouseButton) -> MouseButton {
    match btn {
        winit::event::MouseButton::Left => MouseButton::Left,
        winit::event::MouseButton::Right => MouseButton::Right,
        winit::event::MouseButton::Middle => MouseButton::Middle,
        winit::event::MouseButton::Back => MouseButton::Back,
        winit::event::MouseButton::Forward => MouseButton::Forward,
        winit::event::MouseButton::Other(n) => MouseButton::Other(n),
    }
}

/// 将 winit 的 MouseScrollDelta 转换为自定义 MouseScrollDelta
pub(crate) fn convert_scroll_delta(delta: winit::event::MouseScrollDelta) -> MouseScrollDelta {
    match delta {
        winit::event::MouseScrollDelta::PixelDelta(pos) => {
            MouseScrollDelta::PixelDelta(pos.x, pos.y)
        }
        winit::event::MouseScrollDelta::LineDelta(x, y) => MouseScrollDelta::LineDelta(x, y),
    }
}

/// 将 winit 的 Ime 转换为自定义 ImeEvent
pub(crate) fn convert_ime(ime: winit::event::Ime) -> ImeEvent {
    match ime {
        winit::event::Ime::Enabled => ImeEvent::Enabled,
        winit::event::Ime::Preedit(text, cursor) => ImeEvent::Preedit { text, cursor },
        winit::event::Ime::Commit(text) => ImeEvent::Commit(text),
        winit::event::Ime::Disabled => ImeEvent::Disabled,
    }
}

/// 将 winit 的 TouchPhase 转换为自定义 TouchPhase
pub(crate) fn convert_touch_phase(phase: winit::event::TouchPhase) -> TouchPhase {
    match phase {
        winit::event::TouchPhase::Started => TouchPhase::Started,
        winit::event::TouchPhase::Moved => TouchPhase::Moved,
        winit::event::TouchPhase::Ended => TouchPhase::Ended,
        winit::event::TouchPhase::Cancelled => TouchPhase::Cancelled,
    }
}

/// 从 winit KeyboardInput WindowEvent 中提取 AppEvent::KeyboardInput
pub(crate) fn convert_keyboard_input(
    device_id: winit::event::DeviceId,
    event: winit::event::KeyEvent,
    is_synthetic: bool,
) -> AppEvent {
    let _ = (device_id, is_synthetic);
    let key_text = match &event.logical_key {
        winit::keyboard::Key::Named(named) => format!("{:?}", named),
        winit::keyboard::Key::Character(ch) => ch.to_string(),
        winit::keyboard::Key::Unidentified(_) => String::from("Unidentified"),
        winit::keyboard::Key::Dead(dead) => {
            if let Some(ch) = dead {
                format!("Dead({})", ch)
            } else {
                String::from("Dead")
            }
        }
    };
    let pressed = element_state_to_pressed(event.state);
    AppEvent::KeyboardInput { key: key_text, pressed }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_event_resized_values() {
        let event = AppEvent::Resized {
            width: 1920,
            height: 1080,
        };
        if let AppEvent::Resized { width, height } = event {
            assert_eq!(width, 1920);
            assert_eq!(height, 1080);
        } else {
            panic!("Expected Resized variant");
        }
    }

    #[test]
    fn test_app_event_keyboard_input_values() {
        let pressed = AppEvent::KeyboardInput {
            key: "A".to_string(),
            pressed: true,
        };
        if let AppEvent::KeyboardInput { key, pressed: p } = &pressed {
            assert_eq!(key, "A");
            assert!(p);
        } else {
            panic!("Expected KeyboardInput variant");
        }

        let released = AppEvent::KeyboardInput {
            key: "Escape".to_string(),
            pressed: false,
        };
        if let AppEvent::KeyboardInput { pressed: p, .. } = &released {
            assert!(!p);
        } else {
            panic!("Expected KeyboardInput variant");
        }
    }

    #[test]
    fn test_app_event_debug_format() {
        assert!(!format!("{:?}", AppEvent::RedrawRequested).is_empty());
        assert!(!format!("{:?}", AppEvent::CloseRequested).is_empty());
        assert!(!format!("{:?}", AppEvent::Focused).is_empty());
        assert!(!format!("{:?}", AppEvent::Unfocused).is_empty());
        assert!(
            format!(
                "{:?}",
                AppEvent::Resized {
                    width: 100,
                    height: 200
                }
            )
            .contains("100")
        );
        assert!(
            format!(
                "{:?}",
                AppEvent::KeyboardInput {
                    key: "X".into(),
                    pressed: true
                }
            )
            .contains("X")
        );
    }

    #[test]
    fn test_app_event_focused_unfocused_distinct() {
        let focused = format!("{:?}", AppEvent::Focused);
        let unfocused = format!("{:?}", AppEvent::Unfocused);
        assert_ne!(focused, unfocused);
    }

    // --- 新增事件类型测试 ---

    #[test]
    fn test_mouse_button_variants() {
        assert_eq!(format!("{:?}", MouseButton::Left), "Left");
        assert_eq!(format!("{:?}", MouseButton::Right), "Right");
        assert_eq!(format!("{:?}", MouseButton::Middle), "Middle");
        assert_eq!(format!("{:?}", MouseButton::Back), "Back");
        assert_eq!(format!("{:?}", MouseButton::Forward), "Forward");
        assert_eq!(format!("{:?}", MouseButton::Other(8)), "Other(8)");
    }

    #[test]
    fn test_mouse_button_equality() {
        assert_eq!(MouseButton::Left, MouseButton::Left);
        assert_ne!(MouseButton::Left, MouseButton::Right);
        assert_eq!(MouseButton::Other(3), MouseButton::Other(3));
        assert_ne!(MouseButton::Other(3), MouseButton::Other(4));
    }

    #[test]
    fn test_mouse_scroll_delta_pixel() {
        let delta = MouseScrollDelta::PixelDelta(10.0, 20.0);
        assert_eq!(delta, MouseScrollDelta::PixelDelta(10.0, 20.0));
    }

    #[test]
    fn test_mouse_scroll_delta_line() {
        let delta = MouseScrollDelta::LineDelta(3.0, -1.0);
        assert_eq!(delta, MouseScrollDelta::LineDelta(3.0, -1.0));
    }

    #[test]
    fn test_ime_event_enabled() {
        let e = ImeEvent::Enabled;
        assert!(format!("{:?}", e).contains("Enabled"));
    }

    #[test]
    fn test_ime_event_preedit() {
        let e = ImeEvent::Preedit {
            text: "你好".to_string(),
            cursor: Some((0, 2)),
        };
        if let ImeEvent::Preedit { text, cursor } = e {
            assert_eq!(text, "你好");
            assert_eq!(cursor, Some((0, 2)));
        } else {
            panic!("Expected Preedit");
        }
    }

    #[test]
    fn test_ime_event_preedit_no_cursor() {
        let e = ImeEvent::Preedit {
            text: "abc".to_string(),
            cursor: None,
        };
        if let ImeEvent::Preedit { text, cursor } = e {
            assert_eq!(text, "abc");
            assert!(cursor.is_none());
        } else {
            panic!("Expected Preedit");
        }
    }

    #[test]
    fn test_ime_event_commit() {
        let e = ImeEvent::Commit("你好世界".to_string());
        if let ImeEvent::Commit(text) = e {
            assert_eq!(text, "你好世界");
        } else {
            panic!("Expected Commit");
        }
    }

    #[test]
    fn test_ime_event_disabled() {
        let e = ImeEvent::Disabled;
        assert!(format!("{:?}", e).contains("Disabled"));
    }

    #[test]
    fn test_ime_event_equality() {
        assert_eq!(ImeEvent::Enabled, ImeEvent::Enabled);
        assert_ne!(ImeEvent::Enabled, ImeEvent::Disabled);
        assert_eq!(
            ImeEvent::Commit("a".to_string()),
            ImeEvent::Commit("a".to_string())
        );
    }

    #[test]
    fn test_touch_phase_variants() {
        assert_eq!(TouchPhase::Started, TouchPhase::Started);
        assert_ne!(TouchPhase::Started, TouchPhase::Ended);
    }

    #[test]
    fn test_touch_event_fields() {
        let te = TouchEvent {
            id: 42,
            phase: TouchPhase::Moved,
            x: 100.0,
            y: 200.0,
        };
        assert_eq!(te.id, 42);
        assert_eq!(te.phase, TouchPhase::Moved);
        assert!((te.x - 100.0).abs() < f64::EPSILON);
        assert!((te.y - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_app_event_mouse_moved() {
        let e = AppEvent::MouseMoved { x: 50.0, y: 75.0 };
        if let AppEvent::MouseMoved { x, y } = e {
            assert!((x - 50.0).abs() < f64::EPSILON);
            assert!((y - 75.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected MouseMoved");
        }
    }

    #[test]
    fn test_app_event_mouse_input() {
        let e = AppEvent::MouseInput {
            button: MouseButton::Left,
            pressed: true,
        };
        if let AppEvent::MouseInput { button, pressed } = e {
            assert_eq!(button, MouseButton::Left);
            assert!(pressed);
        } else {
            panic!("Expected MouseInput");
        }
    }

    #[test]
    fn test_app_event_mouse_wheel() {
        let e = AppEvent::MouseWheel {
            delta: MouseScrollDelta::LineDelta(1.0, 0.0),
        };
        if let AppEvent::MouseWheel { delta } = &e {
            assert_eq!(*delta, MouseScrollDelta::LineDelta(1.0, 0.0));
        } else {
            panic!("Expected MouseWheel");
        }
    }

    #[test]
    fn test_app_event_touch() {
        let e = AppEvent::Touch(TouchEvent {
            id: 0,
            phase: TouchPhase::Started,
            x: 10.0,
            y: 20.0,
        });
        if let AppEvent::Touch(te) = &e {
            assert_eq!(te.id, 0);
            assert_eq!(te.phase, TouchPhase::Started);
        } else {
            panic!("Expected Touch");
        }
    }

    #[test]
    fn test_app_event_ime() {
        let e = AppEvent::Ime(ImeEvent::Commit("abc".to_string()));
        if let AppEvent::Ime(ImeEvent::Commit(s)) = &e {
            assert_eq!(s, "abc");
        } else {
            panic!("Expected Ime(Commit)");
        }
    }

    // --- 转换函数测试 ---

    #[test]
    fn test_convert_mouse_button_all_variants() {
        assert_eq!(
            convert_mouse_button(winit::event::MouseButton::Left),
            MouseButton::Left
        );
        assert_eq!(
            convert_mouse_button(winit::event::MouseButton::Right),
            MouseButton::Right
        );
        assert_eq!(
            convert_mouse_button(winit::event::MouseButton::Middle),
            MouseButton::Middle
        );
        assert_eq!(
            convert_mouse_button(winit::event::MouseButton::Back),
            MouseButton::Back
        );
        assert_eq!(
            convert_mouse_button(winit::event::MouseButton::Forward),
            MouseButton::Forward
        );
        assert_eq!(
            convert_mouse_button(winit::event::MouseButton::Other(9)),
            MouseButton::Other(9)
        );
    }

    #[test]
    fn test_convert_scroll_delta_pixel() {
        let pos = winit::dpi::PhysicalPosition::new(5.0, -3.0);
        let delta = winit::event::MouseScrollDelta::PixelDelta(pos);
        let result = convert_scroll_delta(delta);
        assert_eq!(result, MouseScrollDelta::PixelDelta(5.0, -3.0));
    }

    #[test]
    fn test_convert_scroll_delta_line() {
        let delta = winit::event::MouseScrollDelta::LineDelta(2.0, -1.0);
        let result = convert_scroll_delta(delta);
        assert_eq!(result, MouseScrollDelta::LineDelta(2.0, -1.0));
    }

    #[test]
    fn test_convert_ime_enabled() {
        let result = convert_ime(winit::event::Ime::Enabled);
        assert_eq!(result, ImeEvent::Enabled);
    }

    #[test]
    fn test_convert_ime_preedit() {
        let result = convert_ime(winit::event::Ime::Preedit("abc".to_string(), Some((0, 1))));
        assert_eq!(
            result,
            ImeEvent::Preedit {
                text: "abc".to_string(),
                cursor: Some((0, 1))
            }
        );
    }

    #[test]
    fn test_convert_ime_preedit_no_cursor() {
        let result = convert_ime(winit::event::Ime::Preedit(String::new(), None));
        assert_eq!(
            result,
            ImeEvent::Preedit {
                text: String::new(),
                cursor: None
            }
        );
    }

    #[test]
    fn test_convert_ime_commit() {
        let result = convert_ime(winit::event::Ime::Commit("hello".to_string()));
        assert_eq!(result, ImeEvent::Commit("hello".to_string()));
    }

    #[test]
    fn test_convert_ime_disabled() {
        let result = convert_ime(winit::event::Ime::Disabled);
        assert_eq!(result, ImeEvent::Disabled);
    }

    #[test]
    fn test_convert_touch_phase_all() {
        assert_eq!(
            convert_touch_phase(winit::event::TouchPhase::Started),
            TouchPhase::Started
        );
        assert_eq!(
            convert_touch_phase(winit::event::TouchPhase::Moved),
            TouchPhase::Moved
        );
        assert_eq!(
            convert_touch_phase(winit::event::TouchPhase::Ended),
            TouchPhase::Ended
        );
        assert_eq!(
            convert_touch_phase(winit::event::TouchPhase::Cancelled),
            TouchPhase::Cancelled
        );
    }

    #[test]
    fn test_element_state_to_pressed() {
        assert!(element_state_to_pressed(winit::event::ElementState::Pressed));
        assert!(!element_state_to_pressed(winit::event::ElementState::Released));
    }

    #[test]
    fn test_app_event_new_variants_debug() {
        assert!(!format!("{:?}", AppEvent::MouseMoved { x: 0.0, y: 0.0 }).is_empty());
        assert!(
            !format!(
                "{:?}",
                AppEvent::MouseInput {
                    button: MouseButton::Left,
                    pressed: true
                }
            )
            .is_empty()
        );
        assert!(
            !format!(
                "{:?}",
                AppEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(1.0, 0.0)
                }
            )
            .is_empty()
        );
        assert!(
            !format!(
                "{:?}",
                AppEvent::Touch(TouchEvent {
                    id: 0,
                    phase: TouchPhase::Started,
                    x: 0.0,
                    y: 0.0
                })
            )
            .is_empty()
        );
        assert!(!format!("{:?}", AppEvent::Ime(ImeEvent::Enabled)).is_empty());
    }

    // --- Additional coverage tests ---

    #[test]
    fn test_mouse_button_copy() {
        let btn = MouseButton::Left;
        let btn2 = btn;
        assert_eq!(btn, btn2);
    }

    #[test]
    fn test_mouse_button_other_copy() {
        let btn = MouseButton::Other(16);
        let btn2 = btn;
        assert_eq!(btn, btn2);
    }

    #[test]
    fn test_mouse_scroll_delta_copy() {
        let delta = MouseScrollDelta::PixelDelta(100.0, 200.0);
        let delta2 = delta;
        assert_eq!(delta, delta2);
    }

    #[test]
    fn test_mouse_scroll_delta_line_copy() {
        let delta = MouseScrollDelta::LineDelta(-5.0, 3.0);
        let delta2 = delta;
        assert_eq!(delta, delta2);
    }

    #[test]
    fn test_touch_phase_copy() {
        let phase = TouchPhase::Moved;
        let phase2 = phase;
        assert_eq!(phase, phase2);
    }

    #[test]
    fn test_touch_phase_all_variants_distinct() {
        let phases = [TouchPhase::Started, TouchPhase::Moved, TouchPhase::Ended, TouchPhase::Cancelled];
        for i in 0..phases.len() {
            for j in 0..phases.len() {
                if i == j {
                    assert_eq!(phases[i], phases[j]);
                } else {
                    assert_ne!(phases[i], phases[j]);
                }
            }
        }
    }

    #[test]
    fn test_touch_event_clone() {
        let te = TouchEvent {
            id: 99,
            phase: TouchPhase::Ended,
            x: 500.0,
            y: -10.0,
        };
        let te2 = te.clone();
        assert_eq!(te.id, te2.id);
        assert_eq!(te.phase, te2.phase);
        assert!((te.x - te2.x).abs() < f64::EPSILON);
        assert!((te.y - te2.y).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ime_event_preedit_equality() {
        let a = ImeEvent::Preedit {
            text: "x".to_string(),
            cursor: Some((0, 1)),
        };
        let b = ImeEvent::Preedit {
            text: "x".to_string(),
            cursor: Some((0, 1)),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_ime_event_preedit_inequality() {
        let a = ImeEvent::Preedit {
            text: "a".to_string(),
            cursor: None,
        };
        let b = ImeEvent::Preedit {
            text: "b".to_string(),
            cursor: None,
        };
        assert_ne!(a, b);
    }

    #[test]
    fn test_ime_event_commit_empty_string() {
        let e = ImeEvent::Commit(String::new());
        if let ImeEvent::Commit(s) = e {
            assert!(s.is_empty());
        } else {
            panic!("Expected Commit");
        }
    }

    #[test]
    fn test_ime_event_preedit_empty_text_with_cursor() {
        let e = ImeEvent::Preedit {
            text: String::new(),
            cursor: Some((0, 0)),
        };
        if let ImeEvent::Preedit { text, cursor } = e {
            assert!(text.is_empty());
            assert_eq!(cursor, Some((0, 0)));
        } else {
            panic!("Expected Preedit");
        }
    }

    #[test]
    fn test_app_event_resized_large_values() {
        let event = AppEvent::Resized {
            width: u32::MAX,
            height: u32::MAX,
        };
        if let AppEvent::Resized { width, height } = event {
            assert_eq!(width, u32::MAX);
            assert_eq!(height, u32::MAX);
        } else {
            panic!("Expected Resized");
        }
    }

    #[test]
    fn test_app_event_mouse_moved_zero() {
        let e = AppEvent::MouseMoved { x: 0.0, y: 0.0 };
        if let AppEvent::MouseMoved { x, y } = e {
            assert!((x - 0.0).abs() < f64::EPSILON);
            assert!((y - 0.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected MouseMoved");
        }
    }

    #[test]
    fn test_app_event_mouse_moved_negative_coords() {
        let e = AppEvent::MouseMoved { x: -999.5, y: -0.1 };
        if let AppEvent::MouseMoved { x, y } = e {
            assert!((x - (-999.5)).abs() < 1e-10);
            assert!((y - (-0.1)).abs() < 1e-10);
        } else {
            panic!("Expected MouseMoved");
        }
    }

    #[test]
    fn test_app_event_mouse_input_right_released() {
        let e = AppEvent::MouseInput {
            button: MouseButton::Right,
            pressed: false,
        };
        if let AppEvent::MouseInput { button, pressed } = e {
            assert_eq!(button, MouseButton::Right);
            assert!(!pressed);
        } else {
            panic!("Expected MouseInput");
        }
    }

    #[test]
    fn test_app_event_mouse_input_middle_pressed() {
        let e = AppEvent::MouseInput {
            button: MouseButton::Middle,
            pressed: true,
        };
        if let AppEvent::MouseInput { button, pressed } = e {
            assert_eq!(button, MouseButton::Middle);
            assert!(pressed);
        } else {
            panic!("Expected MouseInput");
        }
    }

    #[test]
    fn test_app_event_mouse_input_back_forward() {
        let back = AppEvent::MouseInput {
            button: MouseButton::Back,
            pressed: true,
        };
        let fwd = AppEvent::MouseInput {
            button: MouseButton::Forward,
            pressed: true,
        };
        if let AppEvent::MouseInput { button, .. } = &back {
            assert_eq!(*button, MouseButton::Back);
        } else {
            panic!("Expected MouseInput");
        }
        if let AppEvent::MouseInput { button, .. } = &fwd {
            assert_eq!(*button, MouseButton::Forward);
        } else {
            panic!("Expected MouseInput");
        }
    }

    #[test]
    fn test_app_event_mouse_wheel_pixel_delta_large() {
        let e = AppEvent::MouseWheel {
            delta: MouseScrollDelta::PixelDelta(100000.0, -99999.0),
        };
        if let AppEvent::MouseWheel { delta } = &e {
            assert_eq!(*delta, MouseScrollDelta::PixelDelta(100000.0, -99999.0));
        } else {
            panic!("Expected MouseWheel");
        }
    }

    #[test]
    fn test_app_event_touch_started_at_origin() {
        let e = AppEvent::Touch(TouchEvent {
            id: 0,
            phase: TouchPhase::Started,
            x: 0.0,
            y: 0.0,
        });
        if let AppEvent::Touch(te) = &e {
            assert_eq!(te.id, 0);
            assert_eq!(te.phase, TouchPhase::Started);
        } else {
            panic!("Expected Touch");
        }
    }

    #[test]
    fn test_app_event_ime_preedit_dispatch() {
        let e = AppEvent::Ime(ImeEvent::Preedit {
            text: "abc".to_string(),
            cursor: Some((0, 3)),
        });
        if let AppEvent::Ime(ImeEvent::Preedit { text, cursor }) = &e {
            assert_eq!(text, "abc");
            assert_eq!(*cursor, Some((0, 3)));
        } else {
            panic!("Expected Ime(Preedit)");
        }
    }

    #[test]
    fn test_mouse_button_all_debug_roundtrip() {
        let variants: Vec<MouseButton> = vec![
            MouseButton::Left,
            MouseButton::Right,
            MouseButton::Middle,
            MouseButton::Back,
            MouseButton::Forward,
            MouseButton::Other(0),
            MouseButton::Other(u16::MAX),
        ];
        for v in &variants {
            let debug = format!("{:?}", v);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_mouse_scroll_delta_pixel_inequality() {
        let a = MouseScrollDelta::PixelDelta(1.0, 2.0);
        let b = MouseScrollDelta::PixelDelta(1.0, 3.0);
        assert_ne!(a, b);
    }

    #[test]
    fn test_mouse_scroll_delta_cross_variant_inequality() {
        let pixel = MouseScrollDelta::PixelDelta(1.0, 2.0);
        let line = MouseScrollDelta::LineDelta(1.0, 2.0);
        assert_ne!(pixel, line);
    }
}

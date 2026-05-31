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
        winit::event::MouseScrollDelta::PixelDelta(pos) => MouseScrollDelta::PixelDelta(pos.x, pos.y),
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
        assert_eq!(ImeEvent::Commit("a".to_string()), ImeEvent::Commit("a".to_string()));
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
        assert_eq!(convert_mouse_button(winit::event::MouseButton::Left), MouseButton::Left);
        assert_eq!(
            convert_mouse_button(winit::event::MouseButton::Right),
            MouseButton::Right
        );
        assert_eq!(
            convert_mouse_button(winit::event::MouseButton::Middle),
            MouseButton::Middle
        );
        assert_eq!(convert_mouse_button(winit::event::MouseButton::Back), MouseButton::Back);
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
        assert_eq!(convert_touch_phase(winit::event::TouchPhase::Moved), TouchPhase::Moved);
        assert_eq!(convert_touch_phase(winit::event::TouchPhase::Ended), TouchPhase::Ended);
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
        let phases = [
            TouchPhase::Started,
            TouchPhase::Moved,
            TouchPhase::Ended,
            TouchPhase::Cancelled,
        ];
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

    // --- 高优先级事件处理测试 ---

    /// 验证：窗口 resize 事件携带正确的新尺寸（包括极端值和常见分辨率）
    #[test]
    fn test_resize_event_carries_correct_dimensions() {
        let cases: Vec<(u32, u32)> = vec![
            (1920, 1080), // Full HD
            (2560, 1440), // QHD
            (1366, 768),  // 常见笔记本
            (1, 1),       // 最小值
        ];
        for (w, h) in cases {
            let event = AppEvent::Resized { width: w, height: h };
            if let AppEvent::Resized { width, height } = event {
                assert_eq!(width, w, "resize width 不匹配: 期望 {w}, 实际 {width}");
                assert_eq!(height, h, "resize height 不匹配: 期望 {h}, 实际 {height}");
            } else {
                panic!("Expected Resized variant");
            }
        }
    }

    /// 验证：鼠标移动事件坐标精确传递（包括分数值和负值）
    #[test]
    fn test_mouse_move_coordinates_precision() {
        let cases: Vec<(f64, f64)> = vec![
            (0.0, 0.0),
            (1920.5, 1080.25), // 分数坐标
            (-100.0, -200.0),  // 窗口外
        ];
        for (x, y) in cases {
            let event = AppEvent::MouseMoved { x, y };
            if let AppEvent::MouseMoved { x: ex, y: ey } = event {
                assert!((ex - x).abs() < f64::EPSILON, "x 坐标不精确: 期望 {x}, 实际 {ex}");
                assert!((ey - y).abs() < f64::EPSILON, "y 坐标不精确: 期望 {y}, 实际 {ey}");
            } else {
                panic!("Expected MouseMoved variant");
            }
        }
    }

    /// 验证：IME 组合事件完整流程 — Enabled → 多次 Preedit → Commit → Disabled
    #[test]
    fn test_ime_composition_full_lifecycle() {
        // 模拟拼音输入"中"的完整流程
        let enabled = ImeEvent::Enabled;
        let preedit1 = ImeEvent::Preedit {
            text: "z".to_string(),
            cursor: Some((0, 1)),
        };
        let preedit2 = ImeEvent::Preedit {
            text: "zh".to_string(),
            cursor: Some((0, 2)),
        };
        let preedit3 = ImeEvent::Preedit {
            text: "zhon".to_string(),
            cursor: Some((0, 4)),
        };
        let preedit4 = ImeEvent::Preedit {
            text: "zhong".to_string(),
            cursor: Some((0, 5)),
        };
        let commit = ImeEvent::Commit("中".to_string());
        let disabled = ImeEvent::Disabled;

        // 验证每个阶段的事件类型和内容
        assert!(matches!(enabled, ImeEvent::Enabled));
        assert_eq!(
            preedit3,
            ImeEvent::Preedit {
                text: "zhon".to_string(),
                cursor: Some((0, 4))
            }
        );
        assert_eq!(commit, ImeEvent::Commit("中".to_string()));
        assert!(matches!(disabled, ImeEvent::Disabled));

        // 验证完整流程按序收集
        let lifecycle: Vec<ImeEvent> = vec![enabled, preedit1, preedit2, preedit3, preedit4, commit, disabled];
        assert_eq!(lifecycle.len(), 7);

        // 验证 commit 文本
        if let ImeEvent::Commit(text) = &lifecycle[5] {
            assert!(!text.is_empty(), "IME commit 文本不应为空");
            assert_eq!(text, "中");
        } else {
            panic!("第 6 个事件应为 Commit");
        }

        // 验证 Preedit 文本逐步增长
        let texts: Vec<&str> = lifecycle[1..=4]
            .iter()
            .map(|e| {
                if let ImeEvent::Preedit { text, .. } = e {
                    text.as_str()
                } else {
                    ""
                }
            })
            .collect();
        assert_eq!(texts, vec!["z", "zh", "zhon", "zhong"]);
    }

    /// 验证：键盘修饰键状态通过 element_state_to_pressed 正确转换
    /// （Ctrl/Shift 按下时 pressed=true，释放时 pressed=false）
    #[test]
    fn test_keyboard_modifier_state_conversion() {
        // 模拟 Ctrl+Shift 组合键的按下和释放
        // 1. Ctrl 按下
        let ctrl_pressed = element_state_to_pressed(winit::event::ElementState::Pressed);
        assert!(ctrl_pressed, "Ctrl 按下时 pressed 应为 true");

        // 2. Shift 按下（同时 Ctrl 仍按住）
        let shift_pressed = element_state_to_pressed(winit::event::ElementState::Pressed);
        assert!(shift_pressed, "Shift 按下时 pressed 应为 true");

        // 3. 字符键按下（Ctrl+Shift+A）
        let char_pressed = element_state_to_pressed(winit::event::ElementState::Pressed);
        assert!(char_pressed, "字符键按下时 pressed 应为 true");

        // 4. 字符键释放
        let char_released = element_state_to_pressed(winit::event::ElementState::Released);
        assert!(!char_released, "字符键释放时 pressed 应为 false");

        // 5. Shift 释放
        let shift_released = element_state_to_pressed(winit::event::ElementState::Released);
        assert!(!shift_released, "Shift 释放时 pressed 应为 false");

        // 6. Ctrl 释放
        let ctrl_released = element_state_to_pressed(winit::event::ElementState::Released);
        assert!(!ctrl_released, "Ctrl 释放时 pressed 应为 false");

        // 验证 AppEvent::KeyboardInput 正确承载修饰键按下/释放状态
        let ctrl_down_event = AppEvent::KeyboardInput {
            key: "Control".to_string(),
            pressed: ctrl_pressed,
        };
        let shift_down_event = AppEvent::KeyboardInput {
            key: "Shift".to_string(),
            pressed: shift_pressed,
        };
        if let AppEvent::KeyboardInput { key, pressed } = ctrl_down_event {
            assert_eq!(key, "Control");
            assert!(pressed);
        } else {
            panic!("Expected KeyboardInput");
        }
        if let AppEvent::KeyboardInput { key, pressed } = shift_down_event {
            assert_eq!(key, "Shift");
            assert!(pressed);
        } else {
            panic!("Expected KeyboardInput");
        }

        // 释放后的事件
        let ctrl_up_event = AppEvent::KeyboardInput {
            key: "Control".to_string(),
            pressed: ctrl_released,
        };
        if let AppEvent::KeyboardInput { pressed, .. } = ctrl_up_event {
            assert!(!pressed, "Ctrl 释放事件 pressed 应为 false");
        } else {
            panic!("Expected KeyboardInput");
        }
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
}

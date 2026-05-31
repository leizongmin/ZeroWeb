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
        };
        if let AppEvent::MouseInput { button, pressed } = press {
            assert_eq!(button, MouseButton::Left, "按钮应为左键");
            assert!(pressed, "应为按下状态");
        } else {
            panic!("Expected MouseInput variant");
        }

        // 左键释放
        let release = AppEvent::MouseInput {
            button: MouseButton::Left,
            pressed: false,
        };
        if let AppEvent::MouseInput { button, pressed } = release {
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
                pressed,
            };
            if let AppEvent::KeyboardInput { key, pressed: p } = event {
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
}

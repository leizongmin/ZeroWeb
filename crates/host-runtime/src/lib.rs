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

    /// 测试鼠标事件坐标的正确性。
    /// 验证 AppEvent::MouseMoved 携带的 x/y 坐标与构造时一致，
    /// 包括整数坐标、分数坐标和负坐标。
    #[test]
    fn test_mouse_event_coordinates() {
        use crate::event::AppEvent;

        let cases: Vec<(f64, f64)> = vec![(100.0, 200.0), (0.0, 0.0), (1920.5, 1080.25), (-50.0, -100.0)];
        for (x, y) in cases {
            let event = AppEvent::MouseMoved { x, y };
            if let AppEvent::MouseMoved { x: ex, y: ey } = event {
                assert!((ex - x).abs() < f64::EPSILON, "x mismatch: expected {x}, got {ex}");
                assert!((ey - y).abs() < f64::EPSILON, "y mismatch: expected {y}, got {ey}");
            } else {
                panic!("Expected MouseMoved variant");
            }
        }
    }

    /// 测试键盘事件的按键名称和修饰键状态。
    /// 验证 AppEvent::KeyboardInput 的 key 和 pressed 字段正确存储，
    /// 模拟 Ctrl+Shift+X 组合键序列。
    #[test]
    fn test_keyboard_event_key_code() {
        use crate::event::AppEvent;

        // 模拟 Ctrl+Shift+X 组合键序列
        let events: Vec<AppEvent> = vec![
            AppEvent::KeyboardInput {
                key: "Control".to_string(),
                pressed: true,
            },
            AppEvent::KeyboardInput {
                key: "Shift".to_string(),
                pressed: true,
            },
            AppEvent::KeyboardInput {
                key: "X".to_string(),
                pressed: true,
            },
            AppEvent::KeyboardInput {
                key: "X".to_string(),
                pressed: false,
            },
            AppEvent::KeyboardInput {
                key: "Shift".to_string(),
                pressed: false,
            },
            AppEvent::KeyboardInput {
                key: "Control".to_string(),
                pressed: false,
            },
        ];

        assert_eq!(events.len(), 6, "should have 6 events in the sequence");

        // 验证修饰键按下
        if let AppEvent::KeyboardInput { key, pressed } = &events[0] {
            assert_eq!(key, "Control");
            assert!(pressed, "Control pressed should be true");
        } else {
            panic!("Expected KeyboardInput");
        }
        if let AppEvent::KeyboardInput { key, pressed } = &events[1] {
            assert_eq!(key, "Shift");
            assert!(pressed, "Shift pressed should be true");
        } else {
            panic!("Expected KeyboardInput");
        }

        // 验证字符键按下和释放
        if let AppEvent::KeyboardInput { key, pressed } = &events[2] {
            assert_eq!(key, "X");
            assert!(pressed, "X pressed should be true");
        } else {
            panic!("Expected KeyboardInput");
        }
        if let AppEvent::KeyboardInput { key, pressed } = &events[3] {
            assert_eq!(key, "X");
            assert!(!pressed, "X released pressed should be false");
        } else {
            panic!("Expected KeyboardInput");
        }

        // 验证修饰键释放顺序
        if let AppEvent::KeyboardInput { key, pressed } = &events[4] {
            assert_eq!(key, "Shift");
            assert!(!pressed, "Shift released pressed should be false");
        } else {
            panic!("Expected KeyboardInput");
        }
        if let AppEvent::KeyboardInput { key, pressed } = &events[5] {
            assert_eq!(key, "Control");
            assert!(!pressed, "Control released pressed should be false");
        } else {
            panic!("Expected KeyboardInput");
        }
    }

    /// 测试窗口 resize 事件的宽高字段。
    /// 验证 AppEvent::Resized 携带的 width/height 与构造值一致。
    #[test]
    fn test_window_event_resize() {
        use crate::event::AppEvent;

        let cases: Vec<(u32, u32)> = vec![(800, 600), (0, 0), (3840, 2160), (1, 1)];
        for (w, h) in cases {
            let event = AppEvent::Resized { width: w, height: h };
            if let AppEvent::Resized { width, height } = event {
                assert_eq!(width, w, "width mismatch: expected {w}, got {width}");
                assert_eq!(height, h, "height mismatch: expected {h}, got {height}");
            } else {
                panic!("Expected Resized variant");
            }
        }
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
}

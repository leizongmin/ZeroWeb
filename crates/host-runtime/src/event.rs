//! 事件类型 — 窗口事件和输入事件

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
        /// 按键
        key: String,
        /// 是否按下（true = 按下, false = 释放）
        pressed: bool,
    },
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
}

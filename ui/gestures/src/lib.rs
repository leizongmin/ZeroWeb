//! # zero-ui-gestures
//!
//! 手势识别（spec §8.4.1 `zero-ui-gestures` / FR-016）。
//!
//! 手势进入 arena；未被 chrome 消费时转发 WebView（spec §8.4.1B）。
//! M1 提供 TapRecognizer（按下→抬起在阈值内→识别）与 arena 决策模型骨架。

use zero_ui_core::geometry::Point;

/// 手势事件。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GestureEvent {
    /// 单击（position）。
    Tap(Point),
    /// 拖拽/平移开始。
    DragStart(Point),
    DragUpdate(Point),
    DragEnd(Point),
}

/// arena 决策。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureDecision {
    /// 接受（声明胜出，其余识别器取消）。
    Accept,
    /// 拒绝（让出）。
    Reject,
    /// 仍未决。
    Pending,
}

/// Tap 识别器：press 后在移动阈值内 release → Tap。
#[derive(Debug, Clone)]
pub struct TapRecognizer {
    pressed_at: Option<Point>,
    move_tolerance: f32,
}

impl Default for TapRecognizer {
    fn default() -> TapRecognizer {
        TapRecognizer {
            pressed_at: None,
            move_tolerance: 8.0,
        }
    }
}

impl TapRecognizer {
    pub fn new(move_tolerance: f32) -> TapRecognizer {
        TapRecognizer {
            pressed_at: None,
            move_tolerance,
        }
    }

    /// 喂入指针事件（down/move/up 位置）；返回是否识别为 tap。
    pub fn on_down(&mut self, p: Point) {
        self.pressed_at = Some(p);
    }

    pub fn on_move(&mut self, p: Point) -> bool {
        // 移动超阈值 → 取消。
        if let Some(start) = self.pressed_at {
            let d = ((p.x - start.x).powi(2) + (p.y - start.y).powi(2)).sqrt();
            if d > self.move_tolerance {
                self.pressed_at = None;
                return false;
            }
        }
        true
    }

    pub fn on_up(&mut self, p: Point) -> Option<Point> {
        let tap = if let Some(start) = self.pressed_at {
            let d = ((p.x - start.x).powi(2) + (p.y - start.y).powi(2)).sqrt();
            if d <= self.move_tolerance { Some(start) } else { None }
        } else {
            None
        };
        self.pressed_at = None;
        tap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tap_recognized_within_tolerance() {
        let mut r = TapRecognizer::default();
        r.on_down(Point::new(10.0, 10.0));
        assert!(r.on_move(Point::new(12.0, 10.0))); // 小移动仍有效
        assert_eq!(r.on_up(Point::new(11.0, 11.0)), Some(Point::new(10.0, 10.0)));
    }

    #[test]
    fn tap_cancelled_by_large_move() {
        let mut r = TapRecognizer::default();
        r.on_down(Point::new(0.0, 0.0));
        assert!(!r.on_move(Point::new(50.0, 0.0))); // 超阈值 → 取消
        assert_eq!(r.on_up(Point::new(50.0, 0.0)), None);
    }

    #[test]
    fn custom_tolerance_and_idle_paths() {
        // 自定义容差构造器。
        let mut r = TapRecognizer::new(5.0);
        // 未 down 时 on_move → true（无起点，不取消）。
        assert!(r.on_move(Point::new(100.0, 100.0)));
        // 未 down 时 on_up → None。
        assert_eq!(r.on_up(Point::new(100.0, 100.0)), None);
        // down 后，移动 4（< 容差 5）仍有效，6（> 容差 5）取消。
        r.on_down(Point::new(0.0, 0.0));
        assert!(r.on_move(Point::new(4.0, 0.0)));
        assert!(!r.on_move(Point::new(6.0, 0.0)));
    }
}

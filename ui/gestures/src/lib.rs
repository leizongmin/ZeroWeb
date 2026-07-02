//! # zero-ui-gestures
//!
//! 手势识别（spec §8.4.1 `zero-ui-gestures` / FR-016 / IF-010 `GestureRecognizer` /
//! §8.4.1B 手势先入 arena 未被 chrome 消费才转发 WebView / §8.8 tap/drag/pinch/fling/arena 测）。
//!
//! 提供：[`PointerEvent`]（归一化指针事件，winit 类型不泄漏）+ [`GestureRecognizer`] trait
//! （IF-010）+ [`TapRecognizer`] / [`PanRecognizer`] / [`PinchRecognizer`] + [`GestureArena`]
//! （多识别器竞争仲裁）。
//!
//! **依赖方向**：winit-specific 类型不得出现在公共 API（spec §6.4）；宿主负责把 winit 事件归一为
//! [`PointerEvent`] 喂入。

pub mod arena;
pub mod event;
pub mod recognition;
pub mod recognizers;

pub use arena::GestureArena;
pub use event::{PointerEvent, PointerPhase};
pub use recognition::{Gesture, GestureRecognizer, GestureResult, PanPhase};
pub use recognizers::{PanRecognizer, PinchRecognizer, TapRecognizer};

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::geometry::Point;

    #[test]
    fn tap_recognized_within_tolerance() {
        let mut r = TapRecognizer::default();
        assert!(matches!(
            r.handle_pointer(&PointerEvent::down(0, Point::new(10.0, 10.0), 0)),
            GestureResult::Pending
        ));
        // 小移动仍 Pending（未超阈值）。
        assert!(matches!(
            r.handle_pointer(&PointerEvent::move_(0, Point::new(12.0, 10.0), 5)),
            GestureResult::Pending
        ));
        // 抬起在阈值内 → Won(Tap)。
        let res = r.handle_pointer(&PointerEvent::up(0, Point::new(11.0, 11.0), 10));
        assert!(matches!(res, GestureResult::Won(Gesture::Tap(_))));
    }

    #[test]
    fn tap_yields_on_large_move() {
        let mut r = TapRecognizer::default();
        r.handle_pointer(&PointerEvent::down(0, Point::new(0.0, 0.0), 0));
        // 大位移 → Yield（让 Pan 赢）。
        assert!(matches!(
            r.handle_pointer(&PointerEvent::move_(0, Point::new(50.0, 0.0), 5)),
            GestureResult::Yield
        ));
        // 此后抬起 → Yield（已不是 tap）。
        assert!(matches!(
            r.handle_pointer(&PointerEvent::up(0, Point::new(50.0, 0.0), 10)),
            GestureResult::Yield
        ));
    }

    #[test]
    fn tap_custom_tolerance_and_idle() {
        let mut r = TapRecognizer::new(5.0);
        // 未 down 时 Move → Pending。
        assert!(matches!(
            r.handle_pointer(&PointerEvent::move_(0, Point::new(100.0, 100.0), 0)),
            GestureResult::Pending
        ));
        // 未 down 时 Up → Yield（无起点）。
        assert!(matches!(
            r.handle_pointer(&PointerEvent::up(0, Point::new(100.0, 100.0), 1)),
            GestureResult::Yield
        ));
        r.handle_pointer(&PointerEvent::down(0, Point::new(0.0, 0.0), 2));
        assert!(matches!(
            r.handle_pointer(&PointerEvent::move_(0, Point::new(4.0, 0.0), 3)),
            GestureResult::Pending
        ));
        assert!(matches!(
            r.handle_pointer(&PointerEvent::move_(0, Point::new(6.0, 0.0), 4)),
            GestureResult::Yield
        ));
    }

    #[test]
    fn pan_start_update_end_below_fling() {
        let mut r = PanRecognizer::with_thresholds(10.0, 100.0); // 高 fling 阈值 → 不发 Fling
        r.handle_pointer(&PointerEvent::down(0, Point::new(0.0, 0.0), 0));
        // 未越阈值 → Pending。
        assert!(matches!(
            r.handle_pointer(&PointerEvent::move_(0, Point::new(5.0, 0.0), 10)),
            GestureResult::Pending
        ));
        // 越阈值 → Won(Pan Start)。
        assert!(matches!(
            r.handle_pointer(&PointerEvent::move_(0, Point::new(20.0, 0.0), 20)),
            GestureResult::Won(Gesture::Pan {
                phase: PanPhase::Start,
                ..
            })
        ));
        // 续移 → Update。
        assert!(matches!(
            r.handle_pointer(&PointerEvent::move_(0, Point::new(40.0, 0.0), 30)),
            GestureResult::Won(Gesture::Pan {
                phase: PanPhase::Update,
                ..
            })
        ));
        // 抬起（速度低）→ Pan End（非 Fling）。
        assert!(matches!(
            r.handle_pointer(&PointerEvent::up(0, Point::new(42.0, 0.0), 40)),
            GestureResult::Won(Gesture::Pan {
                phase: PanPhase::End,
                ..
            })
        ));
    }

    #[test]
    fn pan_fling_on_fast_release() {
        let mut r = PanRecognizer::with_thresholds(10.0, 2.0);
        r.handle_pointer(&PointerEvent::down(0, Point::new(0.0, 0.0), 0));
        r.handle_pointer(&PointerEvent::move_(0, Point::new(50.0, 0.0), 16));
        r.handle_pointer(&PointerEvent::move_(0, Point::new(500.0, 0.0), 20)); // 高速
        // 抬起：速度 = (520-500)/(24-20) = 5 px/ms > 2.0 → Fling。
        let res = r.handle_pointer(&PointerEvent::up(0, Point::new(520.0, 0.0), 24));
        match res {
            GestureResult::Won(Gesture::Fling { velocity }) => {
                assert!(velocity.x > 2.0, "fling velocity above threshold, got {}", velocity.x);
            }
            other => panic!("expected Fling, got {other:?}"),
        }
    }

    #[test]
    fn pinch_scales_by_distance_ratio() {
        let mut r = PinchRecognizer::new();
        // 双指按下：起始距离 = 100（(0,-50)-(0,50)）。
        r.handle_pointer(&PointerEvent::down(1, Point::new(0.0, -50.0), 0));
        // 仅一指 → 还不能 pinch（Pending）。
        assert!(matches!(
            r.handle_pointer(&PointerEvent::down(2, Point::new(0.0, 50.0), 1)),
            GestureResult::Pending
        ));
        // 张开到距离 200 → scale 2.0；pivot = 中点 (0,50)。
        let res = r.handle_pointer(&PointerEvent::move_(2, Point::new(0.0, 150.0), 3));
        match res {
            GestureResult::Won(Gesture::Pinch { scale, pivot }) => {
                assert!((scale - 2.0).abs() < 1e-4, "scale = 200/100 = 2.0, got {scale}");
                assert!((pivot.y - 50.0).abs() < 1e-4, "pivot midpoint y=50, got {}", pivot.y);
            }
            other => panic!("expected Pinch, got {other:?}"),
        }
        // 收拢：pointer1 仍在 (0,-50)，pointer2 到 (0,0) → 距离 50 → scale 0.5。
        let res = r.handle_pointer(&PointerEvent::move_(2, Point::new(0.0, 0.0), 4));
        match res {
            GestureResult::Won(Gesture::Pinch { scale, .. }) => {
                assert!((scale - 0.5).abs() < 1e-4, "scale = 50/100 = 0.5, got {scale}");
            }
            other => panic!("expected Pinch, got {other:?}"),
        }
    }
}

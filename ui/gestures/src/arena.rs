//! 手势 arena（spec §8.4.1B / §8.8：多个识别器竞争，首个 Won 胜出，其余取消）。
//!
//! arena 把指针流分发给所有成员识别器；竞争中首个 [`GestureResult::Won`] 声明胜出，其余被
//! `cancel`。胜出后该序列内后续事件**只路由给胜者**；所有指针抬起后序列结束、复位（下次 Down
//! 重新竞争）。这解决了 chrome/WebView 手势冲突（§8.4.1B：手势先入 arena，未被 chrome 消费才转发 WebView）。

use std::collections::HashSet;

use crate::event::{PointerEvent, PointerPhase};
use crate::recognition::{Gesture, GestureRecognizer, GestureResult};

/// 手势 arena：聚合多个识别器并仲裁。
#[derive(Default)]
pub struct GestureArena {
    members: Vec<Box<dyn GestureRecognizer>>,
    winner: Option<usize>,
    active_pointers: HashSet<u32>,
}

impl GestureArena {
    pub fn new() -> GestureArena {
        GestureArena::default()
    }

    /// 加入一个识别器（builder）。
    pub fn push<R: GestureRecognizer + 'static>(&mut self, recognizer: R) -> &mut Self {
        self.members.push(Box::new(recognizer));
        self
    }

    /// 当前成员数。
    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// 当前是否有胜者（用于断言/调试）。
    pub fn winner_index(&self) -> Option<usize> {
        self.winner
    }

    /// 路由一次指针事件，返回 arena 决出的手势（若有）。
    pub fn route(&mut self, event: &PointerEvent) -> Option<Gesture> {
        // 维护活动指针集合（用于判定序列结束）。
        match event.phase {
            PointerPhase::Down => {
                self.active_pointers.insert(event.id);
            }
            PointerPhase::Up => {
                self.active_pointers.remove(&event.id);
            }
            PointerPhase::Move => {}
        }

        let emitted = if let Some(w) = self.winner {
            // 序列已有胜者：独占路由。
            match self.members[w].handle_pointer(event) {
                GestureResult::Won(g) => Some(g),
                _ => None,
            }
        } else {
            // 竞争：路由给所有成员，首个 Won 胜出。
            let mut won: Option<(usize, Gesture)> = None;
            for (i, m) in self.members.iter_mut().enumerate() {
                if let GestureResult::Won(g) = m.handle_pointer(event) {
                    won = Some((i, g));
                    break;
                }
            }
            if let Some((win_idx, g)) = won {
                // 取消其余成员。
                for (j, m) in self.members.iter_mut().enumerate() {
                    if j != win_idx {
                        m.cancel();
                    }
                }
                self.winner = Some(win_idx);
                Some(g)
            } else {
                None
            }
        };

        // 所有指针抬起 → 序列结束 → 复位胜者（下次 Down 重新竞争）。
        if self.active_pointers.is_empty() {
            self.winner = None;
        }

        emitted
    }

    /// 强制复位 arena（清胜负、各成员 cancel）。
    pub fn reset(&mut self) {
        for m in &mut self.members {
            m.cancel();
        }
        self.winner = None;
        self.active_pointers.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::PointerEvent;
    use crate::recognition::Gesture;
    use crate::recognizers::{PanRecognizer, TapRecognizer};
    use zero_ui_core::geometry::Point;

    #[test]
    fn arena_tap_wins_on_quick_release_pan_yields() {
        // Tap + Pan 竞争：小位移快抬 → Tap 赢；Pan 在大位移时赢。
        let mut arena = GestureArena::new();
        arena.push(TapRecognizer::default());
        arena.push(PanRecognizer::new());

        // 按下 → 无胜者。
        let g = arena.route(&PointerEvent::down(0, Point::new(10.0, 10.0), 0));
        assert!(g.is_none());
        assert!(arena.winner_index().is_none());

        // 小移动 → 仍无胜者（未越 Pan 阈值，Tap 未取消）。
        assert!(
            arena
                .route(&PointerEvent::move_(0, Point::new(11.0, 11.0), 10))
                .is_none()
        );

        // 抬起（位移在 Tap 容差内）→ Tap 赢。
        let g = arena.route(&PointerEvent::up(0, Point::new(11.0, 11.0), 20));
        assert!(matches!(g, Some(Gesture::Tap(_))), "quick small release → Tap wins");
        // 抬起后序列结束 → 胜者复位。
        assert!(arena.winner_index().is_none());
    }

    #[test]
    fn arena_pan_wins_on_large_move_tap_cancelled() {
        let mut arena = GestureArena::new();
        arena.push(TapRecognizer::default());
        arena.push(PanRecognizer::new());

        assert!(arena.route(&PointerEvent::down(0, Point::new(0.0, 0.0), 0)).is_none());
        // 大位移 → Pan 赢（Start）；Tap 被 cancel。
        let g = arena
            .route(&PointerEvent::move_(0, Point::new(50.0, 0.0), 16))
            .expect("large move → Pan Start");
        assert!(matches!(
            g,
            Gesture::Pan {
                phase: crate::recognition::PanPhase::Start,
                ..
            }
        ));
        assert_eq!(arena.winner_index(), Some(1), "Pan (index 1) won");

        // 后续 Move 独占路由给 Pan（Tap 已取消，不再竞争）。
        let g2 = arena
            .route(&PointerEvent::move_(0, Point::new(80.0, 0.0), 32))
            .expect("Pan Update routed to winner");
        assert!(matches!(
            g2,
            Gesture::Pan {
                phase: crate::recognition::PanPhase::Update,
                ..
            }
        ));

        // 抬起 → Pan End（速度未超 fling 阈值）。
        let g3 = arena.route(&PointerEvent::up(0, Point::new(82.0, 0.0), 48));
        assert!(matches!(
            g3,
            Some(Gesture::Pan {
                phase: crate::recognition::PanPhase::End,
                ..
            })
        ));
    }

    #[test]
    fn arena_pan_fling_on_fast_release() {
        let mut arena = GestureArena::new();
        arena.push(PanRecognizer::new());
        arena.route(&PointerEvent::down(0, Point::new(0.0, 0.0), 0));
        arena.route(&PointerEvent::move_(0, Point::new(50.0, 0.0), 16));
        // 极快移动（大位移小 dt → 高速度）后抬起 → Fling。
        let g = arena.route(&PointerEvent::move_(0, Point::new(500.0, 0.0), 20));
        assert!(matches!(g, Some(Gesture::Pan { .. })));
        let up = arena.route(&PointerEvent::up(0, Point::new(520.0, 0.0), 24));
        // 速度 = (520-500)/(24-20) = 5 px/ms > fling 阈值 2.0 → Fling。
        assert!(
            matches!(up, Some(Gesture::Fling { .. })),
            "fast release → Fling, got {up:?}"
        );
    }

    #[test]
    fn arena_resets_between_sequences() {
        let mut arena = GestureArena::new();
        arena.push(TapRecognizer::default());
        // 序列 1：Tap。
        arena.route(&PointerEvent::down(0, Point::new(0.0, 0.0), 0));
        arena.route(&PointerEvent::up(0, Point::new(0.0, 0.0), 5));
        assert!(arena.winner_index().is_none(), "reset after first sequence");
        // 序列 2：再 Tap 应正常识别（成员已复位）。
        let g = arena.route(&PointerEvent::down(0, Point::new(5.0, 5.0), 100));
        assert!(g.is_none());
        let g = arena.route(&PointerEvent::up(0, Point::new(5.0, 5.0), 110));
        assert!(
            matches!(g, Some(Gesture::Tap(_))),
            "second sequence Tap recognized after reset"
        );
    }

    #[test]
    fn empty_arena_emits_nothing() {
        let mut arena = GestureArena::new();
        assert!(arena.route(&PointerEvent::down(0, Point::new(0.0, 0.0), 0)).is_none());
        assert!(arena.is_empty());
    }

    #[test]
    fn arena_calls_default_cancel_on_non_overriding_recognizer() {
        // 一个不覆盖 cancel 的识别器（用 trait 默认 cancel）；当另一成员抢先胜出时，
        // arena 调它的默认 cancel（覆盖 GestureRecognizer::cancel 默认实现）。
        struct Passive;
        impl GestureRecognizer for Passive {
            fn handle_pointer(&mut self, _event: &PointerEvent) -> GestureResult {
                GestureResult::Pending
            }
        }
        let mut arena = GestureArena::new();
        arena.push(Passive);
        arena.push(TapRecognizer::default());
        arena.route(&PointerEvent::down(0, Point::new(0.0, 0.0), 0));
        // 抬起 → Tap 胜出，Passive 被默认 cancel。
        let g = arena.route(&PointerEvent::up(0, Point::new(0.0, 0.0), 5));
        assert!(matches!(g, Some(Gesture::Tap(_))));
    }
}

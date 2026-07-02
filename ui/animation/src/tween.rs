//! Tween — 值域插值（spec §8.4.1 `tween.rs`）。

use crate::curve::{Curve, evaluate};

/// 在 `duration_ms` 内从 `from` 到 `to` 的标量 tween。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tween {
    pub from: f32,
    pub to: f32,
    pub duration_ms: i64,
    pub curve: Curve,
}

impl Tween {
    pub fn new(from: f32, to: f32, duration_ms: i64) -> Tween {
        Tween {
            from,
            to,
            duration_ms,
            curve: Curve::EaseInOut,
        }
    }

    /// 给定已过毫秒，返回当前插值；未开始返回 from，已结束返回 to。
    pub fn sample(&self, elapsed_ms: i64) -> f32 {
        if self.duration_ms <= 0 {
            return self.to;
        }
        let t = (elapsed_ms as f32 / self.duration_ms as f32).clamp(0.0, 1.0);
        let e = evaluate(self.curve, t);
        self.from + (self.to - self.from) * e
    }

    /// 是否已完成。
    pub fn is_done(&self, elapsed_ms: i64) -> bool {
        elapsed_ms >= self.duration_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tween_progresses_and_completes() {
        let tw = Tween::new(0.0, 100.0, 100);
        assert!((tw.sample(0).abs()) < 1e-6);
        assert!(tw.sample(50) > 0.0 && tw.sample(50) < 100.0);
        assert!((tw.sample(100) - 100.0).abs() < 1e-4);
        assert!(tw.is_done(100));
    }
}

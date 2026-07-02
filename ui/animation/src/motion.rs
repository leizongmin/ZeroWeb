//! Reduced-motion 偏好（spec §8.4.1B「动画遵循 reduced motion」/ §8.5 可访问性）。
//!
//! 用户启用系统「减少动态效果」时，动画应跳过过渡、直接到终态。本模块提供 [`MotionPreference`]
//! 与 tween/spring 的 reduced-motion 采样：`Reduced` 时 tween 立即返回 `to`，spring 立即贴合 target。

use crate::curve::Curve;
use crate::spring::Spring;
use crate::tween::Tween;

/// 动效偏好。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionPreference {
    /// 正常动画。
    Full,
    /// 减少动态效果（跳过过渡，直接到终态）。
    Reduced,
}

impl MotionPreference {
    pub fn is_reduced(self) -> bool {
        matches!(self, MotionPreference::Reduced)
    }
}

/// 用偏好采样 tween：`Reduced` 直接返回终态 `to`，否则正常插值。
pub fn sample_tween(tween: &Tween, elapsed_ms: i64, pref: MotionPreference) -> f32 {
    if pref.is_reduced() {
        tween.to
    } else {
        tween.sample(elapsed_ms)
    }
}

/// 用偏好驱动 spring：`Reduced` 直接贴合 target（位置=target，速度清零）。
/// 返回应展示的位置；调用方在 `Reduced` 时可据此跳过逐帧步进。
pub fn settle_spring(spring: &mut Spring, pref: MotionPreference) -> f32 {
    if pref.is_reduced() {
        spring.retarget(spring.target);
        // 直接贴合目标（位置=target，速度=0）。
        spring.launch(spring.target, 0.0);
        spring.target
    } else {
        spring.position()
    }
}

/// 是否应跑动画（用于 transition：Reduced 时跳过）。
pub fn should_animate(pref: MotionPreference, curve: Curve) -> bool {
    let _ = curve;
    !pref.is_reduced()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduced_tween_snaps_to_target() {
        let tw = Tween::new(0.0, 100.0, 1000);
        // Full → 中途插值。
        let mid_full = sample_tween(&tw, 500, MotionPreference::Full);
        assert!(mid_full > 0.0 && mid_full < 100.0);
        // Reduced → 任何时刻都直接到 100（含 t=0）。
        assert_eq!(sample_tween(&tw, 0, MotionPreference::Reduced), 100.0);
        assert_eq!(sample_tween(&tw, 500, MotionPreference::Reduced), 100.0);
    }

    #[test]
    fn reduced_spring_snaps_to_target() {
        let mut s = Spring::smooth(100.0);
        s.launch(0.0, 5000.0); // 有速度、不在目标
        let pos = settle_spring(&mut s, MotionPreference::Reduced);
        assert_eq!(pos, 100.0);
        assert!((s.position() - 100.0).abs() < 1e-6);
        assert!(s.velocity().abs() < 1e-6, "velocity cleared on reduced-motion snap");
    }

    #[test]
    fn full_spring_keeps_running_position() {
        let mut s = Spring::smooth(100.0);
        s.launch(0.0, 0.0);
        // Full 下 settle_spring 返回当前位置（未步进 → 起始位置）。
        assert_eq!(settle_spring(&mut s, MotionPreference::Full), 0.0);
    }

    #[test]
    fn should_animate_respects_preference() {
        assert!(should_animate(MotionPreference::Full, Curve::Linear));
        assert!(!should_animate(MotionPreference::Reduced, Curve::Linear));
    }
}

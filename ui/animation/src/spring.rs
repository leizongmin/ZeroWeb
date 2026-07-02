//! Spring — 弹簧物理动画（spec §8.4.1 `spring.rs` / §8.8 spring 测 / §8.4.1B sheet fling dismiss）。
//!
//! 与 [`crate::Tween`]（时长驱动、固定曲线）不同，spring 用质量-弹簧-阻尼物理积分到目标，
//! 天然带初速度（fling 接管）与过冲（bouncy）。用半隐式 Euler 数值积分（dt 步进）。

/// 弹簧预设（常见 UI 触感）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    /// 目标位置。
    pub target: f32,
    /// 刚度 k（越大越「硬」，到达越快）。
    pub stiffness: f32,
    /// 阻尼 c（越大过冲越小）。
    pub damping: f32,
    /// 质量。
    pub mass: f32,
    position: f32,
    velocity: f32,
}

impl Spring {
    /// 临界阻尼（无过冲，平滑到达；典型 UI 过渡）。
    pub fn smooth(target: f32) -> Spring {
        Spring {
            target,
            stiffness: 170.0,
            damping: 26.0,
            mass: 1.0,
            position: target,
            velocity: 0.0,
        }
    }

    /// 紧凑快速（更硬，更快到位）。
    pub fn snappy(target: f32) -> Spring {
        Spring {
            target,
            stiffness: 300.0,
            damping: 30.0,
            mass: 1.0,
            position: target,
            velocity: 0.0,
        }
    }

    /// 弹性（欠阻尼，有轻微过冲；sheet/卡片回弹）。
    pub fn bouncy(target: f32) -> Spring {
        Spring {
            target,
            stiffness: 200.0,
            damping: 14.0,
            mass: 1.0,
            position: target,
            velocity: 0.0,
        }
    }

    /// 自定义参数。
    pub fn with_params(target: f32, stiffness: f32, damping: f32, mass: f32) -> Spring {
        Spring {
            target,
            stiffness,
            damping,
            mass,
            position: target,
            velocity: 0.0,
        }
    }

    /// 当前位置 / 速度。
    pub fn position(&self) -> f32 {
        self.position
    }
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    /// 从 `position` 以 `initial_velocity` 开始，向当前 `target` 运动（fling 接管用）。
    pub fn launch(&mut self, position: f32, initial_velocity: f32) {
        self.position = position;
        self.velocity = initial_velocity;
    }

    /// 改目标，保留当前速度（惯性延续）。
    pub fn retarget(&mut self, target: f32) {
        self.target = target;
    }

    /// 步进 `dt_ms` 毫秒（半隐式 Euler，子步长拆分保证稳定性）。返回新位置。
    pub fn step(&mut self, dt_ms: i64) -> f32 {
        if dt_ms <= 0 {
            return self.position;
        }
        let mut dt = (dt_ms as f32) / 1000.0;
        // 子步长：单步 dt 不超过 1/120s，保证大 dt 数值稳定。
        const MAX_STEP: f32 = 1.0 / 120.0;
        while dt > 0.0 {
            let h = dt.min(MAX_STEP);
            dt -= h;
            // 弹簧力 F = -k*(x - target) - c*v；加速度 a = F/m。
            let force = -self.stiffness * (self.position - self.target) - self.damping * self.velocity;
            let accel = force / self.mass;
            // 半隐式 Euler：先更新速度，再用新速度更新位置（比显式 Euler 稳定）。
            self.velocity += accel * h;
            self.position += self.velocity * h;
        }
        self.position
    }

    /// 是否已稳定（位置接近目标且速度足够小）。
    pub fn is_settled(&self, epsilon: f32) -> bool {
        (self.position - self.target).abs() < epsilon && self.velocity.abs() < epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooth_spring_settles_to_target_without_overshoot() {
        // §8.8 spring 测：临界阻尼从 0→100，单调逼近、最终稳定在 100（无显著过冲）。
        let mut s = Spring::smooth(100.0);
        s.launch(0.0, 0.0);
        let mut max_pos = 0.0_f32;
        for _ in 0..2000 {
            let p = s.step(16);
            max_pos = max_pos.max(p);
        }
        assert!(
            (s.position() - 100.0).abs() < 0.5,
            "settled near target, got {}",
            s.position()
        );
        assert!(s.is_settled(1.0));
        // 临界阻尼不应明显过冲（≤ 目标 + 小容差）。
        assert!(
            max_pos <= 100.0 + 1.0,
            "smooth spring should not overshoot much, max={max_pos}"
        );
    }

    #[test]
    fn bouncy_spring_overshoots_then_settles() {
        // 欠阻尼弹簧会过冲（超过目标）再回弹稳定。
        let mut s = Spring::bouncy(100.0);
        s.launch(0.0, 0.0);
        let mut max_pos = 0.0_f32;
        for _ in 0..4000 {
            let p = s.step(16);
            max_pos = max_pos.max(p);
        }
        assert!(max_pos > 100.0, "bouncy spring should overshoot target, max={max_pos}");
        assert!(
            (s.position() - 100.0).abs() < 0.5,
            "eventually settles, got {}",
            s.position()
        );
    }

    #[test]
    fn spring_carries_initial_velocity_fling() {
        // §8.4.1B fling：松手后以初速度继续，spring 把它带到目标。
        let mut s = Spring::smooth(0.0);
        s.launch(0.0, 5000.0); // 大初速度
        s.step(16);
        // 带初速度 → 先朝速度方向冲过目标，再被弹簧拉回。
        assert!(s.position() > 0.0, "initial velocity moves position");
        for _ in 0..4000 {
            s.step(16);
        }
        assert!((s.position() - 0.0).abs() < 0.5, "settles back to target 0");
    }

    #[test]
    fn retarget_keeps_velocity() {
        let mut s = Spring::smooth(0.0);
        s.launch(0.0, 1000.0);
        s.step(16);
        let v_before = s.velocity();
        s.retarget(50.0);
        assert_eq!(s.target, 50.0);
        // 速度保留（惯性延续）。
        assert!((s.velocity() - v_before).abs() < 1e-4);
    }

    #[test]
    fn step_zero_dt_is_noop() {
        let mut s = Spring::smooth(10.0);
        s.launch(0.0, 0.0);
        assert_eq!(s.step(0), 0.0, "dt<=0 leaves position unchanged");
    }

    #[test]
    fn large_dt_stable_due_to_substeps() {
        // 大 dt（如掉帧 500ms）不应数值爆炸；子步长拆分保证稳定。
        let mut s = Spring::smooth(100.0);
        s.launch(0.0, 0.0);
        let p = s.step(500);
        assert!(p.is_finite(), "no NaN/inf from large dt");
        assert!(p.abs() < 1e6, "bounded, got {p}");
    }

    #[test]
    fn presets_and_custom_params_reach_target() {
        // snappy 预设更快到位（更硬刚度）。
        let mut snappy = Spring::snappy(100.0);
        snappy.launch(0.0, 0.0);
        for _ in 0..2000 {
            snappy.step(16);
        }
        assert!((snappy.position() - 100.0).abs() < 0.5);

        // 自定义参数也到达目标。
        let mut custom = Spring::with_params(50.0, 120.0, 20.0, 1.0);
        custom.launch(0.0, 0.0);
        for _ in 0..2000 {
            custom.step(16);
        }
        assert!(
            custom.is_settled(0.5),
            "custom spring settles, pos={}",
            custom.position()
        );
    }

    #[test]
    fn is_settled_false_while_moving() {
        let mut s = Spring::smooth(100.0);
        s.launch(0.0, 5000.0); // 大速度
        s.step(16);
        // 移动中、远离目标 → 未稳定。
        assert!(!s.is_settled(1.0));
    }
}

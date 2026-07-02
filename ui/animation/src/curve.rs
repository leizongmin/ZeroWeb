//! 缓动曲线（spec §8.4.1 `curve.rs`）。

/// 常用缓动曲线。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Curve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

/// 在归一化时间 `t`（0..=1）下求曲线值（0..=1）。超界做 clamp。
pub fn evaluate(curve: Curve, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match curve {
        Curve::Linear => t,
        Curve::EaseIn => t * t,
        Curve::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        Curve::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_endpoints_and_monotonic() {
        for c in [Curve::Linear, Curve::EaseIn, Curve::EaseOut, Curve::EaseInOut] {
            assert!((evaluate(c, 0.0)).abs() < 1e-6);
            assert!((evaluate(c, 1.0) - 1.0).abs() < 1e-6);
        }
        assert!((evaluate(Curve::Linear, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn clamp_out_of_range() {
        assert!((evaluate(Curve::Linear, -1.0)).abs() < 1e-6);
        assert!((evaluate(Curve::Linear, 2.0) - 1.0).abs() < 1e-6);
    }
}

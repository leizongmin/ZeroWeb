//! # zero-ui-design-system
//!
//! 设计系统（spec §8.4.1 `zero-ui-design-system` / FR-016 / DC-15 首个风格包 Zero default）。
//!
//! M1 提供 Density、MotionTokens（尊重 reduced motion）、ComponentVariant 骨架；
//! 首个风格包 `zero_default()` 作为后续 M2/M4 风格包（Fluent/Cupertino/Material）的基础。

use serde::{Deserialize, Serialize};

/// 信息密度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Density {
    Compact,
    #[default]
    Comfortable,
}

impl Density {
    /// density → 间距倍率。
    pub fn spacing_factor(self) -> f32 {
        match self {
            Density::Compact => 0.8,
            Density::Comfortable => 1.0,
        }
    }
}

/// 动效 token。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MotionTokens {
    pub fast_ms: u32,
    pub normal_ms: u32,
    pub slow_ms: u32,
    pub reduced_motion: bool,
}

impl Default for MotionTokens {
    fn default() -> MotionTokens {
        MotionTokens {
            fast_ms: 120,
            normal_ms: 220,
            slow_ms: 360,
            reduced_motion: false,
        }
    }
}

impl MotionTokens {
    /// reduced motion 启用时，所有时长压缩到 ~0（瞬切）。
    pub fn effective_ms(self, kind: MotionDuration) -> u32 {
        if self.reduced_motion {
            0
        } else {
            match kind {
                MotionDuration::Fast => self.fast_ms,
                MotionDuration::Normal => self.normal_ms,
                MotionDuration::Slow => self.slow_ms,
            }
        }
    }
}

pub enum MotionDuration {
    Fast,
    Normal,
    Slow,
}

/// 组件变体描述（风格包内组件的命名变体）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentVariant {
    pub component: String,
    pub variant: String,
}

/// Zero default 风格包（DC-15 首个风格包）。
pub fn zero_default() -> (Density, MotionTokens) {
    (Density::Comfortable, MotionTokens::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_spacing() {
        assert_eq!(Density::Compact.spacing_factor(), 0.8);
        assert_eq!(Density::Comfortable.spacing_factor(), 1.0);
    }

    #[test]
    fn reduced_motion_zeroes_duration() {
        let mut m = MotionTokens::default();
        assert_eq!(m.effective_ms(MotionDuration::Normal), 220);
        m.reduced_motion = true;
        assert_eq!(m.effective_ms(MotionDuration::Normal), 0);
    }

    #[test]
    fn zero_default_pack() {
        let (d, m) = zero_default();
        assert_eq!(d, Density::Comfortable);
        assert_eq!(m.normal_ms, 220);
    }
}

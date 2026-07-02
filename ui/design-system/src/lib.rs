//! # zero-ui-design-system
//!
//! 设计系统（spec §8.4.1 `zero-ui-design-system` / FR-016 / DC-15 首个风格包 Zero default）。
//!
//! 提供 [`StylePack`]（风格包 = 信息密度 + 动效 token + 间距/圆角/字号 token 聚合）；
//! [`zero_default`] 是首个风格包（DC-15 终局需求），后续 Fluent/Cupertino/Material 风格包以之为基。
//!
//! **与主题的边界**：本 crate 只管几何/动效/字号的密度 token；**颜色** 由 `ui/core::theme`
//! 的 semantic token（按 light/dark 主题）提供，不在风格包内重复（spec DC-5）。

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

/// 间距 token（密度缩放）：xs/sm/md/lg/xl/xxl，单位逻辑像素。
///
/// 基线步长取自设计规范（4 / 8 / 12 / 16 / 24 / 32），实际值 = 基线 × density 缩放。
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SpacingTokens {
    pub density: Density,
}

impl SpacingTokens {
    fn s(self, base: f32) -> f32 {
        base * self.density.spacing_factor()
    }
    pub fn xs(self) -> f32 {
        self.s(4.0)
    }
    pub fn sm(self) -> f32 {
        self.s(8.0)
    }
    pub fn md(self) -> f32 {
        self.s(12.0)
    }
    pub fn lg(self) -> f32 {
        self.s(16.0)
    }
    pub fn xl(self) -> f32 {
        self.s(24.0)
    }
    pub fn xxl(self) -> f32 {
        self.s(32.0)
    }
}

/// 圆角 token（逻辑像素）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RadiusTokens {
    pub none: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    /// 完全圆角（pill / 圆形）——用大数表示「取短边一半」语义。
    pub full: f32,
}

impl Default for RadiusTokens {
    fn default() -> RadiusTokens {
        RadiusTokens {
            none: 0.0,
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            full: 9999.0,
        }
    }
}

/// 字号刻度（逻辑像素，不受密度缩放——文字缩放由平台 text-scale 负责）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TypographyScale {
    pub caption: f32,
    pub body: f32,
    pub title: f32,
    pub headline: f32,
}

impl Default for TypographyScale {
    fn default() -> TypographyScale {
        TypographyScale {
            caption: 11.0,
            body: 13.0,
            title: 16.0,
            headline: 20.0,
        }
    }
}

/// 组件变体描述（风格包内组件的命名变体）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentVariant {
    pub component: String,
    pub variant: String,
}

/// 风格包：聚合信息密度 + 动效 + 间距/圆角/字号 token（DC-15 首个风格包 Zero default 用）。
///
/// 不含颜色（颜色来自 `ui/core::theme::SemanticTokens`，按 light/dark 主题切换）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StylePack {
    pub name: String,
    pub density: Density,
    pub motion: MotionTokens,
    pub spacing: SpacingTokens,
    pub radius: RadiusTokens,
    pub typography: TypographyScale,
}

impl StylePack {
    /// 推导间距 token（便捷：按当前 density 缩放）。
    pub fn spacing_value(&self, step: SpacingStep) -> f32 {
        let b = match step {
            SpacingStep::Xs => 4.0,
            SpacingStep::Sm => 8.0,
            SpacingStep::Md => 12.0,
            SpacingStep::Lg => 16.0,
            SpacingStep::Xl => 24.0,
            SpacingStep::Xxl => 32.0,
        };
        b * self.density.spacing_factor()
    }
}

/// 间距刻度枚举（配合 [`StylePack::spacing_value`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpacingStep {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
}

/// Zero default 风格包（DC-15 首个风格包）：Comfortable 密度 + 默认动效 + 标准间距/圆角/字号。
pub fn zero_default() -> StylePack {
    StylePack {
        name: "zero.default".to_string(),
        density: Density::Comfortable,
        motion: MotionTokens::default(),
        spacing: SpacingTokens::default(),
        radius: RadiusTokens::default(),
        typography: TypographyScale::default(),
    }
}

/// 紧凑风格包（小窗 / 信息密集布局）：Compact 密度（间距 ×0.8）。
pub fn zero_compact() -> StylePack {
    StylePack {
        name: "zero.compact".to_string(),
        density: Density::Compact,
        ..zero_default()
    }
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
        assert_eq!(m.effective_ms(MotionDuration::Slow), 0);
    }

    #[test]
    fn spacing_tokens_scale_with_density() {
        let comfortable = SpacingTokens {
            density: Density::Comfortable,
        };
        assert_eq!(comfortable.sm(), 8.0);
        assert_eq!(comfortable.lg(), 16.0);
        let compact = SpacingTokens {
            density: Density::Compact,
        };
        assert!((compact.sm() - 6.4).abs() < 1e-4, "compact sm = 8*0.8");
        assert!((compact.xl() - 19.2).abs() < 1e-4, "compact xl = 24*0.8");
    }

    #[test]
    fn radius_and_typography_defaults() {
        let r = RadiusTokens::default();
        assert_eq!(r.none, 0.0);
        assert_eq!(r.sm, 4.0);
        assert!(r.full > 1000.0);
        let t = TypographyScale::default();
        assert!(t.caption < t.body && t.body < t.title && t.title < t.headline);
    }

    #[test]
    fn zero_default_pack_is_concrete_style_pack() {
        // DC-15 首个风格包：StylePack 聚合 density+motion+spacing+radius+typography。
        let pack = zero_default();
        assert_eq!(pack.name, "zero.default");
        assert_eq!(pack.density, Density::Comfortable);
        assert_eq!(pack.motion.normal_ms, 220);
        // 间距便捷 API。
        assert_eq!(pack.spacing_value(SpacingStep::Sm), 8.0);
        assert_eq!(pack.spacing_value(SpacingStep::Xl), 24.0);
    }

    #[test]
    fn zero_compact_pack_scales_spacing() {
        let pack = zero_compact();
        assert_eq!(pack.density, Density::Compact);
        assert!((pack.spacing_value(SpacingStep::Lg) - 12.8).abs() < 1e-4, "16*0.8");
        // 其余继承 zero default。
        assert_eq!(pack.typography, TypographyScale::default());
    }

    #[test]
    fn style_pack_serde_roundtrip() {
        // 风格包可序列化（便于主题包/资源打包）。
        let pack = zero_default();
        let json = serde_json::to_string(&pack).unwrap();
        let back: StylePack = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pack);
    }
}

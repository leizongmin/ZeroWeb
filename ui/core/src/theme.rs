//! 主题系统（spec FR-007 / IF-003 / DC-5）。
//!
//! 组件只消费 **semantic token**（如 `surface`、`on_surface`、`primary`），不硬编码浏览器色值。
//! 系统主题变化时 `ThemeResolver` 生成新 Theme 并发 `ThemeChanged`；字体/间距不变时仅触发
//! `needs_paint`（spec FR-007 / DC-5 关键不变量）。

use crate::invalidation::InvalidationFlags;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// RGBA 颜色（分量 0.0..=1.0，线性或 sRGB 由绘制后端约定；此处仅承载值）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
}

/// 主题标识。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThemeId(pub CompactString);

impl ThemeId {
    pub fn new(name: &str) -> ThemeId {
        ThemeId(CompactString::new(name))
    }
}

/// 用户偏好（spec IF-003）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorSchemePreference {
    System,
    Light,
    Dark,
    Custom(ThemeId),
}

/// 解析后的实际配色方案（spec IF-003）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolvedColorScheme {
    Light,
    Dark,
    HighContrastLight,
    HighContrastDark,
}

impl ResolvedColorScheme {
    pub fn is_dark(self) -> bool {
        matches!(self, ResolvedColorScheme::Dark | ResolvedColorScheme::HighContrastDark)
    }
    pub fn is_high_contrast(self) -> bool {
        matches!(
            self,
            ResolvedColorScheme::HighContrastLight | ResolvedColorScheme::HighContrastDark
        )
    }
}

/// 系统主题快照（由 `ui/runtime::platform` 探测）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemThemeSnapshot {
    pub system_scheme: ResolvedColorScheme,
    pub high_contrast: bool,
}

/// 语义 token 集合（组件消费层）。新增 token 集中在此。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SemanticTokens {
    /// 窗口/页面底色。
    pub background: Color,
    /// 默认文本/前景。
    pub on_background: Color,
    /// 控件表面（卡片、toolbar）。
    pub surface: Color,
    pub on_surface: Color,
    /// 主品牌色（强调按钮、选中态）。
    pub primary: Color,
    pub on_primary: Color,
    /// 危险/错误。
    pub error: Color,
    pub on_error: Color,
}

impl SemanticTokens {
    /// 浅色基线 token。
    pub fn light() -> SemanticTokens {
        SemanticTokens {
            background: Color::WHITE,
            on_background: Color::BLACK,
            surface: Color::rgb(0.96, 0.96, 0.96),
            on_surface: Color::rgb(0.1, 0.1, 0.1),
            primary: Color::rgb(0.13, 0.58, 0.95),
            on_primary: Color::WHITE,
            error: Color::rgb(0.86, 0.21, 0.27),
            on_error: Color::WHITE,
        }
    }
    /// 深色基线 token。
    pub fn dark() -> SemanticTokens {
        SemanticTokens {
            background: Color::rgb(0.12, 0.12, 0.13),
            on_background: Color::rgb(0.92, 0.92, 0.92),
            surface: Color::rgb(0.18, 0.18, 0.2),
            on_surface: Color::rgb(0.9, 0.9, 0.9),
            primary: Color::rgb(0.4, 0.73, 1.0),
            on_primary: Color::BLACK,
            error: Color::rgb(0.95, 0.5, 0.5),
            on_error: Color::BLACK,
        }
    }
}

/// 字体排印 token（字号/字重/行高）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TypographyTokens {
    pub body_size_px: f32,
    pub body_weight: u16,
    pub body_line_height: f32,
    pub heading_size_px: f32,
    pub heading_weight: u16,
}

impl TypographyTokens {
    pub fn default_typography() -> TypographyTokens {
        TypographyTokens {
            body_size_px: 14.0,
            body_weight: 400,
            body_line_height: 1.4,
            heading_size_px: 18.0,
            heading_weight: 600,
        }
    }
}

/// 间距栅格 token。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpacingTokens {
    pub unit: f32, // 基础步长（如 4px）
}

impl SpacingTokens {
    pub fn default_spacing() -> SpacingTokens {
        SpacingTokens { unit: 4.0 }
    }
}

/// 圆角 token。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RadiusTokens {
    pub small: f32,
    pub medium: f32,
    pub large: f32,
}

impl RadiusTokens {
    pub fn default_radius() -> RadiusTokens {
        RadiusTokens {
            small: 2.0,
            medium: 6.0,
            large: 12.0,
        }
    }
}

/// 阴影 token（简化：两层 elevation）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShadowTokens {
    pub elevation1: f32,
    pub elevation2: f32,
}

impl ShadowTokens {
    pub fn default_shadow() -> ShadowTokens {
        ShadowTokens {
            elevation1: 2.0,
            elevation2: 8.0,
        }
    }
}

/// 原始调色板（自定义主题覆盖用）。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ColorPalette {
    pub primary_override: Option<Color>,
    pub surface_override: Option<Color>,
}

/// 解析后的主题（spec IF-003 `Theme`）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    pub id: ThemeId,
    pub name: String,
    pub scheme: ResolvedColorScheme,
    pub tokens: SemanticTokens,
    pub palette: ColorPalette,
    pub typography: TypographyTokens,
    pub spacing: SpacingTokens,
    pub radius: RadiusTokens,
    pub shadow: ShadowTokens,
}

/// 系统主题变化时由 runtime 发出的事件（spec FR-007 `ThemeChanged`）。
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeChanged {
    pub new_theme: Theme,
    /// 相对旧主题，需要触发哪些失效。
    pub invalidation: InvalidationFlags,
}

impl ThemeChanged {
    /// 仅主题色变化（字体/间距/圆角不变）→ 只 needs_paint。
    pub fn paint_only(new_theme: Theme) -> ThemeChanged {
        ThemeChanged {
            new_theme,
            invalidation: InvalidationFlags::NEEDS_PAINT,
        }
    }
}

/// 比较两个主题，得出需要触发的失效。
///
/// - tokens/palette/shadow 变化 → `needs_paint`。
/// - typography/spacing 变化 → 叠加 `needs_layout`（影响测量与布局）。
pub fn diff_invalidation(old: &Theme, new: &Theme) -> InvalidationFlags {
    let mut flags = InvalidationFlags::CLEAN;
    if old.tokens != new.tokens || old.palette != new.palette || old.shadow != new.shadow {
        flags |= InvalidationFlags::NEEDS_PAINT;
    }
    if old.typography != new.typography || old.spacing != new.spacing {
        flags |= InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT;
    }
    flags
}

/// 主题解析器：偏好 + 系统快照 → 解析方案 → Theme（spec FR-007 `ThemeResolver`）。
#[derive(Debug, Clone)]
pub struct ThemeResolver;

impl ThemeResolver {
    pub fn resolve_scheme(preference: &ColorSchemePreference, system: SystemThemeSnapshot) -> ResolvedColorScheme {
        match preference {
            ColorSchemePreference::System => system.system_scheme,
            ColorSchemePreference::Light => {
                if system.high_contrast {
                    ResolvedColorScheme::HighContrastLight
                } else {
                    ResolvedColorScheme::Light
                }
            }
            ColorSchemePreference::Dark => {
                if system.high_contrast {
                    ResolvedColorScheme::HighContrastDark
                } else {
                    ResolvedColorScheme::Dark
                }
            }
            // 自定义主题的方案在 Theme 加载时已确定；此处退回 system 兜底。
            ColorSchemePreference::Custom(_) => system.system_scheme,
        }
    }

    /// 由解析方案生成基线主题。自定义 palette 覆盖会替换对应 semantic token。
    pub fn build_theme(id: ThemeId, name: &str, scheme: ResolvedColorScheme, palette: ColorPalette) -> Theme {
        let mut tokens = if scheme.is_dark() {
            SemanticTokens::dark()
        } else {
            SemanticTokens::light()
        };
        if let Some(primary) = palette.primary_override {
            tokens.primary = primary;
        }
        if let Some(surface) = palette.surface_override {
            tokens.surface = surface;
        }
        Theme {
            id,
            name: name.to_string(),
            scheme,
            tokens,
            palette,
            typography: TypographyTokens::default_typography(),
            spacing: SpacingTokens::default_spacing(),
            radius: RadiusTokens::default_radius(),
            shadow: ShadowTokens::default_shadow(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_preference_follows_system_snapshot() {
        let dark_sys = SystemThemeSnapshot {
            system_scheme: ResolvedColorScheme::Dark,
            high_contrast: false,
        };
        assert_eq!(
            ThemeResolver::resolve_scheme(&ColorSchemePreference::System, dark_sys),
            ResolvedColorScheme::Dark
        );
        let hc_sys = SystemThemeSnapshot {
            system_scheme: ResolvedColorScheme::Light,
            high_contrast: true,
        };
        assert_eq!(
            ThemeResolver::resolve_scheme(&ColorSchemePreference::Light, hc_sys),
            ResolvedColorScheme::HighContrastLight
        );
    }

    #[test]
    fn color_only_change_is_paint_only_invalidation() {
        // DC-5：主题色切换、字体/间距不变 → 仅 needs_paint。
        let t1 = ThemeResolver::build_theme(
            ThemeId::new("zero"),
            "Zero",
            ResolvedColorScheme::Light,
            ColorPalette::default(),
        );
        let t2 = ThemeResolver::build_theme(
            ThemeId::new("zero"),
            "Zero",
            ResolvedColorScheme::Dark,
            ColorPalette::default(),
        );
        let inv = diff_invalidation(&t1, &t2);
        assert!(inv.contains(InvalidationFlags::NEEDS_PAINT));
        assert!(
            !inv.contains(InvalidationFlags::NEEDS_LAYOUT),
            "color-only theme change must not request layout"
        );
    }

    #[test]
    fn typography_change_requests_layout() {
        let t1 = ThemeResolver::build_theme(
            ThemeId::new("zero"),
            "Zero",
            ResolvedColorScheme::Light,
            ColorPalette::default(),
        );
        let mut t2 = t1.clone();
        t2.typography.body_size_px = 16.0; // 字号变化 → 影响测量
        let inv = diff_invalidation(&t1, &t2);
        assert!(inv.contains(InvalidationFlags::NEEDS_LAYOUT));
        assert!(inv.contains(InvalidationFlags::NEEDS_PAINT));
    }

    #[test]
    fn custom_palette_overrides_semantic_token() {
        let brand = Color::rgb(0.2, 0.8, 0.4);
        let palette = ColorPalette {
            primary_override: Some(brand),
            surface_override: None,
        };
        let theme = ThemeResolver::build_theme(ThemeId::new("brand"), "Brand", ResolvedColorScheme::Light, palette);
        assert_eq!(theme.tokens.primary, brand);
    }
}

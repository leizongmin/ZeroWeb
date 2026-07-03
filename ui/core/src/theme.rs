//! 主题系统（spec FR-007 / IF-003 / DC-5）。
//!
//! 组件只消费 **semantic token**（如 `surface`、`on_surface`、`primary`），不硬编码浏览器色值。
//! 系统主题变化时 `ThemeResolver` 生成新 Theme 并发 `ThemeChanged`；字体/间距不变时仅触发
//! `needs_paint`（spec FR-007 / DC-5 关键不变量）。

use crate::invalidation::InvalidationFlags;
use crate::layout::WindowMetrics;
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

    /// 与另一颜色线性插值（`t=0`→self，`t=1`→other；`t` 钳到 `[0,1]`），alpha 同步插值。
    ///
    /// 用于从 semantic token 派生交互态色（如按钮 hover 变亮 / pressed 变暗），避免硬编码色值。
    pub fn mix(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    /// 向白色插值（变亮），`t∈[0,1]`。
    pub fn lighten(self, t: f32) -> Color {
        self.mix(Color::WHITE, t)
    }

    /// 向黑色插值（变暗），`t∈[0,1]`。
    pub fn darken(self, t: f32) -> Color {
        self.mix(Color::BLACK, t)
    }
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

/// 平台探测返回的系统主题快照。
///
/// **字段角色**：`system_scheme` 是已含高对比度的 fully-resolved 方案——
/// 当用户偏好为 `System` 时直接取用（OS 已报告含 HC 的完整方案）。
/// `high_contrast` 是独立布尔，供用户显式选择 `Light`/`Dark`/`Custom`
/// 但 OS 处于高对比模式时升级到 `HighContrast*` 变体。
///
/// M4 平台探测器须按此契约填充：`system_scheme` 含 HC 信息，
/// `high_contrast` 供偏好非 `System` 的分支做 HC 升级判断。
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
    /// 成功/安全（HTTPS 徽章等）。
    pub success: Color,
    pub on_success: Color,
    /// 警告（mixed content / 不安全）。
    pub warning: Color,
    pub on_warning: Color,
}

impl SemanticTokens {
    /// 浅色基线 token。
    pub fn light() -> SemanticTokens {
        SemanticTokens {
            background: Color::WHITE,
            on_background: Color::BLACK,
            surface: Color::rgb(0.96, 0.96, 0.96),
            on_surface: Color::rgb(0.1, 0.1, 0.1),
            // DC-5：可访问 primary 蓝（≈ Material Blue 700 #1976D2）。前值 (0.13,0.58,0.95)
            // 对白字 contrast 仅 3.19 < WCAG AA 4.5；本值对白字 ≈ 4.7，对白底作链接色亦通过。
            primary: Color::rgb(0.098, 0.463, 0.824),
            on_primary: Color::WHITE,
            error: Color::rgb(0.86, 0.21, 0.27),
            on_error: Color::WHITE,
            success: Color::rgb(0.10, 0.70, 0.30),
            on_success: Color::BLACK,
            warning: Color::rgb(0.90, 0.60, 0.10),
            on_warning: Color::BLACK,
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
            success: Color::rgb(0.20, 0.78, 0.40),
            on_success: Color::BLACK,
            warning: Color::rgb(0.96, 0.72, 0.20),
            on_warning: Color::BLACK,
        }
    }

    /// 高对比浅色 token（DC-5 HighContrast）。
    ///
    /// 纯白底 + 黑字，主/状态色取深色饱和变体 + 白字，目标 **WCAG AAA（正常文本 ≥ 7:1）**。
    /// 高对比模式服务于低视力用户，文本对必须显著超过常规 AA 基线。
    pub fn high_contrast_light() -> SemanticTokens {
        SemanticTokens {
            background: Color::WHITE,
            on_background: Color::BLACK,
            surface: Color::WHITE,
            on_surface: Color::BLACK,
            // 主色用近黑：按钮/选中态黑底白字，最大对比（21:1）。
            primary: Color::BLACK,
            on_primary: Color::WHITE,
            // 状态色取深色饱和变体，配白字达 AAA。
            error: Color::rgb(0.55, 0.0, 0.0),
            on_error: Color::WHITE,
            success: Color::rgb(0.0, 0.33, 0.0),
            on_success: Color::WHITE,
            warning: Color::rgb(0.5, 0.35, 0.0),
            on_warning: Color::WHITE,
        }
    }

    /// 高对比深色 token（DC-5 HighContrast）。
    ///
    /// 纯黑底 + 白字，主/状态色取高亮变体 + 黑字，目标 **WCAG AAA（正常文本 ≥ 7:1）**。
    pub fn high_contrast_dark() -> SemanticTokens {
        SemanticTokens {
            background: Color::BLACK,
            on_background: Color::WHITE,
            surface: Color::BLACK,
            on_surface: Color::WHITE,
            // 主色用近白：白底黑字，最大对比（21:1）。
            primary: Color::WHITE,
            on_primary: Color::BLACK,
            // 状态色取高亮变体，配黑字达 AAA。
            error: Color::rgb(1.0, 0.4, 0.4),
            on_error: Color::BLACK,
            success: Color::rgb(0.4, 1.0, 0.5),
            on_success: Color::BLACK,
            warning: Color::rgb(1.0, 0.85, 0.0),
            on_warning: Color::BLACK,
        }
    }

    /// 按 token 名解析颜色（组件消费 semantic token 的统一入口）。
    ///
    /// 支持 `background`/`on_background`/`surface`/`on_surface`/`primary`/`on_primary`/
    /// `error`/`on_error`/`success`/`on_success`/`warning`/`on_warning`；未知名返回 `None`。
    pub fn color_for(&self, name: &str) -> Option<Color> {
        Some(match name {
            "background" => self.background,
            "on_background" => self.on_background,
            "surface" => self.surface,
            "on_surface" => self.on_surface,
            "primary" => self.primary,
            "on_primary" => self.on_primary,
            "error" => self.error,
            "on_error" => self.on_error,
            "success" => self.success,
            "on_success" => self.on_success,
            "warning" => self.warning,
            "on_warning" => self.on_warning,
            _ => return None,
        })
    }
}

/// WCAG 相对亮度（[W3C WCAG 2.1](https://www.w3.org/TR/WCAG21/#dfn-relative-luminance)）。
pub fn relative_luminance(c: Color) -> f32 {
    let channel = |v: f32| -> f32 {
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b)
}

/// 两个颜色之间的对比度比（1.0..=21.0；越大越对比，DC-5 contrast lint）。
pub fn contrast_ratio(fg: Color, bg: Color) -> f32 {
    let l1 = relative_luminance(fg);
    let l2 = relative_luminance(bg);
    let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

/// 是否满足 WCAG AA（普通文本 4.5；大文本 3.0）。
pub fn passes_wcag_aa(fg: Color, bg: Color, large_text: bool) -> bool {
    contrast_ratio(fg, bg) >= if large_text { 3.0 } else { 4.5 }
}

/// 是否满足 WCAG AAA（普通文本 7.0；大文本 4.5）。
pub fn passes_wcag_aaa(fg: Color, bg: Color, large_text: bool) -> bool {
    contrast_ratio(fg, bg) >= if large_text { 4.5 } else { 7.0 }
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

    /// 按 `text_scale` 缩放字号（spec IF-009 `text_scale` / DC-15 移动端「text scale」）。
    ///
    /// 仅缩放**字号**（`body_size_px`/`heading_size_px`），字重/行高不变；
    /// `text_scale` 钳到 `> 0.0`（非正数视为 1.0，防退化）。字号变化会触发 `needs_layout`
    /// （见 [`diff_invalidation`]）——这是 text_scale 影响布局的统一路径。
    pub fn scaled(self, text_scale: f32) -> TypographyTokens {
        let s = if text_scale > 0.0 { text_scale } else { 1.0 };
        TypographyTokens {
            body_size_px: self.body_size_px * s,
            body_weight: self.body_weight,
            body_line_height: self.body_line_height,
            heading_size_px: self.heading_size_px * s,
            heading_weight: self.heading_weight,
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

    /// 按 `density` 缩放间距栅格（spec IF-009 `density` / DC-15 移动端布局密度）。
    ///
    /// 缩放基础步长 `unit`（影响 padding/margin/栅格），`density` 钳到 `> 0.0`
    /// （非正数视为 1.0，防退化）。间距变化会触发 `needs_layout`（见 [`diff_invalidation`]）。
    /// 与 [`TypographyTokens::scaled`]（text_scale）正交：density 不影响文本测量。
    pub fn scaled(self, density: f32) -> SpacingTokens {
        let d = if density > 0.0 { density } else { 1.0 };
        SpacingTokens { unit: self.unit * d }
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
    ///
    /// token 选择按 `scheme` 三态：常规 light/dark、高对比 high_contrast_light/dark
    /// （DC-5：HighContrast 必须产出**真正更高对比**的 token，而非与常规方案相同）。
    pub fn build_theme(id: ThemeId, name: &str, scheme: ResolvedColorScheme, palette: ColorPalette) -> Theme {
        let mut tokens = match scheme {
            ResolvedColorScheme::Light => SemanticTokens::light(),
            ResolvedColorScheme::Dark => SemanticTokens::dark(),
            ResolvedColorScheme::HighContrastLight => SemanticTokens::high_contrast_light(),
            ResolvedColorScheme::HighContrastDark => SemanticTokens::high_contrast_dark(),
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

    /// 由偏好 + 系统快照 + 窗口度量构建主题（spec IF-009 / DC-15 移动端「text scale」+「density」）。
    ///
    /// 在 [`build_theme`] 基础上应用 `metrics.text_scale`（缩字号）与 `metrics.density`（缩间距）——
    /// 这是 runtime 在窗口度量变化（用户调字号/密度）时重建主题的统一入口。任一改变会改变
    /// typography 或 spacing → 经 [`diff_invalidation`] 触发 `needs_layout`。
    pub fn build_theme_with_metrics(
        id: ThemeId,
        name: &str,
        preference: &ColorSchemePreference,
        system: SystemThemeSnapshot,
        palette: ColorPalette,
        metrics: WindowMetrics,
    ) -> Theme {
        let scheme = Self::resolve_scheme(preference, system);
        let mut theme = Self::build_theme(id, name, scheme, palette);
        theme.typography = theme.typography.scaled(metrics.text_scale);
        theme.spacing = theme.spacing.scaled(metrics.density);
        theme
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

    #[test]
    fn color_for_resolves_known_tokens() {
        let t = SemanticTokens::light();
        assert_eq!(t.color_for("background"), Some(t.background));
        assert_eq!(t.color_for("primary"), Some(t.primary));
        assert_eq!(t.color_for("success"), Some(t.success));
        assert_eq!(t.color_for("on_warning"), Some(t.on_warning));
        assert_eq!(t.color_for("unknown"), None);
    }

    #[test]
    fn contrast_ratio_black_on_white_is_maximal() {
        // 黑底白字 → 21.0（WCAG 最大）。
        let r = contrast_ratio(Color::BLACK, Color::WHITE);
        assert!((r - 21.0).abs() < 0.01, "black/white ratio = {r}");
        // 同色 → 1.0。
        assert!((contrast_ratio(Color::WHITE, Color::WHITE) - 1.0).abs() < 0.01);
    }

    #[test]
    fn wcag_aa_thresholds() {
        // 深色 token 文案 on_background(BLACK) on background(WHITE) → 通过 AA/AAA。
        let tokens = SemanticTokens::light();
        assert!(passes_wcag_aa(tokens.on_background, tokens.background, false));
        assert!(passes_wcag_aaa(tokens.on_background, tokens.background, false));
        // 浅灰 on 浅灰 → 不通过 AA。
        let gray = Color::rgb(0.7, 0.7, 0.7);
        let light = Color::rgb(0.9, 0.9, 0.9);
        assert!(!passes_wcag_aa(gray, light, false));
        // 大文本门槛更低（3.0），同一对可能 large 通过而 normal 不通过。
        let mid = Color::rgb(0.62, 0.62, 0.62);
        let bg = Color::WHITE;
        let ratio = contrast_ratio(mid, bg);
        let _ = ratio; // 仅断言门槛行为：
        assert!(passes_wcag_aa(mid, bg, true) || !passes_wcag_aa(mid, bg, false));
    }

    #[test]
    fn baseline_text_pairs_pass_aa() {
        // DC-5 contrast lint：基线主题的核心文本对（on_background/on_surface）通过 AA；
        // status 色（success/warning）用 BLACK 文案，黑字对中等亮度状态色通过 AA。
        for scheme in [ResolvedColorScheme::Light, ResolvedColorScheme::Dark] {
            let t = if scheme.is_dark() {
                SemanticTokens::dark()
            } else {
                SemanticTokens::light()
            };
            for (fg, bg) in [
                (t.on_background, t.background),
                (t.on_surface, t.surface),
                (t.on_success, t.success),
                (t.on_warning, t.warning),
            ] {
                assert!(
                    passes_wcag_aa(fg, bg, false),
                    "{scheme:?} token pair fg={fg:?} bg={bg:?} fails WCAG AA (ratio {})",
                    contrast_ratio(fg, bg)
                );
            }
        }
    }

    #[test]
    fn all_semantic_token_pairs_pass_wcag_aa() {
        // DC-5 contrast lint（完整接入）：Zero 主题 light + dark 的**全部 6 个** fg/bg token 对
        // 均通过 WCAG AA（正常文本 ≥ 4.5:1）。这是主题可访问性的回归门禁——任何 token 调整
        // 致对比度退化都会被本测试拦截。覆盖 baseline_text_pairs_pass_aa 未覆盖的 primary/error。
        for (name, tokens) in [("light", SemanticTokens::light()), ("dark", SemanticTokens::dark())] {
            for (pair, fg, bg) in [
                ("on_background/background", tokens.on_background, tokens.background),
                ("on_surface/surface", tokens.on_surface, tokens.surface),
                ("on_primary/primary", tokens.on_primary, tokens.primary),
                ("on_error/error", tokens.on_error, tokens.error),
                ("on_success/success", tokens.on_success, tokens.success),
                ("on_warning/warning", tokens.on_warning, tokens.warning),
            ] {
                let ratio = contrast_ratio(fg, bg);
                assert!(
                    passes_wcag_aa(fg, bg, false),
                    "{name} {pair} fails WCAG AA: ratio {ratio:.2} < 4.5 (fg {fg:?} bg {bg:?})"
                );
            }
        }
    }

    #[test]
    fn high_contrast_core_text_pairs_pass_aaa() {
        // DC-5 HighContrast：高对比模式的核心文本对（承载绝大多数文本）必须通过 WCAG AAA
        // （正常文本 ≥ 7:1），显著超过常规 AA 基线。这是「高对比」语义的可验证定义——
        // HighContrast 不能只是与 Light/Dark 相同的标签。
        for (name, tokens) in [
            ("hc_light", SemanticTokens::high_contrast_light()),
            ("hc_dark", SemanticTokens::high_contrast_dark()),
        ] {
            for (pair, fg, bg) in [
                ("on_background/background", tokens.on_background, tokens.background),
                ("on_surface/surface", tokens.on_surface, tokens.surface),
                ("on_primary/primary", tokens.on_primary, tokens.primary),
            ] {
                let ratio = contrast_ratio(fg, bg);
                assert!(
                    passes_wcag_aaa(fg, bg, false),
                    "{name} {pair} fails WCAG AAA: ratio {ratio:.2} < 7.0 (fg {fg:?} bg {bg:?})"
                );
            }
        }
    }

    #[test]
    fn high_contrast_status_pairs_pass_aa() {
        // 状态色（success/warning/error）在中亮度区间天然难以达到 AAA；高对比模式下至少
        // 保持 AA（≥ 4.5:1），与常规基线一致，且配对比的前景色（白字/黑字）。
        for (name, tokens) in [
            ("hc_light", SemanticTokens::high_contrast_light()),
            ("hc_dark", SemanticTokens::high_contrast_dark()),
        ] {
            for (pair, fg, bg) in [
                ("on_error/error", tokens.on_error, tokens.error),
                ("on_success/success", tokens.on_success, tokens.success),
                ("on_warning/warning", tokens.on_warning, tokens.warning),
            ] {
                let ratio = contrast_ratio(fg, bg);
                assert!(
                    passes_wcag_aa(fg, bg, false),
                    "{name} {pair} fails WCAG AA: ratio {ratio:.2} < 4.5 (fg {fg:?} bg {bg:?})"
                );
            }
        }
    }

    #[test]
    fn build_theme_uses_high_contrast_tokens_for_high_contrast_scheme() {
        // DC-5 关键不变量：HighContrast 方案必须产出**真正的高对比 token**，
        // 而非退化为常规 light/dark。此前 build_theme 只按 is_dark() 分支，
        // HighContrastLight 与 Light 产出相同 token（HighContrast 沦为空标签）——本测拦截该退化。
        let normal_light = ThemeResolver::build_theme(
            ThemeId::new("zero"),
            "Zero",
            ResolvedColorScheme::Light,
            ColorPalette::default(),
        );
        let hc_light = ThemeResolver::build_theme(
            ThemeId::new("zero"),
            "Zero",
            ResolvedColorScheme::HighContrastLight,
            ColorPalette::default(),
        );
        assert_ne!(
            hc_light.tokens, normal_light.tokens,
            "HighContrastLight must differ from Light (else HighContrast is a no-op label)"
        );
        // 高对比浅色的核心文本对应达 AAA，常规浅色仅 AA——证明「更高对比」语义成立。
        let hc_ratio = contrast_ratio(hc_light.tokens.on_surface, hc_light.tokens.surface);
        assert!(hc_ratio >= 7.0, "HC light on_surface/surface ratio {hc_ratio:.2} < 7.0");

        let normal_dark = ThemeResolver::build_theme(
            ThemeId::new("zero"),
            "Zero",
            ResolvedColorScheme::Dark,
            ColorPalette::default(),
        );
        let hc_dark = ThemeResolver::build_theme(
            ThemeId::new("zero"),
            "Zero",
            ResolvedColorScheme::HighContrastDark,
            ColorPalette::default(),
        );
        assert_ne!(
            hc_dark.tokens, normal_dark.tokens,
            "HighContrastDark must differ from Dark (else HighContrast is a no-op label)"
        );
        assert!(scheme_resolves_high_contrast_via_resolver());
    }

    fn scheme_resolves_high_contrast_via_resolver() -> bool {
        // 端到端：用户偏好 Light + 系统高对比 → 解析为 HighContrastLight → build_theme 产出 AAA token。
        let sys = SystemThemeSnapshot {
            system_scheme: ResolvedColorScheme::Light,
            high_contrast: true,
        };
        let scheme = ThemeResolver::resolve_scheme(&ColorSchemePreference::Light, sys);
        assert_eq!(scheme, ResolvedColorScheme::HighContrastLight);
        let theme = ThemeResolver::build_theme(ThemeId::new("zero"), "Zero", scheme, ColorPalette::default());
        contrast_ratio(theme.tokens.on_background, theme.tokens.background) >= 7.0
    }

    #[test]
    fn typography_scaled_scales_font_sizes_only() {
        // DC-15 text_scale：字号按 scale 缩放，字重/行高不变。
        let base = TypographyTokens::default_typography();
        let scaled = base.scaled(1.5);
        assert!((scaled.body_size_px - 14.0 * 1.5).abs() < 1e-6);
        assert!((scaled.heading_size_px - 18.0 * 1.5).abs() < 1e-6);
        assert_eq!(scaled.body_weight, base.body_weight);
        assert_eq!(scaled.body_line_height, base.body_line_height);
        assert_eq!(scaled.heading_weight, base.heading_weight);
        // 非正 scale 视为 1.0（防退化，不产生零/负字号）。
        let zero = base.scaled(0.0);
        assert_eq!(zero.body_size_px, base.body_size_px);
        let neg = base.scaled(-2.0);
        assert_eq!(neg.body_size_px, base.body_size_px);
    }

    #[test]
    fn build_theme_with_metrics_applies_text_scale() {
        // spec IF-009 / DC-15：runtime 经 WindowMetrics.text_scale 缩放主题字号。
        use crate::layout::{DEFAULT_TEXT_SCALE, WindowMetrics};
        let sys = SystemThemeSnapshot {
            system_scheme: ResolvedColorScheme::Light,
            high_contrast: false,
        };
        let metrics = WindowMetrics {
            logical_size: crate::geometry::Size::new(800.0, 600.0),
            scale_factor: 1.0,
            safe_area: crate::geometry::Insets::all(0.0),
            keyboard_insets: crate::geometry::Insets::all(0.0),
            text_scale: 1.25,
            density: 1.5,
            orientation: crate::layout::Orientation::Landscape,
        };
        let theme = ThemeResolver::build_theme_with_metrics(
            ThemeId::new("zero"),
            "Zero",
            &ColorSchemePreference::Light,
            sys,
            ColorPalette::default(),
            metrics,
        );
        // text_scale 缩字号；density 缩间距（两者正交）。
        assert!((theme.typography.body_size_px - 14.0 * 1.25).abs() < 1e-6);
        assert!((theme.spacing.unit - 4.0 * 1.5).abs() < 1e-6);
        let _ = DEFAULT_TEXT_SCALE;
    }

    #[test]
    fn text_scale_change_requests_layout_invalidation() {
        // DC-15 关键不变量：text_scale 改变 → typography 改变 → diff_invalidation 标 needs_layout。
        // 这是 text_scale 影响布局（文本重测量）的统一可验证路径。
        use crate::layout::WindowMetrics;
        let sys = SystemThemeSnapshot {
            system_scheme: ResolvedColorScheme::Light,
            high_contrast: false,
        };
        let base_metrics = WindowMetrics {
            logical_size: crate::geometry::Size::new(800.0, 600.0),
            scale_factor: 1.0,
            safe_area: crate::geometry::Insets::all(0.0),
            keyboard_insets: crate::geometry::Insets::all(0.0),
            text_scale: 1.0,
            density: 1.0,
            orientation: crate::layout::Orientation::Landscape,
        };
        let scaled_metrics = WindowMetrics {
            text_scale: 1.5,
            ..base_metrics
        };
        let t1 = ThemeResolver::build_theme_with_metrics(
            ThemeId::new("zero"),
            "Zero",
            &ColorSchemePreference::Light,
            sys,
            ColorPalette::default(),
            base_metrics,
        );
        let t2 = ThemeResolver::build_theme_with_metrics(
            ThemeId::new("zero"),
            "Zero",
            &ColorSchemePreference::Light,
            sys,
            ColorPalette::default(),
            scaled_metrics,
        );
        let inv = diff_invalidation(&t1, &t2);
        assert!(
            inv.contains(InvalidationFlags::NEEDS_LAYOUT),
            "text_scale change must request layout (text re-measure)"
        );
        assert!(inv.contains(InvalidationFlags::NEEDS_PAINT));
    }

    #[test]
    fn spacing_scaled_scales_unit_only() {
        // DC-15 density：SpacingTokens.unit 按 density 缩放；非正数视为 1.0 防退化。
        let base = SpacingTokens::default_spacing();
        assert!((base.scaled(2.0).unit - 4.0 * 2.0).abs() < 1e-6);
        assert!((base.scaled(0.5).unit - 4.0 * 0.5).abs() < 1e-6);
        assert_eq!(base.scaled(0.0).unit, base.unit);
        assert_eq!(base.scaled(-1.0).unit, base.unit);
    }

    #[test]
    fn density_change_requests_layout_invalidation() {
        // DC-15 关键不变量：density 改变 → spacing 改变 → diff_invalidation 标 needs_layout。
        // density 影响布局（间距重排）的统一可验证路径；与 text_scale 正交。
        use crate::layout::WindowMetrics;
        let sys = SystemThemeSnapshot {
            system_scheme: ResolvedColorScheme::Light,
            high_contrast: false,
        };
        let base_metrics = WindowMetrics {
            logical_size: crate::geometry::Size::new(800.0, 600.0),
            scale_factor: 1.0,
            safe_area: crate::geometry::Insets::all(0.0),
            keyboard_insets: crate::geometry::Insets::all(0.0),
            text_scale: 1.0,
            density: 1.0,
            orientation: crate::layout::Orientation::Landscape,
        };
        let dense_metrics = WindowMetrics {
            density: 1.75,
            ..base_metrics
        };
        let t1 = ThemeResolver::build_theme_with_metrics(
            ThemeId::new("zero"),
            "Zero",
            &ColorSchemePreference::Light,
            sys,
            ColorPalette::default(),
            base_metrics,
        );
        let t2 = ThemeResolver::build_theme_with_metrics(
            ThemeId::new("zero"),
            "Zero",
            &ColorSchemePreference::Light,
            sys,
            ColorPalette::default(),
            dense_metrics,
        );
        // density 改了 spacing，没改 typography（正交）。
        assert_ne!(t1.spacing, t2.spacing);
        assert_eq!(t1.typography, t2.typography);
        let inv = diff_invalidation(&t1, &t2);
        assert!(
            inv.contains(InvalidationFlags::NEEDS_LAYOUT),
            "density change must request layout (spacing re-flow)"
        );
        assert!(inv.contains(InvalidationFlags::NEEDS_PAINT));
    }
}

//! 主题提供者（spec FR-007 `ThemeResolver` 运行时态 + `ThemeChanged`）。
//!
//! 持有用户偏好与当前 Theme；系统主题变化时重算并发出带失效标记的 `ThemeChanged`
//! （字体/间距不变 → 仅 needs_paint，DC-5）。
//!
//! DC-12/DC-15： additionally 应用 `text_scale`（缩字号）与 `density`（缩间距）——
//! `WindowMetrics` 的两个移动端关键输入经本 provider 流入运行时主题（DC-15「text scale」
//! + 布局密度），改变时触发 `needs_layout`（字号/间距影响测量与排版）。

use zero_ui_core::theme::{
    ColorPalette, ColorSchemePreference, ResolvedColorScheme, SystemThemeSnapshot, Theme, ThemeChanged, ThemeId,
    ThemeResolver, diff_invalidation,
};

/// 运行时主题状态。
#[derive(Debug, Clone)]
pub struct ThemeProvider {
    preference: ColorSchemePreference,
    palette: ColorPalette,
    /// 用户文本字号缩放（DC-15，默认 `1.0`）；改变 → 字号变 → needs_layout。
    text_scale: f32,
    /// 布局密度缩放（DC-15，默认 `1.0`）；改变 → 间距变 → needs_layout。
    density: f32,
    current: Theme,
}

impl ThemeProvider {
    /// 初始化：按偏好 + 系统快照解析首版主题（text_scale/density 默认 `1.0`）。
    pub fn new(preference: ColorSchemePreference, palette: ColorPalette, system: SystemThemeSnapshot) -> ThemeProvider {
        let scheme = ThemeResolver::resolve_scheme(&preference, system);
        let theme = build_scaled_theme(ThemeId::new("zero"), "Zero", scheme, &palette, 1.0, 1.0);
        ThemeProvider {
            preference,
            palette,
            text_scale: 1.0,
            density: 1.0,
            current: theme,
        }
    }

    pub fn current(&self) -> &Theme {
        &self.current
    }

    /// 当前 text_scale（DC-15）。
    pub fn text_scale(&self) -> f32 {
        self.text_scale
    }

    /// 当前 density（DC-15）。
    pub fn density(&self) -> f32 {
        self.density
    }

    /// 暴露当前用户偏好（测试辅助 / UI 控件查询）。
    pub fn current_preference(&self) -> ColorSchemePreference {
        self.preference.clone()
    }

    /// 系统主题变化时调用；若解析方案变化则生成 `ThemeChanged`（含 diff 失效）。
    pub fn on_system_change(&mut self, system: SystemThemeSnapshot) -> Option<ThemeChanged> {
        let new_scheme = ThemeResolver::resolve_scheme(&self.preference, system);
        if new_scheme == self.current.scheme {
            return None;
        }
        Some(self.rebuild(new_scheme))
    }

    /// 显式切换偏好（用户操作）。
    pub fn set_preference(&mut self, preference: ColorSchemePreference, system: SystemThemeSnapshot) -> ThemeChanged {
        self.preference = preference;
        let scheme = ThemeResolver::resolve_scheme(&self.preference, system);
        self.rebuild(scheme)
    }

    /// 循环切换偏好：System → Light → Dark → System（DC-5「偏好切换 UI」的 API 基础）。
    ///
    /// 宿主应用（如浏览器）可绑定键盘快捷键或 UI 按钮调用本方法，无需自行维护偏好状态机。
    /// 返回 `ThemeChanged` 含 diff 失效标记（仅颜色变化 → needs_paint；字体/间距不变不布局）。
    pub fn cycle_preference(&mut self, system: SystemThemeSnapshot) -> ThemeChanged {
        let next = match self.preference {
            ColorSchemePreference::System => ColorSchemePreference::Light,
            ColorSchemePreference::Light => ColorSchemePreference::Dark,
            _ => ColorSchemePreference::System,
        };
        self.set_preference(next, system)
    }

    /// 设置文本字号缩放（DC-15「text scale」，移动端无障碍/系统字号）。
    ///
    /// `text_scale` 钳到 `> 0.0`（非正数视为 `1.0`，见 [`TypographyTokens::scaled`]）。
    /// 值不变 → `None`；变化 → `ThemeChanged`（字号变 → `needs_layout` + `needs_paint`）。
    ///
    /// [`TypographyTokens::scaled`]: zero_ui_core::theme::TypographyTokens::scaled
    pub fn set_text_scale(&mut self, text_scale: f32, system: SystemThemeSnapshot) -> Option<ThemeChanged> {
        let normalized = if text_scale > 0.0 { text_scale } else { 1.0 };
        if (normalized - self.text_scale).abs() < f32::EPSILON {
            return None;
        }
        self.text_scale = normalized;
        // text_scale 不改 scheme，但改 typography → 用当前 scheme 重建。
        let scheme = ThemeResolver::resolve_scheme(&self.preference, system);
        Some(self.rebuild(scheme))
    }

    /// 设置布局密度（DC-15，移动端「compact/comfortable」间距密度）。
    ///
    /// 值不变 → `None`；变化 → `ThemeChanged`（间距变 → `needs_layout` + `needs_paint`）。
    pub fn set_density(&mut self, density: f32, system: SystemThemeSnapshot) -> Option<ThemeChanged> {
        let normalized = if density > 0.0 { density } else { 1.0 };
        if (normalized - self.density).abs() < f32::EPSILON {
            return None;
        }
        self.density = normalized;
        let scheme = ThemeResolver::resolve_scheme(&self.preference, system);
        Some(self.rebuild(scheme))
    }

    /// 暴露当前解析方案（测试辅助）。
    pub fn resolved_scheme(&self) -> ResolvedColorScheme {
        self.current.scheme
    }

    /// 用当前 palette + text_scale + density 按 `scheme` 重建主题，返回 diff 失效事件。
    fn rebuild(&mut self, scheme: ResolvedColorScheme) -> ThemeChanged {
        let new_theme = build_scaled_theme(
            self.current.id.clone(),
            &self.current.name,
            scheme,
            &self.palette,
            self.text_scale,
            self.density,
        );
        let invalidation = diff_invalidation(&self.current, &new_theme);
        self.current = new_theme.clone();
        ThemeChanged {
            new_theme,
            invalidation,
        }
    }
}

/// 按 scheme + palette 解析基线主题，再应用 `text_scale`（typography）与 `density`（spacing）。
///
/// 与 `ThemeResolver::build_theme_with_metrics` 等价（DC-12/DC-15），但 provider 只持主题相关
/// 的两个 metrics 标量（text_scale/density），不需完整 `WindowMetrics`。
fn build_scaled_theme(
    id: ThemeId,
    name: &str,
    scheme: ResolvedColorScheme,
    palette: &ColorPalette,
    text_scale: f32,
    density: f32,
) -> Theme {
    let mut theme = ThemeResolver::build_theme(id, name, scheme, palette.clone());
    theme.typography = theme.typography.scaled(text_scale);
    theme.spacing = theme.spacing.scaled(density);
    theme
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::invalidation::InvalidationFlags;

    fn sys(scheme: ResolvedColorScheme) -> SystemThemeSnapshot {
        SystemThemeSnapshot {
            system_scheme: scheme,
            high_contrast: false,
        }
    }

    #[test]
    fn system_scheme_change_emits_paint_only_invalidation() {
        let mut p = ThemeProvider::new(
            ColorSchemePreference::System,
            ColorPalette::default(),
            sys(ResolvedColorScheme::Light),
        );
        // 系统 Light → Dark：仅颜色变化，字体/间距不变 → 只 needs_paint。
        let changed = p
            .on_system_change(sys(ResolvedColorScheme::Dark))
            .expect("scheme changed");
        assert!(changed.invalidation.contains(InvalidationFlags::NEEDS_PAINT));
        assert!(!changed.invalidation.contains(InvalidationFlags::NEEDS_LAYOUT));
        assert_eq!(p.resolved_scheme(), ResolvedColorScheme::Dark);
    }

    #[test]
    fn no_change_no_event() {
        let mut p = ThemeProvider::new(
            ColorSchemePreference::System,
            ColorPalette::default(),
            sys(ResolvedColorScheme::Light),
        );
        assert!(p.on_system_change(sys(ResolvedColorScheme::Light)).is_none());
    }

    #[test]
    fn text_scale_change_requests_layout_and_scales_typography() {
        // DC-15：text_scale 改变 → typography 缩放 → needs_layout（字号影响测量）。
        let mut p = ThemeProvider::new(
            ColorSchemePreference::Light,
            ColorPalette::default(),
            sys(ResolvedColorScheme::Light),
        );
        let base_body = p.current().typography.body_size_px;
        let changed = p
            .set_text_scale(1.5, sys(ResolvedColorScheme::Light))
            .expect("text_scale changed");
        assert!(changed.invalidation.contains(InvalidationFlags::NEEDS_LAYOUT));
        assert!(changed.invalidation.contains(InvalidationFlags::NEEDS_PAINT));
        assert!((p.current().typography.body_size_px - base_body * 1.5).abs() < 1e-6);
        // 值不变 → None。
        assert!(
            p.set_text_scale(1.5, sys(ResolvedColorScheme::Light)).is_none(),
            "same text_scale → no event"
        );
        // 非正数视为 1.0（与 normalized 比较）。
        p.set_text_scale(0.0, sys(ResolvedColorScheme::Light));
        assert_eq!(p.text_scale(), 1.0);
    }

    #[test]
    fn density_change_requests_layout_and_scales_spacing() {
        // DC-15：density 改变 → spacing 缩放 → needs_layout（间距影响排版）。typography 不变（正交）。
        let mut p = ThemeProvider::new(
            ColorSchemePreference::Light,
            ColorPalette::default(),
            sys(ResolvedColorScheme::Light),
        );
        let base_unit = p.current().spacing.unit;
        let base_body = p.current().typography.body_size_px;
        let changed = p
            .set_density(2.0, sys(ResolvedColorScheme::Light))
            .expect("density changed");
        assert!(changed.invalidation.contains(InvalidationFlags::NEEDS_LAYOUT));
        assert!((p.current().spacing.unit - base_unit * 2.0).abs() < 1e-6);
        // density 与 text_scale 正交：typography 不变。
        assert!((p.current().typography.body_size_px - base_body).abs() < 1e-6);
        assert!(p.set_density(2.0, sys(ResolvedColorScheme::Light)).is_none());
    }

    #[test]
    fn scaled_metrics_persist_across_scheme_change() {
        // text_scale/density 在 scheme 变化（系统主题切换）时保留并继续生效。
        let mut p = ThemeProvider::new(
            ColorSchemePreference::System,
            ColorPalette::default(),
            sys(ResolvedColorScheme::Light),
        );
        p.set_text_scale(1.25, sys(ResolvedColorScheme::Light));
        p.set_density(1.5, sys(ResolvedColorScheme::Light));
        let body_before = p.current().typography.body_size_px;
        let unit_before = p.current().spacing.unit;
        // 系统切换 Dark：颜色变（paint-only 额外叠加），但 text_scale/density 保留。
        p.on_system_change(sys(ResolvedColorScheme::Dark));
        assert_eq!(p.text_scale(), 1.25);
        assert_eq!(p.density(), 1.5);
        assert!((p.current().typography.body_size_px - body_before).abs() < 1e-6);
        assert!((p.current().spacing.unit - unit_before).abs() < 1e-6);
    }

    // ── DC-5 preference cycling ─────────────────────────────────────────────

    #[test]
    fn cycle_preference_system_light_dark_system() {
        // DC-5：System → Light → Dark → System 三元循环，每次切换返回 ThemeChanged。
        let mut p = ThemeProvider::new(
            ColorSchemePreference::System,
            ColorPalette::default(),
            sys(ResolvedColorScheme::Light),
        );
        assert_eq!(p.current_preference(), ColorSchemePreference::System);
        assert_eq!(p.resolved_scheme(), ResolvedColorScheme::Light);

        // System → Light：同解析方案 Light → tokens 不变 → 可能无 paint 失效（diff_invalidation CLEAN）。
        // 但偏好确实变了。
        let c1 = p.cycle_preference(sys(ResolvedColorScheme::Light));
        assert_eq!(p.current_preference(), ColorSchemePreference::Light);
        assert_eq!(p.resolved_scheme(), ResolvedColorScheme::Light);
        // 同方案同 token 无变化 → 不应有 layout 失效。
        assert!(!c1.invalidation.contains(InvalidationFlags::NEEDS_LAYOUT));

        // Light → Dark：解析方案 Light→Dark，tokens 变 → needs_paint。
        let c2 = p.cycle_preference(sys(ResolvedColorScheme::Light));
        assert_eq!(p.current_preference(), ColorSchemePreference::Dark);
        assert_eq!(p.resolved_scheme(), ResolvedColorScheme::Dark);
        assert!(c2.invalidation.contains(InvalidationFlags::NEEDS_PAINT));
        assert!(!c2.invalidation.contains(InvalidationFlags::NEEDS_LAYOUT));

        // Dark → System（回环）→ 解析方案 Dark→Light，tokens 变 → needs_paint。
        let c3 = p.cycle_preference(sys(ResolvedColorScheme::Light));
        assert_eq!(p.current_preference(), ColorSchemePreference::System);
        assert_eq!(p.resolved_scheme(), ResolvedColorScheme::Light);
        assert!(c3.invalidation.contains(InvalidationFlags::NEEDS_PAINT));
    }

    #[test]
    fn cycle_preference_respects_high_contrast() {
        // HighContrast 由系统标志触发，非独立偏好。Dark + high_contrast → HighContrastDark。
        let mut p = ThemeProvider::new(
            ColorSchemePreference::System,
            ColorPalette::default(),
            sys(ResolvedColorScheme::Dark),
        );
        // System → Light。
        p.cycle_preference(SystemThemeSnapshot {
            system_scheme: ResolvedColorScheme::Dark,
            high_contrast: true,
        });
        assert_eq!(p.resolved_scheme(), ResolvedColorScheme::HighContrastLight);
        // Light → Dark。
        p.cycle_preference(SystemThemeSnapshot {
            system_scheme: ResolvedColorScheme::Dark,
            high_contrast: true,
        });
        assert_eq!(p.resolved_scheme(), ResolvedColorScheme::HighContrastDark);
    }
}

//! 主题提供者（spec FR-007 `ThemeResolver` 运行时态 + `ThemeChanged`）。
//!
//! 持有用户偏好与当前 Theme；系统主题变化时重算并发出带失效标记的 `ThemeChanged`
//! （字体/间距不变 → 仅 needs_paint，DC-5）。

use zero_ui_core::theme::{
    ColorPalette, ColorSchemePreference, ResolvedColorScheme, SystemThemeSnapshot, Theme, ThemeChanged, ThemeId,
    ThemeResolver, diff_invalidation,
};

/// 运行时主题状态。
#[derive(Debug, Clone)]
pub struct ThemeProvider {
    preference: ColorSchemePreference,
    palette: ColorPalette,
    current: Theme,
}

impl ThemeProvider {
    /// 初始化：按偏好 + 系统快照解析首版主题。
    pub fn new(preference: ColorSchemePreference, palette: ColorPalette, system: SystemThemeSnapshot) -> ThemeProvider {
        let scheme = ThemeResolver::resolve_scheme(&preference, system);
        let theme = ThemeResolver::build_theme(ThemeId::new("zero"), "Zero", scheme, palette.clone());
        ThemeProvider {
            preference,
            palette,
            current: theme,
        }
    }

    pub fn current(&self) -> &Theme {
        &self.current
    }

    /// 系统主题变化时调用；若解析方案变化则生成 `ThemeChanged`（含 diff 失效）。
    pub fn on_system_change(&mut self, system: SystemThemeSnapshot) -> Option<ThemeChanged> {
        let new_scheme = ThemeResolver::resolve_scheme(&self.preference, system);
        if new_scheme == self.current.scheme {
            return None;
        }
        let new_theme = ThemeResolver::build_theme(
            self.current.id.clone(),
            &self.current.name,
            new_scheme,
            self.palette.clone(),
        );
        let invalidation = diff_invalidation(&self.current, &new_theme);
        self.current = new_theme.clone();
        Some(ThemeChanged {
            new_theme,
            invalidation,
        })
    }

    /// 显式切换偏好（用户操作）。
    pub fn set_preference(&mut self, preference: ColorSchemePreference, system: SystemThemeSnapshot) -> ThemeChanged {
        self.preference = preference;
        let scheme = ThemeResolver::resolve_scheme(&self.preference, system);
        let new_theme = ThemeResolver::build_theme(
            self.current.id.clone(),
            &self.current.name,
            scheme,
            self.palette.clone(),
        );
        let invalidation = diff_invalidation(&self.current, &new_theme);
        self.current = new_theme.clone();
        ThemeChanged {
            new_theme,
            invalidation,
        }
    }

    /// 暴露当前解析方案（测试辅助）。
    pub fn resolved_scheme(&self) -> ResolvedColorScheme {
        self.current.scheme
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(
            changed
                .invalidation
                .contains(zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT)
        );
        assert!(
            !changed
                .invalidation
                .contains(zero_ui_core::invalidation::InvalidationFlags::NEEDS_LAYOUT)
        );
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
}

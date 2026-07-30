//! `color-scheme` used-scheme 合成测试（CSS Color Adjust L1 §2.3）。
//!
//! 验证 used color-scheme 由 `color-scheme` 属性列表与 `prefers-color-scheme` 媒体查询
//! 合成：`light dark` + prefers=dark → dark（全局暗模式 theming 的核心）。
//! 经 `compute_styles`（生产路径 `lib.rs` 传 `self.prefers_color_scheme`）端到端验证。

use super::super::*;
use super::helpers::make_test_dom;
use zero_css_parser::Parser as CssParser;
use zero_css_parser::media_query::PrefersColorSchemeValue;
use zero_css_parser::values::ColorValue;

use crate::property::apply::parse_color_scheme_dark;

const GREEN: ColorValue = ColorValue::Rgba(0, 128, 0, 255);
const RED: ColorValue = ColorValue::Rgba(255, 0, 0, 255);

// ── 单元：parse_color_scheme_dark 合成矩阵 ──────────────────────────────

#[test]
fn test_parse_color_scheme_synthesis_matrix() {
    // 仅 dark → dark（不受 prefers 影响）
    assert!(parse_color_scheme_dark("dark", false));
    assert!(parse_color_scheme_dark("dark", true));
    assert!(parse_color_scheme_dark("only dark", false));
    assert!(parse_color_scheme_dark("dark only", true));

    // 仅 light → light
    assert!(!parse_color_scheme_dark("light", false));
    assert!(!parse_color_scheme_dark("light", true));
    assert!(!parse_color_scheme_dark("only light", true));

    // light + dark（两种均可用）→ 由 prefers 决定（★ 合成核心）
    assert!(!parse_color_scheme_dark("light dark", false));
    assert!(parse_color_scheme_dark("light dark", true));
    assert!(!parse_color_scheme_dark("dark light", false));
    assert!(parse_color_scheme_dark("dark light", true));
    assert!(parse_color_scheme_dark("only light dark", true));

    // normal / 缺省 / 仅 custom-ident → 保守 light（不因暗 OS 整体翻转未 opt-in 页面）
    assert!(!parse_color_scheme_dark("normal", true));
    assert!(!parse_color_scheme_dark("only", true));
    assert!(!parse_color_scheme_dark("", true));
    assert!(parse_color_scheme_dark("light dark weird", true)); // 含 light+dark 仍合成
}

// ── 端到端：compute_styles + set_prefers_color_scheme ──────────────────

/// 解析 CSS，按指定 prefers 跑 compute_styles，返回 div 的计算 background-color。
fn div_background_with_prefers(css: &str, prefers: PrefersColorSchemeValue) -> ColorValue {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let stylesheets = vec![CssParser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    sys.set_prefers_color_scheme(prefers);
    let styles = sys.compute_styles(&doc, &stylesheets);
    styles.get(&div).expect("div 应有计算样式").background_color.clone()
}

#[test]
/// `color-scheme: light dark` + prefers=dark → light-dark 取 dark 参数 green。
/// 这是 R2285 light-dark() 与本轮 used-scheme 合成协作的全局暗模式核心场景。
fn test_light_dark_synthesizes_to_dark_when_prefers_dark() {
    let css = "div { color-scheme: light dark; background-color: light-dark(red, green); }";
    assert_eq!(
        div_background_with_prefers(css, PrefersColorSchemeValue::Dark),
        GREEN,
        "color-scheme:light dark + prefers-color-scheme:dark 应合成 dark，light-dark 取 green"
    );
}

#[test]
/// `color-scheme: light dark` + prefers=light → light-dark 取 light 参数 red。
fn test_light_dark_synthesizes_to_light_when_prefers_light() {
    let css = "div { color-scheme: light dark; background-color: light-dark(red, green); }";
    assert_eq!(
        div_background_with_prefers(css, PrefersColorSchemeValue::Light),
        RED,
        "color-scheme:light dark + prefers-color-scheme:light 应合成 light，light-dark 取 red"
    );
}

#[test]
/// `color-scheme: dark`（仅 dark）→ 不受 prefers 影响，恒取 dark 参数。
/// 回归守护：仅 dark 列表不因 prefers=light 退回 light。
fn test_color_scheme_dark_only_ignores_prefers_light() {
    let css = "div { color-scheme: dark; background-color: light-dark(red, green); }";
    assert_eq!(
        div_background_with_prefers(css, PrefersColorSchemeValue::Light),
        GREEN,
        "color-scheme:dark 应恒为 dark，不受 prefers-color-scheme:light 影响"
    );
}

#[test]
/// 未声明 color-scheme（默认 normal）+ prefers=dark → 不翻转（保守 light）。
/// 守护：未 opt-in 的页面不应在暗 OS 上整体变暗。
fn test_undeclared_color_scheme_stays_light_under_dark_prefers() {
    let css = "div { background-color: light-dark(red, green); }";
    assert_eq!(
        div_background_with_prefers(css, PrefersColorSchemeValue::Dark),
        RED,
        "未声明 color-scheme（normal）即使 prefers=dark 也应保守取 light"
    );
}

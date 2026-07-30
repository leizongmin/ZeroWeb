//! `@property` 注册自定义属性（CSS Properties and Values API）端到端测试。
//!
//! 验证 `@property` 注册的 `initial-value` 在 `var()` 解析时作为兜底默认值，
//! 以及 `inherits` 描述符控制继承/重置语义。经 `compute_styles`（注册预扫描入口）。

use super::super::*;
use super::helpers::make_test_dom;
use zero_css_parser::Parser as CssParser;
use zero_css_parser::values::ColorValue;

const GREEN: ColorValue = ColorValue::Rgba(0, 128, 0, 255);
const RED: ColorValue = ColorValue::Rgba(255, 0, 0, 255);

/// 解析 CSS，对 `html > body > div#main > p.text` DOM 跑 compute_styles，返回 (div, p) 的计算色。
fn compute_colors(css: &str) -> (ColorValue, ColorValue) {
    let (doc, _html, _body, div, p) = make_test_dom();
    let stylesheets = vec![CssParser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_color = styles.get(&div).expect("div 应有计算样式").color.clone();
    let p_color = styles.get(&p).expect("p 应有计算样式").color.clone();
    (div_color, p_color)
}

#[test]
/// `@property` 的 initial-value 作为 var() 兜底默认值（未显式声明时）。
/// `@property --x { initial-value: green } div { color: var(--x) }` → div 绿。
fn test_property_initial_value_seeds_var() {
    let css = r#"
        @property --x { syntax: "<color>"; inherits: true; initial-value: green; }
        div { color: var(--x); }
    "#;
    let (div_color, _) = compute_colors(css);
    assert_eq!(div_color, GREEN, "var(--x) 未显式声明时应解析为 initial-value green");
}

#[test]
/// 显式声明覆盖 initial-value。
/// `div { --x: red; color: var(--x) }` → div 红（显式声明优先于注册初值）。
fn test_property_explicit_override_initial_value() {
    let css = r#"
        @property --x { syntax: "<color>"; inherits: true; initial-value: green; }
        div { --x: red; color: var(--x); }
    "#;
    let (div_color, _) = compute_colors(css);
    assert_eq!(div_color, RED, "显式 --x: red 应覆盖 initial-value");
}

#[test]
/// `inherits: true` 注册属性像普通自定义属性一样继承到子元素。
/// `div { --x: red } p { color: var(--x) }`（--x 注册 inherits:true）→ p 红。
fn test_property_inherits_true_to_child() {
    let css = r#"
        @property --x { syntax: "<color>"; inherits: true; initial-value: green; }
        div { --x: red; }
        p { color: var(--x); }
    "#;
    let (_, p_color) = compute_colors(css);
    assert_eq!(p_color, RED, "inherits:true 时 --x:red 应继承到 p");
}

#[test]
/// `inherits: false` 注册属性**不**继承——子元素重置为 initial-value。
/// `div { --x: red } p { color: var(--x) }`（--x 注册 inherits:false，初值 green）→ p 绿（非继承的红）。
fn test_property_inherits_false_resets_to_initial_value() {
    let css = r#"
        @property --x { syntax: "<color>"; inherits: false; initial-value: green; }
        div { --x: red; }
        p { color: var(--x); }
    "#;
    let (_, p_color) = compute_colors(css);
    assert_eq!(
        p_color, GREEN,
        "inherits:false 时 p 不应继承 div 的 --x:red，而重置为 initial-value green"
    );
}

#[test]
/// 未注册的 var() 仍按原行为：未定义引用无回退 → invalid at computed-value-time → 回退默认。
/// 确保 @property 基础设施对未注册属性零影响（回归守护）。
fn test_property_unregistered_var_unchanged() {
    let css = "div { color: var(--undefined); }";
    let (div_color, _) = compute_colors(css);
    // 未定义、无回退 → color 声明无效 → 默认黑（inherit initial）。
    assert_eq!(
        div_color,
        ColorValue::Rgba(0, 0, 0, 255),
        "未注册未定义 var 应保持默认行为"
    );
}

// ── light-dark() + color-scheme（CSS Color Adjust）──────────────────────
// 验证 light-dark(L, D) 按元素 used color-scheme 取参。driving: css-variables
// registered-property-light-dark（@property 注册 + color-scheme: dark + var() 组合）。

/// 解析 CSS 跑 compute_styles，返回 div 的计算 background-color。
fn div_background_color(css: &str) -> ColorValue {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let stylesheets = vec![CssParser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &stylesheets);
    styles.get(&div).expect("div 应有计算样式").background_color.clone()
}

#[test]
/// `color-scheme: dark` + `light-dark(red, green)` → 取 dark 参数 green。
fn test_light_dark_dark_scheme_picks_dark_arg() {
    let css = "div { color-scheme: dark; background-color: light-dark(red, green); }";
    assert_eq!(
        div_background_color(css),
        GREEN,
        "color-scheme:dark 时 light-dark 应取第二参 green"
    );
}

#[test]
/// `color-scheme: light`（默认）+ `light-dark(red, green)` → 取 light 参数 red。
fn test_light_dark_light_scheme_picks_light_arg() {
    let css = "div { color-scheme: light; background-color: light-dark(red, green); }";
    assert_eq!(
        div_background_color(css),
        RED,
        "color-scheme:light 时 light-dark 应取首参 red"
    );
}

#[test]
/// 无 color-scheme 声明 → 默认 light → 取首参。
fn test_light_dark_default_scheme_is_light() {
    let css = "div { background-color: light-dark(red, green); }";
    assert_eq!(div_background_color(css), RED, "无 color-scheme 默认 light");
}

#[test]
/// driving reftest 镜像：@property 注册 + color-scheme:dark + var() 组合。
/// `@property --test-color { initial-value: red } .square { color-scheme: dark;
/// --test-color: light-dark(red, green); background-color: var(--test-color); }` → green。
fn test_registered_property_light_dark_reftest_mirror() {
    let css = r#"
        @property --test-color {
          syntax: "<color>";
          inherits: true;
          initial-value: red;
        }
        div { color-scheme: dark; --test-color: light-dark(red, green); background-color: var(--test-color); }
    "#;
    assert_eq!(
        div_background_color(css),
        GREEN,
        "color-scheme:dark + 注册属性 var(light-dark(red,green)) 应解析为 green"
    );
}

#[test]
/// color-scheme 继承：父 div 设 color-scheme:dark，子 p 的 light-dark() 应取 dark 参数。
fn test_color_scheme_inherited_to_child_light_dark() {
    let (doc, _html, _body, div, p) = make_test_dom();
    let css = r#"
        div { color-scheme: dark; }
        p { background-color: light-dark(red, green); }
    "#;
    let stylesheets = vec![CssParser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &stylesheets);
    let p_bg = styles.get(&p).expect("p 应有计算样式").background_color.clone();
    assert_eq!(
        p_bg, GREEN,
        "color-scheme:dark 应继承到 p，其 light-dark 取 dark 参数 green"
    );
    let _ = div;
}

//! `::first-letter` 伪元素计算样式端到端测试（CSS2 §5.12.2，R3867）。
//!
//! 验证：① `div:first-letter { color }` 规则匹配时 `first_letter_pseudo` 存储
//! 计算色（穿透到 paint 侧首字形覆色的数据源）；② 无 `::first-letter` 声明时
//! 不存储（含同页存在 `::first-line` 等其它伪元素规则的场景——此前回归：default
//! 黑被误存为伪元素色，transparent 文本首字符被染黑，background-image-first-line）。

use super::super::*;
use super::helpers::make_test_dom;
use zero_css_parser::Parser as CssParser;
use zero_css_parser::values::ColorValue;

const GREEN: ColorValue = ColorValue::Rgba(0, 128, 0, 255);

/// 解析 CSS，对 `html > body > div#main > p.text` DOM 跑 compute_styles，返回 div 的 first_letter_pseudo 色选项。
fn compute_first_letter_color(css: &str) -> Option<ColorValue> {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let stylesheets = vec![CssParser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &stylesheets);
    styles
        .get(&div)
        .expect("div 应有计算样式")
        .first_letter_pseudo
        .as_ref()
        .map(|s| s.color.clone())
}

#[test]
/// `div:first-letter { color: green }` 匹配 → first_letter_pseudo 存储绿色。
fn test_first_letter_rule_stores_pseudo_style() {
    let css = "div { color: black; } div:first-letter { color: green; }";
    assert_eq!(
        compute_first_letter_color(css),
        Some(GREEN),
        "::first-letter 规则匹配时应存储伪元素 color"
    );
}

#[test]
/// 无 `::first-letter` 声明 → 不存储（None）。
fn test_no_first_letter_rule_stores_none() {
    let css = "div { color: red; }";
    assert_eq!(
        compute_first_letter_color(css),
        None,
        "无 ::first-letter 声明时 first_letter_pseudo 应为 None"
    );
}

#[test]
/// 同页有 `::first-line` 规则但无 `::first-letter` 声明 → 不存储（回归锚：
/// 伪元素规则存在不能作为 ::first-letter 有匹配的信号；default 黑不得误存，
/// 否则 color:transparent 元素的首字符被染黑——background-image-first-line）。
fn test_first_line_rule_does_not_leak_into_first_letter() {
    let css = "div { color: red; } div:first-line { color: blue; }";
    assert_eq!(
        compute_first_letter_color(css),
        None,
        "::first-line 规则不应触发 ::first-letter 存储"
    );
}

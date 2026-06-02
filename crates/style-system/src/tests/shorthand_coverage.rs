// ═══════════════════════════════════════════════════════════════════
// shorthand 覆盖率测试
//
// 通过完整管线（StyleSystem::compute_styles）测试简写属性展开，
// 覆盖 shorthand::expand_shorthands 中的各分支。
// ═══════════════════════════════════════════════════════════════════

use super::super::*;
use super::helpers::*;

/// 辅助：通过完整管线计算样式
fn compute_style(doc: &zero_dom::Document, element: zero_dom::NodeId, declarations: &[(&str, &str)]) -> ComputedStyle {
    let rules: Vec<zero_css_parser::ast::Rule> =
        vec![zero_css_parser::ast::Rule::Style(zero_css_parser::ast::StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: declarations
                .iter()
                .map(|(p, v)| zero_css_parser::ast::Declaration {
                    property: (*p).to_string(),
                    value: (*v).to_string(),
                    important: false,
                })
                .collect(),
        })];
    let stylesheets = vec![zero_css_parser::Stylesheet { rules }];
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(doc, &stylesheets);
    styles.get(&element).cloned().unwrap_or_default()
}

#[test]
/// border-image 简写 - "none" 值
fn test_border_image_shorthand_none() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("border-image", "none")]);
    // border-image: none → 默认值
    assert_eq!(s.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// border-image 简写 - url 和切片值
fn test_border_image_shorthand_with_url() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("border-image", "url(img.png) 30")]);
    assert_eq!(s.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// transition 简写使用 cubic-bezier 时序函数
fn test_transition_shorthand_with_cubic_bezier() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(
        &doc,
        div,
        &[("transition", "all 0.3s cubic-bezier(0.1, 0.7, 1.0, 0.1) 0.1s")],
    );
    assert!(!s.transition_property.is_empty());
}

#[test]
/// animation 简写使用名称和持续时间
fn test_animation_shorthand_with_name_and_duration() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("animation", "slide 2s ease-in-out infinite alternate")]);
    assert!(!s.animation_name.is_empty());
}

#[test]
/// columns 简写使用宽度和数量组合
fn test_columns_shorthand_with_width_and_count() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("columns", "3 200px")]);
    assert_eq!(
        s.column_count,
        crate::property::types::ColumnCountComputedValue::Number(3)
    );
    assert_eq!(
        s.column_width,
        crate::property::types::ColumnWidthComputedValue::Length(LengthValue::Px(200.0))
    );
}

#[test]
/// grid-area 简写测试 - 4 个值
fn test_grid_area_shorthand_four_values() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("grid-area", "1 / 2 / 3 / 4")]);
    assert_eq!(s.grid_row_start, crate::property::GridLineValue::Line(1));
    assert_eq!(s.grid_row_end, crate::property::GridLineValue::Line(3));
    assert_eq!(s.grid_column_start, crate::property::GridLineValue::Line(2));
    assert_eq!(s.grid_column_end, crate::property::GridLineValue::Line(4));
}

#[test]
/// text-decoration 简写使用 underline blue
fn test_text_decoration_shorthand_with_underline() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("text-decoration", "underline blue")]);
    assert!(matches!(
        s.text_decoration_line,
        crate::property::types::TextDecorationLineValue::Underline
    ));
}

#[test]
/// gap 简写测试 - 双值
fn test_gap_shorthand_double_value() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("gap", "10px 20px")]);
    assert_eq!(s.row_gap, LengthValue::Px(10.0));
    assert_eq!(s.column_gap, LengthValue::Px(20.0));
}

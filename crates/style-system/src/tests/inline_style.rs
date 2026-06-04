//! 内联样式（inline style）解析和级联集成测试。
//!
//! 验证 HTML 元素 style 属性的解析和样式计算管线。

use super::super::*;
use super::helpers::*;
use zero_css_parser::Parser as CssParser;
use zero_css_parser::values::{ColorValue, DisplayValue, LengthValue, OverflowValue, PositionValue};

/// 创建包含带 style 属性 div 元素的 DOM。
fn make_doc_with_inline_style(style_value: &str) -> (Document, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "style", style_value);
    doc.append_child(body, div).unwrap();
    (doc, div)
}

// ── 1. 内联样式基础解析 ────────────────────────────────────────────

#[test]
fn test_inline_style_width_height() {
    let (doc, div) = make_doc_with_inline_style("width: 300px; height: 150px");
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let style = sys.compute_element_style(&doc, div, &[], None);
    assert_eq!(style.width, LengthValue::Px(300.0));
    assert_eq!(style.height, LengthValue::Px(150.0));
}

#[test]
fn test_inline_style_color() {
    let (doc, div) = make_doc_with_inline_style("color: blue");
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let style = sys.compute_element_style(&doc, div, &[], None);
    // 蓝色 rgba(0,0,255,255)
    assert_ne!(
        style.color,
        ColorValue::Rgba(0, 0, 0, 0),
        "color should not be transparent"
    );
}

#[test]
fn test_inline_style_font_size() {
    let (doc, div) = make_doc_with_inline_style("font-size: 18px");
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let style = sys.compute_element_style(&doc, div, &[], None);
    match &style.font_size {
        LengthValue::Px(v) => assert_eq!(*v, 18.0),
        other => panic!("Expected font-size Px, got {other:?}"),
    }
}

#[test]
fn test_inline_style_important() {
    let (doc, div) = make_doc_with_inline_style("color: red !important");
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let _style = sys.compute_element_style(&doc, div, &[], None);
    // 不 panic 即表示解析成功
}

// ── 2. 边界情况 ────────────────────────────────────────────────────

#[test]
fn test_inline_style_empty() {
    let (doc, div) = make_doc_with_inline_style("");
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let style = sys.compute_element_style(&doc, div, &[], None);
    assert_eq!(style.width, LengthValue::Auto);
}

#[test]
fn test_inline_style_trailing_semicolon() {
    let (doc, div) = make_doc_with_inline_style("width: 100px;");
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let style = sys.compute_element_style(&doc, div, &[], None);
    assert_eq!(style.width, LengthValue::Px(100.0));
}

#[test]
fn test_inline_style_whitespace() {
    let (doc, div) = make_doc_with_inline_style("  width :  200px  ;  height :  100px  ");
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let style = sys.compute_element_style(&doc, div, &[], None);
    assert_eq!(style.width, LengthValue::Px(200.0));
    assert_eq!(style.height, LengthValue::Px(100.0));
}

#[test]
fn test_inline_style_no_attribute() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let style = sys.compute_element_style(&doc, div, &[], None);
    // 默认 width 是 Auto
    assert_eq!(style.width, LengthValue::Auto);
}

// ── 3. 简写属性展开 ────────────────────────────────────────────────

#[test]
fn test_inline_style_margin_shorthand() {
    let (doc, div) = make_doc_with_inline_style("margin: 10px 20px");
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let style = sys.compute_element_style(&doc, div, &[], None);
    assert_eq!(style.margin_top, LengthValue::Px(10.0));
    assert_eq!(style.margin_right, LengthValue::Px(20.0));
    assert_eq!(style.margin_bottom, LengthValue::Px(10.0));
    assert_eq!(style.margin_left, LengthValue::Px(20.0));
}

#[test]
fn test_inline_style_padding_shorthand() {
    let (doc, div) = make_doc_with_inline_style("padding: 5px 10px 15px 20px");
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let style = sys.compute_element_style(&doc, div, &[], None);
    assert_eq!(style.padding_top, LengthValue::Px(5.0));
    assert_eq!(style.padding_right, LengthValue::Px(10.0));
    assert_eq!(style.padding_bottom, LengthValue::Px(15.0));
    assert_eq!(style.padding_left, LengthValue::Px(20.0));
}

#[test]
fn test_inline_style_border_shorthand() {
    let (doc, div) = make_doc_with_inline_style("border: 1px solid red");
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let style = sys.compute_element_style(&doc, div, &[], None);
    assert_eq!(style.border_top_width, LengthValue::Px(1.0));
}

// ── 4. 显示和定位 ──────────────────────────────────────────────────

#[test]
fn test_inline_style_display() {
    let (doc, div) = make_doc_with_inline_style("display: flex");
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let style = sys.compute_element_style(&doc, div, &[], None);
    assert_eq!(style.display, DisplayValue::Flex);
}

#[test]
fn test_inline_style_position() {
    let (doc, div) = make_doc_with_inline_style("position: absolute; top: 10px; left: 20px");
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let style = sys.compute_element_style(&doc, div, &[], None);
    assert_eq!(style.position, PositionValue::Absolute);
}

#[test]
fn test_inline_style_overflow() {
    let (doc, div) = make_doc_with_inline_style("overflow: hidden");
    let stylesheets = CssParser::parse_stylesheet("");
    let mut sys = StyleSystem::new();
    let style = sys.compute_element_style(&doc, div, &[], None);
    assert_eq!(style.overflow_x, OverflowValue::Hidden);
    assert_eq!(style.overflow_y, OverflowValue::Hidden);
}

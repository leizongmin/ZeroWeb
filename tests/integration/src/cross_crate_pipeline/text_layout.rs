use std::collections::HashMap;

use zero_css_parser::Parser as CssParser;
use zero_css_parser::values::{
    AlignmentValue, ColorValue, DisplayValue, FlexDirectionValue, FlexWrapValue, FontWeightValue, LengthValue,
    OverflowValue, PositionValue, TransformFunction, TransformValue, parse_transform,
};
use zero_dom::Document;
use zero_engine::RenderPipeline;
use zero_layout_engine::LayoutEngine;
use zero_render_foundation::color::Color;
use zero_style_system::{ComputedStyle, GridLineValue, StyleSystem};

// ── 辅助函数 ──

/// 创建 html > body 基础 DOM，返回 (doc, body NodeId)。
fn make_doc_with_body() -> (Document, zero_dom::NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    (doc, body)
}

/// 在 LayoutBox 子树中查找指定 node_id 的盒子。
fn find_box_by_node_id(
    root: &zero_layout_engine::LayoutBox,
    target_id: zero_dom::NodeId,
) -> Option<&zero_layout_engine::LayoutBox> {
    if root.node_id == Some(target_id) {
        return Some(root);
    }
    for child in &root.children {
        if let Some(found) = find_box_by_node_id(child, target_id) {
            return Some(found);
        }
    }
    None
}

// ── 测试 ──

/// CSS Transform 管线集成测试。
///
/// 通过 css-parser 解析含多个变换函数的 transform 值，
/// 再由 style-system 计算样式，验证 ComputedStyle.transform 包含
/// rotate(45deg) → scale(2) → translate(10px, 20px) 三个函数且顺序正确。

#[test]
fn test_visibility_hidden_render_pipeline() {
    let html = r#"<div class="hidden">text</div>"#;
    let css = r#"
        .hidden { visibility: hidden; background-color: red; }
    "#;

    let result = RenderPipeline::new(800.0, 600.0).render_html(html, css);
    assert!(
        result.primitives().fills.is_empty(),
        "visibility:hidden 不应产生 fill 图元，实际有 {} 个",
        result.primitives().fills.len()
    );
}

/// CSS 多 transition 属性管线集成测试。
///
/// 通过 transition-property、transition-duration 长属性分别设置多个值，
/// 验证多 transition 管线正确存储。
#[test]
fn test_multiple_transitions_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "multi");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .multi {
            transition-property: opacity, transform;
            transition-duration: 0.3s, 0.5s;
        }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert!(
        div_style.transition_property.contains(&"opacity".to_string()),
        "transition-property 应包含 opacity，实际为 {:?}",
        div_style.transition_property
    );
    assert!(
        div_style.transition_property.contains(&"transform".to_string()),
        "transition-property 应包含 transform，实际为 {:?}",
        div_style.transition_property
    );
    assert!(
        div_style.transition_duration.contains(&0.3),
        "transition-duration 应包含 0.3，实际为 {:?}",
        div_style.transition_duration
    );
    assert!(
        div_style.transition_duration.contains(&0.5),
        "transition-duration 应包含 0.5，实际为 {:?}",
        div_style.transition_duration
    );
}

// ── CSS 表格/布局/字体 变体属性管线集成测试 ──

/// CSS table-layout 管线集成测试。
///
/// 解析 table-layout: fixed，验证计算样式。
#[test]
fn test_table_layout_fixed_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let table = doc.create_element("table");
    doc.set_attribute(table, "class", "fixed-layout");
    doc.append_child(body, table).unwrap();

    let css = r#"
        .fixed-layout { table-layout: fixed; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let table_style = styles.get(&table).expect("table 应有计算样式");
    assert_eq!(
        table_style.table_layout,
        zero_style_system::property::TableLayoutValue::Fixed,
        "table-layout 应为 Fixed"
    );
}

/// CSS caption-side 管线集成测试。
///
/// 解析 caption-side: bottom，验证计算样式。
#[test]
fn test_caption_side_bottom_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let caption = doc.create_element("caption");
    doc.set_attribute(caption, "class", "bottom-cap");
    doc.append_child(body, caption).unwrap();

    let css = r#"
        .bottom-cap { caption-side: bottom; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let cap_style = styles.get(&caption).expect("caption 应有计算样式");
    assert_eq!(
        cap_style.caption_side,
        zero_style_system::property::CaptionSideValue::Bottom,
        "caption-side 应为 Bottom"
    );
}

/// CSS border-collapse 管线集成测试。
///
/// 解析 border-collapse: collapse，验证计算样式。
#[test]
fn test_border_collapse_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let table = doc.create_element("table");
    doc.set_attribute(table, "class", "collapse");
    doc.append_child(body, table).unwrap();

    let css = r#"
        .collapse { border-collapse: collapse; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let table_style = styles.get(&table).expect("table 应有计算样式");
    assert_eq!(
        table_style.border_collapse,
        zero_style_system::property::BorderCollapseValue::Collapse,
        "border-collapse 应为 Collapse"
    );
}

/// CSS resize 管线集成测试。
///
/// 解析 resize: both，验证计算样式。
#[test]
fn test_resize_both_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "resizable");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .resizable { resize: both; overflow: auto; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.resize,
        zero_style_system::property::ResizeValue::Both,
        "resize 应为 Both"
    );
}

/// CSS word-break 管线集成测试。
///
/// 解析 word-break: break-all，验证计算样式。
#[test]
fn test_word_break_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "break-all");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .break-all { word-break: break-all; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.word_break,
        zero_style_system::property::WordBreakValue::BreakAll,
        "word-break 应为 BreakAll"
    );
}

/// CSS writing-mode 管线集成测试。
///
/// 解析 writing-mode: vertical-rl，验证计算样式。
#[test]
fn test_writing_mode_vertical_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "vertical");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .vertical { writing-mode: vertical-rl; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.writing_mode,
        zero_style_system::property::WritingModeValue::VerticalRl,
        "writing-mode 应为 VerticalRl"
    );
}

/// CSS isolation 管线集成测试。
///
/// 解析 isolation: isolate，验证计算样式。
#[test]
fn test_isolation_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "isolated");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .isolated { isolation: isolate; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.isolation,
        zero_style_system::property::IsolationValue::Isolate,
        "isolation 应为 Isolate"
    );
}

/// CSS isolation 继承性验证。
///
/// isolation 不继承，子元素应默认为 Auto。
#[test]
fn test_isolation_not_inherited_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "isolated");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .isolated { isolation: isolate; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert_eq!(
        child_style.isolation,
        zero_style_system::property::IsolationValue::Auto,
        "isolation 不应继承，子元素应为 Auto"
    );
}

// ── CSS flexbox / 字体 / 自定义属性 / overflow 管线集成测试 ──

/// CSS flex-direction: column 管线集成测试。
///
/// 解析 display: flex; flex-direction: column，通过 style-system 计算样式，
/// 验证 ComputedStyle 中 display 为 Flex、flex_direction 为 Column。
#[test]
fn test_flex_direction_column_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "col-flex");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .col-flex { display: flex; flex-direction: column; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(div_style.display, DisplayValue::Flex, "display 应为 Flex");
    assert_eq!(
        div_style.flex_direction,
        FlexDirectionValue::Column,
        "flex-direction 应为 Column"
    );
}

/// CSS justify-content: center 管线集成测试。
///
/// 解析 display: flex; justify-content: center，通过 style-system 计算样式，
/// 验证 ComputedStyle 中 justify_content 为 Center。
#[test]
fn test_flex_justify_center_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "centered");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .centered { display: flex; justify-content: center; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.justify_content,
        AlignmentValue::Center,
        "justify-content 应为 Center"
    );
}

/// CSS align-items: stretch 管线集成测试。
///
/// 解析 display: flex; align-items: stretch，通过 style-system 计算样式，
/// 验证 ComputedStyle 中 align_items 为 Stretch。
#[test]
fn test_flex_align_items_stretch_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "stretch");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .stretch { display: flex; align-items: stretch; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.align_items,
        AlignmentValue::Stretch,
        "align-items 应为 Stretch"
    );
}

/// CSS flex-wrap: wrap 管线集成测试。
///
/// 解析 display: flex; flex-wrap: wrap，通过 style-system 计算样式，
/// 验证 ComputedStyle 中 flex_wrap 为 Wrap。
#[test]
fn test_flex_wrap_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "wrap");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .wrap { display: flex; flex-wrap: wrap; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(div_style.flex_wrap, FlexWrapValue::Wrap, "flex-wrap 应为 Wrap");
}

/// CSS font-family 管线集成测试。
///
/// 解析 font-family: Arial, sans-serif，通过 style-system 计算样式，
/// 验证 ComputedStyle 中 font_family 为包含 "Arial" 和 "sans-serif" 的 Vec。
#[test]
fn test_font_family_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "fonted");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .fonted { font-family: Arial, sans-serif; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert!(
        div_style.font_family.contains(&"Arial".to_string()),
        "font-family 应包含 Arial，实际为 {:?}",
        div_style.font_family
    );
    assert!(
        div_style.font_family.contains(&"sans-serif".to_string()),
        "font-family 应包含 sans-serif，实际为 {:?}",
        div_style.font_family
    );
}

/// CSS font-weight: bold 管线集成测试。
///
/// 解析 font-weight: bold，通过 style-system 计算样式，
/// 验证 ComputedStyle 中 font_weight 为 FontWeightValue::Bold。
#[test]
fn test_font_weight_bold_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "bold");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .bold { font-weight: bold; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(div_style.font_weight, FontWeightValue::Bold, "font-weight 应为 Bold");
}

/// CSS line-height 数值管线集成测试。
///
/// 解析 line-height: 1.5，通过 style-system 计算样式，
/// 验证 ComputedStyle 中 line_height 为 Number(1.5)。
#[test]
fn test_line_height_number_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "lh");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .lh { line-height: 1.5; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.line_height,
        zero_style_system::property::LineHeightValue::Number(1.5),
        "line-height 应为 Number(1.5)"
    );
}

/// CSS 自定义属性 var() 回退值管线集成测试。
///
/// 定义 --x: red，通过 var(--y, blue) 引用未定义变量 --y，
/// 验证 color 使用回退值 blue（即 ColorValue::Rgba(0, 0, 255, 255)）。
#[test]
fn test_custom_property_var_fallback_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "a");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .a { --x: red; color: var(--y, blue); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.color,
        ColorValue::Rgba(0, 0, 255, 255),
        "color 应回退为蓝色 (0, 0, 255, 255)，实际为 {:?}",
        div_style.color
    );
}

/// CSS overflow 双值简写管线集成测试。
///
/// 解析 overflow: hidden scroll，通过 style-system 简写展开，
/// 验证 overflow_x 为 Hidden、overflow_y 为 Scroll。
#[test]
fn test_overflow_shorthand_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "overflowed");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .overflowed { overflow: hidden scroll; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(div_style.overflow_x, OverflowValue::Hidden, "overflow-x 应为 Hidden");
    assert_eq!(div_style.overflow_y, OverflowValue::Scroll, "overflow-y 应为 Scroll");
}

// ── CSS Grid / Position / Box Model 管线集成测试 ──

/// CSS grid-template-columns 管线集成测试。
///
/// 解析含 display: grid; grid-template-columns: 1fr 2fr 100px 的 CSS，
/// 通过 style-system 计算样式，验证 grid_template_columns 为 Some 且包含预期值。
#[test]
fn test_grid_template_columns_pipeline() {
    let (mut doc, body) = make_doc_with_body();

    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "grid");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .grid { display: grid; grid-template-columns: 1fr 2fr 100px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(div_style.display, DisplayValue::Grid, "div 的 display 应为 Grid");
    assert!(
        div_style.grid_template_columns.is_some(),
        "grid_template_columns 不应为 None"
    );
    let cols = div_style.grid_template_columns.as_ref().unwrap();
    assert!(cols.contains("1fr"), "grid_template_columns 应包含 1fr");
    assert!(cols.contains("2fr"), "grid_template_columns 应包含 2fr");
    assert!(cols.contains("100px"), "grid_template_columns 应包含 100px");
}

/// CSS grid-template-rows 管线集成测试。
///
/// 解析含 display: grid; grid-template-rows: auto 200px 的 CSS，
/// 通过 style-system 计算样式，验证 grid_template_rows 为 Some 且包含预期值。
#[test]
fn test_grid_template_rows_pipeline() {
    let (mut doc, body) = make_doc_with_body();

    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "grid");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .grid { display: grid; grid-template-rows: auto 200px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(div_style.display, DisplayValue::Grid, "div 的 display 应为 Grid");
    assert!(div_style.grid_template_rows.is_some(), "grid_template_rows 不应为 None");
    let rows = div_style.grid_template_rows.as_ref().unwrap();
    assert!(rows.contains("auto"), "grid_template_rows 应包含 auto");
    assert!(rows.contains("200px"), "grid_template_rows 应包含 200px");
}

/// CSS grid-auto-flow 管线集成测试。
///
/// 解析含 display: grid; grid-auto-flow: dense 的 CSS，
/// 通过 style-system 计算样式，验证 grid_auto_flow 为 RowDense（dense 等价于 row dense）。
#[test]
fn test_grid_auto_flow_pipeline() {
    let (mut doc, body) = make_doc_with_body();

    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "grid");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .grid { display: grid; grid-auto-flow: dense; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(div_style.display, DisplayValue::Grid, "div 的 display 应为 Grid");
    assert!(
        div_style.grid_auto_flow == zero_style_system::property::GridAutoFlowValue::RowDense
            || div_style.grid_auto_flow == zero_style_system::property::GridAutoFlowValue::ColumnDense,
        "grid_auto_flow 应为 RowDense 或 ColumnDense，实际为 {:?}",
        div_style.grid_auto_flow
    );
}

/// CSS display: grid 管线集成测试。
///
/// 解析含 display: grid 的 CSS，通过 style-system 计算样式，
/// 验证 display == DisplayValue::Grid。
#[test]
fn test_display_grid_pipeline() {
    let (mut doc, body) = make_doc_with_body();

    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "container");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .container { display: grid; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(div_style.display, DisplayValue::Grid, "div 的 display 应为 Grid");
}

/// CSS position: absolute 管线集成测试。
///
/// 解析含 position: absolute; top: 10px; left: 20px 的 CSS，
/// 通过 style-system 计算样式，验证 position 为 Absolute，
/// top 和 left 为 Px(10.0) 和 Px(20.0)。
#[test]
fn test_position_absolute_pipeline() {
    let (mut doc, body) = make_doc_with_body();

    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "abs");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .abs { position: absolute; top: 10px; left: 20px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.position,
        PositionValue::Absolute,
        "div 的 position 应为 Absolute"
    );
    assert_eq!(div_style.top, LengthValue::Px(10.0), "div 的 top 应为 Px(10.0)");
    assert_eq!(div_style.left, LengthValue::Px(20.0), "div 的 left 应为 Px(20.0)");
}

/// CSS margin 简写管线集成测试。
///
/// 解析含 margin: 10px 20px 的 CSS，通过 style-system 简写展开，
/// 验证 margin_top 为 Px(10.0)，margin_right 为 Px(20.0)。
#[test]
fn test_margin_shorthand_pipeline() {
    let (mut doc, body) = make_doc_with_body();

    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "spaced");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .spaced { margin: 10px 20px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(div_style.margin_top, LengthValue::Px(10.0), "margin_top 应为 Px(10.0)");
    assert_eq!(
        div_style.margin_right,
        LengthValue::Px(20.0),
        "margin_right 应为 Px(20.0)"
    );
}

/// CSS padding 简写管线集成测试。
///
/// 解析含 padding: 5px 15px 的 CSS，通过 style-system 简写展开，
/// 验证 padding_top 为 Px(5.0)，padding_right 为 Px(15.0)。
#[test]
fn test_padding_shorthand_pipeline() {
    let (mut doc, body) = make_doc_with_body();

    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "padded");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .padded { padding: 5px 15px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(div_style.padding_top, LengthValue::Px(5.0), "padding_top 应为 Px(5.0)");
    assert_eq!(
        div_style.padding_right,
        LengthValue::Px(15.0),
        "padding_right 应为 Px(15.0)"
    );
}

/// CSS width + height 管线集成测试。
///
/// 解析含 width: 300px; height: 200px 的 CSS，
/// 通过 style-system 计算样式，验证 width 和 height 正确设置。
#[test]
fn test_width_height_pipeline() {
    let (mut doc, body) = make_doc_with_body();

    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "sized");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .sized { width: 300px; height: 200px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(div_style.width, LengthValue::Px(300.0), "width 应为 Px(300.0)");
    assert_eq!(div_style.height, LengthValue::Px(200.0), "height 应为 Px(200.0)");
}

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
fn test_gradient_with_background_color_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "combo");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .combo {
            background-color: white;
            background-image: linear-gradient(red, blue);
        }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");

    // 验证 background-color 为白色
    assert_eq!(
        div_style.background_color,
        zero_css_parser::values::ColorValue::Rgba(255, 255, 255, 255),
        "background-color 应为 white (255, 255, 255, 255)"
    );

    // 验证 background-image 为渐变
    assert!(
        matches!(
            &div_style.background_image[0],
            zero_style_system::property::BackgroundImageComputedValue::Gradient(_)
        ),
        "background_image 应为 Gradient 变体"
    );
}

/// CSS linear-gradient 角度方向管线测试。
///
/// 解析 background-image: linear-gradient(90deg, red, green, blue)，
/// 验证方向为 Angle(90.0)。
#[test]
fn test_linear_gradient_angle_direction_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "angle-grad");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .angle-grad { background-image: linear-gradient(90deg, red, green, blue); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    match &div_style.background_image[0] {
        zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => match grad {
            zero_css_parser::values::GradientValue::Linear(lin) => {
                match &lin.direction {
                    zero_css_parser::values::GradientDirection::Angle(a) => {
                        assert!((a - 90.0).abs() < 0.01, "方向应为 Angle(90.0)，实际为 Angle({})", a);
                    }
                    other => panic!("方向应为 Angle 变体，实际为 {:?}", other),
                }
                assert_eq!(lin.stops.len(), 3, "应有 3 个色标");
            }
            other => panic!("渐变应为 Linear，实际为 {:?}", other),
        },
        other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
    }
}

/// CSS radial-gradient 自定义位置管线测试。
///
/// 解析 background-image: radial-gradient(circle at 25% 75%, red, blue)，
/// 验证 position_x 和 position_y 匹配预期值。
#[test]
fn test_radial_gradient_position_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "pos-grad");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .pos-grad { background-image: radial-gradient(circle at 25% 75%, red, blue); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    match &div_style.background_image[0] {
        zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => {
            match grad {
                zero_css_parser::values::GradientValue::Radial(rad) => {
                    // position_x 应为 25% → Percent(25.0)
                    assert!(
                        matches!(&rad.position_x, zero_css_parser::values::LengthValue::Percentage(p) if (*p - 25.0).abs() < 0.01),
                        "position_x 应为 Percent(25.0)，实际为 {:?}",
                        rad.position_x
                    );
                    // position_y 应为 75% → Percent(75.0)
                    assert!(
                        matches!(&rad.position_y, zero_css_parser::values::LengthValue::Percentage(p) if (*p - 75.0).abs() < 0.01),
                        "position_y 应为 Percent(75.0)，实际为 {:?}",
                        rad.position_y
                    );
                }
                other => panic!("渐变应为 Radial，实际为 {:?}", other),
            }
        }
        other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
    }
}

/// CSS linear-gradient 多色标管线测试。
///
/// 解析 background-image: linear-gradient(to right, red 0%, green 50%, blue 100%)，
/// 验证有 3 个色标且位置正确。
#[test]
fn test_linear_gradient_multi_stop_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "multi-stop");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .multi-stop { background-image: linear-gradient(to right, red 0%, green 50%, blue 100%); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    match &div_style.background_image[0] {
        zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => {
            match grad {
                zero_css_parser::values::GradientValue::Linear(lin) => {
                    assert_eq!(lin.stops.len(), 3, "应有 3 个色标");

                    // 验证第一个色标：red 0%
                    assert_eq!(
                        lin.stops[0].color,
                        zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
                        "第一个色标颜色应为红色"
                    );
                    assert!(
                        matches!(&lin.stops[0].position, Some(zero_css_parser::values::LengthValue::Percentage(p)) if (*p - 0.0).abs() < 0.01),
                        "第一个色标位置应为 0%"
                    );

                    // 验证第二个色标：green 50%
                    assert_eq!(
                        lin.stops[1].color,
                        zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255),
                        "第二个色标颜色应为绿色"
                    );
                    assert!(
                        matches!(&lin.stops[1].position, Some(zero_css_parser::values::LengthValue::Percentage(p)) if (*p - 50.0).abs() < 0.01),
                        "第二个色标位置应为 50%"
                    );

                    // 验证第三个色标：blue 100%
                    assert_eq!(
                        lin.stops[2].color,
                        zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255),
                        "第三个色标颜色应为蓝色"
                    );
                    assert!(
                        matches!(&lin.stops[2].position, Some(zero_css_parser::values::LengthValue::Percentage(p)) if (*p - 100.0).abs() < 0.01),
                        "第三个色标位置应为 100%"
                    );
                }
                other => panic!("渐变应为 Linear，实际为 {:?}", other),
            }
        }
        other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
    }
}

// ── CSS opacity / text-decoration / text-transform 管线集成测试 ──

/// CSS opacity 管线集成测试。
///
/// 解析含 opacity: 0.5 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.opacity == 0.5。
#[test]
fn test_opacity_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "semi");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .semi { opacity: 0.5; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert!(
        (div_style.opacity - 0.5).abs() < 0.01,
        "div 的 opacity 应为 0.5，实际为 {}",
        div_style.opacity
    );
}

/// CSS opacity 默认值管线测试。
///
/// 不设置 opacity 时，默认值应为 1.0。
#[test]
fn test_opacity_default_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "plain");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .plain { color: black; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert!(
        (div_style.opacity - 1.0).abs() < 0.01,
        "未设置 opacity 时默认应为 1.0，实际为 {}",
        div_style.opacity
    );
}

/// CSS text-decoration: underline 管线集成测试。
///
/// 解析含 text-decoration: underline 的 CSS，通过简写展开
/// 设置 text-decoration-line，验证 text_decoration_line == Underline。
#[test]
fn test_text_decoration_underline_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "underlined");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .underlined { text-decoration: underline; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.text_decoration_line,
        zero_style_system::property::TextDecorationLineValue::Underline,
        "div 的 text-decoration-line 应为 Underline"
    );
}

/// CSS text-decoration: line-through 管线集成测试。
///
/// 解析含 text-decoration: line-through 的 CSS，通过简写展开
/// 设置 text-decoration-line，验证 text_decoration_line == LineThrough。
#[test]
fn test_text_decoration_line_through_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "struck");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .struck { text-decoration: line-through; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.text_decoration_line,
        zero_style_system::property::TextDecorationLineValue::LineThrough,
        "div 的 text-decoration-line 应为 LineThrough"
    );
}

/// CSS text-decoration: none 管线集成测试。
///
/// 解析含 text-decoration: none 的 CSS，通过简写展开
/// 设置 text-decoration-line，验证 text_decoration_line == None。
#[test]
fn test_text_decoration_none_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "undecorated");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .undecorated { text-decoration: none; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.text_decoration_line,
        zero_style_system::property::TextDecorationLineValue::None,
        "div 的 text-decoration-line 应为 None"
    );
}

/// CSS text-transform: uppercase 管线集成测试。
///
/// 解析含 text-transform: uppercase 的 CSS，通过 style-system 计算样式，
/// 验证 text_transform == Uppercase。
#[test]
fn test_text_transform_uppercase_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "upper");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .upper { text-transform: uppercase; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.text_transform,
        zero_style_system::property::TextTransformValue::Uppercase,
        "div 的 text-transform 应为 Uppercase"
    );
}

/// CSS text-transform: capitalize 管线集成测试。
///
/// 解析含 text-transform: capitalize 的 CSS，通过 style-system 计算样式，
/// 验证 text_transform == Capitalize。
#[test]
fn test_text_transform_capitalize_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "capitalized");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .capitalized { text-transform: capitalize; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.text_transform,
        zero_style_system::property::TextTransformValue::Capitalize,
        "div 的 text-transform 应为 Capitalize"
    );
}

/// CSS text-transform 继承管线测试。
///
/// text-transform 是继承属性。父元素设置 text-transform: uppercase，
/// 子元素不显式设置，应继承父元素的 Uppercase 值。
#[test]
fn test_text_transform_inherited_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "upper-parent");
    doc.append_child(body, parent).unwrap();

    let child = doc.create_element("p");
    doc.set_attribute(child, "class", "child");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .upper-parent { text-transform: uppercase; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 父元素应有 Uppercase
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert_eq!(
        parent_style.text_transform,
        zero_style_system::property::TextTransformValue::Uppercase,
        "parent 的 text-transform 应为 Uppercase"
    );

    // 子元素应继承 text-transform: uppercase
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert_eq!(
        child_style.text_transform,
        zero_style_system::property::TextTransformValue::Uppercase,
        "child 应继承 parent 的 text-transform: Uppercase"
    );
}

/// CSS opacity 渲染管线完整测试。
///
/// 使用 RenderPipeline 渲染含 opacity: 0.5 和 background-color: red 的页面，
/// 验证渲染成功完成（timings.total_ms >= 0）。
#[test]
fn test_opacity_render_pipeline() {
    let html = r#"<html><body>
        <div class="semi" style="width: 200px; height: 100px;">Semi-transparent</div>
    </body></html>"#;
    let css = r#".semi { opacity: 0.5; background-color: red; }"#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    assert!(result.timings.total_ms >= 0.0, "opacity 渲染管线应成功完成");
    // 应生成填充图元（background-color: red）
    assert!(
        !result.primitives.fills.is_empty(),
        "background-color: red 应生成填充图元"
    );
}

/// CSS text-decoration + text-shadow 组合管线测试。
///
/// 同时设置 text-decoration: underline 和 text-shadow: 2px 2px red，
/// 验证两个属性都被正确设置到 computed style 中。
#[test]
fn test_text_decoration_with_text_shadow_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "combo");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .combo { text-decoration: underline; text-shadow: 2px 2px red; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");

    // 验证 text-decoration-line 为 Underline
    assert_eq!(
        div_style.text_decoration_line,
        zero_style_system::property::TextDecorationLineValue::Underline,
        "div 的 text-decoration-line 应为 Underline"
    );

    // 验证 text-shadow 的 offset_x 和 offset_y
    assert!(
        (div_style.text_shadow.offset_x - 2.0).abs() < 0.01,
        "text-shadow offset_x 应为 2.0，实际为 {}",
        div_style.text_shadow.offset_x
    );
    assert!(
        (div_style.text_shadow.offset_y - 2.0).abs() < 0.01,
        "text-shadow offset_y 应为 2.0，实际为 {}",
        div_style.text_shadow.offset_y
    );
    assert_eq!(
        div_style.text_shadow.color,
        zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
        "text-shadow color 应为红色"
    );
}

/// CSS opacity + box-shadow + gradient 组合管线测试。
///
/// 同时设置 opacity: 0.7、box-shadow: 5px 5px blue 和
/// background-image: linear-gradient(red, green)，
/// 验证三个属性都被正确设置到 computed style 中。
#[test]
fn test_opacity_shadow_gradient_combined_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "triple");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .triple {
            opacity: 0.7;
            box-shadow: 5px 5px blue;
            background-image: linear-gradient(red, green);
        }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");

    // 验证 opacity
    assert!(
        (div_style.opacity - 0.7).abs() < 0.01,
        "opacity 应为 0.7，实际为 {}",
        div_style.opacity
    );

    // 验证 box-shadow
    let s = &div_style.box_shadow[0];
    assert!(
        (s.offset_x - 5.0).abs() < 0.01,
        "box-shadow offset_x 应为 5.0，实际为 {}",
        s.offset_x
    );
    assert!(
        (s.offset_y - 5.0).abs() < 0.01,
        "box-shadow offset_y 应为 5.0，实际为 {}",
        s.offset_y
    );

    // 验证 background-image 为渐变
    assert!(
        matches!(
            &div_style.background_image[0],
            zero_style_system::property::BackgroundImageComputedValue::Gradient(_)
        ),
        "background_image 应为 Gradient 变体"
    );
}

/// CSS text-transform: lowercase 管线集成测试。
///
/// 解析含 text-transform: lowercase 的 CSS，通过 style-system 计算样式，
/// 验证 text_transform == Lowercase。
#[test]
fn test_text_transform_lowercase_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "lower");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .lower { text-transform: lowercase; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.text_transform,
        zero_style_system::property::TextTransformValue::Lowercase,
        "div 的 text-transform 应为 Lowercase"
    );
}

// ── CSS transition / animation / 自定义属性 / 交互 / 文本 管线集成测试 ──

/// CSS transition 简写管线集成测试。
///
/// 解析 transition: opacity 0.3s ease-in 0.1s，验证 4 个子属性正确展开。
#[test]
fn test_transition_shorthand_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "fade");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .fade { transition: opacity 0.3s ease-in 0.1s; }
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
        div_style.transition_duration.contains(&0.3),
        "transition-duration 应包含 0.3，实际为 {:?}",
        div_style.transition_duration
    );
    assert!(
        div_style.transition_delay.contains(&0.1),
        "transition-delay 应包含 0.1，实际为 {:?}",
        div_style.transition_delay
    );
    assert!(
        !div_style.transition_timing_function.is_empty(),
        "transition-timing-function 不应为空"
    );
}

/// CSS animation 简写管线集成测试。
///
/// 解析 animation: slideIn 1s ease 0.2s infinite forwards，验证子属性展开。
#[test]
fn test_animation_shorthand_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "animated");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .animated { animation: slideIn 1s ease 0.2s infinite forwards; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert!(
        div_style.animation_name.contains(&"slideIn".to_string()),
        "animation-name 应包含 slideIn，实际为 {:?}",
        div_style.animation_name
    );
    assert!(
        div_style.animation_duration.contains(&1.0),
        "animation-duration 应包含 1.0，实际为 {:?}",
        div_style.animation_duration
    );
    assert!(
        div_style.animation_delay.contains(&0.2),
        "animation-delay 应包含 0.2，实际为 {:?}",
        div_style.animation_delay
    );
}

/// CSS 自定义属性 + var() 管线集成测试。
///
/// 定义 --main-color: #ff0000，通过 var(--main-color) 引用到 color 属性。
#[test]
fn test_custom_property_var_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "themed");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .themed { --main-color: #ff0000; color: var(--main-color); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert!(
        matches!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)),
        "color 应通过 var() 解析为红色 #ff0000，实际为 {:?}",
        div_style.color
    );
}

/// CSS cursor 管线集成测试。
///
/// 解析 cursor: pointer，验证计算样式。
#[test]
fn test_cursor_pointer_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "clickable");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .clickable { cursor: pointer; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.cursor,
        zero_style_system::property::CursorValue::Pointer,
        "cursor 应为 Pointer"
    );
}

/// CSS cursor 继承管线集成测试。
///
/// 父元素 cursor: pointer，子元素应继承。
#[test]
fn test_cursor_inheritance_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "parent");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("span");
    doc.set_attribute(child, "class", "child");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .parent { cursor: move; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert_eq!(
        child_style.cursor,
        zero_style_system::property::CursorValue::Move,
        "cursor 应从父元素继承 Move"
    );
}

/// CSS pointer-events 管线集成测试。
///
/// 解析 pointer-events: none，验证计算样式。
#[test]
fn test_pointer_events_none_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "no-events");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .no-events { pointer-events: none; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.pointer_events,
        zero_style_system::property::PointerEventsValue::None,
        "pointer-events 应为 None"
    );
}

/// CSS white-space 管线集成测试。
///
/// 解析 white-space: pre-wrap，验证计算样式。
#[test]
fn test_white_space_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let pre = doc.create_element("pre");
    doc.set_attribute(pre, "class", "code");
    doc.append_child(body, pre).unwrap();

    let css = r#"
        .code { white-space: pre-wrap; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let pre_style = styles.get(&pre).expect("pre 应有计算样式");
    assert_eq!(
        pre_style.white_space,
        zero_style_system::property::WhiteSpaceValue::PreWrap,
        "white-space 应为 PreWrap"
    );
}

/// CSS letter-spacing 管线集成测试。
///
/// 解析 letter-spacing: 2px，验证计算样式为 Px(2.0)。
#[test]
fn test_letter_spacing_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "spaced");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .spaced { letter-spacing: 2px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.letter_spacing,
        LengthValue::Px(2.0),
        "letter-spacing 应为 2px"
    );
}

/// CSS letter-spacing 继承管线集成测试。
///
/// 父元素 letter-spacing: 3px，子元素应继承。
#[test]
fn test_letter_spacing_inheritance_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "wide");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("span");
    doc.set_attribute(child, "class", "inner");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .wide { letter-spacing: 3px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert_eq!(
        child_style.letter_spacing,
        LengthValue::Px(3.0),
        "letter-spacing 应从父元素继承 3px"
    );
}

/// CSS white-space 继承管线集成测试。
///
/// 父元素 white-space: nowrap，子元素应继承。
#[test]
fn test_white_space_inheritance_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "nowrap");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .nowrap { white-space: nowrap; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert_eq!(
        child_style.white_space,
        zero_style_system::property::WhiteSpaceValue::Nowrap,
        "white-space 应从父元素继承 Nowrap"
    );
}

/// CSS user-select 管线集成测试。
///
/// 解析 user-select: none，验证计算样式。
#[test]
fn test_user_select_none_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "noselect");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .noselect { user-select: none; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.user_select,
        zero_style_system::property::UserSelectValue::None,
        "user-select 应为 None"
    );
}

/// CSS text-decoration-style 管线集成测试。
///
/// 解析 text-decoration-style: dotted，验证样式系统计算值。
#[test]
fn test_text_decoration_style_dotted_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let span = doc.create_element("span");
    doc.set_attribute(span, "class", "dotted");
    doc.append_child(body, span).unwrap();

    let css = r#"
        .dotted { text-decoration: underline dotted; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let span_style = styles.get(&span).expect("span 应有计算样式");
    assert_eq!(
        span_style.text_decoration_line,
        zero_style_system::property::TextDecorationLineValue::Underline,
        "text-decoration-line 应为 Underline"
    );
    assert_eq!(
        span_style.text_decoration_style,
        zero_style_system::property::TextDecorationStyleValue::Dotted,
        "text-decoration-style 应为 Dotted"
    );
}

/// CSS text-decoration-style: dashed 管线集成测试。
#[test]
fn test_text_decoration_style_dashed_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let span = doc.create_element("span");
    doc.set_attribute(span, "class", "dashed");
    doc.append_child(body, span).unwrap();

    let css = r#"
        .dashed { text-decoration: line-through dashed red; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let span_style = styles.get(&span).expect("span 应有计算样式");
    assert_eq!(
        span_style.text_decoration_line,
        zero_style_system::property::TextDecorationLineValue::LineThrough,
        "text-decoration-line 应为 LineThrough"
    );
    assert_eq!(
        span_style.text_decoration_style,
        zero_style_system::property::TextDecorationStyleValue::Dashed,
        "text-decoration-style 应为 Dashed"
    );
}

/// CSS text-decoration-style: wavy 管线集成测试。
#[test]
fn test_text_decoration_style_wavy_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let span = doc.create_element("span");
    doc.set_attribute(span, "class", "wavy");
    doc.append_child(body, span).unwrap();

    let css = r#"
        .wavy { text-decoration-style: wavy; text-decoration-line: underline; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let span_style = styles.get(&span).expect("span 应有计算样式");
    assert_eq!(
        span_style.text_decoration_style,
        zero_style_system::property::TextDecorationStyleValue::Wavy,
        "text-decoration-style 应为 Wavy"
    );
}

/// CSS text-decoration-color 自定义颜色管线集成测试。
///
/// 通过长属性设置 text-decoration-color，验证样式系统计算值。
#[test]
fn test_text_decoration_color_custom_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let span = doc.create_element("span");
    doc.set_attribute(span, "class", "colored");
    doc.append_child(body, span).unwrap();

    let css = r#"
        .colored { text-decoration-line: underline; text-decoration-color: red; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let span_style = styles.get(&span).expect("span 应有计算样式");
    // red 解析为 Named("red")，apply 会转为 Rgba
    assert!(
        !matches!(
            span_style.text_decoration_color,
            zero_css_parser::values::ColorValue::CurrentColor
        ),
        "text-decoration-color 不应为 CurrentColor，应为 red"
    );
}

/// CSS text-decoration 简写（命名颜色）管线集成测试。
#[test]
fn test_text_decoration_shorthand_named_color_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let span = doc.create_element("span");
    doc.set_attribute(span, "class", "sh");
    doc.append_child(body, span).unwrap();

    let css = r#"
        .sh { text-decoration: underline dotted red; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let span_style = styles.get(&span).expect("span 应有计算样式");
    assert_eq!(
        span_style.text_decoration_line,
        zero_style_system::property::TextDecorationLineValue::Underline,
        "text-decoration-line 应为 Underline"
    );
    assert_eq!(
        span_style.text_decoration_style,
        zero_style_system::property::TextDecorationStyleValue::Dotted,
        "text-decoration-style 应为 Dotted"
    );
}

/// CSS text-decoration double 样式管线集成测试。
#[test]
fn test_text_decoration_style_double_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let span = doc.create_element("span");
    doc.set_attribute(span, "class", "dbl");
    doc.append_child(body, span).unwrap();

    let css = r#"
        .dbl { text-decoration: overline double blue; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let span_style = styles.get(&span).expect("span 应有计算样式");
    assert_eq!(
        span_style.text_decoration_line,
        zero_style_system::property::TextDecorationLineValue::Overline,
        "text-decoration-line 应为 Overline"
    );
    assert_eq!(
        span_style.text_decoration_style,
        zero_style_system::property::TextDecorationStyleValue::Double,
        "text-decoration-style 应为 Double"
    );
}

// ═══════════════════════════════════════════════════════════════
//  CSS quotes / scrollbar-gutter / background-attachment / hyphens 管线测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_quotes_pairs_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let q = doc.create_element("q");
    doc.append_child(body, q).unwrap();
    let text = doc.create_text_node("Hello");
    doc.append_child(q, text).unwrap();

    let css = r#"q { quotes: "\201C" "\201D" "\2018" "\2019"; }"#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let q_style = styles.get(&q).expect("q 应有计算样式");
    assert!(
        matches!(q_style.quotes, zero_style_system::QuotesComputedValue::Pairs(_)),
        "quotes 应为 Pairs"
    );
}

#[test]
fn test_scrollbar_gutter_stable_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let css = "div { scrollbar-gutter: stable; overflow: auto; width: 200px; height: 100px; }";
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.scrollbar_gutter,
        zero_style_system::ScrollbarGutterComputedValue::Stable,
        "scrollbar-gutter 应为 Stable"
    );
}

#[test]
fn test_background_attachment_fixed_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let css = "div { background-attachment: fixed; background-image: url(test.png); }";
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.background_attachment,
        zero_style_system::BackgroundAttachmentComputedValue::Fixed,
        "background-attachment 应为 Fixed"
    );
}

#[test]
fn test_hyphens_auto_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let p = doc.create_element("p");
    doc.append_child(body, p).unwrap();

    let css = "p { hyphens: auto; }";
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let p_style = styles.get(&p).expect("p 应有计算样式");
    assert_eq!(
        p_style.hyphens,
        zero_style_system::HyphensComputedValue::Auto,
        "hyphens 应为 Auto"
    );
}

#[test]
fn test_text_wrap_nowrap_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let css = "div { text-wrap: nowrap; }";
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.text_wrap,
        zero_style_system::TextWrapComputedValue::Nowrap,
        "text-wrap 应为 Nowrap"
    );
}

#[test]
fn test_line_clamp_pipeline() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let css = "div { line-clamp: 3; }";
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.line_clamp,
        zero_style_system::LineClampComputedValue::Count(3),
        "line-clamp 应为 Count(3)"
    );
}

// ── CSS 交互/提示属性管线集成测试（新增指示器渲染） ──

#[test]
fn test_image_rendering_pixelated_render_pipeline() {
    let html = r#"<html><body><img style="image-rendering: pixelated; width: 100px; height: 50px;"></body></html>"#;
    let css = "";
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);
    assert!(
        result.timings.total_ms >= 0.0,
        "image-rendering: pixelated 管线应成功完成"
    );
}

#[test]
fn test_isolation_isolate_render_pipeline() {
    let html = r#"<html><body><div style="isolation: isolate;">Stacking context</div></body></html>"#;
    let css = "";
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0, "isolation: isolate 管线应成功完成");
}

#[test]
fn test_will_change_transform_render_pipeline() {
    let html = r#"<html><body><div style="will-change: transform;">Animated</div></body></html>"#;
    let css = "";
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0, "will-change: transform 管线应成功完成");
}

#[test]
fn test_overscroll_behavior_contain_render_pipeline() {
    let html = r#"<html><body><div style="overscroll-behavior: contain;">Scroll</div></body></html>"#;
    let css = "";
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);
    assert!(
        result.timings.total_ms >= 0.0,
        "overscroll-behavior: contain 管线应成功完成"
    );
}

#[test]
fn test_touch_action_none_render_pipeline() {
    let html = r#"<html><body><div style="touch-action: none;">No touch</div></body></html>"#;
    let css = "";
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0, "touch-action: none 管线应成功完成");
}

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
fn test_line_clamp_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "clamped");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .clamped { line-clamp: 3; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.line_clamp,
        zero_style_system::property::LineClampComputedValue::Count(3),
        "div 的 line-clamp 应为 Count(3)"
    );
}

/// CSS background-image 管线集成测试。
///
/// 解析含 background-image: url(bg.png) 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.background_image 为 Url("bg.png")。
#[test]
fn test_background_image_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "bg-img");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .bg-img { background-image: url(bg.png); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        &div_style.background_image[..],
        &[zero_style_system::property::BackgroundImageComputedValue::Url(
            "bg.png".to_string()
        )],
        "div 的 background-image 应为 Url(\"bg.png\")"
    );
}

/// CSS background-repeat 管线集成测试。
///
/// 解析含 background-repeat: no-repeat 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.background_repeat 为 NoRepeat。
#[test]
fn test_background_repeat_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "no-repeat-bg");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .no-repeat-bg { background-repeat: no-repeat; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.background_repeat,
        zero_style_system::property::BackgroundRepeatComputedValue::NoRepeat,
        "div 的 background-repeat 应为 NoRepeat"
    );
}

/// CSS background-size 管线集成测试。
///
/// 解析含 background-size: cover 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.background_size 为 Cover。
#[test]
fn test_background_size_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "cover-bg");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .cover-bg { background-size: cover; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.background_size,
        zero_style_system::property::BackgroundSizeComputedValue::Cover,
        "div 的 background-size 应为 Cover"
    );
}

/// CSS background-attachment 管线集成测试。
///
/// 解析含 background-attachment: fixed 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.background_attachment 为 Fixed。
#[test]
fn test_background_attachment_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "fixed-bg");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .fixed-bg { background-attachment: fixed; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.background_attachment,
        zero_style_system::property::BackgroundAttachmentComputedValue::Fixed,
        "div 的 background-attachment 应为 Fixed"
    );
}

/// CSS background-clip 管线集成测试。
///
/// 解析含 background-clip: content-box 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.background_clip 为 ContentBox。
#[test]
fn test_background_clip_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "clip-bg");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .clip-bg { background-clip: content-box; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.background_clip,
        zero_style_system::property::BackgroundClipComputedValue::ContentBox,
        "div 的 background-clip 应为 ContentBox"
    );
}

/// CSS background-origin 管线集成测试。
///
/// 解析含 background-origin: padding-box 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.background_origin 为 PaddingBox。
#[test]
fn test_background_origin_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "origin-bg");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .origin-bg { background-origin: padding-box; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.background_origin,
        zero_style_system::property::BackgroundOriginComputedValue::PaddingBox,
        "div 的 background-origin 应为 PaddingBox"
    );
}

/// CSS accent-color 管线集成测试。
///
/// 解析含 accent-color: #ff0000 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.accent_color 为红色 (255, 0, 0) 的计算值。
#[test]
fn test_accent_color_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let input = doc.create_element("input");
    doc.set_attribute(input, "class", "accented");
    doc.append_child(body, input).unwrap();

    let css = r#"
        .accented { accent-color: #ff0000; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let input_style = styles.get(&input).expect("input 应有计算样式");
    match &input_style.accent_color {
        zero_style_system::property::AccentColorComputedValue::Color(color) => {
            assert_eq!(
                color,
                &zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
                "accent-color 应为红色 (255, 0, 0, 255)"
            );
        }
        other => panic!("accent-color 应为 Color 变体，实际为 {:?}", other),
    }
}

/// CSS border-image-source 管线集成测试。
///
/// 解析含 border-image-source: url(border.png) 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.border_image_source 为 Url 计算值。
#[test]
fn test_border_image_source_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "bordered");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .bordered { border-image-source: url(border.png); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.border_image_source,
        zero_style_system::property::BorderImageSourceComputedValue::Url("border.png".to_string()),
        "div 的 border-image-source 应为 Url(border.png)"
    );
}

/// CSS border-image-slice 管线集成测试。
#[test]
fn test_border_image_slice_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "sliced");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .sliced { border-image-slice: 30 40 fill; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    use zero_style_system::property::BorderImageSliceComputedComponent;
    assert_eq!(
        div_style.border_image_slice.top,
        BorderImageSliceComputedComponent::Number(30.0),
        "slice top 应为 30"
    );
    assert_eq!(
        div_style.border_image_slice.right,
        BorderImageSliceComputedComponent::Number(40.0),
        "slice right 应为 40"
    );
    assert!(div_style.border_image_slice.fill, "slice fill 应为 true");
}

/// CSS border-image-repeat 管线集成测试。
#[test]
fn test_border_image_repeat_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "repeated");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .repeated { border-image-repeat: round space; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    use zero_style_system::property::BorderImageRepeatComputedMode;
    assert_eq!(
        div_style.border_image_repeat.horizontal,
        BorderImageRepeatComputedMode::Round,
        "repeat 水平应为 Round"
    );
    assert_eq!(
        div_style.border_image_repeat.vertical,
        BorderImageRepeatComputedMode::Space,
        "repeat 垂直应为 Space"
    );
}

/// CSS border-image-width 管线集成测试。
#[test]
fn test_border_image_width_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "widthed");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .widthed { border-image-width: 2 10px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    use zero_style_system::property::BorderImageWidthComputedComponent;
    assert_eq!(
        div_style.border_image_width.top,
        BorderImageWidthComputedComponent::Number(2.0),
        "width top 应为 Number(2.0)"
    );
    assert_eq!(
        div_style.border_image_width.right,
        BorderImageWidthComputedComponent::Length(10.0),
        "width right 应为 Length(10.0)"
    );
}

/// CSS border-image-outset 管线集成测试。
#[test]
fn test_border_image_outset_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "outset");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .outset { border-image-outset: 5px 2; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    use zero_style_system::property::BorderImageOutsetComputedComponent;
    assert_eq!(
        div_style.border_image_outset.top,
        BorderImageOutsetComputedComponent::Length(5.0),
        "outset top 应为 Length(5.0)"
    );
    assert_eq!(
        div_style.border_image_outset.right,
        BorderImageOutsetComputedComponent::Number(2.0),
        "outset right 应为 Number(2.0)"
    );
}

/// CSS text-shadow 管线集成测试。
///
/// 解析含 text-shadow: 2px 3px red 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.text_shadow 的 offset_x=2.0, offset_y=3.0, color 为红色。
#[test]
fn test_text_shadow_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "shadowed");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .shadowed { text-shadow: 2px 3px red; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert!(
        (div_style.text_shadow[0].offset_x - 2.0).abs() < 0.01,
        "text-shadow offset_x 应为 2.0，实际为 {}",
        div_style.text_shadow[0].offset_x
    );
    assert!(
        (div_style.text_shadow[0].offset_y - 3.0).abs() < 0.01,
        "text-shadow offset_y 应为 3.0，实际为 {}",
        div_style.text_shadow[0].offset_y
    );
    assert_eq!(
        div_style.text_shadow[0].color,
        zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
        "text-shadow color 应为红色 (255, 0, 0, 255)"
    );
}

/// CSS box-shadow 管线集成测试。
///
/// 解析含 box-shadow: 5px 10px 20px blue 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.box_shadow 的 offset_x=5.0, offset_y=10.0, blur_radius=20.0。
#[test]
fn test_box_shadow_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "box-shadowed");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .box-shadowed { box-shadow: 5px 10px 20px blue; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    let s = &div_style.box_shadow[0];
    assert!(
        (s.offset_x - 5.0).abs() < 0.01,
        "box-shadow offset_x 应为 5.0，实际为 {}",
        s.offset_x
    );
    assert!(
        (s.offset_y - 10.0).abs() < 0.01,
        "box-shadow offset_y 应为 10.0，实际为 {}",
        s.offset_y
    );
    assert!(
        (s.blur_radius - 20.0).abs() < 0.01,
        "box-shadow blur_radius 应为 20.0，实际为 {}",
        s.blur_radius
    );
}

/// CSS box-shadow inset 管线集成测试。
///
/// 解析含 box-shadow: inset 3px 4px green 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.box_shadow 的 inset=true。
#[test]
fn test_box_shadow_inset_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "inset-shadow");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .inset-shadow { box-shadow: inset 3px 4px green; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    let s = &div_style.box_shadow[0];
    assert!(s.inset, "box-shadow inset 应为 true");
    assert!(
        (s.offset_x - 3.0).abs() < 0.01,
        "box-shadow offset_x 应为 3.0，实际为 {}",
        s.offset_x
    );
    assert!(
        (s.offset_y - 4.0).abs() < 0.01,
        "box-shadow offset_y 应为 4.0，实际为 {}",
        s.offset_y
    );
}

/// CSS text-shadow 继承集成测试。
///
/// 父元素设置 text-shadow，子元素不显式设置，
/// 验证子元素继承了父元素的 text-shadow 值（text-shadow 是继承属性）。
#[test]
fn test_text_shadow_inheritance_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    // 父元素设置 text-shadow
    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "shadowed");
    doc.append_child(body, parent).unwrap();

    // 子元素不设置 text-shadow，应继承
    let child = doc.create_element("p");
    doc.set_attribute(child, "class", "inner");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .shadowed { text-shadow: 2px 3px red; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 验证父元素的 text-shadow
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert!(
        (parent_style.text_shadow[0].offset_x - 2.0).abs() < 0.01,
        "parent text-shadow offset_x 应为 2.0"
    );

    // 验证子元素继承了 text-shadow
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert!(
        (child_style.text_shadow[0].offset_x - 2.0).abs() < 0.01,
        "child 应继承 parent 的 text-shadow offset_x=2.0，实际为 {}",
        child_style.text_shadow[0].offset_x
    );
    assert!(
        (child_style.text_shadow[0].offset_y - 3.0).abs() < 0.01,
        "child 应继承 parent 的 text-shadow offset_y=3.0，实际为 {}",
        child_style.text_shadow[0].offset_y
    );
    assert_eq!(
        child_style.text_shadow[0].color,
        zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
        "child 应继承 parent 的 text-shadow color 为红色"
    );
}

/// CSS outline-width 管线集成测试。
///
/// 解析含 outline-width: 3px 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.outline_width 为 Px(3.0)。
#[test]
fn test_outline_width_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "outlined");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .outlined { outline-width: 3px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.outline_width,
        zero_css_parser::values::LengthValue::Px(3.0),
        "div 的 outline-width 应为 Px(3.0)"
    );
}

/// CSS list-style-image 管线集成测试。
///
/// 解析含 list-style-image: url(marker.png) 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.list_style_image 为 Url("marker.png")。
#[test]
fn test_list_style_image_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let li = doc.create_element("li");
    doc.set_attribute(li, "class", "item");
    doc.append_child(body, li).unwrap();

    let css = r#"
        .item { list-style-image: url(marker.png); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let li_style = styles.get(&li).expect("li 应有计算样式");
    assert_eq!(
        li_style.list_style_image,
        zero_style_system::property::ListStyleImageComputedValue::Url("marker.png".to_string()),
        "li 的 list-style-image 应为 Url(\"marker.png\")"
    );
}

/// CSS column-gap 管线集成测试。
///
/// 解析含 column-gap: 30px 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.column_gap 为 Px(30.0)。
#[test]
fn test_column_gap_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "gap-container");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .gap-container { column-gap: 30px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.column_gap,
        LengthValue::Px(30.0),
        "div 的 column-gap 应为 Px(30.0)"
    );
}

/// CSS text-shadow 继承管线集成测试。
///
/// 父元素 .shadowed 设置 text-shadow: 2px 2px red，
/// 子元素 .inner 不显式设置，应继承父元素的 text-shadow（text-shadow 是继承属性）。
/// 验证子元素的 text_shadow.offset_x == 2.0。
#[test]
fn test_text_shadow_inheritance_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "shadowed");
    doc.append_child(body, parent).unwrap();

    let child = doc.create_element("p");
    doc.set_attribute(child, "class", "inner");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .shadowed { text-shadow: 2px 2px red; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 验证父元素的 text-shadow
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert!(
        (parent_style.text_shadow[0].offset_x - 2.0).abs() < 0.01,
        "parent text-shadow offset_x 应为 2.0"
    );

    // 验证子元素继承了 text-shadow
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert!(
        (child_style.text_shadow[0].offset_x - 2.0).abs() < 0.01,
        "child 应继承 parent 的 text-shadow offset_x=2.0，实际为 {}",
        child_style.text_shadow[0].offset_x
    );
}

/// CSS box-shadow 不继承管线集成测试。
///
/// 父元素 .shadowed 设置 box-shadow: 5px 5px blue，
/// 子元素 .inner 不显式设置，不应继承父元素的 box-shadow（box-shadow 不是继承属性）。
/// 验证子元素的 box_shadow.offset_x == 0.0（默认值）。
#[test]
fn test_box_shadow_not_inherited_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "shadowed");
    doc.append_child(body, parent).unwrap();

    let child = doc.create_element("p");
    doc.set_attribute(child, "class", "inner");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .shadowed { box-shadow: 5px 5px blue; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 验证父元素的 box-shadow
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert!(
        (parent_style.box_shadow[0].offset_x - 5.0).abs() < 0.01,
        "parent box-shadow offset_x 应为 5.0"
    );

    // 验证子元素不继承 box-shadow，box-shadow 列表应为空
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert!(
        child_style.box_shadow.is_empty(),
        "child 不应继承 parent 的 box-shadow，box-shadow 列表应为空"
    );
}

/// CSS outline 简写属性管线集成测试。
///
/// 解析含 outline: 2px solid red 的 CSS，通过 style-system 的简写展开，
/// 验证 outline_width=Px(2.0)、outline_style=Solid、outline_color=red。
#[test]
fn test_outline_shorthand_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "outlined");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .outlined { outline: 2px solid red; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");

    // 验证 outline-width
    assert_eq!(
        div_style.outline_width,
        LengthValue::Px(2.0),
        "div 的 outline-width 应为 Px(2.0)"
    );

    // 验证 outline-style
    assert_eq!(
        div_style.outline_style,
        zero_style_system::property::OutlineStyleValue::Solid,
        "div 的 outline-style 应为 Solid"
    );

    // 验证 outline-color
    assert_eq!(
        div_style.outline_color,
        zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
        "div 的 outline-color 应为红色 (255, 0, 0, 255)"
    );
}

/// CSS gap 简写管线集成测试。
///
/// 解析含 gap: 20px 的 CSS，通过简写展开为 row-gap: 20px 和 column-gap: 20px，
/// 由 style-system 计算样式后验证 ComputedStyle.row_gap 和 column_gap 均为 Px(20.0)。
#[test]
fn test_gap_shorthand_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "gapped");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .gapped { gap: 20px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(div_style.row_gap, LengthValue::Px(20.0), "div 的 row-gap 应为 Px(20.0)");
    assert_eq!(
        div_style.column_gap,
        LengthValue::Px(20.0),
        "div 的 column-gap 应为 Px(20.0)"
    );
}

/// CSS empty-cells 管线集成测试。
///
/// 解析含 empty-cells: hide 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.empty_cells 为 Hide。
#[test]
fn test_empty_cells_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let td = doc.create_element("td");
    doc.set_attribute(td, "class", "empty");
    doc.append_child(body, td).unwrap();

    let css = r#"
        .empty { empty-cells: hide; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let td_style = styles.get(&td).expect("td 应有计算样式");
    assert_eq!(
        td_style.empty_cells,
        zero_style_system::property::EmptyCellsComputedValue::Hide,
        "td 的 empty-cells 应为 Hide"
    );
}

/// CSS border-spacing 管线集成测试。
///
/// 解析含 border-spacing: 5px 10px 的 CSS，通过 style-system 计算样式，
/// 验证 horizontal=5.0, vertical=10.0。
#[test]
fn test_border_spacing_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let table = doc.create_element("table");
    doc.set_attribute(table, "class", "spaced");
    doc.append_child(body, table).unwrap();

    let css = r#"
        .spaced { border-spacing: 5px 10px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let table_style = styles.get(&table).expect("table 应有计算样式");
    assert!(
        (table_style.border_spacing.horizontal - 5.0).abs() < 0.01,
        "border-spacing horizontal 应为 5.0，实际为 {}",
        table_style.border_spacing.horizontal
    );
    assert!(
        (table_style.border_spacing.vertical - 10.0).abs() < 0.01,
        "border-spacing vertical 应为 10.0，实际为 {}",
        table_style.border_spacing.vertical
    );
}

/// CSS empty-cells 继承管线集成测试。
///
/// empty-cells 是继承属性。父元素 .parent 设置 empty-cells: hide，
/// 子元素 .child 不显式设置，应继承 Hide。
#[test]
fn test_empty_cells_inheritance_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let parent = doc.create_element("table");
    doc.set_attribute(parent, "class", "parent");
    doc.append_child(body, parent).unwrap();

    let child = doc.create_element("td");
    doc.set_attribute(child, "class", "child");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .parent { empty-cells: hide; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 验证父元素
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert_eq!(
        parent_style.empty_cells,
        zero_style_system::property::EmptyCellsComputedValue::Hide,
        "parent 的 empty-cells 应为 Hide"
    );

    // 验证子元素继承了 empty-cells: hide
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert_eq!(
        child_style.empty_cells,
        zero_style_system::property::EmptyCellsComputedValue::Hide,
        "child 应继承 parent 的 empty-cells: Hide"
    );
}

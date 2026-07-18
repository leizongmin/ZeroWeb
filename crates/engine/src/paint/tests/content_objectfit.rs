//! CSS `content` 属性渲染和 `object-fit` 渲染单元测试。

use zero_css_parser::values::{ColorValue, CounterActionValue, LengthValue};
use zero_dom::Document;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::geometry::Rect;
use zero_style_system::{ComputedStyle, ContentComputedValue, ObjectFitComputedValue};

use super::super::painter::Painter;
use crate::paint::image_resource_key;

/// 辅助函数：创建简单 LayoutBox。
fn make_box(width: f32, height: f32) -> LayoutBox {
    LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width,
        height,
        content_x: 0.0,
        content_y: 0.0,
        content_width: width,
        content_height: height,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    }
}

// ── CSS content 属性渲染测试 ──────────────────────────────────────────

#[test]
fn test_paint_content_string_generates_glyphs() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.content = ContentComputedValue::String("Hello".to_string());
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);

    let box_node = make_box(200.0, 20.0);
    let glyphs_before = painter.primitives.glyphs.len();
    painter.paint_content(&box_node, 0.0, 0.0, &style);

    // 应该生成 5 个 glyph（"Hello"）
    assert_eq!(painter.primitives.glyphs.len(), glyphs_before + 5);
}

#[test]
fn test_paint_content_normal_generates_nothing() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.content = ContentComputedValue::Normal;

    let box_node = make_box(200.0, 20.0);
    let glyphs_before = painter.primitives.glyphs.len();
    painter.paint_content(&box_node, 0.0, 0.0, &style);

    assert_eq!(painter.primitives.glyphs.len(), glyphs_before);
}

#[test]
fn test_paint_content_none_generates_nothing() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.content = ContentComputedValue::None;

    let box_node = make_box(200.0, 20.0);
    let glyphs_before = painter.primitives.glyphs.len();
    painter.paint_content(&box_node, 0.0, 0.0, &style);

    assert_eq!(painter.primitives.glyphs.len(), glyphs_before);
}

#[test]
fn test_paint_content_counter_decimal() {
    let mut painter = Painter::new();
    painter.counters.insert("section".to_string(), 42);

    let mut style = ComputedStyle::default();
    style.content = ContentComputedValue::Counter {
        name: "section".to_string(),
        style: None,
    };
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);

    let box_node = make_box(200.0, 20.0);
    let glyphs_before = painter.primitives.glyphs.len();
    painter.paint_content(&box_node, 0.0, 0.0, &style);

    // "42" = 2 个 glyph
    assert_eq!(painter.primitives.glyphs.len(), glyphs_before + 2);
}

#[test]
fn test_paint_content_counter_missing_uses_zero() {
    let mut painter = Painter::new();

    let mut style = ComputedStyle::default();
    style.content = ContentComputedValue::Counter {
        name: "nonexistent".to_string(),
        style: None,
    };
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);

    let box_node = make_box(200.0, 20.0);
    let glyphs_before = painter.primitives.glyphs.len();
    painter.paint_content(&box_node, 0.0, 0.0, &style);

    // "0" = 1 个 glyph
    assert_eq!(painter.primitives.glyphs.len(), glyphs_before + 1);
}

#[test]
fn test_paint_content_counter_lower_alpha() {
    let mut painter = Painter::new();
    painter.counters.insert("item".to_string(), 3);

    let mut style = ComputedStyle::default();
    style.content = ContentComputedValue::Counter {
        name: "item".to_string(),
        style: Some("lower-alpha".to_string()),
    };
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);

    let box_node = make_box(200.0, 20.0);
    let glyphs_before = painter.primitives.glyphs.len();
    painter.paint_content(&box_node, 0.0, 0.0, &style);

    // "c" (3rd letter) = 1 个 glyph
    assert_eq!(painter.primitives.glyphs.len(), glyphs_before + 1);
    assert_eq!(painter.primitives.glyphs[glyphs_before].glyph_id, 'c' as u32);
}

#[test]
fn test_paint_content_counter_upper_roman() {
    let mut painter = Painter::new();
    painter.counters.insert("chapter".to_string(), 4);

    let mut style = ComputedStyle::default();
    style.content = ContentComputedValue::Counter {
        name: "chapter".to_string(),
        style: Some("upper-roman".to_string()),
    };
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);

    let box_node = make_box(200.0, 20.0);
    let glyphs_before = painter.primitives.glyphs.len();
    painter.paint_content(&box_node, 0.0, 0.0, &style);

    // "IV" = 2 个 glyph
    assert_eq!(painter.primitives.glyphs.len(), glyphs_before + 2);
    assert_eq!(painter.primitives.glyphs[glyphs_before].glyph_id, 'I' as u32);
    assert_eq!(painter.primitives.glyphs[glyphs_before + 1].glyph_id, 'V' as u32);
}

#[test]
fn test_paint_content_empty_string_generates_nothing() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.content = ContentComputedValue::String(String::new());
    style.font_size = LengthValue::Px(16.0);

    let box_node = make_box(200.0, 20.0);
    let glyphs_before = painter.primitives.glyphs.len();
    painter.paint_content(&box_node, 0.0, 0.0, &style);

    assert_eq!(painter.primitives.glyphs.len(), glyphs_before);
}

#[test]
fn test_paint_content_current_color_generates_glyphs() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.content = ContentComputedValue::String("Test".to_string());
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::CurrentColor;

    let box_node = make_box(200.0, 20.0);
    let glyphs_before = painter.primitives.glyphs.len();
    painter.paint_content(&box_node, 0.0, 0.0, &style);

    // paint_content 使用 color_value_to_render 处理 CurrentColor（回退为黑色）
    // "Test" = 4 个 glyph
    assert_eq!(painter.primitives.glyphs.len(), glyphs_before + 4);
}

#[test]
fn test_paint_content_counter_lower_roman() {
    let mut painter = Painter::new();
    painter.counters.insert("appendix".to_string(), 9);

    let mut style = ComputedStyle::default();
    style.content = ContentComputedValue::Counter {
        name: "appendix".to_string(),
        style: Some("lower-roman".to_string()),
    };
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);

    let box_node = make_box(200.0, 20.0);
    let glyphs_before = painter.primitives.glyphs.len();
    painter.paint_content(&box_node, 0.0, 0.0, &style);

    // "ix" (lower-roman for 9) = 2 个 glyph
    assert_eq!(painter.primitives.glyphs.len(), glyphs_before + 2);
    assert_eq!(painter.primitives.glyphs[glyphs_before].glyph_id, 'i' as u32);
    assert_eq!(painter.primitives.glyphs[glyphs_before + 1].glyph_id, 'x' as u32);
}

#[test]
fn test_paint_content_attr_generates_nothing() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.content = ContentComputedValue::Attr("title".to_string());
    style.font_size = LengthValue::Px(16.0);

    let box_node = make_box(200.0, 20.0);
    let glyphs_before = painter.primitives.glyphs.len();
    painter.paint_content(&box_node, 0.0, 0.0, &style);

    assert_eq!(painter.primitives.glyphs.len(), glyphs_before);
}

#[test]
fn test_paint_content_counter_after_update() {
    let mut painter = Painter::new();

    // 先通过 update_counters 设置计数器
    let mut style_reset = ComputedStyle::default();
    style_reset.counter_reset = vec![CounterActionValue {
        name: "step".to_string(),
        value: Some(5),
    }];
    painter.update_counters(&style_reset);

    let mut style_increment = ComputedStyle::default();
    style_increment.counter_increment = vec![CounterActionValue {
        name: "step".to_string(),
        value: Some(3),
    }];
    painter.update_counters(&style_increment);

    // 现在计数器值应为 5 + 3 = 8
    let mut style = ComputedStyle::default();
    style.content = ContentComputedValue::Counter {
        name: "step".to_string(),
        style: None,
    };
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);

    let box_node = make_box(200.0, 20.0);
    let glyphs_before = painter.primitives.glyphs.len();
    painter.paint_content(&box_node, 0.0, 0.0, &style);

    // "8" = 1 个 glyph
    assert_eq!(painter.primitives.glyphs.len(), glyphs_before + 1);
    assert_eq!(painter.primitives.glyphs[glyphs_before].glyph_id, '8' as u32);
}

// ── object-fit 集成测试（通过 paint pipeline）────────────────────────

#[test]
fn test_paint_img_object_fit_fill_pipeline() {
    let mut doc = Document::new();
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.object_fit = ObjectFitComputedValue::Fill;

    // 创建 img 布局盒
    let img_elem = doc.create_element("img");
    let mut box_node = make_box(200.0, 100.0);
    box_node.node_id = Some(img_elem);

    // paint_img_element 要求 img 有 src 属性
    // 但由于 Document::create_element 创建的是空元素，
    // 没有 src 时应不生成图片
    let images_before = painter.primitives.images.len();
    painter.paint_img_element(&box_node, 0.0, 0.0, &style, &doc);

    // 没有 src → 不生成图片
    assert_eq!(painter.primitives.images.len(), images_before);
}

#[test]
fn test_paint_img_non_img_element_skipped() {
    let mut doc = Document::new();
    let mut painter = Painter::new();
    let style = ComputedStyle::default();

    let div = doc.create_element("div");
    let mut box_node = make_box(200.0, 100.0);
    box_node.node_id = Some(div);

    let images_before = painter.primitives.images.len();
    painter.paint_img_element(&box_node, 0.0, 0.0, &style, &doc);

    assert_eq!(painter.primitives.images.len(), images_before);
}

#[test]
fn test_paint_img_no_node_id_skipped() {
    let mut doc = Document::new();
    let mut painter = Painter::new();
    let style = ComputedStyle::default();

    let box_node = make_box(200.0, 100.0); // node_id = None
    painter.paint_img_element(&box_node, 0.0, 0.0, &style, &doc);

    assert!(painter.primitives.images.is_empty());
}

#[test]
fn test_paint_img_zero_size_skipped() {
    let mut doc = Document::new();
    let mut painter = Painter::new();
    let style = ComputedStyle::default();

    let img_elem = doc.create_element("img");
    let mut box_node = make_box(0.0, 0.0);
    box_node.node_id = Some(img_elem);

    painter.paint_img_element(&box_node, 0.0, 0.0, &style, &doc);
    assert!(painter.primitives.images.is_empty());
}

#[test]
fn test_paint_img_uses_decoded_intrinsic_size_when_available() {
    let mut doc = Document::new();
    let mut painter = Painter::new();
    painter.set_document_url(Some("https://example.com/page.html"));

    let mut style = ComputedStyle::default();
    style.object_fit = ObjectFitComputedValue::None;

    let img_elem = doc.create_element("img");
    doc.set_attribute(img_elem, "src", "/logo.png");

    let mut box_node = make_box(200.0, 100.0);
    box_node.node_id = Some(img_elem);

    painter.image_sizes.insert(
        image_resource_key("/logo.png", Some("https://example.com/page.html")),
        (50.0, 25.0),
    );

    painter.paint_img_element(&box_node, 0.0, 0.0, &style, &doc);

    assert_eq!(painter.primitives.images.len(), 1, "应生成一个图片图元");
    let image = &painter.primitives.images[0];
    assert!((image.rect.origin.x - 75.0).abs() < 0.1, "图片应在容器内水平居中");
    assert!((image.rect.origin.y - 37.5).abs() < 0.1, "图片应在容器内垂直居中");
    assert!((image.rect.size.width - 50.0).abs() < 0.1, "应使用解码后的真实宽度");
    assert!((image.rect.size.height - 25.0).abs() < 0.1, "应使用解码后的真实高度");
}

#[test]
fn test_paint_img_is_clipped_to_content_box() {
    let mut doc = Document::new();
    let mut painter = Painter::new();
    painter.set_document_url(Some("https://example.com/page.html"));

    let mut style = ComputedStyle::default();
    style.object_fit = ObjectFitComputedValue::None;

    let img_elem = doc.create_element("img");
    doc.set_attribute(img_elem, "src", "/wide.png");

    let mut box_node = make_box(200.0, 100.0);
    box_node.node_id = Some(img_elem);

    painter.image_sizes.insert(
        image_resource_key("/wide.png", Some("https://example.com/page.html")),
        (400.0, 50.0),
    );

    painter.paint_img_element(&box_node, 0.0, 0.0, &style, &doc);

    assert_eq!(painter.primitives.images.len(), 1, "应生成一个图片图元");
    let image = &painter.primitives.images[0];
    assert_eq!(
        image.clip,
        Some(Rect::new(0.0, 0.0, 200.0, 100.0)),
        "图片应裁剪到内容盒范围内"
    );
}

// ── R1660 <input> value 文本渲染（form-control slice-2）──────────────────

/// 辅助：parse HTML 取首个 `<input>` 的 NodeId。
fn first_input(html: &str) -> (zero_dom::Document, zero_dom::NodeId) {
    let doc = zero_dom::parse_html(html);
    let id = doc.get_elements_by_tag_name("input")[0];
    (doc, id)
}

#[test]
fn paint_input_value_submit_renders_value_label() {
    let (doc, submit) = first_input(r#"<body><input type="submit" value="Send"></body>"#);
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);

    let mut box_node = make_box(80.0, 22.0);
    box_node.node_id = Some(submit);
    let before = painter.primitives.glyphs.len();
    painter.paint_input_value(&box_node, 0.0, 0.0, &style, &doc);
    // value="Send" → 4 glyphs.
    assert_eq!(painter.primitives.glyphs.len(), before + 4);
}

#[test]
fn paint_input_value_default_submit_uses_submit_label() {
    let (doc, submit) = first_input(r#"<body><input type="submit"></body>"#);
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);

    let mut box_node = make_box(80.0, 22.0);
    box_node.node_id = Some(submit);
    let before = painter.primitives.glyphs.len();
    painter.paint_input_value(&box_node, 0.0, 0.0, &style, &doc);
    // 默认标签 "Submit" → 6 glyphs.
    assert_eq!(painter.primitives.glyphs.len(), before + 6);
}

#[test]
fn paint_input_value_password_renders_bullets() {
    let (doc, pw) = first_input(r#"<body><input type="password" value="ab"></body>"#);
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);

    let mut box_node = make_box(148.0, 21.0);
    box_node.node_id = Some(pw);
    let before = painter.primitives.glyphs.len();
    painter.paint_input_value(&box_node, 0.0, 0.0, &style, &doc);
    // value="ab" → 2 个 •（U+2022）遮罩字符。
    assert_eq!(painter.primitives.glyphs.len(), before + 2);
    assert_eq!(painter.primitives.glyphs[before].glyph_id, '\u{2022}' as u32);
}

#[test]
fn paint_input_value_text_renders_value_and_skips_non_text_types() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);

    // text value="Alice" → 5 glyphs（左对齐）。
    let (doc_text, text_input) = first_input(r#"<body><input type="text" value="Alice"></body>"#);
    let mut box_text = make_box(148.0, 21.0);
    box_text.node_id = Some(text_input);
    let before = painter.primitives.glyphs.len();
    painter.paint_input_value(&box_text, 0.0, 0.0, &style, &doc_text);
    assert_eq!(painter.primitives.glyphs.len(), before + 5);

    // checkbox / radio / hidden / range → 不渲染 value 文本。
    for html in [
        r#"<body><input type="checkbox" value="x"></body>"#,
        r#"<body><input type="radio" value="x"></body>"#,
        r#"<body><input type="hidden" value="x"></body>"#,
        r#"<body><input type="range" value="x"></body>"#,
    ] {
        let (doc, nid) = first_input(html);
        let mut box_node = make_box(13.0, 13.0);
        box_node.node_id = Some(nid);
        let before = painter.primitives.glyphs.len();
        painter.paint_input_value(&box_node, 0.0, 0.0, &style, &doc);
        assert_eq!(
            painter.primitives.glyphs.len(),
            before,
            "non-text input type should render no value glyphs: {html}"
        );
    }
}

#[test]
fn paint_input_value_non_input_element_skipped() {
    let doc = zero_dom::parse_html("<body><div type=\"submit\" value=\"Send\">x</div></body>");
    let div = doc.get_elements_by_tag_name("div")[0];
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    let mut box_node = make_box(80.0, 22.0);
    box_node.node_id = Some(div);
    let before = painter.primitives.glyphs.len();
    painter.paint_input_value(&box_node, 0.0, 0.0, &style, &doc);
    // 非 <input> 元素 → 不渲染。
    assert_eq!(painter.primitives.glyphs.len(), before);
}

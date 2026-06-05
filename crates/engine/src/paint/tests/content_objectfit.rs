//! CSS `content` 属性渲染和 `object-fit` 渲染单元测试。

use std::collections::HashMap;

use zero_css_parser::values::{ColorValue, CounterActionValue, LengthValue};
use zero_dom::Document;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::{ComputedStyle, ContentComputedValue, ObjectFitComputedValue};

use super::super::painter::Painter;

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
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
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

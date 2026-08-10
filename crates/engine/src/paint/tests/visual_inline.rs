#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use std::collections::HashMap;

use zero_css_parser::values::{ColorValue, LengthValue};
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::ComputedStyle;

use super::super::painter::Painter;

// ── 辅助函数（从 visual.rs 复制引用）──

fn make_box(node_id: Option<zero_dom::NodeId>, x: f32, y: f32, width: f32, height: f32) -> LayoutBox {
    LayoutBox {
        node_id,
        x,
        y,
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

// ── 新增测试：overflow clipping with nested elements ──────

/// R792：overflow:hidden 必须裁剪盒子**自身直属文本**（不仅子节点）。
///
/// 原实现 `counts_before_children` 快照取于 `paint_text` 之后，致裁剪范围 [snapshot..end]
/// 只含子节点、漏掉 `paint_text` 发射的直属文本字形——overflow!=visible 的盒子直属
/// 溢出文本（如不可断行的长字符串）不被裁到 content-box 而外溢可见。修复把快照移到
/// `paint_text` 之前。本例 div 宽 30px、直属文本 "AAAAAAAA..."（无断行点）水平溢出，
/// 超出 content box 右边界的字形应被裁为 font_size==0。
#[test]
fn test_overflow_hidden_clips_own_direct_text() {
    let mut doc = zero_dom::Document::new();
    let div = doc.create_element("div");
    let text = doc.create_text_node("AAAAAAAAAAAAAAAAAAAA"); // 20 字符，无断行点，水平溢出 30px
    doc.append_child(div, text).unwrap();

    let mut box_node = make_box(Some(div), 0.0, 0.0, 30.0, 20.0);
    box_node.overflow_x = OverflowClip::Hidden;
    box_node.overflow_y = OverflowClip::Hidden;
    box_node.content_width = 30.0;
    box_node.content_height = 20.0;

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(div, style);

    let mut painter = Painter::new();
    painter.paint(&box_node, &styles, Some(&doc));

    let glyphs = &painter.primitives().glyphs;
    assert!(!glyphs.is_empty(), "paint_text 应为直属文本发射 glyphs");
    // 超出 content box 右边界（x ≥ content_width=30）的字形应被裁（font_size==0）。
    // 未修复时这些字形 font_size>0 → 断言失败，捕获回归。
    let visible_beyond_box = glyphs.iter().any(|g| g.font_size > 0.0 && g.x >= 30.0);
    assert!(
        !visible_beyond_box,
        "overflow:hidden 应裁剪盒子自身直属文本超出 content box 的字形"
    );
}

/// R793：`overflow:hidden` 必须裁剪到 **padding box**（CSS §11.1.1），不是 content box。
///
/// 原实现 clip_rect 起点加 padding、尺寸取 content_width/height（= content box），致溢出内容
/// 落在 content 边与 padding 边之间的条带时被多裁——chromium 保留到 padding 边。本例父盒
/// padding=20、content=10（content box [20,30]，padding box [0,40]），子盒 100×100 红背景
/// 从 content 原点 (20,20) 溢出。修复后子填充右边界应到 padding 边（≈40），而非 content 边（=30）。
#[test]
fn test_overflow_hidden_clips_to_padding_box_not_content_box() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("div");

    // 子盒：红背景，远超父 content box，溢入 padding 条带
    let child_box = make_box(Some(child), 0.0, 0.0, 100.0, 100.0);

    // 父盒：overflow:hidden，四周 padding=20，content 10×10，border 0
    // → content box [20,30]×[20,30]，padding box [0,40]×[0,40]
    let mut parent_box = make_box(Some(parent), 0.0, 0.0, 50.0, 50.0);
    parent_box.content_width = 10.0;
    parent_box.content_height = 10.0;
    parent_box.padding_top = 20.0;
    parent_box.padding_right = 20.0;
    parent_box.padding_bottom = 20.0;
    parent_box.padding_left = 20.0;
    parent_box.overflow_x = OverflowClip::Hidden;
    parent_box.overflow_y = OverflowClip::Hidden;
    parent_box.children = vec![child_box];

    let mut styles = HashMap::new();
    // 父盒透明背景（不发射填充），仅子盒红背景
    styles.insert(parent, ComputedStyle::default());
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, Some(&doc));

    let fills = &painter.primitives().fills;
    // 子红填充从 content 原点 (20,20) 发射；裁剪后 origin 不变（clip_left≤20）。
    let child_fill = fills
        .iter()
        .find(|f| f.rect.origin.x == 20.0 && f.rect.origin.y == 20.0)
        .expect("子盒红填充应从 content 原点 (20,20) 发射");
    let right = child_fill.rect.origin.x + child_fill.rect.size.width;
    // 修复（padding box）：right≈40（padding 边）；未修复（content box）：right=30（content 边）。
    assert!(
        right >= 39.5,
        "overflow 应裁剪到 padding box（right≈40），非 content box（right=30）；实际 right={right}"
    );
}

/// 测试嵌套元素中 overflow:hidden 逐层裁剪。
#[test]
fn test_overflow_hidden_clips_deeply_nested_children() {
    let mut doc = zero_dom::Document::new();
    let grandparent = doc.create_element("div");
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    let child_box = make_box(Some(child), 80.0, 80.0, 50.0, 50.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 200.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 200.0,
        content_height: 200.0,
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
        children: vec![child_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    };
    let grandparent_box = LayoutBox {
        node_id: Some(grandparent),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 100.0,
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
        children: vec![parent_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
        ..Default::default()
    };

    let mut styles = HashMap::new();
    let mut parent_style = ComputedStyle::default();
    parent_style.background_color = ColorValue::Rgba(0, 128, 0, 255);
    styles.insert(parent, parent_style);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&grandparent_box, &styles, None);

    let fills = &painter.primitives().fills;
    assert!(!fills.is_empty(), "should produce fills from parent and child");

    let parent_fill = &fills[0];
    assert!(parent_fill.rect.size.width <= 100.0, "parent width clipped to 100");
    assert!(parent_fill.rect.size.height <= 100.0, "parent height clipped to 100");

    let child_fill = &fills[1];
    assert_eq!(child_fill.rect.origin.x, 80.0);
    assert_eq!(child_fill.rect.origin.y, 80.0);
    assert_eq!(
        child_fill.rect.size.width, 20.0,
        "child width clipped at grandparent boundary"
    );
    assert_eq!(
        child_fill.rect.size.height, 20.0,
        "child height clipped at grandparent boundary"
    );
}

/// 测试双层 overflow:hidden 嵌套，内层和外层各自裁剪。
#[test]
fn test_overflow_hidden_double_nesting_clips() {
    let mut doc = zero_dom::Document::new();
    let outer = doc.create_element("div");
    let inner = doc.create_element("div");
    let child = doc.create_element("span");

    let child_box = make_box(Some(child), 0.0, 0.0, 100.0, 100.0);
    let inner_box = LayoutBox {
        node_id: Some(inner),
        x: 0.0,
        y: 0.0,
        width: 40.0,
        height: 40.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 40.0,
        content_height: 40.0,
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
        children: vec![child_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
        ..Default::default()
    };
    let outer_box = LayoutBox {
        node_id: Some(outer),
        x: 0.0,
        y: 0.0,
        width: 80.0,
        height: 80.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 80.0,
        content_height: 80.0,
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
        children: vec![inner_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
        ..Default::default()
    };

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&outer_box, &styles, None);

    let fill = &painter.primitives().fills[0];
    assert_eq!(fill.rect.size.width, 40.0, "child clipped by inner overflow:hidden");
    assert_eq!(fill.rect.size.height, 40.0, "child clipped by inner overflow:hidden");
}

// ── Inline formatting context 测试 ──

#[test]
fn test_paint_inline_text_in_block() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 255, 255, 255);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.font_size = LengthValue::Px(16.0);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 1, "应生成 1 个背景填充");
    assert_eq!(prims.glyphs.len(), 1, "应生成 1 个 glyph 图元");

    let glyph = &prims.glyphs[0];
    assert_eq!(glyph.font_size, 16.0);
    assert_eq!(glyph.x, 0.0);
    assert_eq!(glyph.y, 16.0);
}

#[test]
fn test_paint_mixed_inline_block() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let block1 = doc.create_element("p");
    let inline_text = doc.create_element("span");
    let block2 = doc.create_element("p");

    let child1 = make_box(Some(block1), 0.0, 0.0, 200.0, 30.0);
    let child2 = make_box(Some(inline_text), 0.0, 30.0, 200.0, 20.0);
    let child3 = make_box(Some(block2), 0.0, 50.0, 200.0, 30.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 80.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 200.0,
        content_height: 80.0,
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
        children: vec![child1, child2, child3],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    };

    let mut styles = HashMap::new();
    let mut parent_style = ComputedStyle::default();
    parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    parent_style.color = ColorValue::CurrentColor;
    styles.insert(parent, parent_style);

    let mut block1_style = ComputedStyle::default();
    block1_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    block1_style.color = ColorValue::CurrentColor;
    styles.insert(block1, block1_style);

    let mut inline_style = ComputedStyle::default();
    inline_style.background_color = ColorValue::Transparent;
    inline_style.color = ColorValue::Rgba(0, 0, 255, 255);
    inline_style.font_size = LengthValue::Px(14.0);
    styles.insert(inline_text, inline_style);

    let mut block2_style = ComputedStyle::default();
    block2_style.background_color = ColorValue::Rgba(0, 255, 0, 255);
    block2_style.color = ColorValue::CurrentColor;
    styles.insert(block2, block2_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 3, "应生成 3 个填充");
    assert_eq!(prims.glyphs.len(), 1, "应生成 1 个 glyph");
    assert_eq!(prims.glyphs[0].y, 44.0);
}

#[test]
fn test_paint_text_with_color() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("span");
    let layout = make_box(Some(elem), 10.0, 20.0, 150.0, 25.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.color = ColorValue::Rgba(255, 0, 0, 255);
    style.font_size = LengthValue::Px(20.0);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let glyph = &painter.primitives().glyphs[0];
    assert_eq!(glyph.color, zero_render_foundation::color::Color::rgb(255, 0, 0));
    assert_eq!(glyph.font_size, 20.0);
    assert_eq!(glyph.x, 10.0);
    assert_eq!(glyph.y, 40.0);
}

#[test]
fn test_paint_text_with_inline_formatting_context() {
    let doc = zero_dom::parse_html("<p>Hello World</p>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    let layout = make_box(Some(p), 0.0, 0.0, 100.0, 50.0);
    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.font_size = LengthValue::Px(16.0);
    styles.insert(p, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    assert!(painter.primitives().glyphs.len() >= 2, "应有至少 2 个 glyph");
}

#[test]
fn test_paint_per_fragment_color_for_spans() {
    // R358：非多列路径 per-fragment color（带 abs-pos guard）。
    // 容器内不同 color 的 span，其文本 glyph 应取各 span 自身 color，而非容器 color。
    let doc = zero_dom::parse_html("<div><span>A</span><span>B</span></div>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let div = doc.first_child(body).unwrap();
    let span1 = doc.first_child(div).unwrap();
    let span2 = doc.last_child(div).unwrap();

    let layout = make_box(Some(div), 0.0, 0.0, 200.0, 30.0);
    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.color = ColorValue::Rgba(0, 0, 0, 255); // 容器黑色
    div_style.font_size = LengthValue::Px(16.0);
    styles.insert(div, div_style);
    let mut s1 = ComputedStyle::default();
    s1.color = ColorValue::Rgba(255, 0, 0, 255); // 红
    s1.font_size = LengthValue::Px(16.0);
    styles.insert(span1, s1);
    let mut s2 = ComputedStyle::default();
    s2.color = ColorValue::Rgba(0, 0, 255, 255); // 蓝
    s2.font_size = LengthValue::Px(16.0);
    styles.insert(span2, s2);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let glyphs = &painter.primitives().glyphs;
    let has_red = glyphs
        .iter()
        .any(|g| g.glyph_id == 'A' as u32 && g.color == zero_render_foundation::color::Color::rgb(255, 0, 0));
    let has_blue = glyphs
        .iter()
        .any(|g| g.glyph_id == 'B' as u32 && g.color == zero_render_foundation::color::Color::rgb(0, 0, 255));
    assert!(has_red, "span1 的 'A' glyph 应为红色（per-fragment color）");
    assert!(has_blue, "span2 的 'B' glyph 应为蓝色（per-fragment color）");
}

#[test]
fn test_paint_text_without_doc_fallback() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("p");
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.font_size = LengthValue::Px(16.0);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);
    assert_eq!(painter.primitives().glyphs.len(), 1);
}

#[test]
fn test_paint_inline_glyph_position_with_offset() {
    let doc = zero_dom::parse_html("<p>Text</p>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    let layout = LayoutBox {
        node_id: Some(p),
        x: 10.0,
        y: 20.0,
        width: 200.0,
        height: 50.0,
        content_x: 15.0,
        content_y: 25.0,
        content_width: 190.0,
        content_height: 40.0,
        border_top: 2.0,
        border_right: 2.0,
        border_bottom: 2.0,
        border_left: 2.0,
        padding_top: 3.0,
        padding_right: 3.0,
        padding_bottom: 3.0,
        padding_left: 3.0,
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
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.font_size = LengthValue::Px(16.0);
    styles.insert(p, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let glyph = &painter.primitives().glyphs[0];
    assert!(glyph.x >= 15.0, "glyph x 应包含偏移");
    assert!(glyph.y >= 25.0, "glyph y 应包含偏移");
}

#[test]
fn test_paint_inline_text_wrapping_multiple_lines() {
    let doc = zero_dom::parse_html("<p>a b c d e f g h</p>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    let layout = make_box(Some(p), 0.0, 0.0, 60.0, 200.0);
    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.font_size = LengthValue::Px(16.0);
    styles.insert(p, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));
    assert!(painter.primitives().glyphs.len() >= 1);
}

#[test]
fn test_paint_inline_mixed_text_and_elements() {
    let doc = zero_dom::parse_html("<p>Hello <b>World</b></p>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    let layout = make_box(Some(p), 0.0, 0.0, 400.0, 50.0);
    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.font_size = LengthValue::Px(16.0);
    styles.insert(p, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));
    assert!(painter.primitives().glyphs.len() >= 2);
}

#[test]
fn test_paint_inline_whitespace_only_no_glyphs() {
    let doc = zero_dom::parse_html("<p>   </p>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    let layout = make_box(Some(p), 0.0, 0.0, 200.0, 50.0);
    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.font_size = LengthValue::Px(16.0);
    styles.insert(p, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));
    assert!(painter.primitives().glyphs.len() <= 1);
}

#[test]
fn test_pipeline_uses_inline_formatting_for_text() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><p>Hello World</p></body></html>";
    let css = "p { color: black; font-size: 16px; }";
    let result = pipeline.render_html(html, css);
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_pipeline_inline_text_with_css_color() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><p>Styled</p></body></html>";
    let css = "p { color: red; font-size: 18px; }";
    let result = pipeline.render_html(html, css);
    assert!(!result.primitives().glyphs.is_empty());
}

// ── letter-spacing 和 word-spacing 渲染测试 ──

#[test]
fn test_letter_spacing_increases_glyph_gap() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><p>AB</p></body></html>";
    let css_base = "p { color: black; font-size: 16px; }";
    let result_base = pipeline.render_html(html, css_base);
    let glyphs_base: Vec<_> = result_base
        .primitives()
        .glyphs
        .iter()
        .filter(|g| g.glyph_id != 0)
        .collect();
    if glyphs_base.len() < 2 {
        return;
    }
    let gap_base = (glyphs_base[1].x - glyphs_base[0].x).abs();

    pipeline = RenderPipeline::new(800.0, 600.0);
    let css_spaced = "p { color: black; font-size: 16px; letter-spacing: 5px; }";
    let result_spaced = pipeline.render_html(html, css_spaced);
    let glyphs_spaced: Vec<_> = result_spaced
        .primitives()
        .glyphs
        .iter()
        .filter(|g| g.glyph_id != 0)
        .collect();
    if glyphs_spaced.len() < 2 {
        return;
    }
    let gap_spaced = (glyphs_spaced[1].x - glyphs_spaced[0].x).abs();

    assert!(
        gap_spaced > gap_base,
        "letter-spacing:5px 应增大间距: {gap_spaced} vs {gap_base}"
    );
}

#[test]
fn test_word_spacing_applied_in_style() {
    use crate::pipeline::RenderPipeline;
    use zero_css_parser::Parser as CssParser;
    use zero_dom::parse_html;

    let html = "<html><body><p>text</p></body></html>";
    let css = "p { color: black; font-size: 16px; word-spacing: 10px; }";
    let doc = parse_html(html);
    let stylesheets = vec![CssParser::parse_stylesheet(css)];
    let mut style_sys = zero_style_system::StyleSystem::new();
    style_sys.set_viewport(800.0, 600.0);
    let styles = style_sys.compute_styles(&doc, &stylesheets);
    assert!(
        styles
            .values()
            .any(|s| matches!(s.word_spacing, LengthValue::Px(v) if v > 0.0))
    );

    let css2 = "p { color: black; font-size: 16px; letter-spacing: 3px; }";
    let stylesheets2 = vec![CssParser::parse_stylesheet(css2)];
    let styles2 = style_sys.compute_styles(&doc, &stylesheets2);
    assert!(
        styles2
            .values()
            .any(|s| matches!(s.letter_spacing, LengthValue::Px(v) if v > 0.0))
    );
}

#[test]
fn test_letter_spacing_zero_no_effect() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><p>AB</p></body></html>";
    let css = "p { color: black; font-size: 16px; letter-spacing: 0px; }";
    let result = pipeline.render_html(html, css);
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_negative_letter_spacing_decreases_gap() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><p>AB</p></body></html>";
    let result_base = pipeline.render_html(html, "p { color: black; font-size: 16px; }");
    let glyphs_base: Vec<_> = result_base
        .primitives()
        .glyphs
        .iter()
        .filter(|g| g.glyph_id != 0)
        .collect();
    if glyphs_base.len() < 2 {
        return;
    }
    let gap_base = (glyphs_base[1].x - glyphs_base[0].x).abs();

    pipeline = RenderPipeline::new(800.0, 600.0);
    let result_neg = pipeline.render_html(html, "p { color: black; font-size: 16px; letter-spacing: -2px; }");
    let glyphs_neg: Vec<_> = result_neg
        .primitives()
        .glyphs
        .iter()
        .filter(|g| g.glyph_id != 0)
        .collect();
    if glyphs_neg.len() < 2 {
        return;
    }
    let gap_neg = (glyphs_neg[1].x - glyphs_neg[0].x).abs();

    assert!(
        gap_neg < gap_base,
        "letter-spacing:-2px 应减小间距: {gap_neg} vs {gap_base}"
    );
}

// ── text-overflow: ellipsis 测试 ──

#[test]
fn test_text_overflow_ellipsis_adds_dots() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(100.0, 50.0);
    let html = "<html><body><p>ABCDEFGHIJKLMNOPQRSTUVWXYZ</p></body></html>";
    let css = "p { color: black; font-size: 16px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; width: 80px; }";
    let result = pipeline.render_html(html, css);
    let has_ellipsis = result.primitives().glyphs.iter().any(|g| g.glyph_id == '.' as u32);
    assert!(has_ellipsis, "text-overflow: ellipsis 应生成 '.' glyph");
}

#[test]
fn test_text_overflow_clip_no_dots() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(100.0, 50.0);
    let html = "<html><body><p>ABCDEFGHIJKLMNOPQRSTUVWXYZ</p></body></html>";
    let css =
        "p { color: black; font-size: 16px; white-space: nowrap; overflow: hidden; text-overflow: clip; width: 80px; }";
    let result = pipeline.render_html(html, css);
    let has_ellipsis = result
        .primitives()
        .glyphs
        .iter()
        .filter(|g| g.glyph_id != 0)
        .any(|g| g.glyph_id == '.' as u32);
    assert!(!has_ellipsis);
}

#[test]
fn test_text_overflow_ellipsis_no_overflow() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><p>Hi</p></body></html>";
    let css = "p { color: black; font-size: 16px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }";
    let result = pipeline.render_html(html, css);
    let dot_count = result
        .primitives()
        .glyphs
        .iter()
        .filter(|g| g.glyph_id == '.' as u32)
        .count();
    assert_eq!(dot_count, 0);
}

#[test]
fn test_text_overflow_ellipsis_needs_hidden_overflow() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(100.0, 50.0);
    let html = "<html><body><p>ABCDEFGHIJKLMNOPQRSTUVWXYZ</p></body></html>";
    let css = "p { color: black; font-size: 16px; white-space: nowrap; overflow: visible; text-overflow: ellipsis; width: 80px; }";
    let result = pipeline.render_html(html, css);
    let dot_count = result
        .primitives()
        .glyphs
        .iter()
        .filter(|g| g.glyph_id == '.' as u32)
        .count();
    assert_eq!(dot_count, 0);
}

// ── line-clamp slice 2 ellipsis 测试（R2467）──

/// line-clamp:2 的 pure-Ahem 块（走 stored 路径：inline_layout 被 R2431 cap 到 2 行）
/// 须在第 2 行末渲 `…`（U+2026）。修复前 stored 路径 ellipsis 漏渲（line_clamp_clamped 未消费）。
#[test]
fn r2467_line_clamp_stored_path_emits_ellipsis() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(400.0, 400.0);
    // width:80px + font:20px/1 Ahem → 每个 "XXXX"=80px 占 1 行；8 词 → 8 行 → line-clamp:2 截到 2 行。
    let html = "<html><body style=\"margin:0\">\
        <div style=\"width:80px; font:20px/1 Ahem; line-clamp:2; overflow:hidden\">\
        XXXX XXXX XXXX XXXX XXXX XXXX XXXX XXXX\
        </div></body></html>";
    let result = pipeline.render_html(html, "");
    let has_ellipsis = result
        .primitives()
        .glyphs
        .iter()
        .any(|g| g.glyph_id == '\u{2026}' as u32);
    assert!(
        has_ellipsis,
        "R2467: line-clamp stored 路径应在末行末尾渲 U+2026 ellipsis"
    );
}

/// line-clamp:5 但内容仅 2 行（不足 N）→ 不截断 → 不应有 ellipsis（防 false ellipsis）。
#[test]
fn r2467_line_clamp_no_ellipsis_when_content_fits() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(400.0, 400.0);
    let html = "<html><body style=\"margin:0\">\
        <div style=\"width:80px; font:20px/1 Ahem; line-clamp:5; overflow:hidden\">\
        XXXX XXXX\
        </div></body></html>";
    let result = pipeline.render_html(html, "");
    let has_ellipsis = result
        .primitives()
        .glyphs
        .iter()
        .any(|g| g.glyph_id == '\u{2026}' as u32);
    assert!(!has_ellipsis, "R2467: 内容不足 line-clamp N 行时不应渲 ellipsis");
}

/// R2469：body{display:none} → body 不生成 principal box，其背景不传播到画布
///（CSS §9.2.4/§14.2）。driving: css-backgrounds background-color-body-propagation-004
///（ref=blank，无红填充）。注：display:contents 同理但 ZW 把 contents 当 block 布局
///（converter maps Contents→Block），body 自身盒仍画 bg → -007 需 display:contents 布局
///（深，defer），本测仅覆盖 display:none 路径。
#[test]
fn r2469_body_no_box_no_canvas_propagation() {
    use crate::pipeline::RenderPipeline;
    let is_red = |c: &zero_render_foundation::color::Color| c.r == 255 && c.g == 0 && c.b == 0;

    // display:none → body 无盒，红背景不应传播到画布
    let mut p = RenderPipeline::new(100.0, 100.0);
    let r = p.render_html("<html><body style=\"background:red; display:none\"></body></html>", "");
    assert!(
        !r.primitives().fills.iter().any(|f| is_red(&f.color)),
        "R2469: body{{display:none}} 背景不应传播到画布"
    );

    // 对照：默认 block body → 红背景应传播到画布（确保修复未误伤正常传播）
    let mut p = RenderPipeline::new(100.0, 100.0);
    let r = p.render_html("<html><body style=\"background:red\"></body></html>", "");
    assert!(
        r.primitives().fills.iter().any(|f| is_red(&f.color)),
        "R2469: 默认 block body 红背景应传播到画布（对照）"
    );
}

// ── CSS filter 渲染测试 ──

#[test]
fn test_filter_blur_generates_filter_primitive() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        "<html><body><p>Hello</p></body></html>",
        "p { color: black; font-size: 16px; filter: blur(5px); }",
    );
    assert!(!result.primitives().filters.is_empty());
}

#[test]
fn test_filter_grayscale_generates_filter_primitive() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        "<html><body><div>Test</div></body></html>",
        "div { color: black; font-size: 16px; filter: grayscale(1); }",
    );
    assert!(!result.primitives().filters.is_empty());
}

#[test]
fn test_filter_none_no_primitive() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        "<html><body><p>Hello</p></body></html>",
        "p { color: black; font-size: 16px; filter: none; }",
    );
    assert!(result.primitives().filters.is_empty());
}

#[test]
fn test_no_filter_property() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        "<html><body><p>Hello</p></body></html>",
        "p { color: black; font-size: 16px; }",
    );
    assert!(result.primitives().filters.is_empty());
}

#[test]
fn test_filter_brightness_value() {
    use crate::pipeline::RenderPipeline;
    use zero_render_foundation::primitive::FilterKind;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        "<html><body><div>Test</div></body></html>",
        "div { color: black; font-size: 16px; filter: brightness(1.5); }",
    );
    let filters = &result.primitives().filters;
    assert_eq!(filters.len(), 1);
    assert!(
        filters[0]
            .filters
            .iter()
            .any(|f| matches!(f, FilterKind::Brightness(v) if (*v - 1.5).abs() < 0.01))
    );
}

#[test]
fn test_filter_drop_shadow() {
    use crate::pipeline::RenderPipeline;
    use zero_render_foundation::primitive::FilterKind;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        "<html><body><div>Test</div></body></html>",
        "div { color: black; font-size: 16px; filter: drop-shadow(2px 3px 4px black); }",
    );
    let filters = &result.primitives().filters;
    assert_eq!(filters.len(), 1);
    assert!(filters[0].filters.iter().any(|f| matches!(f, FilterKind::DropShadow(x, y, blur, _) if (*x - 2.0).abs() < 0.1 && (*y - 3.0).abs() < 0.1 && (*blur - 4.0).abs() < 0.1)));
}

/// R2306：filter 多函数列表按声明顺序生成 FilterPrimitive.filters（CSS Filter Effects：<filter-function>+）。
#[test]
fn test_filter_multiple_functions_emit_all() {
    use crate::pipeline::RenderPipeline;
    use zero_render_foundation::primitive::FilterKind;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        "<html><body><div>Test</div></body></html>",
        "div { color: black; font-size: 16px; filter: blur(5px) brightness(1.5) sepia(0.5); }",
    );
    let filters = &result.primitives().filters;
    assert_eq!(filters.len(), 1, "应生成 1 个 FilterPrimitive（同元素多函数合并）");
    // 3 个函数按声明顺序全部 emit
    assert_eq!(filters[0].filters.len(), 3, "应 emit 3 个 filter 函数");
    assert!(matches!(filters[0].filters[0], FilterKind::Blur(v) if (v - 5.0).abs() < 0.01));
    assert!(matches!(filters[0].filters[1], FilterKind::Brightness(v) if (v - 1.5).abs() < 0.01));
    assert!(matches!(filters[0].filters[2], FilterKind::Sepia(v) if (v - 0.5).abs() < 0.01));
}

// ── CSS text-indent 渲染测试 ──

#[test]
fn test_text_indent_px_offsets_first_line() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline_no_indent = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><p>First line text</p></body></html>";
    let result_base = pipeline_no_indent.render_html(html, "p { color: black; font-size: 16px; }");
    let glyphs_base: Vec<_> = result_base
        .primitives()
        .glyphs
        .iter()
        .filter(|g| g.glyph_id != 0)
        .collect();
    if glyphs_base.is_empty() {
        return;
    }
    let first_x_base = glyphs_base[0].x;

    let mut pipeline_indent = RenderPipeline::new(800.0, 600.0);
    let result_indent = pipeline_indent.render_html(html, "p { color: black; font-size: 16px; text-indent: 32px; }");
    let glyphs_indent: Vec<_> = result_indent
        .primitives()
        .glyphs
        .iter()
        .filter(|g| g.glyph_id != 0)
        .collect();
    if glyphs_indent.is_empty() {
        return;
    }
    let first_x_indent = glyphs_indent[0].x;

    assert!(
        first_x_indent > first_x_base,
        "text-indent: 32px 应右移: {first_x_indent} vs {first_x_base}"
    );
    assert!((first_x_indent - first_x_base - 32.0).abs() < 2.0);
}

#[test]
fn test_text_indent_zero_no_offset() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        "<html><body><p>Text</p></body></html>",
        "p { color: black; font-size: 16px; text-indent: 0; }",
    );
    assert!(
        !result
            .primitives()
            .glyphs
            .iter()
            .filter(|g| g.glyph_id != 0)
            .collect::<Vec<_>>()
            .is_empty()
    );
}

#[test]
fn test_text_indent_em_units() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        "<html><body><p>Indented paragraph</p></body></html>",
        "p { color: black; font-size: 20px; text-indent: 2em; }",
    );
    assert!(
        !result
            .primitives()
            .glyphs
            .iter()
            .filter(|g| g.glyph_id != 0)
            .collect::<Vec<_>>()
            .is_empty()
    );
}

// ── CSS overflow-wrap: break-word 渲染测试 ──

#[test]
fn test_overflow_wrap_break_word_no_panic() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(60.0, 200.0);
    let result = pipeline.render_html(
        "<html><body><p>Supercalifragilisticexpialidocious</p></body></html>",
        "p { color: black; font-size: 14px; overflow-wrap: break-word; }",
    );
    assert!(
        !result
            .primitives()
            .glyphs
            .iter()
            .filter(|g| g.glyph_id != 0)
            .collect::<Vec<_>>()
            .is_empty()
    );
}

#[test]
fn test_overflow_wrap_normal_no_break() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(100.0, 200.0);
    let result = pipeline.render_html(
        "<html><body><p>Short words only</p></body></html>",
        "p { color: black; font-size: 14px; overflow-wrap: normal; }",
    );
    assert!(
        !result
            .primitives()
            .glyphs
            .iter()
            .filter(|g| g.glyph_id != 0)
            .collect::<Vec<_>>()
            .is_empty()
    );
}

// ── R1855：break-word 内容高度测量 ──

/// 递归查找是否存在「窄盒 + 多行高度」的盒子（break-word 长词断成多行的 `<p>`）。
fn has_narrow_multiline_box(b: &LayoutBox) -> bool {
    // width ≤ 100（窄 `<p>`，区别于全宽 body/html）；height ≥ 60（≥3 行 @20px，区别于 1 行溢出）。
    (b.width <= 100.0 && b.height >= 60.0) || b.children.iter().any(has_narrow_multiline_box)
}

/// R1855：overflow-wrap:break-word 容器的内容高度测量须 char-break。
///
/// 窄容器（60px）中的长不可断词（20 字 × 20px = 400px）+ break-word 应断成 ≥4 行，
/// 故 `<p>` box 高度须为多行（≥60px）。`measure_text_content` 此前未传 break_word，
/// taffy 测成 1 行 → box 过矮 → 与 paint/stored IFC（char-break 多行）不一致 →
/// 兄弟错位（word-wrap-002/overflow-wrap-002 等回归）。
#[test]
fn r1855_break_word_measures_multiline_height() {
    use crate::pipeline::RenderPipeline;
    let html = "<html><body><p>AAAAAAAAAAAAAAAAAAAA</p></body></html>";

    // break-word：长词应 char-break 成多行 → 存在窄多行盒。
    let mut p = RenderPipeline::new(120.0, 400.0);
    let r = p.render_html(html, "p { overflow-wrap: break-word; font-size: 20px; width: 60px; }");
    assert!(
        has_narrow_multiline_box(&r.layout.root),
        "break-word <p> 应测成多行（窄盒 height>=60），实际未找到——测量期未 char-break"
    );

    // 对照：overflow-wrap:normal（无 break-word）长词溢出 1 行 → 不应有窄多行盒。
    let mut p2 = RenderPipeline::new(120.0, 400.0);
    let r2 = p2.render_html(html, "p { overflow-wrap: normal; font-size: 20px; width: 60px; }");
    assert!(
        !has_narrow_multiline_box(&r2.layout.root),
        "normal（无 break-word）<p> 应 1 行溢出，不应有窄多行盒"
    );
}

// ── writing-mode 渲染测试 ──

/// 测试 writing-mode: horizontal-tb（默认）字形不旋转。
#[test]
fn test_writing_mode_horizontal_no_rotation() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(200.0, 200.0);
    let result = pipeline.render_html(
        "<html><body><p>Hello</p></body></html>",
        "p { color: black; font-size: 16px; writing-mode: horizontal-tb; }",
    );
    // 所有 glyph 的 rotation 应为 0.0
    for g in &result.primitives().glyphs {
        if g.glyph_id != 0 {
            assert_eq!(g.rotation, 0.0, "horizontal-tb glyph 不应旋转");
        }
    }
}

/// 测试 writing-mode: vertical-rl 字形旋转 90°。
#[test]
fn test_writing_mode_vertical_rl_rotated() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(200.0, 200.0);
    let result = pipeline.render_html(
        "<html><body><p>Hello</p></body></html>",
        "p { color: black; font-size: 16px; writing-mode: vertical-rl; }",
    );
    // 所有非占位 glyph 的 rotation 应为 FRAC_PI_2 (~1.5708)
    let has_rotated = result
        .primitives()
        .glyphs
        .iter()
        .any(|g| g.glyph_id != 0 && (g.rotation - std::f32::consts::FRAC_PI_2).abs() < 0.01);
    assert!(has_rotated, "vertical-rl glyph 应旋转 90°");
}

/// 测试 writing-mode: vertical-lr 字形旋转 90°。
#[test]
fn test_writing_mode_vertical_lr_rotated() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(200.0, 200.0);
    let result = pipeline.render_html(
        "<html><body><p>World</p></body></html>",
        "p { color: black; font-size: 16px; writing-mode: vertical-lr; }",
    );
    let has_rotated = result
        .primitives()
        .glyphs
        .iter()
        .any(|g| g.glyph_id != 0 && (g.rotation - std::f32::consts::FRAC_PI_2).abs() < 0.01);
    assert!(has_rotated, "vertical-lr glyph 应旋转 90°");
}

// ── word-break 渲染测试 ──

/// 测试 word-break: break-all 渲染不崩溃。
#[test]
fn test_word_break_break_all_renders() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(200.0, 200.0);
    let result = pipeline.render_html(
        "<html><body><p>Supercalifragilisticexpialidocious</p></body></html>",
        "p { color: black; font-size: 14px; word-break: break-all; width: 60px; }",
    );
    // break-all 应生成字形（不崩溃），且可能产生多行
    let glyph_count = result.primitives().glyphs.iter().filter(|g| g.glyph_id != 0).count();
    assert!(glyph_count > 0, "break-all 应生成字形");
}

/// 测试 word-break: keep-all 渲染不崩溃。
#[test]
fn test_word_break_keep_all_renders() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(200.0, 200.0);
    let result = pipeline.render_html(
        "<html><body><p>中文文本内容测试</p></body></html>",
        "p { color: black; font-size: 14px; word-break: keep-all; }",
    );
    // keep-all 应生成字形（CJK 文本作为整体）
    let glyph_count = result.primitives().glyphs.iter().filter(|g| g.glyph_id != 0).count();
    assert!(glyph_count > 0, "keep-all 应生成字形");
}

/// 测试 word-break: normal 渲染正常。
#[test]
fn test_word_break_normal_renders() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(200.0, 200.0);
    let result = pipeline.render_html(
        "<html><body><p>Hello World</p></body></html>",
        "p { color: black; font-size: 14px; word-break: normal; }",
    );
    let glyph_count = result.primitives().glyphs.iter().filter(|g| g.glyph_id != 0).count();
    assert!(glyph_count > 0, "normal 应生成字形");
}

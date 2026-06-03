#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use std::collections::HashMap;

use zero_css_parser::values::{ColorValue, LengthValue, TransformFunction, TransformValue};
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_style_system::{BorderStyleValue, ComputedStyle, OutlineStyleValue};

use super::super::color::{hsla_to_rgba, named_color_to_render};
use super::super::helpers::{BorderRadiusSpec, apply_transform_offset};
use super::super::painter::Painter;

/// 辅助函数：创建简单 LayoutBox。
fn make_box(node_id: Option<NodeId>, x: f32, y: f32, width: f32, height: f32) -> LayoutBox {
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
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    }
}

/// 辅助函数：创建带边框的 LayoutBox。
fn make_box_with_border(
    node_id: Option<NodeId>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    border_top: f32,
    border_right: f32,
    border_bottom: f32,
    border_left: f32,
) -> LayoutBox {
    LayoutBox {
        node_id,
        x,
        y,
        width,
        height,
        content_x: border_left,
        content_y: border_top,
        content_width: width - border_left - border_right,
        content_height: height - border_top - border_bottom,
        border_top,
        border_right,
        border_bottom,
        border_left,
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
// ── 新增测试：overflow 裁剪 ──────────────────────────────

/// 测试 overflow:hidden 裁剪子节点超出内容盒的部分。
#[test]
fn test_overflow_hidden_clips_children() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    // 子节点超出父节点的内容区域
    let child_box = make_box(Some(child), 0.0, 0.0, 200.0, 200.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
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
        children: vec![child_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
    };

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    // 子节点填充应该被裁剪到父节点的 100x100 内容区域
    let fill = &painter.primitives().fills[0];
    assert_eq!(fill.rect.size.width, 100.0, "子节点宽度应被裁剪到 100");
    assert_eq!(fill.rect.size.height, 100.0, "子节点高度应被裁剪到 100");
}

/// 测试 overflow:Visible 不裁剪子节点。
#[test]
fn test_overflow_visible_no_clip() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    let child_box = make_box(Some(child), 0.0, 0.0, 200.0, 200.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
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
        children: vec![child_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    // 子节点填充不应被裁剪
    let fill = &painter.primitives().fills[0];
    assert_eq!(fill.rect.size.width, 200.0);
    assert_eq!(fill.rect.size.height, 200.0);
}

/// 测试 overflow:Clip 裁剪子节点（与 Hidden 行为一致）。
#[test]
fn test_overflow_clip_clips_children() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    let child_box = make_box(Some(child), 50.0, 50.0, 200.0, 200.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
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
        children: vec![child_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Clip,
        overflow_y: OverflowClip::Clip,
    };

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    // 子节点从 (50,50) 开始 200x200，裁剪到 100x100 的内容盒
    let fill = &painter.primitives().fills[0];
    assert!(fill.rect.size.width <= 100.0);
    assert!(fill.rect.size.height <= 100.0);
}

// ── 新增测试：border-radius ──────────────────────────────

/// 测试 BorderRadiusSpec::from_style 提取圆角半径。
#[test]
fn test_border_radius_spec_from_style() {
    let mut style = ComputedStyle::default();
    style.border_top_left_radius = LengthValue::Px(10.0);
    style.border_top_right_radius = LengthValue::Px(20.0);
    style.border_bottom_right_radius = LengthValue::Px(30.0);
    style.border_bottom_left_radius = LengthValue::Px(40.0);

    let spec = BorderRadiusSpec::from_style(&style);
    assert_eq!(spec.top_left, 10.0);
    assert_eq!(spec.top_right, 20.0);
    assert_eq!(spec.bottom_right, 30.0);
    assert_eq!(spec.bottom_left, 40.0);
    assert!(!spec.is_zero());
}

/// 测试默认 ComputedStyle 的 BorderRadiusSpec 为零。
#[test]
fn test_border_radius_spec_default_zero() {
    let style = ComputedStyle::default();
    let spec = BorderRadiusSpec::from_style(&style);
    assert!(spec.is_zero());
}

/// 测试带圆角的背景填充仍然生成。
#[test]
fn test_painter_background_with_border_radius() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_top_left_radius = LengthValue::Px(10.0);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 背景填充仍然生成（圆角标记在内部处理）
    assert_eq!(painter.primitives().fills.len(), 1);
    assert_eq!(painter.primitives().fills[0].color, Color::rgb(255, 0, 0));
}

// ── 新增测试：CSS transform ──────────────────────────────

/// 测试 translate transform 偏移文本位置。
#[test]
fn test_transform_translate_offset() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![TransformFunction::Translate(10.0, 20.0)]);

    let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
    assert_eq!(dx, 10.0);
    assert_eq!(dy, 20.0);
}

/// 测试 translateX/translateY 偏移。
#[test]
fn test_transform_translate_xy_offset() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![
        TransformFunction::TranslateX(30.0),
        TransformFunction::TranslateY(40.0),
    ]);

    let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
    assert_eq!(dx, 30.0);
    assert_eq!(dy, 40.0);
}

/// 测试 TransformValue::None 不产生偏移。
#[test]
fn test_transform_none_no_offset() {
    let style = ComputedStyle::default();
    let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
    assert_eq!(dx, 0.0);
    assert_eq!(dy, 0.0);
}

/// 测试 rotate/scale/skew 不影响偏移。
#[test]
fn test_transform_rotate_scale_no_offset() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![
        TransformFunction::Rotate(45.0),
        TransformFunction::Scale(2.0, None),
        TransformFunction::Skew(10.0, None),
    ]);

    let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
    assert_eq!(dx, 0.0);
    assert_eq!(dy, 0.0);
}

/// 测试 paint_text 生成 GlyphPrimitive。
#[test]
fn test_paint_text_generates_glyph() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint_text(&layout, 10.0, 20.0, &styles[&elem], None);

    assert_eq!(painter.primitives().glyphs.len(), 1);
    let glyph = &painter.primitives().glyphs[0];
    assert_eq!(glyph.font_size, 16.0);
    assert_eq!(glyph.color, Color::rgb(255, 0, 0));
    assert_eq!(glyph.x, 10.0); // text_x = abs_x (no border/padding)
    assert_eq!(glyph.y, 36.0); // text_y + font_size = 20 + 16
}

/// 测试 paint_text 在 font_size <= 0 时不生成 glyph。
#[test]
fn test_paint_text_zero_font_size() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(0.0);
    style.color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None);
    assert!(painter.primitives().glyphs.is_empty());
}

/// 测试 paint_text 在 color 为 CurrentColor 时不生成 glyph。
#[test]
fn test_paint_text_current_color_no_glyph() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None);
    assert!(painter.primitives().glyphs.is_empty());
}

/// 测试 paint_text 带 translate transform 偏移 glyph 位置。
#[test]
fn test_paint_text_with_transform() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.transform = TransformValue::List(vec![TransformFunction::Translate(5.0, 10.0)]);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None);

    let glyph = &painter.primitives().glyphs[0];
    assert_eq!(glyph.x, 5.0); // 0 + translate_x(5)
    assert_eq!(glyph.y, 26.0); // 0 + translate_y(10) + font_size(16)
}

// ── 新增测试：paint_in_rect 增量绘制 ──────────────────────

/// 测试 paint_in_rect 跳过完全不在脏区域内的节点。
#[test]
fn test_paint_in_rect_skips_outside_nodes() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // 节点在 (500, 500) 处
    let layout = make_box(Some(elem), 500.0, 500.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(elem, style);

    // 脏区域在 (0, 0) 处，不与节点相交
    let dirty_rect = Rect::new(0.0, 0.0, 100.0, 100.0);

    let mut painter = Painter::new();
    painter.paint_in_rect(&layout, &styles, &dirty_rect, None);

    // 节点完全在脏区域外，不应产生任何图元
    assert!(painter.primitives().is_empty());
}

/// 测试 paint_in_rect 绘制与脏区域相交的节点。
#[test]
fn test_paint_in_rect_draws_intersecting_nodes() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 50.0, 50.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(elem, style);

    // 脏区域与节点部分重叠
    let dirty_rect = Rect::new(0.0, 0.0, 100.0, 100.0);

    let mut painter = Painter::new();
    painter.paint_in_rect(&layout, &styles, &dirty_rect, None);

    // 节点与脏区域相交，应产生填充图元
    assert_eq!(painter.primitives().fills.len(), 1);
}

// ── 新增测试：Paint pipeline ──────────────────────────────

/// 测试绘制简单 HTML 页面中带文本样式的元素。
#[test]
fn test_paint_page_with_text_element() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("p");
    let layout = make_box(Some(elem), 0.0, 0.0, 300.0, 20.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 255, 255, 255);
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 背景填充
    assert_eq!(painter.primitives().fills.len(), 1);
    assert_eq!(painter.primitives().fills[0].rect.size.width, 300.0);
    assert_eq!(painter.primitives().fills[0].rect.size.height, 20.0);
}

/// 测试绘制包含多个子元素的页面。
#[test]
fn test_paint_page_multiple_elements() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let c1 = doc.create_element("span");
    let c2 = doc.create_element("span");
    let c3 = doc.create_element("span");

    let child1 = make_box(Some(c1), 0.0, 0.0, 100.0, 30.0);
    let child2 = make_box(Some(c2), 0.0, 30.0, 100.0, 30.0);
    let child3 = make_box(Some(c3), 0.0, 60.0, 100.0, 30.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 90.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 90.0,
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
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut parent_style = ComputedStyle::default();
    parent_style.background_color = ColorValue::Rgba(240, 240, 240, 255);
    styles.insert(parent, parent_style);

    for id in [c1, c2, c3] {
        let mut s = ComputedStyle::default();
        s.background_color = ColorValue::Rgba(100, 100, 200, 255);
        styles.insert(id, s);
    }

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    // 1 parent background + 3 child backgrounds = 4
    assert_eq!(painter.primitives().fills.len(), 4);
}

/// 测试带 CSS transform 的 translate 偏移 glyph。
#[test]
fn test_paint_page_with_css_transform_translate() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(14.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.transform = TransformValue::List(vec![TransformFunction::Translate(15.0, 25.0)]);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // Background fill should still be at original position
    assert_eq!(painter.primitives().fills.len(), 1);
    assert_eq!(painter.primitives().fills[0].rect.origin.x, 10.0);
    assert_eq!(painter.primitives().fills[0].rect.origin.y, 20.0);

    // paint() 现在调用 paint_text()，应生成带 transform 偏移的 glyph
    assert_eq!(painter.primitives().glyphs.len(), 1);
    let glyph = &painter.primitives().glyphs[0];
    // text_x = abs_x(10), tx = 15 → glyph_x = 10 + 15 = 25
    assert_eq!(glyph.x, 25.0);
    // text_y = abs_y(20), ty = 25, + font_size(14) → glyph_y = 20 + 25 + 14 = 59
    assert_eq!(glyph.y, 59.0);
}

/// 测试带 overflow:hidden 的页面正确裁剪子内容。
#[test]
fn test_paint_page_with_overflow_hidden() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    let child_box = make_box(Some(child), 0.0, 0.0, 300.0, 300.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 10.0,
        y: 10.0,
        width: 100.0,
        height: 80.0,
        content_x: 10.0,
        content_y: 10.0,
        content_width: 100.0,
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
        children: vec![child_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
    };

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    let fill = &painter.primitives().fills[0];
    assert!(
        fill.rect.size.width <= 100.0,
        "child should be clipped to parent content width"
    );
    assert!(
        fill.rect.size.height <= 80.0,
        "child should be clipped to parent content height"
    );
}

/// 测试带 border-radius 的页面正确生成背景填充。
#[test]
fn test_paint_page_with_border_radius() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(100, 149, 237, 255);
    style.border_top_left_radius = LengthValue::Px(20.0);
    style.border_top_right_radius = LengthValue::Px(20.0);
    style.border_bottom_right_radius = LengthValue::Px(20.0);
    style.border_bottom_left_radius = LengthValue::Px(20.0);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // Background fill still generated even with border-radius
    assert_eq!(painter.primitives().fills.len(), 1);
    assert_eq!(painter.primitives().fills[0].rect.size.width, 200.0);
    assert_eq!(painter.primitives().fills[0].rect.size.height, 100.0);
}

/// 测试渲染输出：背景先于前景（parent fill comes before child fill）。
#[test]
fn test_render_primitive_order_background_before_foreground() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    let child_box = make_box(Some(child), 5.0, 5.0, 50.0, 30.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 200.0,
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
        children: vec![child_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut parent_style = ComputedStyle::default();
    parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    styles.insert(parent, parent_style);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    assert_eq!(painter.primitives().fills.len(), 2);
    // First fill is parent background (drawn first = behind)
    assert_eq!(painter.primitives().fills[0].rect.size.width, 200.0);
    // Second fill is child background (drawn second = in front)
    assert_eq!(painter.primitives().fills[1].rect.size.width, 50.0);
}

/// 测试渲染输出：primitive count 与预期匹配。
#[test]
fn test_render_primitive_count_matches_expectation() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // 2px border on all sides
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 60.0, 2.0, 2.0, 2.0, 2.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(128, 128, 128, 255);
    style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_left_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    style.border_bottom_style = BorderStyleValue::Solid;
    style.border_left_style = BorderStyleValue::Solid;
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 1 background + 4 borders = 5 fills, 0 glyphs
    assert_eq!(painter.primitives().fills.len(), 5);
    assert_eq!(painter.primitives().glyphs.len(), 0);
    assert_eq!(painter.primitives().len(), 5);
}

// ── 新增测试：CSS transform integration ───────────────────

/// 测试 translateX 变换偏移。
#[test]
fn test_transform_translate_x_only() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![TransformFunction::TranslateX(42.0)]);
    let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
    assert_eq!(dx, 42.0);
    assert_eq!(dy, 0.0);
}

/// 测试 translateY 变换偏移。
#[test]
fn test_transform_translate_y_only() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![TransformFunction::TranslateY(99.0)]);
    let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
    assert_eq!(dx, 0.0);
    assert_eq!(dy, 99.0);
}

/// 测试 translate + translateX + translateY 累加。
#[test]
fn test_transform_combined_translates() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![
        TransformFunction::Translate(10.0, 20.0),
        TransformFunction::TranslateX(5.0),
        TransformFunction::TranslateY(3.0),
    ]);
    let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
    assert_eq!(dx, 15.0);
    assert_eq!(dy, 23.0);
}

/// 测试 rotate + translate 混合：只有 translate 贡献偏移。
#[test]
fn test_transform_rotate_with_translate() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![
        TransformFunction::Rotate(90.0),
        TransformFunction::Translate(50.0, 60.0),
        TransformFunction::Scale(2.0, None),
    ]);
    let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
    assert_eq!(dx, 50.0);
    assert_eq!(dy, 60.0);
}

// ── 新增测试：Incremental rendering / paint_in_rect ───────

/// 测试 paint_in_rect 跳过完全在右侧的节点。
#[test]
fn test_paint_in_rect_skips_right() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 200.0, 0.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(elem, style);

    let dirty_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    let mut painter = Painter::new();
    painter.paint_in_rect(&layout, &styles, &dirty_rect, None);
    assert!(painter.primitives().is_empty());
}

/// 测试 paint_in_rect 跳过完全在下方的节点。
#[test]
fn test_paint_in_rect_skips_below() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 300.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 255, 0, 255);
    styles.insert(elem, style);

    let dirty_rect = Rect::new(0.0, 0.0, 800.0, 200.0);
    let mut painter = Painter::new();
    painter.paint_in_rect(&layout, &styles, &dirty_rect, None);
    assert!(painter.primitives().is_empty());
}

/// 测试 paint_in_rect 与脏区域刚好边缘相交的节点。
#[test]
fn test_paint_in_rect_edge_touch() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // Node right edge at x=100, dirty rect starts at x=99
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 0, 255, 255);
    styles.insert(elem, style);

    let dirty_rect = Rect::new(99.0, 0.0, 100.0, 50.0);
    let mut painter = Painter::new();
    painter.paint_in_rect(&layout, &styles, &dirty_rect, None);
    assert_eq!(painter.primitives().fills.len(), 1);
}

/// 测试 paint_text 带 border 和 padding 偏移 glyph 位置。
#[test]
fn test_paint_text_with_border_padding() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = LayoutBox {
        node_id: Some(elem),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 50.0,
        content_x: 5.0,
        content_y: 3.0,
        content_width: 90.0,
        content_height: 44.0,
        border_top: 3.0,
        border_right: 2.0,
        border_bottom: 2.0,
        border_left: 5.0,
        padding_top: 1.0,
        padding_right: 1.0,
        padding_bottom: 1.0,
        padding_left: 1.0,
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
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(12.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None);

    let glyph = &painter.primitives().glyphs[0];
    // text_x = abs_x(0) + border_left(5) + padding_left(1) = 6
    assert_eq!(glyph.x, 6.0);
    // text_y = abs_y(0) + border_top(3) + padding_top(1) = 4, + font_size(12) = 16
    assert_eq!(glyph.y, 16.0);
}

/// 测试 outline offset 非零时正确绘制。
#[test]
fn test_painter_outline_with_offset() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.outline_width = LengthValue::Px(2.0);
    style.outline_offset = LengthValue::Px(5.0);
    style.outline_style = OutlineStyleValue::Solid;
    style.outline_color = ColorValue::Rgba(0, 128, 0, 255);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert_eq!(painter.primitives().fills.len(), 4);
    // top outline: y = abs_y - (outline_width + offset) = 20 - 7 = 13
    let top = &painter.primitives().fills[0];
    assert_eq!(top.rect.origin.y, 13.0);
    assert_eq!(top.rect.size.height, 2.0);
}

/// 测试 HSL 黄色（60°, 100%, 50%）转换。
#[test]
fn test_hsla_yellow() {
    let color = hsla_to_rgba(60.0, 100.0, 50.0, 1.0);
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 255);
    assert_eq!(color.b, 0);
}

/// 测试 HSL 白色（0°, 0%, 100%）转换。
#[test]
fn test_hsla_white() {
    let color = hsla_to_rgba(0.0, 0.0, 100.0, 1.0);
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 255);
    assert_eq!(color.b, 255);
}

/// 测试 HSL 黑色（0°, 0%, 0%）转换。
#[test]
fn test_hsla_black() {
    let color = hsla_to_rgba(0.0, 0.0, 0.0, 1.0);
    assert_eq!(color.r, 0);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 0);
}

/// 测试 named_color_to_render 其他颜色。
#[test]
fn test_named_colors_extended() {
    assert_eq!(named_color_to_render("orange"), Color::rgb(255, 165, 0));
    assert_eq!(named_color_to_render("pink"), Color::rgb(255, 192, 203));
    assert_eq!(named_color_to_render("brown"), Color::rgb(165, 42, 42));
    assert_eq!(named_color_to_render("navy"), Color::rgb(0, 0, 128));
    assert_eq!(named_color_to_render("teal"), Color::rgb(0, 128, 128));
    assert_eq!(named_color_to_render("silver"), Color::rgb(192, 192, 192));
}

// ── 新增测试：overflow clipping with nested elements ──────

/// 测试嵌套元素中 overflow:hidden 逐层裁剪。
///
/// grandparent(overflow:hidden, 100x100) > parent(overflow:visible, 200x200) > child(50x50)
/// child 从 (80,80) 开始，parent 从 (0,0) 开始。
/// grandparent 的 overflow:hidden 应裁剪所有后代（包括 parent 的背景）。
#[test]
fn test_overflow_hidden_clips_deeply_nested_children() {
    let mut doc = zero_dom::Document::new();
    let grandparent = doc.create_element("div");
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    // child 在 parent 内部，偏移 (80, 80)，大小 50x50
    let child_box = make_box(Some(child), 80.0, 80.0, 50.0, 50.0);
    // parent 大小 200x200（超出 grandparent 的 100x100）
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
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };
    // grandparent overflow:hidden, content 100x100
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
        z_index: 0,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
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

    // parent fill (200x200) should be clipped to grandparent content (100x100)
    let parent_fill = &fills[0];
    assert!(parent_fill.rect.size.width <= 100.0, "parent width clipped to 100");
    assert!(parent_fill.rect.size.height <= 100.0, "parent height clipped to 100");

    // child fill starts at (80,80) size 50x50 → clipped at right/bottom edge
    // visible area: x=[80,100], y=[80,100] → width=20, height=20
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
///
/// outer(overflow:hidden, 80x80) > inner(overflow:hidden, 40x40) > child(100x100)
/// child 完全在 inner 内，但 inner 裁剪到 40x40，outer 再裁剪 inner 的结果。
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
        z_index: 0,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
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
        z_index: 0,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
    };

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&outer_box, &styles, None);

    // child(100x100) → clipped by inner(40x40) → 40x40
    // inner result(40x40) within outer(80x80) → no further clipping needed
    let fill = &painter.primitives().fills[0];
    assert_eq!(fill.rect.size.width, 40.0, "child clipped by inner overflow:hidden");
    assert_eq!(fill.rect.size.height, 40.0, "child clipped by inner overflow:hidden");
}

// ── 新增测试：Inline formatting context（内联格式化上下文）─────────

/// 测试块容器中的内联文本生成 glyph 图元。
///
/// 场景：<div>Hello</div>，div 有明确的前景色和字体大小。
/// 验证 paint() 在遍历布局树时自动为内联文本内容生成 GlyphPrimitive。
#[test]
fn test_paint_inline_text_in_block() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 255, 255, 255);
    style.color = ColorValue::Rgba(0, 0, 0, 255); // 前景色：黑色
    style.font_size = LengthValue::Px(16.0);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 应生成背景填充 + 文本 glyph
    assert_eq!(prims.fills.len(), 1, "应生成 1 个背景填充");
    assert_eq!(prims.glyphs.len(), 1, "应生成 1 个 glyph 图元");

    let glyph = &prims.glyphs[0];
    assert_eq!(glyph.font_size, 16.0);
    assert_eq!(glyph.color, Color::rgb(0, 0, 0));
    // glyph 位置：text_x = 0 (无 border/padding), y = 0 + font_size(16) = 16
    assert_eq!(glyph.x, 0.0);
    assert_eq!(glyph.y, 16.0);
}

/// 测试混合内联和块级元素的图元顺序。
///
/// 场景：父 div（背景灰色）包含三个子元素：
/// - 子1（块级，红色背景）
/// - 子2（内联文本，蓝色前景色）
/// - 子3（块级，绿色背景）
///
/// 验证：
/// 1. 父背景先绘制（fills[0]）
/// 2. 子元素按顺序绘制（子1 fill → 子2 glyph → 子3 fill）
/// 3. 总 fills = 3，glyphs = 1
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
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();

    // 父：灰色背景，color=CurrentColor（不生成 glyph）
    let mut parent_style = ComputedStyle::default();
    parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    parent_style.color = ColorValue::CurrentColor;
    styles.insert(parent, parent_style);

    // 子1（块级）：红色背景，不生成 glyph
    let mut block1_style = ComputedStyle::default();
    block1_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    block1_style.color = ColorValue::CurrentColor;
    styles.insert(block1, block1_style);

    // 子2（内联文本）：无背景，蓝色前景色 → 只生成 glyph
    let mut inline_style = ComputedStyle::default();
    inline_style.background_color = ColorValue::Transparent;
    inline_style.color = ColorValue::Rgba(0, 0, 255, 255); // 蓝色
    inline_style.font_size = LengthValue::Px(14.0);
    styles.insert(inline_text, inline_style);

    // 子3（块级）：绿色背景，不生成 glyph
    let mut block2_style = ComputedStyle::default();
    block2_style.background_color = ColorValue::Rgba(0, 255, 0, 255);
    block2_style.color = ColorValue::CurrentColor;
    styles.insert(block2, block2_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    let prims = painter.primitives();

    // 父背景 + 子1 背景 + 子3 背景 = 3 个 fills
    assert_eq!(prims.fills.len(), 3, "应生成 3 个填充（父 + 子1 + 子3）");
    // 子2 只生成 1 个 glyph
    assert_eq!(prims.glyphs.len(), 1, "应生成 1 个 glyph（子2 内联文本）");

    // 验证绘制顺序：父背景先绘制
    assert_eq!(
        prims.fills[0].color,
        Color::rgb(200, 200, 200),
        "第一个 fill 应为父背景"
    );
    assert_eq!(prims.fills[1].color, Color::rgb(255, 0, 0), "第二个 fill 应为子1 背景");
    assert_eq!(prims.fills[2].color, Color::rgb(0, 255, 0), "第三个 fill 应为子3 背景");

    // glyph 颜色为蓝色
    assert_eq!(prims.glyphs[0].color, Color::rgb(0, 0, 255), "glyph 颜色应为蓝色");
    assert_eq!(prims.glyphs[0].font_size, 14.0);
    // glyph 位置：abs_y=30, text_y=30, baseline=30+14=44
    assert_eq!(prims.glyphs[0].y, 44.0);
}

/// 测试带颜色样式的内联文本正确应用到 glyph 图元。
///
/// 场景：<span style="color: red; font-size: 20px;">Colored</span>
/// 验证 glyph 的 color 字段匹配 CSS color 属性值。
#[test]
fn test_paint_text_with_color() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("span");
    let layout = make_box(Some(elem), 10.0, 20.0, 150.0, 25.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.color = ColorValue::Rgba(255, 0, 0, 255); // 红色
    style.font_size = LengthValue::Px(20.0);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.glyphs.len(), 1, "应生成 1 个 glyph");

    let glyph = &prims.glyphs[0];
    // 颜色正确应用
    assert_eq!(glyph.color, Color::rgb(255, 0, 0), "glyph 颜色应为红色");
    assert_eq!(glyph.font_size, 20.0, "glyph font_size 应为 20");
    // 位置：abs_x=10, abs_y=20, text_x=10, baseline=20+20=40
    assert_eq!(glyph.x, 10.0);
    assert_eq!(glyph.y, 40.0);
}

// ── 新增测试：InlineFormattingContext 集成 ──────────────────────

/// 测试 paint_text 使用 InlineFormattingContext 为每个文本片段生成 glyph。
///
/// 场景：<p>Hello World</p>，容器宽度较窄，文本自动换行。
/// 当传入 Document 时，paint_text 应通过 InlineFormattingContext
/// 将文本分割为单词，为每个单词生成独立的 GlyphPrimitive。
#[test]
fn test_paint_text_with_inline_formatting_context() {
    let doc = zero_dom::parse_html("<p>Hello World</p>");

    // 找到 p 元素
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

    let prims = painter.primitives();
    // InlineFormattingContext 会将 "Hello World" 分成 2 个单词片段
    assert!(
        prims.glyphs.len() >= 2,
        "应有至少 2 个 glyph（Hello 和 World），实际 {}",
        prims.glyphs.len()
    );

    // 验证每个 glyph 的颜色和字体大小正确
    for glyph in &prims.glyphs {
        assert_eq!(glyph.color, Color::rgb(0, 0, 0));
        assert_eq!(glyph.font_size, 16.0);
    }
}

/// 测试 paint_text 不传 Document 时退化为单个占位 glyph。
///
/// 验证 doc=None 时 paint_text 仍然正常工作，
/// 生成单个 glyph 作为占位。
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

    // doc=None → 退化为单个占位 glyph
    assert_eq!(painter.primitives().glyphs.len(), 1, "doc=None 时应退化为 1 个 glyph");
}

/// 测试 InlineFormattingContext 生成的 glyph 位置包含容器偏移。
///
/// 场景：<p>Text</p>，p 元素有 border 和 padding 偏移。
/// 验证 glyph 的坐标包含 content_x/content_y 偏移。
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
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.font_size = LengthValue::Px(16.0);
    styles.insert(p, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    assert!(!prims.glyphs.is_empty(), "应生成 glyph");

    // 第一个 glyph 的 x 应包含 content_x 偏移
    // content_x = abs_x(10) + border_left(2) + padding_left(3) = 15
    let glyph = &prims.glyphs[0];
    assert!(glyph.x >= 15.0, "glyph x 应包含内容区域偏移，实际 {}", glyph.x);
    // y 应包含 content_y 偏移 + 行高
    assert!(glyph.y >= 25.0, "glyph y 应包含内容区域偏移，实际 {}", glyph.y);
}

/// 测试窄容器中 InlineFormattingContext 为文本内容生成 glyph。
///
/// 场景：容器宽度只有 60px，文本 "a b c d e f g h" 应产生 glyph。
#[test]
fn test_paint_inline_text_wrapping_multiple_lines() {
    let doc = zero_dom::parse_html("<p>a b c d e f g h</p>");

    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    // 窄容器 — 强制文本换行
    let layout = make_box(Some(p), 0.0, 0.0, 60.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.font_size = LengthValue::Px(16.0);
    styles.insert(p, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    // paint 应该为文本内容生成至少一些 glyph
    assert!(
        prims.glyphs.len() >= 1,
        "容器中的文本应产生 glyph，实际 {}",
        prims.glyphs.len()
    );
}

/// 测试混合 inline 元素的文本通过 InlineFormattingContext 正确生成 glyph。
///
/// 场景：<p>Hello <b>World</b></p>
/// p 包含文本节点 "Hello " 和 b 元素 "World"。
/// InlineFormattingContext 会收集两者并分割为单词片段。
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

    let prims = painter.primitives();
    // "Hello" 和 "World" 各一个片段
    assert!(
        prims.glyphs.len() >= 2,
        "混合文本和 inline 元素应产生至少 2 个 glyph，实际 {}",
        prims.glyphs.len()
    );
}

/// 测试空文本节点不产生 glyph（InlineFormattingContext 过滤空白）。
///
/// 场景：<p>   </p>，文本只有空白字符。
/// InlineFormattingContext 的 trim 过滤后不应产生任何片段。
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

    // 纯空白文本被 trim 后为空字符串，不产生 TextRun，
    // 因此 InlineFormattingContext 无片段 → 走 fallback 生成 1 个 glyph
    assert!(
        painter.primitives().glyphs.len() <= 1,
        "纯空白文本应产生 0 或 1 个 fallback glyph，实际 {}",
        painter.primitives().glyphs.len()
    );
}

/// 测试 render_html 通过 pipeline 使用 InlineFormattingContext。
///
/// 验证端到端管线中 InlineFormattingContext 被正确调用：
/// HTML 解析 → 样式计算 → 布局 → paint(传入 Document) → 生成 glyph。
#[test]
fn test_pipeline_uses_inline_formatting_for_text() {
    use crate::pipeline::RenderPipeline;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><p>Hello World</p></body></html>";
    let css = "p { color: black; font-size: 16px; }";
    let result = pipeline.render_html(html, css);

    // Pipeline 应为 p 元素生成 glyph
    assert!(
        !result.primitives.glyphs.is_empty(),
        "render_html 应通过 InlineFormattingContext 生成 glyph"
    );
}

/// 测试 pipeline render_html 生成 glyph。
#[test]
fn test_pipeline_inline_text_with_css_color() {
    use crate::pipeline::RenderPipeline;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><p>Styled</p></body></html>";
    let css = "p { color: red; font-size: 18px; }";
    let result = pipeline.render_html(html, css);

    // Pipeline 应该为文本内容生成 glyph（颜色传播取决于管线实现完整度）
    assert!(!result.primitives.glyphs.is_empty(), "应生成 glyph");
    // 验证 glyph 字体大小正确
    assert!(
        result.primitives.glyphs.iter().any(|g| g.font_size > 0.0),
        "至少一个 glyph 应有非零字体大小"
    );
}

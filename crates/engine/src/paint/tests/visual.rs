#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use std::collections::HashMap;

use zero_css_parser::values::{ColorValue, LengthValue, TransformFunction, TransformValue};
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_style_system::{
    BackgroundClipComputedValue, BackgroundImageComputedValue, BackgroundOriginComputedValue,
    BackgroundPositionComputedValue, BackgroundRepeatComputedValue, BackgroundSizeComputedValue, BorderStyleValue,
    ComputedStyle, OutlineStyleValue,
};

use super::super::color::{hsla_to_rgba, named_color_to_render};
use super::super::helpers::{BorderRadiusSpec, apply_transform_offset};
use super::super::painter::Painter;

/// 辅助函数：创建简单 LayoutBox。
pub(super) fn make_box(node_id: Option<NodeId>, x: f32, y: f32, width: f32, height: f32) -> LayoutBox {
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
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
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
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
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
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Clip,
        overflow_y: OverflowClip::Clip,
        ..Default::default()
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

    // border-radius 生成 RoundedRectPrimitive
    assert_eq!(painter.primitives().rounded_rects.len(), 1);
    assert_eq!(painter.primitives().fills.len(), 0);
    assert_eq!(painter.primitives().rounded_rects[0].color, Color::rgb(255, 0, 0));
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
    painter.paint_text(&layout, 10.0, 20.0, &styles[&elem], None, None);

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
    painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None, None);
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
    painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None, None);
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
    painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None, None);

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
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
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

    // Border-radius generates RoundedRectPrimitive instead of flat FillPrimitive
    assert_eq!(painter.primitives().rounded_rects.len(), 1);
    assert_eq!(painter.primitives().rounded_rects[0].rect.size.width, 200.0);
    assert_eq!(painter.primitives().rounded_rects[0].rect.size.height, 100.0);
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
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(12.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None, None);

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

// ── background-position / background-size / background-clip 测试 ──────

/// 辅助：创建带 node_id 和样式的背景测试环境。
fn setup_bg_test() -> (Painter, zero_dom::NodeId, LayoutBox, HashMap<NodeId, ComputedStyle>) {
    let mut doc = zero_dom::Document::new();
    let node_id = doc.create_element("div");
    let layout = make_box(Some(node_id), 0.0, 0.0, 200.0, 100.0);
    let styles = HashMap::new();
    (Painter::new(), node_id, layout, styles)
}

/// 辅助：创建带 border/padding 的 box。
fn make_box_with_padding(
    node_id: Option<NodeId>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    bt: f32,
    bl: f32,
    pt: f32,
    pl: f32,
    cw: f32,
    ch: f32,
) -> LayoutBox {
    let mut b = make_box(node_id, x, y, w, h);
    b.border_top = bt;
    b.border_left = bl;
    b.padding_top = pt;
    b.padding_left = pl;
    b.content_width = cw;
    b.content_height = ch;
    b
}

/// 测试 background-position: center 居中偏移。
#[test]
fn test_background_position_center() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_position = vec![BackgroundPositionComputedValue::Center];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.origin.x, 0.0);
    assert_eq!(img[0].rect.origin.y, 0.0);
    assert_eq!(img[0].rect.size.width, 200.0);
    assert_eq!(img[0].rect.size.height, 100.0);
}

/// 测试 background-position: right bottom 右下角偏移。
#[test]
fn test_background_position_right_bottom() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_position = vec![BackgroundPositionComputedValue::TwoValue(
        Box::new(BackgroundPositionComputedValue::Right),
        Box::new(BackgroundPositionComputedValue::Bottom),
    )];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.origin.x, 0.0);
    assert_eq!(img[0].rect.origin.y, 0.0);
}

/// 测试 background-position 长度值偏移。
#[test]
fn test_background_position_length() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.background_position = vec![BackgroundPositionComputedValue::TwoValue(
        Box::new(BackgroundPositionComputedValue::Length(20.0)),
        Box::new(BackgroundPositionComputedValue::Length(10.0)),
    )];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.origin.x, 20.0);
    assert_eq!(img[0].rect.origin.y, 10.0);
}

/// 测试 background-position 百分比偏移。
#[test]
fn test_background_position_percent() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.background_size = vec![BackgroundSizeComputedValue::Length(50.0)];
    style.background_position = vec![BackgroundPositionComputedValue::Percent(50.0)];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.size.width, 50.0);
    // percent 50: offset_x = (200 - 50) * 50 / 100 = 75.0
    assert_eq!(img[0].rect.origin.x, 75.0);
}

/// 测试 background-size: cover 覆盖容器。
#[test]
fn test_background_size_cover() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_size = vec![BackgroundSizeComputedValue::Cover];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.size.width, 200.0);
    assert_eq!(img[0].rect.size.height, 100.0);
}

/// 测试 background-size: contain 包含在容器内。
#[test]
fn test_background_size_contain() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_size = vec![BackgroundSizeComputedValue::Contain];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.size.width, 200.0);
    assert_eq!(img[0].rect.size.height, 100.0);
}

/// 测试 background-size: Length(100px) 固定宽度。
#[test]
fn test_background_size_length() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 300.0, 200.0);
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.background_size = vec![BackgroundSizeComputedValue::Length(100.0)];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.size.width, 100.0);
    let expected_h = 100.0 * 200.0 / 300.0;
    assert!((img[0].rect.size.height - expected_h).abs() < 0.01);
}

/// 测试 background-size: Percent(50%) 百分比尺寸。
#[test]
fn test_background_size_percent() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 400.0, 200.0);
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.background_size = vec![BackgroundSizeComputedValue::Percent(50.0)];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.size.width, 200.0);
    let expected_h = 200.0 * 200.0 / 400.0;
    assert!((img[0].rect.size.height - expected_h).abs() < 0.01);
}

/// 测试 background-clip: content-box 限制背景绘制区域。
#[test]
fn test_background_clip_content_box() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box_with_padding(Some(nid), 0.0, 0.0, 200.0, 100.0, 10.0, 10.0, 5.0, 5.0, 180.0, 80.0);
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.background_clip = BackgroundClipComputedValue::ContentBox;
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].rect.origin.x, 15.0);
    assert_eq!(fills[0].rect.origin.y, 15.0);
    assert_eq!(fills[0].rect.size.width, 180.0);
    assert_eq!(fills[0].rect.size.height, 80.0);
}

/// 测试 background-clip: padding-box 限制背景绘制区域。
#[test]
fn test_background_clip_padding_box() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box_with_padding(Some(nid), 0.0, 0.0, 200.0, 100.0, 10.0, 10.0, 0.0, 0.0, 190.0, 90.0);
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.background_clip = BackgroundClipComputedValue::PaddingBox;
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].rect.origin.x, 10.0);
    assert_eq!(fills[0].rect.origin.y, 10.0);
    assert_eq!(fills[0].rect.size.width, 190.0);
    assert_eq!(fills[0].rect.size.height, 90.0);
}

/// 测试 background-clip: border-box（默认值）与无 border 时等价于整盒。
#[test]
fn test_background_clip_border_box_default() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 128, 0, 255);
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].rect.origin.x, 0.0);
    assert_eq!(fills[0].rect.origin.y, 0.0);
    assert_eq!(fills[0].rect.size.width, 200.0);
    assert_eq!(fills[0].rect.size.height, 100.0);
}

/// 测试 background-origin: content-box 影响图片定位。
#[test]
fn test_background_origin_content_box() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box_with_padding(Some(nid), 0.0, 0.0, 200.0, 100.0, 10.0, 10.0, 5.0, 5.0, 180.0, 80.0);
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_origin = BackgroundOriginComputedValue::ContentBox;
    // R2312：no-repeat 隔离 origin 定位断言（repeat 现平铺 painting area=clip box，tile 数会变）。
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.origin.x, 15.0);
    assert_eq!(img[0].rect.origin.y, 15.0);
    assert_eq!(img[0].rect.size.width, 180.0);
    assert_eq!(img[0].rect.size.height, 80.0);
}

/// R2312：background-clip 应用于背景图像（painting area = background-clip box，非 origin box）。
/// 旧 impl 误把 origin box 当 clip；本测试守 `background-clip: content-box` 把图像裁到 content-box。
#[test]
fn test_r2312_background_clip_applied_to_image() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    // border 10 + padding 5 → content-box at (15,15) 180x80；padding-box (10,10) 190x90。
    let layout = make_box_with_padding(Some(nid), 0.0, 0.0, 200.0, 100.0, 10.0, 10.0, 5.0, 5.0, 180.0, 80.0);

    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_clip = BackgroundClipComputedValue::ContentBox; // origin 仍默认 padding-box
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    // 图像从 padding-box origin (10,10) 起绘（Auto size=190×90），被裁到 content-box (15,15) 180×80。
    assert_eq!(img[0].rect.origin.x, 15.0);
    assert_eq!(img[0].rect.origin.y, 15.0);
    assert_eq!(img[0].rect.size.width, 180.0);
    assert_eq!(img[0].rect.size.height, 80.0);
}

/// 测试 background-position + background-size 组合。
#[test]
fn test_background_position_right_bottom_with_small_size() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.background_size = vec![BackgroundSizeComputedValue::Length(50.0)];
    style.background_position = vec![BackgroundPositionComputedValue::TwoValue(
        Box::new(BackgroundPositionComputedValue::Right),
        Box::new(BackgroundPositionComputedValue::Bottom),
    )];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.size.width, 50.0);
    assert_eq!(img[0].rect.origin.x, 150.0);
    assert_eq!(img[0].rect.origin.y, 75.0);
}

/// 测试渐变也受 background-position/size 影响。
#[test]
fn test_gradient_with_position_and_size() {
    use zero_css_parser::values::{GradientColorStop, GradientDirection, GradientValue, LinearGradient};

    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Linear(
        LinearGradient {
            interpolation: Default::default(),
            direction: GradientDirection::ToRight,
            stops: vec![
                GradientColorStop {
                    color: ColorValue::Rgba(255, 0, 0, 255),
                    position: None,
                },
                GradientColorStop {
                    color: ColorValue::Rgba(0, 0, 255, 255),
                    position: None,
                },
            ],
            repeating: false,
        },
    ))];
    style.background_size = vec![BackgroundSizeComputedValue::Percent(50.0)];
    style.background_position = vec![BackgroundPositionComputedValue::TwoValue(
        Box::new(BackgroundPositionComputedValue::Left),
        Box::new(BackgroundPositionComputedValue::Top),
    )];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let gradients = &painter.primitives().gradients;
    assert_eq!(gradients.len(), 1);
    assert_eq!(gradients[0].rect.size.width, 100.0);
    assert_eq!(gradients[0].rect.origin.x, 0.0);
    assert_eq!(gradients[0].rect.origin.y, 0.0);
}

// ── border-image 渲染测试 ──────────────────────────────────────────

/// 测试 border-image: url() 生成 9 宫格图片图元。
#[test]
fn test_border_image_url_9region() {
    use zero_style_system::BorderImageSourceComputedValue;

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 200.0, 100.0);
    layout.border_top = 10.0;
    layout.border_right = 10.0;
    layout.border_bottom = 10.0;
    layout.border_left = 10.0;

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 255, 255, 255);
    style.border_image_source = BorderImageSourceComputedValue::Url("border.png".to_string());
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 4 corners + 4 edges = 8 image primitives (fill=false, no center)
    let images = &painter.primitives().images;
    assert!(
        images.len() >= 8,
        "border-image should generate at least 8 image primitives, got {}",
        images.len()
    );
}

/// 测试 border-image-source: none 不生成图片图元。
#[test]
fn test_border_image_none() {
    use zero_style_system::BorderImageSourceComputedValue;

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 200.0, 100.0);
    layout.border_top = 10.0;
    layout.border_right = 10.0;
    layout.border_bottom = 10.0;
    layout.border_left = 10.0;

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 255, 255, 255);
    // default is None
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let images = &painter.primitives().images;
    assert_eq!(
        images.len(),
        0,
        "border-image:none should not generate image primitives"
    );
}

/// 测试 border-image 带不同边框宽度（不对称）。
#[test]
fn test_border_image_asymmetric_borders() {
    use zero_style_system::BorderImageSourceComputedValue;

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 300.0, 150.0);
    layout.border_top = 5.0;
    layout.border_right = 15.0;
    layout.border_bottom = 10.0;
    layout.border_left = 20.0;

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.border_image_source = BorderImageSourceComputedValue::Url("frame.png".to_string());
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let images = &painter.primitives().images;
    assert!(
        images.len() >= 8,
        "asymmetric border-image should generate at least 8 images, got {}",
        images.len()
    );

    // 验证左上角位置和尺寸
    let top_left = &images[0];
    assert_eq!(top_left.rect.origin.x, 0.0);
    assert_eq!(top_left.rect.origin.y, 0.0);
    assert_eq!(top_left.rect.size.width, 20.0); // border-left
    assert_eq!(top_left.rect.size.height, 5.0); // border-top
}

/// 测试 border-image 带无 border 时跳过绘制。
#[test]
fn test_border_image_no_border() {
    use zero_style_system::BorderImageSourceComputedValue;

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 200.0, 100.0);
    // no borders set

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("border.png".to_string());
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let images = &painter.primitives().images;
    assert_eq!(images.len(), 0, "no border width should skip border-image");
}

/// 测试 column-rule: solid 在 3 列之间绘制 2 条分隔线。
#[test]
fn test_column_rules_solid() {
    use zero_style_system::{ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 600.0, 200.0);

    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Number(3);
    style.column_gap = LengthValue::Px(20.0);
    style.column_rule_style = ColumnRuleStyleComputedValue::Solid;
    style.column_rule_width = ColumnRuleWidthComputedValue::Thin;
    style.column_rule_color = ColorValue::Rgba(128, 128, 128, 255);
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let fills = &painter.primitives().fills;
    // 3 列 → 2 条 rule → 2 个 fill 图元（background 可能也产生 fill，但 rule 的 fill 至少 2 个）
    // 检查至少有 rule 的 fill（每条 rule 是一个细矩形）
    let rule_fills: Vec<_> = fills
        .iter()
        .filter(|f| f.color.a > 0 && f.rect.size.width < 5.0 && f.rect.size.height > 100.0)
        .collect();
    assert!(
        rule_fills.len() >= 2,
        "3 columns should produce at least 2 column-rule fills, got {} rule fills",
        rule_fills.len()
    );
}

/// 测试 column-rule-style: none 不绘制分隔线。
#[test]
fn test_column_rules_none() {
    use zero_style_system::ColumnCountComputedValue;

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 600.0, 200.0);

    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Number(3);
    // column_rule_style 默认为 None
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // column-rule:none 不应产生额外的 stroke 图元
    let strokes = &painter.primitives().strokes;
    assert_eq!(strokes.len(), 0, "column-rule:none should not generate strokes");
}

/// 测试 column-rule-style: dashed 生成 stroke 图元。
#[test]
fn test_column_rules_dashed() {
    use zero_render_foundation::primitive::LineStyle;
    use zero_style_system::{ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 400.0, 100.0);

    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Number(2);
    style.column_gap = LengthValue::Px(10.0);
    style.column_rule_style = ColumnRuleStyleComputedValue::Dashed;
    style.column_rule_width = ColumnRuleWidthComputedValue::Medium;
    style.column_rule_color = ColorValue::Rgba(0, 0, 0, 255);
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let strokes = &painter.primitives().strokes;
    let dashed: Vec<_> = strokes.iter().filter(|s| s.style == LineStyle::Dashed).collect();
    assert!(
        dashed.len() >= 1,
        "2 columns with dashed rule should produce at least 1 dashed stroke, got {}",
        dashed.len()
    );
}

/// 测试 column-count:1（只有 1 列）不绘制 rule。
#[test]
fn test_column_rules_single_column() {
    use zero_style_system::{ColumnCountComputedValue, ColumnRuleStyleComputedValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 200.0, 100.0);

    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Number(1);
    style.column_rule_style = ColumnRuleStyleComputedValue::Solid;
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let strokes = &painter.primitives().strokes;
    assert_eq!(strokes.len(), 0, "1 column should not produce column-rule strokes");
}

/// 测试 list-style-image:url() 生成 ImagePrimitive 标记。
#[test]
fn test_list_style_image_url() {
    use zero_style_system::ListStyleImageComputedValue;

    let mut doc = zero_dom::Document::new();
    let ul = doc.create_element("ul");
    let li = doc.create_element("li");
    let _ = doc.append_child(ul, li);

    let layout = make_box(Some(li), 0.0, 0.0, 200.0, 30.0);

    let mut style = ComputedStyle::default();
    style.list_style_image = ListStyleImageComputedValue::Url("bullet.png".to_string());
    let mut styles = HashMap::new();
    styles.insert(li, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let images = &painter.primitives().images;
    assert!(
        images
            .iter()
            .any(|img| img.rect.size.width > 0.0 && img.rect.size.height > 0.0),
        "list-style-image should generate at least one image primitive"
    );
}

/// 测试 list-style-image:none 不生成图片图元。
#[test]
fn test_list_style_image_none() {
    let mut doc = zero_dom::Document::new();
    let ul = doc.create_element("ul");
    let li = doc.create_element("li");
    let _ = doc.append_child(ul, li);

    let layout = make_box(Some(li), 0.0, 0.0, 200.0, 30.0);

    let mut style = ComputedStyle::default();
    // list-style-image defaults to None
    style.list_style_type = zero_css_parser::values::ListStyleTypeValue::Disc;
    let mut styles = HashMap::new();
    styles.insert(li, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let images = &painter.primitives().images;
    assert_eq!(
        images.len(),
        0,
        "list-style-image:none should not generate image primitives"
    );
}

/// R1882：list-style-type:disc 生成实心圆 marker（圆角矩形 radius=size/2），非方块。
///
/// CSS §12.5 / chromium：disc 为实心圆。旧实现用 add_fill(Rect) 绘方块。修复后用
/// RoundedRectPrimitive（radius = marker_size/2 = 正方形四角全圆 → 圆）近似实心圆。
#[test]
fn test_list_style_disc_renders_circle() {
    let mut doc = zero_dom::Document::new();
    let ul = doc.create_element("ul");
    let li = doc.create_element("li");
    let _ = doc.append_child(ul, li);

    let layout = make_box(Some(li), 0.0, 0.0, 200.0, 30.0);

    let mut style = ComputedStyle::default();
    style.list_style_type = zero_css_parser::values::ListStyleTypeValue::Disc;
    let mut styles = HashMap::new();
    styles.insert(li, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    // disc 应产出 rounded_rect（实心圆），非 fill（方块）。
    assert!(
        !prims.rounded_rects.is_empty(),
        "list-style-type:disc 应生成 rounded_rect（实心圆 marker），实际 rounded_rects 为空"
    );
    let r = &prims.rounded_rects[0];
    // 圆 = 正方形四角 radius = size/2。
    let size = r.rect.size.width;
    assert!(
        (r.top_left_radius - size / 2.0).abs() < 0.01,
        "disc marker 应四角 radius=size/2（实心圆），实际 top_left_radius={} size={}",
        r.top_left_radius,
        size
    );
    assert!(
        prims
            .fills
            .iter()
            .all(|f| (f.rect.size.width - f.rect.size.height).abs() > 0.5 || f.rect.size.width < 2.0),
        "disc marker 不应残留方块 fill（与圆 marker 同尺寸的 fill 应消失）"
    );
}

/// R1883：list-style-type:circle 生成空心圆 outline（PathStroke 多边形），非 2:1 胶囊。
///
/// CSS §12.5 / chromium：circle 为空心圆。旧实现用 add_stroke（length=width + Round cap）
/// 实为 2:1 胶囊（椭圆）。修复后用 add_path_stroke 多边形（24 点圆周）描真圆。
#[test]
fn test_list_style_circle_renders_true_circle() {
    let mut doc = zero_dom::Document::new();
    let ul = doc.create_element("ul");
    let li = doc.create_element("li");
    let _ = doc.append_child(ul, li);

    let layout = make_box(Some(li), 0.0, 0.0, 200.0, 30.0);

    let mut style = ComputedStyle::default();
    style.list_style_type = zero_css_parser::values::ListStyleTypeValue::Circle;
    let mut styles = HashMap::new();
    styles.insert(li, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    // circle 应产出 path_stroke（多边形真圆），非 stroke（线段胶囊）。
    assert!(
        !prims.path_strokes.is_empty(),
        "list-style-type:circle 应生成 path_stroke（多边形真圆 outline），实际 path_strokes 为空"
    );
    // 24 点圆周 = 48 个 f32 顶点。
    assert_eq!(
        prims.path_strokes[0].vertices.len(),
        48,
        "circle marker 应为 24 点圆周多边形（48 f32），实际 {}",
        prims.path_strokes[0].vertices.len()
    );
}

/// 测试 empty-cells:hide 跳过空单元格的背景绘制。
#[test]
fn test_empty_cells_hide() {
    use zero_style_system::EmptyCellsComputedValue;

    let mut doc = zero_dom::Document::new();
    let td = doc.create_element("td");
    let layout = make_box(Some(td), 0.0, 0.0, 100.0, 50.0);

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.empty_cells = EmptyCellsComputedValue::Hide;
    // No children → empty cell
    let mut styles = HashMap::new();
    styles.insert(td, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // empty-cells:hide should skip background for empty cell
    let fills = &painter.primitives().fills;
    assert!(
        fills.iter().all(|f| f.color.r != 255 || f.color.a == 0),
        "empty-cells:hide should not render background for empty cell"
    );
}

/// 测试 empty-cells:show 绘制空单元格的背景。
#[test]
fn test_empty_cells_show() {
    use zero_style_system::EmptyCellsComputedValue;

    let mut doc = zero_dom::Document::new();
    let td = doc.create_element("td");
    let layout = make_box(Some(td), 0.0, 0.0, 100.0, 50.0);

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.empty_cells = EmptyCellsComputedValue::Show;
    let mut styles = HashMap::new();
    styles.insert(td, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let fills = &painter.primitives().fills;
    assert!(
        fills.iter().any(|f| f.color.r == 255 && f.color.a > 0),
        "empty-cells:show should render background for empty cell"
    );
}

// ═══════════════════════════════════════════════════════════════
//  CSS mix-blend-mode 渲染集成测试
// ═══════════════════════════════════════════════════════════════

/// 测试 mix-blend-mode:multiply 生成 BlendModePrimitive。
#[test]
fn test_mix_blend_mode_multiply_generates_blend_primitive() {
    use zero_style_system::MixBlendModeComputedValue;

    let mut doc = zero_dom::Document::new();
    let div = doc.create_element("div");
    let layout = make_box(Some(div), 0.0, 0.0, 200.0, 100.0);

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.mix_blend_mode = MixBlendModeComputedValue::Multiply;

    let mut styles = HashMap::new();
    styles.insert(div, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        !painter.primitives().blend_modes.is_empty(),
        "mix-blend-mode:multiply should generate BlendModePrimitive"
    );
    assert_eq!(
        painter.primitives().blend_modes[0].mode,
        zero_render_foundation::primitive::BlendMode::Multiply
    );
}

/// 测试 mix-blend-mode:normal 不生成 BlendModePrimitive。
#[test]
fn test_mix_blend_mode_normal_no_blend_primitive() {
    use zero_style_system::MixBlendModeComputedValue;

    let mut doc = zero_dom::Document::new();
    let div = doc.create_element("div");
    let layout = make_box(Some(div), 0.0, 0.0, 200.0, 100.0);

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.mix_blend_mode = MixBlendModeComputedValue::Normal;

    let mut styles = HashMap::new();
    styles.insert(div, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().blend_modes.is_empty(),
        "mix-blend-mode:normal should not generate BlendModePrimitive"
    );
}

/// 测试 mix-blend-mode:screen 生成正确模式。
#[test]
fn test_mix_blend_mode_screen_generates_blend_primitive() {
    use zero_style_system::MixBlendModeComputedValue;

    let mut doc = zero_dom::Document::new();
    let div = doc.create_element("div");
    let layout = make_box(Some(div), 0.0, 0.0, 200.0, 100.0);

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 0, 255, 255);
    style.mix_blend_mode = MixBlendModeComputedValue::Screen;

    let mut styles = HashMap::new();
    styles.insert(div, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        !painter.primitives().blend_modes.is_empty(),
        "mix-blend-mode:screen should generate BlendModePrimitive"
    );
    assert_eq!(
        painter.primitives().blend_modes[0].mode,
        zero_render_foundation::primitive::BlendMode::Screen
    );
}

//! CSS 表格和 3D 属性渲染的单元测试。
//!
//! 覆盖 scroll-snap、perspective、backface-visibility、transform-style、
//! border-spacing、caption-side 指示器渲染。

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;

use zero_css_parser::values::ColorValue;
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::{
    BackfaceVisibilityValue, BorderSpacingComputedValue, CaptionSideValue, LengthValue, ScrollSnapAlign,
    ScrollSnapAxis, ScrollSnapStrictness, ScrollSnapType, TransformStyleValue,
};

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
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    }
}

// ── scroll-snap 测试 ──────────────────────────────────────────────

#[test]
fn test_scroll_snap_mandatory_x_axis() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("red".into());
    style.scroll_snap_type = ScrollSnapType {
        strictness: ScrollSnapStrictness::Mandatory,
        axis: ScrollSnapAxis::X,
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // Mandatory X 应在底部生成水平吸附线
    let prims = painter.primitives();
    let has_snap_line = prims
        .fills
        .iter()
        .any(|f| f.rect.size.height == 2.0 && (f.rect.origin.y - 48.0).abs() < 1.0 && f.rect.size.width == 100.0);
    assert!(has_snap_line, "mandatory X 应生成底部水平吸附线");
}

#[test]
fn test_scroll_snap_proximity_y_axis() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("blue".into());
    style.scroll_snap_type = ScrollSnapType {
        strictness: ScrollSnapStrictness::Proximity,
        axis: ScrollSnapAxis::Y,
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // Proximity Y 应在右侧生成垂直吸附线
    let prims = painter.primitives();
    let has_snap_line = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 2.0 && (f.rect.origin.x - 98.0).abs() < 1.0 && f.rect.size.height == 50.0);
    assert!(has_snap_line, "proximity Y 应生成右侧垂直吸附线");
}

#[test]
fn test_scroll_snap_both_axes() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("green".into());
    style.scroll_snap_type = ScrollSnapType {
        strictness: ScrollSnapStrictness::Mandatory,
        axis: ScrollSnapAxis::Both,
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // Both 轴应生成水平和垂直两条线
    let prims = painter.primitives();
    let has_horizontal = prims
        .fills
        .iter()
        .any(|f| f.rect.size.height == 2.0 && f.rect.size.width == 100.0);
    let has_vertical = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 2.0 && f.rect.size.height == 50.0);
    assert!(has_horizontal, "Both 应生成水平线");
    assert!(has_vertical, "Both 应生成垂直线");
}

#[test]
fn test_scroll_snap_none_no_render() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("red".into());
    style.scroll_snap_type = ScrollSnapType {
        strictness: ScrollSnapStrictness::None,
        axis: ScrollSnapAxis::Both,
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // None strictness 不应有 2px 高/宽的吸附线
    let prims = painter.primitives();
    let has_snap_line = prims
        .fills
        .iter()
        .any(|f| f.rect.size.height == 2.0 || f.rect.size.width == 2.0);
    assert!(!has_snap_line, "None strictness 不应渲染吸附线");
}

#[test]
fn test_scroll_snap_align_start_point() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("red".into());
    style.scroll_snap_type = ScrollSnapType {
        strictness: ScrollSnapStrictness::Mandatory,
        axis: ScrollSnapAxis::X,
    };
    style.scroll_snap_align = ScrollSnapAlign::Start;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // Start align 应在左上角生成 4×4 小方块
    let prims = painter.primitives();
    let has_start_point = prims.fills.iter().any(|f| {
        f.rect.size.width == 4.0 && f.rect.size.height == 4.0 && f.rect.origin.x == 0.0 && f.rect.origin.y == 0.0
    });
    assert!(has_start_point, "Start 应在左上角生成对齐点");
}

#[test]
fn test_scroll_snap_align_center_point() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("red".into());
    style.scroll_snap_type = ScrollSnapType {
        strictness: ScrollSnapStrictness::Mandatory,
        axis: ScrollSnapAxis::X,
    };
    style.scroll_snap_align = ScrollSnapAlign::Center;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_center = prims.fills.iter().any(|f| {
        f.rect.size.width == 4.0
            && f.rect.size.height == 4.0
            && (f.rect.origin.x - 48.0).abs() < 1.0
            && (f.rect.origin.y - 23.0).abs() < 1.0
    });
    assert!(has_center, "Center 应在中间生成对齐点");
}

// ── perspective 测试 ──────────────────────────────────────────────

#[test]
fn test_perspective_indicator_renders() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("white".into());
    style.perspective = LengthValue::Px(500.0);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // perspective 应生成消失点标记（1×1 或 1×6 等 fill）
    let prims = painter.primitives();
    assert!(prims.fills.len() > 1, "perspective 应生成消失点标记 fills");
}

#[test]
fn test_perspective_indicator_renders_relative_length() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("white".into());
    style.perspective = LengthValue::Em(2.0);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert!(
        prims.fills.len() > 1,
        "relative perspective length 应生成消失点标记 fills"
    );
}

#[test]
fn test_perspective_zero_no_render() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("white".into());
    style.perspective = LengthValue::Px(0.0);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // perspective=0 只应有背景 fill，无消失点标记
    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 1, "perspective=0 不应渲染消失点标记");
}

// ── backface-visibility 测试 ──────────────────────────────────────

#[test]
fn test_backface_visibility_hidden_renders() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("white".into());
    style.backface_visibility = BackfaceVisibilityValue::Hidden;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // hidden 应生成虚线边框（多条短 fill）
    let prims = painter.primitives();
    assert!(prims.fills.len() > 1, "backface-visibility:hidden 应生成虚线边框");
}

// ── transform-style 测试 ──────────────────────────────────────────

#[test]
fn test_transform_style_preserve3d_renders() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("white".into());
    style.transform_style = TransformStyleValue::Preserve3d;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // preserve-3d 应生成 3D 立方体图标（3 fills：正面+顶面+右面）
    let prims = painter.primitives();
    // 背景占 1 fill，立方体占 3 fills = 4
    assert!(prims.fills.len() >= 4, "preserve-3d 应生成 3D 立方体图标");
}

// ── border-spacing 测试 ──────────────────────────────────────────

#[test]
fn test_border_spacing_renders() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("white".into());
    style.border_spacing = BorderSpacingComputedValue {
        horizontal: 10.0,
        vertical: 5.0,
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // border-spacing 应生成 2 条间距标记线（1px 高的 + 1px 宽的）
    let prims = painter.primitives();
    let has_h_marker = prims
        .fills
        .iter()
        .any(|f| f.rect.size.height == 1.0 && f.rect.size.width > 0.0);
    let has_v_marker = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 1.0 && f.rect.size.height > 0.0);
    assert!(has_h_marker, "应有水平间距标记线");
    assert!(has_v_marker, "应有垂直间距标记线");
}

#[test]
fn test_border_spacing_zero_no_render() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("white".into());
    style.border_spacing = BorderSpacingComputedValue {
        horizontal: 0.0,
        vertical: 0.0,
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 1, "零间距只应有背景 fill");
}

// ── caption-side 测试 ──────────────────────────────────────────────

#[test]
fn test_caption_side_bottom_renders() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("white".into());
    style.caption_side = CaptionSideValue::Bottom;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // bottom 应生成底部指示条（y > 50）
    let prims = painter.primitives();
    let has_bottom_bar = prims
        .fills
        .iter()
        .any(|f| f.rect.origin.y > 50.0 && f.rect.size.height == 3.0);
    assert!(has_bottom_bar, "caption-side:bottom 应在元素下方渲染指示条");
}

#[test]
fn test_caption_side_top_is_default_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Named("white".into());
    style.caption_side = CaptionSideValue::Top;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // Top 是默认值，不应额外渲染指示条
    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 1, "caption-side:top（默认值）不应渲染额外指示条");
}

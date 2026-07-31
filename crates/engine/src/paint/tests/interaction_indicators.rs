//! CSS cursor / image-rendering / isolation / will-change / pointer-events /
//! user-select / overscroll-behavior / touch-action 渲染指示器测试。

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;

use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::{
    ComputedStyle, CursorValue, ImageRenderingValue, IsolationValue, OverscrollBehaviorValue, PointerEventsValue,
    TouchActionValue, UserSelectValue, WillChangeValue,
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

// === cursor 指示器测试 ===

#[test]
fn test_cursor_auto_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default(); // cursor: Auto
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // cursor: Auto 不应生成指示器（4×4 方块在右上角附近）
    let prims = painter.primitives();
    let has_cursor_indicator = prims.fills.iter().any(|f| {
        f.rect.size.width == 4.0
            && f.rect.size.height == 4.0
            && (f.rect.origin.x - 94.0).abs() < 1.0
            && (f.rect.origin.y - 2.0).abs() < 1.0
    });
    assert!(!has_cursor_indicator, "cursor: Auto 不应生成指示器");
}

#[test]
fn test_cursor_pointer_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.cursor = CursorValue::Pointer;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // cursor: pointer 应在右上角生成蓝色 4×4 方块
    let has_indicator = prims.fills.iter().any(|f| {
        f.rect.size.width == 4.0 && f.rect.size.height == 4.0 && f.color.r == 0 && f.color.g == 120 && f.color.b == 215
    });
    assert!(has_indicator, "cursor: pointer 应生成蓝色指示器");
}

#[test]
fn test_cursor_crosshair_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.cursor = CursorValue::Crosshair;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_indicator = prims.fills.iter().any(|f| {
        f.rect.size.width == 4.0 && f.rect.size.height == 4.0 && f.color.r == 255 && f.color.g == 0 && f.color.b == 0
    });
    assert!(has_indicator, "cursor: crosshair 应生成红色指示器");
}

#[test]
fn test_cursor_not_allowed_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.cursor = CursorValue::NotAllowed;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_indicator = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 4.0 && f.rect.size.height == 4.0 && f.color.r == 200 && f.color.g == 0);
    assert!(has_indicator, "cursor: not-allowed 应生成深红指示器");
}

#[test]
fn test_cursor_none_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.cursor = CursorValue::None;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // cursor: none 应生成浅灰低透明度指示器
    let has_indicator = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 4.0 && f.rect.size.height == 4.0 && f.color.a == 100);
    assert!(has_indicator, "cursor: none 应生成浅灰指示器");
}

// === image-rendering 指示器测试 ===

#[test]
fn test_image_rendering_auto_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default(); // image-rendering: Auto
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // image-rendering: Auto 不应在右下角生成指示
    let has_indicator = prims
        .fills
        .iter()
        .any(|f| f.rect.origin.x >= 90.0 && f.rect.origin.y >= 40.0 && f.color.a == 180);
    assert!(!has_indicator, "image-rendering: Auto 不应生成指示器");
}

#[test]
fn test_image_rendering_pixelated_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.image_rendering = ImageRenderingValue::Pixelated;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // pixelated 应生成紫色 4×4 方格
    let has_indicator = prims.fills.iter().any(|f| {
        f.rect.size.width == 4.0 && f.rect.size.height == 4.0 && f.color.r == 255 && f.color.g == 0 && f.color.b == 255
    });
    assert!(has_indicator, "image-rendering: pixelated 应生成紫色方格指示器");
}

#[test]
fn test_image_rendering_crisp_edges_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.image_rendering = ImageRenderingValue::CrispEdges;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // crisp-edges 应生成橙色粗线边框
    let has_indicator = prims
        .fills
        .iter()
        .any(|f| f.color.r == 255 && f.color.g == 140 && f.color.b == 0 && f.color.a == 180);
    assert!(has_indicator, "image-rendering: crisp-edges 应生成橙色指示器");
}

#[test]
fn test_image_rendering_smooth_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.image_rendering = ImageRenderingValue::Smooth;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_indicator = prims
        .fills
        .iter()
        .any(|f| f.color.r == 0 && f.color.g == 200 && f.color.b == 100 && f.color.a == 180);
    assert!(has_indicator, "image-rendering: smooth 应生成绿色指示器");
}

// === isolation 指示器测试 ===

#[test]
fn test_isolation_auto_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default(); // isolation: Auto
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_l_shape = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 8.0 && f.rect.size.height == 2.0 && f.color.r == 128 && f.color.b == 128);
    assert!(!has_l_shape, "isolation: Auto 不应生成 L 形指示器");
}

#[test]
fn test_isolation_isolate_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.isolation = IsolationValue::Isolate;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // isolation: isolate 应生成紫色 L 形标记
    let has_horizontal = prims.fills.iter().any(|f| {
        f.rect.origin.x == 0.0
            && f.rect.origin.y == 0.0
            && f.rect.size.width == 8.0
            && f.rect.size.height == 2.0
            && f.color.r == 128
            && f.color.g == 0
            && f.color.b == 128
    });
    let has_vertical = prims.fills.iter().any(|f| {
        f.rect.origin.x == 0.0
            && f.rect.origin.y == 0.0
            && f.rect.size.width == 2.0
            && f.rect.size.height == 8.0
            && f.color.r == 128
            && f.color.g == 0
            && f.color.b == 128
    });
    assert!(has_horizontal, "isolation: isolate 应生成水平 L 形标记");
    assert!(has_vertical, "isolation: isolate 应生成垂直 L 形标记");
}

// === will-change 指示器测试 ===

#[test]
fn test_will_change_auto_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default(); // will-change: Auto
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_triangle = prims
        .fills
        .iter()
        .any(|f| f.color.r == 255 && f.color.g == 200 && f.color.b == 0);
    assert!(!has_triangle, "will-change: Auto 不应生成三角形指示器");
}

#[test]
fn test_will_change_scroll_position_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.will_change = vec![WillChangeValue::ScrollPosition];
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // will-change 应生成黄色三角形标记
    let has_triangle = prims
        .fills
        .iter()
        .any(|f| f.color.r == 255 && f.color.g == 200 && f.color.b == 0 && f.color.a == 200);
    assert!(has_triangle, "will-change: scroll-position 应生成黄色三角形指示器");
}

#[test]
fn test_will_change_custom_property_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.will_change = vec![WillChangeValue::Custom("transform".to_string())];
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_triangle = prims
        .fills
        .iter()
        .any(|f| f.color.r == 255 && f.color.g == 200 && f.color.b == 0 && f.color.a == 200);
    assert!(has_triangle, "will-change: transform 应生成黄色三角形指示器");
}

/// R2308：多 ident will-change（`will-change: transform opacity`）也应生成指示器。
#[test]
fn test_will_change_multiple_idents_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.will_change = vec![
        WillChangeValue::Custom("transform".to_string()),
        WillChangeValue::Custom("opacity".to_string()),
    ];
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_triangle = prims
        .fills
        .iter()
        .any(|f| f.color.r == 255 && f.color.g == 200 && f.color.b == 0 && f.color.a == 200);
    assert!(has_triangle, "多 ident will-change 应生成黄色三角形指示器");
}

// === pointer-events 指示器测试 ===

#[test]
fn test_pointer_events_auto_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default(); // pointer-events: Auto
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_x = prims
        .strokes
        .iter()
        .any(|s| s.color.r == 220 && s.color.g == 20 && s.color.b == 20);
    assert!(!has_x, "pointer-events: Auto 不应生成 × 标记");
}

#[test]
fn test_pointer_events_none_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.pointer_events = PointerEventsValue::None;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // pointer-events: none 应生成红色 × 标记（两条交叉线）
    let x_strokes: Vec<_> = prims
        .strokes
        .iter()
        .filter(|s| s.color.r == 220 && s.color.g == 20 && s.color.b == 20 && s.color.a == 180)
        .collect();
    assert!(
        x_strokes.len() >= 2,
        "pointer-events: none 应生成至少 2 条交叉线，实际 {} 条",
        x_strokes.len()
    );
}

// === user-select 指示器测试 ===

#[test]
fn test_user_select_auto_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default(); // user-select: Auto
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_lock = prims.fills.iter().any(|f| {
        f.rect.size.width == 6.0
            && f.rect.size.height == 4.0
            && f.color.r == 128
            && f.color.g == 128
            && f.color.b == 128
    });
    assert!(!has_lock, "user-select: Auto 不应生成锁形指示器");
}

#[test]
fn test_user_select_none_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.user_select = UserSelectValue::None;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // user-select: none 应生成灰色锁体（6×4 矩形）+ 半弧锁扣（strokes）
    let has_lock_body = prims.fills.iter().any(|f| {
        f.rect.size.width == 6.0
            && f.rect.size.height == 4.0
            && f.color.r == 128
            && f.color.g == 128
            && f.color.b == 128
            && f.color.a == 180
    });
    assert!(has_lock_body, "user-select: none 应生成锁体矩形");
    // 还应有弧形 strokes
    let has_arc_strokes = prims
        .strokes
        .iter()
        .any(|s| s.color.r == 128 && s.color.g == 128 && s.color.b == 128 && s.color.a == 180);
    assert!(has_arc_strokes, "user-select: none 应生成锁扣 strokes");
}

// === overscroll-behavior 指示器测试 ===

#[test]
fn test_overscroll_behavior_auto_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default(); // overscroll-behavior: Auto
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_line = prims.fills.iter().any(|f| {
        f.rect.size.height == 2.0 && (f.color.r == 255 && f.color.g == 100 || f.color.r == 200 && f.color.g == 0)
    });
    assert!(!has_line, "overscroll-behavior: Auto 不应生成水平线指示器");
}

#[test]
fn test_overscroll_behavior_contain_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.overscroll_behavior_x = OverscrollBehaviorValue::Contain;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // contain 应生成橙色水平线
    let has_line = prims.fills.iter().any(|f| {
        f.rect.size.height == 2.0 && f.rect.size.width == 12.0 && f.color.r == 255 && f.color.g == 100 && f.color.b == 0
    });
    assert!(has_line, "overscroll-behavior: contain 应生成橙色水平线");
}

#[test]
fn test_overscroll_behavior_none_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.overscroll_behavior_x = OverscrollBehaviorValue::None;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // none 应生成深红色水平线（更宽）
    let has_line = prims.fills.iter().any(|f| {
        f.rect.size.height == 2.0 && f.rect.size.width == 16.0 && f.color.r == 200 && f.color.g == 0 && f.color.b == 0
    });
    assert!(has_line, "overscroll-behavior: none 应生成深红色水平线");
}

// === touch-action 指示器测试 ===

#[test]
fn test_touch_action_auto_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default(); // touch-action: Auto
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_dot = prims.fills.iter().any(|f| {
        f.rect.size.width == 3.0 && f.rect.size.height == 3.0 && f.rect.origin.x >= 90.0 && f.rect.origin.y >= 40.0
    });
    assert!(!has_dot, "touch-action: Auto 不应生成指示器");
}

#[test]
fn test_touch_action_none_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.touch_action = TouchActionValue::None;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_indicator = prims.fills.iter().any(|f| {
        f.rect.size.width == 3.0
            && f.rect.size.height == 3.0
            && f.color.r == 200
            && f.color.g == 0
            && f.color.b == 0
            && f.color.a == 180
    });
    assert!(has_indicator, "touch-action: none 应生成红色小点指示器");
}

#[test]
fn test_touch_action_pan_x_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.touch_action = TouchActionValue::PanX;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_indicator = prims.fills.iter().any(|f| {
        f.rect.size.width == 3.0 && f.rect.size.height == 3.0 && f.color.r == 0 && f.color.g == 100 && f.color.b == 200
    });
    assert!(has_indicator, "touch-action: pan-x 应生成蓝色小点指示器");
}

// === 组合测试 ===

#[test]
fn test_multiple_indicators_together() {
    // 测试多个 CSS 属性指示器同时生效
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.cursor = CursorValue::Pointer;
    style.isolation = IsolationValue::Isolate;
    style.will_change = vec![WillChangeValue::Contents];
    style.pointer_events = PointerEventsValue::None;
    style.user_select = UserSelectValue::None;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // cursor indicator: 蓝色 4×4 方块
    assert!(
        prims.fills.iter().any(|f| f.color.g == 120 && f.color.b == 215),
        "应有 cursor 指示器"
    );
    // isolation indicator: 紫色 L 形
    assert!(
        prims
            .fills
            .iter()
            .any(|f| f.color.r == 128 && f.color.b == 128 && f.rect.size.width == 8.0),
        "应有 isolation 指示器"
    );
    // will-change indicator: 黄色三角形
    assert!(
        prims
            .fills
            .iter()
            .any(|f| f.color.r == 255 && f.color.g == 200 && f.color.b == 0),
        "应有 will-change 指示器"
    );
    // pointer-events indicator: 红色 ×
    assert!(
        prims.strokes.iter().any(|s| s.color.r == 220 && s.color.g == 20),
        "应有 pointer-events 指示器"
    );
    // user-select indicator: 灰色锁体
    assert!(
        prims
            .fills
            .iter()
            .any(|f| f.rect.size.width == 6.0 && f.rect.size.height == 4.0 && f.color.r == 128),
        "应有 user-select 指示器"
    );
}

#[test]
fn test_all_default_no_extra_primitives() {
    // 所有属性都是默认值，不应生成任何指示器
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default();
    styles.insert(elem, style);

    let mut painter = Painter::new();
    let prims_before = painter.primitives().fills.len();
    painter.paint(&layout, &styles, None);
    let prims = painter.primitives();

    // 默认样式的元素不应有任何可见图元（transparent 背景）
    assert!(
        prims.fills.is_empty() || prims.fills.len() == prims_before,
        "默认样式不应生成指示器图元"
    );
    assert!(prims.strokes.is_empty(), "默认样式不应生成 stroke 图元");
}

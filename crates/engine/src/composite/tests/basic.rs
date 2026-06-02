// Auto-generated test file — split from engine/composite.rs
use std::collections::HashMap;

use super::super::*;
use super::helpers::*;
use zero_css_parser::values::PositionValue;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::ComputedStyle;

#[test]
fn test_compositing_layer_root_only() {
    let layout = make_box(None, 0.0, 0.0, 800.0, 600.0, false);
    let styles = HashMap::new();
    let layers = promote_compositing_layers(&layout, &styles);

    assert_eq!(layers.len(), 1);
    assert!(layers[0].is_root);
}

/// 测试 opacity < 1.0 的元素被提升为独立图层。
#[test]
fn test_compositing_layer_opacity_promotion() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.opacity = 0.5;
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    // 应该有根图层 + 1 个提升图层
    assert_eq!(layers.len(), 2);
    assert!(layers[0].is_root);
    assert!(!layers[1].is_root);
    assert!((layers[1].opacity - 0.5).abs() < 0.001);
}

/// 测试 position: fixed 的元素被提升为独立图层。
#[test]
fn test_compositing_layer_fixed_position_promotion() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.position = PositionValue::Fixed;
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 2);
    assert!(layers[0].is_root);
    assert!(!layers[1].is_root);
}

/// 测试普通元素不会被提升。
#[test]
fn test_compositing_layer_no_promotion_normal() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    styles.insert(elem, ComputedStyle::default());

    let layers = promote_compositing_layers(&root_box, &styles);
    // 只有根图层
    assert_eq!(layers.len(), 1);
    assert!(layers[0].is_root);
}

/// 测试多个元素同时被提升。
#[test]
fn test_compositing_layer_multiple_promotions() {
    let mut doc = zero_dom::Document::new();
    let elem1 = doc.create_element("div");
    let elem2 = doc.create_element("div");
    let elem3 = doc.create_element("div");

    let child1 = make_box(Some(elem1), 0.0, 0.0, 100.0, 50.0, false);
    let child2 = make_box(Some(elem2), 0.0, 50.0, 100.0, 50.0, false);
    let child3 = make_box(Some(elem3), 0.0, 100.0, 100.0, 50.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();

    // elem1: opacity = 0.5（被提升）
    let mut style1 = ComputedStyle::default();
    style1.opacity = 0.5;
    styles.insert(elem1, style1);

    // elem2: position: fixed（被提升）
    let mut style2 = ComputedStyle::default();
    style2.position = PositionValue::Fixed;
    styles.insert(elem2, style2);

    // elem3: 普通（不提升）
    styles.insert(elem3, ComputedStyle::default());

    let layers = promote_compositing_layers(&root_box, &styles);
    // 根图层 + 2 个提升图层
    assert_eq!(layers.len(), 3);
    assert!(layers[0].is_root);
}

/// 测试 is_fixed=true 的 LayoutBox 被提升（无需 style 中 position=Fixed）。
#[test]
fn test_compositing_layer_layout_fixed_flag() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0, true);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    // Default style (no position: fixed, no opacity) — but is_fixed=true on box
    let mut styles = HashMap::new();
    styles.insert(elem, ComputedStyle::default());

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 2, "is_fixed=true should promote to own layer");
    assert!(layers[0].is_root);
    assert!(!layers[1].is_root);
}

/// 测试 opacity=1.0 的元素不被提升。
#[test]
fn test_compositing_layer_opacity_one_not_promoted() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.opacity = 1.0;
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    // Only root layer — opacity=1.0 should NOT promote
    assert_eq!(layers.len(), 1);
    assert!(layers[0].is_root);
}

/// 测试提升图层的 offset/width/height 正确。
#[test]
fn test_compositing_layer_promoted_geometry() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = make_box(Some(elem), 20.0, 30.0, 200.0, 150.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.opacity = 0.8;
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 2);
    let promoted = &layers[1];
    assert!((promoted.opacity - 0.8).abs() < 0.001);
    assert!((promoted.offset_x - 20.0).abs() < f32::EPSILON);
    assert!((promoted.offset_y - 30.0).abs() < f32::EPSILON);
    assert!((promoted.width - 200.0).abs() < f32::EPSILON);
    assert!((promoted.height - 150.0).abs() < f32::EPSILON);
}

/// 测试根图层包含未提升的子元素。
#[test]
fn test_compositing_layer_root_contains_unpromoted() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    styles.insert(elem, ComputedStyle::default());

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 1);
    // Root layer should contain root box + child box = 2
    assert_eq!(layers[0].boxes.len(), 2);
}

/// 测试 CompositingLayer 的 bounding_box 方法。
#[test]
fn test_compositing_layer_bounding_box() {
    let mut layer = CompositingLayer::new(0);
    layer.boxes.push(make_box(None, 10.0, 20.0, 100.0, 50.0, false));
    layer.boxes.push(make_box(None, 50.0, 30.0, 80.0, 60.0, false));

    let (x, y, w, h) = layer.bounding_box();
    assert_eq!(x, 10.0);
    assert_eq!(y, 20.0);
    assert_eq!(w, 120.0); // max right (130) - min left (10)
    assert_eq!(h, 70.0); // max bottom (90) - min top (20)
}

// ── 新增测试：z-index 堆叠排序 ──────────────────────────

/// 测试显式 z-index 的元素被提升为独立图层。
#[test]
fn test_z_index_promotion() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.z_index = ZIndexValue::Integer(10);
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    // 根图层 + 1 个 z-index 提升图层
    assert_eq!(layers.len(), 2);
    assert!(layers[0].is_root);
    assert!(!layers[1].is_root);
    assert_eq!(layers[1].z_index, 10);
}

/// 测试 z-index 排序：多个图层按 z_index 升序排列。
#[test]
fn test_z_index_sorting_order() {
    let mut doc = zero_dom::Document::new();
    let elem_low = doc.create_element("div");
    let elem_high = doc.create_element("div");
    let elem_mid = doc.create_element("div");

    let child_low = make_box(Some(elem_low), 0.0, 0.0, 100.0, 50.0, false);
    let child_mid = make_box(Some(elem_mid), 0.0, 50.0, 100.0, 50.0, false);
    let child_high = make_box(Some(elem_high), 0.0, 100.0, 100.0, 50.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        children: vec![child_low, child_mid, child_high],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();

    // elem_low: z-index = 10
    let mut style_low = ComputedStyle::default();
    style_low.z_index = ZIndexValue::Integer(10);
    styles.insert(elem_low, style_low);

    // elem_high: z-index = 100
    let mut style_high = ComputedStyle::default();
    style_high.z_index = ZIndexValue::Integer(100);
    styles.insert(elem_high, style_high);

    // elem_mid: z-index = 50
    let mut style_mid = ComputedStyle::default();
    style_mid.z_index = ZIndexValue::Integer(50);
    styles.insert(elem_mid, style_mid);

    let layers = promote_compositing_layers(&root_box, &styles);
    // 根图层 + 3 个 z-index 图层
    assert_eq!(layers.len(), 4);

    // 非根图层按 z_index 升序排列
    assert!(layers[0].is_root);
    assert_eq!(layers[1].z_index, 10); // 低
    assert_eq!(layers[2].z_index, 50); // 中
    assert_eq!(layers[3].z_index, 100); // 高
}

/// 测试负 z-index 元素排在正常流之前。
#[test]
fn test_negative_z_index_ordering() {
    let mut doc = zero_dom::Document::new();
    let elem_neg = doc.create_element("div");
    let elem_pos = doc.create_element("div");

    let child_neg = make_box(Some(elem_neg), 0.0, 0.0, 100.0, 50.0, false);
    let child_pos = make_box(Some(elem_pos), 0.0, 50.0, 100.0, 50.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        children: vec![child_neg, child_pos],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();

    // elem_pos: z-index = 1（正常流）
    let mut style_pos = ComputedStyle::default();
    style_pos.z_index = ZIndexValue::Integer(1);
    styles.insert(elem_pos, style_pos);

    // elem_neg: z-index = -1（在正常流之后绘制 = 在最底层）
    let mut style_neg = ComputedStyle::default();
    style_neg.z_index = ZIndexValue::Integer(-1);
    styles.insert(elem_neg, style_neg);

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 3);
    assert!(layers[0].is_root);
    // 负 z-index 应排在正 z-index 之前
    assert_eq!(layers[1].z_index, -1);
    assert_eq!(layers[2].z_index, 1);
}

/// 测试 z-index: auto 的元素不被提升。
#[test]
fn test_z_index_auto_not_promoted() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.z_index = ZIndexValue::Auto;
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    // z-index: auto 不应被提升
    assert_eq!(layers.len(), 1);
    assert!(layers[0].is_root);
}

/// 测试 CompositingLayer 的 z_index 默认值为 0。
#[test]
fn test_compositing_layer_default_z_index() {
    let layer = CompositingLayer::new(42);
    assert_eq!(layer.z_index, 0);
    assert_eq!(layer.id, 42);
}

// ── 新增测试：z-index compositing / stacking contexts ─────

/// 测试嵌套元素内层 z-index 不影响外层堆叠。
#[test]
fn test_nested_stacking_context_inner_z_index() {
    let mut doc = zero_dom::Document::new();
    let outer = doc.create_element("div");
    let inner = doc.create_element("span");

    let inner_box = make_box(Some(inner), 0.0, 0.0, 50.0, 20.0, false);
    let outer_box = LayoutBox {
        node_id: Some(outer),
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
        children: vec![inner_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    // outer has z-index = 5 -> promoted
    let mut outer_style = ComputedStyle::default();
    outer_style.z_index = ZIndexValue::Integer(5);
    styles.insert(outer, outer_style);

    // inner has z-index = 100 -> also promoted independently
    let mut inner_style = ComputedStyle::default();
    inner_style.z_index = ZIndexValue::Integer(100);
    styles.insert(inner, inner_style);

    let layers = promote_compositing_layers(&outer_box, &styles);
    // root + outer (z=5) + inner (z=100) = 3
    assert_eq!(layers.len(), 3);
    assert_eq!(layers[1].z_index, 5);
    assert_eq!(layers[2].z_index, 100);
}

/// 测试默认 z-index(auto) 的元素和 z-index(0) 的元素排序。
#[test]
fn test_z_index_auto_vs_zero() {
    let mut doc = zero_dom::Document::new();
    let elem_auto = doc.create_element("div");
    let elem_zero = doc.create_element("div");

    let child_auto = make_box(Some(elem_auto), 0.0, 0.0, 100.0, 50.0, false);
    let child_zero = make_box(Some(elem_zero), 0.0, 50.0, 100.0, 50.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        children: vec![child_auto, child_zero],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    // auto -> not promoted
    let mut style_auto = ComputedStyle::default();
    style_auto.z_index = ZIndexValue::Auto;
    styles.insert(elem_auto, style_auto);

    // z-index: 0 -> promoted (explicit, non-auto)
    let mut style_zero = ComputedStyle::default();
    style_zero.z_index = ZIndexValue::Integer(0);
    styles.insert(elem_zero, style_zero);

    let layers = promote_compositing_layers(&root_box, &styles);
    // root + 1 promoted (z-index: 0)
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[1].z_index, 0);
}

/// 测试多个负 z-index 图层排序正确。
#[test]
fn test_multiple_negative_z_index_sorting() {
    let mut doc = zero_dom::Document::new();
    let e1 = doc.create_element("div");
    let e2 = doc.create_element("div");
    let e3 = doc.create_element("div");

    let c1 = make_box(Some(e1), 0.0, 0.0, 50.0, 50.0, false);
    let c2 = make_box(Some(e2), 0.0, 0.0, 50.0, 50.0, false);
    let c3 = make_box(Some(e3), 0.0, 0.0, 50.0, 50.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        children: vec![c1, c2, c3],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut s1 = ComputedStyle::default();
    s1.z_index = ZIndexValue::Integer(-5);
    styles.insert(e1, s1);

    let mut s2 = ComputedStyle::default();
    s2.z_index = ZIndexValue::Integer(-1);
    styles.insert(e2, s2);

    let mut s3 = ComputedStyle::default();
    s3.z_index = ZIndexValue::Integer(-10);
    styles.insert(e3, s3);

    let layers = promote_compositing_layers(&root_box, &styles);
    // root + 3 promoted
    assert_eq!(layers.len(), 4);
    // Sorted ascending: -10, -5, -1
    assert_eq!(layers[1].z_index, -10);
    assert_eq!(layers[2].z_index, -5);
    assert_eq!(layers[3].z_index, -1);
}

/// 测试 opacity + z-index 同时存在的元素只创建一个图层。
#[test]
fn test_opacity_and_z_index_single_layer() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.opacity = 0.5;
    style.z_index = ZIndexValue::Integer(10);
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    // root + 1 promoted (not 2: both conditions on same element)
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[1].z_index, 10);
    assert!((layers[1].opacity - 0.5).abs() < 0.001);
}

/// 测试 CompositingLayer bounding_box 空列表返回极限值。
#[test]
fn test_compositing_layer_bounding_box_empty() {
    let layer = CompositingLayer::new(0);
    let (x, y, _w, _h) = layer.bounding_box();
    assert_eq!(x, f32::MAX);
    assert_eq!(y, f32::MAX);
}

// ── 新增测试：z-index compositing order ──────────────────────

/// 测试高 z-index 图层在低 z-index 图层之后（绘制顺序靠后 = 视觉在上层）。
///
/// 构建两个重叠元素：low(z=1) 和 high(z=10)。
/// 合成排序后 layers[1] 应为 z=1，layers[2] 应为 z=10。
/// 实际渲染时按 layers 顺序绘制，高 z-index 的后绘制 = 覆盖低 z-index。
#[test]
fn test_z_index_higher_renders_on_top() {
    let mut doc = zero_dom::Document::new();
    let elem_low = doc.create_element("div");
    let elem_high = doc.create_element("div");

    let child_low = make_box(Some(elem_low), 0.0, 0.0, 100.0, 100.0, false);
    let child_high = make_box(Some(elem_high), 20.0, 20.0, 100.0, 100.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        children: vec![child_low, child_high],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut style_low = ComputedStyle::default();
    style_low.z_index = ZIndexValue::Integer(1);
    styles.insert(elem_low, style_low);

    let mut style_high = ComputedStyle::default();
    style_high.z_index = ZIndexValue::Integer(10);
    styles.insert(elem_high, style_high);

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 3, "root + 2 promoted layers");
    assert!(layers[0].is_root);

    // layers[1] z-index=1 先绘制（底层），layers[2] z-index=10 后绘制（上层）
    assert_eq!(layers[1].z_index, 1);
    assert_eq!(layers[2].z_index, 10);
    assert!(
        layers[2].z_index > layers[1].z_index,
        "higher z-index layer should render after (on top of) lower z-index layer"
    );
}

/// 测试多个 z-index 值（负/零/正）的完整合成排序。
///
/// 最终顺序：z=-5 → z=0 → z=3 → z=100，保证高 z-index 覆盖低 z-index。
#[test]
fn test_z_index_compositing_full_sorting_order() {
    let mut doc = zero_dom::Document::new();
    let e_neg = doc.create_element("div");
    let e_zero = doc.create_element("div");
    let e_pos = doc.create_element("div");
    let e_high = doc.create_element("div");

    let c_neg = make_box(Some(e_neg), 0.0, 0.0, 50.0, 50.0, false);
    let c_zero = make_box(Some(e_zero), 0.0, 0.0, 50.0, 50.0, false);
    let c_pos = make_box(Some(e_pos), 0.0, 0.0, 50.0, 50.0, false);
    let c_high = make_box(Some(e_high), 0.0, 0.0, 50.0, 50.0, false);

    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        children: vec![c_neg, c_zero, c_pos, c_high],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut s_neg = ComputedStyle::default();
    s_neg.z_index = ZIndexValue::Integer(-5);
    styles.insert(e_neg, s_neg);

    let mut s_zero = ComputedStyle::default();
    s_zero.z_index = ZIndexValue::Integer(0);
    styles.insert(e_zero, s_zero);

    let mut s_pos = ComputedStyle::default();
    s_pos.z_index = ZIndexValue::Integer(3);
    styles.insert(e_pos, s_pos);

    let mut s_high = ComputedStyle::default();
    s_high.z_index = ZIndexValue::Integer(100);
    styles.insert(e_high, s_high);

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 5, "root + 4 promoted layers");

    // 严格验证升序排列
    assert!(layers[0].is_root);
    assert_eq!(layers[1].z_index, -5);
    assert_eq!(layers[2].z_index, 0);
    assert_eq!(layers[3].z_index, 3);
    assert_eq!(layers[4].z_index, 100);

    // 验证单调递增
    for i in 1..layers.len() - 1 {
        assert!(
            layers[i].z_index <= layers[i + 1].z_index,
            "layers should be sorted by ascending z-index"
        );
    }
}

// ── 新增测试：重叠元素堆叠顺序与合成原因 ──────────────────────

/// 测试多个重叠元素按 z-index 升序绘制，保证高 z-index 覆盖低 z-index。
///
/// 构建五个重叠元素：z-index 分别为 -3, 0, 5, 10, 100。
/// 验证合成排序后 layers[1..] 严格按 z_index 升序排列。
#[test]
fn test_overlapping_elements_stacking_order() {
    let mut doc = zero_dom::Document::new();
    let e1 = doc.create_element("div");
    let e2 = doc.create_element("div");
    let e3 = doc.create_element("div");
    let e4 = doc.create_element("div");
    let e5 = doc.create_element("div");

    // 所有元素重叠在同一位置
    let c1 = make_box(Some(e1), 0.0, 0.0, 200.0, 200.0, false);
    let c2 = make_box(Some(e2), 10.0, 10.0, 200.0, 200.0, false);
    let c3 = make_box(Some(e3), 20.0, 20.0, 200.0, 200.0, false);
    let c4 = make_box(Some(e4), 30.0, 30.0, 200.0, 200.0, false);
    let c5 = make_box(Some(e5), 40.0, 40.0, 200.0, 200.0, false);
    let root_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
        children: vec![c1, c2, c3, c4, c5],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();

    let mut s1 = ComputedStyle::default();
    s1.z_index = ZIndexValue::Integer(10);
    styles.insert(e1, s1);

    let mut s2 = ComputedStyle::default();
    s2.z_index = ZIndexValue::Integer(-3);
    styles.insert(e2, s2);

    let mut s3 = ComputedStyle::default();
    s3.z_index = ZIndexValue::Integer(100);
    styles.insert(e3, s3);

    let mut s4 = ComputedStyle::default();
    s4.z_index = ZIndexValue::Integer(0);
    styles.insert(e4, s4);

    let mut s5 = ComputedStyle::default();
    s5.z_index = ZIndexValue::Integer(5);
    styles.insert(e5, s5);

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 6, "root + 5 promoted layers");

    // 验证严格升序
    assert!(layers[0].is_root);
    assert_eq!(layers[1].z_index, -3);
    assert_eq!(layers[2].z_index, 0);
    assert_eq!(layers[3].z_index, 5);
    assert_eq!(layers[4].z_index, 10);
    assert_eq!(layers[5].z_index, 100);
}

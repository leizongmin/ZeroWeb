// Auto-generated test file — split from engine/composite.rs
use std::collections::HashMap;

use super::super::*;
use super::helpers::*;
use zero_css_parser::values::PositionValue;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::ComputedStyle;

#[test]
fn test_compositing_reason_opacity_only() {
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
    style.opacity = 0.3;
    // z_index 保持 Auto，position 保持默认
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 2, "opacity < 1.0 should promote to own layer");
    assert!(layers[0].is_root);
    assert!(!layers[1].is_root);
    assert!((layers[1].opacity - 0.3).abs() < 0.001, "layer opacity should be 0.3");
    // z_index 应为 0（auto）
    assert_eq!(layers[1].z_index, 0, "z_index should be 0 when not explicitly set");
}

/// 测试显式 z-index 是提升合成层的原因。
///
/// 元素仅有 z-index: 42（opacity=1.0, 非 fixed），
/// 验证被提升且 z_index 正确记录。
#[test]
fn test_compositing_reason_z_index_only() {
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
    style.z_index = ZIndexValue::Integer(42);
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 2, "explicit z-index should promote to own layer");
    assert_eq!(layers[1].z_index, 42, "layer z_index should be 42");
    assert!((layers[1].opacity - 1.0).abs() < 0.001, "opacity should remain 1.0");
}

/// 测试负 z-index 元素被提升为独立图层且排在正 z-index 之前。
///
/// 构建三个元素：z=-10, z=0, z=5。
/// 验证负 z-index 图层在根图层之后、正 z-index 图层之前。
#[test]
fn test_negative_z_index_layer_assignment() {
    let mut doc = zero_dom::Document::new();
    let e_neg = doc.create_element("div");
    let e_zero = doc.create_element("div");
    let e_pos = doc.create_element("div");

    let c_neg = make_box(Some(e_neg), 0.0, 0.0, 50.0, 50.0, false);
    let c_zero = make_box(Some(e_zero), 0.0, 0.0, 50.0, 50.0, false);
    let c_pos = make_box(Some(e_pos), 0.0, 0.0, 50.0, 50.0, false);
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
        children: vec![c_neg, c_zero, c_pos],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut s_neg = ComputedStyle::default();
    s_neg.z_index = ZIndexValue::Integer(-10);
    styles.insert(e_neg, s_neg);

    let mut s_zero = ComputedStyle::default();
    s_zero.z_index = ZIndexValue::Integer(0);
    styles.insert(e_zero, s_zero);

    let mut s_pos = ComputedStyle::default();
    s_pos.z_index = ZIndexValue::Integer(5);
    styles.insert(e_pos, s_pos);

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 4, "root + 3 promoted layers");

    // 负 z-index 在最前面（除根图层外）
    assert!(layers[0].is_root);
    assert_eq!(layers[1].z_index, -10, "negative z-index layer should come first");
    assert_eq!(layers[2].z_index, 0);
    assert_eq!(layers[3].z_index, 5);
}

/// 测试 z-index 排序稳定性：相同 z-index 的元素保持原始顺序。
///
/// 三个元素 z-index 都为 10，验证合成后仍为 3 个独立图层。
#[test]
fn test_layer_priority_sorting_stability() {
    let mut doc = zero_dom::Document::new();
    let e1 = doc.create_element("div");
    let e2 = doc.create_element("div");
    let e3 = doc.create_element("div");

    let c1 = make_box(Some(e1), 0.0, 0.0, 50.0, 50.0, false);
    let c2 = make_box(Some(e2), 0.0, 50.0, 50.0, 50.0, false);
    let c3 = make_box(Some(e3), 0.0, 100.0, 50.0, 50.0, false);
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
    // 三个元素都设置 z-index: 10（相同值）
    for id in [e1, e2, e3] {
        let mut s = ComputedStyle::default();
        s.z_index = ZIndexValue::Integer(10);
        styles.insert(id, s);
    }

    let layers = promote_compositing_layers(&root_box, &styles);
    // root + 3 promoted layers（相同 z-index 不应合并）
    assert_eq!(layers.len(), 4, "each element with z-index should be a separate layer");
    assert!(layers[0].is_root);

    // 所有提升的图层 z-index 应相同
    assert_eq!(layers[1].z_index, 10);
    assert_eq!(layers[2].z_index, 10);
    assert_eq!(layers[3].z_index, 10);
}

/// 测试 opacity + position:fixed + z-index 三重原因只创建一个图层。
///
/// 一个元素同时满足三个提升条件，应只创建一个合成层。
#[test]
fn test_triple_compositing_reason_single_layer() {
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
    style.position = PositionValue::Fixed;
    style.z_index = ZIndexValue::Integer(20);
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    // root + 1 promoted (not 3: all conditions on same element)
    assert_eq!(layers.len(), 2);
    assert!((layers[1].opacity - 0.5).abs() < 0.001);
    assert_eq!(layers[1].z_index, 20);
}

/// 测试根图层包含多个未提升子元素时包围盒计算正确。
#[test]
fn test_root_layer_bounding_box_with_unpromoted_children() {
    let mut doc = zero_dom::Document::new();
    let e1 = doc.create_element("div");
    let e2 = doc.create_element("div");

    // 两个普通子元素（不被提升）
    let c1 = make_box(Some(e1), 0.0, 0.0, 300.0, 200.0, false);
    let c2 = make_box(Some(e2), 100.0, 50.0, 400.0, 300.0, false);
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
        children: vec![c1, c2],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    styles.insert(e1, ComputedStyle::default());
    styles.insert(e2, ComputedStyle::default());

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 1, "only root layer");
    // root layer width 应取最远右边界 max(800, 300, 500) = 800
    assert_eq!(layers[0].width, 800.0);
    // root layer height 应取最远底边界 max(600, 200, 350) = 600
    assert_eq!(layers[0].height, 600.0);
}

// ── 边界条件测试 ──────────────────────────────────────────

/// 测试 opacity = 0.0 被提升且 layer.opacity 为 0.0。
#[test]
fn test_opacity_zero_promoted() {
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
    style.opacity = 0.0;
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    // 根图层 + 1 个提升图层
    assert_eq!(layers.len(), 2);
    assert!(layers[0].is_root);
    assert!(!layers[1].is_root);
    assert!(
        (layers[1].opacity - 0.0).abs() < f32::EPSILON,
        "promoted layer opacity should be 0.0"
    );
}

/// 测试 z_index = i32::MAX 排序正确。
#[test]
fn test_z_index_max_sorting() {
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
    s1.z_index = ZIndexValue::Integer(0);
    styles.insert(e1, s1);

    let mut s2 = ComputedStyle::default();
    s2.z_index = ZIndexValue::Integer(i32::MAX);
    styles.insert(e2, s2);

    let mut s3 = ComputedStyle::default();
    s3.z_index = ZIndexValue::Integer(100);
    styles.insert(e3, s3);

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 4, "root + 3 promoted layers");

    // 验证升序排列：0 < 100 < i32::MAX
    assert!(layers[0].is_root);
    assert_eq!(layers[1].z_index, 0);
    assert_eq!(layers[2].z_index, 100);
    assert_eq!(layers[3].z_index, i32::MAX);
}

/// 测试 z_index = i32::MIN 排序正确。
#[test]
fn test_z_index_min_sorting() {
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
    s1.z_index = ZIndexValue::Integer(i32::MIN);
    styles.insert(e1, s1);

    let mut s2 = ComputedStyle::default();
    s2.z_index = ZIndexValue::Integer(-5);
    styles.insert(e2, s2);

    let mut s3 = ComputedStyle::default();
    s3.z_index = ZIndexValue::Integer(0);
    styles.insert(e3, s3);

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 4, "root + 3 promoted layers");

    // 验证升序排列：i32::MIN < -5 < 0
    assert!(layers[0].is_root);
    assert_eq!(layers[1].z_index, i32::MIN);
    assert_eq!(layers[2].z_index, -5);
    assert_eq!(layers[3].z_index, 0);
}

/// 测试所有子元素都被提升时根层尺寸。
#[test]
fn test_root_layer_size_when_all_promoted() {
    let mut doc = zero_dom::Document::new();
    let e1 = doc.create_element("div");
    let e2 = doc.create_element("div");

    // 两个子元素都有 opacity < 1.0，都会被提升
    let c1 = make_box(Some(e1), 0.0, 0.0, 200.0, 100.0, false);
    let c2 = make_box(Some(e2), 50.0, 50.0, 300.0, 200.0, false);
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
        children: vec![c1, c2],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();

    // 两个子元素都有 opacity < 1.0
    let mut s1 = ComputedStyle::default();
    s1.opacity = 0.5;
    styles.insert(e1, s1);

    let mut s2 = ComputedStyle::default();
    s2.opacity = 0.8;
    styles.insert(e2, s2);

    let layers = promote_compositing_layers(&root_box, &styles);
    // 根图层 + 2 个提升图层
    assert_eq!(layers.len(), 3);

    // 根图层仍然存在
    assert!(layers[0].is_root);
    // 根图层只包含根 box 自身（两个子元素都被提升了）
    assert_eq!(layers[0].boxes.len(), 1, "root layer should only contain root box");
}

// ── 边界条件测试：父子提升关系 / 包围盒 / 根图层尺寸 ───────────

/// 测试父元素被提升（有 z-index），子元素未提升 → 子元素进入根图层而非父元素的提升图层。
///
/// 构建树：root → parent(z-index=5, promoted) → child(无 z-index, not promoted)。
/// 验证子元素出现在根图层中，不在父元素的提升图层中。
#[test]
fn test_promoted_parent_with_non_promoted_child() {
    let mut doc = zero_dom::Document::new();
    let parent_elem = doc.create_element("div");
    let child_elem = doc.create_element("span");

    // 子元素不提升
    let child_box = make_box(Some(child_elem), 0.0, 0.0, 50.0, 20.0, false);
    // 父元素有 z-index → 会被提升
    let parent_box = LayoutBox {
        node_id: Some(parent_elem),
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    // 父元素设置 z-index → 被提升
    let mut parent_style = ComputedStyle::default();
    parent_style.z_index = ZIndexValue::Integer(5);
    styles.insert(parent_elem, parent_style);

    // 子元素默认样式 → 不被提升
    styles.insert(child_elem, ComputedStyle::default());

    let layers = promote_compositing_layers(&parent_box, &styles);
    // 根图层 + 1 个父元素提升图层 = 2
    assert_eq!(layers.len(), 2, "root + 1 promoted parent layer");

    // 父元素的提升图层只包含父元素自身
    let promoted = &layers[1];
    assert_eq!(promoted.boxes.len(), 1, "promoted layer should contain parent box only");
    assert_eq!(promoted.z_index, 5);

    // 子元素应该在根图层中（未被提升的元素进入根图层）
    let root = &layers[0];
    assert!(root.is_root);
    // 根图层包含根 box 自身 + child box（因为 child 未被提升）
    assert!(
        root.boxes.len() >= 1,
        "root layer should contain the non-promoted child"
    );
}

/// 测试父元素和子元素都被提升（都有 z-index）→ 产生两个独立的提升图层。
///
/// 构建树：root → parent(z-index=5) → child(z-index=10)。
/// 验证有两个独立的提升图层，分别为 z=5 和 z=10。
#[test]
fn test_promoted_parent_with_promoted_child() {
    let mut doc = zero_dom::Document::new();
    let parent_elem = doc.create_element("div");
    let child_elem = doc.create_element("span");

    let child_box = make_box(Some(child_elem), 0.0, 0.0, 50.0, 20.0, false);
    let parent_box = LayoutBox {
        node_id: Some(parent_elem),
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut parent_style = ComputedStyle::default();
    parent_style.z_index = ZIndexValue::Integer(5);
    styles.insert(parent_elem, parent_style);

    let mut child_style = ComputedStyle::default();
    child_style.z_index = ZIndexValue::Integer(10);
    styles.insert(child_elem, child_style);

    let layers = promote_compositing_layers(&parent_box, &styles);
    // 根图层 + parent(z=5) + child(z=10) = 3
    assert_eq!(layers.len(), 3, "root + 2 promoted layers");

    // 验证两个提升图层
    assert!(layers[0].is_root);
    assert_eq!(layers[1].z_index, 5, "first promoted layer should be z=5");
    assert_eq!(layers[2].z_index, 10, "second promoted layer should be z=10");
}

/// 测试 CompositingLayer 只包含一个 box 时 bounding_box 返回该 box 的尺寸。
#[test]
fn test_bounding_box_single_box() {
    let mut layer = CompositingLayer::new(0);
    layer.boxes.push(make_box(None, 15.0, 25.0, 80.0, 60.0, false));

    let (x, y, w, h) = layer.bounding_box();
    assert!((x - 15.0).abs() < f32::EPSILON, "x should be 15.0");
    assert!((y - 25.0).abs() < f32::EPSILON, "y should be 25.0");
    assert!((w - 80.0).abs() < f32::EPSILON, "width should be 80.0");
    assert!((h - 60.0).abs() < f32::EPSILON, "height should be 60.0");
}

/// 测试 10+ 层具有各种 z-index 的合成排序正确。
///
/// 构建 12 个元素，分别设置不同的 z-index（包括负数、零、正数、
/// 大值、相同值），验证合成后根图层 + 12 个提升图层按 z-index 升序排列。
#[test]
fn test_composite_many_layers() {
    let mut doc = zero_dom::Document::new();
    let num_elements = 12;
    let mut elements = Vec::with_capacity(num_elements);
    let mut child_boxes = Vec::with_capacity(num_elements);

    for i in 0..num_elements {
        let elem = doc.create_element("div");
        elements.push(elem);
        child_boxes.push(make_box(Some(elem), (i as f32) * 10.0, 0.0, 50.0, 50.0, false));
    }

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
        children: child_boxes,
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    // 12 个 z-index 值：负数、零、正数、大值、重复值
    let z_indices: &[i32] = &[-100, -5, -1, 0, 0, 1, 3, 10, 10, 42, 999, i32::MAX];
    assert_eq!(z_indices.len(), num_elements);

    let mut styles = HashMap::new();
    for (i, &z) in z_indices.iter().enumerate() {
        let mut style = ComputedStyle::default();
        style.z_index = ZIndexValue::Integer(z);
        styles.insert(elements[i], style);
    }

    let layers = promote_compositing_layers(&root_box, &styles);

    // root + 12 promoted layers
    assert_eq!(layers.len(), 13, "root + 12 promoted layers");
    assert!(layers[0].is_root);

    // 非 root 图层应按 z-index 升序排列
    let non_root_z: Vec<i32> = layers[1..].iter().map(|l| l.z_index).collect();
    let expected_sorted: Vec<i32> = {
        let mut v = z_indices.to_vec();
        v.sort();
        v
    };
    assert_eq!(
        non_root_z, expected_sorted,
        "promoted layers should be sorted by ascending z-index"
    );

    // 验证单调递增
    for i in 1..layers.len() - 1 {
        assert!(
            layers[i].z_index <= layers[i + 1].z_index,
            "layers[{}].z_index ({}) should be <= layers[{}].z_index ({})",
            i,
            layers[i].z_index,
            i + 1,
            layers[i + 1].z_index
        );
    }
}

/// 测试根图层尺寸随深层孙子元素扩展。
///
/// 构建树：root(200x100) → child(0,0,100,80) → grandchild(0,0,400,300)。
/// 孙子元素超出根的尺寸，根图层的 width/height 应扩展。
#[test]
fn test_root_layer_encompasses_grandchildren() {
    let mut doc = zero_dom::Document::new();
    let child_elem = doc.create_element("div");
    let grandchild_elem = doc.create_element("span");

    // 孙子元素比根元素大
    let grandchild_box = make_box(Some(grandchild_elem), 0.0, 0.0, 400.0, 300.0, false);
    let child_box = LayoutBox {
        node_id: Some(child_elem),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 80.0,
        content_x: 0.0,
        content_y: 0.0,
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
        children: vec![grandchild_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };
    let root_box = LayoutBox {
        node_id: None,
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
        overflow_x: OverflowClip::Visible,
        z_index: 0,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    styles.insert(child_elem, ComputedStyle::default());
    styles.insert(grandchild_elem, ComputedStyle::default());

    let layers = promote_compositing_layers(&root_box, &styles);
    assert_eq!(layers.len(), 1, "only root layer (no promotions)");

    // 根图层尺寸应至少扩展到孙子元素的范围
    let root = &layers[0];
    assert!(
        root.width >= 400.0,
        "root layer width should encompass grandchild (>=400), got {}",
        root.width
    );
    assert!(
        root.height >= 300.0,
        "root layer height should encompass grandchild (>=300), got {}",
        root.height
    );
}

// ── 边界条件测试：CompositingLayer 默认字段验证 / position:absolute + z-index ──

/// 测试 CompositingLayer::new() 所有字段均为默认值。
#[test]
fn test_compositing_layer_new_default_fields() {
    let layer = CompositingLayer::new(7);
    assert_eq!(layer.id, 7);
    assert!(layer.boxes.is_empty());
    assert_eq!(layer.offset_x, 0.0);
    assert_eq!(layer.offset_y, 0.0);
    assert_eq!(layer.width, 0.0);
    assert_eq!(layer.height, 0.0);
    assert!((layer.opacity - 1.0).abs() < f32::EPSILON, "默认 opacity 应为 1.0");
    assert!(!layer.is_root, "默认 is_root 应为 false");
    assert_eq!(layer.z_index, 0);
}

/// 测试 position: absolute 的元素配合 z-index 被提升为独立图层。
#[test]
fn test_compositing_layer_position_absolute_with_z_index() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0, false);
    // 标记为 absolute
    let mut abs_child = child_box;
    abs_child.is_absolute = true;
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
        children: vec![abs_child],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.position = PositionValue::Absolute;
    style.z_index = ZIndexValue::Integer(5);
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    // 根图层 + 1 个 z-index 提升图层（position:absolute 本身不提升，但 z-index 非 Auto 会提升）
    assert_eq!(layers.len(), 2, "z-index:5 应提升为独立图层");
    assert!(layers[0].is_root);
    assert_eq!(layers[1].z_index, 5);
}

/// 测试 position:absolute 但 z-index:auto 时不被提升。
#[test]
fn test_compositing_layer_absolute_auto_z_index_not_promoted() {
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
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.position = PositionValue::Absolute;
    style.z_index = ZIndexValue::Auto;
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    // position:absolute + z-index:auto -> opacity=1.0, 非 fixed -> 不被提升
    assert_eq!(layers.len(), 1, "absolute + z-index:auto 不应提升");
    assert!(layers[0].is_root);
}

/// 测试 CompositingLayer 手动修改字段后 bounding_box 正确计算。
#[test]
fn test_compositing_layer_manual_modification_bounding_box() {
    let mut layer = CompositingLayer::new(0);
    layer.boxes.push(make_box(None, 0.0, 0.0, 100.0, 100.0, false));
    layer.boxes.push(make_box(None, 200.0, 200.0, 50.0, 50.0, false));
    layer.offset_x = 10.0;
    layer.offset_y = 20.0;
    layer.opacity = 0.7;
    layer.is_root = true;

    assert!((layer.opacity - 0.7).abs() < f32::EPSILON);
    assert!(layer.is_root);
    let (x, y, w, h) = layer.bounding_box();
    assert_eq!(x, 0.0);
    assert_eq!(y, 0.0);
    assert_eq!(w, 250.0); // max right (250) - min left (0)
    assert_eq!(h, 250.0); // max bottom (250) - min top (0)
}

/// 测试 CompositingLayer::new() 的 Debug 和 Clone 实现。
#[test]
fn test_compositing_layer_debug_clone() {
    let mut layer = CompositingLayer::new(42);
    layer.boxes.push(make_box(None, 10.0, 20.0, 50.0, 30.0, false));
    layer.z_index = 3;

    let cloned = layer.clone();
    assert_eq!(cloned.id, 42);
    assert_eq!(cloned.z_index, 3);
    assert_eq!(cloned.boxes.len(), 1);

    let debug_str = format!("{:?}", layer);
    assert!(debug_str.contains("42"));
}

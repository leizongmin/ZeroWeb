#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use std::collections::HashMap;

use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::ComputedStyle;

use crate::composite::promote_compositing_layers;
/// 测试单个 div 元素经样式和布局计算后，合成层至少返回一个图层。
#[test]
fn test_composite_single_box() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = LayoutBox {
        node_id: Some(elem),
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
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    styles.insert(elem, ComputedStyle::default());

    let layers = promote_compositing_layers(&child_box, &styles);
    assert!(!layers.is_empty(), "composite should return at least one layer");
    assert!(layers[0].is_root, "first layer should be root");
}
/// 创建两个重叠元素，分别设置 z-index=1 和 z-index=10，
/// 验证合成后非根图层按 z-index 升序排列，
/// 高 z-index 图层排在低 z-index 之后（后绘制 = 视觉在上层）。
#[test]
fn test_composite_z_index_ordering() {
    use crate::composite::promote_compositing_layers;
    use zero_layout_engine::types::OverflowClip;
    use zero_style_system::property::ZIndexValue;

    let mut doc = zero_dom::Document::new();
    let elem_low = doc.create_element("div");
    let elem_high = doc.create_element("div");

    let child_low = LayoutBox {
        node_id: Some(elem_low),
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
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };
    let child_high = LayoutBox {
        node_id: Some(elem_high),
        x: 50.0,
        y: 50.0,
        width: 100.0,
        height: 100.0,
        content_x: 50.0,
        content_y: 50.0,
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
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };
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
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = std::collections::HashMap::new();
    let mut style_low = ComputedStyle::default();
    style_low.z_index = ZIndexValue::Integer(1);
    styles.insert(elem_low, style_low);

    let mut style_high = ComputedStyle::default();
    style_high.z_index = ZIndexValue::Integer(10);
    styles.insert(elem_high, style_high);

    let layers = promote_compositing_layers(&root_box, &styles);

    // 根图层 + 2 个提升图层
    assert_eq!(layers.len(), 3, "root + 2 promoted layers");
    assert!(layers[0].is_root);

    // z-index 升序：1 在前（先绘制/底层），10 在后（后绘制/上层）
    assert_eq!(layers[1].z_index, 1, "first promoted layer should be z=1");
    assert_eq!(layers[2].z_index, 10, "second promoted layer should be z=10");
    assert!(
        layers[2].z_index > layers[1].z_index,
        "higher z-index should render after (on top of) lower z-index"
    );
}
/// 测试合成层提升子元素并验证图层 z-index 排序正确。
#[test]
fn test_composite_promoted_child_z_ordering() {
    use zero_style_system::property::ZIndexValue;

    let mut doc = zero_dom::Document::new();
    let elem_a = doc.create_element("div");
    let elem_b = doc.create_element("div");
    let elem_c = doc.create_element("div");

    let child_a = LayoutBox {
        node_id: Some(elem_a),
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
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };
    let child_b = LayoutBox {
        node_id: Some(elem_b),
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
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };
    let child_c = LayoutBox {
        node_id: Some(elem_c),
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
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };
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
        children: vec![child_a, child_b, child_c],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = std::collections::HashMap::new();
    let mut sa = ComputedStyle::default();
    sa.z_index = ZIndexValue::Integer(5);
    styles.insert(elem_a, sa);

    let mut sb = ComputedStyle::default();
    sb.z_index = ZIndexValue::Integer(-1);
    styles.insert(elem_b, sb);

    let mut sc = ComputedStyle::default();
    sc.z_index = ZIndexValue::Integer(10);
    styles.insert(elem_c, sc);

    let layers = promote_compositing_layers(&root_box, &styles);
    // 根图层 + 3 个提升图层
    assert_eq!(layers.len(), 4, "root + 3 promoted layers");
    assert!(layers[0].is_root);
    // 提升的图层按 z-index 升序：-1, 5, 10
    assert_eq!(layers[1].z_index, -1);
    assert_eq!(layers[2].z_index, 5);
    assert_eq!(layers[3].z_index, 10);
}
/// 验证 promote_compositing_layers 返回值中 layers[0] 始终为根图层。
#[test]
fn test_composite_root_layer_always_present() {
    use crate::composite::promote_compositing_layers;
    use zero_style_system::property::ZIndexValue;

    // 场景 1：空布局（仅根）
    let empty_root = LayoutBox {
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
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };
    let layers = promote_compositing_layers(&empty_root, &HashMap::new());
    assert!(!layers.is_empty(), "应至少有根图层");
    assert!(layers[0].is_root, "第一个图层应为根图层");
    assert_eq!(layers[0].id, 0, "根图层 id 应为 0");

    // 场景 2：有提升子元素
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let child_box = LayoutBox {
        node_id: Some(elem),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 50.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 50.0,
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
    };
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
    style.z_index = ZIndexValue::Integer(5);
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);
    assert!(!layers.is_empty(), "应至少有根图层");
    assert!(layers[0].is_root, "有提升子元素时第一个图层仍为根图层");
}
/// 创建两个子元素（opacity < 1.0），两者均被提升为独立合成层。
/// 验证根图层仍然排在最前，且根图层只包含根布局盒自身。
#[test]
fn test_composite_root_layer_first_when_all_children_promoted() {
    use zero_style_system::property::ZIndexValue;

    let mut doc = zero_dom::Document::new();
    let elem1 = doc.create_element("div");
    let elem2 = doc.create_element("div");

    let child1 = LayoutBox {
        node_id: Some(elem1),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 50.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 50.0,
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
    };
    let child2 = LayoutBox {
        node_id: Some(elem2),
        x: 100.0,
        y: 0.0,
        width: 100.0,
        height: 50.0,
        content_x: 100.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 50.0,
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
    };
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
        children: vec![child1, child2],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();

    // 两个子元素都有 opacity < 1.0，都会被提升
    let mut style1 = ComputedStyle::default();
    style1.opacity = 0.5;
    style1.z_index = ZIndexValue::Integer(1);
    styles.insert(elem1, style1);

    let mut style2 = ComputedStyle::default();
    style2.opacity = 0.7;
    style2.z_index = ZIndexValue::Integer(2);
    styles.insert(elem2, style2);

    let layers = promote_compositing_layers(&root_box, &styles);

    // 根图层 + 2 个提升图层
    assert_eq!(layers.len(), 3, "应有根图层 + 2 个提升图层");

    // 根图层始终为第一个
    assert!(layers[0].is_root, "第一个图层应为根图层");
    assert_eq!(layers[0].id, 0, "根图层 id 应为 0");

    // 提升的图层按 z-index 升序排列
    assert_eq!(layers[1].z_index, 1);
    assert_eq!(layers[2].z_index, 2);

    // 根图层只包含根布局盒（子元素都被提升了）
    assert_eq!(layers[0].boxes.len(), 1, "根图层应只包含根布局盒自身");
}
/// 创建一个 opacity=0.4 的子元素，验证其被提升为独立图层，
/// 且提升图层的 opacity 值精确匹配 0.4。
#[test]
fn test_composite_opacity_value_propagation() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = LayoutBox {
        node_id: Some(elem),
        x: 10.0,
        y: 20.0,
        width: 200.0,
        height: 100.0,
        content_x: 10.0,
        content_y: 20.0,
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
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };
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
    style.opacity = 0.4;
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);

    // 根图层 + 1 个提升图层
    assert_eq!(layers.len(), 2, "应有根图层 + 1 个提升图层");
    assert!(layers[0].is_root, "第一个图层应为根图层");
    assert!(!layers[1].is_root, "第二个图层应为提升图层");

    // opacity 值应精确传播到提升图层
    assert!(
        (layers[1].opacity - 0.4).abs() < 0.001,
        "提升图层 opacity 应为 0.4，实际 {}",
        layers[1].opacity
    );

    // 提升图层的几何信息正确
    assert_eq!(layers[1].offset_x, 10.0, "offset_x 应为子元素 x");
    assert_eq!(layers[1].offset_y, 20.0, "offset_y 应为子元素 y");
    assert_eq!(layers[1].width, 200.0, "width 应为子元素宽度");
    assert_eq!(layers[1].height, 100.0, "height 应为子元素高度");
}
/// 三个元素均设置 z-index=5，验证合成层返回根图层 + 3 个提升图层，
/// 且提升图层的 z-index 均为 5，顺序按 DOM 出现顺序排列。
#[test]
fn test_composite_identical_z_index_values() {
    use zero_style_system::property::ZIndexValue;

    let mut doc = zero_dom::Document::new();
    let elem_a = doc.create_element("div");
    let elem_b = doc.create_element("div");
    let elem_c = doc.create_element("div");

    let make_box = |elem_id| LayoutBox {
        node_id: Some(elem_id),
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
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let child_a = make_box(elem_a);
    let child_b = make_box(elem_b);
    let child_c = make_box(elem_c);

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
        children: vec![child_a, child_b, child_c],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();
    for &elem in &[elem_a, elem_b, elem_c] {
        let mut style = ComputedStyle::default();
        style.z_index = ZIndexValue::Integer(5);
        styles.insert(elem, style);
    }

    let layers = promote_compositing_layers(&root_box, &styles);

    // 根图层 + 3 个提升图层
    assert_eq!(layers.len(), 4, "根图层 + 3 个相同 z-index 提升图层");
    assert!(layers[0].is_root, "第一个图层应为根图层");

    // 所有提升图层的 z-index 均为 5
    assert_eq!(layers[1].z_index, 5, "第一个提升图层 z-index 应为 5");
    assert_eq!(layers[2].z_index, 5, "第二个提升图层 z-index 应为 5");
    assert_eq!(layers[3].z_index, 5, "第三个提升图层 z-index 应为 5");
}
/// 构建三个子元素：z-index 分别为 -10、0、5。
/// 验证负 z-index 图层在合成排序中排在最前面（根图层之后），
/// 且所有提升图层按 z-index 严格升序排列。
#[test]
fn test_composite_negative_z_index_values() {
    use zero_style_system::property::ZIndexValue;

    let mut doc = zero_dom::Document::new();
    let elem_neg = doc.create_element("div");
    let elem_zero = doc.create_element("div");
    let elem_pos = doc.create_element("div");

    let make_box = |elem_id| LayoutBox {
        node_id: Some(elem_id),
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
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let child_neg = make_box(elem_neg);
    let child_zero = make_box(elem_zero);
    let child_pos = make_box(elem_pos);

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
        children: vec![child_neg, child_zero, child_pos],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    let mut styles = HashMap::new();

    let mut style_neg = ComputedStyle::default();
    style_neg.z_index = ZIndexValue::Integer(-10);
    styles.insert(elem_neg, style_neg);

    let mut style_zero = ComputedStyle::default();
    style_zero.z_index = ZIndexValue::Integer(0);
    styles.insert(elem_zero, style_zero);

    let mut style_pos = ComputedStyle::default();
    style_pos.z_index = ZIndexValue::Integer(5);
    styles.insert(elem_pos, style_pos);

    let layers = promote_compositing_layers(&root_box, &styles);

    // 根图层 + 3 个提升图层
    assert_eq!(layers.len(), 4, "应有根图层 + 3 个提升图层");
    assert!(layers[0].is_root, "第一个图层应为根图层");

    // 提升图层按 z-index 严格升序：-10, 0, 5
    assert_eq!(layers[1].z_index, -10, "第一个提升图层 z-index 应为 -10");
    assert_eq!(layers[2].z_index, 0, "第二个提升图层 z-index 应为 0");
    assert_eq!(layers[3].z_index, 5, "第三个提升图层 z-index 应为 5");

    // 验证严格单调递增
    assert!(layers[1].z_index < layers[2].z_index, "负 z-index 应排在零之前");
    assert!(
        layers[2].z_index < layers[3].z_index,
        "零 z-index 应排在正 z-index 之前"
    );
}
/// opacity=0 的元素完全透明但仍占据布局空间。
/// 与 visibility:hidden 不同，opacity=0 的元素应被提升为独立合成层，
/// 合成层在最终合成时处理透明度。验证提升图层存在且 opacity 值精确为 0.0。
#[test]
fn test_composite_zero_opacity_element_promoted() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    let child_box = LayoutBox {
        node_id: Some(elem),
        x: 50.0,
        y: 50.0,
        width: 200.0,
        height: 100.0,
        content_x: 50.0,
        content_y: 50.0,
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
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };
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
    style.opacity = 0.0;
    styles.insert(elem, style);

    let layers = promote_compositing_layers(&root_box, &styles);

    // 根图层 + 1 个提升图层（opacity < 1.0 触发提升）
    assert_eq!(layers.len(), 2, "应有根图层 + 1 个零透明度提升图层");
    assert!(layers[0].is_root, "第一个图层应为根图层");
    assert!(!layers[1].is_root, "第二个图层应为提升图层");

    // 提升图层的 opacity 应精确为 0.0
    assert!(
        (layers[1].opacity - 0.0).abs() < 0.001,
        "零透明度提升图层 opacity 应为 0.0，实际 {}",
        layers[1].opacity
    );

    // 提升图层几何信息正确
    assert_eq!(layers[1].offset_x, 50.0, "offset_x 应为子元素 x");
    assert_eq!(layers[1].offset_y, 50.0, "offset_y 应为子元素 y");
    assert_eq!(layers[1].width, 200.0, "width 应为子元素宽度");
    assert_eq!(layers[1].height, 100.0, "height 应为子元素高度");
}

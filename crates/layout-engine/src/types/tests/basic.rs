use std::sync::Arc;
// Auto-generated test file — split from layout-engine/types.rs
use super::super::*;

/// 测试默认 LayoutBox 的基本属性。
#[test]
fn test_layout_box_default() {
    let box0 = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 0.0,
        content_height: 0.0,
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
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    assert_eq!(box0.width, 0.0);
    assert_eq!(box0.height, 0.0);
    assert!(!box0.is_absolute);
    assert!(!box0.is_fixed);
    assert!(box0.children.is_empty());
}

/// 测试 absolute_position。
#[test]
fn test_layout_box_absolute_position() {
    let box0 = LayoutBox {
        node_id: None,
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
        content_x: 12.0,
        content_y: 22.0,
        content_width: 96.0,
        content_height: 46.0,
        border_top: 1.0,
        border_right: 1.0,
        border_bottom: 1.0,
        border_left: 1.0,
        padding_top: 1.0,
        padding_right: 1.0,
        padding_bottom: 1.0,
        padding_left: 1.0,
        margin_top: 5.0,
        margin_right: 5.0,
        margin_bottom: 5.0,
        margin_left: 5.0,
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    let (abs_x, abs_y) = box0.absolute_position();
    assert_eq!(abs_x, 10.0);
    assert_eq!(abs_y, 20.0);
}

/// 测试 outer_area。
#[test]
fn test_layout_box_outer_area() {
    let box0 = LayoutBox {
        node_id: None,
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
        margin_top: 10.0,
        margin_right: 10.0,
        margin_bottom: 10.0,
        margin_left: 10.0,
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    // 总宽度 = 10 + 100 + 10 = 120, 总高度 = 10 + 50 + 10 = 70
    let area = box0.outer_area();
    assert!((area - 120.0 * 70.0).abs() < 0.001);
}

/// 测试 content box 计算。
#[test]
fn test_layout_box_content_box() {
    let box0 = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 80.0,
        content_x: 5.0,
        content_y: 5.0,
        content_width: 90.0,
        content_height: 70.0,
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
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    // content = 100 - 2*2 - 2*3 = 100 - 10 = 90
    assert!((box0.content_width - 90.0).abs() < 0.001);
    assert!((box0.content_height - 70.0).abs() < 0.001);
}

/// 测试 OverflowClip 各变体。
#[test]
fn test_overflow_clip_variants() {
    assert_eq!(OverflowClip::Visible, OverflowClip::Visible);
    assert_eq!(OverflowClip::Hidden, OverflowClip::Hidden);
    assert_eq!(OverflowClip::Clip, OverflowClip::Clip);
    assert_eq!(OverflowClip::Scroll, OverflowClip::Scroll);
    assert_ne!(OverflowClip::Visible, OverflowClip::Hidden);
}

/// 测试 LayoutResult 的视口信息。
#[test]
fn test_layout_result_viewport() {
    let result = LayoutResult {
        root: Arc::new(LayoutBox {
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
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            clear: zero_css_parser::values::ClearValue::None,
            z_index: 0,
            float: zero_css_parser::values::FloatValue::None,
            ..Default::default()
        }),
        viewport_width: 800.0,
        viewport_height: 600.0,
        paint_skip_node_ids: Default::default(),
    };
    assert!((result.viewport_width - 800.0).abs() < 0.001);
    assert!((result.viewport_height - 600.0).abs() < 0.001);
}

/// 测试带子节点的 LayoutBox。
#[test]
fn test_layout_box_with_children() {
    let child = LayoutBox {
        node_id: None,
        x: 10.0,
        y: 10.0,
        width: 50.0,
        height: 30.0,
        content_x: 10.0,
        content_y: 10.0,
        content_width: 50.0,
        content_height: 30.0,
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
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    let parent = LayoutBox {
        node_id: None,
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
        children: vec![child],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    assert_eq!(parent.children.len(), 1);
    assert!((parent.children[0].x - 10.0).abs() < 0.001);
}

/// 测试嵌套绝对位置计算。
#[test]
fn test_layout_box_nested_absolute_position() {
    let child = LayoutBox {
        node_id: None,
        x: 20.0,
        y: 30.0,
        width: 50.0,
        height: 50.0,
        content_x: 20.0,
        content_y: 30.0,
        content_width: 50.0,
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
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    let (abs_x, abs_y) = child.absolute_position_with_parent(10.0, 20.0);
    assert!((abs_x - 30.0).abs() < 0.001);
    assert!((abs_y - 50.0).abs() < 0.001);
}

/// 测试零尺寸元素。
#[test]
fn test_layout_box_zero_size() {
    let box0 = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 0.0,
        content_height: 0.0,
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
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    assert!((box0.outer_area()).abs() < 0.001);
}

/// 测试 LayoutBox 的 clone。
#[test]
fn test_layout_box_clone() {
    let box0 = LayoutBox {
        node_id: None,
        x: 5.0,
        y: 10.0,
        width: 100.0,
        height: 50.0,
        content_x: 7.0,
        content_y: 12.0,
        content_width: 96.0,
        content_height: 46.0,
        border_top: 1.0,
        border_right: 1.0,
        border_bottom: 1.0,
        border_left: 1.0,
        padding_top: 1.0,
        padding_right: 1.0,
        padding_bottom: 1.0,
        padding_left: 1.0,
        margin_top: 2.0,
        margin_right: 2.0,
        margin_bottom: 2.0,
        margin_left: 2.0,
        children: vec![],
        is_absolute: true,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Scroll,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 10,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    let cloned = box0.clone();
    assert!((cloned.x - 5.0).abs() < 0.001);
    assert!(cloned.is_absolute);
    assert_eq!(cloned.overflow_x, OverflowClip::Hidden);
    assert_eq!(cloned.overflow_y, OverflowClip::Scroll);
    assert_eq!(cloned.z_index, 10);
}

/// 测试 LayoutBox 的 z_index 字段。
#[test]
fn test_layout_box_z_index() {
    // 默认 z_index 为 0（对应 auto）
    let box_default = LayoutBox {
        node_id: None,
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
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    assert_eq!(box_default.z_index, 0);

    // 正 z-index
    let box_positive = LayoutBox {
        node_id: None,
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
        is_absolute: true,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 999,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    assert_eq!(box_positive.z_index, 999);

    // 负 z-index
    let box_negative = LayoutBox {
        node_id: None,
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
        is_absolute: true,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: -1,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    assert_eq!(box_negative.z_index, -1);
}

/// 测试 z-index 排序：多个 LayoutBox 按 z_index 排序后顺序正确。
#[test]
fn test_layout_box_z_index_sorting() {
    let boxes = vec![
        LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 50.0,
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
            is_absolute: true,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            clear: zero_css_parser::values::ClearValue::None,
            z_index: 10,
            float: zero_css_parser::values::FloatValue::None,
            ..Default::default()
        },
        LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 50.0,
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
            is_absolute: true,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            clear: zero_css_parser::values::ClearValue::None,
            z_index: -1,
            float: zero_css_parser::values::FloatValue::None,
            ..Default::default()
        },
        LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 50.0,
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
            is_absolute: true,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            clear: zero_css_parser::values::ClearValue::None,
            z_index: 5,
            float: zero_css_parser::values::FloatValue::None,
            ..Default::default()
        },
    ];

    let mut sorted = boxes;
    sorted.sort_by_key(|b| b.z_index);
    assert_eq!(sorted[0].z_index, -1);
    assert_eq!(sorted[1].z_index, 5);
    assert_eq!(sorted[2].z_index, 10);
}

// -- 边界条件测试 --

/// 测试 LayoutBox outer_area 为负值（负 margin）
#[test]
fn test_layout_box_negative_margin_outer_area() {
    // 负 margin 可以让 outer_area 变为负值或零
    let box0 = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 10.0,
        content_height: 10.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: -20.0,
        margin_right: -20.0,
        margin_bottom: -20.0,
        margin_left: -20.0,
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    // total_width = -20 + 10 + -20 = -30, total_height = -20 + 10 + -20 = -30
    // outer_area = -30 * -30 = 900（两个负值相乘为正）
    let area = box0.outer_area();
    assert!(area >= 0.0 || area.is_nan(), "负 margin 导致 outer_area 为 {}", area);
}

/// 测试 LayoutBox 深层嵌套 absolute_position_with_parent
#[test]
fn test_layout_box_deeply_nested_position() {
    // 3 层嵌套，验证绝对位置累积正确
    let level3 = LayoutBox {
        node_id: None,
        x: 5.0,
        y: 5.0,
        width: 10.0,
        height: 10.0,
        content_x: 5.0,
        content_y: 5.0,
        content_width: 10.0,
        content_height: 10.0,
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
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    let level2 = LayoutBox {
        node_id: None,
        x: 20.0,
        y: 30.0,
        width: 100.0,
        height: 100.0,
        content_x: 20.0,
        content_y: 30.0,
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
        children: vec![level3],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    let level1 = LayoutBox {
        node_id: None,
        x: 100.0,
        y: 200.0,
        width: 500.0,
        height: 500.0,
        content_x: 100.0,
        content_y: 200.0,
        content_width: 500.0,
        content_height: 500.0,
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
        children: vec![level2],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    // level1 → level2: (100+20, 200+30) = (120, 230)
    let (abs_x2, abs_y2) = level1.children[0].absolute_position_with_parent(100.0, 200.0);
    assert!((abs_x2 - 120.0).abs() < 0.001);
    assert!((abs_y2 - 230.0).abs() < 0.001);

    // level1 → level2 → level3: (120+5, 230+5) = (125, 235)
    let (abs_x3, abs_y3) = level1.children[0].children[0].absolute_position_with_parent(abs_x2, abs_y2);
    assert!((abs_x3 - 125.0).abs() < 0.001);
    assert!((abs_y3 - 235.0).abs() < 0.001);
}

/// 测试 LayoutBox is_sticky 字段
#[test]
fn test_layout_box_sticky_flag() {
    // 创建 is_sticky = true 的 LayoutBox
    let box0 = LayoutBox {
        node_id: None,
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
        is_sticky: true,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    assert!(box0.is_sticky, "is_sticky 应为 true");
    assert!(!box0.is_absolute, "is_absolute 应为 false");
    assert!(!box0.is_fixed, "is_fixed 应为 false");
}

/// 测试 LayoutBox z_index 为负值
#[test]
fn test_layout_box_negative_z_index() {
    // LayoutBox with z_index = -1
    let box0 = LayoutBox {
        node_id: None,
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
        is_absolute: true,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: -1,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    assert_eq!(box0.z_index, -1);
    assert!(box0.z_index < 0, "z_index 应为负值");
}

/// 测试 LayoutBox 零尺寸子元素
#[test]
fn test_layout_box_zero_size_children() {
    // 子元素 width=0, height=0
    let child1 = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 0.0,
        content_height: 0.0,
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
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    let child2 = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 0.0,
        content_height: 0.0,
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
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    let parent = LayoutBox {
        node_id: None,
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
        children: vec![child1, child2],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    assert_eq!(parent.children.len(), 2);
    assert!((parent.children[0].width).abs() < 0.001);
    assert!((parent.children[1].height).abs() < 0.001);
}

/// 测试 LayoutBox 大量子元素
#[test]
fn test_layout_box_many_children() {
    // 100 个子元素，验证数量
    let children: Vec<LayoutBox> = (0..100)
        .map(|i| LayoutBox {
            node_id: None,
            x: i as f32,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            content_x: i as f32,
            content_y: 0.0,
            content_width: 10.0,
            content_height: 10.0,
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
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            clear: zero_css_parser::values::ClearValue::None,
            z_index: 0,
            float: zero_css_parser::values::FloatValue::None,
            ..Default::default()
        })
        .collect();
    let parent = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 1000.0,
        height: 10.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 1000.0,
        content_height: 10.0,
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
        children,
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    assert_eq!(parent.children.len(), 100);
    // 验证第一个和最后一个子元素
    assert!((parent.children[0].x - 0.0).abs() < 0.001);
    assert!((parent.children[99].x - 99.0).abs() < 0.001);
}

// -- 边界条件测试（第三批）--

/// 测试 LayoutBox 同时标记多个定位标志。
///
/// 在极端情况下，一个元素不应同时标记为 absolute 和 fixed，
/// 验证默认状态下所有定位标志互不干扰。
#[test]
fn test_layout_box_position_flags_mutually_exclusive() {
    let mut box_abs = LayoutBox {
        node_id: None,
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
        is_absolute: true,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    assert!(box_abs.is_absolute);
    assert!(!box_abs.is_fixed);
    assert!(!box_abs.is_sticky);

    // 切换为 fixed
    box_abs.is_absolute = false;
    box_abs.is_fixed = true;
    assert!(!box_abs.is_absolute);
    assert!(box_abs.is_fixed);

    // 切换为 sticky
    box_abs.is_fixed = false;
    box_abs.is_sticky = true;
    assert!(!box_abs.is_fixed);
    assert!(box_abs.is_sticky);
}

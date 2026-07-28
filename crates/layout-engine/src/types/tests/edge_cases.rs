// 边界条件和极端值测试 — types 模块。
use super::super::*;

// ── LayoutBox 极端值测试 ──

/// 测试 LayoutBox 使用 f32 最小正值尺寸。
///
/// f32::MIN_POSITIVE 非常小（约 1.175e-38），
/// 两个 MIN_POSITIVE 相乘会下溢为 0.0，这是正常的浮点行为。
#[test]
fn test_layout_box_f32_min_positive_size() {
    let box0 = LayoutBox {
        node_id: None,
        x: f32::MIN_POSITIVE,
        y: f32::MIN_POSITIVE,
        width: f32::MIN_POSITIVE,
        height: f32::MIN_POSITIVE,
        content_x: 0.0,
        content_y: 0.0,
        content_width: f32::MIN_POSITIVE,
        content_height: f32::MIN_POSITIVE,
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
    assert!(box0.width > 0.0, "f32::MIN_POSITIVE 应为正数");
    assert!(box0.height > 0.0, "f32::MIN_POSITIVE 应为正数");
    // MIN_POSITIVE * MIN_POSITIVE 下溢为 0.0（subnormal 浮点行为）
    let area = box0.outer_area();
    assert!(
        area >= 0.0 && !area.is_nan(),
        "MIN_POSITIVE outer_area 应为 0.0（下溢），实际 {}",
        area
    );
}

/// 测试 LayoutBox 极负坐标下的 absolute_position_with_parent。
#[test]
fn test_layout_box_extreme_negative_position() {
    let box0 = LayoutBox {
        node_id: None,
        x: -1000000.0,
        y: -2000000.0,
        width: 100.0,
        height: 100.0,
        content_x: -1000000.0,
        content_y: -2000000.0,
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
    let (abs_x, abs_y) = box0.absolute_position();
    assert!((abs_x - (-1000000.0)).abs() < 0.01);
    assert!((abs_y - (-2000000.0)).abs() < 0.01);

    let (abs_x2, abs_y2) = box0.absolute_position_with_parent(-500000.0, -300000.0);
    assert!((abs_x2 - (-1500000.0)).abs() < 0.01);
    assert!((abs_y2 - (-2300000.0)).abs() < 0.01);
}

/// 测试 LayoutBox content clamp：负 content 不应出现（由 .max(0.0) 保护）。
///
/// 在 LayoutBox 的字段中，content_width 和 content_height
/// 可以被构造为负值（因为它是 pub 字段），验证直接构造的行为。
#[test]
fn test_layout_box_negative_content_direct_construction() {
    let box0 = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
        content_x: 0.0,
        content_y: 0.0,
        // 直接构造负 content 值，验证不 panic
        content_width: -5.0,
        content_height: -3.0,
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
    // LayoutBox 是 pub 字段结构体，直接构造可以设置负值
    assert!((box0.content_width - (-5.0)).abs() < 0.001);
    assert!((box0.content_height - (-3.0)).abs() < 0.001);
}

/// 测试 LayoutBox 5 层嵌套的 absolute_position_with_parent 累积。
#[test]
fn test_layout_box_five_level_nested_position() {
    let make_box = |x: f32, y: f32| LayoutBox {
        node_id: None,
        x,
        y,
        width: 10.0,
        height: 10.0,
        content_x: x,
        content_y: y,
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

    // 5 层嵌套，每层 x=10, y=10
    let level5 = make_box(10.0, 10.0);
    let level4 = LayoutBox {
        children: vec![level5],
        ..make_box(10.0, 10.0)
    };
    let level3 = LayoutBox {
        children: vec![level4],
        ..make_box(10.0, 10.0)
    };
    let level2 = LayoutBox {
        children: vec![level3],
        ..make_box(10.0, 10.0)
    };
    let level1 = LayoutBox {
        children: vec![level2],
        ..make_box(10.0, 10.0)
    };

    // 从 level1 开始累加：10 + 10 + 10 + 10 + 10 = 50
    let (x2, y2) = level1.children[0].absolute_position_with_parent(10.0, 10.0);
    assert!((x2 - 20.0).abs() < 0.001);
    assert!((y2 - 20.0).abs() < 0.001);

    let (x3, y3) = level1.children[0].children[0].absolute_position_with_parent(x2, y2);
    assert!((x3 - 30.0).abs() < 0.001);
    assert!((y3 - 30.0).abs() < 0.001);

    let (x5, y5) = level1.children[0].children[0].children[0].children[0].absolute_position_with_parent(40.0, 40.0);
    assert!((x5 - 50.0).abs() < 0.001, "第 5 层绝对 x 应为 50，实际 {}", x5);
    assert!((y5 - 50.0).abs() < 0.001, "第 5 层绝对 y 应为 50，实际 {}", y5);
}

/// 测试 LayoutBox outer_area 在单侧负 margin 下的计算。
#[test]
fn test_layout_box_single_negative_margin_outer_area() {
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
        margin_top: -10.0,
        margin_right: 5.0,
        margin_bottom: 5.0,
        margin_left: -10.0,
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
    // total_width = -10 + 100 + 5 = 95
    // total_height = -10 + 50 + 5 = 45
    // outer_area = 95 * 45 = 4275
    let area = box0.outer_area();
    assert!(
        (area - 4275.0).abs() < 0.001,
        "单侧负 margin outer_area 应为 4275，实际 {}",
        area
    );
}

/// 测试 LayoutBox 同时标记 is_fixed 和 is_absolute 的极端情况。
#[test]
fn test_layout_box_fixed_and_absolute_flags() {
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
        is_fixed: true,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 5,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    // 构造允许同时标记，验证读取正确
    assert!(box0.is_absolute);
    assert!(box0.is_fixed);
    assert_eq!(box0.z_index, 5);
}

/// 测试 LayoutBox 使用 i32::MIN 和 i32::MAX 作为 z_index。
#[test]
fn test_layout_box_extreme_z_index() {
    let box_min = LayoutBox {
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
        z_index: i32::MIN,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    assert_eq!(box_min.z_index, i32::MIN);

    let box_max = LayoutBox {
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
        z_index: i32::MAX,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    assert_eq!(box_max.z_index, i32::MAX);
}

// ── OverflowClip 边界条件 ──

/// 测试 OverflowClip 所有变体两两不等（除了自身）。
#[test]
fn test_overflow_clip_all_variants_distinct() {
    let variants = [
        OverflowClip::Visible,
        OverflowClip::Hidden,
        OverflowClip::Clip,
        OverflowClip::Scroll,
    ];
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i == j {
                assert_eq!(a, b, "相同变体应相等");
            } else {
                assert_ne!(a, b, "不同变体 {:?} 和 {:?} 应不相等", a, b);
            }
        }
    }
}

// ── LayoutResult 边界条件 ──

/// 测试 LayoutResult 极大视口尺寸。
#[test]
fn test_layout_result_large_viewport() {
    let result = LayoutResult {
        root: LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 7680.0,
            height: 4320.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 7680.0,
            content_height: 4320.0,
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
        },
        viewport_width: 7680.0,
        viewport_height: 4320.0,
        paint_skip_node_ids: Default::default(),
    };
    assert!((result.viewport_width - 7680.0).abs() < 0.001);
    assert!((result.viewport_height - 4320.0).abs() < 0.001);
}

/// 测试 LayoutBox 多层子节点递归访问。
#[test]
fn test_layout_box_recursive_child_access() {
    let leaf1 = LayoutBox {
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
    let leaf2 = LayoutBox {
        node_id: None,
        x: 15.0,
        y: 5.0,
        width: 10.0,
        height: 10.0,
        content_x: 15.0,
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
    let parent = LayoutBox {
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
        children: vec![leaf1, leaf2],
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
    assert!((parent.children[0].x - 5.0).abs() < 0.001);
    assert!((parent.children[1].x - 15.0).abs() < 0.001);
}

/// 测试 LayoutBox 使用 f32::INFINITY 坐标。
#[test]
fn test_layout_box_infinity_coordinates() {
    let box0 = LayoutBox {
        node_id: None,
        x: f32::INFINITY,
        y: f32::NEG_INFINITY,
        width: 100.0,
        height: 100.0,
        content_x: f32::INFINITY,
        content_y: f32::NEG_INFINITY,
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
    let (abs_x, abs_y) = box0.absolute_position();
    assert!(abs_x.is_infinite() && abs_x.is_sign_positive());
    assert!(abs_y.is_infinite() && abs_y.is_sign_negative());
}

//! LayoutBox 方法补充覆盖测试。
use std::sync::Arc;

use crate::types::{LayoutBox, LayoutResult, OverflowClip};

fn make_box_at(x: f32, y: f32, w: f32, h: f32) -> LayoutBox {
    LayoutBox {
        node_id: None,
        x,
        y,
        width: w,
        height: h,
        content_x: x,
        content_y: y,
        content_width: w,
        content_height: h,
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
    }
}

fn make_box_with_margins(ml: f32, mr: f32, mt: f32, mb: f32) -> LayoutBox {
    LayoutBox {
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
        margin_top: mt,
        margin_right: mr,
        margin_bottom: mb,
        margin_left: ml,
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
    }
}

// ---- absolute_position 测试 ----

#[test]
fn test_absolute_position_origin() {
    let b = make_box_at(0.0, 0.0, 100.0, 100.0);
    let (x, y) = b.absolute_position();
    assert_eq!(x, 0.0);
    assert_eq!(y, 0.0);
}

#[test]
fn test_absolute_position_with_offset() {
    let b = make_box_at(42.5, 99.9, 10.0, 10.0);
    let (x, y) = b.absolute_position();
    assert_eq!(x, 42.5);
    assert_eq!(y, 99.9);
}

#[test]
fn test_absolute_position_negative() {
    let b = make_box_at(-10.0, -20.0, 50.0, 50.0);
    let (x, y) = b.absolute_position();
    assert_eq!(x, -10.0);
    assert_eq!(y, -20.0);
}

// ---- absolute_position_with_parent 测试 ----

#[test]
fn test_abs_pos_with_parent_zero() {
    let b = make_box_at(10.0, 20.0, 50.0, 50.0);
    let (x, y) = b.absolute_position_with_parent(0.0, 0.0);
    assert_eq!(x, 10.0);
    assert_eq!(y, 20.0);
}

#[test]
fn test_abs_pos_with_parent_offset() {
    let b = make_box_at(10.0, 20.0, 50.0, 50.0);
    let (x, y) = b.absolute_position_with_parent(100.0, 200.0);
    assert_eq!(x, 110.0);
    assert_eq!(y, 220.0);
}

#[test]
fn test_abs_pos_with_parent_negative() {
    let b = make_box_at(50.0, 60.0, 10.0, 10.0);
    let (x, y) = b.absolute_position_with_parent(-30.0, -40.0);
    assert_eq!(x, 20.0);
    assert_eq!(y, 20.0);
}

// ---- outer_area 测试 ----

#[test]
fn test_outer_area_no_margins() {
    let b = make_box_with_margins(0.0, 0.0, 0.0, 0.0);
    assert_eq!(b.outer_area(), 5000.0); // 100 * 50
}

#[test]
fn test_outer_area_with_margins() {
    let b = make_box_with_margins(10.0, 10.0, 5.0, 5.0);
    // total_w = 10 + 100 + 10 = 120, total_h = 5 + 50 + 5 = 60, area = 7200
    assert_eq!(b.outer_area(), 7200.0);
}

#[test]
fn test_outer_area_asymmetric_margins() {
    let b = make_box_with_margins(5.0, 15.0, 10.0, 20.0);
    // total_w = 5 + 100 + 15 = 120, total_h = 10 + 50 + 20 = 80, area = 9600
    assert_eq!(b.outer_area(), 9600.0);
}

#[test]
fn test_outer_area_negative_margins() {
    let b = make_box_with_margins(-5.0, -5.0, -10.0, -10.0);
    // total_w = -5 + 100 + -5 = 90, total_h = -10 + 50 + -10 = 30, area = 2700
    assert_eq!(b.outer_area(), 2700.0);
}

// ---- OverflowClip 测试 ----

#[test]
fn test_overflow_clip_variants_distinct() {
    let variants = [
        OverflowClip::Visible,
        OverflowClip::Hidden,
        OverflowClip::Clip,
        OverflowClip::Scroll,
    ];
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn test_overflow_clip_copy_preserves() {
    let v = OverflowClip::Scroll;
    let copied = v;
    assert_eq!(v, copied);
}

// ---- LayoutBox 定位标志测试 ----

#[test]
fn test_layout_box_absolute_flag() {
    let mut b = make_box_at(0.0, 0.0, 100.0, 100.0);
    b.is_absolute = true;
    assert!(b.is_absolute);
    assert!(!b.is_fixed);
    assert!(!b.is_sticky);
}

#[test]
fn test_layout_box_fixed_flag() {
    let mut b = make_box_at(0.0, 0.0, 100.0, 100.0);
    b.is_fixed = true;
    assert!(!b.is_absolute);
    assert!(b.is_fixed);
}

#[test]
fn test_layout_box_sticky_flag() {
    let mut b = make_box_at(0.0, 0.0, 100.0, 100.0);
    b.is_sticky = true;
    assert!(b.is_sticky);
}

#[test]
fn test_layout_box_multiple_position_flags() {
    let mut b = make_box_at(0.0, 0.0, 100.0, 100.0);
    b.is_absolute = true;
    b.is_fixed = true;
    assert!(b.is_absolute);
    assert!(b.is_fixed);
}

// ---- z-index 测试 ----

#[test]
fn test_z_index_positive() {
    let mut b = make_box_at(0.0, 0.0, 100.0, 100.0);
    b.z_index = 999;
    assert_eq!(b.z_index, 999);
}

#[test]
fn test_z_index_negative() {
    let mut b = make_box_at(0.0, 0.0, 100.0, 100.0);
    b.z_index = -100;
    assert_eq!(b.z_index, -100);
}

#[test]
fn test_z_index_zero_auto() {
    let b = make_box_at(0.0, 0.0, 100.0, 100.0);
    assert_eq!(b.z_index, 0);
}

// ---- 子节点层级测试 ----

#[test]
fn test_children_empty() {
    let b = make_box_at(0.0, 0.0, 100.0, 100.0);
    assert!(b.children.is_empty());
}

#[test]
fn test_children_with_nested() {
    let mut parent = make_box_at(0.0, 0.0, 200.0, 200.0);
    let mut child = make_box_at(10.0, 10.0, 180.0, 180.0);
    let grandchild = make_box_at(5.0, 5.0, 170.0, 170.0);
    child.children = vec![grandchild];
    parent.children = vec![child];
    assert_eq!(parent.children.len(), 1);
    assert_eq!(parent.children[0].children.len(), 1);
    assert_eq!(parent.children[0].children[0].width, 170.0);
}

// ---- LayoutResult 测试 ----

#[test]
fn test_layout_result_viewport() {
    let result = LayoutResult {
        root: Arc::new(make_box_at(0.0, 0.0, 1024.0, 768.0)),
        viewport_width: 1024.0,
        viewport_height: 768.0,
        paint_skip_node_ids: Default::default(),
    };
    assert_eq!(result.viewport_width, 1024.0);
    assert_eq!(result.viewport_height, 768.0);
    assert_eq!(result.root.width, 1024.0);
}

#[test]
fn test_layout_result_with_children() {
    let mut root = make_box_at(0.0, 0.0, 800.0, 600.0);
    root.children = vec![make_box_at(0.0, 0.0, 800.0, 50.0), make_box_at(0.0, 50.0, 600.0, 550.0)];
    let result = LayoutResult {
        root: Arc::new(root),
        viewport_width: 800.0,
        viewport_height: 600.0,
        paint_skip_node_ids: Default::default(),
    };
    assert_eq!(result.root.children.len(), 2);
    assert_eq!(result.root.children[0].height, 50.0);
    assert_eq!(result.root.children[1].y, 50.0);
}

// ---- clone 测试 ----

#[test]
fn test_layout_box_clone_independence() {
    let mut original = make_box_at(10.0, 20.0, 100.0, 50.0);
    original.children = vec![make_box_at(5.0, 5.0, 90.0, 40.0)];
    let cloned = original.clone();

    // Verify cloned data matches
    assert_eq!(cloned.x, 10.0);
    assert_eq!(cloned.children.len(), 1);
    assert_eq!(cloned.children[0].width, 90.0);
}

// ---- Debug 格式化测试 ----

#[test]
fn test_layout_box_debug_output() {
    let b = make_box_at(10.0, 20.0, 100.0, 50.0);
    let debug = format!("{:?}", b);
    // 应包含关键字段
    assert!(debug.contains("x"));
    assert!(debug.contains("width"));
}

// ---- 边界值测试 ----

#[test]
fn test_zero_size_box() {
    let b = make_box_at(0.0, 0.0, 0.0, 0.0);
    assert_eq!(b.width, 0.0);
    assert_eq!(b.height, 0.0);
    assert_eq!(b.outer_area(), 0.0);
}

#[test]
fn test_very_large_box() {
    let b = make_box_at(0.0, 0.0, f32::MAX, f32::MAX);
    assert!(b.width.is_finite() || b.width == f32::MAX);
}

#[test]
fn test_content_area_independent_of_position() {
    let mut b = make_box_at(1000.0, 2000.0, 50.0, 30.0);
    b.content_x = 1005.0;
    b.content_y = 2005.0;
    b.content_width = 40.0;
    b.content_height = 20.0;
    // content 区域独立于盒子位置
    assert_eq!(b.content_width, 40.0);
    assert_eq!(b.content_height, 20.0);
    assert_eq!(b.width, 50.0);
    assert_eq!(b.height, 30.0);
}

// ---- border + padding + margin 组合测试 ----

#[test]
fn test_border_padding_margin_combination() {
    let b = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 80.0,
        content_x: 16.0, // border(2) + padding(14) = 16
        content_y: 16.0,
        content_width: 70.0, // 100 - 2 - 2 - 13 - 13 = 70
        content_height: 50.0,
        border_top: 2.0,
        border_right: 2.0,
        border_bottom: 2.0,
        border_left: 2.0,
        padding_top: 14.0,
        padding_right: 13.0,
        padding_bottom: 13.0,
        padding_left: 13.0,
        margin_top: 10.0,
        margin_right: 10.0,
        margin_bottom: 10.0,
        margin_left: 10.0,
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Scroll,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 5,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    // outer area = (10+100+10) * (10+80+10) = 120 * 100 = 12000
    assert_eq!(b.outer_area(), 12000.0);
    assert_eq!(b.overflow_x, OverflowClip::Hidden);
    assert_eq!(b.overflow_y, OverflowClip::Scroll);
    assert_eq!(b.z_index, 5);
}

// ── snapshot 和 nth_box 测试 ──────────────────────────────────────

/// 测试 LayoutResult::snapshot 基本输出。
#[test]
fn test_layout_result_snapshot_basic() {
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
    let snap = result.snapshot();
    assert!(
        snap.contains("viewport: 800.00x600.00"),
        "snapshot should contain viewport"
    );
    assert!(
        snap.contains("size=(800.00,600.00)"),
        "snapshot should contain root size"
    );
}

/// 测试 LayoutResult::snapshot 带 border/padding/margin。
#[test]
fn test_layout_result_snapshot_with_box_model() {
    let result = LayoutResult {
        root: Arc::new(LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
            content_x: 6.0,
            content_y: 6.0,
            content_width: 88.0,
            content_height: 68.0,
            border_top: 2.0,
            border_right: 2.0,
            border_bottom: 2.0,
            border_left: 2.0,
            padding_top: 4.0,
            padding_right: 4.0,
            padding_bottom: 4.0,
            padding_left: 4.0,
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
        }),
        viewport_width: 800.0,
        viewport_height: 600.0,
        paint_skip_node_ids: Default::default(),
    };
    let snap = result.snapshot();
    assert!(snap.contains("border="), "should show border");
    assert!(snap.contains("padding="), "should show padding");
    assert!(snap.contains("margin="), "should show margin");
}

/// 测试 LayoutResult::snapshot 带 z-index 和定位标志。
#[test]
fn test_layout_result_snapshot_flags() {
    let result = LayoutResult {
        root: Arc::new(LayoutBox {
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
            children: vec![LayoutBox {
                node_id: None,
                x: 10.0,
                y: 10.0,
                width: 50.0,
                height: 50.0,
                content_x: 10.0,
                content_y: 10.0,
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
            }],
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
    let snap = result.snapshot();
    assert!(snap.contains("abs"), "should show absolute flag");
    assert!(snap.contains("z=5"), "should show z-index");
    assert!(snap.contains("  [-]"), "child should be indented");
}

/// 测试 LayoutBox::nth_box 基本查找。
#[test]
fn test_nth_box_basic() {
    let root = LayoutBox {
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
        children: vec![
            LayoutBox {
                node_id: None,
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 50.0,
                content_x: 0.0,
                content_y: 0.0,
                content_width: 800.0,
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
            },
            LayoutBox {
                node_id: None,
                x: 0.0,
                y: 50.0,
                width: 800.0,
                height: 550.0,
                content_x: 0.0,
                content_y: 50.0,
                content_width: 800.0,
                content_height: 550.0,
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
        ],
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
    // Index 0 = root
    let (_x0, _y0, w0, h0) = root.nth_box(0).unwrap();
    assert_eq!(w0, 800.0);
    assert_eq!(h0, 600.0);
    // Index 1 = first child
    let (x1, _y1, _w1, h1) = root.nth_box(1).unwrap();
    assert_eq!(x1, 0.0);
    assert_eq!(h1, 50.0);
    // Index 2 = second child
    let (_x2, y2, _w2, _h2) = root.nth_box(2).unwrap();
    assert_eq!(y2, 50.0);
    // Out of range
    assert!(root.nth_box(3).is_none());
}

/// 测试 LayoutBox::count_boxes。
#[test]
fn test_count_boxes() {
    let root = LayoutBox {
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
        children: vec![LayoutBox {
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
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            clear: zero_css_parser::values::ClearValue::None,
            z_index: 0,
            float: zero_css_parser::values::FloatValue::None,
            ..Default::default()
        }],
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
    // root(1) + child(1) = 2
    assert_eq!(root.count_boxes(), 2);
}

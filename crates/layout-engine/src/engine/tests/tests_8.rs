use super::*;
use crate::types::{LayoutBox, OverflowClip};
use zero_css_parser::values::OverflowValue;

// ── convert_overflow_to_clip / adjust_fixed_to_viewport 边界条件测试 ──

/// 测试 convert_overflow_to_clip：Visible 映射为 Visible。
#[test]
fn test_convert_overflow_to_clip_visible() {
    let result = convert_overflow_to_clip(&OverflowValue::Visible);
    assert_eq!(result, OverflowClip::Visible);
}

/// 测试 convert_overflow_to_clip：Hidden 映射为 Hidden。
#[test]
fn test_convert_overflow_to_clip_hidden() {
    let result = convert_overflow_to_clip(&OverflowValue::Hidden);
    assert_eq!(result, OverflowClip::Hidden);
}

/// 测试 convert_overflow_to_clip：Clip 映射为 Clip。
#[test]
fn test_convert_overflow_to_clip_clip() {
    let result = convert_overflow_to_clip(&OverflowValue::Clip);
    assert_eq!(result, OverflowClip::Clip);
}

/// 测试 convert_overflow_to_clip：Scroll 映射为 Scroll。
#[test]
fn test_convert_overflow_to_clip_scroll() {
    let result = convert_overflow_to_clip(&OverflowValue::Scroll);
    assert_eq!(result, OverflowClip::Scroll);
}

/// 测试 convert_overflow_to_clip：Auto 映射为 Scroll。
#[test]
fn test_convert_overflow_to_clip_auto() {
    let result = convert_overflow_to_clip(&OverflowValue::Auto);
    assert_eq!(result, OverflowClip::Scroll);
}

/// 测试 adjust_fixed_to_viewport：fixed 子元素加上祖先偏移。
#[test]
fn test_adjust_fixed_to_viewport_nested() {
    let fixed_child = LayoutBox {
        node_id: None,
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
        content_x: 10.0,
        content_y: 20.0,
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
        is_fixed: true,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        z_index: 0,
    };
    let mut root = LayoutBox {
        node_id: None,
        x: 50.0,
        y: 60.0,
        width: 800.0,
        height: 600.0,
        content_x: 50.0,
        content_y: 60.0,
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
        children: vec![fixed_child],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        z_index: 0,
    };

    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);

    // fixed 子元素应加上父级偏移：x = 10 + 50 = 60, y = 20 + 60 = 80
    let child = &root.children[0];
    assert!(
        (child.x - 60.0).abs() < 0.001,
        "fixed 子元素 x 应为 60.0，实际 {}",
        child.x
    );
    assert!(
        (child.y - 80.0).abs() < 0.001,
        "fixed 子元素 y 应为 80.0，实际 {}",
        child.y
    );
}

/// 测试 adjust_fixed_to_viewport：根节点为 fixed 时加上初始偏移。
#[test]
fn test_adjust_fixed_to_viewport_at_root() {
    let mut root = LayoutBox {
        node_id: None,
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 100.0,
        content_x: 10.0,
        content_y: 20.0,
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
        is_fixed: true,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        z_index: 0,
    };

    adjust_fixed_to_viewport(&mut root, 100.0, 200.0);

    // fixed 根节点：x = 10 + 100 = 110, y = 20 + 200 = 220
    assert!(
        (root.x - 110.0).abs() < 0.001,
        "fixed 根节点 x 应为 110.0，实际 {}",
        root.x
    );
    assert!(
        (root.y - 220.0).abs() < 0.001,
        "fixed 根节点 y 应为 220.0，实际 {}",
        root.y
    );
}

/// 测试 adjust_fixed_to_viewport：非 fixed 元素不改变坐标。
#[test]
fn test_adjust_fixed_to_viewport_non_fixed_unchanged() {
    let mut root = LayoutBox {
        node_id: None,
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 100.0,
        content_x: 10.0,
        content_y: 20.0,
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
        z_index: 0,
    };

    adjust_fixed_to_viewport(&mut root, 100.0, 200.0);

    // 非 fixed 元素坐标不变
    assert!((root.x - 10.0).abs() < 0.001, "非 fixed 元素 x 应不变，实际 {}", root.x);
    assert!((root.y - 20.0).abs() < 0.001, "非 fixed 元素 y 应不变，实际 {}", root.y);
}

/// 测试 adjust_fixed_to_viewport：深层嵌套 fixed 元素累积祖先偏移。
#[test]
fn test_adjust_fixed_to_viewport_deeply_nested() {
    let fixed_leaf = LayoutBox {
        node_id: None,
        x: 5.0,
        y: 5.0,
        width: 50.0,
        height: 50.0,
        content_x: 5.0,
        content_y: 5.0,
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
        is_fixed: true,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        z_index: 0,
    };
    let mid = LayoutBox {
        node_id: None,
        x: 30.0,
        y: 40.0,
        width: 200.0,
        height: 200.0,
        content_x: 30.0,
        content_y: 40.0,
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
        children: vec![fixed_leaf],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        z_index: 0,
    };
    let mut root = LayoutBox {
        node_id: None,
        x: 100.0,
        y: 200.0,
        width: 800.0,
        height: 600.0,
        content_x: 100.0,
        content_y: 200.0,
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
        children: vec![mid],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        z_index: 0,
    };

    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);

    // fixed leaf 偏移 = root.x(100) + mid.x(30) + leaf.x(5) = 135
    let leaf = &root.children[0].children[0];
    assert!(
        (leaf.x - 135.0).abs() < 0.001,
        "深层 fixed x 应为 135.0，实际 {}",
        leaf.x
    );
    assert!(
        (leaf.y - 245.0).abs() < 0.001,
        "深层 fixed y 应为 245.0，实际 {}",
        leaf.y
    );
}

/// 测试 adjust_fixed_to_viewport：fixed 元素的子元素以 fixed 为偏移基准（归零）。
#[test]
fn test_adjust_fixed_to_viewport_fixed_resets_offset() {
    let normal_child = LayoutBox {
        node_id: None,
        x: 15.0,
        y: 25.0,
        width: 50.0,
        height: 50.0,
        content_x: 15.0,
        content_y: 25.0,
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
        z_index: 0,
    };
    let fixed_parent = LayoutBox {
        node_id: None,
        x: 10.0,
        y: 20.0,
        width: 200.0,
        height: 200.0,
        content_x: 10.0,
        content_y: 20.0,
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
        children: vec![normal_child],
        is_absolute: false,
        is_fixed: true,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        z_index: 0,
    };
    let mut root = LayoutBox {
        node_id: None,
        x: 100.0,
        y: 200.0,
        width: 800.0,
        height: 600.0,
        content_x: 100.0,
        content_y: 200.0,
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
        children: vec![fixed_parent],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        z_index: 0,
    };

    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);

    // fixed parent: x = 10 + 100 = 110, y = 20 + 200 = 220
    let fp = &root.children[0];
    assert!((fp.x - 110.0).abs() < 0.001, "fixed parent x 应为 110");
    assert!((fp.y - 220.0).abs() < 0.001, "fixed parent y 应为 220");

    // fixed 元素的子元素 offset 归零，所以 normal_child 不被偏移
    let child = &root.children[0].children[0];
    assert!(
        (child.x - 15.0).abs() < 0.001,
        "fixed 子元素的子节点 x 应为 15.0（offset 归零），实际 {}",
        child.x
    );
    assert!(
        (child.y - 25.0).abs() < 0.001,
        "fixed 子元素的子节点 y 应为 25.0（offset 归零），实际 {}",
        child.y
    );
}

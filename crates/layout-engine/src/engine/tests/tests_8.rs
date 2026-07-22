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
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
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
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };

    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);

    // R324：fixed 子元素须视口相对——扣除父级累积偏移：x = 10 - 50 = -40, y = 20 - 60 = -40
    // （field 为父相对值；painter 累积后绝对坐标 = 50+(-40)=10 / 60+(-40)=20 = CSS left/top 视口相对）
    let child = &root.children[0];
    assert!(
        (child.x - (-40.0)).abs() < 0.001,
        "fixed 子元素 x 应为 -40.0，实际 {}",
        child.x
    );
    assert!(
        (child.y - (-40.0)).abs() < 0.001,
        "fixed 子元素 y 应为 -40.0，实际 {}",
        child.y
    );
}

/// 测试 adjust_fixed_to_viewport：四 inset 全 auto 的 fixed 保持在静态位置，不被移到视口原点。
///
/// R1874：`position:fixed` 全 inset auto 时位置 = 静态位置（§10.3.7/§10.6.4）。旧实现对
/// 所有 fixed 一律扣除祖先偏移，把无 inset 的 fixed 错误地移到视口原点 (0,0)。修复后
/// `fixed_insets_all_auto` 标记的 fixed 跳过扣除，x/y 保持 taffy 算出的静态坐标。
/// （对应 WPT CSS2/abspos/static-fixed-inside-abspos：fixed 应覆盖父 abspos 块而非移到原点。）
#[test]
fn test_adjust_fixed_to_viewport_all_auto_insets_keeps_static_position() {
    let fixed_child = LayoutBox {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
        is_fixed: true,
        fixed_insets_all_auto: true,
        ..Default::default()
    };
    let mut root = LayoutBox {
        x: 50.0,
        y: 60.0,
        width: 800.0,
        height: 600.0,
        children: vec![fixed_child],
        ..Default::default()
    };

    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);

    // 全 auto inset：不扣除祖先偏移，x/y 保持静态值 10.0 / 20.0（painter 累积得绝对坐标）。
    let child = &root.children[0];
    assert!(
        (child.x - 10.0).abs() < 0.001,
        "全 auto inset fixed x 应保持 10.0（静态位置），实际 {}",
        child.x
    );
    assert!(
        (child.y - 20.0).abs() < 0.001,
        "全 auto inset fixed y 应保持 20.0（静态位置），实际 {}",
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
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };

    adjust_fixed_to_viewport(&mut root, 100.0, 200.0);

    // R324：fixed 根节点扣除传入偏移（视口相对）：x = 10 - 100 = -90, y = 20 - 200 = -180
    assert!(
        (root.x - (-90.0)).abs() < 0.001,
        "fixed 根节点 x 应为 -90.0，实际 {}",
        root.x
    );
    assert!(
        (root.y - (-180.0)).abs() < 0.001,
        "fixed 根节点 y 应为 -180.0，实际 {}",
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
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
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
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
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
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
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
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };

    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);

    // R324：fixed leaf 扣除累积祖先偏移（视口相对）：x = 5 - (100+30) = -125, y = 5 - (200+40) = -235
    // （painter 累积后绝对坐标 = 130+(-125)=5 / 240+(-235)=5 = CSS left/top 视口相对）
    let leaf = &root.children[0].children[0];
    assert!(
        (leaf.x - (-125.0)).abs() < 0.001,
        "深层 fixed x 应为 -125.0，实际 {}",
        leaf.x
    );
    assert!(
        (leaf.y - (-235.0)).abs() < 0.001,
        "深层 fixed y 应为 -235.0，实际 {}",
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
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
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
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
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
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };

    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);

    // R324：fixed parent 扣除累积祖先偏移（视口相对）：x = 10 - 100 = -90, y = 20 - 200 = -180
    let fp = &root.children[0];
    assert!((fp.x - (-90.0)).abs() < 0.001, "fixed parent x 应为 -90");
    assert!((fp.y - (-180.0)).abs() < 0.001, "fixed parent y 应为 -180");

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

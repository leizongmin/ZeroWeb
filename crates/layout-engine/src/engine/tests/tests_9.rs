// 边界条件和极端值测试 — engine 模块私有函数。
use super::*;
use crate::types::{LayoutBox, OverflowClip};
use zero_css_parser::values::OverflowValue;

// ── convert_overflow_to_clip 边界条件 ──

/// 测试 convert_overflow_to_clip：Visible 映射为 Visible。
#[test]
fn test_overflow_visible_round_trip() {
    let result = convert_overflow_to_clip(&OverflowValue::Visible);
    assert_eq!(result, OverflowClip::Visible);
    assert_eq!(result, OverflowClip::Visible, "Visible 应可复制且比较");
}

/// 测试 convert_overflow_to_clip：所有变体映射正确。
#[test]
fn test_overflow_all_variants_complete_mapping() {
    // Visible
    assert_eq!(convert_overflow_to_clip(&OverflowValue::Visible), OverflowClip::Visible);
    // Hidden
    assert_eq!(convert_overflow_to_clip(&OverflowValue::Hidden), OverflowClip::Hidden);
    // Clip
    assert_eq!(convert_overflow_to_clip(&OverflowValue::Clip), OverflowClip::Clip);
    // Scroll
    assert_eq!(convert_overflow_to_clip(&OverflowValue::Scroll), OverflowClip::Scroll);
    // Auto → Scroll
    assert_eq!(convert_overflow_to_clip(&OverflowValue::Auto), OverflowClip::Scroll);
}

// ── adjust_fixed_to_viewport 边界条件 ──

/// 测试 adjust_fixed_to_viewport：传入零偏移量时 fixed 元素坐标不变。
#[test]
fn test_adjust_fixed_zero_parent_offset() {
    let mut root = LayoutBox {
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
        is_fixed: true,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
    };
    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);
    assert!((root.x - 0.0).abs() < 0.001, "零偏移 + 零坐标 = 0");
    assert!((root.y - 0.0).abs() < 0.001);
}

/// 测试 adjust_fixed_to_viewport：fixed 元素在负坐标父级中。
#[test]
fn test_adjust_fixed_negative_parent_offset() {
    let fixed_child = LayoutBox {
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
        is_absolute: false,
        is_fixed: true,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
    };
    let mut root = LayoutBox {
        node_id: None,
        x: -100.0,
        y: -200.0,
        width: 800.0,
        height: 600.0,
        content_x: -100.0,
        content_y: -200.0,
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
    };
    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);

    // fixed child: x = 10 + (-100) = -90, y = 10 + (-200) = -190
    let child = &root.children[0];
    assert!(
        (child.x - (-90.0)).abs() < 0.001,
        "fixed child x 应为 -90，实际 {}",
        child.x
    );
    assert!(
        (child.y - (-190.0)).abs() < 0.001,
        "fixed child y 应为 -190，实际 {}",
        child.y
    );
}

/// 测试 adjust_fixed_to_viewport：两个连续 fixed 元素互不影响。
#[test]
fn test_adjust_fixed_sibling_fixed_elements() {
    let fixed1 = LayoutBox {
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
    };
    let fixed2 = LayoutBox {
        node_id: None,
        x: 100.0,
        y: 200.0,
        width: 50.0,
        height: 50.0,
        content_x: 100.0,
        content_y: 200.0,
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
    };
    let mut root = LayoutBox {
        node_id: None,
        x: 50.0,
        y: 50.0,
        width: 800.0,
        height: 600.0,
        content_x: 50.0,
        content_y: 50.0,
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
        children: vec![fixed1, fixed2],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
    };
    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);

    // fixed1: x = 5 + 50 = 55, y = 5 + 50 = 55
    let c1 = &root.children[0];
    assert!((c1.x - 55.0).abs() < 0.001, "fixed1 x 应为 55，实际 {}", c1.x);
    assert!((c1.y - 55.0).abs() < 0.001, "fixed1 y 应为 55，实际 {}", c1.y);

    // fixed2: x = 100 + 50 = 150, y = 200 + 50 = 250
    let c2 = &root.children[1];
    assert!((c2.x - 150.0).abs() < 0.001, "fixed2 x 应为 150，实际 {}", c2.x);
    assert!((c2.y - 250.0).abs() < 0.001, "fixed2 y 应为 250，实际 {}", c2.y);
}

/// 测试 adjust_fixed_to_viewport：fixed 元素包含 absolute 子元素。
///
/// fixed 元素的子元素不应被视作 fixed，它们的坐标不变。
/// 但由于 fixed 元素的 offset 归零，子元素的后续偏移应从 0 开始。
#[test]
fn test_adjust_fixed_with_absolute_child() {
    let abs_grandchild = LayoutBox {
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
        is_absolute: true,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
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
        children: vec![abs_grandchild],
        is_absolute: false,
        is_fixed: true,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
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
    };
    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);

    // fixed parent: x = 10 + 100 = 110, y = 20 + 200 = 220
    let fp = &root.children[0];
    assert!((fp.x - 110.0).abs() < 0.001, "fixed parent x 应为 110");
    assert!((fp.y - 220.0).abs() < 0.001, "fixed parent y 应为 220");

    // absolute grandchild: offset 从 fixed 归零后重新累加
    // 由于 fixed parent offset 归零，absolute child 以 0 为基，
    // 它自身的 x=20 不变（不是 fixed，所以坐标不被修改）
    let gc = &root.children[0].children[0];
    assert!((gc.x - 20.0).abs() < 0.001, "absolute child x 应为 20，实际 {}", gc.x);
    assert!((gc.y - 30.0).abs() < 0.001, "absolute child y 应为 30，实际 {}", gc.y);
}

/// 测试 adjust_fixed_to_viewport：空 children 不 panic。
#[test]
fn test_adjust_fixed_empty_children_no_panic() {
    let mut root = LayoutBox {
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
    };
    // 不应 panic
    adjust_fixed_to_viewport(&mut root, 100.0, 200.0);
    assert!((root.x - 0.0).abs() < 0.001);
}

// ── has_direct_text 边界条件 ──

/// 测试 has_direct_text：空元素返回 false。
#[test]
fn test_has_direct_text_empty_element() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    assert!(!has_direct_text(&doc, div), "空 div 不应有直接文本");
}

/// 测试 has_direct_text：仅有空白文本的元素返回 false。
#[test]
fn test_has_direct_text_whitespace_only() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();
    let text = doc.create_text_node("   ");
    doc.append_child(div, text).unwrap();

    assert!(!has_direct_text(&doc, div), "仅有空白文本的 div 不应被视为有直接文本");
}

/// 测试 has_direct_text：有非空文本的元素返回 true。
#[test]
fn test_has_direct_text_with_content() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();
    let text = doc.create_text_node("Hello");
    doc.append_child(div, text).unwrap();

    assert!(has_direct_text(&doc, div), "有文本内容的 div 应返回 true");
}

// ── measure_text_content 边界条件 ──

/// 测试 measure_text_content：无文本节点时返回 Size::ZERO。
#[test]
fn test_measure_text_content_no_text() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let styles = HashMap::new();
    let size = measure_text_content(
        &doc,
        &styles,
        div,
        taffy::geometry::Size {
            width: None,
            height: None,
        },
        taffy::geometry::Size {
            width: taffy::style::AvailableSpace::Definite(800.0),
            height: taffy::style::AvailableSpace::Definite(600.0),
        },
    );
    assert_eq!(size.width, 0.0, "无文本节点宽度应为 0");
    assert_eq!(size.height, 0.0, "无文本节点高度应为 0");
}

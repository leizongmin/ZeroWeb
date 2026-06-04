//! LayoutEngine compute 补充覆盖测试。
//!
//! 测试 LayoutEngine::compute 方法在不同 DOM 结构和样式配置下的行为，
//! 包括基本布局计算、定位元素、溢出处理和 z-index 提取。

use super::*;
use zero_css_parser::values::*;
use zero_dom::Document;
use zero_style_system::ComputedStyle;

/// 辅助：创建 html > body 并返回 (doc, html_id, body_id)。
fn make_doc() -> (Document, NodeId, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    (doc, html, body)
}

/// 辅助：创建 block 样式。
fn block_style(w: f64, h: f64) -> ComputedStyle {
    let mut s = ComputedStyle::default();
    s.display = DisplayValue::Block;
    if w > 0.0 {
        s.width = LengthValue::Px(w);
    }
    if h > 0.0 {
        s.height = LengthValue::Px(h);
    }
    s
}

// ---- 基本布局计算 ----

#[test]
fn test_compute_empty_document() {
    let doc = Document::new();
    let styles = std::collections::HashMap::new();
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert_eq!(result.viewport_width, 800.0);
    assert_eq!(result.viewport_height, 600.0);
}

#[test]
fn test_compute_simple_block() {
    let (mut doc, _, body) = make_doc();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = std::collections::HashMap::new();
    styles.insert(div, block_style(200.0, 100.0));

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert_eq!(result.root.width, 800.0);
}

#[test]
fn test_compute_two_blocks_stacked() {
    let (mut doc, _, body) = make_doc();
    let div1 = doc.create_element("div");
    let div2 = doc.create_element("div");
    doc.append_child(body, div1).unwrap();
    doc.append_child(body, div2).unwrap();

    let mut styles = std::collections::HashMap::new();
    styles.insert(div1, block_style(100.0, 50.0));
    styles.insert(div2, block_style(100.0, 30.0));

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // 第二个 div 应该在第一个下方
    let body_box = &result.root.children[0];
    assert_eq!(body_box.children.len(), 2);
    assert!(body_box.children[1].y >= body_box.children[0].y + body_box.children[0].height * 0.5);
}

#[test]
fn test_compute_viewport_sizes_preserved() {
    let doc = Document::new();
    let styles = std::collections::HashMap::new();
    let mut engine = crate::engine::LayoutEngine::new(1920.0, 1080.0);
    let result = engine.compute(&doc, &styles);
    assert_eq!(result.viewport_width, 1920.0);
    assert_eq!(result.viewport_height, 1080.0);
}

#[test]
fn test_compute_narrow_viewport() {
    let (mut doc, _, body) = make_doc();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = std::collections::HashMap::new();
    styles.insert(div, block_style(375.0, 100.0));

    let mut engine = crate::engine::LayoutEngine::new(375.0, 812.0);
    let result = engine.compute(&doc, &styles);
    assert_eq!(result.viewport_width, 375.0);
}

// ---- 定位元素测试 ----

#[test]
fn test_compute_absolute_position() {
    let (mut doc, _, body) = make_doc();
    let container = doc.create_element("div");
    let abs_div = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    doc.append_child(container, abs_div).unwrap();

    let mut container_style = block_style(400.0, 300.0);
    container_style.position = PositionValue::Relative;

    let mut abs_style = block_style(100.0, 50.0);
    abs_style.position = PositionValue::Absolute;
    abs_style.left = LengthValue::Px(10.0);
    abs_style.top = LengthValue::Px(20.0);

    let mut styles = std::collections::HashMap::new();
    styles.insert(container, container_style);
    styles.insert(abs_div, abs_style);

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 找到 absolute 定位的盒子
    let container_box = find_child_by_node_id(&result.root, container).expect("container");
    let abs_box = find_child_by_node_id(&result.root, abs_div).expect("abs_div");
    assert!(abs_box.is_absolute);
}

#[test]
fn test_compute_fixed_position() {
    let (mut doc, _, body) = make_doc();
    let fixed_div = doc.create_element("div");
    doc.append_child(body, fixed_div).unwrap();

    let mut style = block_style(100.0, 50.0);
    style.position = PositionValue::Fixed;
    style.left = LengthValue::Px(0.0);
    style.top = LengthValue::Px(0.0);

    let mut styles = std::collections::HashMap::new();
    styles.insert(fixed_div, style);

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let fixed_box = find_child_by_node_id(&result.root, fixed_div).expect("fixed_div");
    assert!(fixed_box.is_fixed);
}

#[test]
fn test_compute_sticky_position() {
    let (mut doc, _, body) = make_doc();
    let sticky_div = doc.create_element("div");
    doc.append_child(body, sticky_div).unwrap();

    let mut style = block_style(100.0, 50.0);
    style.position = PositionValue::Sticky;
    style.top = LengthValue::Px(10.0);

    let mut styles = std::collections::HashMap::new();
    styles.insert(sticky_div, style);

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let sticky_box = find_child_by_node_id(&result.root, sticky_div).expect("sticky_div");
    assert!(sticky_box.is_sticky);
}

// ---- 溢出处理测试 ----

#[test]
fn test_compute_overflow_hidden() {
    let (mut doc, _, body) = make_doc();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut style = block_style(200.0, 100.0);
    style.overflow_x = OverflowValue::Hidden;
    style.overflow_y = OverflowValue::Hidden;

    let mut styles = std::collections::HashMap::new();
    styles.insert(div, style);

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let div_box = find_child_by_node_id(&result.root, div).expect("div");
    assert_eq!(div_box.overflow_x, crate::types::OverflowClip::Hidden);
    assert_eq!(div_box.overflow_y, crate::types::OverflowClip::Hidden);
}

#[test]
fn test_compute_overflow_scroll() {
    let (mut doc, _, body) = make_doc();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut style = block_style(200.0, 100.0);
    style.overflow_x = OverflowValue::Scroll;
    style.overflow_y = OverflowValue::Auto;

    let mut styles = std::collections::HashMap::new();
    styles.insert(div, style);

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let div_box = find_child_by_node_id(&result.root, div).expect("div");
    assert_eq!(div_box.overflow_x, crate::types::OverflowClip::Scroll);
    assert_eq!(div_box.overflow_y, crate::types::OverflowClip::Scroll);
}

#[test]
fn test_compute_overflow_clip() {
    let (mut doc, _, body) = make_doc();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut style = block_style(200.0, 100.0);
    style.overflow_x = OverflowValue::Clip;

    let mut styles = std::collections::HashMap::new();
    styles.insert(div, style);

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let div_box = find_child_by_node_id(&result.root, div).expect("div");
    assert_eq!(div_box.overflow_x, crate::types::OverflowClip::Clip);
}

#[test]
fn test_compute_overflow_visible_default() {
    let (mut doc, _, body) = make_doc();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = std::collections::HashMap::new();
    styles.insert(div, block_style(200.0, 100.0));

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let div_box = find_child_by_node_id(&result.root, div).expect("div");
    assert_eq!(div_box.overflow_x, crate::types::OverflowClip::Visible);
    assert_eq!(div_box.overflow_y, crate::types::OverflowClip::Visible);
}

// ---- z-index 测试 ----

#[test]
fn test_compute_z_index_integer() {
    let (mut doc, _, body) = make_doc();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut style = block_style(100.0, 50.0);
    style.position = PositionValue::Relative;
    style.z_index = ZIndexValue::Integer(10);

    let mut styles = std::collections::HashMap::new();
    styles.insert(div, style);

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let div_box = find_child_by_node_id(&result.root, div).expect("div");
    assert_eq!(div_box.z_index, 10);
}

#[test]
fn test_compute_z_index_auto() {
    let (mut doc, _, body) = make_doc();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut style = block_style(100.0, 50.0);
    style.z_index = ZIndexValue::Auto;

    let mut styles = std::collections::HashMap::new();
    styles.insert(div, style);

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let div_box = find_child_by_node_id(&result.root, div).expect("div");
    assert_eq!(div_box.z_index, 0);
}

#[test]
fn test_compute_z_index_negative() {
    let (mut doc, _, body) = make_doc();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut style = block_style(100.0, 50.0);
    style.position = PositionValue::Relative;
    style.z_index = ZIndexValue::Integer(-5);

    let mut styles = std::collections::HashMap::new();
    styles.insert(div, style);

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let div_box = find_child_by_node_id(&result.root, div).expect("div");
    assert_eq!(div_box.z_index, -5);
}

// ---- 嵌套结构测试 ----

#[test]
fn test_compute_three_levels_nesting() {
    let (mut doc, _, body) = make_doc();
    let outer = doc.create_element("div");
    let inner = doc.create_element("div");
    let deepest = doc.create_element("span");
    doc.append_child(body, outer).unwrap();
    doc.append_child(outer, inner).unwrap();
    doc.append_child(inner, deepest).unwrap();

    let mut styles = std::collections::HashMap::new();
    styles.insert(outer, block_style(400.0, 300.0));
    styles.insert(inner, block_style(200.0, 150.0));
    styles.insert(deepest, block_style(100.0, 50.0));

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 验证三层嵌套存在
    let outer_box = find_child_by_node_id(&result.root, outer).expect("outer");
    assert_eq!(outer_box.children.len(), 1);
    assert_eq!(outer_box.children[0].children.len(), 1);
}

#[test]
fn test_compute_multiple_siblings() {
    let (mut doc, _, body) = make_doc();
    let mut ids = Vec::new();
    for i in 0..5 {
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();
        ids.push(div);

        let mut style = block_style(100.0, 30.0);
        if i % 2 == 0 {
            style.overflow_y = OverflowValue::Hidden;
        }
        // 存入 styles 在下面
    }

    let mut styles = std::collections::HashMap::new();
    for (i, &id) in ids.iter().enumerate() {
        let mut style = block_style(100.0, 30.0);
        if i % 2 == 0 {
            style.overflow_y = OverflowValue::Hidden;
        }
        styles.insert(id, style);
    }

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let body_box = &result.root.children[0];
    assert_eq!(body_box.children.len(), 5);
}

// ---- display:none 测试 ----

#[test]
fn test_compute_display_none_element() {
    let (mut doc, _, body) = make_doc();
    let visible = doc.create_element("div");
    let hidden = doc.create_element("div");
    doc.append_child(body, visible).unwrap();
    doc.append_child(body, hidden).unwrap();

    let mut hidden_style = block_style(100.0, 50.0);
    hidden_style.display = DisplayValue::None;

    let mut styles = std::collections::HashMap::new();
    styles.insert(visible, block_style(100.0, 50.0));
    styles.insert(hidden, hidden_style);

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // hidden 元素不应出现在布局树中
    let hidden_box = find_child_by_node_id(&result.root, hidden);
    // display:none 的元素可能存在但尺寸为 0，或被完全排除
    if let Some(hb) = hidden_box {
        assert_eq!(hb.width, 0.0);
        assert_eq!(hb.height, 0.0);
    }
    // visible 元素应正常布局
    let vis_box = find_child_by_node_id(&result.root, visible).expect("visible");
    assert!(vis_box.width > 0.0);
}

// ---- 无样式元素测试 ----

#[test]
fn test_compute_element_without_style() {
    let (mut doc, _, body) = make_doc();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    // 不给 div 设置样式
    let styles = std::collections::HashMap::new();

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // 不应 panic
    assert_eq!(result.viewport_width, 800.0);
}

// ---- flex 容器测试 ----

#[test]
fn test_compute_flex_container() {
    let (mut doc, _, body) = make_doc();
    let flex = doc.create_element("div");
    let item1 = doc.create_element("div");
    let item2 = doc.create_element("div");
    doc.append_child(body, flex).unwrap();
    doc.append_child(flex, item1).unwrap();
    doc.append_child(flex, item2).unwrap();

    let mut flex_style = block_style(400.0, 100.0);
    flex_style.display = DisplayValue::Flex;
    flex_style.flex_direction = FlexDirectionValue::Row;

    let mut styles = std::collections::HashMap::new();
    styles.insert(flex, flex_style);
    styles.insert(item1, block_style(200.0, 100.0));
    styles.insert(item2, block_style(200.0, 100.0));

    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let flex_box = find_child_by_node_id(&result.root, flex).expect("flex");
    assert_eq!(flex_box.children.len(), 2);
    // flex 行方向：两个 item 应水平排列
    assert!(flex_box.children[1].x > flex_box.children[0].x);
}

use super::*;
use zero_css_parser::values::{
    AlignmentValue, DisplayValue, FlexDirectionValue, FlexWrapValue, LengthValue, OverflowValue, PositionValue,
};
use zero_dom::Document;

/// 创建带指定 display 和 size 的 ComputedStyle。
/// 测试简单 block 布局。
#[test]
fn test_compute_simple_block_layout() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    styles.insert(div, make_style_with_display(DisplayValue::Block, 100.0, 50.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    assert!((result.viewport_width - 800.0).abs() < 0.001);
    assert!((result.viewport_height - 600.0).abs() < 0.001);
}

/// 测试 block 垂直堆叠。
#[test]
fn test_compute_block_vertical_stack() {
    let (mut doc, body) = make_doc_with_body();
    let mut div_ids = Vec::new();
    for _ in 0..3 {
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();
        div_ids.push(div);
    }

    let mut styles = HashMap::new();
    for id in div_ids {
        styles.insert(id, make_style_with_display(DisplayValue::Block, 100.0, 30.0));
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试纯文本 block 元素具有内容高度，兄弟节点不会重叠。
#[test]
fn test_compute_text_blocks_have_nonzero_height() {
    let (mut doc, body) = make_doc_with_body();
    let first = doc.create_element("p");
    let first_text = doc.create_text_node("First paragraph");
    doc.append_child(first, first_text).unwrap();
    doc.append_child(body, first).unwrap();
    let second = doc.create_element("p");
    let second_text = doc.create_text_node("Second paragraph");
    doc.append_child(second, second_text).unwrap();
    doc.append_child(body, second).unwrap();

    let mut styles = HashMap::new();
    styles.insert(first, make_style_with_display(DisplayValue::Block, 0.0, 0.0));
    styles.insert(second, make_style_with_display(DisplayValue::Block, 0.0, 0.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let first_box = find_child_by_node_id(&result.root, first).unwrap();
    let second_box = find_child_by_node_id(&result.root, second).unwrap();

    assert!(first_box.height > 0.0, "first text block should have height");
    assert!(second_box.height > 0.0, "second text block should have height");
    assert!(
        second_box.y >= first_box.y + first_box.height,
        "second text block should be laid out after first: first={first_box:?}, second={second_box:?}"
    );
}

/// 测试 flex row 布局。
#[test]
fn test_compute_flex_row() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    let mut item_ids = Vec::new();
    for _ in 0..3 {
        let item = doc.create_element("span");
        doc.append_child(container, item).unwrap();
        item_ids.push(item);
    }

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_direction = FlexDirectionValue::Row;
    container_style.width = LengthValue::Px(300.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    for id in item_ids {
        let mut item_style = ComputedStyle::default();
        item_style.width = LengthValue::Px(80.0);
        item_style.height = LengthValue::Px(40.0);
        styles.insert(id, item_style);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试 flex column 布局。
#[test]
fn test_compute_flex_column() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(container, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(container, item2).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_direction = FlexDirectionValue::Column;
    container_style.width = LengthValue::Px(200.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    for id in [item1, item2] {
        let mut item_style = ComputedStyle::default();
        item_style.width = LengthValue::Px(100.0);
        item_style.height = LengthValue::Px(50.0);
        styles.insert(id, item_style);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试 flex-grow。
#[test]
fn test_compute_flex_grow() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(container, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(container, item2).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.width = LengthValue::Px(200.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    let mut item1_style = ComputedStyle::default();
    item1_style.flex_grow = 1.0;
    item1_style.height = LengthValue::Px(50.0);
    styles.insert(item1, item1_style);

    let mut item2_style = ComputedStyle::default();
    item2_style.flex_grow = 2.0;
    item2_style.height = LengthValue::Px(50.0);
    styles.insert(item2, item2_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试 flex-wrap。
#[test]
fn test_compute_flex_wrap() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_wrap = FlexWrapValue::Wrap;
    container_style.width = LengthValue::Px(100.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    for _ in 0..5 {
        let item = doc.create_element("span");
        doc.append_child(container, item).unwrap();
        let mut item_style = ComputedStyle::default();
        item_style.width = LengthValue::Px(50.0);
        item_style.height = LengthValue::Px(30.0);
        styles.insert(item, item_style);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试 flex gap。
#[test]
fn test_compute_flex_gap() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(container, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(container, item2).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.gap = LengthValue::Px(10.0);
    container_style.width = LengthValue::Px(200.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    for id in [item1, item2] {
        let mut item_style = ComputedStyle::default();
        item_style.width = LengthValue::Px(50.0);
        item_style.height = LengthValue::Px(50.0);
        styles.insert(id, item_style);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试 flex 居中对齐。
#[test]
fn test_compute_flex_alignment_center() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item = doc.create_element("span");
    doc.append_child(container, item).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.justify_content = AlignmentValue::Center;
    container_style.align_items = AlignmentValue::Center;
    container_style.width = LengthValue::Px(200.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    let mut item_style = ComputedStyle::default();
    item_style.width = LengthValue::Px(50.0);
    item_style.height = LengthValue::Px(50.0);
    styles.insert(item, item_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试 space-between 对齐。
#[test]
fn test_compute_flex_space_between() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(container, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(container, item2).unwrap();
    let item3 = doc.create_element("span");
    doc.append_child(container, item3).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.justify_content = AlignmentValue::SpaceBetween;
    container_style.width = LengthValue::Px(300.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    for id in [item1, item2, item3] {
        let mut item_style = ComputedStyle::default();
        item_style.width = LengthValue::Px(50.0);
        item_style.height = LengthValue::Px(50.0);
        styles.insert(id, item_style);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试 grid 基本布局。
#[test]
fn test_compute_grid_basic() {
    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(grid, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(grid, item2).unwrap();

    let mut styles = HashMap::new();
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.width = LengthValue::Px(200.0);
    grid_style.height = LengthValue::Px(200.0);
    styles.insert(grid, grid_style);

    for id in [item1, item2] {
        styles.insert(id, ComputedStyle::default());
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试 grid 带 template。
#[test]
fn test_compute_grid_with_template() {
    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(grid, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(grid, item2).unwrap();
    let item3 = doc.create_element("span");
    doc.append_child(grid, item3).unwrap();
    let item4 = doc.create_element("span");
    doc.append_child(grid, item4).unwrap();

    let mut styles = HashMap::new();
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.width = LengthValue::Px(200.0);
    grid_style.height = LengthValue::Px(200.0);
    styles.insert(grid, grid_style);

    for id in [item1, item2, item3, item4] {
        styles.insert(id, ComputedStyle::default());
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试绝对定位。
#[test]
fn test_compute_absolute_position() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let abs_child = doc.create_element("span");
    doc.append_child(container, abs_child).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(200.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(10.0);
    abs_style.left = LengthValue::Px(20.0);
    abs_style.width = LengthValue::Px(50.0);
    abs_style.height = LengthValue::Px(50.0);
    styles.insert(abs_child, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试相对定位。
#[test]
fn test_compute_relative_position() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let rel_child = doc.create_element("span");
    doc.append_child(container, rel_child).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.width = LengthValue::Px(200.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    let mut rel_style = ComputedStyle::default();
    rel_style.position = PositionValue::Relative;
    rel_style.top = LengthValue::Px(5.0);
    rel_style.left = LengthValue::Px(5.0);
    rel_style.width = LengthValue::Px(50.0);
    rel_style.height = LengthValue::Px(50.0);
    styles.insert(rel_child, rel_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试 overflow hidden。
#[test]
fn test_compute_overflow_hidden() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.overflow_x = OverflowValue::Hidden;
    container_style.overflow_y = OverflowValue::Scroll;
    container_style.width = LengthValue::Px(100.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试嵌套 flex。
#[test]
fn test_compute_nested_flex() {
    let (mut doc, body) = make_doc_with_body();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(outer, inner).unwrap();
    let item = doc.create_element("span");
    doc.append_child(inner, item).unwrap();

    let mut styles = HashMap::new();
    let mut outer_style = ComputedStyle::default();
    outer_style.display = DisplayValue::Flex;
    outer_style.flex_direction = FlexDirectionValue::Column;
    outer_style.width = LengthValue::Px(200.0);
    outer_style.height = LengthValue::Px(200.0);
    styles.insert(outer, outer_style);

    let mut inner_style = ComputedStyle::default();
    inner_style.display = DisplayValue::Flex;
    inner_style.flex_direction = FlexDirectionValue::Row;
    styles.insert(inner, inner_style);

    let mut item_style = ComputedStyle::default();
    item_style.width = LengthValue::Px(50.0);
    item_style.height = LengthValue::Px(50.0);
    styles.insert(item, item_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试 padding 效果。
#[test]
fn test_compute_padding_effect() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let child = doc.create_element("span");
    doc.append_child(container, child).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(200.0);
    container_style.height = LengthValue::Px(200.0);
    container_style.padding_top = LengthValue::Px(10.0);
    container_style.padding_left = LengthValue::Px(10.0);
    styles.insert(container, container_style);

    let mut child_style = ComputedStyle::default();
    child_style.width = LengthValue::Px(100.0);
    child_style.height = LengthValue::Px(100.0);
    styles.insert(child, child_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试 border 效果。
#[test]
fn test_compute_border_effect() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(200.0);
    container_style.height = LengthValue::Px(200.0);
    container_style.border_top_width = LengthValue::Px(5.0);
    container_style.border_bottom_width = LengthValue::Px(5.0);
    container_style.border_left_width = LengthValue::Px(5.0);
    container_style.border_right_width = LengthValue::Px(5.0);
    styles.insert(container, container_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试 margin 效果。
#[test]
fn test_compute_margin_effect() {
    let (mut doc, body) = make_doc_with_body();
    let child = doc.create_element("div");
    doc.append_child(body, child).unwrap();

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.width = LengthValue::Px(100.0);
    child_style.height = LengthValue::Px(100.0);
    child_style.margin_top = LengthValue::Px(20.0);
    child_style.margin_left = LengthValue::Px(20.0);
    styles.insert(child, child_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试 min/max size。
#[test]
fn test_compute_min_max_size() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.min_width = LengthValue::Px(100.0);
    div_style.max_width = LengthValue::Px(300.0);
    div_style.min_height = LengthValue::Px(50.0);
    div_style.max_height = LengthValue::Px(200.0);
    styles.insert(div, div_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width > 0.0);
}

/// 测试零尺寸元素。
#[test]
fn test_compute_zero_size_element() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    styles.insert(div, ComputedStyle::default());

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // 即使没有显式尺寸，布局也应成功
    assert!(result.root.width >= 0.0);
}

// ── 几何验证补充测试 ──

/// 查找 body 的第一个子元素在布局树中的位置。
/// 验证 block 布局中子元素的正确尺寸和位置。
#[test]
fn test_block_child_exact_geometry() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    styles.insert(div, make_style_with_display(DisplayValue::Block, 200.0, 100.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div_box = find_child_by_node_id(&result.root, div).expect("div found");
    // div 的宽度应该是 200，高度 100
    assert_eq!(div_box.width, 200.0, "div width should be 200px");
    assert_eq!(div_box.height, 100.0, "div height should be 100px");
}

/// 验证 padding 出现在布局盒中（taffy 默认 content-box：padding 增加总尺寸）。
#[test]
fn test_padding_values_in_layout() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.display = DisplayValue::Block;
    div_style.width = LengthValue::Px(200.0);
    div_style.height = LengthValue::Px(100.0);
    div_style.padding_top = LengthValue::Px(10.0);
    div_style.padding_bottom = LengthValue::Px(10.0);
    div_style.padding_left = LengthValue::Px(20.0);
    div_style.padding_right = LengthValue::Px(20.0);
    styles.insert(div, div_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div_box = find_child_by_node_id(&result.root, div).expect("div found");
    assert_eq!(div_box.padding_left, 20.0);
    assert_eq!(div_box.padding_right, 20.0);
    assert_eq!(div_box.padding_top, 10.0);
    assert_eq!(div_box.padding_bottom, 10.0);
    // 总宽度 = width + padding_left + padding_right
    assert_eq!(div_box.width, 240.0, "total width = 200 + 20 + 20");
    // 内容区域 = width（content-box 模式）
    assert_eq!(div_box.content_width, 200.0, "content width = 200 (content-box)");
}

/// 验证 border 正确出现在布局盒中。
#[test]
fn test_border_values_in_layout() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.display = DisplayValue::Block;
    div_style.width = LengthValue::Px(200.0);
    div_style.height = LengthValue::Px(100.0);
    div_style.border_top_width = LengthValue::Px(5.0);
    div_style.border_bottom_width = LengthValue::Px(5.0);
    div_style.border_left_width = LengthValue::Px(10.0);
    div_style.border_right_width = LengthValue::Px(10.0);
    styles.insert(div, div_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div_box = find_child_by_node_id(&result.root, div).expect("div found");
    assert_eq!(div_box.border_top, 5.0);
    assert_eq!(div_box.border_bottom, 5.0);
    assert_eq!(div_box.border_left, 10.0);
    assert_eq!(div_box.border_right, 10.0);
    // 总宽度 = width + border_left + border_right
    assert_eq!(div_box.width, 220.0, "total width = 200 + 10 + 10");
}

/// 验证两个 block 子元素垂直堆叠。
#[test]
fn test_block_stack_y_positions() {
    let (mut doc, body) = make_doc_with_body();
    let div1 = doc.create_element("div");
    doc.append_child(body, div1).unwrap();
    let div2 = doc.create_element("div");
    doc.append_child(body, div2).unwrap();

    let mut styles = HashMap::new();
    styles.insert(div1, make_style_with_display(DisplayValue::Block, 100.0, 50.0));
    styles.insert(div2, make_style_with_display(DisplayValue::Block, 100.0, 50.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let box1 = find_child_by_node_id(&result.root, div1).expect("div1 found");
    let box2 = find_child_by_node_id(&result.root, div2).expect("div2 found");

    // div2 应在 div1 下方
    assert!(
        box2.y >= box1.y + box1.height,
        "div2 (y={}) should be below div1 (y={}, h={})",
        box2.y,
        box1.y,
        box1.height
    );
}

/// 验证 flex 行中子元素水平排列。
#[test]
fn test_flex_row_children_horizontal() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(container, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(container, item2).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);
    styles.insert(item1, make_style_with_display(DisplayValue::Block, 100.0, 50.0));
    styles.insert(item2, make_style_with_display(DisplayValue::Block, 100.0, 50.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let box1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let box2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

    // item2 应在 item1 右侧
    assert!(
        box2.x > box1.x,
        "item2 (x={}) should be right of item1 (x={})",
        box2.x,
        box1.x
    );
}

/// 验证 overflow 属性正确传递。
#[test]
fn test_overflow_values_propagated() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.display = DisplayValue::Block;
    div_style.overflow_x = OverflowValue::Hidden;
    div_style.overflow_y = OverflowValue::Scroll;
    div_style.width = LengthValue::Px(100.0);
    styles.insert(div, div_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div_box = find_child_by_node_id(&result.root, div).expect("div found");
    assert_eq!(div_box.overflow_x, OverflowClip::Hidden);
    assert_eq!(div_box.overflow_y, OverflowClip::Scroll);
}

/// 验证空 DOM 文档布局不 panic。
#[test]
fn test_layout_empty_document() {
    let doc = Document::new();
    let styles = HashMap::new();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(result.root.width >= 0.0);
}

/// 验证绝对定位元素标记正确。
#[test]
fn test_absolute_position_flag() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let abs_child = doc.create_element("span");
    doc.append_child(container, abs_child).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(200.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.width = LengthValue::Px(50.0);
    abs_style.height = LengthValue::Px(50.0);
    styles.insert(abs_child, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let abs_box = find_child_by_node_id(&result.root, abs_child).expect("abs found");
    assert!(abs_box.is_absolute, "should be flagged as absolute");
    assert_eq!(abs_box.width, 50.0);
    assert_eq!(abs_box.height, 50.0);
}

// ── 新增集成测试 ──

/// 测试 block 布局中嵌套元素的几何位置正确。
///
/// 结构：body > div(200x300) > div(100x150)
/// 内部 div 应在外部 div 的内容区域中定位。
#[test]
fn test_block_nested_element_geometry() {
    let (mut doc, body) = make_doc_with_body();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(outer, inner).unwrap();

    let mut styles = HashMap::new();
    styles.insert(outer, make_style_with_display(DisplayValue::Block, 200.0, 300.0));
    styles.insert(inner, make_style_with_display(DisplayValue::Block, 100.0, 150.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let outer_box = find_child_by_node_id(&result.root, outer).expect("outer found");
    let inner_box = find_child_by_node_id(&result.root, inner).expect("inner found");

    assert_eq!(outer_box.width, 200.0, "外层 div 宽度应为 200");
    assert_eq!(outer_box.height, 300.0, "外层 div 高度应为 300");
    assert_eq!(inner_box.width, 100.0, "内层 div 宽度应为 100");
    assert_eq!(inner_box.height, 150.0, "内层 div 高度应为 150");

    // 内层 div 应在外层 div 内部
    assert!(inner_box.x >= outer_box.content_x, "内层 x 应 >= 外层内容区域 x");
}

/// 测试三层嵌套 block 布局。
///
/// body > div > div > div，每层尺寸递减。
#[test]
fn test_block_deep_nesting() {
    let (mut doc, body) = make_doc_with_body();
    let d1 = doc.create_element("div");
    doc.append_child(body, d1).unwrap();
    let d2 = doc.create_element("div");
    doc.append_child(d1, d2).unwrap();
    let d3 = doc.create_element("div");
    doc.append_child(d2, d3).unwrap();

    let mut styles = HashMap::new();
    styles.insert(d1, make_style_with_display(DisplayValue::Block, 600.0, 400.0));
    styles.insert(d2, make_style_with_display(DisplayValue::Block, 400.0, 200.0));
    styles.insert(d3, make_style_with_display(DisplayValue::Block, 200.0, 100.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, d1).expect("d1 found");
    let b2 = find_child_by_node_id(&result.root, d2).expect("d2 found");
    let b3 = find_child_by_node_id(&result.root, d3).expect("d3 found");

    assert_eq!(b1.width, 600.0);
    assert_eq!(b2.width, 400.0);
    assert_eq!(b3.width, 200.0);
    assert_eq!(b3.height, 100.0);
}

/// 测试 block 布局中多个子元素垂直堆叠，间距精确。
#[test]
fn test_block_stack_with_margin() {
    let (mut doc, body) = make_doc_with_body();
    let div1 = doc.create_element("div");
    doc.append_child(body, div1).unwrap();
    let div2 = doc.create_element("div");
    doc.append_child(body, div2).unwrap();

    let mut styles = HashMap::new();
    let mut style1 = make_style_with_display(DisplayValue::Block, 100.0, 50.0);
    style1.margin_bottom = LengthValue::Px(20.0);
    styles.insert(div1, style1);
    styles.insert(div2, make_style_with_display(DisplayValue::Block, 100.0, 50.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let box1 = find_child_by_node_id(&result.root, div1).expect("div1 found");
    let box2 = find_child_by_node_id(&result.root, div2).expect("div2 found");

    // div2 应在 div1 底部 + margin_bottom 之后
    let expected_y = box1.y + box1.height + box1.margin_bottom;
    assert!(
        (box2.y - expected_y).abs() < 0.01,
        "div2.y ({}) 应等于 div1.y({}) + div1.height({}) + margin_bottom({}) = {}",
        box2.y,
        box1.y,
        box1.height,
        box1.margin_bottom,
        expected_y
    );
}

/// 测试 flex-direction: row — 子元素水平排列。
#[test]
fn test_flex_row_direction_layout() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(container, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(container, item2).unwrap();
    let item3 = doc.create_element("span");
    doc.append_child(container, item3).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_direction = FlexDirectionValue::Row;
    container_style.width = LengthValue::Px(300.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    for id in [item1, item2, item3] {
        styles.insert(id, make_style_with_display(DisplayValue::Block, 80.0, 40.0));
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");
    let b3 = find_child_by_node_id(&result.root, item3).expect("item3 found");

    // Row 方向：三个元素应水平排列，x 递增
    assert!(b2.x > b1.x, "item2 应在 item1 右侧");
    assert!(b3.x > b2.x, "item3 应在 item2 右侧");

    // y 应相同（同一行）
    assert!(
        (b1.y - b2.y).abs() < 0.01 && (b2.y - b3.y).abs() < 0.01,
        "三个元素应在同一行"
    );
}

/// 测试 flex-direction: column — 子元素垂直排列。
#[test]
fn test_flex_column_direction_layout() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(container, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(container, item2).unwrap();
    let item3 = doc.create_element("span");
    doc.append_child(container, item3).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_direction = FlexDirectionValue::Column;
    container_style.width = LengthValue::Px(300.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    for id in [item1, item2, item3] {
        styles.insert(id, make_style_with_display(DisplayValue::Block, 80.0, 40.0));
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");
    let b3 = find_child_by_node_id(&result.root, item3).expect("item3 found");

    // Column 方向：三个元素应垂直排列，y 递增
    assert!(b2.y > b1.y, "item2 应在 item1 下方");
    assert!(b3.y > b2.y, "item3 应在 item2 下方");

    // x 应相同（同一列）
    assert!(
        (b1.x - b2.x).abs() < 0.01 && (b2.x - b3.x).abs() < 0.01,
        "三个元素应在同一列"
    );
}

/// 测试 flex-direction: row-reverse — 子元素反向水平排列。
#[test]
fn test_flex_row_reverse_direction() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(container, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(container, item2).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_direction = FlexDirectionValue::RowReverse;
    container_style.width = LengthValue::Px(300.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    for id in [item1, item2] {
        styles.insert(id, make_style_with_display(DisplayValue::Block, 80.0, 40.0));
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

    // Row-reverse：item1 在右，item2 在左
    assert!(b2.x < b1.x, "row-reverse 中 item2 应在 item1 左侧（x 更小）");
}

/// 测试 flex-direction: column-reverse — 子元素反向垂直排列。
#[test]
fn test_flex_column_reverse_direction() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(container, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(container, item2).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_direction = FlexDirectionValue::ColumnReverse;
    container_style.width = LengthValue::Px(200.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    for id in [item1, item2] {
        styles.insert(id, make_style_with_display(DisplayValue::Block, 80.0, 40.0));
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

    // Column-reverse：item1 在下方，item2 在上方
    assert!(b2.y < b1.y, "column-reverse 中 item2 应在 item1 上方（y 更小）");
}

/// 测试 Grid 布局中显式的行/列放置。
///
/// 2x2 grid，显式指定每个子元素的 grid-row/grid-column。
#[test]
fn test_grid_explicit_placement() {
    use zero_style_system::GridLineValue;

    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let item1 = doc.create_element("span");
    doc.append_child(grid, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(grid, item2).unwrap();
    let item3 = doc.create_element("span");
    doc.append_child(grid, item3).unwrap();
    let item4 = doc.create_element("span");
    doc.append_child(grid, item4).unwrap();

    let mut styles = HashMap::new();

    // 2 列 2 行的 grid
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px 100px".to_string());
    grid_style.grid_template_rows = Some("50px 50px".to_string());
    grid_style.width = LengthValue::Px(200.0);
    grid_style.height = LengthValue::Px(100.0);
    styles.insert(grid, grid_style);

    // item1: row 1, col 1
    let mut item1_style = ComputedStyle::default();
    item1_style.grid_row_start = GridLineValue::Line(1);
    item1_style.grid_row_end = GridLineValue::Line(2);
    item1_style.grid_column_start = GridLineValue::Line(1);
    item1_style.grid_column_end = GridLineValue::Line(2);
    styles.insert(item1, item1_style);

    // item2: row 1, col 2
    let mut item2_style = ComputedStyle::default();
    item2_style.grid_row_start = GridLineValue::Line(1);
    item2_style.grid_row_end = GridLineValue::Line(2);
    item2_style.grid_column_start = GridLineValue::Line(2);
    item2_style.grid_column_end = GridLineValue::Line(3);
    styles.insert(item2, item2_style);

    // item3: row 2, col 1
    let mut item3_style = ComputedStyle::default();
    item3_style.grid_row_start = GridLineValue::Line(2);
    item3_style.grid_row_end = GridLineValue::Line(3);
    item3_style.grid_column_start = GridLineValue::Line(1);
    item3_style.grid_column_end = GridLineValue::Line(2);
    styles.insert(item3, item3_style);

    // item4: row 2, col 2
    let mut item4_style = ComputedStyle::default();
    item4_style.grid_row_start = GridLineValue::Line(2);
    item4_style.grid_row_end = GridLineValue::Line(3);
    item4_style.grid_column_start = GridLineValue::Line(2);
    item4_style.grid_column_end = GridLineValue::Line(3);
    styles.insert(item4, item4_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");
    let b3 = find_child_by_node_id(&result.root, item3).expect("item3 found");
    let b4 = find_child_by_node_id(&result.root, item4).expect("item4 found");

    // item1 (0,0) vs item2 (0,1): item2 应在 item1 右侧
    assert!(
        b2.x > b1.x,
        "item2 (col 2) 应在 item1 (col 1) 右侧: {} vs {}",
        b2.x,
        b1.x
    );

    // item1 (0,0) vs item3 (1,0): item3 应在 item1 下方
    assert!(
        b3.y > b1.y,
        "item3 (row 2) 应在 item1 (row 1) 下方: {} vs {}",
        b3.y,
        b1.y
    );

    // item4 (1,1) 应在 item3 (1,0) 右侧
    assert!(
        b4.x > b3.x,
        "item4 (col 2) 应在 item3 (col 1) 右侧: {} vs {}",
        b4.x,
        b3.x
    );

    // 所有格子宽度应约 100px
    assert!(
        (b1.width - 100.0).abs() < 1.0,
        "item1 宽度应约 100px，实际 {}",
        b1.width
    );
    assert!(
        (b4.width - 100.0).abs() < 1.0,
        "item4 宽度应约 100px，实际 {}",
        b4.width
    );
}

/// 测试 Grid 布局中 span 放置。
#[test]
fn test_grid_span_placement() {
    use zero_style_system::GridLineValue;

    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let wide_item = doc.create_element("span");
    doc.append_child(grid, wide_item).unwrap();
    let normal_item = doc.create_element("span");
    doc.append_child(grid, normal_item).unwrap();

    let mut styles = HashMap::new();

    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px 100px 100px".to_string());
    grid_style.grid_template_rows = Some("50px".to_string());
    grid_style.width = LengthValue::Px(300.0);
    grid_style.height = LengthValue::Px(50.0);
    styles.insert(grid, grid_style);

    // wide_item: 跨两列
    let mut wide_style = ComputedStyle::default();
    wide_style.grid_column_start = GridLineValue::Line(1);
    wide_style.grid_column_end = GridLineValue::Span(2);
    wide_style.grid_row_start = GridLineValue::Line(1);
    wide_style.grid_row_end = GridLineValue::Line(2);
    styles.insert(wide_item, wide_style);

    // normal_item: 一列
    let mut normal_style = ComputedStyle::default();
    normal_style.grid_column_start = GridLineValue::Line(3);
    normal_style.grid_column_end = GridLineValue::Line(4);
    normal_style.grid_row_start = GridLineValue::Line(1);
    normal_style.grid_row_end = GridLineValue::Line(2);
    styles.insert(normal_item, normal_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let wide_box = find_child_by_node_id(&result.root, wide_item).expect("wide found");
    let normal_box = find_child_by_node_id(&result.root, normal_item).expect("normal found");

    // 宽元素应跨两列（约 200px）
    assert!(
        wide_box.width > normal_box.width,
        "跨两列元素应比单列元素宽: {} vs {}",
        wide_box.width,
        normal_box.width
    );
    assert!(
        (wide_box.width - 200.0).abs() < 1.0,
        "跨两列宽度应约 200px，实际 {}",
        wide_box.width
    );
    assert!(
        (normal_box.width - 100.0).abs() < 1.0,
        "单列宽度应约 100px，实际 {}",
        normal_box.width
    );

    // 两个元素应在同一行
    assert!((wide_box.y - normal_box.y).abs() < 0.01, "同行元素 y 应相同");
}

/// 测试 Grid 布局中 fr 单位轨道。
#[test]
fn test_grid_fr_tracks() {
    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let item1 = doc.create_element("span");
    doc.append_child(grid, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(grid, item2).unwrap();

    let mut styles = HashMap::new();

    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("1fr 2fr".to_string());
    grid_style.grid_template_rows = Some("100px".to_string());
    grid_style.width = LengthValue::Px(300.0);
    grid_style.height = LengthValue::Px(100.0);
    styles.insert(grid, grid_style);

    styles.insert(item1, ComputedStyle::default());
    styles.insert(item2, ComputedStyle::default());

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

    // 1fr : 2fr = 100px : 200px
    assert!((b1.width - 100.0).abs() < 1.0, "1fr 应约 100px，实际 {}", b1.width);
    assert!((b2.width - 200.0).abs() < 1.0, "2fr 应约 200px，实际 {}", b2.width);
}

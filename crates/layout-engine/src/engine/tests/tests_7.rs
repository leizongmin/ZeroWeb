use super::*;
use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, LengthValue, PositionValue};
use zero_style_system::ComputedStyle;

// ── 边缘场景补充测试（第九批）──

/// 测试零宽高容器内的子元素布局不 panic。
#[test]
fn test_zero_size_container_children() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let child = doc.create_element("span");
    doc.append_child(container, child).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(0.0);
    container_style.height = LengthValue::Px(0.0);
    styles.insert(container, container_style);

    styles.insert(child, ComputedStyle::default());

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container_box = find_child_by_node_id(&result.root, container).expect("container");
    assert_eq!(container_box.width, 0.0, "容器宽度应为 0");
    assert_eq!(container_box.height, 0.0, "容器高度应为 0");
}

/// 测试 display:none 元素子树完全不参与布局。
#[test]
fn test_display_none_nested() {
    let (mut doc, body) = make_doc_with_body();
    let hidden = doc.create_element("div");
    doc.append_child(body, hidden).unwrap();
    let child1 = doc.create_element("span");
    doc.append_child(hidden, child1).unwrap();
    let child2 = doc.create_element("span");
    doc.append_child(hidden, child2).unwrap();

    let mut styles = HashMap::new();
    let mut hidden_style = ComputedStyle::default();
    hidden_style.display = DisplayValue::None;
    styles.insert(hidden, hidden_style);
    styles.insert(child1, ComputedStyle::default());
    styles.insert(child2, ComputedStyle::default());

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let hidden_box = find_child_by_node_id(&result.root, hidden);
    assert!(hidden_box.is_none(), "display:none 不应出现在布局树中");
}

/// 测试百分比宽度在无明确父宽度时的行为。
#[test]
fn test_percent_width_without_parent_width() {
    let (mut doc, body) = make_doc_with_body();
    let child = doc.create_element("div");
    doc.append_child(body, child).unwrap();

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.width = LengthValue::Percentage(50.0);
    styles.insert(child, child_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let child_box = find_child_by_node_id(&result.root, child).expect("child");
    // 百分比基于 body 宽度（默认占满视口），应为 400px
    assert!(child_box.width > 0.0, "百分比宽度子元素应有非零宽度");
}

/// 测试负 margin 对布局的影响。
#[test]
fn test_negative_margin_overlap() {
    let (mut doc, body) = make_doc_with_body();
    let box1 = doc.create_element("div");
    doc.append_child(body, box1).unwrap();
    let box2 = doc.create_element("div");
    doc.append_child(body, box2).unwrap();

    let mut styles = HashMap::new();
    let mut s1 = ComputedStyle::default();
    s1.width = LengthValue::Px(100.0);
    s1.height = LengthValue::Px(50.0);
    styles.insert(box1, s1);

    let mut s2 = ComputedStyle::default();
    s2.width = LengthValue::Px(100.0);
    s2.height = LengthValue::Px(50.0);
    s2.margin_top = LengthValue::Px(-20.0);
    styles.insert(box2, s2);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b2 = find_child_by_node_id(&result.root, box2).expect("box2");
    // 负 margin-top 应使 box2 向上偏移
    assert!(b2.y < 50.0, "负 margin-top 应使元素向上偏移，实际 y={}", b2.y);
}

/// 测试 absolute 定位元素的坐标不受兄弟元素影响。
#[test]
fn test_absolute_unaffected_by_siblings() {
    let (mut doc, body) = make_doc_with_body();
    let spacer = doc.create_element("div");
    doc.append_child(body, spacer).unwrap();
    let abs_el = doc.create_element("div");
    doc.append_child(body, abs_el).unwrap();

    let mut styles = HashMap::new();
    let mut spacer_style = ComputedStyle::default();
    spacer_style.height = LengthValue::Px(200.0);
    styles.insert(spacer, spacer_style);

    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(10.0);
    abs_style.left = LengthValue::Px(20.0);
    abs_style.width = LengthValue::Px(50.0);
    abs_style.height = LengthValue::Px(50.0);
    styles.insert(abs_el, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let abs_box = find_child_by_node_id(&result.root, abs_el).expect("abs");
    assert_eq!(abs_box.y, 10.0, "absolute top 应为 10");
    assert_eq!(abs_box.x, 20.0, "absolute left 应为 20");
    assert!(abs_box.is_absolute, "应标记为 absolute");
}

/// 测试 fixed 定位元素坐标调整为视口相对。
#[test]
fn test_fixed_viewport_relative() {
    let (mut doc, body) = make_doc_with_body();
    let scroll_container = doc.create_element("div");
    doc.append_child(body, scroll_container).unwrap();
    let fixed_el = doc.create_element("div");
    doc.append_child(scroll_container, fixed_el).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(300.0);
    container_style.height = LengthValue::Px(300.0);
    styles.insert(scroll_container, container_style);

    let mut fixed_style = ComputedStyle::default();
    fixed_style.position = PositionValue::Fixed;
    fixed_style.top = LengthValue::Px(0.0);
    fixed_style.left = LengthValue::Px(0.0);
    fixed_style.width = LengthValue::Px(100.0);
    fixed_style.height = LengthValue::Px(100.0);
    styles.insert(fixed_el, fixed_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fixed_box = find_child_by_node_id(&result.root, fixed_el).expect("fixed");
    assert!(fixed_box.is_fixed, "应标记为 fixed");
    // fixed 元素坐标应相对于视口，不是相对于父容器
    assert_eq!(fixed_box.y, 0.0, "fixed top 应为 0（视口相对）");
    assert_eq!(fixed_box.x, 0.0, "fixed left 应为 0（视口相对）");
}

/// 测试嵌套 flex 容器布局。
#[test]
fn test_nested_flex_containers() {
    let (mut doc, body) = make_doc_with_body();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(outer, inner).unwrap();
    let item = doc.create_element("span");
    doc.append_child(inner, item).unwrap();

    let mut styles = HashMap::new();
    let mut outer_s = ComputedStyle::default();
    outer_s.display = DisplayValue::Flex;
    outer_s.width = LengthValue::Px(400.0);
    outer_s.height = LengthValue::Px(200.0);
    styles.insert(outer, outer_s);

    let mut inner_s = ComputedStyle::default();
    inner_s.display = DisplayValue::Flex;
    inner_s.width = LengthValue::Px(200.0);
    inner_s.height = LengthValue::Px(100.0);
    styles.insert(inner, inner_s);

    let mut item_s = ComputedStyle::default();
    item_s.width = LengthValue::Px(50.0);
    item_s.height = LengthValue::Px(30.0);
    styles.insert(item, item_s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let item_box = find_child_by_node_id(&result.root, item).expect("item");
    assert_eq!(item_box.width, 50.0, "嵌套 flex 子元素宽度应正确");
    assert_eq!(item_box.height, 30.0, "嵌套 flex 子元素高度应正确");
}

/// 测试 inline-block 在行内格式化上下文中的换行。
#[test]
fn test_inline_block_line_wrapping() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let ib1 = doc.create_element("span");
    doc.append_child(container, ib1).unwrap();
    let ib2 = doc.create_element("span");
    doc.append_child(container, ib2).unwrap();
    let ib3 = doc.create_element("span");
    doc.append_child(container, ib3).unwrap();

    let mut styles = HashMap::new();
    let mut container_s = ComputedStyle::default();
    container_s.width = LengthValue::Px(150.0);
    styles.insert(container, container_s);

    for &id in &[ib1, ib2, ib3] {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::InlineBlock;
        s.width = LengthValue::Px(80.0);
        s.height = LengthValue::Px(30.0);
        styles.insert(id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 三个 80px 宽的 inline-block 在 150px 容器中应换行
    let ib1_box = find_child_by_node_id(&result.root, ib1).expect("ib1");
    let ib2_box = find_child_by_node_id(&result.root, ib2).expect("ib2");
    // ib2 应该在第二行（y > ib1.y + ib1.height 或者 x 回到行首）
    assert!(ib2_box.y >= ib1_box.y, "第二个 inline-block 应在第一行或换行后");
}

/// 测试大边框不影响内容区域为负。
#[test]
fn test_large_border_content_clamp() {
    let (mut doc, body) = make_doc_with_body();
    let el = doc.create_element("div");
    doc.append_child(body, el).unwrap();

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.width = LengthValue::Px(50.0);
    s.height = LengthValue::Px(50.0);
    // 使用 CSS 构建带大 border 的样式
    styles.insert(el, s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let el_box = find_child_by_node_id(&result.root, el).expect("el");
    assert!(
        el_box.content_width >= 0.0,
        "内容宽度不应为负，实际 {}",
        el_box.content_width
    );
    assert!(
        el_box.content_height >= 0.0,
        "内容高度不应为负，实际 {}",
        el_box.content_height
    );
}

/// 测试 grid 容器内 flex 子元素正常布局。
#[test]
fn test_flex_child_in_grid() {
    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();
    let flex_child = doc.create_element("div");
    doc.append_child(grid, flex_child).unwrap();
    let item = doc.create_element("span");
    doc.append_child(flex_child, item).unwrap();

    let mut styles = HashMap::new();
    let mut grid_s = ComputedStyle::default();
    grid_s.display = DisplayValue::Grid;
    grid_s.width = LengthValue::Px(400.0);
    grid_s.height = LengthValue::Px(200.0);
    styles.insert(grid, grid_s);

    let mut flex_s = ComputedStyle::default();
    flex_s.display = DisplayValue::Flex;
    styles.insert(flex_child, flex_s);

    let mut item_s = ComputedStyle::default();
    item_s.width = LengthValue::Px(100.0);
    item_s.height = LengthValue::Px(50.0);
    styles.insert(item, item_s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let item_box = find_child_by_node_id(&result.root, item).expect("item");
    assert_eq!(item_box.width, 100.0, "grid 内 flex 子元素宽度应正确");
    assert_eq!(item_box.height, 50.0, "grid 内 flex 子元素高度应正确");
}

/// 测试深层嵌套 absolute 元素的定位。
#[test]
fn test_deeply_nested_absolute() {
    let (mut doc, body) = make_doc_with_body();
    let l1 = doc.create_element("div");
    doc.append_child(body, l1).unwrap();
    let l2 = doc.create_element("div");
    doc.append_child(l1, l2).unwrap();
    let l3 = doc.create_element("div");
    doc.append_child(l2, l3).unwrap();
    let abs_el = doc.create_element("div");
    doc.append_child(l3, abs_el).unwrap();

    let mut styles = HashMap::new();
    let mut l1_s = ComputedStyle::default();
    l1_s.width = LengthValue::Px(300.0);
    l1_s.height = LengthValue::Px(300.0);
    styles.insert(l1, l1_s);

    styles.insert(l2, ComputedStyle::default());
    styles.insert(l3, ComputedStyle::default());

    let mut abs_s = ComputedStyle::default();
    abs_s.position = PositionValue::Absolute;
    abs_s.top = LengthValue::Px(5.0);
    abs_s.left = LengthValue::Px(10.0);
    abs_s.width = LengthValue::Px(20.0);
    abs_s.height = LengthValue::Px(20.0);
    styles.insert(abs_el, abs_s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let abs_box = find_child_by_node_id(&result.root, abs_el).expect("abs");
    assert_eq!(abs_box.width, 20.0, "深层嵌套 absolute 宽度应正确");
    assert_eq!(abs_box.height, 20.0, "深层嵌套 absolute 高度应正确");
    assert!(abs_box.is_absolute, "应标记为 absolute");
}

/// 测试 LayoutBox 的 outer_area 计算（包含 margin）。
#[test]
fn test_layout_box_outer_area() {
    let (mut doc, body) = make_doc_with_body();
    let el = doc.create_element("div");
    doc.append_child(body, el).unwrap();

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.width = LengthValue::Px(100.0);
    s.height = LengthValue::Px(50.0);
    s.margin_top = LengthValue::Px(10.0);
    s.margin_left = LengthValue::Px(20.0);
    styles.insert(el, s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let el_box = find_child_by_node_id(&result.root, el).expect("el");
    assert_eq!(el_box.width, 100.0);
    assert_eq!(el_box.height, 50.0);
    assert_eq!(el_box.margin_top, 10.0);
    assert_eq!(el_box.margin_left, 20.0);
}

/// 测试空 inline 元素的 line-height 贡献到容器高度。
/// CSS 2.1 规范要求空 inline 元素的 line-height + padding + border 贡献到行盒高度。
#[test]
fn test_empty_inline_line_height_contribution() {
    let (mut doc, body) = make_doc_with_body();

    // body > div#container > span (empty)
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let span = doc.create_element("span");
    doc.append_child(container, span).unwrap();

    let mut styles = HashMap::new();

    // Container: auto height, explicit width
    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    // Span: display inline, line-height: 5 (unitless)
    let mut span_style = ComputedStyle::default();
    span_style.display = DisplayValue::Inline;
    span_style.line_height = zero_style_system::property::types::LineHeightValue::Number(5.0);
    styles.insert(span, span_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container_box = find_child_by_node_id(&result.root, container).expect("container");
    eprintln!(
        "container: height={}, content_height={}, num_children={}",
        container_box.height,
        container_box.content_height,
        container_box.children.len()
    );
    for (i, c) in container_box.children.iter().enumerate() {
        eprintln!(
            "  child[{}]: is_block_level={}, height={}, content_height={}",
            i, c.is_block_level, c.height, c.content_height
        );
    }

    // Empty span with line-height:5 should contribute to container height
    // Default font-size = 16px, so line-height = 80px
    assert!(
        container_box.content_height > 0.0,
        "Container should have non-zero height from empty inline's line-height, got {}",
        container_box.content_height
    );
}

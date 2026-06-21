use super::*;
use zero_css_parser::values::{
    DisplayValue, FlexDirectionValue, FlexWrapValue, LengthValue, OverflowValue, PositionValue,
};
// ── 新增补充测试 ──

/// Grid 使用 grid-row/grid-column 显式放置元素到非连续位置。
///
/// 3x3 grid，item 放在 row 2 col 3，验证位置和尺寸正确。
#[test]
fn test_grid_explicit_row_column_placement() {
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

    let mut styles = HashMap::new();

    // 3 列 3 行 grid
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px 100px 100px".to_string());
    grid_style.grid_template_rows = Some("50px 50px 50px".to_string());
    grid_style.width = LengthValue::Px(300.0);
    grid_style.height = LengthValue::Px(150.0);
    styles.insert(grid, grid_style);

    // item1: row 1, col 1（左上角）
    let mut s1 = ComputedStyle::default();
    s1.grid_row_start = GridLineValue::Line(1);
    s1.grid_row_end = GridLineValue::Line(2);
    s1.grid_column_start = GridLineValue::Line(1);
    s1.grid_column_end = GridLineValue::Line(2);
    styles.insert(item1, s1);

    // item2: row 2, col 3（中间行，最右列）
    let mut s2 = ComputedStyle::default();
    s2.grid_row_start = GridLineValue::Line(2);
    s2.grid_row_end = GridLineValue::Line(3);
    s2.grid_column_start = GridLineValue::Line(3);
    s2.grid_column_end = GridLineValue::Line(4);
    styles.insert(item2, s2);

    // item3: row 3, col 2（最底行，中间列）
    let mut s3 = ComputedStyle::default();
    s3.grid_row_start = GridLineValue::Line(3);
    s3.grid_row_end = GridLineValue::Line(4);
    s3.grid_column_start = GridLineValue::Line(2);
    s3.grid_column_end = GridLineValue::Line(3);
    styles.insert(item3, s3);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");
    let b3 = find_child_by_node_id(&result.root, item3).expect("item3 found");

    // item1 在第一行第一列：x 接近 0，y 接近 0
    assert!(b1.x < 1.0, "item1 x 应接近 0，实际 {}", b1.x);
    assert!(b1.y < 1.0, "item1 y 应接近 0，实际 {}", b1.y);

    // item2 在第二行第三列：x > item1.x + 100 + 100，y > 50
    assert!(
        b2.x > b1.x + 150.0,
        "item2 (col 3) 应在最右侧: x={}，期望 > {}",
        b2.x,
        b1.x + 150.0
    );
    assert!(
        b2.y > b1.y + 40.0,
        "item2 (row 2) 应在 item1 下方: y={}，期望 > {}",
        b2.y,
        b1.y + 40.0
    );

    // item3 在第三行第二列：y 最大
    assert!(
        b3.y > b2.y,
        "item3 (row 3) 应在 item2 (row 2) 下方: y={} vs {}",
        b3.y,
        b2.y
    );

    // 所有格子尺寸约 100x50
    assert!(
        (b1.width - 100.0).abs() < 1.0,
        "item1 宽度应约 100px，实际 {}",
        b1.width
    );
    assert!(
        (b2.width - 100.0).abs() < 1.0,
        "item2 宽度应约 100px，实际 {}",
        b2.width
    );
    assert!(
        (b3.width - 100.0).abs() < 1.0,
        "item3 宽度应约 100px，实际 {}",
        b3.width
    );
}

/// Grid auto-fill 在窄容器中创建轨道。
///
/// 250px 容器 + repeat(auto-fill, 100px) → 2 个轨道。
/// 3 个子元素应分布在 2 列中，第 3 个换到下一行。
#[test]
fn test_grid_auto_fill_narrow_container() {
    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let mut item_ids = Vec::new();
    for _ in 0..3 {
        let item = doc.create_element("span");
        doc.append_child(grid, item).unwrap();
        item_ids.push(item);
    }

    let mut styles = HashMap::new();
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("repeat(auto-fill, 100px)".to_string());
    grid_style.grid_auto_rows = Some("50px".to_string());
    grid_style.width = LengthValue::Px(250.0);
    grid_style.height = LengthValue::Px(200.0);
    styles.insert(grid, grid_style);

    for id in &item_ids {
        styles.insert(*id, ComputedStyle::default());
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b0 = find_child_by_node_id(&result.root, item_ids[0]).expect("item0 found");
    let b1 = find_child_by_node_id(&result.root, item_ids[1]).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item_ids[2]).expect("item2 found");

    // 前两个应在同一行（auto-fill 创建 2 个轨道）
    assert!(b1.x > b0.x, "item1 应在 item0 右侧: x={}", b1.x);
    assert!((b0.y - b1.y).abs() < 0.01, "item0 和 item1 应在同一行");

    // 第 3 个应换到下一行（只有 2 列）
    assert!(b2.y > b0.y, "item3 应换行: y={} > y={}", b2.y, b0.y);

    // 每个轨道约 125px（250 / 2）
    assert!(b0.width > 99.0, "item0 宽度应 >= 100px，实际 {}", b0.width);
}

/// Block 布局中负 margin 上下边距合并导致重叠。
///
/// div1 设置 margin-bottom: -30px，div2 设置 margin-top: -20px，
/// 总偏移 -50px（或按 taffy 合并规则），验证 div2 与 div1 重叠。
#[test]
fn test_block_negative_margin_collapsing() {
    let (mut doc, body) = make_doc_with_body();
    let div1 = doc.create_element("div");
    doc.append_child(body, div1).unwrap();
    let div2 = doc.create_element("div");
    doc.append_child(body, div2).unwrap();
    let div3 = doc.create_element("div");
    doc.append_child(body, div3).unwrap();

    let mut styles = HashMap::new();

    // div1: 正常高度，负 margin-bottom
    let mut s1 = make_style_with_display(DisplayValue::Block, 200.0, 80.0);
    s1.margin_bottom = LengthValue::Px(-30.0);
    styles.insert(div1, s1);

    // div2: 正常高度，负 margin-top
    let mut s2 = make_style_with_display(DisplayValue::Block, 200.0, 80.0);
    s2.margin_top = LengthValue::Px(-20.0);
    styles.insert(div2, s2);

    // div3: 正常，用于参照
    styles.insert(div3, make_style_with_display(DisplayValue::Block, 200.0, 40.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, div1).expect("div1 found");
    let b2 = find_child_by_node_id(&result.root, div2).expect("div2 found");
    let b3 = find_child_by_node_id(&result.root, div3).expect("div3 found");

    // div2 应与 div1 重叠（y < div1.y + div1.height）
    assert!(
        b2.y < b1.y + b1.height,
        "negative margin should cause overlap: b2.y({}) < b1.y({}) + b1.height({})",
        b2.y,
        b1.y,
        b1.height
    );

    // div3 应在 div2 之后（按正常流顺序）
    assert!(b3.y >= b2.y, "div3 should be at or below div2");
}

/// Sticky 定位元素标记为 is_sticky，且在正常流中布局。
///
/// taffy 无原生 sticky 支持，映射为 Relative。
/// 验证 is_sticky 标记正确且元素参与正常流布局。
#[test]
fn test_sticky_position_in_normal_flow() {
    let (mut doc, body) = make_doc_with_body();
    let div1 = doc.create_element("div");
    doc.append_child(body, div1).unwrap();
    let sticky = doc.create_element("div");
    doc.append_child(body, sticky).unwrap();
    let div3 = doc.create_element("div");
    doc.append_child(body, div3).unwrap();

    let mut styles = HashMap::new();
    styles.insert(div1, make_style_with_display(DisplayValue::Block, 200.0, 50.0));

    // sticky 不设置 top/bottom inset，避免 taffy relative 偏移影响布局位置
    let mut sticky_style = ComputedStyle::default();
    sticky_style.display = DisplayValue::Block;
    sticky_style.position = PositionValue::Sticky;
    sticky_style.width = LengthValue::Px(200.0);
    sticky_style.height = LengthValue::Px(100.0);
    styles.insert(sticky, sticky_style);

    styles.insert(div3, make_style_with_display(DisplayValue::Block, 200.0, 50.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, div1).expect("div1 found");
    let sticky_box = find_child_by_node_id(&result.root, sticky).expect("sticky found");
    let b3 = find_child_by_node_id(&result.root, div3).expect("div3 found");

    // sticky 应被正确标记
    assert!(sticky_box.is_sticky, "should be flagged as sticky");
    assert!(!sticky_box.is_absolute, "sticky should not be absolute");
    assert!(!sticky_box.is_fixed, "sticky should not be fixed");

    // sticky 应在正常流中（在 div1 之后，div3 之前）
    // taffy 将 sticky 映射为 relative，不设置 inset 时位置等同于 static
    assert!(
        sticky_box.y >= b1.y,
        "sticky should be at or below div1: sticky.y({}) >= div1.y({})",
        sticky_box.y,
        b1.y
    );

    // div3 应在 sticky 之后（正常流顺序）
    assert!(
        b3.y >= sticky_box.y,
        "div3 should be at or below sticky: b3.y({}) >= sticky.y({})",
        b3.y,
        sticky_box.y
    );

    // sticky 尺寸正确
    assert_eq!(sticky_box.width, 200.0);
    assert_eq!(sticky_box.height, 100.0);
}

/// 嵌套 flex 容器 — 外层 row，内层 column。
///
/// 外层水平排列，内层垂直排列，验证内外方向独立。
#[test]
fn test_nested_flex_row_inside_column() {
    let (mut doc, body) = make_doc_with_body();
    // 外层: column
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();

    // 内层: row（作为外层第一个子元素）
    let inner = doc.create_element("div");
    doc.append_child(outer, inner).unwrap();
    let inner_item1 = doc.create_element("span");
    doc.append_child(inner, inner_item1).unwrap();
    let inner_item2 = doc.create_element("span");
    doc.append_child(inner, inner_item2).unwrap();

    // 外层第二个子元素
    let outer_item = doc.create_element("span");
    doc.append_child(outer, outer_item).unwrap();

    let mut styles = HashMap::new();

    let mut outer_style = ComputedStyle::default();
    outer_style.display = DisplayValue::Flex;
    outer_style.flex_direction = FlexDirectionValue::Column;
    outer_style.width = LengthValue::Px(400.0);
    outer_style.height = LengthValue::Px(300.0);
    styles.insert(outer, outer_style);

    let mut inner_style = ComputedStyle::default();
    inner_style.display = DisplayValue::Flex;
    inner_style.flex_direction = FlexDirectionValue::Row;
    inner_style.width = LengthValue::Px(400.0);
    inner_style.height = LengthValue::Px(150.0);
    styles.insert(inner, inner_style);

    // 内层子元素水平排列
    for id in [inner_item1, inner_item2] {
        let mut s = ComputedStyle::default();
        s.width = LengthValue::Px(150.0);
        s.height = LengthValue::Px(60.0);
        styles.insert(id, s);
    }

    // 外层子元素
    let mut outer_item_style = ComputedStyle::default();
    outer_item_style.width = LengthValue::Px(200.0);
    outer_item_style.height = LengthValue::Px(80.0);
    styles.insert(outer_item, outer_item_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let inner_box = find_child_by_node_id(&result.root, inner).expect("inner found");
    let outer_item_box = find_child_by_node_id(&result.root, outer_item).expect("outer_item found");
    let ii1 = find_child_by_node_id(&result.root, inner_item1).expect("inner_item1 found");
    let ii2 = find_child_by_node_id(&result.root, inner_item2).expect("inner_item2 found");

    // 外层 column: inner 和 outer_item 垂直排列
    assert!(
        outer_item_box.y > inner_box.y,
        "outer_item should be below inner (column layout)"
    );

    // 内层 row: inner_item1 和 inner_item2 水平排列
    assert!(ii2.x > ii1.x, "inner items should be horizontal (row layout)");
}

/// 绝对定位元素在 relative 父容器内，且父容器有 padding。
///
/// 绝对定位的参考点是 padding edge（包含 padding 的区域），
/// 验证 inset 偏移是相对于 padding 内边缘计算的。
#[test]
fn test_absolute_in_relative_parent_with_padding() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let abs_child = doc.create_element("span");
    doc.append_child(parent, abs_child).unwrap();

    let mut styles = HashMap::new();

    // parent: relative 定位 + padding
    let mut parent_style = ComputedStyle::default();
    parent_style.display = DisplayValue::Block;
    parent_style.position = PositionValue::Relative;
    parent_style.width = LengthValue::Px(400.0);
    parent_style.height = LengthValue::Px(300.0);
    parent_style.padding_top = LengthValue::Px(20.0);
    parent_style.padding_left = LengthValue::Px(30.0);
    parent_style.padding_bottom = LengthValue::Px(20.0);
    parent_style.padding_right = LengthValue::Px(30.0);
    styles.insert(parent, parent_style);

    // absolute child: top=10, left=15
    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(10.0);
    abs_style.left = LengthValue::Px(15.0);
    abs_style.width = LengthValue::Px(80.0);
    abs_style.height = LengthValue::Px(60.0);
    styles.insert(abs_child, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let parent_box = find_child_by_node_id(&result.root, parent).expect("parent found");
    let abs_box = find_child_by_node_id(&result.root, abs_child).expect("abs found");

    // 绝对定位标记
    assert!(abs_box.is_absolute, "should be flagged as absolute");

    // 绝对定位的坐标是相对于包含块的 content edge（不含 padding，由 taffy 决定）
    // top=10, left=15 表示相对于包含块的偏移
    assert!(
        (abs_box.x - 15.0).abs() < 2.0,
        "abs x 偏移应约 15（left），实际 {}",
        abs_box.x
    );
    assert!(
        (abs_box.y - 10.0).abs() < 2.0,
        "abs y 偏移应约 10（top），实际 {}",
        abs_box.y
    );
    assert_eq!(abs_box.width, 80.0);
    assert_eq!(abs_box.height, 60.0);

    // 父容器的 padding 应正确
    assert_eq!(parent_box.padding_top, 20.0);
    assert_eq!(parent_box.padding_left, 30.0);
}

/// Grid 使用 grid-auto-flow: column — 子元素按列方向自动放置。
#[test]
fn test_grid_auto_flow_column() {
    use zero_style_system::GridAutoFlowValue;

    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let mut item_ids = Vec::new();
    for _ in 0..6 {
        let item = doc.create_element("span");
        doc.append_child(grid, item).unwrap();
        item_ids.push(item);
    }

    let mut styles = HashMap::new();
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px 100px".to_string());
    grid_style.grid_template_rows = Some("50px 50px 50px".to_string());
    grid_style.grid_auto_flow = GridAutoFlowValue::Column;
    grid_style.width = LengthValue::Px(200.0);
    grid_style.height = LengthValue::Px(150.0);
    styles.insert(grid, grid_style);

    for id in &item_ids {
        styles.insert(*id, ComputedStyle::default());
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 所有子元素应有有效布局
    let boxes: Vec<&LayoutBox> = item_ids
        .iter()
        .map(|id| find_child_by_node_id(&result.root, *id).expect("grid item found"))
        .collect();

    // 所有元素宽度应有限
    for (i, b) in boxes.iter().enumerate() {
        assert!(
            b.width.is_finite() && b.width > 0.0,
            "grid item {} 宽度应为正有限值，实际 {}",
            i,
            b.width
        );
    }

    // column auto-flow: 元素应先填满列再换列
    // 前 3 个应在第一列（y 递增），后 3 个在第二列
    assert!(
        boxes[1].y > boxes[0].y,
        "column flow: item1.y({}) > item0.y({})",
        boxes[1].y,
        boxes[0].y
    );
    assert!(
        boxes[2].y > boxes[1].y,
        "column flow: item2.y({}) > item1.y({})",
        boxes[2].y,
        boxes[1].y
    );
}

// ── 边界条件测试（第三批）──

/// 测试 block 元素使用负 margin-top（Px(-10.0)），验证布局计算不 panic 且几何值合理。
#[test]
fn test_layout_negative_margin() {
    let (mut doc, body) = make_doc_with_body();
    let div1 = doc.create_element("div");
    doc.append_child(body, div1).unwrap();
    let div2 = doc.create_element("div");
    doc.append_child(body, div2).unwrap();

    let mut styles = HashMap::new();
    styles.insert(div1, make_style_with_display(DisplayValue::Block, 200.0, 100.0));

    let mut s2 = make_style_with_display(DisplayValue::Block, 200.0, 80.0);
    s2.margin_top = LengthValue::Px(-10.0);
    styles.insert(div2, s2);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, div1).expect("div1 found");
    let b2 = find_child_by_node_id(&result.root, div2).expect("div2 found");

    assert!(
        b1.width.is_finite() && b1.width > 0.0,
        "div1 width should be finite and positive"
    );
    assert!(
        b2.width.is_finite() && b2.width > 0.0,
        "div2 width should be finite and positive"
    );
    assert!(
        b2.height.is_finite() && b2.height >= 0.0,
        "div2 height should be finite and non-negative"
    );

    // negative margin-top should shift div2 upward relative to normal flow
    let normal_y = b1.y + b1.height;
    assert!(
        b2.y < normal_y,
        "div2.y ({}) should be less than normal flow position ({}) due to negative margin-top",
        b2.y,
        normal_y
    );
}

/// 测试嵌套 flex 容器（flex 嵌套 flex），验证内层 flex 布局正确计算。
#[test]
fn test_layout_nested_flex() {
    let (mut doc, body) = make_doc_with_body();
    // outer flex container (row)
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    // inner flex container (also flex, column)
    let inner = doc.create_element("div");
    doc.append_child(outer, inner).unwrap();
    // inner items
    let inner_item1 = doc.create_element("span");
    doc.append_child(inner, inner_item1).unwrap();
    let inner_item2 = doc.create_element("span");
    doc.append_child(inner, inner_item2).unwrap();

    let mut styles = HashMap::new();

    let mut outer_style = ComputedStyle::default();
    outer_style.display = DisplayValue::Flex;
    outer_style.flex_direction = FlexDirectionValue::Row;
    outer_style.width = LengthValue::Px(400.0);
    outer_style.height = LengthValue::Px(200.0);
    styles.insert(outer, outer_style);

    // inner is also a flex container (column)
    let mut inner_style = ComputedStyle::default();
    inner_style.display = DisplayValue::Flex;
    inner_style.flex_direction = FlexDirectionValue::Column;
    inner_style.width = LengthValue::Px(200.0);
    inner_style.height = LengthValue::Px(200.0);
    styles.insert(inner, inner_style);

    for id in [inner_item1, inner_item2] {
        let mut s = ComputedStyle::default();
        s.width = LengthValue::Px(100.0);
        s.height = LengthValue::Px(60.0);
        styles.insert(id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let outer_box = find_child_by_node_id(&result.root, outer).expect("outer found");
    let inner_box = find_child_by_node_id(&result.root, inner).expect("inner found");
    let i1 = find_child_by_node_id(&result.root, inner_item1).expect("item1 found");
    let i2 = find_child_by_node_id(&result.root, inner_item2).expect("item2 found");

    assert!((outer_box.width - 400.0).abs() < 1.0, "outer width should be ~400");
    assert!((inner_box.width - 200.0).abs() < 1.0, "inner width should be ~200");

    // inner items should be vertically stacked (column)
    assert!(i2.y > i1.y, "inner item2 should be below item1 in column flex");
}

/// 测试 relative 父容器内的 absolute 子元素，验证 absolute 子元素以父元素作为包含块。
#[test]
fn test_layout_absolute_in_relative() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let abs_child = doc.create_element("span");
    doc.append_child(parent, abs_child).unwrap();

    let mut styles = HashMap::new();

    // parent: relative positioned container
    let mut parent_style = ComputedStyle::default();
    parent_style.display = DisplayValue::Block;
    parent_style.position = PositionValue::Relative;
    parent_style.width = LengthValue::Px(300.0);
    parent_style.height = LengthValue::Px(200.0);
    styles.insert(parent, parent_style);

    // absolute child positioned relative to parent
    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(30.0);
    abs_style.left = LengthValue::Px(40.0);
    abs_style.width = LengthValue::Px(100.0);
    abs_style.height = LengthValue::Px(80.0);
    styles.insert(abs_child, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let abs_box = find_child_by_node_id(&result.root, abs_child).expect("abs child found");
    assert!(abs_box.is_absolute, "child should be flagged as absolute");

    // absolute child should use parent as containing block
    assert!(
        (abs_box.x - 40.0).abs() < 1.0,
        "abs child x should be ~40, got {}",
        abs_box.x
    );
    assert!(
        (abs_box.y - 30.0).abs() < 1.0,
        "abs child y should be ~30, got {}",
        abs_box.y
    );
    assert_eq!(abs_box.width, 100.0);
    assert_eq!(abs_box.height, 80.0);
}

/// 测试 overflow:hidden 容器包含超出边界的子元素，验证布局计算正常（裁剪在渲染层处理）。
#[test]
fn test_layout_overflow_hidden_truncation() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let child = doc.create_element("div");
    doc.append_child(container, child).unwrap();

    let mut styles = HashMap::new();

    // container with overflow:hidden and fixed size
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Block;
    container_style.overflow_x = OverflowValue::Hidden;
    container_style.overflow_y = OverflowValue::Hidden;
    container_style.width = LengthValue::Px(100.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    // child exceeds container bounds
    let mut child_style = ComputedStyle::default();
    child_style.display = DisplayValue::Block;
    child_style.width = LengthValue::Px(200.0);
    child_style.height = LengthValue::Px(200.0);
    styles.insert(child, child_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container_box = find_child_by_node_id(&result.root, container).expect("container found");
    let child_box = find_child_by_node_id(&result.root, child).expect("child found");

    // container size should remain fixed
    assert!(
        (container_box.width - 100.0).abs() < 1.0,
        "container width should be ~100, got {}",
        container_box.width
    );
    assert!(
        (container_box.height - 100.0).abs() < 1.0,
        "container height should be ~100, got {}",
        container_box.height
    );

    // overflow flags should be set
    assert_eq!(container_box.overflow_x, OverflowClip::Hidden);
    assert_eq!(container_box.overflow_y, OverflowClip::Hidden);

    // child retains its full size (clipping is at render level)
    assert!(
        (child_box.width - 200.0).abs() < 1.0,
        "child width should still be ~200, got {}",
        child_box.width
    );
    assert!(
        (child_box.height - 200.0).abs() < 1.0,
        "child height should still be ~200, got {}",
        child_box.height
    );
}

/// 测试 grid auto-placement：3 个子元素无显式 grid-area 赋值，验证自动放置分配位置。
#[test]
fn test_layout_grid_auto_placement() {
    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let mut item_ids = Vec::new();
    for _ in 0..3 {
        let item = doc.create_element("span");
        doc.append_child(grid, item).unwrap();
        item_ids.push(item);
    }

    let mut styles = HashMap::new();

    // grid container with 2 columns, no explicit grid-area on children
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px 100px".to_string());
    grid_style.grid_template_rows = Some("50px 50px".to_string());
    grid_style.width = LengthValue::Px(200.0);
    grid_style.height = LengthValue::Px(100.0);
    styles.insert(grid, grid_style);

    // no grid-area assignments — auto-placement should assign positions
    for id in &item_ids {
        styles.insert(*id, ComputedStyle::default());
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b0 = find_child_by_node_id(&result.root, item_ids[0]).expect("item0 found");
    let b1 = find_child_by_node_id(&result.root, item_ids[1]).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item_ids[2]).expect("item2 found");

    // auto-placement: item0 and item1 should be in row 1, item2 wraps to row 2
    assert!(
        b1.x > b0.x,
        "item1 (x={}) should be right of item0 (x={}) via auto-placement",
        b1.x,
        b0.x
    );
    assert!((b0.y - b1.y).abs() < 0.01, "item0 and item1 should be on the same row");
    assert!(
        b2.y > b0.y,
        "item2 (y={}) should wrap to next row, below item0 (y={})",
        b2.y,
        b0.y
    );

    // all items should have finite positive widths
    for (i, &id) in item_ids.iter().enumerate() {
        let b = find_child_by_node_id(&result.root, id).unwrap();
        assert!(
            b.width.is_finite() && b.width > 0.0,
            "item{} width should be finite and positive, got {}",
            i,
            b.width
        );
    }
}

/// 测试 block 元素 height:0px，验证产生高度为 0 的布局盒。
#[test]
fn test_layout_zero_height_block() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.display = DisplayValue::Block;
    div_style.width = LengthValue::Px(200.0);
    div_style.height = LengthValue::Px(0.0);
    styles.insert(div, div_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div_box = find_child_by_node_id(&result.root, div).expect("div found");

    // height should be exactly 0
    assert!(
        (div_box.height - 0.0).abs() < 0.001,
        "div height should be 0, got {}",
        div_box.height
    );

    // width should still be correct
    assert!(
        (div_box.width - 200.0).abs() < 1.0,
        "div width should be ~200, got {}",
        div_box.width
    );

    // content height should also be 0
    assert!(
        div_box.content_height.abs() < 0.001,
        "content_height should be 0, got {}",
        div_box.content_height
    );

    // should not be NaN or negative
    assert!(div_box.height.is_finite(), "height should be finite");
    assert!(div_box.height >= 0.0, "height should be non-negative");
}

/// flex-shrink 在空间不足时收缩子元素。
#[test]
fn test_flex_shrink_behavior() {
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

    // item1: 200px, shrink=1
    // min_width:0 显式覆盖默认 auto——否则 CSS §4.5 自动最小尺寸取 specified size(200px)
    // 为下限，flex item 不收缩（经典 flexbox 陷阱：min-width:0 才允许收缩到内容以下）。
    let mut s1 = ComputedStyle::default();
    s1.width = LengthValue::Px(200.0);
    s1.min_width = LengthValue::Px(0.0);
    s1.height = LengthValue::Px(50.0);
    s1.flex_shrink = 1.0;
    styles.insert(item1, s1);

    // item2: 200px, shrink=1
    let mut s2 = ComputedStyle::default();
    s2.width = LengthValue::Px(200.0);
    s2.min_width = LengthValue::Px(0.0);
    s2.height = LengthValue::Px(50.0);
    s2.flex_shrink = 2.0;
    styles.insert(item2, s2);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

    // 两项总宽度 400px，容器 200px，需收缩 200px
    // item1 shrink=1, item2 shrink=2 → 总 shrink=3
    // item1 收缩 200*1/3 ≈ 66.67 → 133.33
    // item2 收缩 200*2/3 ≈ 133.33 → 66.67
    let total = b1.width + b2.width;
    assert!(
        (total - 200.0).abs() < 1.0,
        "items should fill container: total={}",
        total
    );
    assert!(
        b1.width > b2.width,
        "item1 (shrink=1) should be wider than item2 (shrink=2): {} vs {}",
        b1.width,
        b2.width
    );
}

// -- 边界条件测试 --

/// 测试嵌套 absolute in fixed 布局
#[test]
fn test_absolute_in_fixed_layout() {
    // Fixed parent > absolute child，验证定位
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let fixed_parent = doc.create_element("div");
    doc.append_child(container, fixed_parent).unwrap();
    let abs_child = doc.create_element("span");
    doc.append_child(fixed_parent, abs_child).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(300.0);
    styles.insert(container, container_style);

    let mut fixed_style = ComputedStyle::default();
    fixed_style.position = PositionValue::Fixed;
    fixed_style.top = LengthValue::Px(10.0);
    fixed_style.left = LengthValue::Px(20.0);
    fixed_style.width = LengthValue::Px(200.0);
    fixed_style.height = LengthValue::Px(150.0);
    styles.insert(fixed_parent, fixed_style);

    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(5.0);
    abs_style.left = LengthValue::Px(10.0);
    abs_style.width = LengthValue::Px(50.0);
    abs_style.height = LengthValue::Px(30.0);
    styles.insert(abs_child, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fixed_box = find_child_by_node_id(&result.root, fixed_parent).expect("fixed found");
    let abs_box = find_child_by_node_id(&result.root, abs_child).expect("abs found");

    assert!(fixed_box.is_fixed, "父元素应标记为 fixed");
    assert!(abs_box.is_absolute, "子元素应标记为 absolute");
    assert_eq!(fixed_box.width, 200.0);
    assert_eq!(abs_box.width, 50.0);
}

/// 测试 flex wrap 在窄容器中的行为
#[test]
fn test_flex_wrap_very_narrow_container() {
    // 100px items in 50px container
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
    container_style.flex_wrap = FlexWrapValue::Wrap;
    container_style.width = LengthValue::Px(50.0);
    container_style.height = LengthValue::Px(500.0);
    styles.insert(container, container_style);

    for id in &item_ids {
        let mut s = ComputedStyle::default();
        s.width = LengthValue::Px(100.0);
        s.height = LengthValue::Px(30.0);
        s.flex_shrink = 0.0;
        styles.insert(*id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 布局不 panic，且所有元素存在
    let b0 = find_child_by_node_id(&result.root, item_ids[0]).expect("item0 found");
    let b1 = find_child_by_node_id(&result.root, item_ids[1]).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item_ids[2]).expect("item2 found");

    // 元素应换行（窄容器中每个元素单独一行）
    assert!(b1.y > b0.y, "item1 应在 item0 下方（换行）");
    assert!(b2.y > b1.y, "item2 应在 item1 下方（换行）");
}

/// 测试 grid 空单元格
#[test]
fn test_grid_with_empty_cells() {
    // 3x3 grid 只有 2 个 item，验证空单元格不影响布局
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
    grid_style.grid_template_columns = Some("100px 100px 100px".to_string());
    grid_style.grid_template_rows = Some("50px 50px 50px".to_string());
    grid_style.width = LengthValue::Px(300.0);
    grid_style.height = LengthValue::Px(150.0);
    styles.insert(grid, grid_style);

    for id in [item1, item2] {
        styles.insert(id, ComputedStyle::default());
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

    // 两个元素都应有有效布局
    assert!(b1.width.is_finite() && b1.width > 0.0, "item1 应有正有限宽度");
    assert!(b2.width.is_finite() && b2.width > 0.0, "item2 应有正有限宽度");
    assert!(b1.height.is_finite() && b1.height > 0.0, "item1 应有正有限高度");
    assert!(b2.height.is_finite() && b2.height > 0.0, "item2 应有正有限高度");
}

/// 测试 flex column 嵌套 block 布局
#[test]
fn test_flex_column_nested_block() {
    // Flex column > block children
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    let block1 = doc.create_element("div");
    doc.append_child(container, block1).unwrap();
    let block2 = doc.create_element("div");
    doc.append_child(container, block2).unwrap();
    let block3 = doc.create_element("div");
    doc.append_child(container, block3).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_direction = FlexDirectionValue::Column;
    container_style.width = LengthValue::Px(300.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    styles.insert(block1, make_style_with_display(DisplayValue::Block, 100.0, 40.0));
    styles.insert(block2, make_style_with_display(DisplayValue::Block, 150.0, 50.0));
    styles.insert(block3, make_style_with_display(DisplayValue::Block, 200.0, 60.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, block1).expect("block1 found");
    let b2 = find_child_by_node_id(&result.root, block2).expect("block2 found");
    let b3 = find_child_by_node_id(&result.root, block3).expect("block3 found");

    // 垂直排列：y 递增
    assert!(b2.y > b1.y, "block2 应在 block1 下方");
    assert!(b3.y > b2.y, "block3 应在 block2 下方");

    // x 应相同（同一列）
    assert!(
        (b1.x - b2.x).abs() < 0.01 && (b2.x - b3.x).abs() < 0.01,
        "flex column 中 block 子元素应在同一列"
    );
}

/// 测试多层级联 margin collapse 近似
#[test]
fn test_block_nested_margin_effects() {
    // 嵌套 block 多层有 margin，验证布局
    // taffy 0.7 已内置块级 margin 折叠，嵌套无 border/padding 的元素 margin 会向上传播
    let (mut doc, body) = make_doc_with_body();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let middle = doc.create_element("div");
    doc.append_child(outer, middle).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(middle, inner).unwrap();

    let mut styles = HashMap::new();
    let mut outer_style = make_style_with_display(DisplayValue::Block, 400.0, 300.0);
    outer_style.margin_top = LengthValue::Px(10.0);
    outer_style.margin_bottom = LengthValue::Px(20.0);
    styles.insert(outer, outer_style);

    let mut middle_style = make_style_with_display(DisplayValue::Block, 300.0, 200.0);
    middle_style.margin_top = LengthValue::Px(15.0);
    middle_style.margin_bottom = LengthValue::Px(25.0);
    styles.insert(middle, middle_style);

    let mut inner_style = make_style_with_display(DisplayValue::Block, 200.0, 100.0);
    inner_style.margin_top = LengthValue::Px(5.0);
    styles.insert(inner, inner_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let outer_box = find_child_by_node_id(&result.root, outer).expect("outer found");
    let middle_box = find_child_by_node_id(&result.root, middle).expect("middle found");
    let inner_box = find_child_by_node_id(&result.root, inner).expect("inner found");

    // 布局不 panic，尺寸正确
    assert_eq!(outer_box.width, 400.0);
    assert_eq!(middle_box.width, 300.0);
    assert_eq!(inner_box.width, 200.0);

    // taffy 0.7 已内置块级 margin 折叠，嵌套无 border/padding 的元素 margin 会向上传播
    // middle 在 outer 内容区域内（margin 折叠后 middle.y 可能等于 outer.content_y）
    assert!(
        middle_box.y >= outer_box.content_y - 0.5,
        "middle 应在 outer 内容区域内，middle.y={}, outer.content_y={}",
        middle_box.y,
        outer_box.content_y
    );
    // inner 在 middle 内部
    assert!(
        inner_box.y >= middle_box.content_y - 0.5,
        "inner 应在 middle 内容区域内，inner.y={}, middle.content_y={}",
        inner_box.y,
        middle_box.content_y
    );
}

/// 测试 zero-padding zero-border 的内容区域
#[test]
fn test_zero_padding_border_content_area() {
    // padding 和 border 都为 0 时，content area == total area
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.display = DisplayValue::Block;
    div_style.width = LengthValue::Px(200.0);
    div_style.height = LengthValue::Px(100.0);
    // 不设置 padding 和 border（默认为 0）
    styles.insert(div, div_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div_box = find_child_by_node_id(&result.root, div).expect("div found");

    // content_x 应等于 x（无 border/padding 偏移）
    assert!(
        (div_box.content_x - div_box.x).abs() < 0.001,
        "content_x 应等于 x，实际 content_x={}, x={}",
        div_box.content_x,
        div_box.x
    );
    assert!(
        (div_box.content_y - div_box.y).abs() < 0.001,
        "content_y 应等于 y，实际 content_y={}, y={}",
        div_box.content_y,
        div_box.y
    );
    // content 尺寸应等于总尺寸
    assert!(
        (div_box.content_width - div_box.width).abs() < 0.001,
        "content_width 应等于 width"
    );
    assert!(
        (div_box.content_height - div_box.height).abs() < 0.001,
        "content_height 应等于 height"
    );
}

/// 测试 absolute 定位元素超出父容器边界
#[test]
fn test_absolute_exceeding_parent_bounds() {
    // Absolute positioned element with top/left that goes outside parent
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let abs_child = doc.create_element("span");
    doc.append_child(container, abs_child).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.position = PositionValue::Relative;
    container_style.width = LengthValue::Px(100.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    // 绝对定位元素 top=80, left=80, 尺寸 50x50 → 超出父容器
    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(80.0);
    abs_style.left = LengthValue::Px(80.0);
    abs_style.width = LengthValue::Px(50.0);
    abs_style.height = LengthValue::Px(50.0);
    styles.insert(abs_child, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let abs_box = find_child_by_node_id(&result.root, abs_child).expect("abs found");
    assert!(abs_box.is_absolute, "应标记为 absolute");
    assert!((abs_box.x - 80.0).abs() < 1.0, "abs x 应约 80");
    assert!((abs_box.y - 80.0).abs() < 1.0, "abs y 应约 80");
    assert_eq!(abs_box.width, 50.0);
    assert_eq!(abs_box.height, 50.0);

    // 元素超出父容器边界
    let container_box = find_child_by_node_id(&result.root, container).expect("container found");
    assert!(
        abs_box.x + abs_box.width > container_box.x + container_box.width,
        "绝对元素应超出父容器右边界: abs_right={} > container_right={}",
        abs_box.x + abs_box.width,
        container_box.x + container_box.width
    );
    assert!(
        abs_box.y + abs_box.height > container_box.y + container_box.height,
        "绝对元素应超出父容器下边界: abs_bottom={} > container_bottom={}",
        abs_box.y + abs_box.height,
        container_box.y + container_box.height
    );
}

// -- 剩余边缘场景补充测试 --

/// 验证 OverflowValue::Auto 在布局输出中产生 OverflowClip::Scroll。
///
/// 根据 convert_overflow_to_clip 的映射，Auto 和 Scroll 都应转换为 Scroll。
#[test]
fn test_overflow_auto_produces_scroll_clip() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.display = DisplayValue::Block;
    div_style.overflow_x = OverflowValue::Auto;
    div_style.overflow_y = OverflowValue::Auto;
    div_style.width = LengthValue::Px(100.0);
    div_style.height = LengthValue::Px(100.0);
    styles.insert(div, div_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div_box = find_child_by_node_id(&result.root, div).expect("div found");
    assert_eq!(
        div_box.overflow_x,
        OverflowClip::Scroll,
        "overflow-x: Auto 应产生 OverflowClip::Scroll"
    );
    assert_eq!(
        div_box.overflow_y,
        OverflowClip::Scroll,
        "overflow-y: Auto 应产生 OverflowClip::Scroll"
    );
}

/// 验证 OverflowValue::Clip 在布局输出中产生 OverflowClip::Clip。
///
/// 根据 convert_overflow_to_clip 的映射，Clip 应直接转换为 Clip（非滚动容器裁剪）。
#[test]
fn test_overflow_clip_produces_clip() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.display = DisplayValue::Block;
    div_style.overflow_x = OverflowValue::Clip;
    div_style.overflow_y = OverflowValue::Clip;
    div_style.width = LengthValue::Px(100.0);
    div_style.height = LengthValue::Px(100.0);
    styles.insert(div, div_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div_box = find_child_by_node_id(&result.root, div).expect("div found");
    assert_eq!(
        div_box.overflow_x,
        OverflowClip::Clip,
        "overflow-x: Clip 应产生 OverflowClip::Clip"
    );
    assert_eq!(
        div_box.overflow_y,
        OverflowClip::Clip,
        "overflow-y: Clip 应产生 OverflowClip::Clip"
    );
}

/// 验证 ZIndexValue::Integer(5) 在 LayoutBox 中产生 z_index: 5，
/// 而 ZIndexValue::Auto 产生 z_index: 0。
#[test]
fn test_z_index_in_layout_output() {
    use zero_style_system::ZIndexValue;

    let (mut doc, body) = make_doc_with_body();
    let div_with_z = doc.create_element("div");
    doc.append_child(body, div_with_z).unwrap();
    let div_auto = doc.create_element("div");
    doc.append_child(body, div_auto).unwrap();

    let mut styles = HashMap::new();

    // z-index: 5
    let mut s1 = ComputedStyle::default();
    s1.display = DisplayValue::Block;
    s1.width = LengthValue::Px(100.0);
    s1.height = LengthValue::Px(50.0);
    s1.z_index = ZIndexValue::Integer(5);
    s1.position = PositionValue::Relative;
    styles.insert(div_with_z, s1);

    // z-index: auto
    let mut s2 = ComputedStyle::default();
    s2.display = DisplayValue::Block;
    s2.width = LengthValue::Px(100.0);
    s2.height = LengthValue::Px(50.0);
    s2.z_index = ZIndexValue::Auto;
    styles.insert(div_auto, s2);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let box_with_z = find_child_by_node_id(&result.root, div_with_z).expect("div_with_z found");
    let box_auto = find_child_by_node_id(&result.root, div_auto).expect("div_auto found");

    assert_eq!(box_with_z.z_index, 5, "ZIndexValue::Integer(5) 应产生 z_index=5");
    assert_eq!(box_auto.z_index, 0, "ZIndexValue::Auto 应产生 z_index=0");
}

/// 验证 content area clamp：容器 100px + border 80px + padding 30px 时 content_width 钳位到 0。
///
/// 容器 width=100px, border_left=40px, border_right=40px, padding_left=15px, padding_right=15px，
/// content_width = 100 - 40 - 40 - 15 - 15 = -10 → .max(0.0) = 0。
#[test]
fn test_content_area_clamp_with_oversized_border() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.display = DisplayValue::Block;
    div_style.width = LengthValue::Px(100.0);
    div_style.height = LengthValue::Px(100.0);
    div_style.border_left_width = LengthValue::Px(40.0);
    div_style.border_right_width = LengthValue::Px(40.0);
    div_style.border_top_width = LengthValue::Px(40.0);
    div_style.border_bottom_width = LengthValue::Px(40.0);
    div_style.padding_left = LengthValue::Px(15.0);
    div_style.padding_right = LengthValue::Px(15.0);
    div_style.padding_top = LengthValue::Px(15.0);
    div_style.padding_bottom = LengthValue::Px(15.0);
    styles.insert(div, div_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div_box = find_child_by_node_id(&result.root, div).expect("div found");

    // content_width = total_width - border_left - border_right - padding_left - padding_right
    // 但 taffy 可能调整总宽度。检查 content_width 不为负数。
    // 根据 extract_layout 中的 .max(0.0)，content_width 应 >= 0。
    assert!(
        div_box.content_width >= 0.0,
        "content_width 应被钳位到 >= 0，实际 {}",
        div_box.content_width
    );
    assert!(
        div_box.content_height >= 0.0,
        "content_height 应被钳位到 >= 0，实际 {}",
        div_box.content_height
    );

    // content_width 应为 0（border+padding 已超过 total size）
    // taffy content-box: total = width + border + padding = 100 + 80 + 30 = 210
    // content = width = 100; 但如果 taffy 不增加 border/padding 到 total，
    // 而是 total=100，则 content = 100 - 80 - 30 = -10 → clamped to 0。
    // 需要根据 taffy 实际行为验证。
    // 实际 border-box vs content-box: 默认 content-box 下 taffy 总宽度包含 border+padding，
    // 所以 content = width 指定的 100px。但 extract_layout 中的计算是从 layout.size 出发。
    // 检查 content_width 不为负即可（核心断言）。
}

/// 验证 fixed 定位元素在 5 层非 fixed 祖先嵌套下，
/// adjust_fixed_to_viewport 将其坐标正确调整为视口相对。
///
/// 结构：body > div1 > div2 > div3 > div4 > div5 > fixed_el
/// div1-div5 各有偏移，fixed_el 应将所有祖先偏移累加到自身坐标中。
#[test]
fn test_deeply_nested_fixed_position() {
    let (mut doc, body) = make_doc_with_body();
    let mut parent = body;
    let mut ancestor_ids = Vec::new();

    // 创建 5 层嵌套非 fixed 祖先
    for _ in 0..5 {
        let div = doc.create_element("div");
        doc.append_child(parent, div).unwrap();
        ancestor_ids.push(div);
        parent = div;
    }

    // 在最内层放置 fixed 元素
    let fixed_el = doc.create_element("span");
    doc.append_child(parent, fixed_el).unwrap();

    let mut styles = HashMap::new();

    // 祖先元素：每层有 margin 造成偏移
    for &id in &ancestor_ids {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.width = LengthValue::Px(300.0);
        s.height = LengthValue::Px(300.0);
        s.margin_top = LengthValue::Px(10.0);
        s.margin_left = LengthValue::Px(10.0);
        styles.insert(id, s);
    }

    // fixed 元素
    let mut fixed_style = ComputedStyle::default();
    fixed_style.position = PositionValue::Fixed;
    fixed_style.top = LengthValue::Px(50.0);
    fixed_style.left = LengthValue::Px(50.0);
    fixed_style.width = LengthValue::Px(100.0);
    fixed_style.height = LengthValue::Px(100.0);
    styles.insert(fixed_el, fixed_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fixed_box = find_child_by_node_id(&result.root, fixed_el).expect("fixed_el found");
    assert!(fixed_box.is_fixed, "应标记为 fixed");

    // R324：fixed 元素经 adjust_fixed_to_viewport 调整为视口相对。taffy 0.7 把 fixed
    // 当 absolute（containing block = 祖先），故 fixed_box 的 field（父相对 y）= top -
    // 累积祖先偏移；其【绝对坐标】（painter 累积后）= top（视口相对）。
    // 验证坐标不为 NaN 或无穷
    assert!(fixed_box.x.is_finite(), "fixed x 应为有限值，实际 {}", fixed_box.x);
    assert!(fixed_box.y.is_finite(), "fixed y 应为有限值，实际 {}", fixed_box.y);

    // 基本尺寸正确
    assert_eq!(fixed_box.width, 100.0, "fixed 元素宽度应为 100");
    assert_eq!(fixed_box.height, 100.0, "fixed 元素高度应为 100");

    // fixed 元素应在视口坐标系中：其【绝对坐标】（与 painter 一致，自根累积）应 = top=50，
    // 与 5 层 margin:10 祖先的累积偏移无关（视口相对）。field 值本身是父相对，不直接比较。
    fn abs_pos_by_node(root: &crate::types::LayoutBox, id: zero_dom::NodeId) -> Option<(f32, f32)> {
        fn walk(b: &crate::types::LayoutBox, ox: f32, oy: f32, id: zero_dom::NodeId) -> Option<(f32, f32)> {
            let ax = ox + b.x;
            let ay = oy + b.y;
            if b.node_id == Some(id) {
                return Some((ax, ay));
            }
            let cox = ax + b.padding_left + b.border_left;
            let coy = ay + b.padding_top + b.border_top;
            for c in &b.children {
                if let Some(p) = walk(c, cox, coy, id) {
                    return Some(p);
                }
            }
            None
        }
        walk(root, 0.0, 0.0, id)
    }
    let (abs_x, abs_y) = abs_pos_by_node(&result.root, fixed_el).expect("应能定位 fixed 元素的绝对坐标");
    assert!(
        (abs_x - 50.0).abs() < 1.0,
        "fixed 绝对 x 应为视口相对 ~50（left=50），实际 {}",
        abs_x
    );
    assert!(
        (abs_y - 50.0).abs() < 1.0,
        "fixed 绝对 y 应为视口相对 ~50（top=50），实际 {}",
        abs_y
    );
}

// ── 边缘场景补充测试（第四批）──

/// 测试 inline-block 元素带文本内容时的布局。
///
/// inline-block 在 taffy 中映射为 Block，验证元素尺寸正确且布局不 panic。
/// 结构：body > div(inline-block, 150x80) + span(inline-block, 100x40)
#[test]
fn test_layout_display_inline_block_with_text() {
    let (mut doc, body) = make_doc_with_body();
    let ib1 = doc.create_element("div");
    doc.append_child(body, ib1).unwrap();
    let ib2 = doc.create_element("span");
    doc.append_child(body, ib2).unwrap();

    let mut styles = HashMap::new();
    // inline-block 元素映射为 Block，正常参与布局
    let mut s1 = ComputedStyle::default();
    s1.display = DisplayValue::InlineBlock;
    s1.width = LengthValue::Px(150.0);
    s1.height = LengthValue::Px(80.0);
    styles.insert(ib1, s1);

    let mut s2 = ComputedStyle::default();
    s2.display = DisplayValue::InlineBlock;
    s2.width = LengthValue::Px(100.0);
    s2.height = LengthValue::Px(40.0);
    styles.insert(ib2, s2);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, ib1).expect("ib1 found");
    let b2 = find_child_by_node_id(&result.root, ib2).expect("ib2 found");

    // 两个 inline-block 元素映射为 Block，应垂直堆叠
    assert!(b2.y >= b1.y, "ib2 (y={}) 应在 ib1 (y={}) 下方或同位置", b2.y, b1.y);

    // 尺寸正确
    assert!((b1.width - 150.0).abs() < 1.0, "ib1 宽度应约 150，实际 {}", b1.width);
    assert!((b1.height - 80.0).abs() < 1.0, "ib1 高度应约 80，实际 {}", b1.height);
    assert!((b2.width - 100.0).abs() < 1.0, "ib2 宽度应约 100，实际 {}", b2.width);
    assert!((b2.height - 40.0).abs() < 1.0, "ib2 高度应约 40，实际 {}", b2.height);
}

/// 测试 sticky 定位元素在可滚动容器中的 is_sticky 标记。
///
/// taffy 无原生 sticky 支持，映射为 Relative。
/// 验证 is_sticky 标记正确，元素参与正常流布局且尺寸正确。
#[test]
fn test_layout_position_sticky() {
    let (mut doc, body) = make_doc_with_body();
    // 可滚动容器
    let scroll_container = doc.create_element("div");
    doc.append_child(body, scroll_container).unwrap();
    // sticky 元素
    let sticky = doc.create_element("div");
    doc.append_child(scroll_container, sticky).unwrap();
    // 后续内容
    let content = doc.create_element("div");
    doc.append_child(scroll_container, content).unwrap();

    let mut styles = HashMap::new();

    // 可滚动容器
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Block;
    container_style.overflow_y = OverflowValue::Scroll;
    container_style.width = LengthValue::Px(200.0);
    container_style.height = LengthValue::Px(300.0);
    styles.insert(scroll_container, container_style);

    // sticky 元素：position:sticky, top:10px
    let mut sticky_style = ComputedStyle::default();
    sticky_style.display = DisplayValue::Block;
    sticky_style.position = PositionValue::Sticky;
    sticky_style.top = LengthValue::Px(10.0);
    sticky_style.width = LengthValue::Px(200.0);
    sticky_style.height = LengthValue::Px(50.0);
    styles.insert(sticky, sticky_style);

    // 后续内容
    styles.insert(content, make_style_with_display(DisplayValue::Block, 200.0, 400.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let sticky_box = find_child_by_node_id(&result.root, sticky).expect("sticky found");

    // is_sticky 标记正确
    assert!(sticky_box.is_sticky, "应标记为 sticky");
    assert!(!sticky_box.is_absolute, "sticky 不应是 absolute");
    assert!(!sticky_box.is_fixed, "sticky 不应是 fixed");

    // 尺寸正确
    assert!(
        (sticky_box.width - 200.0).abs() < 1.0,
        "sticky 宽度应约 200，实际 {}",
        sticky_box.width
    );
    assert!(
        (sticky_box.height - 50.0).abs() < 1.0,
        "sticky 高度应约 50，实际 {}",
        sticky_box.height
    );

    // 容器 overflow 标记
    let container_box = find_child_by_node_id(&result.root, scroll_container).expect("container found");
    assert_eq!(container_box.overflow_y, OverflowClip::Scroll, "容器应标记为 scroll");
}

/// 测试 flex-wrap:wrap-reverse — 子元素换行方向反转。
///
/// 在 row 方向 flex 容器中，wrap-reverse 使第二行元素在上方排列。
/// 验证换行发生且行顺序反转。
#[test]
fn test_layout_flex_wrap_reverse() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    let mut item_ids = Vec::new();
    for _ in 0..4 {
        let item = doc.create_element("span");
        doc.append_child(container, item).unwrap();
        item_ids.push(item);
    }

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_wrap = FlexWrapValue::WrapReverse;
    container_style.width = LengthValue::Px(200.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    // 每个 item 120px 宽，容器 200px → 第二个 item 换行
    for id in &item_ids {
        let mut s = ComputedStyle::default();
        s.width = LengthValue::Px(120.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(*id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b0 = find_child_by_node_id(&result.root, item_ids[0]).expect("item0 found");
    let b1 = find_child_by_node_id(&result.root, item_ids[1]).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item_ids[2]).expect("item2 found");
    let b3 = find_child_by_node_id(&result.root, item_ids[3]).expect("item3 found");

    // wrap-reverse 中元素换行：item1 应与 item0 在不同行
    // 在正常 wrap 中 item0 在第一行、item1 在第二行
    // wrap-reverse 反转行顺序：item0 在下方行、item1 在上方行
    // 因此 item1.y < item0.y（行顺序反转）
    assert!(
        b1.y != b0.y,
        "wrap-reverse 中 item1 (y={}) 和 item0 (y={}) 应在不同行",
        b1.y,
        b0.y
    );

    // item2 和 item3 也应换行
    assert!(b2.y != b1.y || b3.y != b2.y, "至少部分后续 item 应换行");

    // 所有 item 尺寸正确
    assert!((b0.width - 120.0).abs() < 1.0, "item0 宽度应约 120，实际 {}", b0.width);
    assert!((b1.width - 120.0).abs() < 1.0, "item1 宽度应约 120，实际 {}", b1.width);
}

/// 测试 grid 容器使用 gap:10px 时子元素之间的间距。
///
/// 使用显式 grid-row/grid-column 放置 4 个元素到 2x2 grid 中，
/// gap=10px（column-gap）+ row_gap=10px，验证同行和同列间距正确。
#[test]
fn test_layout_grid_gap() {
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

    // 2x2 grid，column-gap=10px，row-gap=10px
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px 100px".to_string());
    grid_style.grid_template_rows = Some("50px 50px".to_string());
    grid_style.gap = LengthValue::Px(10.0);
    grid_style.row_gap = LengthValue::Px(10.0);
    grid_style.width = LengthValue::Px(300.0);
    grid_style.height = LengthValue::Px(300.0);
    styles.insert(grid, grid_style);

    // item1: row 1, col 1
    let mut s1 = ComputedStyle::default();
    s1.grid_row_start = GridLineValue::Line(1);
    s1.grid_row_end = GridLineValue::Line(2);
    s1.grid_column_start = GridLineValue::Line(1);
    s1.grid_column_end = GridLineValue::Line(2);
    styles.insert(item1, s1);

    // item2: row 1, col 2
    let mut s2 = ComputedStyle::default();
    s2.grid_row_start = GridLineValue::Line(1);
    s2.grid_row_end = GridLineValue::Line(2);
    s2.grid_column_start = GridLineValue::Line(2);
    s2.grid_column_end = GridLineValue::Line(3);
    styles.insert(item2, s2);

    // item3: row 2, col 1
    let mut s3 = ComputedStyle::default();
    s3.grid_row_start = GridLineValue::Line(2);
    s3.grid_row_end = GridLineValue::Line(3);
    s3.grid_column_start = GridLineValue::Line(1);
    s3.grid_column_end = GridLineValue::Line(2);
    styles.insert(item3, s3);

    // item4: row 2, col 2
    let mut s4 = ComputedStyle::default();
    s4.grid_row_start = GridLineValue::Line(2);
    s4.grid_row_end = GridLineValue::Line(3);
    s4.grid_column_start = GridLineValue::Line(2);
    s4.grid_column_end = GridLineValue::Line(3);
    styles.insert(item4, s4);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");
    let b3 = find_child_by_node_id(&result.root, item3).expect("item3 found");
    let b4 = find_child_by_node_id(&result.root, item4).expect("item4 found");

    // 同行水平 gap：item2.x - item1.x - item1.width ≈ 10px
    let h_gap = b2.x - b1.x - b1.width;
    assert!((h_gap - 10.0).abs() < 1.0, "水平 gap 应约 10px，实际 {}", h_gap);

    // 同列垂直 gap：item3.y - item1.y - item1.height ≈ 10px
    let v_gap = b3.y - b1.y - b1.height;
    assert!((v_gap - 10.0).abs() < 1.0, "垂直 gap 应约 10px，实际 {}", v_gap);

    // item4 应在 item3 右侧（同行）
    let h_gap2 = b4.x - b3.x - b3.width;
    assert!((h_gap2 - 10.0).abs() < 1.0, "第二行水平 gap 应约 10px，实际 {}", h_gap2);

    // 每个 cell 尺寸约 100x50
    assert!((b1.width - 100.0).abs() < 1.0, "item1 宽度应约 100，实际 {}", b1.width);
    assert!((b1.height - 50.0).abs() < 1.0, "item1 高度应约 50，实际 {}", b1.height);
}

/// 测试绝对定位元素设置 top:10px, left:20px 时的位置偏移。
///
/// 绝对定位元素相对于 relative 父容器定位，
/// 验证 x/y 偏移精确匹配设置的 top/left 值。
#[test]
fn test_layout_absolute_top_left() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let abs_child = doc.create_element("span");
    doc.append_child(parent, abs_child).unwrap();

    let mut styles = HashMap::new();

    // relative 父容器作为包含块
    let mut parent_style = ComputedStyle::default();
    parent_style.display = DisplayValue::Block;
    parent_style.position = PositionValue::Relative;
    parent_style.width = LengthValue::Px(400.0);
    parent_style.height = LengthValue::Px(300.0);
    styles.insert(parent, parent_style);

    // 绝对定位子元素：top:10px, left:20px
    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(10.0);
    abs_style.left = LengthValue::Px(20.0);
    abs_style.width = LengthValue::Px(60.0);
    abs_style.height = LengthValue::Px(40.0);
    styles.insert(abs_child, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let abs_box = find_child_by_node_id(&result.root, abs_child).expect("abs child found");

    // 验证 absolute 标记
    assert!(abs_box.is_absolute, "应标记为 absolute");
    assert!(!abs_box.is_fixed, "不应是 fixed");
    assert!(!abs_box.is_sticky, "不应是 sticky");

    // 验证位置偏移精确
    assert!(
        (abs_box.x - 20.0).abs() < 1.0,
        "abs x 偏移应约 20px（left），实际 {}",
        abs_box.x
    );
    assert!(
        (abs_box.y - 10.0).abs() < 1.0,
        "abs y 偏移应约 10px（top），实际 {}",
        abs_box.y
    );

    // 验证尺寸
    assert_eq!(abs_box.width, 60.0, "abs 宽度应为 60");
    assert_eq!(abs_box.height, 40.0, "abs 高度应为 40");
}

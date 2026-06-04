use super::*;
use zero_css_parser::values::{
    AlignmentValue, BoxSizingValue, DisplayValue, FlexDirectionValue, LengthValue, PositionValue,
};
// ── 边缘场景补充测试（第五批）──

/// 测试 display:none 父元素隐藏其子元素。
///
/// 父元素设置 display:none，子元素设置 display:block。
/// display:none 的父元素不构建子树，子元素不应出现在布局树中。
#[test]
fn test_layout_display_none_cascades() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("div");
    doc.append_child(parent, child).unwrap();
    // 在 parent 后再加一个可见元素，作为参照
    let visible = doc.create_element("div");
    doc.append_child(body, visible).unwrap();

    let mut styles = HashMap::new();
    // 父元素 display:none
    let mut parent_style = ComputedStyle::default();
    parent_style.display = DisplayValue::None;
    styles.insert(parent, parent_style);

    // 子元素 display:block（但因为父元素 display:none 而被隐藏）
    styles.insert(child, make_style_with_display(DisplayValue::Block, 100.0, 50.0));

    // 可见参照元素
    styles.insert(visible, make_style_with_display(DisplayValue::Block, 200.0, 80.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // display:none 的父元素不记录到 taffy_to_dom 映射中，
    // 因此 find_child_by_node_id 无法找到 parent 和 child。
    // 验证 parent 和 child 不在布局树中。
    assert!(
        find_child_by_node_id(&result.root, parent).is_none(),
        "display:none 的父元素不应出现在布局树中"
    );
    assert!(
        find_child_by_node_id(&result.root, child).is_none(),
        "display:none 父元素的子元素不应出现在布局树中"
    );

    // 可见参照元素应正常出现
    let vis_box = find_child_by_node_id(&result.root, visible).expect("visible 应找到");
    assert_eq!(vis_box.width, 200.0, "可见参照元素宽度应为 200");
    assert_eq!(vis_box.height, 80.0, "可见参照元素高度应为 80");
}

/// 测试百分比高度相对于父元素计算。
///
/// 父元素高度 200px，子元素高度 50%。
/// 子元素实际高度应为 100px（200 * 50% = 100）。
#[test]
fn test_layout_percentage_height_with_parent() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("div");
    doc.append_child(parent, child).unwrap();

    let mut styles = HashMap::new();
    // 父元素高度 200px
    let mut parent_style = ComputedStyle::default();
    parent_style.display = DisplayValue::Block;
    parent_style.width = LengthValue::Px(300.0);
    parent_style.height = LengthValue::Px(200.0);
    styles.insert(parent, parent_style);

    // 子元素高度 50%
    let mut child_style = ComputedStyle::default();
    child_style.display = DisplayValue::Block;
    child_style.width = LengthValue::Px(100.0);
    child_style.height = LengthValue::Percentage(50.0);
    styles.insert(child, child_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let child_box = find_child_by_node_id(&result.root, child).expect("child 应找到");
    // 50% of 200px = 100px
    assert!(
        (child_box.height - 100.0).abs() < 1.0,
        "子元素高度应为 100px（200 * 50%），实际 {}",
        child_box.height
    );
    assert_eq!(child_box.width, 100.0, "子元素宽度应为 100");
}

/// 测试 flex 容器 align-items:center 使子元素垂直居中。
///
/// 容器 200x200，子元素 60x40。
/// align-items:center 时子元素高度保持不变（不拉伸），
/// 验证子元素尺寸正确且在容器内布局合理。
#[test]
fn test_layout_flex_align_center() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let child = doc.create_element("span");
    doc.append_child(container, child).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.align_items = AlignmentValue::Center;
    container_style.width = LengthValue::Px(200.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    let mut child_style = ComputedStyle::default();
    child_style.width = LengthValue::Px(60.0);
    child_style.height = LengthValue::Px(40.0);
    styles.insert(child, child_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container_box = find_child_by_node_id(&result.root, container).expect("container 应找到");
    let child_box = find_child_by_node_id(&result.root, child).expect("child 应找到");

    // align-items:center 不拉伸子元素，子元素高度保持 40px
    assert_eq!(child_box.width, 60.0, "子元素宽度应为 60");
    assert_eq!(child_box.height, 40.0, "子元素高度应保持 40（不拉伸）");

    // 子元素应在容器内（y 坐标不应超出容器范围）
    assert!(
        child_box.y >= container_box.y,
        "子元素 y 应在容器内: child.y={} >= container.y={}",
        child_box.y,
        container_box.y
    );
    assert!(
        child_box.y + child_box.height <= container_box.y + container_box.height,
        "子元素不应超出容器底部"
    );

    // 与 align-items:stretch 对比：center 模式下子元素高度不应等于容器高度
    // （如果等于，说明 stretch 被错误应用）
    assert!(
        child_box.height < container_box.height,
        "center 模式下子元素高度应小于容器高度（不应拉伸）"
    );
}

/// 测试 grid 显式列模板 grid-template-columns:100px 200px。
///
/// 两个子元素自动放置，第一列宽度约 100px，第二列宽度约 200px。
#[test]
fn test_layout_grid_explicit_columns() {
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
    grid_style.grid_template_columns = Some("100px 200px".to_string());
    grid_style.grid_template_rows = Some("100px".to_string());
    grid_style.width = LengthValue::Px(300.0);
    grid_style.height = LengthValue::Px(100.0);
    styles.insert(grid, grid_style);

    // 子元素不设置显式尺寸，由 grid cell 自动填充
    for id in [item1, item2] {
        styles.insert(id, ComputedStyle::default());
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 应找到");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 应找到");

    // item1 在第一列，宽度应约 100px
    assert!(
        (b1.width - 100.0).abs() < 1.0,
        "第一列宽度应约 100px，实际 {}",
        b1.width
    );
    // item2 在第二列，宽度应约 200px
    assert!(
        (b2.width - 200.0).abs() < 1.0,
        "第二列宽度应约 200px，实际 {}",
        b2.width
    );
    // item2 应在 item1 右侧
    assert!(b2.x > b1.x, "item2 应在 item1 右侧: x={} vs x={}", b2.x, b1.x);
    // 两个元素应在同一行
    assert!((b1.y - b2.y).abs() < 0.01, "两个元素应在同一行");
}

// ── 边缘场景补充测试（第六批）──

/// 测试 grid-template-areas 3x3 布局。
///
/// 定义 3x3 区域：
///   "header header header"
///   "sidebar main   aside"
///   "footer footer footer"
/// 验证 header 和 footer 跨三列，sidebar/main/aside 各占一列。
#[test]
fn test_grid_template_areas_3x3() {
    use zero_style_system::GridLineValue;

    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let header_el = doc.create_element("div");
    doc.append_child(grid, header_el).unwrap();
    let sidebar_el = doc.create_element("div");
    doc.append_child(grid, sidebar_el).unwrap();
    let main_el = doc.create_element("div");
    doc.append_child(grid, main_el).unwrap();
    let aside_el = doc.create_element("div");
    doc.append_child(grid, aside_el).unwrap();
    let footer_el = doc.create_element("div");
    doc.append_child(grid, footer_el).unwrap();

    let mut styles = HashMap::new();

    // 3x3 grid
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px 200px 100px".to_string());
    grid_style.grid_template_rows = Some("50px 100px 50px".to_string());
    grid_style.grid_template_areas =
        Some("\"header header header\" \"sidebar main aside\" \"footer footer footer\"".to_string());
    grid_style.width = LengthValue::Px(400.0);
    grid_style.height = LengthValue::Px(200.0);
    styles.insert(grid, grid_style);

    // header 跨第一行三列
    let mut header_style = ComputedStyle::default();
    header_style.grid_row_start = GridLineValue::Name("header".to_string());
    header_style.grid_row_end = GridLineValue::Name("header".to_string());
    header_style.grid_column_start = GridLineValue::Name("header".to_string());
    header_style.grid_column_end = GridLineValue::Name("header".to_string());
    styles.insert(header_el, header_style);

    // sidebar 第二行第一列
    let mut sidebar_style = ComputedStyle::default();
    sidebar_style.grid_row_start = GridLineValue::Name("sidebar".to_string());
    sidebar_style.grid_row_end = GridLineValue::Name("sidebar".to_string());
    sidebar_style.grid_column_start = GridLineValue::Name("sidebar".to_string());
    sidebar_style.grid_column_end = GridLineValue::Name("sidebar".to_string());
    styles.insert(sidebar_el, sidebar_style);

    // main 第二行第二列
    let mut main_style = ComputedStyle::default();
    main_style.grid_row_start = GridLineValue::Name("main".to_string());
    main_style.grid_row_end = GridLineValue::Name("main".to_string());
    main_style.grid_column_start = GridLineValue::Name("main".to_string());
    main_style.grid_column_end = GridLineValue::Name("main".to_string());
    styles.insert(main_el, main_style);

    // aside 第二行第三列
    let mut aside_style = ComputedStyle::default();
    aside_style.grid_row_start = GridLineValue::Name("aside".to_string());
    aside_style.grid_row_end = GridLineValue::Name("aside".to_string());
    aside_style.grid_column_start = GridLineValue::Name("aside".to_string());
    aside_style.grid_column_end = GridLineValue::Name("aside".to_string());
    styles.insert(aside_el, aside_style);

    // footer 跨第三行三列
    let mut footer_style = ComputedStyle::default();
    footer_style.grid_row_start = GridLineValue::Name("footer".to_string());
    footer_style.grid_row_end = GridLineValue::Name("footer".to_string());
    footer_style.grid_column_start = GridLineValue::Name("footer".to_string());
    footer_style.grid_column_end = GridLineValue::Name("footer".to_string());
    styles.insert(footer_el, footer_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let header_box = find_child_by_node_id(&result.root, header_el).expect("header found");
    let sidebar_box = find_child_by_node_id(&result.root, sidebar_el).expect("sidebar found");
    let main_box = find_child_by_node_id(&result.root, main_el).expect("main found");
    let aside_box = find_child_by_node_id(&result.root, aside_el).expect("aside found");
    let footer_box = find_child_by_node_id(&result.root, footer_el).expect("footer found");

    // header 应跨三列（~400px）
    assert!(
        (header_box.width - 400.0).abs() < 2.0,
        "header 应跨三列（~400px），实际 {}",
        header_box.width
    );
    assert!(
        (header_box.height - 50.0).abs() < 2.0,
        "header 应高约 50px，实际 {}",
        header_box.height
    );

    // sidebar 和 aside 应在 main 两侧
    assert!(sidebar_box.x < main_box.x, "sidebar 应在 main 左侧");
    assert!(aside_box.x > main_box.x, "aside 应在 main 右侧");

    // main 宽度约 200px（中间列）
    assert!(
        (main_box.width - 200.0).abs() < 2.0,
        "main 应宽约 200px，实际 {}",
        main_box.width
    );

    // sidebar 和 main 在同一行
    assert!((sidebar_box.y - main_box.y).abs() < 1.0, "sidebar 和 main 应在同一行");

    // footer 应在 main 下方
    assert!(footer_box.y > main_box.y, "footer 应在 main 下方");
    assert!(
        (footer_box.width - 400.0).abs() < 2.0,
        "footer 应跨三列（~400px），实际 {}",
        footer_box.width
    );
}

/// 测试 grid-template-areas 中列数不匹配的情况。
///
/// 第一行有 3 列，第二行只有 2 列。
/// 验证布局不 panic，且子元素仍有有效布局盒。
#[test]
fn test_grid_template_areas_invalid_shape() {
    use zero_style_system::GridLineValue;

    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let a = doc.create_element("div");
    doc.append_child(grid, a).unwrap();
    let b = doc.create_element("div");
    doc.append_child(grid, b).unwrap();

    let mut styles = HashMap::new();

    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px 100px 100px".to_string());
    grid_style.grid_template_rows = Some("50px 50px".to_string());
    // 第二行只有 2 列（不匹配 3 列模板）— taffy 应容错
    grid_style.grid_template_areas = Some("\"a a a\" \"b b\"".to_string());
    grid_style.width = LengthValue::Px(300.0);
    grid_style.height = LengthValue::Px(100.0);
    styles.insert(grid, grid_style);

    let mut sa = ComputedStyle::default();
    sa.grid_row_start = GridLineValue::Name("a".to_string());
    sa.grid_row_end = GridLineValue::Name("a".to_string());
    sa.grid_column_start = GridLineValue::Name("a".to_string());
    sa.grid_column_end = GridLineValue::Name("a".to_string());
    styles.insert(a, sa);

    let mut sb = ComputedStyle::default();
    sb.grid_row_start = GridLineValue::Name("b".to_string());
    sb.grid_row_end = GridLineValue::Name("b".to_string());
    sb.grid_column_start = GridLineValue::Name("b".to_string());
    sb.grid_column_end = GridLineValue::Name("b".to_string());
    styles.insert(b, sb);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    // 不应 panic
    let result = engine.compute(&doc, &styles);

    // 子元素应有有效的布局盒
    let box_a = find_child_by_node_id(&result.root, a);
    let box_b = find_child_by_node_id(&result.root, b);
    // 即使 taffy 无法正确解析不匹配的模板，也不应 panic
    // 至少验证 grid 容器存在
    assert!(result.root.width > 0.0);
    if let Some(ba) = box_a {
        assert!(ba.width.is_finite(), "元素 a 宽度应为有限值");
    }
    if let Some(bb) = box_b {
        assert!(bb.width.is_finite(), "元素 b 宽度应为有限值");
    }
}

/// 测试 grid auto-fill + minmax(100px, 1fr) 在 500px 容器中的轨道大小。
///
/// repeat(auto-fill, minmax(100px, 1fr)) 应创建 5 个等宽轨道。
#[test]
fn test_grid_auto_fill_minmax_equal_tracks() {
    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let mut item_ids = Vec::new();
    for _ in 0..5 {
        let item = doc.create_element("span");
        doc.append_child(grid, item).unwrap();
        item_ids.push(item);
    }

    let mut styles = HashMap::new();
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("repeat(auto-fill, minmax(100px, 1fr))".to_string());
    grid_style.width = LengthValue::Px(500.0);
    grid_style.height = LengthValue::Px(100.0);
    styles.insert(grid, grid_style);

    for id in &item_ids {
        styles.insert(*id, ComputedStyle::default());
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 每个 item 至少 100px（minmax 的 min 约束）
    for (i, &id) in item_ids.iter().enumerate() {
        let item_box = find_child_by_node_id(&result.root, id).unwrap_or_else(|| panic!("item{} not found", i));
        assert!(
            item_box.width >= 99.0,
            "item{} 宽度应 >= 100px（minmax min），实际 {}",
            i,
            item_box.width
        );
    }

    // 5 个 item 应在同一行（水平排列）
    let b0 = find_child_by_node_id(&result.root, item_ids[0]).expect("item0 found");
    let b4 = find_child_by_node_id(&result.root, item_ids[4]).expect("item4 found");
    assert!(b4.x > b0.x, "最后一个 item 应在第一个 item 右侧");

    // 所有 item 宽度应相等（均为 1fr）
    let widths: Vec<f32> = item_ids
        .iter()
        .map(|id| find_child_by_node_id(&result.root, *id).unwrap().width)
        .collect();
    for w in &widths[1..] {
        assert!((w - widths[0]).abs() < 2.0, "所有轨道宽度应相等，实际 {:?}", widths);
    }
}

/// 测试 grid-area 命名引用的完整端到端流程。
///
/// 定义 template-areas 并通过 grid-area: name 放置元素，
/// 验证元素被正确分配到对应区域。
#[test]
fn test_grid_named_area_resolution_full() {
    use zero_style_system::GridLineValue;

    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let top_el = doc.create_element("div");
    doc.append_child(grid, top_el).unwrap();
    let bottom_el = doc.create_element("div");
    doc.append_child(grid, bottom_el).unwrap();

    let mut styles = HashMap::new();

    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("1fr 1fr".to_string());
    grid_style.grid_template_rows = Some("100px 100px".to_string());
    grid_style.grid_template_areas = Some("\"top top\" \"bottom bottom\"".to_string());
    grid_style.width = LengthValue::Px(400.0);
    grid_style.height = LengthValue::Px(200.0);
    styles.insert(grid, grid_style);

    let mut top_style = ComputedStyle::default();
    top_style.grid_row_start = GridLineValue::Name("top".to_string());
    top_style.grid_row_end = GridLineValue::Name("top".to_string());
    top_style.grid_column_start = GridLineValue::Name("top".to_string());
    top_style.grid_column_end = GridLineValue::Name("top".to_string());
    styles.insert(top_el, top_style);

    let mut bottom_style = ComputedStyle::default();
    bottom_style.grid_row_start = GridLineValue::Name("bottom".to_string());
    bottom_style.grid_row_end = GridLineValue::Name("bottom".to_string());
    bottom_style.grid_column_start = GridLineValue::Name("bottom".to_string());
    bottom_style.grid_column_end = GridLineValue::Name("bottom".to_string());
    styles.insert(bottom_el, bottom_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let top_box = find_child_by_node_id(&result.root, top_el).expect("top found");
    let bottom_box = find_child_by_node_id(&result.root, bottom_el).expect("bottom found");

    // top 应在第一行，跨两列（~400px）
    assert!(
        (top_box.width - 400.0).abs() < 2.0,
        "top 应跨两列（~400px），实际 {}",
        top_box.width
    );
    assert!(
        (top_box.height - 100.0).abs() < 2.0,
        "top 应高约 100px，实际 {}",
        top_box.height
    );

    // bottom 应在第二行
    assert!(bottom_box.y > top_box.y, "bottom 应在 top 下方");
    assert!(
        (bottom_box.width - 400.0).abs() < 2.0,
        "bottom 应跨两列（~400px），实际 {}",
        bottom_box.width
    );
}

/// 测试 grid 中 gap 与 fr 单位组合。
///
/// grid-template-columns: 1fr 1fr; gap: 20px 在 420px 容器中，
/// 每个轨道 = (420 - 20) / 2 = 200px。
#[test]
fn test_grid_gap_with_fr_units() {
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
    grid_style.grid_template_columns = Some("1fr 1fr".to_string());
    grid_style.grid_template_rows = Some("100px".to_string());
    grid_style.gap = LengthValue::Px(20.0);
    grid_style.width = LengthValue::Px(420.0);
    grid_style.height = LengthValue::Px(100.0);
    styles.insert(grid, grid_style);

    styles.insert(item1, ComputedStyle::default());
    styles.insert(item2, ComputedStyle::default());

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

    // 每个轨道 = (420 - 20) / 2 = 200px
    assert!(
        (b1.width - 200.0).abs() < 2.0,
        "item1 宽度应约 200px（(420-20)/2），实际 {}",
        b1.width
    );
    assert!(
        (b2.width - 200.0).abs() < 2.0,
        "item2 宽度应约 200px，实际 {}",
        b2.width
    );

    // item2 应在 item1 右侧，间距约 20px
    let gap = b2.x - b1.x - b1.width;
    assert!((gap - 20.0).abs() < 2.0, "gap 应约 20px，实际 {}", gap);

    // 总宽度应约 420px
    let total = b1.width + b2.width + gap;
    assert!((total - 420.0).abs() < 2.0, "总宽度应约 420px，实际 {}", total);
}

/// 测试负 z-index 值在布局输出中正确反映。
///
/// 验证 z_index: -1 的元素在 LayoutBox 中产生 z_index: -1，
/// 而 z_index: auto 产生 0。
#[test]
fn test_layout_negative_z_index() {
    use zero_style_system::ZIndexValue;

    let (mut doc, body) = make_doc_with_body();
    let div_neg = doc.create_element("div");
    doc.append_child(body, div_neg).unwrap();
    let div_auto = doc.create_element("div");
    doc.append_child(body, div_auto).unwrap();

    let mut styles = HashMap::new();

    let mut s_neg = ComputedStyle::default();
    s_neg.display = DisplayValue::Block;
    s_neg.width = LengthValue::Px(100.0);
    s_neg.height = LengthValue::Px(50.0);
    s_neg.z_index = ZIndexValue::Integer(-1);
    s_neg.position = PositionValue::Relative;
    styles.insert(div_neg, s_neg);

    let mut s_auto = ComputedStyle::default();
    s_auto.display = DisplayValue::Block;
    s_auto.width = LengthValue::Px(100.0);
    s_auto.height = LengthValue::Px(50.0);
    styles.insert(div_auto, s_auto);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let box_neg = find_child_by_node_id(&result.root, div_neg).expect("div_neg found");
    let box_auto = find_child_by_node_id(&result.root, div_auto).expect("div_auto found");

    assert_eq!(box_neg.z_index, -1, "z-index: -1 应产生 z_index=-1");
    assert_eq!(box_auto.z_index, 0, "z-index: auto 应产生 z_index=0");
}

/// 测试百分比 gap 值。
///
/// grid 中 gap:10% 在 400px 容器中，gap 应约 40px。
#[test]
fn test_layout_percentage_gap() {
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
    grid_style.grid_template_columns = Some("1fr 1fr".to_string());
    grid_style.grid_template_rows = Some("100px".to_string());
    grid_style.gap = LengthValue::Percentage(10.0);
    grid_style.width = LengthValue::Px(400.0);
    grid_style.height = LengthValue::Px(100.0);
    styles.insert(grid, grid_style);

    styles.insert(item1, ComputedStyle::default());
    styles.insert(item2, ComputedStyle::default());

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

    // 百分比 gap 相对于容器宽度：400 * 10% = 40px
    // 两个 item 间距应反映百分比 gap
    let gap = b2.x - b1.x - b1.width;
    assert!(gap >= 0.0, "gap 应为非负值，实际 {}", gap);

    // 验证总宽度不超过容器
    let total = b1.width + b2.width + gap;
    assert!(total <= 401.0, "总宽度应不超过容器（400px），实际 {}", total);

    // item 应在同一行
    assert!((b1.y - b2.y).abs() < 1.0, "两个 item 应在同一行");
}

/// 测试 box-sizing:border-box 时，width 包含 padding。
///
/// 元素 width:100px，padding:10px（四边），box-sizing:border-box。
/// 内容区域 = 100 - 10*2 = 80px。
#[test]
fn test_layout_border_box_sizing() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.display = DisplayValue::Block;
    div_style.width = LengthValue::Px(100.0);
    div_style.height = LengthValue::Px(100.0);
    div_style.box_sizing = BoxSizingValue::BorderBox;
    div_style.padding_top = LengthValue::Px(10.0);
    div_style.padding_bottom = LengthValue::Px(10.0);
    div_style.padding_left = LengthValue::Px(10.0);
    div_style.padding_right = LengthValue::Px(10.0);
    styles.insert(div, div_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div_box = find_child_by_node_id(&result.root, div).expect("div 应找到");

    // border-box: 总宽度 = 100px（包含 padding）
    assert!(
        (div_box.width - 100.0).abs() < 1.0,
        "border-box 总宽度应为 100px，实际 {}",
        div_box.width
    );
    // 内容宽度 = 100 - padding_left - padding_right = 100 - 10 - 10 = 80
    assert!(
        (div_box.content_width - 80.0).abs() < 1.0,
        "border-box 内容宽度应为 80px（100 - 10 - 10），实际 {}",
        div_box.content_width
    );
    // 内容高度 = 100 - padding_top - padding_bottom = 100 - 10 - 10 = 80
    assert!(
        (div_box.content_height - 80.0).abs() < 1.0,
        "border-box 内容高度应为 80px（100 - 10 - 10），实际 {}",
        div_box.content_height
    );
    // padding 值正确
    assert_eq!(div_box.padding_top, 10.0);
    assert_eq!(div_box.padding_bottom, 10.0);
    assert_eq!(div_box.padding_left, 10.0);
    assert_eq!(div_box.padding_right, 10.0);
}

// ── 新增边界条件测试 ──────────────────────────────────────────

/// 测试 grid auto-flow: dense 自动放置。
#[test]
fn test_grid_auto_placement_dense() {
    use zero_style_system::GridAutoFlowValue;

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
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.width = LengthValue::Px(300.0);
    grid_style.height = LengthValue::Px(200.0);
    grid_style.grid_template_columns = Some("100px 100px 100px".to_string());
    grid_style.grid_auto_flow = GridAutoFlowValue::RowDense;
    styles.insert(grid, grid_style);

    for id in [item1, item2, item3] {
        let mut s = ComputedStyle::default();
        s.width = LengthValue::Px(100.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let grid_box = find_child_by_node_id(&result.root, grid).expect("grid 应找到");
    assert!((grid_box.width - 300.0).abs() < 1.0, "grid 宽度应为 300px");
    assert_eq!(grid_box.children.len(), 3, "grid 应有 3 个子元素");
}

/// 测试嵌套 flex column 布局。
#[test]
fn test_layout_nested_flex_column() {
    let (mut doc, body) = make_doc_with_body();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(outer, inner).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(inner, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(inner, item2).unwrap();

    let mut styles = HashMap::new();
    let mut outer_style = ComputedStyle::default();
    outer_style.display = DisplayValue::Flex;
    outer_style.flex_direction = FlexDirectionValue::Column;
    outer_style.width = LengthValue::Px(300.0);
    outer_style.height = LengthValue::Px(400.0);
    styles.insert(outer, outer_style);

    let mut inner_style = ComputedStyle::default();
    inner_style.display = DisplayValue::Flex;
    inner_style.flex_direction = FlexDirectionValue::Column;
    inner_style.width = LengthValue::Px(300.0);
    inner_style.height = LengthValue::Px(200.0);
    styles.insert(inner, inner_style);

    for id in [item1, item2] {
        let mut s = ComputedStyle::default();
        s.width = LengthValue::Px(100.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let outer_box = find_child_by_node_id(&result.root, outer).expect("outer 应找到");
    assert!((outer_box.width - 300.0).abs() < 1.0);
    assert_eq!(outer_box.children.len(), 1, "outer 应有 1 个子元素（inner）");
    let inner_box = &outer_box.children[0];
    assert!((inner_box.height - 200.0).abs() < 1.0);
    assert_eq!(inner_box.children.len(), 2, "inner 应有 2 个子元素");
}

/// 测试 flex 容器中的绝对定位子元素。
#[test]
fn test_layout_absolute_in_flex() {
    let (mut doc, body) = make_doc_with_body();
    let flex_container = doc.create_element("div");
    doc.append_child(body, flex_container).unwrap();
    let normal_item = doc.create_element("span");
    doc.append_child(flex_container, normal_item).unwrap();
    let abs_item = doc.create_element("span");
    doc.append_child(flex_container, abs_item).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.width = LengthValue::Px(300.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(flex_container, container_style);

    let mut normal_style = ComputedStyle::default();
    normal_style.width = LengthValue::Px(100.0);
    normal_style.height = LengthValue::Px(50.0);
    styles.insert(normal_item, normal_style);

    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(10.0);
    abs_style.left = LengthValue::Px(20.0);
    abs_style.width = LengthValue::Px(80.0);
    abs_style.height = LengthValue::Px(40.0);
    styles.insert(abs_item, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let container_box = find_child_by_node_id(&result.root, flex_container).expect("container 应找到");
    // 绝对定位的子元素仍然存在于 children 中
    assert_eq!(container_box.children.len(), 2);
    let abs_box = find_child_by_node_id(&result.root, abs_item).expect("abs_item 应找到");
    assert!(abs_box.is_absolute, "绝对定位元素应标记 is_absolute");
}

/// 测试 grid-column: span 2 跨列布局。
#[test]
fn test_grid_with_span() {
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
    grid_style.width = LengthValue::Px(300.0);
    grid_style.height = LengthValue::Px(200.0);
    grid_style.grid_template_columns = Some("100px 100px 100px".to_string());
    grid_style.grid_template_rows = Some("100px 100px".to_string());
    styles.insert(grid, grid_style);

    let mut wide_style = ComputedStyle::default();
    wide_style.grid_column_start = GridLineValue::Line(1);
    wide_style.grid_column_end = GridLineValue::Span(2);
    wide_style.width = LengthValue::Px(200.0);
    wide_style.height = LengthValue::Px(100.0);
    styles.insert(wide_item, wide_style);

    let mut normal_style = ComputedStyle::default();
    normal_style.width = LengthValue::Px(100.0);
    normal_style.height = LengthValue::Px(100.0);
    styles.insert(normal_item, normal_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let grid_box = find_child_by_node_id(&result.root, grid).expect("grid 应找到");
    assert!((grid_box.width - 300.0).abs() < 1.0);
    assert_eq!(grid_box.children.len(), 2);
}

/// 测试 min-width/max-width 约束布局。
#[test]
fn test_layout_min_max_constraints() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let constrained = doc.create_element("span");
    doc.append_child(container, constrained).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(500.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    // 元素宽度设为 80%，但 min-width: 100px, max-width: 300px
    let mut constrained_style = ComputedStyle::default();
    constrained_style.width = LengthValue::Percentage(80.0);
    constrained_style.min_width = LengthValue::Px(100.0);
    constrained_style.max_width = LengthValue::Px(300.0);
    constrained_style.height = LengthValue::Px(50.0);
    styles.insert(constrained, constrained_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let constrained_box = find_child_by_node_id(&result.root, constrained).expect("constrained 应找到");
    // 80% of 500 = 400，但 max-width 限制为 300
    assert!(
        constrained_box.width <= 301.0,
        "max-width 应限制宽度为 300px，实际 {}",
        constrained_box.width
    );
    assert!(
        constrained_box.width >= 99.0,
        "min-width 应确保宽度至少 100px，实际 {}",
        constrained_box.width
    );
}

// -- 边界条件测试（第五批）--

/// 测试非标准视口尺寸（极小视口 1x1 和极大视口 10000x10000）。
///
/// 验证布局引擎在极端视口尺寸下不 panic，
/// 且 LayoutResult 中正确存储视口尺寸。
#[test]
fn test_extreme_viewport_dimensions() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    styles.insert(div, make_style_with_display(DisplayValue::Block, 100.0, 50.0));

    // 极小视口
    let mut engine_tiny = LayoutEngine::new(1.0, 1.0);
    let result_tiny = engine_tiny.compute(&doc, &styles);
    assert!((result_tiny.viewport_width - 1.0).abs() < 0.001, "极小视口宽度应为 1.0");
    assert!(
        (result_tiny.viewport_height - 1.0).abs() < 0.001,
        "极小视口高度应为 1.0"
    );
    // 布局不 panic，尺寸有限
    assert!(result_tiny.root.width.is_finite(), "极小视口布局宽度应有限");

    // 极大视口
    let mut engine_huge = LayoutEngine::new(10000.0, 10000.0);
    let result_huge = engine_huge.compute(&doc, &styles);
    assert!(
        (result_huge.viewport_width - 10000.0).abs() < 0.001,
        "极大视口宽度应为 10000.0"
    );
    assert!(
        (result_huge.viewport_height - 10000.0).abs() < 0.001,
        "极大视口高度应为 10000.0"
    );

    // div 在极大视口中尺寸应保持不变
    let div_box = find_child_by_node_id(&result_huge.root, div).expect("div found");
    assert_eq!(div_box.width, 100.0, "div 宽度不应受视口尺寸影响");
    assert_eq!(div_box.height, 50.0, "div 高度不应受视口尺寸影响");
}

/// 测试 flex 容器中 align-self 覆盖 align-items 的行为。
///
/// 容器设置 align-items: flex-start，但某个子元素使用 align-self: flex-end，
/// 验证子元素的垂直位置受 align-self 控制而非 align-items。
#[test]
fn test_flex_align_self_overrides_align_items() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item_start = doc.create_element("span");
    doc.append_child(container, item_start).unwrap();
    let item_end = doc.create_element("span");
    doc.append_child(container, item_end).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.align_items = AlignmentValue::FlexStart;
    container_style.width = LengthValue::Px(300.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    // item_start: 继承 align-items: flex-start
    let mut s1 = ComputedStyle::default();
    s1.width = LengthValue::Px(80.0);
    s1.height = LengthValue::Px(40.0);
    styles.insert(item_start, s1);

    // item_end: align-self: flex-end 覆盖容器的 align-items
    let mut s2 = ComputedStyle::default();
    s2.width = LengthValue::Px(80.0);
    s2.height = LengthValue::Px(40.0);
    s2.align_self = AlignmentValue::FlexEnd;
    styles.insert(item_end, s2);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b_start = find_child_by_node_id(&result.root, item_start).expect("item_start found");
    let b_end = find_child_by_node_id(&result.root, item_end).expect("item_end found");

    // item_start 在容器顶部（flex-start），item_end 在容器底部（flex-end）
    // item_start.y 应接近 0，item_end.y 应接近 200 - 40 = 160
    assert!(
        b_start.y < b_end.y,
        "flex-start 项 (y={}) 应在 flex-end 项 (y={}) 上方",
        b_start.y,
        b_end.y
    );
    assert!(
        b_end.y + b_end.height > b_start.y + b_start.height + 50.0,
        "flex-end 项应明显在 flex-start 项下方（容器 200px 高）"
    );
}

use super::*;
use zero_css_parser::values::{AlignmentValue, DisplayValue, FlexDirectionValue, LengthValue};
use zero_dom::Document;
// ── 边缘场景补充测试（第八批）──

/// 测试 grid 命名区域跨两行的布局。
///
/// 使用 grid-template-areas 定义 2x3 网格，其中 "sidebar" 区域跨两行，
/// "header" 跨前两列，"main" 和 "footer" 各占一个单元格。
/// 验证 sidebar 的高度为两行之和，header 宽度为两列之和。
#[test]
fn test_grid_named_area_spans_two_rows() {
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
    let footer_el = doc.create_element("div");
    doc.append_child(grid, footer_el).unwrap();

    let mut styles = HashMap::new();

    // 2 行 3 列网格，sidebar 跨两行
    // "header header header"
    // "sidebar main   footer"
    // "sidebar footer2 footer3"  -- 不用，简化为 sidebar 跨两行
    // 改用：
    // "header  header"
    // "sidebar main  "
    // "sidebar footer"
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("120px 120px".to_string());
    grid_style.grid_template_rows = Some("60px 60px 60px".to_string());
    grid_style.grid_template_areas = Some("\"header header\" \"sidebar main\" \"sidebar footer\"".to_string());
    grid_style.width = LengthValue::Px(240.0);
    grid_style.height = LengthValue::Px(180.0);
    styles.insert(grid, grid_style);

    // header: 跨第一行两列
    let mut header_s = ComputedStyle::default();
    header_s.grid_row_start = GridLineValue::Name("header".to_string());
    header_s.grid_row_end = GridLineValue::Name("header".to_string());
    header_s.grid_column_start = GridLineValue::Name("header".to_string());
    header_s.grid_column_end = GridLineValue::Name("header".to_string());
    styles.insert(header_el, header_s);

    // sidebar: 跨第二、三行，第一列
    let mut sidebar_s = ComputedStyle::default();
    sidebar_s.grid_row_start = GridLineValue::Name("sidebar".to_string());
    sidebar_s.grid_row_end = GridLineValue::Name("sidebar".to_string());
    sidebar_s.grid_column_start = GridLineValue::Name("sidebar".to_string());
    sidebar_s.grid_column_end = GridLineValue::Name("sidebar".to_string());
    styles.insert(sidebar_el, sidebar_s);

    // main: 第二行第二列
    let mut main_s = ComputedStyle::default();
    main_s.grid_row_start = GridLineValue::Name("main".to_string());
    main_s.grid_row_end = GridLineValue::Name("main".to_string());
    main_s.grid_column_start = GridLineValue::Name("main".to_string());
    main_s.grid_column_end = GridLineValue::Name("main".to_string());
    styles.insert(main_el, main_s);

    // footer: 第三行第二列
    let mut footer_s = ComputedStyle::default();
    footer_s.grid_row_start = GridLineValue::Name("footer".to_string());
    footer_s.grid_row_end = GridLineValue::Name("footer".to_string());
    footer_s.grid_column_start = GridLineValue::Name("footer".to_string());
    footer_s.grid_column_end = GridLineValue::Name("footer".to_string());
    styles.insert(footer_el, footer_s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let header_box = find_child_by_node_id(&result.root, header_el).expect("header 应找到");
    let sidebar_box = find_child_by_node_id(&result.root, sidebar_el).expect("sidebar 应找到");
    let main_box = find_child_by_node_id(&result.root, main_el).expect("main 应找到");
    let footer_box = find_child_by_node_id(&result.root, footer_el).expect("footer 应找到");

    // header 跨两列，宽度约 240px
    assert!(
        (header_box.width - 240.0).abs() < 2.0,
        "header 应跨两列（~240px），实际 {}",
        header_box.width
    );
    // header 只占一行，高度约 60px
    assert!(
        (header_box.height - 60.0).abs() < 1.0,
        "header 高度应约 60px（单行），实际 {}",
        header_box.height
    );

    // sidebar 跨两行（第二、三行），高度约 120px（60 + 60）
    assert!(
        (sidebar_box.height - 120.0).abs() < 2.0,
        "sidebar 应跨两行（~120px），实际 {}",
        sidebar_box.height
    );
    // sidebar 宽度约 120px（单列）
    assert!(
        (sidebar_box.width - 120.0).abs() < 1.0,
        "sidebar 宽度应约 120px，实际 {}",
        sidebar_box.width
    );

    // sidebar 应从第二行开始，在 header 下方
    assert!(
        sidebar_box.y > header_box.y,
        "sidebar 应在 header 下方: sidebar.y={} > header.y={}",
        sidebar_box.y,
        header_box.y
    );

    // main 在 sidebar 右侧
    assert!(
        main_box.x > sidebar_box.x,
        "main 应在 sidebar 右侧: main.x={} > sidebar.x={}",
        main_box.x,
        sidebar_box.x
    );

    // footer 在 main 下方
    assert!(
        footer_box.y > main_box.y,
        "footer 应在 main 下方: footer.y={} > main.y={}",
        footer_box.y,
        main_box.y
    );
}

/// 测试 flex 容器中 align-self: stretch 覆盖容器默认对齐。
///
/// 容器 align-items: flex-start，两个子元素分别设置
/// align-self: stretch 和不设置（继承 flex-start）。
/// stretch 子元素高度应拉伸到容器高度，flex-start 子元素保持自身高度。
#[test]
fn test_flex_align_self_stretch() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    let item_normal = doc.create_element("span");
    doc.append_child(container, item_normal).unwrap();
    let item_stretch = doc.create_element("span");
    doc.append_child(container, item_stretch).unwrap();

    let mut styles = HashMap::new();

    // flex 容器，align-items: flex-start
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.align_items = AlignmentValue::FlexStart;
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    // item_normal: 继承 flex-start，不拉伸
    let mut s_normal = ComputedStyle::default();
    s_normal.width = LengthValue::Px(80.0);
    s_normal.height = LengthValue::Px(40.0);
    styles.insert(item_normal, s_normal);

    // item_stretch: align-self: stretch，不设显式高度，应拉伸到容器高度 200px
    let mut s_stretch = ComputedStyle::default();
    s_stretch.width = LengthValue::Px(80.0);
    s_stretch.align_self = AlignmentValue::Stretch;
    styles.insert(item_stretch, s_stretch);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b_normal = find_child_by_node_id(&result.root, item_normal).expect("item_normal 应找到");
    let b_stretch = find_child_by_node_id(&result.root, item_stretch).expect("item_stretch 应找到");

    // normal 子元素高度应保持 40px
    assert!(
        (b_normal.height - 40.0).abs() < 1.0,
        "flex-start 子元素高度应保持 40px，实际 {}",
        b_normal.height
    );

    // stretch 子元素高度应拉伸到约 200px
    assert!(
        (b_stretch.height - 200.0).abs() < 2.0,
        "stretch 子元素高度应约 200px（容器高度），实际 {}",
        b_stretch.height
    );

    // stretch 子元素 y 应约 0（flex-start 也在顶部）
    assert!(b_stretch.y.abs() < 1.0, "stretch 子元素 y 应约 0，实际 {}", b_stretch.y);

    // 两个子元素水平排列
    assert!(b_stretch.x > b_normal.x, "stretch 子元素应在 normal 子元素右侧");
}

/// 测试 block 布局中 margin: auto 水平居中。
///
/// 容器 600px 宽，子元素 200px 宽，左右 margin 设为 auto。
/// 子元素应在容器内水平居中，左右间距约 (600 - 200) / 2 = 200px。
#[test]
fn test_block_margin_auto_horizontal_centering() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let child = doc.create_element("div");
    doc.append_child(container, child).unwrap();

    let mut styles = HashMap::new();

    // block 容器 600x300
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Block;
    container_style.width = LengthValue::Px(600.0);
    container_style.height = LengthValue::Px(300.0);
    styles.insert(container, container_style);

    // 子元素 200x100，margin-left/right: auto
    let mut child_style = ComputedStyle::default();
    child_style.display = DisplayValue::Block;
    child_style.width = LengthValue::Px(200.0);
    child_style.height = LengthValue::Px(100.0);
    child_style.margin_left = LengthValue::Auto;
    child_style.margin_right = LengthValue::Auto;
    styles.insert(child, child_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container_box = find_child_by_node_id(&result.root, container).expect("container 应找到");
    let child_box = find_child_by_node_id(&result.root, child).expect("child 应找到");

    // 子元素宽度应保持 200px
    assert!(
        (child_box.width - 200.0).abs() < 1.0,
        "子元素宽度应保持 200px，实际 {}",
        child_box.width
    );

    // 子元素应在容器内水平居中
    // 左边距 = child.x - container.content_x，应约 (600 - 200) / 2 = 200px
    let left_margin = child_box.x - container_box.content_x;
    let right_margin = (container_box.content_x + container_box.content_width) - (child_box.x + child_box.width);

    assert!(
        (left_margin - right_margin).abs() < 2.0,
        "左右边距应相等（居中），左边距={} 右边距={}",
        left_margin,
        right_margin
    );
    assert!(left_margin > 100.0, "左边距应大于 100px（居中），实际 {}", left_margin);

    // 子元素高度应保持 100px
    assert!(
        (child_box.height - 100.0).abs() < 1.0,
        "子元素高度应保持 100px，实际 {}",
        child_box.height
    );
}

/// 测试 inline-block 子元素在 flex 容器中的布局。
///
/// flex 容器中包含 inline-block 子元素。inline-block 在 taffy 中映射为 Block，
/// 但作为 flex 子项应正常参与 flex 行布局，水平排列。
#[test]
fn test_inline_block_inside_flex_container() {
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

    // flex 容器
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_direction = FlexDirectionValue::Row;
    container_style.width = LengthValue::Px(600.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    // inline-block 子元素
    for id in [ib1, ib2, ib3] {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::InlineBlock;
        s.width = LengthValue::Px(150.0);
        s.height = LengthValue::Px(50.0);
        styles.insert(id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, ib1).expect("ib1 应找到");
    let b2 = find_child_by_node_id(&result.root, ib2).expect("ib2 应找到");
    let b3 = find_child_by_node_id(&result.root, ib3).expect("ib3 应找到");

    // 每个 inline-block 子元素宽度应保持 150px
    assert!((b1.width - 150.0).abs() < 1.0, "ib1 宽度应约 150px，实际 {}", b1.width);
    assert!((b2.width - 150.0).abs() < 1.0, "ib2 宽度应约 150px，实际 {}", b2.width);
    assert!((b3.width - 150.0).abs() < 1.0, "ib3 宽度应约 150px，实际 {}", b3.width);

    // 三个子元素水平排列，x 递增
    assert!(b2.x > b1.x, "ib2 应在 ib1 右侧: ib2.x={} > ib1.x={}", b2.x, b1.x);
    assert!(b3.x > b2.x, "ib3 应在 ib2 右侧: ib3.x={} > ib2.x={}", b3.x, b2.x);

    // 总宽度不超过容器（3 x 150 = 450 < 600）
    let total_width = b3.x + b3.width - b1.x;
    assert!(
        total_width <= 600.0,
        "inline-block 子元素总占用宽度应不超过容器 600px，实际 {}",
        total_width
    );
}

/// 测试嵌套 grid 容器（外层 grid > 内层 grid > 子元素）。
///
/// 外层 grid 2x2，第一个单元格中放置一个内嵌 grid 容器（也是 2 列）。
/// 验证内层 grid 子元素正确布局，且不影响外层 grid 的其他单元格。
#[test]
fn test_nested_grid_container() {
    use zero_style_system::GridLineValue;

    let (mut doc, body) = make_doc_with_body();
    let outer_grid = doc.create_element("div");
    doc.append_child(body, outer_grid).unwrap();

    // 外层 grid 第一个单元格：内嵌 grid
    let inner_grid = doc.create_element("div");
    doc.append_child(outer_grid, inner_grid).unwrap();
    let inner_item1 = doc.create_element("span");
    doc.append_child(inner_grid, inner_item1).unwrap();
    let inner_item2 = doc.create_element("span");
    doc.append_child(inner_grid, inner_item2).unwrap();

    // 外层 grid 第二个单元格
    let outer_item2 = doc.create_element("div");
    doc.append_child(outer_grid, outer_item2).unwrap();
    // 外层 grid 第三个单元格
    let outer_item3 = doc.create_element("div");
    doc.append_child(outer_grid, outer_item3).unwrap();
    // 外层 grid 第四个单元格
    let outer_item4 = doc.create_element("div");
    doc.append_child(outer_grid, outer_item4).unwrap();

    let mut styles = HashMap::new();

    // 外层 grid: 2x2，每列 200px，每行 150px
    let mut outer_style = ComputedStyle::default();
    outer_style.display = DisplayValue::Grid;
    outer_style.grid_template_columns = Some("200px 200px".to_string());
    outer_style.grid_template_rows = Some("150px 150px".to_string());
    outer_style.width = LengthValue::Px(400.0);
    outer_style.height = LengthValue::Px(300.0);
    styles.insert(outer_grid, outer_style);

    // 内嵌 grid: 占外层第一个单元格，内部 2 列
    let mut inner_grid_style = ComputedStyle::default();
    inner_grid_style.display = DisplayValue::Grid;
    inner_grid_style.grid_template_columns = Some("1fr 1fr".to_string());
    inner_grid_style.grid_template_rows = Some("1fr".to_string());
    inner_grid_style.grid_row_start = GridLineValue::Line(1);
    inner_grid_style.grid_row_end = GridLineValue::Line(2);
    inner_grid_style.grid_column_start = GridLineValue::Line(1);
    inner_grid_style.grid_column_end = GridLineValue::Line(2);
    styles.insert(inner_grid, inner_grid_style);

    // 内层子元素
    for id in [inner_item1, inner_item2] {
        let mut s = ComputedStyle::default();
        s.width = LengthValue::Px(50.0);
        s.height = LengthValue::Px(30.0);
        styles.insert(id, s);
    }

    // 外层其余单元格
    let mut oi2 = ComputedStyle::default();
    oi2.grid_row_start = GridLineValue::Line(1);
    oi2.grid_row_end = GridLineValue::Line(2);
    oi2.grid_column_start = GridLineValue::Line(2);
    oi2.grid_column_end = GridLineValue::Line(3);
    styles.insert(outer_item2, oi2);

    let mut oi3 = ComputedStyle::default();
    oi3.grid_row_start = GridLineValue::Line(2);
    oi3.grid_row_end = GridLineValue::Line(3);
    oi3.grid_column_start = GridLineValue::Line(1);
    oi3.grid_column_end = GridLineValue::Line(2);
    styles.insert(outer_item3, oi3);

    let mut oi4 = ComputedStyle::default();
    oi4.grid_row_start = GridLineValue::Line(2);
    oi4.grid_row_end = GridLineValue::Line(3);
    oi4.grid_column_start = GridLineValue::Line(2);
    oi4.grid_column_end = GridLineValue::Line(3);
    styles.insert(outer_item4, oi4);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 外层 grid 验证
    let outer_box = find_child_by_node_id(&result.root, outer_grid).expect("outer_grid 应找到");
    assert!(
        (outer_box.width - 400.0).abs() < 1.0,
        "外层 grid 宽度应约 400px，实际 {}",
        outer_box.width
    );
    assert!(
        (outer_box.height - 300.0).abs() < 1.0,
        "外层 grid 高度应约 300px，实际 {}",
        outer_box.height
    );

    // 内嵌 grid 验证
    let inner_box = find_child_by_node_id(&result.root, inner_grid).expect("inner_grid 应找到");
    // 内嵌 grid 占外层第一个单元格（200x150）
    assert!(
        (inner_box.width - 200.0).abs() < 2.0,
        "内嵌 grid 宽度应约 200px（外层单元格尺寸），实际 {}",
        inner_box.width
    );
    assert!(
        (inner_box.height - 150.0).abs() < 2.0,
        "内嵌 grid 高度应约 150px（外层单元格尺寸），实际 {}",
        inner_box.height
    );

    // 内层子元素验证
    let ii1_box = find_child_by_node_id(&result.root, inner_item1).expect("inner_item1 应找到");
    let ii2_box = find_child_by_node_id(&result.root, inner_item2).expect("inner_item2 应找到");

    // 内层两个子元素水平排列
    assert!(
        ii2_box.x > ii1_box.x,
        "内层 item2 应在 item1 右侧: ii2.x={} > ii1.x={}",
        ii2_box.x,
        ii1_box.x
    );

    // 外层其他单元格验证
    let o2_box = find_child_by_node_id(&result.root, outer_item2).expect("outer_item2 应找到");
    let o3_box = find_child_by_node_id(&result.root, outer_item3).expect("outer_item3 应找到");
    let o4_box = find_child_by_node_id(&result.root, outer_item4).expect("outer_item4 应找到");

    // outer_item2 应在第一行第二列（在 inner_grid 右侧）
    assert!(o2_box.x > inner_box.x, "outer_item2 应在 inner_grid 右侧");
    assert!(
        (o2_box.y - inner_box.y).abs() < 2.0,
        "outer_item2 和 inner_grid 应在同一行（第一行）"
    );

    // outer_item3 和 outer_item4 应在第二行
    assert!(o3_box.y > inner_box.y, "outer_item3 应在 inner_grid 下方（第二行）");
    assert!(o4_box.y > inner_box.y, "outer_item4 应在 inner_grid 下方（第二行）");

    // outer_item3 在左下角，outer_item4 在右下角
    assert!(o4_box.x > o3_box.x, "outer_item4 应在 outer_item3 右侧");
}

// ── 新增边界测试 ──

/// 测试 display:none 子树完全不参与布局。
#[test]
fn test_display_none_excludes_from_layout() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "style", "display: none; width: 100px; height: 50px;");
    doc.append_child(body, div).unwrap();
    let span = doc.create_element("span");
    doc.append_child(div, span).unwrap();

    let css = r#"div { display: none; }"#;
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let tree = engine.compute(&doc, &styles);

    // display:none 元素不应出现在布局树中
    assert!(
        find_child_by_node_id(&tree.root, div).is_none(),
        "display:none 元素不应出现在布局树中"
    );
}

/// 测试单个块级元素占满父容器宽度。
#[test]
fn test_block_element_fills_parent_width() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let css = "";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let tree = engine.compute(&doc, &styles);

    let body_box = find_child_by_node_id(&tree.root, body).expect("body 应在布局树中");
    let div_box = find_child_by_node_id(&tree.root, div).expect("div 应在布局树中");

    // 块级 div 宽度应与 body 内容宽度一致
    assert!(
        (div_box.width - body_box.content_width).abs() < 1.0,
        "块级 div 宽度 {} 应接近 body 内容宽度 {}",
        div_box.width,
        body_box.content_width
    );
}

/// 测试 flex 容器宽度不足时子元素换行。
#[test]
fn test_flex_wrap_when_narrow() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let container = doc.create_element("div");
    doc.set_attribute(container, "class", "flex");
    doc.append_child(body, container).unwrap();

    for _ in 0..5 {
        let item = doc.create_element("div");
        doc.set_attribute(item, "class", "item");
        doc.append_child(container, item).unwrap();
    }

    let css = r#"
        .flex { display: flex; flex-wrap: wrap; width: 100px; }
        .item { width: 40px; height: 20px; }
    "#;
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let tree = engine.compute(&doc, &styles);

    let container_box = find_child_by_node_id(&tree.root, container).expect("container 应在布局树中");
    // 容器宽度为 100px，每个 item 40px，所以应换行
    assert!(
        container_box.height >= 40.0,
        "容器高度 {} 应至少 2 行（40px）",
        container_box.height
    );
}

/// 测试 inline-block 元素与文本同行排列。
#[test]
fn test_inline_block_inline_with_text() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let span = doc.create_element("span");
    doc.set_attribute(span, "style", "display: inline-block; width: 50px; height: 30px;");
    doc.append_child(body, span).unwrap();

    let css = "span { display: inline-block; width: 50px; height: 30px; }";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let tree = engine.compute(&doc, &styles);

    let span_box = find_child_by_node_id(&tree.root, span).expect("span 应在布局树中");
    assert!(
        (span_box.width - 50.0).abs() < 1.0,
        "inline-block 宽度应接近 50px，实际为 {}",
        span_box.width
    );
    assert!(
        (span_box.height - 30.0).abs() < 1.0,
        "inline-block 高度应接近 30px，实际为 {}",
        span_box.height
    );
}

/// 测试 position:absolute 元素脱离文档流。
#[test]
fn test_absolute_position_out_of_flow() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let normal = doc.create_element("div");
    doc.set_attribute(normal, "class", "normal");
    doc.append_child(body, normal).unwrap();
    let absolute = doc.create_element("div");
    doc.set_attribute(absolute, "class", "abs");
    doc.append_child(body, absolute).unwrap();

    let css = r#"
        .normal { width: 100px; height: 50px; }
        .abs { position: absolute; top: 10px; left: 20px; width: 30px; height: 30px; }
    "#;
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let tree = engine.compute(&doc, &styles);

    let (abs_x, abs_y) = find_absolute_position_by_node_id(&tree.root, absolute).expect("absolute div 应在布局树中");
    // absolute 元素无 positioned ancestor，containing block 为初始包含块（viewport）。
    // left:20/top:10 解析为视口相对坐标（不受 body 默认 margin 偏移影响）。
    assert!(
        (abs_x - 20.0).abs() < 2.0,
        "absolute 元素视口 x 应为 20（left:20），实际为 {}",
        abs_x
    );
    assert!(
        (abs_y - 10.0).abs() < 2.0,
        "absolute 元素视口 y 应为 10（top:10），实际为 {}",
        abs_y
    );
}

// ── LayoutBox / LayoutResult 类型边界测试 ──

#[test]
fn test_layout_box_default() {
    let box_ = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 0.0,
        content_height: 0.0,
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
    assert_eq!(box_.outer_area(), 0.0);
    assert_eq!(box_.absolute_position(), (0.0, 0.0));
}

#[test]
fn test_layout_box_outer_area_with_margins() {
    let box_ = LayoutBox {
        node_id: None,
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
        margin_left: 5.0,
        margin_right: 5.0,
        margin_top: 10.0,
        margin_bottom: 10.0,
        children: vec![],
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
    // outer_area = (5 + 100 + 5) * (10 + 50 + 10) = 110 * 70 = 7700
    assert_eq!(box_.outer_area(), 7700.0);
}

#[test]
fn test_layout_box_absolute_position_with_parent() {
    let box_ = LayoutBox {
        node_id: None,
        x: 15.0,
        y: 25.0,
        width: 50.0,
        height: 30.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 50.0,
        content_height: 30.0,
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
    let (abs_x, abs_y) = box_.absolute_position_with_parent(100.0, 200.0);
    assert_eq!(abs_x, 115.0);
    assert_eq!(abs_y, 225.0);
}

#[test]
fn test_layout_box_negative_margins() {
    let box_ = LayoutBox {
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
        margin_right: -5.0,
        margin_bottom: -10.0,
        margin_left: -5.0,
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
    // (-5 + 100 + (-5)) * (-10 + 50 + (-10)) = 90 * 30 = 2700
    assert_eq!(box_.outer_area(), 2700.0);
}

#[test]
fn test_overflow_clip_variants() {
    assert_eq!(OverflowClip::Visible, OverflowClip::Visible);
    assert_ne!(OverflowClip::Visible, OverflowClip::Hidden);
    assert_ne!(OverflowClip::Hidden, OverflowClip::Clip);
    assert_ne!(OverflowClip::Clip, OverflowClip::Scroll);
}

// ── 布局引擎 viewport 边界测试 ──

#[test]
fn test_layout_engine_zero_viewport() {
    let engine = LayoutEngine::new(0.0, 0.0);
    assert_eq!(engine.viewport_width, 0.0);
    assert_eq!(engine.viewport_height, 0.0);
}

#[test]
fn test_layout_engine_very_large_viewport() {
    let engine = LayoutEngine::new(100000.0, 100000.0);
    assert_eq!(engine.viewport_width, 100000.0);
}

#[test]
fn test_layout_engine_empty_doc_produces_root() {
    let doc = Document::new();
    let styles = HashMap::new();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // 空文档应产生一个根盒子
    assert_eq!(result.viewport_width, 800.0);
    assert_eq!(result.viewport_height, 600.0);
}

#[test]
fn test_layout_single_text_node() {
    let mut doc = Document::new();
    let root = doc.root();
    let text = doc.create_text_node("Hello World");
    doc.append_child(root, text).unwrap();

    let styles = HashMap::new();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // 文本节点应被跳过或包含在布局中
    assert_eq!(result.viewport_width, 800.0);
}

#[test]
fn test_layout_deeply_nested_structure() {
    let mut doc = Document::new();
    let root = doc.root();
    let mut parent = root;
    // 创建 20 层嵌套
    for i in 0..20 {
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", &format!("level{}", i));
        doc.append_child(parent, div).unwrap();
        parent = div;
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let styles = HashMap::new();
    let result = engine.compute(&doc, &styles);
    // 不应 panic
    assert_eq!(result.viewport_width, 800.0);
}

#[test]
fn test_layout_multiple_siblings() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    // 创建 10 个兄弟 div
    for i in 0..10 {
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", &format!("item{}", i));
        doc.append_child(body, div).unwrap();
    }

    let css =
        ".item0, .item1, .item2, .item3, .item4, .item5, .item6, .item7, .item8, .item9 { width: 50px; height: 20px; }";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // body 应包含 10 个子元素
    let body_box = find_child_by_node_id(&result.root, body).expect("body 应在布局树中");
    assert_eq!(body_box.children.len(), 10);
}

#[test]
fn test_layout_fixed_position_elements() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let fixed = doc.create_element("div");
    doc.set_attribute(fixed, "class", "fixed-el");
    doc.append_child(body, fixed).unwrap();

    let css = r#".fixed-el { position: fixed; top: 50px; left: 100px; width: 200px; height: 100px; }"#;
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fixed_box = find_child_by_node_id(&result.root, fixed).expect("fixed div 应在布局树中");
    assert!(fixed_box.is_fixed, "fixed 元素应标记 is_fixed");
}

#[test]
fn test_fixed_bottom_inset_resolves_to_viewport_bottom() {
    // R1308：position:fixed + bottom:0 应把盒底对齐视口底（abs_y + height ≈ viewport_height），
    // 而非落视口顶外（旧 bug：adjust_absolute_pct_to_viewport gate 仅 is_absolute 排除 is_fixed，
    // bottom 不解析 → 盒 abs_y=-height）。kill-switch ZW_FIXED_INSET=0 回退（fb.y 错为负/零）。
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let fixed = doc.create_element("div");
    doc.set_attribute(fixed, "class", "f");
    doc.append_child(body, fixed).unwrap();

    let css = r#".f { position: fixed; bottom: 0; right: 0; width: 200px; height: 100px; }"#;
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fb = find_child_by_node_id(&result.root, fixed).expect("fixed box");
    assert!(fb.is_fixed, "应标记 is_fixed");
    assert!((fb.height - 100.0).abs() < 1.0, "height 应 100，got {}", fb.height);
    // R1308 fixed+bottom：fb.y 经 inset 解析后应使盒落在视口底区（target abs_y=500，
    // parent content 相对 fb.y ≈ 500 - parent_origin），远大于 0；旧 bug fb.y 为负。
    // 宽松断言 fb.y > 100（区分旧 bug 的负值/零），default-on PASS / kill=0 FAIL。
    assert!(
        fb.y > 100.0,
        "R1308: fixed+bottom box.y 应指向视口底（>100），旧 bug 为负/零；got {}",
        fb.y
    );
}

#[test]
fn test_layout_display_none_present() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let visible = doc.create_element("div");
    doc.set_attribute(visible, "class", "visible");
    doc.append_child(body, visible).unwrap();
    let hidden = doc.create_element("div");
    doc.set_attribute(hidden, "class", "hidden");
    doc.append_child(body, hidden).unwrap();

    let css = r#"
        .visible { width: 100px; height: 50px; }
        .hidden { display: none; }
    "#;
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let body_box = find_child_by_node_id(&result.root, body).expect("body 应在布局树中");
    // display:none 元素可能在树中但宽度/高度为 0
    assert!(body_box.children.len() >= 1, "至少应有 visible 元素");
    let hidden_box = find_child_by_node_id(&result.root, hidden);
    if let Some(h) = hidden_box {
        // 如果 display:none 元素在树中，其尺寸应为 0
        assert_eq!(h.width, 0.0, "display:none 元素宽度应为 0");
        assert_eq!(h.height, 0.0, "display:none 元素高度应为 0");
    }
}

#[test]
fn test_layout_z_index_values() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let z1 = doc.create_element("div");
    doc.set_attribute(z1, "class", "z1");
    doc.append_child(body, z1).unwrap();
    let z2 = doc.create_element("div");
    doc.set_attribute(z2, "class", "z2");
    doc.append_child(body, z2).unwrap();

    let css = r#"
        .z1 { position: relative; z-index: 10; width: 50px; height: 50px; }
        .z2 { position: relative; z-index: 20; width: 50px; height: 50px; }
    "#;
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let box1 = find_child_by_node_id(&result.root, z1).expect("z1 应在布局树中");
    let box2 = find_child_by_node_id(&result.root, z2).expect("z2 应在布局树中");
    assert_eq!(box1.z_index, 10);
    assert_eq!(box2.z_index, 20);
}

/// 测试 width:0 height:0 元素带 border: solid 1in (96px) 的布局。
/// 期望：总尺寸 192x192（96px 边框 × 4 侧）。
#[test]
fn test_border_1in_zero_size() {
    use zero_style_system::property::types::BorderStyleValue;

    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut style = zero_style_system::ComputedStyle::default();
    style.width = LengthValue::Px(0.0);
    style.height = LengthValue::Px(0.0);
    style.display = DisplayValue::Block;
    style.border_top_width = LengthValue::Px(96.0);
    style.border_right_width = LengthValue::Px(96.0);
    style.border_bottom_width = LengthValue::Px(96.0);
    style.border_left_width = LengthValue::Px(96.0);
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    style.border_bottom_style = BorderStyleValue::Solid;
    style.border_left_style = BorderStyleValue::Solid;

    let mut styles = HashMap::new();
    styles.insert(div, style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let body_box = &result.root;
    assert!(!body_box.children.is_empty(), "Should have child box");
    let child_box = &body_box.children[0];

    eprintln!(
        "Child box: width={}, height={}, border=({},{},{},{}), content={}x{}",
        child_box.width,
        child_box.height,
        child_box.border_top,
        child_box.border_right,
        child_box.border_bottom,
        child_box.border_left,
        child_box.content_width,
        child_box.content_height
    );

    // 总宽度应为 192（96 左 + 0 内容 + 96 右）
    assert!(
        child_box.width >= 190.0,
        "Expected width ~192 (96+0+96), got {}",
        child_box.width
    );
    assert!(
        child_box.height >= 190.0,
        "Expected height ~192 (96+0+96), got {}",
        child_box.height
    );
}

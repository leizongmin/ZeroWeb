use super::*;
use zero_css_parser::values::{ClearValue, DisplayValue, FloatValue, LengthValue, PositionValue, VisibilityValue};

/// 测试 inline-only 容器收缩后，后续普通流兄弟应同步上移。
#[test]
fn test_inline_only_container_shrink_reflows_following_sibling() {
    let (mut doc, body) = make_doc_with_body();
    let first = doc.create_element("div");
    doc.append_child(body, first).unwrap();
    let img1 = doc.create_element("img");
    let img2 = doc.create_element("img");
    {
        let elem = doc.get_mut(img1).unwrap();
        if let zero_dom::NodeKind::Element(e) = &mut elem.kind {
            e.set_attribute("width", "96");
            e.set_attribute("height", "96");
        }
    }
    {
        let elem = doc.get_mut(img2).unwrap();
        if let zero_dom::NodeKind::Element(e) = &mut elem.kind {
            e.set_attribute("width", "96");
            e.set_attribute("height", "144");
        }
    }
    doc.append_child(first, img1).unwrap();
    doc.append_child(first, img2).unwrap();

    let second = doc.create_element("div");
    doc.append_child(body, second).unwrap();
    let img3 = doc.create_element("img");
    {
        let elem = doc.get_mut(img3).unwrap();
        if let zero_dom::NodeKind::Element(e) = &mut elem.kind {
            e.set_attribute("width", "96");
            e.set_attribute("height", "96");
        }
    }
    doc.append_child(second, img3).unwrap();

    let mut styles = HashMap::new();
    let mut first_style = ComputedStyle::default();
    first_style.display = DisplayValue::Block;
    styles.insert(first, first_style);

    let mut second_style = ComputedStyle::default();
    second_style.display = DisplayValue::Block;
    styles.insert(second, second_style);

    let mut img_style = ComputedStyle::default();
    img_style.display = DisplayValue::Inline;
    styles.insert(img1, img_style.clone());
    styles.insert(img2, img_style.clone());
    styles.insert(img3, img_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let first_box = find_child_by_node_id(&result.root, first).expect("first found");
    let second_box = find_child_by_node_id(&result.root, second).expect("second found");

    assert!(
        (first_box.height - 144.0).abs() < 1.0,
        "first inline-only container should shrink to tallest image height, got {}",
        first_box.height
    );
    assert!(
        (second_box.y - (first_box.y + first_box.height)).abs() < 1.0,
        "following sibling should be reflowed after shrink: second.y={}, first.bottom={}",
        second_box.y,
        first_box.y + first_box.height
    );
}

/// 测试 clear-float-003：空普通块的自折叠 margin 不应错误抬高后续 clear:right 浮动。
#[test]
fn test_clear_float_003_negative_margin_clear_float_can_overlap_prior_float() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let first_float = doc.create_element("div");
    doc.append_child(container, first_float).unwrap();
    let spacer = doc.create_element("div");
    doc.append_child(container, spacer).unwrap();
    let cleared_float = doc.create_element("div");
    doc.append_child(container, cleared_float).unwrap();

    let mut styles = HashMap::new();

    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Block;
    container_style.width = LengthValue::Px(192.0);
    container_style.height = LengthValue::Px(192.0);
    styles.insert(container, container_style);

    let mut first_float_style = ComputedStyle::default();
    first_float_style.display = DisplayValue::Block;
    first_float_style.float = FloatValue::Right;
    first_float_style.width = LengthValue::Px(96.0);
    first_float_style.height = LengthValue::Px(96.0);
    styles.insert(first_float, first_float_style);

    let mut spacer_style = ComputedStyle::default();
    spacer_style.display = DisplayValue::Block;
    spacer_style.height = LengthValue::Px(0.0);
    spacer_style.margin_top = LengthValue::Px(96.0);
    spacer_style.margin_bottom = LengthValue::Px(96.0);
    styles.insert(spacer, spacer_style);

    let mut cleared_float_style = ComputedStyle::default();
    cleared_float_style.display = DisplayValue::Block;
    cleared_float_style.float = FloatValue::Right;
    cleared_float_style.clear = ClearValue::Right;
    cleared_float_style.width = LengthValue::Px(96.0);
    cleared_float_style.height = LengthValue::Px(96.0);
    cleared_float_style.margin_top = LengthValue::Px(-96.0);
    styles.insert(cleared_float, cleared_float_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let first_box = find_child_by_node_id(&result.root, first_float).expect("first float found");
    let cleared_box = find_child_by_node_id(&result.root, cleared_float).expect("cleared float found");

    assert!(
        (first_box.y - 0.0).abs() < 0.5,
        "first float should stay at top, got y={}",
        first_box.y
    );
    assert!(
        (cleared_box.y - 0.0).abs() < 0.5,
        "cleared float should keep the same top as the prior float, got y={}",
        cleared_box.y
    );
}

/// 测试 BFC 浮动排斥：overflow:hidden 的块级元素不得与左浮动重叠。
/// CSS 2.1 §9.5: BFC 元素的 border box 不得与同一格式化上下文中的浮动元素重叠。
#[test]
fn test_bfc_float_avoidance_left() {
    let (mut doc, body) = make_doc_with_body();
    let float_elem = doc.create_element("div");
    doc.append_child(body, float_elem).unwrap();
    let bfc_elem = doc.create_element("div");
    doc.append_child(body, bfc_elem).unwrap();

    let mut styles = HashMap::new();

    // 左浮动：50x50
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.width = LengthValue::Px(50.0);
    fl.height = LengthValue::Px(50.0);
    fl.float = FloatValue::Left;
    styles.insert(float_elem, fl);

    // overflow:hidden（建立 BFC）：100x100
    let mut bfc = ComputedStyle::default();
    bfc.display = DisplayValue::Block;
    bfc.width = LengthValue::Px(100.0);
    bfc.height = LengthValue::Px(100.0);
    bfc.overflow_x = zero_css_parser::values::OverflowValue::Hidden;
    bfc.overflow_y = zero_css_parser::values::OverflowValue::Hidden;
    styles.insert(bfc_elem, bfc);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fl_box = find_child_by_node_id(&result.root, float_elem).expect("float found");
    let bfc_box = find_child_by_node_id(&result.root, bfc_elem).expect("bfc found");

    // BFC 元素的左边缘应在浮动元素的右边缘之后或与之间距
    assert!(
        bfc_box.x >= fl_box.x + fl_box.width - 0.5,
        "BFC 元素不得与左浮动重叠: bfc.x={}, float_right={}",
        bfc_box.x,
        fl_box.x + fl_box.width
    );
}

/// 测试 BFC 浮动排斥：overflow:hidden 的块级元素不得与右浮动重叠。
#[test]
fn test_bfc_float_avoidance_right() {
    let (mut doc, body) = make_doc_with_body();
    let float_elem = doc.create_element("div");
    doc.append_child(body, float_elem).unwrap();
    let bfc_elem = doc.create_element("div");
    doc.append_child(body, bfc_elem).unwrap();

    let mut styles = HashMap::new();

    // 右浮动：50x50
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.width = LengthValue::Px(50.0);
    fl.height = LengthValue::Px(50.0);
    fl.float = FloatValue::Right;
    styles.insert(float_elem, fl);

    // overflow:hidden（建立 BFC）：200x100
    let mut bfc = ComputedStyle::default();
    bfc.display = DisplayValue::Block;
    bfc.width = LengthValue::Px(200.0);
    bfc.height = LengthValue::Px(100.0);
    bfc.overflow_x = zero_css_parser::values::OverflowValue::Hidden;
    bfc.overflow_y = zero_css_parser::values::OverflowValue::Hidden;
    styles.insert(bfc_elem, bfc);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fl_box = find_child_by_node_id(&result.root, float_elem).expect("float found");
    let bfc_box = find_child_by_node_id(&result.root, bfc_elem).expect("bfc found");

    // BFC 元素的右边缘不应超过右浮动的左边缘
    assert!(
        bfc_box.x + bfc_box.width <= fl_box.x + 0.5,
        "BFC 元素不得与右浮动重叠: bfc_right={}, float_left={}",
        bfc_box.x + bfc_box.width,
        fl_box.x
    );
}

/// 测试孤立 table-row-group 作为块级兄弟时仍应布局其匿名行内的 table-cell。
///
/// 该场景对应 clear-applies-to-001：`display: table-row-group` 不应触发 clear，
/// 但其内部匿名行仍必须参与 table 布局，否则单元格会停留在 taffy 的错误位置。
#[test]
fn test_orphan_table_row_group_positions_anonymous_cells() {
    let (mut doc, body) = make_doc_with_body();
    let float_elem = doc.create_element("div");
    doc.append_child(body, float_elem).unwrap();

    let row_group = doc.create_element("div");
    doc.append_child(body, row_group).unwrap();

    let cell_a = doc.create_element("div");
    doc.append_child(row_group, cell_a).unwrap();
    let cell_b = doc.create_element("div");
    doc.append_child(row_group, cell_b).unwrap();

    let mut styles = HashMap::new();

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(320.0);
    fl.height = LengthValue::Px(96.0);
    styles.insert(float_elem, fl);

    let mut rg = ComputedStyle::default();
    rg.display = DisplayValue::TableRowGroup;
    rg.clear = ClearValue::Both;
    rg.background_color = zero_css_parser::values::ColorValue::Named("blue".to_string());
    styles.insert(row_group, rg);

    let mut cell = ComputedStyle::default();
    cell.display = DisplayValue::TableCell;
    cell.width = LengthValue::Px(48.0);
    cell.height = LengthValue::Px(48.0);
    cell.background_color = zero_css_parser::values::ColorValue::Named("blue".to_string());
    styles.insert(cell_a, cell.clone());
    styles.insert(cell_b, cell);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let float_box = find_child_by_node_id(&result.root, float_elem).expect("float found");
    let row_group_box = find_child_by_node_id(&result.root, row_group).expect("row group found");
    let cell_a_box = find_child_by_node_id(&result.root, cell_a).expect("cell_a found");
    let cell_b_box = find_child_by_node_id(&result.root, cell_b).expect("cell_b found");

    assert!(
        row_group_box.x < float_box.x + float_box.width - 0.5,
        "table-row-group clear should not apply: rg.x={}, float_right={}",
        row_group_box.x,
        float_box.x + float_box.width
    );
    assert!(
        (cell_a_box.x - row_group_box.x).abs() < 1.0,
        "first anonymous cell should start at row group left edge: cell_a.x={}, rg.x={}",
        cell_a_box.x,
        row_group_box.x
    );
    assert!(
        cell_b_box.x >= cell_a_box.x + cell_a_box.width - 0.5,
        "second anonymous cell should be positioned after the first: cell_b.x={}, cell_a.right={}",
        cell_b_box.x,
        cell_a_box.x + cell_a_box.width
    );
    assert!(
        (cell_a_box.y - row_group_box.y).abs() < 1.0 && (cell_b_box.y - row_group_box.y).abs() < 1.0,
        "anonymous row cells should align to the row group top: cell_a.y={}, cell_b.y={}, rg.y={}",
        cell_a_box.y,
        cell_b_box.y,
        row_group_box.y
    );
}

/// 测试嵌套 block 上的 clear:both 仍需清除祖先容器中更早的浮动。
///
/// 对应 clear-applies-to-009：float 是 body 的直接子元素，clear:block 在后续 div 内部。
#[test]
fn test_nested_block_clear_sees_ancestor_floats() {
    let (mut doc, body) = make_doc_with_body();
    let float_elem = doc.create_element("div");
    doc.append_child(body, float_elem).unwrap();

    let wrapper = doc.create_element("div");
    doc.append_child(body, wrapper).unwrap();

    let clear_block = doc.create_element("div");
    doc.append_child(wrapper, clear_block).unwrap();

    let mut styles = HashMap::new();

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(320.0);
    fl.height = LengthValue::Px(96.0);
    styles.insert(float_elem, fl);

    let mut wr = ComputedStyle::default();
    wr.display = DisplayValue::Block;
    styles.insert(wrapper, wr);

    let mut cb = ComputedStyle::default();
    cb.display = DisplayValue::Block;
    cb.clear = ClearValue::Both;
    cb.width = LengthValue::Px(96.0);
    cb.height = LengthValue::Px(96.0);
    styles.insert(clear_block, cb);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let float_box = find_child_by_node_id(&result.root, float_elem).expect("float found");
    let clear_box = find_child_by_node_id(&result.root, clear_block).expect("clear block found");
    let (_, float_abs_y) = find_absolute_position_by_node_id(&result.root, float_elem).expect("float abs");
    let (_, clear_abs_y) = find_absolute_position_by_node_id(&result.root, clear_block).expect("clear abs");

    assert!(
        clear_abs_y >= float_abs_y + float_box.height - 0.5,
        "nested clear block should be placed below earlier float: clear_abs_y={}, float_bottom_abs={}",
        clear_abs_y,
        float_abs_y + float_box.height
    );
    assert!(clear_box.height > 0.0, "clear block should still have its own box");
}

#[test]
fn test_border_collapse_table_wins() {
    let html = r#"<html><body style="margin:0"><table style="border: 5px solid green; border-collapse: collapse"><tr><td style="border: 4.95px solid red; width: 50px; height: 50px"></td></tr></table></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // Find the table cell by looking for small boxes with borders.
    // 必须跳过 table 本身（也可能匹配 width < 100 条件），
    // 只返回最内层（叶子级别）的 cell 盒。
    fn find_cell(box_node: &crate::types::LayoutBox) -> Option<&crate::types::LayoutBox> {
        for child in &box_node.children {
            // 先递归搜索子元素，优先返回更深的 cell
            if let Some(c) = find_cell(child) {
                return Some(c);
            }
            // 如果子元素中没有 cell，检查当前 child 是否匹配
            if child.border_top > 0.0 && child.width < 100.0 && child.width > 10.0 {
                return Some(child);
            }
        }
        None
    }

    if let Some(cell) = find_cell(&result.root) {
        // Table border (5px) should win over cell border (4.95px)
        // After resolve_collapsed_borders, cell border_top should be ~5.0
        assert!(
            cell.border_top >= 4.9,
            "cell border_top should be ~5.0 (table wins), got {}",
            cell.border_top
        );
        // Color override should be set for top edge (green from table)
        let top_color_override = cell.collapsed_border_color_overrides[0];
        assert!(
            top_color_override.is_some(),
            "top color override should be set (table's green), got None"
        );
        // Green = Rgba(0, 128, 0, 255) = 0x008000FF
        if let Some(c) = top_color_override {
            assert_eq!(
                c, 0x008000FF,
                "top color override should be green (0x008000FF), got {:#010X}",
                c
            );
        }
    }
}

/// 测试显式宽度的 table 中 auto 宽度单元格的列宽分布。
///
/// 对应 multicol-fill-001：`<table style="width:400px">` 含两个 auto 宽度单元格，
/// 每列应约为表宽一半（200px），列宽总和不超过表宽。
/// 旧实现把 auto 单元格宽度取 `intrinsic.max(cell_box.width)`，而 taffy 给单元格的
/// block-level 宽度等于表全宽，导致每列都被撑到全宽、列总和溢出表宽。
#[test]
fn test_explicit_width_table_auto_cells_distribute() {
    let html = r#"<html><body style="margin:0"><table style="width: 400px"><tr><td>A</td><td>B</td></tr></table></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 找 table 盒（宽度应 == 400）
    fn find_table(box_node: &crate::types::LayoutBox) -> Option<&crate::types::LayoutBox> {
        if (box_node.width - 400.0).abs() < 1.0 && box_node.content_width > 300.0 {
            return Some(box_node);
        }
        for child in &box_node.children {
            if let Some(t) = find_table(child) {
                return Some(t);
            }
        }
        None
    }
    let table = find_table(&result.root).expect("table not found");
    // 表宽应被显式 width:400px 约束
    assert!(
        (table.width - 400.0).abs() < 2.0,
        "table width should be ~400 (explicit), got {}",
        table.width
    );

    // 收集 table 子树中所有叶子盒的宽度（td：文本存于 inline_layout，td 本身是叶子）
    let mut cell_widths: Vec<f32> = Vec::new();
    fn walk(box_node: &crate::types::LayoutBox, out: &mut Vec<f32>) {
        if box_node.children.is_empty() && box_node.width > 5.0 && box_node.width < 400.0 {
            out.push(box_node.width);
        }
        for child in &box_node.children {
            walk(child, out);
        }
    }
    walk(table, &mut cell_widths);
    // 两个单元格宽度总和不应超过表宽（~400），而非旧 bug 那样每列 = 表全宽（总和 ≈ 800）。
    let total: f32 = cell_widths.iter().sum();
    assert!(
        total <= 410.0,
        "cell widths total {} should not exceed table width ~400 (old bug gave ~800)",
        total
    );
}

/// 测试百分比 max-height 在包含块高度明确时被收紧。
///
/// 对应 fieldset-as-item-overflow：父级有明确 height（100px），子级 `max-height: 100%`
/// 且内容更高（200px）。CSS §10.7 要求按包含块高度解析百分比 max-height，子级应被
/// 收紧到 ~100px，而非沿用内容高度 200px。taffy 0.7 不会收紧 height:auto 的块盒，
/// 由 `clamp_percentage_max_height` 后处理补齐。
#[test]
fn test_percentage_max_height_clamps_to_definite_cb() {
    let html = r#"<html><body style="margin:0">
<div style="height:100px;">
  <div id="mid" style="max-height:100%;">
    <div style="height:200px;"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find_mid(box_node: &crate::types::LayoutBox) -> Option<&crate::types::LayoutBox> {
        if (box_node.height - 100.0).abs() < 1.0 && box_node.children.len() == 1 {
            return Some(box_node);
        }
        for child in &box_node.children {
            if let Some(m) = find_mid(child) {
                return Some(m);
            }
        }
        None
    }
    let mid = find_mid(&result.root).expect("clamped mid div not found");
    assert!(
        mid.height <= 101.0,
        "percentage max-height:100% of 100px CB should clamp to ~100px, got {}",
        mid.height
    );
    assert!(
        mid.height < 150.0,
        "mid div must not keep content-driven 200px height (got {})",
        mid.height
    );
}

/// 测试百分比 max-height 在包含块高度不明确（auto）时不收紧。
///
/// CSS §10.5：当包含块高度由内容决定时，百分比 height/max-height 视为 auto（不解析）。
#[test]
fn test_percentage_max_height_no_clamp_when_cb_indefinite() {
    let html = r#"<html><body style="margin:0">
<div>
  <div id="mid" style="max-height:50%;">
    <div style="height:200px;"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find_mid(box_node: &crate::types::LayoutBox) -> Option<&crate::types::LayoutBox> {
        // 内容驱动的 mid 盒：有 1 个子且自身高度 > 150（容纳 200px 内容）
        if box_node.children.len() == 1 && box_node.height > 150.0 && box_node.height < 260.0 {
            return Some(box_node);
        }
        for child in &box_node.children {
            if let Some(m) = find_mid(child) {
                return Some(m);
            }
        }
        None
    }
    let mid = find_mid(&result.root).expect("content-driven mid div not found");
    // 父级高度 auto（内容决定）→ max-height:50% 不应解析 → mid 保持 ~200px（内容高）
    assert!(
        mid.height >= 195.0,
        "indefinite CB: percentage max-height must not clamp (got {}), expected ~200px content height",
        mid.height
    );
}

/// 测试 table 的 height 属性作为内容高度下限（CSS 2.1 §17.5.3）。
///
/// ZeroWeb 的 table 后处理（apply_table_size_constraints）此前完全忽略 style.height，
/// 仅用 intrinsic 行高填表格高度。CSS 规定 table 的 'height' 是内容高度的「下限」
/// （min 语义）：表格至少这么高，内容更高则增长。本测试用 Px 高度覆盖核心 min 语义。
/// 对应 chromium Oracle 缺口 table-grid-item-dynamic-004（height:% + padding-top）。
#[test]
fn test_table_height_as_minimum_px() {
    let html = r#"<html><body style="margin:0">
<div style="height:300px;">
  <table style="height:200px;">
    <tr><td>x</td></tr>
  </table>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // height:200px（content-box，无 padding/border）→ 表格至少 ~200px。
    // 修复前表格仅 intrinsic 行高（~16px）；300px 的父 div 不在 (180,260) 区间。
    fn find_table(box_node: &crate::types::LayoutBox) -> Option<&crate::types::LayoutBox> {
        if box_node.height > 180.0 && box_node.height < 260.0 {
            return Some(box_node);
        }
        for child in &box_node.children {
            if let Some(t) = find_table(child) {
                return Some(t);
            }
        }
        None
    }
    let table =
        find_table(&result.root).expect("table with height:200px should grow to ~200px (CSS §17.5.3 min semantics)");
    assert!(
        table.height >= 195.0,
        "table height:200px should make table at least ~200px tall, got {}",
        table.height
    );
}

/// 测试 clamp_percentage_max_height 对 table 百分比 height 的下限解析（直接调用）。
///
/// 手工构造 div(明确 content=100px) > table(height:100%, display:table, content=16) 树，
/// 直接调用后处理函数，验证百分比 height 相对明确 CB 解析为内容高度下限（CSS §17.5.3 + §10.5）。
/// 注：engine.compute 路径中 table 的匿名包装盒会打断直接父子 CB 传递，故直接测函数；
/// 百分比路径的端到端正确性由 reftest table-grid-item-dynamic-004 覆盖（chromium 差距 11%→2.98%）。
#[test]
fn test_table_percentage_height_resolves_as_minimum() {
    use zero_css_parser::values::BoxSizingValue;
    let (mut doc, _body) = make_doc_with_body();
    let div = doc.create_element("div");
    let table = doc.create_element("table");

    let mut styles = HashMap::new();
    let mut div_s = ComputedStyle::default();
    div_s.display = DisplayValue::Block;
    div_s.height = LengthValue::Px(100.0);
    styles.insert(div, div_s);
    let mut t_s = ComputedStyle::default();
    t_s.display = DisplayValue::Table;
    t_s.height = LengthValue::Percentage(100.0);
    t_s.box_sizing = BoxSizingValue::ContentBox;
    styles.insert(table, t_s);

    // table 初始 content_height = intrinsic(16)；div content=100（明确 CB）
    let table_box = LayoutBox {
        node_id: Some(table),
        content_height: 16.0,
        height: 16.0,
        ..Default::default()
    };
    let mut div_box = LayoutBox {
        node_id: Some(div),
        content_height: 100.0,
        height: 100.0,
        ..Default::default()
    };
    div_box.children.push(table_box);

    super::super::clamp_percentage_max_height(&mut div_box, None, &styles);

    let t = &div_box.children[0];
    assert!(
        t.content_height >= 99.0,
        "table height:100% of 100px div should grow content to ~100 (CSS §17.5.3 + §10.5), got {}",
        t.content_height
    );
}

/// R2057：abspos + max-height:max-content/fit-content 关键字 cap。
///
/// convert_max_length_to_dimension 把这些关键字转 taffy auto（无 max 约束），taffy 0.12
/// max_size 不支持 content keyword → abspos top+bottom 拉伸到 CB 高度，max-height 不 cap。
/// clamp_percentage_max_height 的 R2057 分支在 taffy 后用 content_height（max child bottom）
/// cap。构造 abspos(height=200 拉伸, max-height:max-content) > child(height=100)，验证 cap 到 100。
#[test]
fn test_abspos_max_height_keyword_caps_stretched_height() {
    let (mut doc, _body) = make_doc_with_body();
    let abs = doc.create_element("div");
    let child = doc.create_element("div");

    let mut styles = HashMap::new();
    let mut a_s = ComputedStyle::default();
    a_s.display = DisplayValue::Block;
    a_s.position = PositionValue::Absolute;
    a_s.max_height = LengthValue::MaxContent;
    styles.insert(abs, a_s);

    // child height=100（definite content）；abspos 拉伸到 200（top+bottom stretch）
    let child_box = LayoutBox {
        node_id: Some(child),
        y: 0.0,
        height: 100.0,
        content_height: 100.0,
        ..Default::default()
    };
    let mut abs_box = LayoutBox {
        node_id: Some(abs),
        is_absolute: true,
        height: 200.0,
        content_height: 200.0,
        ..Default::default()
    };
    abs_box.children.push(child_box);

    super::super::clamp_percentage_max_height(&mut abs_box, None, &styles);

    assert!(
        abs_box.height <= 101.0,
        "abspos max-height:max-content 应 cap 拉伸高度到 content(100)，got {}",
        abs_box.height
    );
    assert!(
        abs_box.content_height <= 101.0,
        "abspos content_height 应同步 cap，got {}",
        abs_box.content_height
    );
}

/// 测试 table column 的 visibility:collapse。
///
/// 对应 visibility-collapse-colspan-003：中间列被 `visibility:collapse` 折叠，
/// 其宽度应为 0，且最后一行的 colspan 单元格应只占非折叠列宽度并裁剪溢出内容。
/// 非折叠列的显式 width 不应被 colspan 单元格的长内容撑开。
#[test]
fn test_table_column_visibility_collapse() {
    use zero_css_parser::values::LengthValue;
    let (mut doc, body) = make_doc_with_body();

    // <table>
    let table = doc.create_element("table");
    doc.append_child(body, table).unwrap();

    // <col> x3, 中间一个 visibility:collapse
    let col0 = doc.create_element("col");
    doc.append_child(table, col0).unwrap();
    let col1 = doc.create_element("col");
    doc.append_child(table, col1).unwrap();
    let col2 = doc.create_element("col");
    doc.append_child(table, col2).unwrap();

    // <tr> with 3 <td>, firstCol=65, thirdCol=160
    let row = doc.create_element("tr");
    doc.append_child(table, row).unwrap();
    let td0 = doc.create_element("td");
    doc.append_child(row, td0).unwrap();
    let td1 = doc.create_element("td");
    doc.append_child(row, td1).unwrap();
    let td2 = doc.create_element("td");
    doc.append_child(row, td2).unwrap();

    // <tr> with colspan=3 cell
    let row2 = doc.create_element("tr");
    doc.append_child(table, row2).unwrap();
    let td_span = doc.create_element("td");
    doc.set_attribute(td_span, "colspan", "3");
    doc.append_child(row2, td_span).unwrap();

    let mut styles = HashMap::new();

    // col1 visibility:collapse
    let mut c1 = ComputedStyle::default();
    c1.display = DisplayValue::TableColumn;
    c1.visibility = VisibilityValue::Collapse;
    styles.insert(col1, c1);

    let mut c0 = ComputedStyle::default();
    c0.display = DisplayValue::TableColumn;
    styles.insert(col0, c0);

    let mut c2 = ComputedStyle::default();
    c2.display = DisplayValue::TableColumn;
    styles.insert(col2, c2);

    // table display
    let mut tbl = ComputedStyle::default();
    tbl.display = DisplayValue::Table;
    styles.insert(table, tbl);

    let mut tr = ComputedStyle::default();
    tr.display = DisplayValue::TableRow;
    styles.insert(row, tr.clone());
    styles.insert(row2, tr);

    let mut cell = ComputedStyle::default();
    cell.display = DisplayValue::TableCell;
    styles.insert(td1, cell.clone());
    styles.insert(td_span, cell.clone());

    // firstCol width:65px
    let mut first = ComputedStyle::default();
    first.display = DisplayValue::TableCell;
    first.width = LengthValue::Px(65.0);
    styles.insert(td0, first.clone());

    // thirdCol width:160px
    let mut third = ComputedStyle::default();
    third.display = DisplayValue::TableCell;
    third.width = LengthValue::Px(160.0);
    styles.insert(td2, third);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let td0_box = find_child_by_node_id(&result.root, td0).expect("td0 found");
    let td2_box = find_child_by_node_id(&result.root, td2).expect("td2 found");
    let td1_box = find_child_by_node_id(&result.root, td1).expect("td1 found");
    let td_span_box = find_child_by_node_id(&result.root, td_span).expect("colspan cell found");

    // 折叠列单元格宽度应为 0
    assert!(
        td1_box.width.abs() < 0.5,
        "collapsed column cell width should be 0, got {}",
        td1_box.width
    );

    // 非折叠列应保持显式宽度，不被 colspan 长内容撑开
    assert!(
        (td0_box.width - 65.0).abs() < 1.5,
        "firstCol width should be ~65 (explicit, not inflated by colspan), got {}",
        td0_box.width
    );
    assert!(
        (td2_box.width - 160.0).abs() < 1.5,
        "thirdCol width should be ~160 (explicit, not inflated by colspan), got {}",
        td2_box.width
    );

    // colspan 单元格宽度 = 65 + 0 + 160 = 225（仅非折叠列）
    assert!(
        (td_span_box.width - 225.0).abs() < 3.0,
        "colspan-3 cell spanning collapsed col should be ~225 (sum of non-collapsed cols), got {}",
        td_span_box.width
    );

    // colspan 单元格应设置 overflow_x:Hidden 以裁剪溢出内容
    assert_eq!(
        td_span_box.overflow_x,
        crate::types::OverflowClip::Hidden,
        "colspan cell spanning collapsed column must clip overflow"
    );
}

/// 测试：无 border/padding 的容器中，第一个流内子元素的 margin-top 与父容器折叠后，
/// 后续 float 的定位不应把该 margin-top 双重计入（CSS §8.3.1 margin 与父折叠）。
/// 复现 inline-formatting-context-002/003：`<p>`(mt=16) 后跟 float，float 应位于
/// p.border_bottom + p.margin_bottom，而非额外加上 p.margin_top。
#[test]
fn test_float_after_first_child_margin_collapses_with_parent() {
    let (mut doc, body) = make_doc_with_body();
    let p = doc.create_element("p");
    doc.append_child(body, p).unwrap();
    let float_div = doc.create_element("div");
    doc.append_child(body, float_div).unwrap();

    let mut styles = HashMap::new();
    // body：无 border/padding（默认），使首个子元素 margin-top 与之折叠
    let mut body_style = ComputedStyle::default();
    body_style.display = DisplayValue::Block;
    body_style.margin_top = LengthValue::Px(16.0);
    body_style.margin_bottom = LengthValue::Px(8.0);
    styles.insert(body, body_style);

    // 第一个流内子元素 <p>：margin-top=16（与 body 折叠），height=19，margin-bottom=16
    let mut ps = ComputedStyle::default();
    ps.display = DisplayValue::Block;
    ps.margin_top = LengthValue::Px(16.0);
    ps.height = LengthValue::Px(19.0);
    ps.margin_bottom = LengthValue::Px(16.0);
    styles.insert(p, ps);

    // float：高度 19.2
    let mut fs = ComputedStyle::default();
    fs.display = DisplayValue::Block;
    fs.float = FloatValue::Left;
    fs.height = LengthValue::Px(19.2);
    styles.insert(float_div, fs);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let p_box = find_child_by_node_id(&result.root, p).expect("p found");
    let f_box = find_child_by_node_id(&result.root, float_div).expect("float found");

    // float 与 p 底边的间距应等于 p.margin_bottom（16），而非 16+16（双重计入 p.margin_top）。
    let gap = f_box.y - (p_box.y + p_box.height);
    assert!(
        (gap - 16.0).abs() < 1.0,
        "float 应位于 p.border_bottom + p.margin_bottom（间距≈16），实际 gap={}（p.y={} h={} float.y={}）",
        gap,
        p_box.y,
        p_box.height,
        f_box.y
    );
}

/// 测试容器 margin-top 不与首个 float 子元素的 margin 错误折叠（CSS §8.3.1）。
///
/// taffy 把 float 当作普通 block 排列，当容器的首个流内子元素是 float 且容器无
/// border-top/padding-top（margin 可与首个子元素折叠）时，容器的 margin-top 会被
/// 错误折叠到该 float 的 margin（取 max），使容器（及其全部内容）整体偏低。
/// ZeroWeb 在 float 定位后处理中检测并修正（CSS §8.3.1：float 的 margin 不折叠）：
/// 仅当容器 margin_top == 首个 float 子元素 margin_top 且该 float margin 自身未被
/// taffy 膨胀时，把多折叠的量从容器 y 扣除并恢复 margin_top。
#[test]
fn test_container_margin_not_collapsed_with_first_float_child() {
    let (mut doc, body) = make_doc_with_body();
    // body 的首个流内子元素是 margin-top=16 的 float（大于 body 自身 margin 8）
    let float_div = doc.create_element("div");
    doc.append_child(body, float_div).unwrap();

    let mut styles = HashMap::new();
    // body：margin-top=8，无 border/padding（使 margin 可与首个子元素折叠）
    let mut body_style = ComputedStyle::default();
    body_style.display = DisplayValue::Block;
    body_style.margin_top = LengthValue::Px(8.0);
    styles.insert(body, body_style);

    // 首个子元素：float:left，margin-top=16
    let mut fs = ComputedStyle::default();
    fs.display = DisplayValue::Block;
    fs.float = FloatValue::Left;
    fs.margin_top = LengthValue::Px(16.0);
    fs.height = LengthValue::Px(20.0);
    styles.insert(float_div, fs);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let body_box = find_child_by_node_id(&result.root, body).expect("body found");

    // 修正后 body.y 应反映 body 自身 margin-top（8），而非与 float margin 折叠后的 16。
    //（未修正时 taffy 会把 body.margin_top 折叠到 float 的 16，使 body.y=16，整体偏低 8px。）
    assert!(
        (body_box.y - 8.0).abs() < 1.0,
        "body.y 应为自身 margin-top（8），CSS §8.3.1 float margin 不与容器折叠；实际 body.y={}（margin_top={} declared={})",
        body_box.y,
        body_box.margin_top,
        body_box.declared_margin_top
    );
    // margin_top 也应恢复为声明值 8（未被折叠放大到 16）
    assert!(
        (body_box.margin_top - 8.0).abs() < 0.5,
        "body.margin_top 应恢复为声明值 8，实际={}",
        body_box.margin_top
    );
}

/// 测试 width:auto 的浮动元素 shrink-to-fit 到块级子元素宽度（CSS §10.3.5）。
///
/// taffy 把 float 当作普通 block（width:auto 填满可用宽度），但 CSS §10.3.5 规定
/// 浮动非替换元素 width:auto 应收缩到内容（max-content）宽度。ZeroWeb 在 float 后
/// 处理中对 width:auto 且有块级子元素的 float 收缩到子元素最大 border-box 宽度
///（仅当窄于当前宽度）。纯文本 float 的 shrink-to-fit 需 IFC 测量，留作后续。
#[test]
fn test_float_width_auto_shrink_to_fit_block_child() {
    let (mut doc, body) = make_doc_with_body();
    let float_div = doc.create_element("div");
    doc.append_child(body, float_div).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(float_div, inner).unwrap();

    let mut styles = HashMap::new();
    let mut body_style = ComputedStyle::default();
    body_style.display = DisplayValue::Block;
    styles.insert(body, body_style);

    // float：width:auto（未显式设置 width，默认 Auto）
    let mut fs = ComputedStyle::default();
    fs.display = DisplayValue::Block;
    fs.float = FloatValue::Left;
    styles.insert(float_div, fs);

    // 块级子元素：width:96px（1in）
    let mut inner_style = ComputedStyle::default();
    inner_style.display = DisplayValue::Block;
    inner_style.width = LengthValue::Px(96.0);
    inner_style.height = LengthValue::Px(96.0);
    styles.insert(inner, inner_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let f_box = find_child_by_node_id(&result.root, float_div).expect("float found");

    // float 应 shrink-to-fit 到子元素宽度 96，而非填满可用宽度（~784）。
    assert!(
        (f_box.width - 96.0).abs() < 1.0,
        "width:auto 的 float 应 shrink-to-fit 到块级子元素宽度（96），实际 width={}",
        f_box.width
    );
}

/// 测试根元素 <html> 的 position:relative inset 被正确应用。
///
/// taffy 0.7 不会对**根节点**应用 position:relative 的 top/left inset（根总在 0,0），
/// 但会对非根 block-level 元素应用。ZeroWeb 在 extract_layout 后手动补上根的
/// relative 偏移（CSS 2.1 §9.4.3），使根及其 abspos 后代（CB=根 padding box）
/// 整体偏移到正确视觉位置。
#[test]
fn test_root_relative_position_applies_inset() {
    let (doc, body) = make_doc_with_body();
    let html = doc.parent_node(body).expect("html 是 body 的父节点");

    let mut styles = HashMap::new();
    let mut html_style = ComputedStyle::default();
    html_style.display = DisplayValue::Block;
    html_style.position = PositionValue::Relative;
    html_style.top = LengthValue::Px(100.0);
    html_style.left = LengthValue::Px(100.0);
    html_style.height = LengthValue::Px(100.0);
    styles.insert(html, html_style);

    // body 默认样式
    let mut body_style = ComputedStyle::default();
    body_style.display = DisplayValue::Block;
    styles.insert(body, body_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 根 html 应被 relative inset 偏移到 (100,100)
    assert!(
        (result.root.x - 100.0).abs() < 0.5 && (result.root.y - 100.0).abs() < 0.5,
        "根 relative 元素应应用 top/left inset：期望 (100,100)，实际 root.x={} root.y={}",
        result.root.x,
        result.root.y
    );

    // 对照：根为 static 时不应偏移
    let (doc2, body2) = make_doc_with_body();
    let html2 = doc2.parent_node(body2).expect("html");
    let mut styles2 = HashMap::new();
    let mut html2_style = ComputedStyle::default();
    html2_style.display = DisplayValue::Block;
    html2_style.height = LengthValue::Px(100.0);
    styles2.insert(html2, html2_style);
    let mut body2_style = ComputedStyle::default();
    body2_style.display = DisplayValue::Block;
    styles2.insert(body2, body2_style);
    let result2 = LayoutEngine::new(800.0, 600.0).compute(&doc2, &styles2);
    assert!(
        (result2.root.x - 0.0).abs() < 0.5 && (result2.root.y - 0.0).abs() < 0.5,
        "static 根不应偏移：实际 root.x={} root.y={}",
        result2.root.x,
        result2.root.y
    );
}

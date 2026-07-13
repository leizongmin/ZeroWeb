//! R1382 回归测试：浮动块化 table-internal（§9.7）内匿名 table 包装与尺寸。
//!
//! 背景（float-applies-to-001~004）：`#test` 设 `display:table-row-group; float:right`，
//! 经 CSS §9.7 块化为 `Block + float:right`。其 `.row`（table-row）子按 §17.2.1.1 应被
//! 合并到一个匿名 table 包装盒，grid 布局后 #test 浮动 shrink-to-fit 到 96×96 浮右。
//!
//! 修复三处：
//! 1. `adjust_table_layout_inner`：`inside_table` 仅在父元素本身构成 table 结构时向子传递
//!    （块化的 Block 打断表结构），否则 .row 被误判「在表内」而不触发匿名包装。
//! 2. `apply_table_size_constraints`：匿名 table（无 node_id）按 intrinsic 落尺寸（不再
//!    early-return），否则表盒 width/height 留 inherited 值。
//! 3. `merge_orphan_table_run`：浮动父块的 shrink-to-fit 宽/高回填匿名 table 真实尺寸
//!    并重定位（右浮保持右缘、左浮保持左缘）。

use super::*;
use zero_css_parser::values::{ColorValue, DisplayValue, FloatValue, LengthValue};
use zero_style_system::ComputedStyle;

/// 构造 float-applies-to-001 结构：#table(table,width:100%) 内含 #test(块化 Block+float)，
/// 其下 2 个 .row(table-row)，每行 2 个 .cell(table-cell 48×48)。
/// 返回 (doc, styles, 各节点 id)。
fn build_float_applies_to_001(
    float: FloatValue,
) -> (
    zero_dom::Document,
    HashMap<zero_dom::NodeId, ComputedStyle>,
    zero_dom::NodeId,
    zero_dom::NodeId,
    zero_dom::NodeId,
    zero_dom::NodeId,
) {
    let (mut doc, body) = make_doc_with_body();
    let table = doc.create_element("div");
    doc.append_child(body, table).unwrap();
    let test = doc.create_element("div");
    doc.append_child(table, test).unwrap();
    let row1 = doc.create_element("div");
    doc.append_child(test, row1).unwrap();
    let row2 = doc.create_element("div");
    doc.append_child(test, row2).unwrap();
    let cell_a = doc.create_element("div");
    doc.append_child(row1, cell_a).unwrap();
    let cell_b = doc.create_element("div");
    doc.append_child(row1, cell_b).unwrap();
    let cell_c = doc.create_element("div");
    doc.append_child(row2, cell_c).unwrap();
    let cell_d = doc.create_element("div");
    doc.append_child(row2, cell_d).unwrap();
    let t_a = doc.create_text_node("a");
    doc.append_child(cell_a, t_a).ok();
    let t_b = doc.create_text_node("b");
    doc.append_child(cell_b, t_b).ok();
    let t_c = doc.create_text_node("c");
    doc.append_child(cell_c, t_c).ok();
    let t_d = doc.create_text_node("d");
    doc.append_child(cell_d, t_d).ok();

    let mut styles = HashMap::new();
    let mut t = ComputedStyle::default();
    t.display = DisplayValue::Table;
    t.width = LengthValue::Percentage(100.0);
    styles.insert(table, t);

    // #test：style-system 已把 table-row-group+float 块化为 Block+float。
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Block;
    ts.float = float;
    ts.background_color = ColorValue::Named("blue".to_string());
    styles.insert(test, ts);

    let mut row = ComputedStyle::default();
    row.display = DisplayValue::TableRow;
    styles.insert(row1, row.clone());
    styles.insert(row2, row);

    let mut cell = ComputedStyle::default();
    cell.display = DisplayValue::TableCell;
    cell.width = LengthValue::Px(48.0);
    cell.height = LengthValue::Px(48.0);
    cell.color = ColorValue::Named("blue".to_string());
    styles.insert(cell_a, cell.clone());
    styles.insert(cell_b, cell.clone());
    styles.insert(cell_c, cell.clone());
    styles.insert(cell_d, cell);

    (doc, styles, test, row1, cell_a, cell_b)
}

/// R1382：table-row-group + float:right 块化后，内部 .row 经匿名 table 包装 + grid，
/// #test shrink-to-fit 到 96×96 浮右（右缘贴容器右边）。
#[test]
fn r1382_float_applies_to_row_group_right() {
    let (doc, styles, test, _row1, cell_a, cell_b) = build_float_applies_to_001(FloatValue::Right);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let test_box = find_child_by_node_id(&result.root, test).expect("#test found");
    // 96×96（2 列 × 2 行，cell 48×48）。
    assert!(
        (test_box.width - 96.0).abs() < 1.0,
        "#test width should be 96 (anon table 2 cols), got {}",
        test_box.width
    );
    assert!(
        (test_box.height - 96.0).abs() < 1.0,
        "#test height should be 96 (anon table 2 rows), got {}",
        test_box.height
    );
    // 右浮动：右缘贴容器右边（800）。
    assert!(
        (test_box.x + test_box.width - 800.0).abs() < 1.0,
        "#test right edge should hug container right (800), got x={} right={}",
        test_box.x,
        test_box.x + test_box.width
    );

    // cell 应并排（grid），非垂直堆叠。
    let cell_a_box = find_child_by_node_id(&result.root, cell_a).expect("cell_a found");
    let cell_b_box = find_child_by_node_id(&result.root, cell_b).expect("cell_b found");
    assert!(
        cell_b_box.x >= cell_a_box.x + 47.0,
        "cells should be side by side (grid), cell_a.x={} cell_b.x={}",
        cell_a_box.x,
        cell_b_box.x
    );
}

/// R1382：float:left 变体——左浮动保持左缘，宽仍 96。
#[test]
fn r1382_float_applies_to_row_group_left() {
    let (doc, styles, test, _row1, _cell_a, _cell_b) = build_float_applies_to_001(FloatValue::Left);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let test_box = find_child_by_node_id(&result.root, test).expect("#test found");
    assert!(
        (test_box.width - 96.0).abs() < 1.0,
        "#test width should be 96, got {}",
        test_box.width
    );
    // 左浮动：左缘贴容器左边（0）。
    assert!(
        test_box.x.abs() < 1.0,
        "#test left edge should hug container left (0), got x={}",
        test_box.x
    );
}

/// R1382 回归守护：真实 table 内的 table-row-group（非块化、非浮动）仍正常工作——
/// `inside_table` 修复不得把在表内的 row-group 子误判为孤立。
#[test]
fn r1382_in_table_row_group_not_treated_as_orphan() {
    let (mut doc, body) = make_doc_with_body();
    let table = doc.create_element("div");
    doc.append_child(body, table).unwrap();
    let rg = doc.create_element("div");
    doc.append_child(table, rg).unwrap();
    let row = doc.create_element("div");
    doc.append_child(rg, row).unwrap();
    let cell_a = doc.create_element("div");
    doc.append_child(row, cell_a).unwrap();
    let cell_b = doc.create_element("div");
    doc.append_child(row, cell_b).unwrap();
    let t_a = doc.create_text_node("a");
    doc.append_child(cell_a, t_a).ok();
    let t_b = doc.create_text_node("b");
    doc.append_child(cell_b, t_b).ok();

    let mut styles = HashMap::new();
    let mut t = ComputedStyle::default();
    t.display = DisplayValue::Table;
    styles.insert(table, t);
    let mut rgs = ComputedStyle::default();
    rgs.display = DisplayValue::TableRowGroup;
    styles.insert(rg, rgs);
    let mut rs = ComputedStyle::default();
    rs.display = DisplayValue::TableRow;
    styles.insert(row, rs);
    let mut cell = ComputedStyle::default();
    cell.display = DisplayValue::TableCell;
    cell.width = LengthValue::Px(48.0);
    cell.height = LengthValue::Px(48.0);
    styles.insert(cell_a, cell.clone());
    styles.insert(cell_b, cell);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // cell 并排（grid 在真实 table 内正常工作）。
    let cell_a_box = find_child_by_node_id(&result.root, cell_a).expect("cell_a found");
    let cell_b_box = find_child_by_node_id(&result.root, cell_b).expect("cell_b found");
    assert!(
        cell_b_box.x >= cell_a_box.x + 47.0,
        "in-table cells should be side by side, cell_a.x={} cell_b.x={}",
        cell_a_box.x,
        cell_b_box.x
    );
}

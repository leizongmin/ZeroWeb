//! R1390 回归测试：table-cell 建立 BFC，其高度须包含浮动子（CSS §9.4.1 + §10.6.7），
//! 且该 BFC 包含高度须传播到行高与 table 高度。
//!
//! 背景（floats-wrap-bfc-001-left-table）：outer table(width:300) > td 内含
//!   - float:left div 100×100
//!   - 内含 150×50 内容的子元素（in-flow，比浮动矮）
//!
//! td 作为 BFC 必须包含浮动（高度 ≥ 100），且 table 高度须反映该单元格的 BFC 高度。
//!
//! 修复前症状：taffy 给 td 的 cell.height 不含浮动（≈50），position_cells 的 row_height
//! 取 cell.height（taffy 值，早于 cell 内容高度修正），导致 table 高度 = 50 < 浮动 100，
//! td 溢出不可见，与 chromium oracle 差 2.08%（aqua 背景只画到 50 而非 100）。
//!
//! 修复：row_height 取 max(taffy cell.height, cell_float_aware_content_height)，
//! 使 table 高度反映单元格 BFC 的浮动包含。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_style_system::ComputedStyle;

/// 构造 floats-wrap-bfc-001 的核心结构：table(width:300) > tr > td 内含
/// float:left(100×100) + in-flow block(150×50)。
/// 返回 (doc, styles, table_id, td_id, float_id)。
fn build_float_in_table_cell() -> (
    zero_dom::Document,
    HashMap<zero_dom::NodeId, ComputedStyle>,
    zero_dom::NodeId,
    zero_dom::NodeId,
    zero_dom::NodeId,
) {
    let (mut doc, body) = make_doc_with_body();
    let table = doc.create_element("div");
    doc.append_child(body, table).unwrap();
    let tr = doc.create_element("div");
    doc.append_child(table, tr).unwrap();
    let td = doc.create_element("div");
    doc.append_child(tr, td).unwrap();
    let float_div = doc.create_element("div");
    doc.append_child(td, float_div).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(td, inner).unwrap();

    let mut styles = HashMap::new();
    let mut t = ComputedStyle::default();
    t.display = DisplayValue::Table;
    t.width = LengthValue::Px(300.0);
    styles.insert(table, t);

    let mut tr_s = ComputedStyle::default();
    tr_s.display = DisplayValue::TableRow;
    styles.insert(tr, tr_s);

    let mut td_s = ComputedStyle::default();
    td_s.display = DisplayValue::TableCell;
    styles.insert(td, td_s);

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(100.0);
    fl.height = LengthValue::Px(100.0);
    styles.insert(float_div, fl);

    let mut in_s = ComputedStyle::default();
    in_s.display = DisplayValue::Block;
    in_s.width = LengthValue::Px(150.0);
    in_s.height = LengthValue::Px(50.0);
    styles.insert(inner, in_s);

    (doc, styles, table, td, float_div)
}

/// R1390：table-cell 的 BFC 高度（含浮动）传播到 table 高度。
/// 修复前 table.height ≈ 50（taffy cell 值，不含浮动），应 ≥ 100。
#[test]
fn r1390_table_cell_bfc_contains_float_propagates_to_table_height() {
    let (doc, styles, table, td, _float) = build_float_in_table_cell();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let table_box = find_child_by_node_id(&result.root, table).expect("table found");
    let td_box = find_child_by_node_id(&result.root, td).expect("td found");

    // 单元格 BFC 必须包含 100 高的浮动子。
    assert!(
        td_box.height >= 99.0,
        "td (BFC) height should contain the 100px float, got {}",
        td_box.height
    );
    // table 高度须反映单元格 BFC 高度（修复前被 taffy cell 值 50 截断）。
    assert!(
        table_box.height >= 99.0,
        "table height should propagate td BFC float-containment (>=100), got {}",
        table_box.height
    );
}

/// R1390：单元格只含 in-flow 子（无浮动）时，BFC 高度 = in-flow 子高度，table 行为不变。
/// 守护「无浮动 no-op」——避免 cell_float_aware_content_height 对普通单元格副作用。
#[test]
fn r1390_table_cell_no_float_unchanged() {
    let (mut doc, body) = make_doc_with_body();
    let table = doc.create_element("div");
    doc.append_child(body, table).unwrap();
    let tr = doc.create_element("div");
    doc.append_child(table, tr).unwrap();
    let td = doc.create_element("div");
    doc.append_child(tr, td).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(td, inner).unwrap();

    let mut styles = HashMap::new();
    let mut t = ComputedStyle::default();
    t.display = DisplayValue::Table;
    t.width = LengthValue::Px(300.0);
    styles.insert(table, t);
    let mut tr_s = ComputedStyle::default();
    tr_s.display = DisplayValue::TableRow;
    styles.insert(tr, tr_s);
    let mut td_s = ComputedStyle::default();
    td_s.display = DisplayValue::TableCell;
    styles.insert(td, td_s);
    let mut in_s = ComputedStyle::default();
    in_s.display = DisplayValue::Block;
    in_s.width = LengthValue::Px(150.0);
    in_s.height = LengthValue::Px(50.0);
    styles.insert(inner, in_s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let table_box = find_child_by_node_id(&result.root, table).expect("table found");
    // 无浮动：table 高度仅来自 in-flow 子（50），不受 float_bottom=0 的 max 影响。
    assert!(
        table_box.height < 80.0,
        "table without float should not inflate (in-flow 50), got {}",
        table_box.height
    );
}

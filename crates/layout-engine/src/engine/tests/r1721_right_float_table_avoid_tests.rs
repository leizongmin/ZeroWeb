//! R1721：float:right table 避到 float 左侧（mirror of float:left 右避）。
//!
//! `floats-wrap-bfc-002-right-table`：外 table 300 内 `float:right 100×100` + 嵌套 auto table。
//! chromium（ref）：table beside float **左侧**（x=0, width=200=float.left_edge）。
//! ZW 旧（R1721 前）：table_float_fix 仅 float:left（table 放 float 右侧 content_width-max_right），
//! float:right 的 right_edge≈content_width → 右侧无空间 → table 错误推 below（y=100+）。
//! R1721 fix：纯右 float → target=(0, natural_y, right_float_left)，table beside 左填到 float 左边。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_style_system::ComputedStyle;

fn build_right_float_table() -> (
    zero_dom::Document,
    HashMap<zero_dom::NodeId, ComputedStyle>,
    zero_dom::NodeId,
    zero_dom::NodeId,
    zero_dom::NodeId,
) {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_r = doc.create_element("div");
    doc.append_child(container, float_r).unwrap();
    let table = doc.create_element("div");
    doc.append_child(container, table).unwrap();
    let tr = doc.create_element("div");
    doc.append_child(table, tr).unwrap();
    let td = doc.create_element("div");
    doc.append_child(tr, td).unwrap();
    let cell_content = doc.create_element("div");
    doc.append_child(td, cell_content).unwrap();

    let mut styles = HashMap::new();
    // 容器 width=300（auto height）。
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(300.0);
    styles.insert(container, cont);

    // float:right 100×100（贴右侧，left_edge=200）。
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Right;
    fl.width = LengthValue::Px(100.0);
    fl.height = LengthValue::Px(100.0);
    styles.insert(float_r, fl);

    let mut t = ComputedStyle::default();
    t.display = DisplayValue::Table;
    styles.insert(table, t);
    let mut tr_s = ComputedStyle::default();
    tr_s.display = DisplayValue::TableRow;
    styles.insert(tr, tr_s);
    let mut td_s = ComputedStyle::default();
    td_s.display = DisplayValue::TableCell;
    styles.insert(td, td_s);
    // cell 内容宽 50（< 200 beside 空间，table 应 beside 非 below）。
    let mut cc = ComputedStyle::default();
    cc.display = DisplayValue::Block;
    cc.width = LengthValue::Px(50.0);
    cc.height = LengthValue::Px(20.0);
    styles.insert(cell_content, cc);

    (doc, styles, container, table, float_r)
}

/// R1721：float:right 时 table 应 beside float 左侧（x≈0, width≈200），非推 below（y≈100）。
/// load-bearing：关闭 fix（env ZW_TABLE_FLOAT_RIGHT_AVOID=0）则 table 被推 below（y≈100）。
#[test]
fn r1721_right_float_table_beside_left_not_below() {
    let (doc, styles, _container, table, _float_r) = build_right_float_table();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let table_box = find_child_by_node_id_root(&result.root, table).expect("table found");
    // table 在 float 左侧 beside：x≈0（左边缘），width≈200（填到 float left_edge），y≈0（非 below）。
    assert!(
        table_box.width >= 190.0 && table_box.width <= 210.0,
        "beside float:right table width 应 ≈200（填到 float left_edge），实际 {}",
        table_box.width
    );
    assert!(
        table_box.y < 50.0,
        "table 应 beside float（y≈0）非推 below（y≈100），实际 y={}",
        table_box.y
    );
}

/// 辅助：从 result.root 深度查找 node_id 对应盒（table 嵌套在 container 下）。
fn find_child_by_node_id_root(root: &LayoutBox, target_id: zero_dom::NodeId) -> Option<&LayoutBox> {
    if root.node_id == Some(target_id) {
        return Some(root);
    }
    for c in &root.children {
        if let Some(found) = find_child_by_node_id_root(c, target_id) {
            return Some(found);
        }
    }
    None
}

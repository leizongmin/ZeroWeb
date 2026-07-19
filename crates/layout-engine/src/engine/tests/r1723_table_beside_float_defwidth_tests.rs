//! R1723：definite-width table 旁 float 放不下 → 推到 float 下方保声明宽（非 shrink beside）。
//!
//! `floats-wrap-bfc-005` 子案 1/2：`<table width="50%">`（=150）旁 200px float（300 容器，
//! 可用 beside 宽 100）。CSS §9.5：BFC table border-box 不重叠 float，definite 宽 150 > 100 可用
//! → 推到 float 下方（y=20）保持 width=150。ZW 旧（R1723 前）：step5 把 table shrink 到 100 beside
//! （table_float_fix C 算法读到 shrink 后宽 100 → 误判 fits beside）。R1723：用 declared
//! effective_w（Percentage 解析）做 fit 决策，below 时恢复声明宽。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_style_system::ComputedStyle;

/// 构造 bfc-005 单子案：300 容器内 float 200×20 + table width=50%（应 150）。
fn build_bfc005(
    float_dir: FloatValue,
) -> (
    zero_dom::Document,
    HashMap<zero_dom::NodeId, ComputedStyle>,
    zero_dom::NodeId,
    zero_dom::NodeId,
) {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_div = doc.create_element("div");
    doc.append_child(container, float_div).unwrap();
    let table = doc.create_element("div");
    doc.append_child(container, table).unwrap();
    let tr = doc.create_element("div");
    doc.append_child(table, tr).unwrap();
    let td = doc.create_element("div");
    doc.append_child(tr, td).unwrap();
    let cell_content = doc.create_element("div");
    doc.append_child(td, cell_content).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(300.0);
    styles.insert(container, cont);

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = float_dir;
    fl.width = LengthValue::Px(200.0);
    fl.height = LengthValue::Px(20.0);
    styles.insert(float_div, fl);

    // table width=50%（=150，> 100 beside 可用 → 应推下保宽）。
    let mut t = ComputedStyle::default();
    t.display = DisplayValue::Table;
    t.width = LengthValue::Percentage(50.0);
    styles.insert(table, t);
    let mut tr_s = ComputedStyle::default();
    tr_s.display = DisplayValue::TableRow;
    styles.insert(tr, tr_s);
    let mut td_s = ComputedStyle::default();
    td_s.display = DisplayValue::TableCell;
    td_s.height = LengthValue::Px(20.0);
    styles.insert(td, td_s);
    let mut cc = ComputedStyle::default();
    cc.display = DisplayValue::Block;
    cc.width = LengthValue::Px(50.0);
    cc.height = LengthValue::Px(20.0);
    styles.insert(cell_content, cc);

    (doc, styles, table, float_div)
}

/// R1723 子案 1：float:left + table 50%。table 应推到 float 下方（y≈20）保 width=150，
/// 非 shrink 到 100 beside（y≈0）。
#[test]
fn r1723_left_float_defwidth_table_pushed_below() {
    let (doc, styles, table, _float_div) = build_bfc005(FloatValue::Left);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let table_box = find_child_by_node_id_root(&result.root, table).expect("table found");
    assert!(
        table_box.width >= 145.0 && table_box.width <= 155.0,
        "float:left definite-width table 应保持 width≈150（推下非 shrink），实际 {}",
        table_box.width
    );
    assert!(
        table_box.y > 10.0,
        "table 应推到 float 下方（y≈20）非 beside（y≈0），实际 y={}",
        table_box.y
    );
}

/// R1723 子案 2：float:right + table 50%。table 应推到 float 下方（y≈20）保 width=150，
/// 非 beside 左侧 shrink 到 100。
#[test]
fn r1723_right_float_defwidth_table_pushed_below() {
    let (doc, styles, table, _float_div) = build_bfc005(FloatValue::Right);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let table_box = find_child_by_node_id_root(&result.root, table).expect("table found");
    assert!(
        table_box.width >= 145.0 && table_box.width <= 155.0,
        "float:right definite-width table 应保持 width≈150（推下非 shrink），实际 {}",
        table_box.width
    );
    assert!(
        table_box.y > 10.0,
        "table 应推到 float 下方（y≈20）非 beside（y≈0），实际 y={}",
        table_box.y
    );
}

/// R1723 回归守卫：auto-width table 旁 float 仍 beside 填满（R1613/R1721 行为不变）。
/// auto-width table（无 width）旁 float:left 100，容器 300 → table beside 填 200，y≈0。
#[test]
fn r1723_auto_width_table_still_beside_fill() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_l = doc.create_element("div");
    doc.append_child(container, float_l).unwrap();
    let table = doc.create_element("div");
    doc.append_child(container, table).unwrap();
    let tr = doc.create_element("div");
    doc.append_child(table, tr).unwrap();
    let td = doc.create_element("div");
    doc.append_child(tr, td).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(300.0);
    styles.insert(container, cont);
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(100.0);
    fl.height = LengthValue::Px(100.0);
    styles.insert(float_l, fl);
    // auto-width table（无 width 声明）→ declared_w=None，effective_w=当前 table_w。
    let mut t = ComputedStyle::default();
    t.display = DisplayValue::Table;
    styles.insert(table, t);
    let mut tr_s = ComputedStyle::default();
    tr_s.display = DisplayValue::TableRow;
    styles.insert(tr, tr_s);
    let mut td_s = ComputedStyle::default();
    td_s.display = DisplayValue::TableCell;
    td_s.height = LengthValue::Px(20.0);
    styles.insert(td, td_s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let table_box = find_child_by_node_id_root(&result.root, table).expect("table found");
    // auto table beside float:left → 填到 float 右可用宽 ≈200，y≈0（非推下）。
    assert!(
        table_box.y < 50.0,
        "auto-width table 应 beside float（y≈0）非推下，实际 y={}",
        table_box.y
    );
    assert!(
        table_box.x > 50.0,
        "auto-width table 应在 float 右侧（x≈100），实际 x={}",
        table_box.x
    );
}

/// 辅助：深度查找 node_id 对应盒。
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

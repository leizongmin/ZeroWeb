//! R5444：cell 固有宽不计 OOF（absolute/fixed）子（CSS2 §17.5.2 + css-position-3
//! §9.1，R5370 同门）。
//!
//! driving：css/CSS2/css21-errata/s-11-1-1b-005（html{display:table} + body{
//! display:table-cell;width:20px} + abs p）1.52%——cell/表被 abs p 的 max-content
//! 撑到 279.5，cell 黑底带随之 279.5 宽（chromium oracle：表/cell = 20，方块 20×20）。
//! 探针定谳（probe C/D/E）：三臂旧径全泄——显式宽臂（abs p width:300px → 列 300）、
//! R4028 max-content 臂（auto cell → 279.5）、95% 启发式臂（laid-out 宽）。R4029 的
//! 「abspos 定位以 cell 为包含块」前提勘误：abs 子 y 按 ICB 解析（top:0+mt16，cell
//! 在 y=40 时 abs_y 仍 16）、宽按父 cell 退化可用宽塌 min-content（43）——cell 计入
//! abs 宽只撑列不救 abs 子。`ZW_CELL_INTRINSIC_OOF_SKIP=0` 回退计数行为。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find(root: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

fn compute(html: &str, ids: &[&'static str]) -> (crate::types::LayoutResult, Vec<(&'static str, NodeId)>) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let pairs: Vec<(&'static str, NodeId)> = ids
        .iter()
        .map(|n| (*n, doc.get_element_by_id(n).expect("id")))
        .collect();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    (engine.compute(&doc, &styles), pairs)
}

/// probe C 形（嵌套 table > cell(width:20px) > abs p 自动宽长文本）：表/cell 宽 = 20
/// （显式 cell 宽 + 无流内内容 → 无 max-content 泄漏）。旧缺陷签名：表/cell = 279.5
/// （abs p max-content 经 R4028/R4029 臂计入）。
#[test]
fn r5444_cell_intrinsic_skips_abs_child_max_content() {
    let html = r#"<html><body style="margin:0">
<div id="tbl" style="display:table">
  <div id="cell" style="display:table-cell; width:20px; height:20px; background:black;">
    <p style="position:absolute; top:0">Test passes if there is a black square below.</p>
  </div>
</div>
</body></html>"#;
    let (result, ids) = compute(html, &["tbl", "cell"]);
    let tbl = find(&result.root, ids[0].1).expect("tbl box");
    let cell = find(&result.root, ids[1].1).expect("cell box");
    eprintln!("R5444DBG tbl w={} cell w={}", tbl.width, cell.width);
    assert!(
        tbl.width <= 25.0,
        "表宽应 ≈20（abs 子不计固有宽），实际 {}",
        tbl.width
    );
    assert!(
        cell.width <= 25.0,
        "cell 宽应 ≈20（显式宽不被 abs 子 max-content 撑破），实际 {}",
        cell.width
    );
}

/// 流内内容照常计入（防过跳）：cell 内显式宽 50 的流内 div 决定列宽，abs 子不计也不
/// 干扰。
#[test]
fn r5444_in_flow_child_still_contributes() {
    let html = r#"<html><body style="margin:0">
<div id="tbl" style="display:table">
  <div id="cell" style="display:table-cell">
    <div id="flow" style="width:50px; height:20px;"></div>
    <p style="position:absolute; top:0">Test passes if there is a black square below.</p>
  </div>
</div>
</body></html>"#;
    let (result, ids) = compute(html, &["tbl", "cell"]);
    let tbl = find(&result.root, ids[0].1).expect("tbl box");
    let cell = find(&result.root, ids[1].1).expect("cell box");
    assert!(
        (cell.width - 50.0).abs() < 2.0,
        "cell 宽应 = 流内内容 50（abs 子不计），实际 {}",
        cell.width
    );
    assert!(
        tbl.width <= 52.0,
        "表宽收缩到流内内容（≈50），实际 {}",
        tbl.width
    );
}

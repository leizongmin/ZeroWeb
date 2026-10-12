//! R5080（CSS2 §17.3 Applies-to）：min/max-height 不适用于 table-column-group /
//! table-column——三层同门（converter 清零 / postprocess 1)/1a)/1b) 钳制层跳过 /
//! 行高链不受扰）。
//! driving: css/CSS2/normal-flow min-height-applies-to-005（colgroup min-height:1in
//! 旧渲 96×96 红块，应无红）/ 006。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.x, b.y, b.width));
        }
        stack.extend(b.children.iter());
    }
    None
}

fn find_height(root: &LayoutBox, node_id: NodeId) -> Option<f32> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some(b.height);
        }
        stack.extend(b.children.iter());
    }
    None
}

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// 005 形态：colgroup min-height:96 + 表 1in 宽双行 19px 白底 cell——colgroup 盒不得
/// 被 min-height 撑高（taffy 层与后处理钳制层都不得写入），表高 = 行高 38。
#[test]
fn r5080_colgroup_min_height_not_applied() {
    let html = r#"<html><body style="margin:0"><div style="display:table;table-layout:fixed;width:96px;background:red"><div style="background:red;display:table-column-group;min-height:96px"><div style="display:table-column"></div><div style="display:table-column"></div></div><div style="display:table-row"><div style="background:white;display:table-cell;height:19px"></div><div style="background:white;display:table-cell;height:19px"></div></div></div></body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    // divs[0]=table、divs[1]=colgroup。
    let cg_h = find_height(&result.root, divs[1]).expect("colgroup box");
    assert!(
        cg_h <= 0.5 || cg_h.is_nan(),
        "colgroup 盒高不得被 min-height:96 撑高（§17.3），实际 {cg_h}"
    );
    let (_, _, th) = find_box(&result.root, divs[0]).expect("table box");
    let _ = th;
    let table_h = find_height(&result.root, divs[0]).expect("table height");
    assert!(
        (table_h - 19.0).abs() < 0.5,
        "表高应 = 行高链 19（无 96 泄漏），实际 {table_h}"
    );
}

/// 006 形态：table-column 自身 min-height——同门清零。
#[test]
fn r5080_column_min_height_not_applied() {
    let html = r#"<html><body style="margin:0"><div style="display:table;table-layout:fixed;width:96px;background:red"><div style="display:table-column;min-height:96px;background:red"></div><div style="display:table-row"><div style="background:white;display:table-cell;height:19px"></div><div style="background:white;display:table-cell;height:19px"></div></div></div></body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let col_h = find_height(&result.root, divs[1]).expect("column box");
    assert!(
        col_h <= 0.5 || col_h.is_nan(),
        "table-column 盒高不得被 min-height:96 撑高（§17.3），实际 {col_h}"
    );
}

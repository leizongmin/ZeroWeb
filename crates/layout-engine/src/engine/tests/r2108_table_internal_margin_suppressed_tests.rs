//! R2108：CSS §8.4 margin 不应用于 table-cell / table-column / table-column-group。
//! driving cluster：margin-applies-to-005/006/007 + margin-bottom-applies-to-005/006/007
//! （css/CSS2/margin-padding-clear，6 案全 0.00% PASS）。cell 的 **padding** 仍应用（§17.5）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeKind;
use zero_style_system::StyleSystem;

fn layout(html: &str) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result.root)
}

/// DFS 找最深的 `div` 元素盒（用于定位 driving case 中的 table-cell div）。
fn deepest_div<'a>(doc: &zero_dom::Document, root: &'a LayoutBox) -> Option<&'a LayoutBox> {
    fn dfs<'a>(b: &'a LayoutBox, doc: &zero_dom::Document, depth: i32, best: &mut Option<(i32, &'a LayoutBox)>) {
        let is_div = b
            .node_id
            .and_then(|id| doc.get(id))
            .map(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name() == "div"))
            .unwrap_or(false);
        if is_div && best.is_none_or(|(bd, _)| depth > bd) {
            *best = Some((depth, b));
        }
        for c in &b.children {
            dfs(c, doc, depth + 1, best);
        }
    }
    let mut best: Option<(i32, &LayoutBox)> = None;
    dfs(root, doc, 0, &mut best);
    best.map(|(_, b)| b)
}

/// R2108 主驱：display:table-cell 的 margin:50px 应被忽略（margin=0），CSS §8.4。
#[test]
fn r2108_table_cell_margin_suppressed() {
    let html = r#"<div style="display:table;">
  <div style="display:table-row;">
    <div style="display:table-cell; margin:50px; width:100px; height:100px;">cell</div>
  </div>
</div>"#;
    let (doc, root) = layout(html);
    let cell = deepest_div(&doc, &root).expect("table-cell div not found");
    // R2108：margin:50px 应被忽略 → margin_top/margin_left ≈ 0（非 50）。
    assert!(
        cell.margin_top.abs() < 1.0 && cell.margin_left.abs() < 1.0,
        "table-cell margin must be suppressed (CSS §8.4); got mt={} ml={}",
        cell.margin_top,
        cell.margin_left
    );
}

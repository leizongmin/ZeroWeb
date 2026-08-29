//! R3792（css-sizing-4 §4.1 transferred size）：块内路径 aspect-ratio transferred width。
//!
//! `width:auto + aspect-ratio + definite height` 的块盒对父 `width:min-content` /
//! `max-content` 的 intrinsic 宽贡献 = height × ratio。旧 box_content_max_width 对此类
//! 盒测 0 → 父回退 Auto 满宽（intrinsic-size-001：`width:min-content` 父内
//! `height:100px; aspect-ratio:1/1` 子应贡献 100px 绿方块，旧渲满宽绿条）。
//! 内部 table 盒排除（css-sizing-4：「aspect-ratio does not apply to internal table
//! boxes」，table-element-001：td ratio 不 transferred）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.width, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

/// intrinsic-size-001 驱动案：min-content 父内 aspect-ratio 子应贡献 100px 宽（绿方块）。
#[test]
fn r3792_aspect_ratio_child_contributes_to_min_content_parent() {
    let html = r#"<html><body style="margin:0">
<div style="width: min-content; height: 100px; background: green;">
  <div style="height: 100px; aspect-ratio: 1/1;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let outer = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .next()
        .expect("outer div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, h) = find_box(&result.root, outer).expect("outer box found");
    assert!((h - 100.0).abs() < 1.0, "R3792: 外层高应 100，实际 {h}");
    assert!(
        (w - 100.0).abs() < 1.5,
        "R3792: aspect-ratio 子 transferred width 应使 min-content 父宽=100，实际 {w}（旧测 0 → 父满宽 784）"
    );
}

/// table-element-001 守卫：内部 table 盒（td）的 aspect-ratio 不 transferred——
/// td `height:50px; aspect-ratio:4/1` 不应得 200px intrinsic 宽。
#[test]
fn r3792_internal_table_box_excluded_from_transfer() {
    let html = r#"<html><body style="margin:0">
<table><tr>
<th style='background: green; width: 100px; aspect-ratio: 1/1;'>x</th>
<td style='background: red; height: 50px; aspect-ratio: 4/1;'>y</td>
</tr></table>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let td = doc.get_elements_by_tag_name("td").into_iter().next().expect("td");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, _h) = find_box(&result.root, td).expect("td box found");
    assert!(
        w < 120.0,
        "R3792: 内部 table 盒 aspect-ratio 不应 transferred（td 宽 {w} 应远小于 200）"
    );
}

//! R3817 回归：position-absolute-center-006——abspos display:table（left+right:0 +
//! margin:auto + height:100px）内含 100×100 block 子。
//!
//! 匿名 cell 包裹臂（R3815/R3817）把该子包入匿名 cell 后，表走 grid 正常路径，
//! apply_table_size_constraints 收缩表宽须保留水平 auto-margin 居中（§10.3.7）：
//! 表收 100 后应居中于 200px CB 的 x=50。旧实现 grid 非空路径无居中臂（仅
//! shrink_table_to_block_content 早返路径有），x 留 left=0。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

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

#[test]
fn r3817_abspos_table_hcenter_after_anon_cell_wrap() {
    let html = r#"<html><body>
<div style="position:relative; width:200px; height:100px; margin-left:-50px">
  <div id="t" style="display:table; position:absolute; background:green; left:0; right:0; margin:auto; height:100px">
    <div id="c" style="width:100px; height:100px"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let t = doc.get_element_by_id("t").expect("t");
    let c = doc.get_element_by_id("c").expect("c");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let tb = find(&result.root, t).expect("table box");
    let cb = find(&result.root, c).expect("child box");

    // 表收缩到内容宽 100，水平居中于 200px CB（x=50）。
    assert!(
        (tb.width - 100.0).abs() < 1.0 && tb.height - 100.0 < 1.0,
        "table should be 100x100, got {}x{}",
        tb.width,
        tb.height
    );
    assert!(
        (tb.x - 50.0).abs() < 1.0,
        "abspos table should center horizontally (x=50), got x={}",
        tb.x
    );
    // 子块几何不受包裹影响。
    assert!(
        (cb.width - 100.0).abs() < 1.0 && (cb.height - 100.0).abs() < 1.0,
        "child should stay 100x100, got {}x{}",
        cb.width,
        cb.height
    );
}

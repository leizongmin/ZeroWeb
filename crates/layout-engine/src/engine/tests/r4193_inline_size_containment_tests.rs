//! R4193（css-contain-3 §containment-inline-size）：contain:inline-size 的内联尺寸抑制。
//!
//! `contain:inline-size + width:fit-content` 容器内联尺寸按**空内容**求解（CIS-or-0）：
//! regular-container/flex/grid 三案（各 5.26%）——内容 100px 不参与，纯 border 0 50px
//! = 100 宽绿方块。两道修：① converter size 臂对 has_inline_size() 生效（width 轴
//! CIS-or-0，height 轴照常）；② intrinsic 测量趟跳过 inline-size-contained 元素
//!（防测量值覆盖受控宽）。

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
        for c in &b.children {
            stack.push(c);
        }
    }
    None
}

/// block 容器：contain:inline-size + width:fit-content + border 0 50px → 100 宽（纯边框）。
#[test]
fn r4193_inline_size_containment_collapses_fit_content_width() {
    let html = r#"<html><head><style>body { margin: 0; }</style></head><body>
<div style="contain:inline-size; width:fit-content; border:solid green; border-width:0 50px; background:red;">
  <div style="width:100px; height:100px;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(
        "body { margin: 0; } div { contain: inline-size; width: fit-content; border: solid green; border-width: 0 50px; }",
    );
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let container = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .next()
        .expect("container div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, h) = find_box(&result.root, container).expect("container box");
    assert!(
        (w - 100.0).abs() < 0.5,
        "R4193: inline-size containment 下 fit-content 宽应为纯边框 100，实际 {w}（旧实现测内容 200→300）"
    );
    assert!(
        (h - 100.0).abs() < 0.5,
        "R4193: block 轴无 containment，高度照常 = 内容 100，实际 {h}"
    );
}

//! R4185（css-flexbox-1 §algo-cross-line step 4）：单行 flex 容器 line cross 钳制。
//!
//! flexbox-single-line-clamp-1：容器 `display:flex; max-height:200px`，panel（cross Auto，
//! stretch 对齐）内容 402 → 应钳到 200；panel 内定高 tall-child（height:400）不动
//! （钳制的是 line，非 item 的 definite cross）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<f32> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some(b.height);
        }
        for c in &b.children {
            stack.push(c);
        }
    }
    None
}

/// 单行 row flex + max-height:200：stretch panel 钳 200，定高 tall-child 保持 400。
#[test]
fn r4185_single_line_flex_cross_clamped_to_max_height() {
    let html = r#"<html><head><style>
.container { display: flex; max-height: 200px; }
.panel { width: 150px; border: 1px solid purple; box-sizing: border-box; }
.tall-child { width: 50px; height: 400px; }
</style></head><body style="margin:0">
<div class="container">
  <div class="panel">
    <div class="tall-child"></div>
  </div>
</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(
        ".container { display: flex; max-height: 200px; } .panel { width: 150px; border: 1px solid purple; box-sizing: border-box; } .tall-child { width: 50px; height: 400px; }",
    );
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let panel = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .nth(1)
        .expect("panel div");
    let tall = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .nth(2)
        .expect("tall-child div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let panel_h = find_box(&result.root, panel).expect("panel box");
    let tall_h = find_box(&result.root, tall).expect("tall box");
    assert!(
        (panel_h - 200.0).abs() < 0.5,
        "R4185: stretch panel 应钳到 line cross 200，实际 {panel_h}"
    );
    assert!(
        (tall_h - 400.0).abs() < 0.5,
        "R4185: 定高 tall-child 不受 line 钳制，实际 {tall_h}"
    );
}

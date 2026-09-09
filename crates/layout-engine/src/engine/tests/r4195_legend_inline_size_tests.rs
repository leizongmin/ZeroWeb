//! R4195（css-contain-3 §containment-inline-size）：无 form 父的 fieldset legend
//! inline-size 抑制。
//!
//! contain-inline-size-legend（11.54%）：fieldset 直挂 body（`width:fit-content;
//! min-width:0`），legend 带 `contain:inline-size; border:0 50px`，内容 500px。
//! R4062 臂仅在 `<form>`（auto-height Block）触发的 layout_direct_fieldsets 中可达
//! ——fieldset-only 分派 gate 遇非控件子（`<p>` 兄弟 / 无 form）整树 return，臂永不
//! 执行 → legend width:auto fill fieldset content 宽 604（应 100 纯边框）。
//! R4195 独立递归 pass `shrink_inline_size_legends` 补齐：fieldset 直接子 legend 带
//! inline-size containment → content_width=0，legend 宽 = padding + border。

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

/// fieldset 直挂 body（无 form、有 `<p>` 兄弟）：inline-size-contained legend 宽 =
/// 纯边框 100（内容 500px 不参与），高度照常（块轴无 containment）。
#[test]
fn r4195_inline_size_legend_without_form_collapses_width() {
    let html = r#"<html><head><style>body { margin: 0; }</style></head><body>
<p>x</p>
<fieldset style="width:fit-content; min-width:0;">
  <legend style="contain:inline-size; border:solid blue; border-width:0 50px; padding:0; background:red;">
    <div style="width:500px; height:100px;"></div>
  </legend>
  <div style="height:100px; background:yellow;"></div>
</fieldset>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet("body { margin: 0; }");
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let legend = doc.get_elements_by_tag_name("legend")[0];
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, h) = find_box(&result.root, legend).expect("legend box");
    assert!(
        (w - 100.0).abs() < 0.5,
        "R4195: 无 form 父时 inline-size-contained legend 宽应为纯边框 100，实际 {w}"
    );
    assert!(
        (h - 100.0).abs() < 0.5,
        "R4195: legend 块轴无 containment，高度照常 = 内容 100，实际 {h}"
    );
}

/// 无 containment 的 legend 不受影响（普通 shrink-to-fit 语义）。
#[test]
fn r4195_plain_legend_unaffected() {
    let html = r#"<html><head><style>body { margin: 0; }</style></head><body>
<fieldset style="width:fit-content; min-width:0;">
  <legend style="border:solid blue; border-width:0 50px; padding:0; background:red;">
    <div style="width:500px; height:100px;"></div>
  </legend>
</fieldset>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet("body { margin: 0; }");
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let legend = doc.get_elements_by_tag_name("legend")[0];
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, _h) = find_box(&result.root, legend).expect("legend box");
    assert!(
        w > 600.0,
        "R4195: 无 containment 的 legend 照常 shrink-to-fit 内容（500+frame），实际 {w}"
    );
}

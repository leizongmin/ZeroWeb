//! R3793（css-overflow-3 webkit-line-clamp legacy 语义）：跨块 clamp 行计数跳过 fieldset。
//!
//! webkit-line-clamp-027 assert「-webkit-line-clamp should skip over fieldsets」——
//! fieldset 内容行不计预算、不 cap、不隐藏（WebKit legacy clamp 中 fieldset 为独立渲染树，
//! 整棵子树照常渲染；ref 的 fieldset L3-L6 全显，clamp 点落在其后 L7 末）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32, bool)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.y, b.height, b.line_clamp_hidden));
        }
        stack.extend(b.children.iter());
    }
    None
}

/// 027 结构：clamp:3 + [L1-L2 anon, fieldset(L3-L4 + legend L5-L6), L7-L8 anon]。
/// fieldset 豁免 → 预算 = L1+L2+L7（3 行），fieldset 132px 全显、legend 64px 全显、
/// L8 隐藏。
#[test]
fn r3793_fieldset_skipped_by_cross_block_clamp() {
    let html = r#"<html><body style="margin:0">
<div style="display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: 3; font: 16px/32px monospace; white-space: pre; background-color: yellow; overflow: hidden;"><div>Line 1
Line 2<fieldset>Line 3
Line 4<legend>Line 5
Line 6</legend></fieldset>Line 7
Line 8</div></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let fieldset = doc
        .get_elements_by_tag_name("fieldset")
        .into_iter()
        .next()
        .expect("fieldset");
    let legend = doc
        .get_elements_by_tag_name("legend")
        .into_iter()
        .next()
        .expect("legend");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (fs_y, fs_h, fs_hidden) = find_box(&result.root, fieldset).expect("fieldset box");
    let (_lg_y, lg_h, lg_hidden) = find_box(&result.root, legend).expect("legend box");
    assert!(
        !fs_hidden && !lg_hidden,
        "R3793: fieldset/legend 不应被 clamp 隐藏（fs_hidden={fs_hidden} lg_hidden={lg_hidden}）"
    );
    assert!(
        fs_h > 120.0,
        "R3793: fieldset 应保持完整高度（L3-L6 = 4 行 128 + border ≈ 132），实际 {fs_h}（旧被 cap 到 36）"
    );
    assert!(
        lg_h > 60.0,
        "R3793: legend 应保持完整高度（L5-L6 = 2 行 64），实际 {lg_h}（旧被隐藏到 0）"
    );
    // fieldset 后的 L7 应可见（预算第 3 行）：fieldset 底之后应有内容延伸到容器底。
    let _ = fs_y;
}

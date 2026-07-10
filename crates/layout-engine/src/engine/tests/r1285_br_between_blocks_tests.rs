//! R1285：`<br>` 在 block 兄弟间的 line-height strut 修复（R1284 root cause → 落地）。
//!
//! `<div/><br><div/>`：br 经 convert_display 映射为 taffy Block leaf（无内容 → height 0），
//! 致 br 在块间渲染 0px（chromium ~line-height）。R1285 在 build_subtree 对 br（有 block
//! 同胞时）设 taffy min_size.height = line-height strut，使其占一行高（CSS §10.8.1 strut）。
//!
//! load-bearing：default-on 时 br.height ≈ line-height；kill-switch `ZW_BR_LINEHEIGHT=0`
//! 时 br.height ≈ 0（旧行为）。A/B：table-cell-width-0 20.43→3.71（br 间隙 0→19px）+
//! css-flexbox +2，全 dir NET 0，welcome 字节一致。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::{Document, NodeKind};
use zero_style_system::StyleSystem;

/// 递归找到首个 `<br>` 元素的 LayoutBox。
fn find_br_box<'a>(root: &'a LayoutBox, doc: &Document) -> Option<&'a LayoutBox> {
    if let Some(nid) = root.node_id
        && doc
            .get(nid)
            .is_some_and(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("br")))
    {
        return Some(root);
    }
    for c in &root.children {
        if let Some(b) = find_br_box(c, doc) {
            return Some(b);
        }
    }
    None
}

/// R1285：`<br>` 处于 block 兄弟之间时须有 line-height strut 高度（chromium ~19px），
/// 旧行为（无 min-height）渲染 0px 致后续块累积垂直错位（table-cell-width-0 20.43%）。
/// default-on：br.height ≈ line-height（16 × NORMAL_LINE_HEIGHT_RATIO ≈ 18.6）。
/// kill-switch `ZW_BR_LINEHEIGHT=0`：br.height ≈ 0（旧行为，证 load-bearing）。
#[test]
fn test_br_between_blocks_has_lineheight_strut() {
    let html = r#"<html><body style="margin:0"><div style="height:20px;width:50px"></div><br><div style="height:20px;width:50px"></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let br = find_br_box(&result.root, &doc).expect("should find <br> LayoutBox");
    // default-on：br 有 line-height strut（>10px）。kill-switch=0（旧行为）：br ≈ 0px。
    assert!(
        br.height > 10.0,
        "<br> between block siblings must have line-height strut height; got {}",
        br.height
    );
}

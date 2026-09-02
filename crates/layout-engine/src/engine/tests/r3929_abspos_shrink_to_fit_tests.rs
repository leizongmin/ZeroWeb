//! R3929（CSS2 §10.3.7/§10.6.4）：abspos 非 replaced 元素 shrink-to-fit 尺寸。
//!
//! taffy 对 width:auto + 全/半 auto 水平 inset 的 abspos 不做内容测量（layout dump
//! 实证 0 宽）——absolute-non-replaced-max-height-002（`&nbsp;` + Ahem 100px 应 100 宽）
//! /009（top:25 定 + bottom:auto）渲 0 宽方块。本 pass 在 intrinsic re-run 组内对
//! taffy 给 0 的 abspos 盒按内容 max-content（≤CB−已定 inset）回填宽度、按行高回填
//! 高度；taffy 已解出非 0 值（stretch、float 内容等）不覆写。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

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

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// absolute-non-replaced-max-height-002：全 auto 水平 inset 的 abspos（`&nbsp;` +
/// 100px 字号）宽应收缩适配到内容 100（taffy 旧给 0）。
#[test]
fn r3929_abspos_width_shrinks_to_content() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 200px; height: 200px;">
<div style="position: absolute; width: auto; font: 100px/1 Ahem, sans-serif; max-height: 50px;">&nbsp;</div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.last().expect("target div");
    let (w, _h) = find_box(&result.root, tid).expect("target box");
    assert!(
        (w - 100.0).abs() < 1.0,
        "R3929: abspos 全 auto inset 宽应收缩到内容 100，实际 {w}"
    );
}

/// absolute-non-replaced-max-height-009：height:auto + top 定 + bottom:auto 的 abspos
/// 高应收缩到行高（taffy 旧给 0）。
#[test]
fn r3929_abspos_height_shrinks_to_line_height() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 200px; height: 200px;">
<div style="position: absolute; top: 25px; bottom: auto; height: auto; font: 100px/1 Ahem, sans-serif;">&nbsp;</div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.last().expect("target div");
    let (_w, h) = find_box(&result.root, tid).expect("target box");
    assert!(
        (h - 100.0).abs() < 1.0,
        "R3929: abspos height:auto 垂直非双定高应收缩到行高 100，实际 {h}"
    );
}

/// taffy 已正确给出宽（水平双定 inset = stretch）时不覆写——margin-applies-to 族
/// 防回归锚（v1 版无此守卫致 23 案回归）。
#[test]
fn r3929_stretch_inset_not_overridden() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 200px; height: 200px;">
<div style="position: absolute; left: 0; right: 0; width: auto; height: 50px;"></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.last().expect("target div");
    let (w, _h) = find_box(&result.root, tid).expect("initial box");
    assert!(
        (w - 200.0).abs() < 1.0,
        "R3929: 双定 inset stretch 宽应保持 200 不被覆写，实际 {w}"
    );
}

/// 内含 float 后代的 abspos 跳过（float 子 max-width 语义 max-content 近似失准，
/// absolute-non-replaced-width-019/020 防回归锚）。
#[test]
fn r3929_float_descendant_skipped() {
    let html = r#"<html><body style="margin:0">
<div style="position: absolute; width: auto; font: 30px/4 Ahem, sans-serif;"><span style="float: left; max-width: 4em;">12345678</span></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.last().expect("target div");
    let (w, _h) = find_box(&result.root, tid).expect("target box");
    assert!(
        (w - 960.0).abs() > 4.0,
        "R3929: float 后代盒不应被 max-content（≈960）覆写——gate 应跳过，实际 {w}"
    );
}

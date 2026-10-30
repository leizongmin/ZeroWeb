//! R5348（CSS2 §10.3.9 shrink-to-fit）：inline-block width:auto 收缩 pass 的 grow 补腿。
//!
//! taffy 0.7 把 auto 宽 inline-block 拉伸到可用宽；旧「shrink-only」守卫在内容更宽时
//! no-op，父盒滞留拉伸值——definite/transferred 块级子溢出（intrinsic-percent-replaced-014：
//! inner inline-block + padding-top:100% + canvas h:100% 链 → 子 transferred 100，
//! 父应 100 溢出 25 容器）。grow 目标 = 流内子已布局 margin-box 最大者与可用宽取大
//!（可用宽保 min-content 下界：可换行文本不触发）。kill-switch `ZW_IB_SHRINK_GROW=0`
//!（腿由 env A/B 取证，不做进程内测试）。

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

/// 主驱（pe 探针 + 014 前置形态）：25px 容器内 auto 宽 inline-block 含 definite 100px
/// 块级子——shrink-to-fit = min(max(min-content, available), max-content) = 100，
/// 溢出容器；旧 shrink-only 滞留 taffy 拉伸值 25。
#[test]
fn r5348_inline_block_grows_to_definite_block_child() {
    let html = r#"<html><body style="margin:0">
<div style="width:25px;height:50px">
  <div style="display:inline-block;background:green"><div style="width:100px;height:10px"></div></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    // 文档序：divs[0] = 容器、divs[1] = inline-block、divs[2] = definite 子。
    let divs = doc.get_elements_by_tag_name("div");
    assert!(divs.len() >= 3, "容器/inline-block/子三层，实际 {}", divs.len());
    let ib_id = divs[1];
    let (w, _h) = find_box(&result.root, ib_id).expect("target box");
    assert!(
        (w - 100.0).abs() < 0.5,
        "R5348: inline-block 应 grow 到 definite 块级子 100（溢出 25 容器），实际 {w}"
    );
}

/// intrinsic-percent-replaced-014 形态：inline-block + padding-top:100%（对 25px CB
/// 解析 25）+ canvas h:100%（内容高 25）→ transferred 100×25——子已布局宽喂 grow。
#[test]
fn r5348_inline_block_grows_to_transferred_replaced_child() {
    let html = r#"<html><body style="margin:0">
<div style="width:25px;height:50px">
  <div style="display:inline-block;box-sizing:border-box;height:100%;padding-top:100%;background:green">
    <canvas width="40" height="10" style="display:block;height:100%"></canvas>
  </div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let inner_id = *divs.last().expect("target inner div");
    let (w, _h) = find_box(&result.root, inner_id).expect("target box");
    assert!(
        (w - 100.0).abs() < 0.5,
        "R5348: 014 形态 inline-block 应 grow 到 canvas transferred 100，实际 {w}"
    );
}

/// R4034b 同语义守卫：contain:size 的 inline-block 按 CIS/空内容定尺，已布局子宽
/// 不参与（contain-size-inline-block-001 回归哨）。
#[test]
fn r5348_contain_size_inline_block_does_not_grow() {
    let html = r#"<html><body style="margin:0">
<div style="width:25px;height:50px">
  <span style="display:inline-block;contain:size;background:green">
    <div style="width:100px;height:10px"></div>
  </span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let spans = doc.get_elements_by_tag_name("span");
    let sid = *spans.last().expect("target span");
    let (w, _h) = find_box(&result.root, sid).expect("target box");
    assert!(
        w < 50.0,
        "R5348: contain:size inline-block 不得按内容 grow（按空内容定尺），实际 {w}"
    );
}

/// 可换行文本下界守卫：窄容器内长文本 inline-block 靠换行收敛于可用宽，
/// 不得 grow 到未折行文本宽（laid-out 行宽 ≤ available 天然不触发）。
#[test]
fn r5348_wrapped_text_inline_block_stays_at_available() {
    let html = r#"<html><body style="margin:0">
<div style="width:25px;height:50px">
  <span style="display:inline-block;background:green">xxxxxxxxxx xxxxxxxxxx xxxxxxxxxx xxxxxxxxxx</span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let spans = doc.get_elements_by_tag_name("span");
    let sid = *spans.last().expect("target span");
    let (w, _h) = find_box(&result.root, sid).expect("target box");
    assert!(
        w < 200.0,
        "R5348: 可换行文本 inline-block 应停在可用宽附近（未折行宽 ~600 不得出现），实际 {w}"
    );
}

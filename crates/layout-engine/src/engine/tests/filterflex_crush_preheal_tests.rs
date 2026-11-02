//! t-fup-trending-filter-flex（github.com/trending 过滤条 flex 塌陷，details 38×8 vs
//! Chrome ~194×21）：块容器（div/details/flex item）含 inline-block 族子时容器高度
//! 塌陷的回归测试。根因三步链（`inline_finalization.rs` `remeasure_inline_only_containers`
//! pre-heal 注释 4307-4329 有全链与规范链接）：
//! ① `recompute_bfc_auto_height` 把纯 inline 内容的 auto 高 flow-root（含 inline-block，
//!    CSS2 §9.4.1）压成 frame 高（content_height=0）；
//! ② 本函数父臂 `ib_sizes` 自先递归后读到塌陷值；
//! ③ 收缩臂把容器砍到塌陷行高。
//! 修复 = 纯原子行流容器在 `ib_sizes` 前对塌陷原子子预跑子臂 remeasure（pre-heal）。
//!
//! 断言语义（不锁死绝对像素）：塌陷伪影 = 原子子 content_height≈0 + 容器高 = 塌陷行高；
//! 愈后语义 = 子 content_height ≥ 一整行（16px 基线）+ 容器高与子盒高一致。绝对值受
//! 已记录的 per-char intrinsic measure 偏差（~8% 窄 → fixture 文本折 2 行，span 38 而非
//! 单行 22）影响，不作为断言目标。kill-switch `ZW_IB_CRUSH_PREHEAL=0` 回退塌陷基线
//!（LazyLock 进程级缓存，与 float_positioning.rs 同款；env 切换不在本文件进程内覆盖，
//! 同 r5215 口径——OFF 轮以独立进程 `ZW_IB_CRUSH_PREHEAL=0` 运行取反证）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(1280.0, 800.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(1280.0, 800.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// 按 DOM node_id 找 LayoutBox。
fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

/// 取第一个匹配 tag 的盒与 node_id。
fn first_tag<'a>(doc: &zero_dom::Document, root: &'a LayoutBox, tag: &str) -> (NodeId, &'a LayoutBox) {
    let id = *doc
        .get_elements_by_tag_name(tag)
        .first()
        .unwrap_or_else(|| panic!("no <{tag}> in doc"));
    let b = find_box(root, id).unwrap_or_else(|| panic!("no LayoutBox for <{tag}>"));
    (id, b)
}

/// 愈后语义断言：原子子内容高度 ≥ 一整行（塌陷伪影为 0），容器高与子盒高一致
///（父臂收缩以真行盒高解，不再砍到 frame 高）。
fn assert_healed(container: &LayoutBox, atomic: &LayoutBox, ctx: &str) {
    assert!(
        atomic.content_height >= 16.0,
        "{ctx}: 原子子 content_height 应 ≥ 16（一整行；塌陷伪影为 0），实际 {}",
        atomic.content_height
    );
    assert!(
        (container.height - atomic.height).abs() < 1.0,
        "{ctx}: 容器高应与原子子盒高一致（{} vs {}；塌陷基线为 frame 高 6/0）",
        container.height,
        atomic.height
    );
}

/// 对照 A（div + inline-block span，padding 3px 8px）：修复前 div 22→6、span ch=0。
#[test]
fn crush_preheal_div_inline_block_child_not_crushed() {
    let html = r#"<html><body style="margin:0;font:14px sans-serif">
<div class="wrap"><span style="display:inline-block;padding:3px 8px">Spoken Language: Any</span></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let (_, div) = first_tag(&doc, &result.root, "div");
    let (_, span) = first_tag(&doc, &result.root, "span");
    assert_healed(div, span, "A div>span(padding)");
}

/// 对照 B（details + inline-block summary）：活体过滤条同构（details 为 plain block
/// 容器）。修复前 details 塌 6（活体 8）。
#[test]
fn crush_preheal_details_summary_live_shape_not_crushed() {
    let html = r#"<html><body style="margin:0;font:14px sans-serif">
<details><summary style="display:inline-block;padding:3px 8px">Spoken Language: Any</summary></details>
</body></html>"#;
    let (doc, result) = layout(html);
    let (_, details) = first_tag(&doc, &result.root, "details");
    let (_, summary) = first_tag(&doc, &result.root, "summary");
    assert_healed(details, summary, "B details>summary");
}

/// 邻近边界 C（inline-block 无 padding）：修复前塌 0 → 降级零宽 run 路径，同一修复面
/// 覆盖（证非只对带 padding 样例有效）。
#[test]
fn crush_preheal_bare_inline_block_boundary_not_crushed() {
    let html = r#"<html><body style="margin:0;font:14px sans-serif">
<div class="wrap"><span style="display:inline-block">Spoken Language: Any</span></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let (_, div) = first_tag(&doc, &result.root, "div");
    let (_, span) = first_tag(&doc, &result.root, "span");
    assert_healed(div, span, "C div>span(裸)");
}

/// 对照 D（div + 纯文本基线）：含文本容器不进 pre-heal 臂（R5164 守护域不扩），
/// 纯文本容器高度保持单行基线 16，不受修复扰动。
#[test]
fn crush_preheal_plain_text_container_control() {
    let html = r#"<html><body style="margin:0;font:14px sans-serif">
<div class="wrap">Spoken Language: Any</div>
</body></html>"#;
    let (_, result) = layout(html);
    let doc = zero_dom::parse_html(html);
    let (_, div) = first_tag(&doc, &result.root, "div");
    assert!(
        (div.height - 16.0).abs() < 1.0,
        "D: 纯文本容器基线高应 16（不受修复影响），实际 {}",
        div.height
    );
}

/// 活体组合形态：flex 行内 3 个 details（flex item = plain block 容器）各含
/// inline-block summary——github.com/trending 过滤条的最小复现结构。修复前整排
/// details 塌至 frame 高；修复后逐个愈合（同一 pre-heal 面，flex 容器臂不动）。
#[test]
fn crush_preheal_flex_row_details_live_composition() {
    let html = r#"<html><body style="margin:0;font:14px sans-serif">
<div style="display:flex;align-items:center">
<details><summary style="display:inline-block;padding:3px 8px">Language: Any</summary></details>
<details><summary style="display:inline-block;padding:3px 8px">Spoken Language: Any</summary></details>
<details><summary style="display:inline-block;padding:3px 8px">Repository: Any</summary></details>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let flex = find_box(
        &result.root,
        *doc.get_elements_by_tag_name("div").first().expect("flex div"),
    )
    .expect("flex box");
    let details_boxes: Vec<&LayoutBox> = flex
        .children
        .iter()
        .filter(|b| {
            b.node_id.is_some_and(|id| {
                doc.get(id)
                    .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "details"))
            })
        })
        .collect();
    assert_eq!(details_boxes.len(), 3, "活体组合：flex 行应有 3 个 details 子盒");
    let summaries = doc.get_elements_by_tag_name("summary");
    for (i, details) in details_boxes.iter().enumerate() {
        let sid = summaries[i];
        let summary = find_box(&result.root, sid).expect("summary box");
        assert_healed(details, summary, &format!("flex details[{i}]"));
    }
}

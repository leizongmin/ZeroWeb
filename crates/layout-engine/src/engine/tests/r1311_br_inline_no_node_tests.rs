//! R1311b：纯 inline 上下文中、且父块有后续 in-flow 兄弟的 `<br>` 不建 taffy 节点。
//!
//! `<p>line1<br>line2</p><div>...</div>`：旧实现把 `<br>` 建为 0 高 taffy Block leaf，
//! 使 p 成 `new_with_children([br])` 容器——taffy 按子（br=0）定 p 高、忽略 IFC measure
//! 回调，后续兄弟 div 因此与 p 重叠（position-absolute-percentage-inherit-001 谱系）。
//! R1311b 在 build_subtree 元素子循环跳过「纯 inline（无 block 同胞）+ 父块有后续 in-flow
//! 兄弟」的 br（精确触发条件），让 p 成 leaf 由 IFC 测高，兄弟正确定位。末子 br 父块
//! （如 welcome p.tagline）豁免——无后续兄弟可错位，跳过反引发容器高度连锁重排。
//! br-between-blocks（R1285 strut）仍建节点。
//!
//! load-bearing：default-on 时 p 后的 div.abs_y ≥ p 底（排在 p 之后）；
//! kill-switch `ZW_BR_INLINE_NO_NODE=0`（旧行为）时 div 与 p 重叠（div.y ≈ p.y）。
//! A/B：position-absolute-percentage-inherit-001 11.02→0.00% FLIP；visuren +1；
//! welcome 16.97% 字节一致（p.tagline 末子豁免）；css-flexbox/normal-flow/linebox NET 0。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::{Document, NodeKind};
use zero_style_system::StyleSystem;

/// 递归找到首个指定 tag 名的 LayoutBox。
fn find_box_by_tag<'a>(root: &'a LayoutBox, doc: &Document, tag: &str) -> Option<&'a LayoutBox> {
    if let Some(nid) = root.node_id
        && doc
            .get(nid)
            .is_some_and(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case(tag)))
    {
        return Some(root);
    }
    for c in &root.children {
        if let Some(b) = find_box_by_tag(c, doc, tag) {
            return Some(b);
        }
    }
    None
}

/// R1311b：`<p>a<br>b</p>` 后有 in-flow 块级兄弟时，该兄弟必须排在 p 之后（不与 p 重叠）。
/// 旧行为（br 建 0 高 taffy 节点）使 p 被测为 0 高，div 与 p 同位重叠。
/// default-on：div.y ≥ p.y + p.height（div 在 p 下方）。
/// kill-switch `ZW_BR_INLINE_NO_NODE=0`：div.y ≈ p.y（重叠，证 load-bearing）。
#[test]
fn test_block_after_p_with_br_is_not_overlapping() {
    // p 含 `<br>`（纯 inline 上下文）+ 一个后续 div（in-flow 兄弟）。
    let html = r#"<html><body style="margin:0"><p>line one<br>line two</p><div style="height:40px;width:100px"></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let p = find_box_by_tag(&result.root, &doc, "p").expect("should find <p> LayoutBox");
    // p 应含两行（br 换行），高度 > 一行（证 br 由 IFC 处理贡献行高）。
    assert!(
        p.height > 20.0,
        "<p>line one<br>line two</p> should span two lines; got p.height={}",
        p.height
    );

    let div = find_box_by_tag(&result.root, &doc, "div").expect("should find <div> LayoutBox");
    // div 必须排在 p 之后：div.y ≥ p.y + p.height（同为 body 子，y 相对同一原点）。
    // 旧行为（br 节点致 p 测 0 高 + margin-collapse-through）使 div.y ≈ p.y（重叠）。
    assert!(
        div.y >= p.y + p.height - 0.5,
        "<div> after <p>a<br>b</p> must be placed below p (div.y={} >= p.y+p.height={}); \
         overlap indicates br taffy node made p measure 0-height",
        div.y,
        p.y + p.height
    );
}

/// R1311b 守卫：br-between-blocks（有 block 同胞）仍建 taffy 节点（R1285 strut），不被跳过。
/// `<div/><br><div/>`：br 须占 line-height（R1285），不因 R1311b 消失。
#[test]
fn test_br_between_blocks_still_has_node() {
    let html = r#"<html><body style="margin:0"><div style="height:20px;width:50px"></div><br><div style="height:20px;width:50px"></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // body 的两个 div；br 占 line-height strut（R1285），第二个 div 应明显在第一个下方。
    let body = find_box_by_tag(&result.root, &doc, "body").expect("should find <body>");
    let divs: Vec<&LayoutBox> = body
        .children
        .iter()
        .filter(|c| {
            c.node_id
                .and_then(|nid| doc.get(nid))
                .is_some_and(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("div")))
        })
        .collect();
    assert_eq!(divs.len(), 2, "expected two <div> siblings around <br>");
    // br 占 line-height strut（R1285），第二个 div 在第一个下方 > 20px（div 高 20 + br strut）。
    assert!(
        divs[1].y > divs[0].y + 20.0,
        "<br> between blocks must still occupy line-height (R1285 strut preserved by R1311b); \
         second div.y={} should exceed first div.y+20={}",
        divs[1].y,
        divs[0].y + 20.0
    );
}

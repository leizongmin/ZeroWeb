//! R1743：body/html height propagation 修复测试（R1654 documented bug）。
//!
//! 根因：taffy 经 ctx_node 测含 `<br>`/多行 inline 内容的块子时欠计（br-split 子 taffy
//! 测 ~0），remeasure_inline_only_containers 之后子盒 height 已正确，但父盒仍持 taffy
//! 旧值 → 父 h=0 或仅计首子。修复 = shift_siblings_after_ifc_grow 末尾 max-bottom 回填
//!（margin-collapse-safe：Block-only gate + 仅 block-level 子 + 负 margin 守卫 + 仅增大）。
//! kill-switch = `ZW_IFC_PARENT_HEIGHT_BACKFILL=0`（default-on；env gate 验证见 master.md A/B）。
use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::StyleSystem;

fn compute(html: &str) -> (Document, LayoutBox, HashMap<NodeId, zero_style_system::ComputedStyle>) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let root = engine.compute(&doc, &styles).root;
    (doc, root, styles)
}

/// 找指定 tag 名（首个）的 (NodeId, &LayoutBox)。
fn find_tag<'a>(doc: &'a Document, b: &'a LayoutBox, tag: &str) -> Option<(NodeId, &'a LayoutBox)> {
    if let Some(nid) = b.node_id
        && let Some(node) = doc.get(nid)
        && let NodeKind::Element(e) = &node.kind
        && e.local_name().eq_ignore_ascii_case(tag)
    {
        return Some((nid, b));
    }
    for c in &b.children {
        if let Some(found) = find_tag(doc, c, tag) {
            return Some(found);
        }
    }
    None
}

/// 单 block 子含 `<br>` → body h 必须 ≈ div h（旧 bug：body h=0）。
#[test]
fn r1743_body_height_single_div_br() {
    let (doc, root, _) = compute(r#"<html><body><div>line one<br>line two<br>line three</div></body></html>"#);
    let (_, body) = find_tag(&doc, &root, "body").expect("body");
    let (_, div) = find_tag(&doc, &root, "div").expect("div");
    assert!(
        body.height >= div.height - 1.0,
        "body h={:.1} 应 ≈ div h={:.1}（br-split 子 remeasure 后父须回填）",
        body.height,
        div.height
    );
    assert!(body.height > 40.0, "body h={:.1} 应 > 40（3 行 br）", body.height);
}

/// 两 block 子含 `<br>` → body h 须含两子（旧 bug：body 仅计首子）。
#[test]
fn r1743_body_height_two_div_br() {
    let (doc, root, _) = compute(r#"<html><body><div>a<br>b</div><div>c<br>d</div></body></html>"#);
    let (_, body) = find_tag(&doc, &root, "body").expect("body");
    // 两 div 各 ~2 行（~37px），body 应 > 60（两子叠加），旧 bug body=37（仅首子）
    assert!(
        body.height > 60.0,
        "body h={:.1} 应 > 60（两 br-split 子叠加，旧 bug 仅计首子=37）",
        body.height
    );
}

/// address（UA 块级 + br）→ body 须含 address 高（fixture 27 谱系）。
#[test]
fn r1743_body_height_address_br() {
    let (doc, root, _) = compute(r#"<html><body><address>x<br>y<br>z</address></body></html>"#);
    let (_, body) = find_tag(&doc, &root, "body").expect("body");
    let (_, address) = find_tag(&doc, &root, "address").expect("address");
    assert!(
        body.height >= address.height - 1.0,
        "body h={:.1} 应 ≈ address h={:.1}",
        body.height,
        address.height
    );
    assert!(body.height > 40.0, "body h={:.1} 应 > 40", body.height);
}

/// 回归守卫：inline-only 容器（含 inline img）高度由 IFC 决定，max-bottom 回填不应误扩
///（仅 block-level 子计入）。test_inline_only_container_shrink_* 谱系。
#[test]
fn r1743_inline_only_container_not_overgrown() {
    // div 含两 inline img（96 / 144），高度应 = 144（tallest img），不应被 max-bottom 误扩。
    let (doc, root, _) = compute(
        r#"<html><body><div><img src="a.png" width="96" height="96"><img src="b.png" width="96" height="144"></div></body></html>"#,
    );
    let (_, div) = find_tag(&doc, &root, "div").expect("div");
    assert!(
        (div.height - 144.0).abs() < 6.0,
        "inline-only div h 应 ≈ 144（tallest img），不应被 max-bottom 回填误扩，实际 {:.1}",
        div.height
    );
}

/// R1745 回归守卫：flex/grid 容器自身由 taffy flex/grid 布局测高，**不**走 R1743 的
/// shift_siblings max-bottom 回填（shift_active 排除 flex/grid）。调查确认 taffy flex/grid
/// 正确测量 br-split 子内容高度 → 父容器高度正确传播。此测试钉死该负结果：若未来改动
/// 破坏 flex/grid 子高度测量，此守卫会捕获。flex/grid 容器 + br-split 子 → 容器 h 须 ≈ 子 h。
#[test]
fn r1745_flex_grid_parent_propagates_child_height() {
    for display in ["flex", "grid"] {
        let html = format!(r#"<html><body><div style="display:{display}"><div>a<br>b<br>c</div></div></body></html>"#);
        let (doc, root, _) = &compute(&html);
        // 外层 display:{display} div（首个非 body div）
        let outer = find_tag(doc, root, "div").expect("outer div").1;
        // 内层 br-split 子 div（最后一个 div）
        let mut inner = None;
        let mut stack = vec![root];
        while let Some(b) = stack.pop() {
            if let Some(nid) = b.node_id
                && let Some(node) = doc.get(nid)
                && let NodeKind::Element(e) = &node.kind
                && e.local_name().eq_ignore_ascii_case("div")
            {
                inner = Some(b);
            }
            stack.extend(b.children.iter());
        }
        let inner = inner.expect("inner div");
        assert!(
            outer.height >= inner.height - 1.0,
            "display:{display} 容器 h={:.1} 应 ≈ 子 div h={:.1}（taffy flex/grid 须正确测高）",
            outer.height,
            inner.height
        );
        assert!(
            outer.height > 40.0,
            "display:{display} 容器 h={:.1} 应 > 40（3 行 br 子）",
            outer.height
        );
    }
}

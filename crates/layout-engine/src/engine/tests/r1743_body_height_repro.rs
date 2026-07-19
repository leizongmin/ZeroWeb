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

/// R1746 负 margin 单子 + br 长高 → 父高度回填（refine R1743 negative-margin guard）。
/// 旧 R1743 guard 见负 margin 子即全跳过父 → 单子负 margin + br 长高致父 h=0。
/// refine：负 margin 父仅当 content_height ≈ 0（明显 taffy 误测）时放行长高。
#[test]
fn r1746_negative_margin_single_child_br_grows_parent() {
    for css in ["margin-bottom:-10px", "margin-top:-10px"] {
        let html = format!(r#"<html><body><div><p style="{css}">a<br>b<br>c<br>d<br>e</p></div></body></html>"#);
        let (doc, root, _) = compute(&html);
        let (_, div) = find_tag(&doc, &root, "div").expect("div");
        let (_, p) = find_tag(&doc, &root, "p").expect("p");
        assert!(
            div.height >= p.height - 1.0,
            "[{css}] div h={:.1} 应 ≈ p h={:.1}（负 margin 折叠到父外，不影响父内容高）",
            div.height,
            p.height
        );
        assert!(div.height > 40.0, "[{css}] div h={:.1} 应 > 40", div.height);
    }
}

/// R1746 守卫：负 margin 父 content_height > 0 时不应被误扩（避 R1163 回归）。
/// 一个已有合理高度的负 margin 父不应被 max-bottom 重算（防 margin-collapse 误扩）。
#[test]
fn r1746_negative_margin_nonzero_parent_not_overgrown() {
    // body 有自身 margin（content_height > 0 after taffy），含负 margin 子；
    // R1743 refine 仅 content_height<1.0 放行，故 body 已有高度时不触发 max-bottom 重算。
    // 这里用显式 height 的父确保 content_height 非 0，验证不被误扩。
    let html = r#"<html><body><div style="height:50px;overflow:hidden"><p style="margin-top:-200px">tall<br>content<br>that<br>overflows<br>upward<br>a<br>b<br>c<br>d<br>e</p></div></body></html>"#;
    let (doc, root, _) = compute(html);
    let (_, div) = find_tag(&doc, &root, "div").expect("div");
    // 显式 height:50px 的父不应被 max-bottom 拉到子全高（负 mt -200 拉子向上溢出父顶）
    // declared_height_auto=false → parent_backfill_active=false → 不触发；h 应保持 ~50。
    assert!(
        div.height < 80.0,
        "显式 height 父 h={:.1} 不应被负 mt 子拉到全高（应保持 ~50）",
        div.height
    );
}

/// R1747：inline-block br-split shrink-to-fit 宽 = 最宽行宽（非全文本累加）。
/// 旧 `text_content_max_width` 用 `doc.text_content`（扁平化 br）把多行测成单行
///（"short<br>much longer line<br>mid" → 201.6px 累加，应 max-line 131.2px）。
/// CSS css-sizing-3：forced break（br）产生独立 line，max-content 取最宽 line。
#[test]
fn r1747_inline_block_br_shrink_to_widest_line() {
    let longest = r#"<html><body><span style="display:inline-block">much longer line</span></body></html>"#;
    let br_split =
        r#"<html><body><span style="display:inline-block">short<br>much longer line<br>mid</span></body></html>"#;
    let (doc_l, root_l, _) = compute(longest);
    let (doc_b, root_b, _) = compute(br_split);
    let w_longest = find_tag(&doc_l, &root_l, "span").expect("span").1.width;
    let w_br = find_tag(&doc_b, &root_b, "span").expect("span").1.width;
    let delta = (w_br - w_longest).abs();
    assert!(
        delta < 3.0,
        "br-split shrink-to-fit width ({:.1}) 应 ≈ 最宽行宽 ({:.1})，差 {:.1}px（旧：201.6 累加全文本 bug）",
        w_br,
        w_longest,
        delta
    );
}

/// R1747：float br-split shrink-to-fit 宽 = 最宽行宽（< 200，旧 bug 会 > 200）。
#[test]
fn r1747_float_br_shrink_to_widest_line() {
    let html = r#"<html><body><div style="float:left">short<br>much longer line here<br>mid</div></body></html>"#;
    let (doc, root, _) = compute(html);
    let div = find_tag(&doc, &root, "div").expect("div").1;
    assert!(
        div.width < 200.0,
        "float br-split shrink-to-fit width {:.1} 应 < 200（最宽行宽，非全文本累加）",
        div.width
    );
    assert!(
        div.width > 100.0,
        "float br-split shrink-to-fit width {:.1} 应 > 100（最宽行 'much longer line here' 真实宽）",
        div.width
    );
}

/// R1747 守卫：无 br 的纯文本 max-content 不变（单段 = 全文本累加，行为同旧）。
#[test]
fn r1747_no_br_unchanged() {
    let html =
        r#"<html><body><span style="display:inline-block">a single long line of text content</span></body></html>"#;
    let (doc, root, _) = compute(html);
    let span = find_tag(&doc, &root, "span").expect("span").1;
    assert!(
        span.width > 200.0 && span.width < 320.0,
        "无 br 单行 span 宽 {:.1} 应 ≈ 270（行为不变）",
        span.width
    );
}

/// 找首个满足谓词的 LayoutBox（深度优先）。
fn find_first_box<'a>(b: &'a LayoutBox, pred: &dyn Fn(&LayoutBox) -> bool) -> Option<&'a LayoutBox> {
    if pred(b) {
        return Some(b);
    }
    for c in &b.children {
        if let Some(f) = find_first_box(c, pred) {
            return Some(f);
        }
    }
    None
}

/// R1748：R109 split inline 片段内含 `<br>` → 片段 shrink-to-fit 应取最宽行宽（非全文本累加）。
///
/// 对照：split inline 片段 `short<br>much longer line<br>mid` 的收缩宽应 ≈ 仅含
/// `much longer line` 的片段宽（最宽行），而非三段文本累加（旧 bug 过宽）。
#[test]
fn r1748_r109_split_br_fragment_widest_line() {
    // split inline：span 含 br 内容 + block 子 div（触发 R109 拆分）。首片段 = br 内容。
    let br_split = r#"<html><body><span style="display:inline;background:yellow">short<br>much longer line<br>mid<div>block</div></span></body></html>"#;
    // 对照：首片段仅含最宽行「much longer line」（无 br）。
    let widest_only = r#"<html><body><span style="display:inline;background:yellow">much longer line<div>block</div></span></body></html>"#;
    let (_doc_b, root_b, _) = compute(br_split);
    let (_doc_w, root_w, _) = compute(widest_only);
    // 首个匿名块片段盒 = 带 fragment_node_ids 的 LayoutBox。
    let frag_b = find_first_box(&root_b, &|b| b.fragment_node_ids.is_some()).expect("br-split 片段盒");
    let frag_w = find_first_box(&root_w, &|b| b.fragment_node_ids.is_some()).expect("widest-only 片段盒");
    let delta = (frag_b.width - frag_w.width).abs();
    assert!(
        delta < 3.0,
        "br-split 片段宽 ({:.1}) 应 ≈ 最宽行片段宽 ({:.1})（差 {:.1}；旧 bug 全文本累加会显著过宽）",
        frag_b.width,
        frag_w.width,
        delta
    );
    // sanity：片段宽应明显小于「三段文本累加」的近似（short+much longer line+mid ≫ 最宽行）。
    assert!(
        frag_b.width < 200.0,
        "br-split 片段宽 {:.1} 应 < 200（最宽行宽，非全文本累加）",
        frag_b.width
    );
}

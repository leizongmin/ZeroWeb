//! t-zchip 回归：flex 容器内 display:block 子项（GitHub trending 的 nav.subnav 形）flex
//! base size 探测量得 min-content（chips 竖排，活体 nav 塌 110px = 最宽 chip）而非
//! max-content（chips 横排）。css-flexbox-1 §9.2 step E：used flex basis 为 content 时
//! flex base size = item 以「content」当作 max-content 求得的 main size——taffy
//! flexbox.rs E 臂 available_space.main 已传 MaxContent，若仍量出 min-content 则为
//! block 容器/inline 内容测量层的宽度解析错臂。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::sync::Arc;
use zero_dom::Document;
use zero_style_system::StyleSystem;

/// 取第一个指定 id 的元素盒（深度优先）。
fn find_id<'a>(box_node: &'a LayoutBox, doc: &Document, id: &str) -> Option<&'a LayoutBox> {
    let hit = box_node
        .node_id
        .and_then(|nid| doc.get(nid))
        .and_then(|n| match &n.kind {
            zero_dom::NodeKind::Element(e) => (e.get_attribute("id").as_deref() == Some(id)).then_some(()),
            _ => None,
        });
    if hit.is_some() {
        return Some(box_node);
    }
    box_node.children.iter().find_map(|c| find_id(c, doc, id))
}

fn compute(html: &str) -> (Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let root = Arc::try_unwrap(engine.compute(&doc, &styles).root).unwrap_or_else(|arc| (*arc).clone());
    (doc, root)
}

/// 复现形（github.com/trending Box-header 简化）：flex header > block nav > 2 个
/// inline-block chip。nav 是 flex item，flex-basis auto → content：base size 应为
/// max-content（两 chip 横排宽度和），而非 min-content（最宽 chip 单宽，chips 竖排）。
#[test]
fn t_zchip_flex_block_child_base_size_is_max_content() {
    let html = r#"<html><body><div id="hdr" style="display:flex;padding:16px;">
  <nav id="nav" style="display:block;"><a id="a1" style="display:inline-block;padding:5px 16px;">Repositories</a><a id="a2" style="display:inline-block;padding:5px 16px;">Developers</a></nav>
  <div id="right">Right</div>
</div></body></html>"#;
    let (doc, root) = compute(html);
    let nav = find_id(&root, &doc, "nav").expect("nav box");
    let a1 = find_id(&root, &doc, "a1").expect("a1 box");
    let a2 = find_id(&root, &doc, "a2").expect("a2 box");
    let chips_sum = a1.width + a2.width;
    assert!(
        nav.width >= chips_sum - 1.0,
        "flex 容器内 block 子项 nav w={:.1} 应为 max-content（chips 横排 {:.1}+{:.1}={:.1}），\
         明显小于即 min-content 塌缩（chips 竖排，css-flexbox-1 §9.2.E）",
        nav.width,
        a1.width,
        a2.width,
        chips_sum
    );
    // 邻近边界：chip 同行（竖排修复判据）——a2 顶部与 a1 平齐。
    assert!(
        (a1.y - a2.y).abs() < 1.0,
        "两 chip 应同行（a1 top {:.1} vs a2 top {:.1}），竖排即 min-content 塌缩",
        a1.y,
        a2.y
    );
}

/// 邻近边界（既有行为 guard）：同结构但 header 为 display:block——nav 是流内 block，
/// auto 宽 fill 容器（fixture1 C 形，活体 block 场景不回退）。
#[test]
fn t_zchip_block_header_nav_fills_container() {
    let html = r#"<html><body><div id="hdr" style="display:block;padding:16px;">
  <nav id="nav" style="display:block;"><a id="a1" style="display:inline-block;padding:5px 16px;">Repositories</a><a id="a2" style="display:inline-block;padding:5px 16px;">Developers</a></nav>
  <div id="right">Right</div>
</div></body></html>"#;
    let (doc, root) = compute(html);
    let nav = find_id(&root, &doc, "nav").expect("nav box");
    let hdr = find_id(&root, &doc, "hdr").expect("hdr box");
    // header 内容盒 800 - 32(pad) = 768；nav fill 768。
    assert!(
        (nav.width - (hdr.width - 32.0)).abs() < 2.0,
        "block 流内 nav w={:.1} 应 fill 容器内容宽 {:.1}",
        nav.width,
        hdr.width - 32.0
    );
    let a1 = find_id(&root, &doc, "a1").expect("a1 box");
    let a2 = find_id(&root, &doc, "a2").expect("a2 box");
    assert!((a1.y - a2.y).abs() < 1.0, "block 场景两 chip 同行不回退");
}

/// 变体：flex 容器内 block 子项含文本+inline-block 混合（不只有 atomic inline）——
/// max-content 同样适用（inline 文本不强制断行）。
#[test]
fn t_zchip_flex_block_child_mixed_inline_max_content() {
    let html = r#"<html><body><div id="hdr" style="display:flex;padding:8px;">
  <nav id="nav" style="display:block;"><span id="txt">Filter:</span><a id="a1" style="display:inline-block;padding:2px 8px;">One</a></nav>
</div></body></html>"#;
    let (doc, root) = compute(html);
    let nav = find_id(&root, &doc, "nav").expect("nav box");
    let txt = find_id(&root, &doc, "txt").expect("txt box");
    let a1 = find_id(&root, &doc, "a1").expect("a1 box");
    assert!(
        nav.width > txt.width + a1.width,
        "混合 inline 的 block flex item w={:.1} 应超 inline 内容和 {:.1}+{:.1}（max-content 单行），\
         小于即 min-content 臂误用",
        nav.width,
        txt.width,
        a1.width
    );
    let _ = find_id(&root, &doc, "a2");
}

//! R5235 D1：rel span 包裹原子 img（R2156 提升）后文本 run 的 img advance 丢失复现。
//!
//! `<div style="font-family:Ahem"><span style="position:relative">PRE<img width=32 height=32>POST</span>Inner</div>`：
//! span 含原子后代 → R2156 提升路径（img 升容器 taffy 子、span 无自身盒、文本留 IFC）。
//! R5234 ZW_DEBUG_IFC 证 items 流完整（div 内 text=3 + inline_block=1）——collect 无恙，
//! 丢面在行装配输出或后续消费。本组测试跑完整布局，断言 img 后文本 run x 含 img advance：
//! POST.x ≈ PRE.x + PRE.width + 32；Inner.x ≈ POST.x + POST.width（CSS2 §10.8.1 行内
//! 原子盒按 margin-box 参与行内布局推进）。
//!
//! 对照臂：无 position 的普通 span 同构——隔离「position:relative 特异」与「R2156
//! 提升路径通用」两种候选面。
//!
//! img 用 width/height 属性给尺寸（collect_items img 分支 HTML attr 优先级），单测无需
//! image cache 即走已加载 32×32 等价路径（R5234 探针 a 的 rel span 叠盖形态）。

use crate::engine::LayoutEngine;
use crate::types::{InlineLayoutFragment, LayoutBox};
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

/// 展开 IFC 全部非空文本 fragment（跨行扁平化，保持行内顺序）。
fn text_fragments(layout: &[crate::types::InlineLayoutLine]) -> Vec<&InlineLayoutFragment> {
    layout
        .iter()
        .flat_map(|line| line.fragments.iter())
        .filter(|f| !f.text.is_empty())
        .collect()
}

/// 断言 `<span…>PRE<img 32×32>POST</span>Inner` 的文本 run x 推进含 img advance。
///
/// div 用 pure-Ahem 字体（inline_finalization store-gate：仅纯 Ahem 容器存
/// `inline_layout`，R84/R355），Ahem 字形宽 = font-size，文本 x 可精确预期
/// （16px 字号：PRE=48 / POST=64 / Inner=80）。
fn assert_advance_chain(root: &LayoutBox, doc: &Document, label: &str) {
    let div = find_box_by_tag(root, doc, "div").expect("should find <div> LayoutBox");
    let layout = div
        .inline_layout
        .as_ref()
        .unwrap_or_else(|| panic!("{label}: <div> should store IFC (inline_layout)"));
    let frags = text_fragments(layout);

    let pre = frags.iter().find(|f| f.text.trim() == "PRE").unwrap_or_else(|| {
        panic!(
            "{label}: PRE fragment not found; texts={:?}",
            frags.iter().map(|f| &f.text).collect::<Vec<_>>()
        )
    });
    let post = frags
        .iter()
        .find(|f| f.text.trim() == "POST")
        .unwrap_or_else(|| panic!("{label}: POST fragment not found"));
    // POST 必须在 PRE 末尾 + img 32px advance 之后。丢失 advance 时
    // POST.x ≈ PRE.x + PRE.width（R5234 探针 a 叠盖形态：POST 左移 32px）。
    let expected_post_x = pre.x + pre.width + 32.0;
    assert!(
        (post.x - expected_post_x).abs() <= 1.0,
        "{label}: POST.x={:.2} must equal PRE.x+PRE.width+32={:.2} (img advance lost?); \
         pre.x={:.2} pre.width={:.2}",
        post.x,
        expected_post_x,
        pre.x,
        pre.width
    );
    // Inner 紧跟 POST（span 后无空白，无额外间距）；裸 img 臂无 Inner 文本，跳过。
    if let Some(inner) = frags.iter().find(|f| f.text.trim() == "Inner") {
        let expected_inner_x = post.x + post.width;
        assert!(
            (inner.x - expected_inner_x).abs() <= 1.0,
            "{label}: Inner.x={:.2} must equal POST.x+POST.width={:.2}; \
             post.x={:.2} post.width={:.2}",
            inner.x,
            expected_inner_x,
            post.x,
            post.width
        );
    }
}

/// 跑完整布局返回根盒。
fn layout_html(html: &str) -> (Document, std::sync::Arc<LayoutBox>) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result.root)
}

/// R5235 D1 主臂：position:relative 的 span 包裹 img——img 后文本 run 须含 32px advance。
#[test]
fn test_rel_span_img_advance_carried_to_post_text() {
    let html = r#"<html><body style="margin:0"><div style="font-family:Ahem"><span style="position:relative">PRE<img width="32" height="32">POST</span>Inner</div></body></html>"#;
    let (doc, root) = layout_html(html);
    assert_advance_chain(&root, &doc, "rel-span");
}

/// 对照臂：普通 span（无 position）同构——若同失，缺陷在 R2156 提升路径通用段
/// 而非 rel 特异；若通过，则面在 rel 偏移叠加段。
#[test]
fn test_plain_span_img_advance_carried_to_post_text() {
    let html = r#"<html><body style="margin:0"><div style="font-family:Ahem"><span>PRE<img width="32" height="32">POST</span>Inner</div></body></html>"#;
    let (doc, root) = layout_html(html);
    assert_advance_chain(&root, &doc, "plain-span");
}

/// 裸 img 对照（R5234 探针 a 已证正确，此处固化基线防回归）。
#[test]
fn test_bare_img_advance_carried_to_post_text() {
    let html = r#"<html><body style="margin:0"><div style="font-family:Ahem">PRE<img width="32" height="32">POST</div></body></html>"#;
    let (doc, root) = layout_html(html);
    assert_advance_chain(&root, &doc, "bare-img");
}

//! R3893：block 容器混合内容拆分宿主旗标（§9.2.1.1 ②）。
//!
//! 宿主容器的 `is_r109_block_mixed` 须为 true（其直接文本由 Inline 匿名块片段渲染，
//! paint_text 自身路径须跳过）；inline 拆分宿主走既有 `is_r109_split`；纯 block /
//! 纯 inline 容器两旗标均 false。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn find_by_id<'a>(b: &'a LayoutBox, doc: &zero_dom::Document, id: &str) -> Option<&'a LayoutBox> {
    let hit = b.node_id.is_some_and(|nid| {
        doc.get(nid).is_some_and(
            |n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.get_attribute("id").as_deref() == Some(id)),
        )
    });
    if hit {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_by_id(c, doc, id))
}

fn layout(html: &str) -> (zero_dom::Document, crate::types::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute(&doc, &styles);
    (doc, r)
}

/// 混合内容 block 容器（text + div）：宿主 is_r109_block_mixed = true，
/// 且生成 Inline 匿名块片段（fragment_node_ids 非 None）。
#[test]
fn r3893_block_mixed_host_flagged() {
    let html = r#"<html><body style="margin:0"><div id="host">intro <div id="blk">block</div></div></body></html>"#;
    let (doc, r) = layout(html);
    let host = find_by_id(&r.root, &doc, "host").expect("host box");
    assert!(host.is_r109_block_mixed, "mixed-content host must be flagged");
    assert!(!host.is_r109_split, "block-mixed host is not an inline split");
    let frag_count = host.children.iter().filter(|c| c.fragment_node_ids.is_some()).count();
    assert_eq!(frag_count, 1, "one Inline anon-block fragment expected");
}

/// inline 拆分宿主（inline 元素含 block 子）：is_r109_split = true（既有语义），
/// is_r109_block_mixed = false。
#[test]
fn r3893_inline_split_host_unchanged() {
    let html = r#"<html><body style="margin:0"><span id="host">a<div>block</div></span></body></html>"#;
    let (doc, r) = layout(html);
    let host = find_by_id(&r.root, &doc, "host").expect("host box");
    assert!(host.is_r109_split, "inline split host keeps is_r109_split");
    assert!(!host.is_r109_block_mixed, "inline split host is not block-mixed");
}

/// 纯文本 block 容器：两旗标均 false（paint_text 自身路径照常）。
#[test]
fn r3893_pure_text_block_not_flagged() {
    let html = r#"<html><body style="margin:0"><div id="host">just text</div></body></html>"#;
    let (doc, r) = layout(html);
    let host = find_by_id(&r.root, &doc, "host").expect("host box");
    assert!(!host.is_r109_split && !host.is_r109_block_mixed);
}

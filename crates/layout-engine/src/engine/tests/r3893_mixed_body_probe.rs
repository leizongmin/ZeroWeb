//! R3893 diagnostic：ref 页（body 混排 text+div+div+text）的 box 树 + text 归属 dump。
//!
//! quotes-first-letter-002 的 ref 页：`“` 文本 + div + div + `”` 文本 直接做 body 子。
//! §9.2.1.1：inline 内容（文本）应被匿名块包裹，与 block div 垂直堆叠。实测 paint 把
//! div 文本拼进 body 顶部一行（text concatenation）。本 dump 打印 box 树 + text_node
//! 映射，定位串联源。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn tag_of(b: &LayoutBox, doc: &zero_dom::Document) -> String {
    b.node_id
        .and_then(|id| {
            doc.get(id).map(|n| match &n.kind {
                zero_dom::NodeKind::Element(e) => {
                    let cls = e.get_attribute("class").map(|v| format!(".{}", v)).unwrap_or_default();
                    format!("<{}{}>", e.local_name(), cls)
                }
                zero_dom::NodeKind::Text(t) => format!("#text({:?})", t.content.chars().take(16).collect::<String>()),
                _ => "?".to_string(),
            })
        })
        .unwrap_or_else(|| "<anon>".to_string())
}

fn dump(b: &LayoutBox, doc: &zero_dom::Document, depth: usize) {
    let indent = "  ".repeat(depth);
    eprintln!(
        "{}{} disp_block={} anon_text={} w={} h={} x={} y={} node_id={:?} text_nodes={}",
        indent,
        tag_of(b, doc),
        b.is_block_level,
        b.is_anonymous_text_item,
        b.width,
        b.height,
        b.x,
        b.y,
        b.node_id,
        b.text_node_line_heights.len(),
    );
    for c in &b.children {
        dump(c, doc, depth + 1);
    }
}

#[test]
fn r3893_dump_ref_page_mixed_body() {
    let html = r#"<html><head><style>
  .quote { color: green; }
</style></head><body>
“
<div><span class="quote">‘</span>Should not crash or assert and all six quotes should be displayed.’</div>
<div><span class="quote">‘</span>Should not crash or assert and all six quotes should be displayed.’</div>
”</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute(&doc, &styles);
    eprintln!("=== R3893 ref-page box tree (body mixed text+div+div+text) ===");
    dump(&r.root, &doc, 0);
    let _ = styles;
}

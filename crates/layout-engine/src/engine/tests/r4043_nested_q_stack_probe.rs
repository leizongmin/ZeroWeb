//! R4043 诊断：`<p><q>one <q>two</q> three</q></p>`（块容器独子 inline 含嵌套 inline）
//! 的 box 树 dump——观测 q/nested-q 盒的 is_block_level / 几何，定位三行堆叠来源。
//!
//! 实测（product-smoke PNG）：该形态渲染为 3 行堆叠（"one/two/three" 各占一行）；
//! 同构 `<p>x <span>one <span>two</span> three</span></p>`（容器有直接文本）单行正常。
//! quotes-005/013/015/019（`<p dir=…><q>…<q>…</q>…</q></p>`）为此形态，diff 1.05-1.34%。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn tag_of(b: &LayoutBox, doc: &zero_dom::Document) -> String {
    b.node_id
        .and_then(|id| {
            doc.get(id).map(|n| match &n.kind {
                zero_dom::NodeKind::Element(e) => format!("<{}>", e.local_name()),
                zero_dom::NodeKind::Text(t) => format!("#text({:?})", t.content.chars().take(12).collect::<String>()),
                _ => "?".to_string(),
            })
        })
        .unwrap_or_else(|| "<anon>".to_string())
}

fn dump(b: &LayoutBox, doc: &zero_dom::Document, depth: usize) {
    let indent = "  ".repeat(depth);
    eprintln!(
        "{}{} blk={} anon_text={} w={} h={} x={} y={} id={:?} il={} frag={:?}",
        indent,
        tag_of(b, doc),
        b.is_block_level,
        b.is_anonymous_text_item,
        b.width,
        b.height,
        b.x,
        b.y,
        b.node_id,
        b.inline_layout.is_some(),
        b.fragment_node_ids.as_ref().map(|v| v.len()),
    );
    for c in &b.children {
        dump(c, doc, depth + 1);
    }
}

#[test]
fn r4043_dump_nested_q_sole_child_stack() {
    let html = r#"<html><head><style>
  body { font: 32px serif; }
</style></head><body>
<p><q>one <q>two</q> three</q></p>
<p>x <span>one <span>two</span> three</span></p>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute(&doc, &styles);
    eprintln!("=== R4043 box tree: p>q(nested) vs p>x span(nested) ===");
    dump(&r.root, &doc, 0);
}

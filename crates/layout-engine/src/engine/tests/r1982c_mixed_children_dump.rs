//! R1982c diagnostic：block 容器含 mixed inline+block 子时，ZW 生成的 box 树结构。
//!
//! anonymous-boxes-001a（§9.2.1.1）：一个 div height:200px 内有 inline content（text +
//! inline-block span）加一个 block 子 p。规范要求 inline content 被 anonymous block box 包裹。
//! R1982b 发现 span #t 在 box 树找不到。本 dump 打印 anc 子树定位 span 去向（anonymous-block
//! 未生成 / span 在但 node_id=None / 其他），为 R109 spec-rfc 提供精确 box 树结构数据。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn tag_of(b: &LayoutBox, doc: &zero_dom::Document) -> String {
    b.node_id
        .and_then(|id| {
            doc.get(id).map(|n| match &n.kind {
                zero_dom::NodeKind::Element(e) => {
                    let id_attr = e.get_attribute("id").map(|v| format!("#{}", v)).unwrap_or_default();
                    format!("<{}{}>", e.local_name(), id_attr)
                }
                zero_dom::NodeKind::Text(t) => format!("#text({})", t.content.chars().take(12).collect::<String>()),
                _ => "?".to_string(),
            })
        })
        .unwrap_or_else(|| "<anon>".to_string())
}

fn dump(b: &LayoutBox, doc: &zero_dom::Document, depth: usize) {
    let indent = "  ".repeat(depth);
    eprintln!(
        "{}{} disp=? w={} h={} x={} y={} node_id={:?}",
        indent,
        tag_of(b, doc),
        b.width,
        b.height,
        b.x,
        b.y,
        b.node_id
    );
    for c in &b.children {
        dump(c, doc, depth + 1);
    }
}

#[test]
fn r1982c_dump_mixed_children_subtree() {
    let html = r#"<html><body style="margin:0">
<div id="anc" style="height:200px; width:400px; font:40px Ahem">
  text <span id="t" style="display:inline-block; height:50%; width:50px; background:green"></span> more
  <p>block</p>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute(&doc, &styles);
    eprintln!("=== R1982c full box tree (mixed inline+block children) ===");
    dump(&r.root, &doc, 0);
    let _ = styles;
}

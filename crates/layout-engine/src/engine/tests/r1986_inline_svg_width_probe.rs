//! R1986 probe：inline `<svg:svg>` 元素的 width（inline-replaced-width cluster 定性）。
//! normal-flow top fail 簇 inline-replaced-width-*（8+ 案 10-22%）全用 inline `<svg:svg>` 元素
//! （height=300 + 内部 rect 600×300）。若 ZW 给 inline SVG 非 replaced 处理（无 intrinsic size
//! → width=0/default 而非 §10.3.2 的 600），则该簇失败 = inline SVG 渲染 out of scope（goal line 118），
//! 非通用 replaced-width bug。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn find_svg<'a>(r: &'a LayoutBox, d: &zero_dom::Document) -> Option<&'a LayoutBox> {
    let hit = r.node_id.is_some_and(|nid| {
        d.get(nid).is_some_and(
            |n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("svg")),
        )
    });
    if hit {
        return Some(r);
    }
    for c in &r.children {
        if let Some(b) = find_svg(c, d) {
            return Some(b);
        }
    }
    None
}

#[test]
fn r1986_probe_inline_svg_width() {
    let html = r#"<html><body style="margin:0">
<div style="height:300px; width:600px;">
  <svg id="s" version="1.1" height="300" style="display:inline-block; vertical-align:top;">
    <rect x="0" y="0" width="600" height="300" fill="red"></rect>
  </svg>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute(&doc, &styles);
    match find_svg(&r.root, &doc) {
        Some(s) => eprintln!(
            "R1986 inline <svg> height=300 + rect 600x300 => svg width={} height={} (§10.3.2 expect width=600 if treated as replaced; 0/default = not replaced, inline SVG out of scope)",
            s.width, s.height
        ),
        None => eprintln!("R1986 inline <svg> NOT FOUND in box tree (inline SVG not in LayoutBox tree → out of scope)"),
    }
    let _ = styles;
}

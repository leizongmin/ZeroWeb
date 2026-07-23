//! R1982 characterization：R109 §9.2.1.1 anonymous-block 生成对 inline 子 box identity 的影响。
//!
//! anonymous-boxes-001a（visuren，6.14% diff）测：div{height:200px} 内含 inline content + block `<p>`，
//! 触发 §9.2.1.1 anonymous block box 生成。anonymous block 内的 inline 子（img/inline-block height:50%）
//! 的百分比高度应解析对 div（非匿名祖先 CB=200px）→ 50% = 100px。
//!
//! 实测结论（R1982）：INLINE-ONLY（无 block 子）inline-block span height=100 正确；WITH-BLOCK-CHILD
//! （有 `<p>` block 子触发 anonymous-block 生成）span #t 在 box 树中找不到——inline 子 identity 经
//! anonymous-block 生成丢失。即 %height CB 解析本身正确（INLINE-ONLY=100），gap 在 anonymous-block
//! 生成丢失 inline 子 box identity，这是 R109 block-in-inline 结构性缺口（master.md：最大剩余
//! structural lever，deadlock 史须 spec-rfc，多 session）。本 probe 作 durable 定位数据 + 未来 R109 fix
//! 的 success signal（fix 后 WITH-BLOCK-CHILD 应找到 span 且 height=100）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn find_id<'a>(r: &'a LayoutBox, d: &zero_dom::Document, id: &str) -> Option<&'a LayoutBox> {
    let hit = r.node_id.is_some_and(|nid| {
        d.get(nid).is_some_and(|n| match &n.kind {
            zero_dom::NodeKind::Element(e) => e.get_attribute("id").is_some_and(|v| v == id),
            _ => false,
        })
    });
    if hit {
        return Some(r);
    }
    for c in &r.children {
        if let Some(b) = find_id(c, d, id) {
            return Some(b);
        }
    }
    None
}

#[test]
fn r1982_probe_anon_block_child_pct_height_cb() {
    // 带块级子（触发 anonymous-block 生成）。
    let html_with_block = r#"<html><body style="margin:0">
<div id="anc" style="height:200px; width:400px; font:40px Ahem">
  text <span id="t" style="display:inline-block; height:50%; width:50px; background:green"></span> more
  <p>block</p>
</div>
</body></html>"#;
    // 不带块级子（纯 inline，无 anonymous-block）。
    let html_inline_only = r#"<html><body style="margin:0">
<div id="anc" style="height:200px; width:400px; font:40px Ahem">
  text <span id="t" style="display:inline-block; height:50%; width:50px; background:green"></span> more
</div>
</body></html>"#;
    for (label, html) in [("WITH-BLOCK-CHILD", html_with_block), ("INLINE-ONLY", html_inline_only)] {
        let doc = zero_dom::parse_html(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut eng = LayoutEngine::new(800.0, 600.0);
        let r = eng.compute(&doc, &styles);
        match find_id(&r.root, &doc, "t") {
            Some(t) => eprintln!(
                "R1982-ANON [{}] inline-block height:50% => span height={} (§9.2.1.1 expect 100)",
                label, t.height
            ),
            None => eprintln!(
                "R1982-ANON [{}] span #t NOT FOUND in box tree (identity lost via anonymous-block gen?)",
                label
            ),
        }
    }
    // 诊断不强制（characterization）；记录两路径当前行为作 durable 数据。
}

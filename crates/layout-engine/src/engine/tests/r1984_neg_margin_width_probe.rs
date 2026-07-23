//! R1984 characterization：negative margin + width:auto 的 block outer width（replaced-intrinsic-001）。
//!
//! normal-flow subdir 狩猎（reftest-oracle 621/746=83.2%）：top fail 簇 inline-replaced-width-* 用
//! inline `<svg:svg>` 元素（goal line 118 inline SVG 渲染 out of scope）+ min-height-106 scrollbar
//! 依赖（out of scope）；余 in-scope 几何案 replaced-intrinsic-001（11.73%）= negative margin + width:auto。
//! 本 probe 实测该计算：html/outer CB=150px，div{border:2px; margin:-2px} → block outer width 应 =
//! CB - (margin-l + margin-r) = 150 - (-4) = 154px（负 margin 使 block 比 CB 更宽，CSS §10.3.3）。
//! 实测 ZW = 154 ✓ 正确 → replaced-intrinsic-001 的 11.73% diff 非核心 width 计算 bug（是次级效应），
//! 即 normal-flow 无 clean in-scope lever（再证 cross-dir exhaustive framing）。本测试作 durable
//! regression guard 守 negative-margin block width 不变量。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn find_div<'a>(r: &'a LayoutBox, d: &zero_dom::Document) -> Option<&'a LayoutBox> {
    let hit = r.node_id.is_some_and(|nid| {
        d.get(nid).is_some_and(
            |n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("div")),
        )
    });
    if hit {
        return Some(r);
    }
    for c in &r.children {
        if let Some(b) = find_div(c, d) {
            return Some(b);
        }
    }
    None
}

#[test]
fn r1984_neg_margin_block_width_154() {
    let html = r#"<html><body style="margin:0">
<div id="outer" style="width:150px;">
  <div style="border:2px solid lime; height:150px; margin:-2px; background:green;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute(&doc, &styles);
    let outer = find_div(&r.root, &doc).expect("outer div");
    let inner = outer.children.iter().find(|c| c.node_id.is_some()).expect("inner div");
    // CB=150, margin-l=margin-r=-2 → outer width = 150 - (-2 + -2) = 154（负 margin 撑宽）。
    assert!(
        (inner.width - 154.0).abs() < 1.0,
        "neg-margin block width={} (expect 154 = 150 CB + 4 neg-margin)",
        inner.width
    );
    assert!((inner.border_left - 2.0).abs() < 0.5 && (inner.border_right - 2.0).abs() < 0.5);
}

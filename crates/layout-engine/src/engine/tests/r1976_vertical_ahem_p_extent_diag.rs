//! R1976 definitive TBD2 probe：vertical Ahem `<p>` 的 inline_layout extent（生产 container_width 下）。
//!
//! 承接 R1975（break_into_lines 层 preliminary：不 hang 但不触达 vertical max_depth 逻辑）。
//! 本测试走**生产路径** LayoutEngine.compute()：vertical Ahem `<p>` 的 IFC 经 R1099 α-1 设
//! container_width = content_height。若 <p> height=inf（axis-swap bug，R1971），则 container_width=inf
//! → dump IFC 实际产出的 fragment extent（是否 finite / 正确）= definitive TBD2 答案。
//!
//! 6 个 Ahem char @ 20px → vertical 逐字竖排，inline extent（物理 height）应 ≈ 120px。
//! - 若 IFC 产 finite extent ≈120（fragment 覆盖 y=0..120）→ two-phase measure 可行（extent 可计算）。
//! - 若 IFC 产 inf / 0 / 错 → TBD2 否定，two-phase 不可行。
//!
//! 诊断（eprintln）+ durable 数据。Ahem 确 inline_layout 被 stored（compute_final:941 is_pure_ahem）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::Document;
use zero_style_system::StyleSystem;

/// 深度优先找到第一个 local_name == "p" 的元素 LayoutBox。
fn find_p<'a>(root: &'a LayoutBox, doc: &Document) -> Option<&'a LayoutBox> {
    let is_p = root.node_id.is_some_and(|id| {
        doc.get(id).is_some_and(
            |n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("p")),
        )
    });
    if is_p {
        return Some(root);
    }
    for child in &root.children {
        if let Some(b) = find_p(child, doc) {
            return Some(b);
        }
    }
    None
}

#[test]
fn r1976_diag_vertical_ahem_p_inline_extent() {
    let html = r#"<html><body style="margin:0">
<p style="writing-mode:vertical-rl; font:20px/1 Ahem;">AAAAAA</p>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let p = find_p(&result.root, &doc).expect("should find <p>");
    eprintln!(
        "R1976 vertical Ahem <p>: width={} height={} (is_infinite={}) wm={:?}",
        p.width,
        p.height,
        p.height.is_infinite(),
        p.writing_mode
    );
    match &p.inline_layout {
        None => eprintln!("R1976   inline_layout = None (NOT stored — compute_final gate skipped it)"),
        Some(lines) => {
            eprintln!("R1976   inline_layout: {} lines", lines.len());
            let mut max_bottom = 0.0_f32;
            for (i, line) in lines.iter().enumerate() {
                eprintln!(
                    "R1976     line[{}]: y={} height={} fragments={}",
                    i,
                    line.y,
                    line.height,
                    line.fragments.len()
                );
                for (j, frag) in line.fragments.iter().enumerate() {
                    eprintln!(
                        "R1976       frag[{}]: x={} y={} width={} height={}",
                        j, frag.x, frag.y, frag.width, frag.height
                    );
                    // vertical inline extent = fragment 沿 y 方向跨度。
                    let bottom = frag.y.max(0.0) + frag.height.max(0.0);
                    if bottom.is_finite() && bottom > max_bottom {
                        max_bottom = bottom;
                    }
                }
            }
            eprintln!(
                "R1976   === IFC inline extent (max frag y+height) = {} (expected ≈120 for 6 Ahem chars @20px) ===",
                max_bottom
            );
        }
    }
    // 诊断不强制（仅报告）；保持 test PASS 以作 durable 数据载体。
    let _ = (p.height,);
}

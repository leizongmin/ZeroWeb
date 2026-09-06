//! R4074 回归：aspect-ratio 传递高不被 IFC 收缩臂覆盖（css-sizing-4 §4.2）。
//!
//! aspect-ratio 双 auto 的 plain block 高度由 transfer pass（width/ratio 传递 +
//! min/max 钳制）决定。IFC 终化的两处高度写点——remeasure_text_with_float_exclusions
//! 的双向收敛（R3785）与 remeasure_inline_only_containers 的纯 inline 收缩臂——
//! 会按行盒高（内容仅一行 10px）覆盖传递值：block-aspect-ratio-026 的
//! `max-height:100% + padding-bottom:20px + border-box` target 100×100 被收到
//! 100×30。两处写点加 has_ar_transfer 豁免。

use crate::pipeline::RenderPipeline;

fn box_h(body: &str) -> f32 {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = format!("<html><head><style></style></head><body>{body}</body></html>");
    let result = pipeline.render_html(&html, "");
    let mut best = (0.0_f32, 0.0_f32);
    fn walk(b: &zero_layout_engine::types::LayoutBox, best: &mut (f32, f32)) {
        if (b.width - 100.0).abs() < 0.5 && b.height > 0.0 {
            *best = (b.width, b.height);
        }
        for c in &b.children {
            walk(c, best);
        }
    }
    walk(&result.layout.root, &mut best);
    best.1
}

/// max-height 传递 + 纯 inline 内容：高应 100（border-box，含 padding-bottom 20）。
#[test]
fn r4074_ar_height_not_reclaimed_by_ifc() {
    let h = box_h(
        "<div style=\"height: 100px;\">\
<div style=\"aspect-ratio: 1/1; max-height: 100%; padding-bottom: 20px; background: green; box-sizing: border-box;\">\
<div style=\"width: 20px; height: 10px; display: inline-block;\"></div></div></div>",
    );
    assert!(
        (h - 100.0).abs() < 0.5,
        "R4074: aspect-ratio 传递高不被 IFC 收缩（应为 100），got {h}"
    );
}

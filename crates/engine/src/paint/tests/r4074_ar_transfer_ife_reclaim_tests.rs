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

/// R4075（css-sizing-4 §4.2 + §5.2）：transferred min-width（min-height×ratio）不得
/// 覆盖显式 max-width——taffy 通用 min>max 约束表给 200 压过 max-width:100。
/// 断言（022）：外层 100 宽，width:auto 的 in-flow 绿子同步跟随 100。
#[test]
fn r4075_transferred_min_width_yields_to_explicit_max() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><head><style></style></head><body>\
<div style=\"min-height: 200px; max-width: 100px; aspect-ratio: 1/1;\">\
<div style=\"width: 300px; height: 10px;\"></div>\
<div style=\"height: 100px; background: green;\"></div></div></body></html>";
    let result = pipeline.render_html(html, "");
    let mut outer_w = 0.0;
    let mut green_w = 0.0;
    fn walk(b: &zero_layout_engine::types::LayoutBox, outer: &mut f32, green: &mut f32) {
        if *outer == 0.0 && (b.width - 100.0).abs() < 0.5 && (b.height - 200.0).abs() < 0.5 {
            *outer = b.width;
            for c in &b.children {
                if (c.height - 100.0).abs() < 0.5 {
                    *green = c.width;
                }
            }
        }
        for c in &b.children {
            walk(c, outer, green);
        }
    }
    walk(&result.layout.root, &mut outer_w, &mut green_w);
    assert!(
        (outer_w - 100.0).abs() < 0.5,
        "R4075: transferred min-width 不覆盖显式 max-width，外层应 100，got {outer_w}"
    );
    assert!(
        (green_w - 100.0).abs() < 0.5,
        "R4075: width:auto 绿子应跟随容器新宽 100，got {green_w}"
    );
}

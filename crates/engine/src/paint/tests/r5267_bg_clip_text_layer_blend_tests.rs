//! R5267 clip:text 恒色层间混合（css-compositing-1 §background-blend-mode）。
//!
//! 判据锚：background-clip-text-and-blend-mode box1（lighten(lime 层, blue 底色) =
//! cyan）与 clip-text-blend-mode（difference(#f00, normal(#ff0, red 底)) = lime）。
//! 全 normal（含未声明）保持既有 plain OVER 提取字节不变；kill-switch 回退测试在
//! `crates/engine/tests/`（独立进程——进程内 set_var 与并行单测互踩，R3858 教训）。

use crate::pipeline::RenderPipeline;

/// 单页渲染返回首个字形颜色（恒色染色下所有字形同色）。
fn first_glyph_color(html: &str) -> Option<(u8, u8, u8, u8)> {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(html, "");
    result
        .primitives()
        .glyphs
        .first()
        .map(|g| (g.color.r, g.color.g, g.color.b, g.color.a))
}

#[test]
fn lighten_layer_blends_with_background_color_to_cyan() {
    // box1 最小化：lime 图层 lighten blue 底色 → 逐通道 max = cyan（旧行为 plain
    // OVER 得 lime，13.27% 缺口主源）。
    let html = r#"<body style="margin:0"><div style="font-size:40px; background-clip: text; background-image: linear-gradient(rgb(0,255,0), rgb(0,255,0)); background-color: rgb(0,0,255); background-blend-mode: lighten; color: transparent">X</div></body>"#;
    assert_eq!(first_glyph_color(html), Some((0, 255, 255, 255)));
}

#[test]
fn two_layer_difference_composite_yields_lime() {
    // clip-text-blend-mode 最小化：底 red → #ff0 normal → #f00 difference →
    // |255-255,255-0,0-0| = lime（旧行为自顶向下 OVER 得 #ff0 黄——0.30% 假通过源）。
    let html = r#"<body style="margin:0"><div style="font-size:100px; background-image: linear-gradient(#f00, #f00), linear-gradient(#ff0, #ff0); background-color: red; background-blend-mode: difference, normal; background-clip: text; color: transparent">X</div></body>"#;
    assert_eq!(first_glyph_color(html), Some((0, 255, 0, 255)));
}

#[test]
fn all_normal_modes_keep_plain_over_extraction() {
    // 未声明 background-blend-mode → 既有 plain OVER 路径（单层 lime OVER blue = lime）。
    let html = r#"<body style="margin:0"><div style="font-size:40px; background-clip: text; background-image: linear-gradient(rgb(0,255,0), rgb(0,255,0)); background-color: rgb(0,0,255); color: transparent">X</div></body>"#;
    assert_eq!(first_glyph_color(html), Some((0, 255, 0, 255)));
}

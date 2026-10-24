//! R5267 kill-switch 回退锚（独立测试进程——`ZW_BG_CLIP_TEXT_LAYER_BLEND=0` 回退
//! plain OVER 提取；进程内 set_var 会与并行单测互踩，R3858 教训）。

use zero_engine::RenderPipeline;

#[test]
fn kill_switch_restores_plain_over_extraction() {
    // Rust 2024：set_var/remove_var 为 unsafe（进程全局可变）。
    unsafe { std::env::set_var("ZW_BG_CLIP_TEXT_LAYER_BLEND", "0") };
    let html = r#"<body style="margin:0"><div style="font-size:40px; background-clip: text; background-image: linear-gradient(rgb(0,255,0), rgb(0,255,0)); background-color: rgb(0,0,255); background-blend-mode: lighten; color: transparent">X</div></body>"#;
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(html, "");
    let glyph = result
        .primitives()
        .glyphs
        .first()
        .map(|g| (g.color.r, g.color.g, g.color.b, g.color.a));
    // 旧行为：lime 图层 plain OVER blue 底色 = lime（非混合 cyan）。
    assert_eq!(glyph, Some((0, 255, 0, 255)));
    unsafe { std::env::remove_var("ZW_BG_CLIP_TEXT_LAYER_BLEND") };
}

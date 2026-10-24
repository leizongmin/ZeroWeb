//! R5268 kill-switch 回退锚（独立测试进程——`ZW_BG_CLIP_TEXT_PER_LAYER=0` 回退整盒
//! 抑制；进程内 set_var 会与并行单测互踩，R3858 教训）。

use zero_engine::RenderPipeline;

#[test]
fn kill_switch_restores_wholesale_suppression() {
    // Rust 2024：set_var/remove_var 为 unsafe（进程全局可变）。
    unsafe { std::env::set_var("ZW_BG_CLIP_TEXT_PER_LAYER", "0") };
    // box2 形：回退后非 Text 层（red border-box）与 bg 色矩形一并整盒抑制，
    // 字形仍承载恒色合成（R5267 链路不变）：difference(green, red OVER white) = yellow。
    let html = r#"<body style="margin:0"><div style="font-size:100px; background-image: linear-gradient(rgb(0,255,0), rgb(0,255,0)), linear-gradient(rgb(255,0,0), rgb(255,0,0)); background-clip: text, border-box; background-color: rgb(255,255,255); background-blend-mode: difference, normal; color: transparent">X</div></body>"#;
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    let glyph = p.glyphs.first().map(|g| (g.color.r, g.color.g, g.color.b, g.color.a));
    let fills: Vec<(u8, u8, u8, u8)> = p
        .fills
        .iter()
        .map(|f| (f.color.r, f.color.g, f.color.b, f.color.a))
        .collect();
    assert_eq!(glyph, Some((255, 255, 0, 255)), "字形染色链路不受回退影响");
    assert!(
        !fills.contains(&(255, 0, 0, 255)),
        "回退后非 Text 层矩形应整盒抑制（无 red 填充），实际 {fills:?}"
    );
    unsafe { std::env::remove_var("ZW_BG_CLIP_TEXT_PER_LAYER") };
}

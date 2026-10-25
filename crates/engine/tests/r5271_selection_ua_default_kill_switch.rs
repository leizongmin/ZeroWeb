//! R5271 kill-switch 回退锚（独立测试进程——`ZW_SELECTION_HIGHLIGHT_DEFAULT=0`
//! 回退「无作者 ::selection 规则 → 无高亮」旧口径；进程内 set_var 会与并行单测
//! 互踩，R3858 教训。开关 ≡ R5065 同语义族）。

use zero_engine::RenderPipeline;

#[test]
fn kill_switch_restores_no_highlight_without_author_rule() {
    // Rust 2024：set_var/remove_var 为 unsafe（进程全局可变）。
    unsafe { std::env::set_var("ZW_SELECTION_HIGHLIGHT_DEFAULT", "0") };
    let html =
        r#"<body style="margin:0"><div><span id="sel" style="font: 40px Ahem">With getSelection()</span></div></body>"#;
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("#sel".to_string(), 0, 19)]);
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    assert!(
        !p.fills.iter().any(|f| f.color.a > 0),
        "回退后无作者规则不应铺任何高亮 fill，实际 fills {:?}",
        p.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    assert!(
        p.glyphs
            .iter()
            .filter(|g| g.color != Color::rgba(0, 0, 0, 255))
            .all(|g| g.color == Color::rgba(255, 0, 255, 255)),
        "回退后字形应保持元素色 fuchsia，实际 {:?}",
        p.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
    unsafe { std::env::remove_var("ZW_SELECTION_HIGHLIGHT_DEFAULT") };
}

use zero_render_foundation::color::Color;

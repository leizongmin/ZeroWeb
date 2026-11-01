//! R5384 kill-switch 回退锚（独立测试进程——`ZW_CE_CARET_SEL=0` 时管线 Selection
//! 插入点回流被忽略，恢复 R5383「焦点默认位」行为；进程内 set_var 与并行单测互踩，
//! R3858 教训——本文件仅此一测，独占进程环境）。

use zero_engine::RenderPipeline;
use zero_render_foundation::color::Color;

#[test]
fn kill_switch_ignores_selection_target() {
    // Rust 2024：set_var/remove_var 为 unsafe（进程全局可变）。
    unsafe { std::env::set_var("ZW_CE_CARET_SEL", "0") };
    let html = r#"<body style="margin:0"><style>#t:focus{outline:none}</style><div id="t" contenteditable style="caret-color: lime; caret-shape: block; caret-animation: manual; font-size: 5em">abc</div></body>"#;
    let mut pipeline = RenderPipeline::new(600.0, 300.0);
    pipeline.set_ce_caret(Some(("#t".into(), vec![0], 2, true)));
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    // 开关关闭：目标忽略 → 无焦点 → 无 caret fill（若目标被采纳，行尾 caret 会发射）。
    assert!(
        !p.fills.iter().any(|f| f.color == Color::rgba(0, 255, 0, 255)),
        "开关关闭后 Selection 目标不应驱动 caret fill，实际 fills {:?}",
        p.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    unsafe { std::env::remove_var("ZW_CE_CARET_SEL") };
}

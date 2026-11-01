//! R5383 kill-switch 回退锚（独立测试进程——`ZW_CE_CARET=0` 回退 contenteditable
//! 宿主不绘插入 caret 旧行为；进程内 set_var 与并行单测互踩，R3858 教训——本文件
//! 仅此一测，独占进程环境）。

use zero_engine::RenderPipeline;
use zero_render_foundation::color::Color;

#[test]
fn kill_switch_restores_no_editable_caret() {
    // Rust 2024：set_var/remove_var 为 unsafe（进程全局可变）。
    unsafe { std::env::set_var("ZW_CE_CARET", "0") };
    let html = r#"<body style="margin:0"><style>#t:focus{outline:none}</style><div id="t" contenteditable style="caret-color: lime; caret-shape: block; font-size: 5em">abc</div></body>"#;
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_focused_selector(Some("#t"));
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    assert!(
        !p.fills.iter().any(|f| f.color == Color::rgba(0, 255, 0, 255)),
        "开关关闭后 CE 宿主不应绘制 caret fill，实际 fills {:?}",
        p.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    unsafe { std::env::remove_var("ZW_CE_CARET") };
}

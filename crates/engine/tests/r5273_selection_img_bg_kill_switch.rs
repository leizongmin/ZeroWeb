//! R5273 kill-switch 回退锚（独立测试进程——`ZW_SELECTION_IMG_BG=0` 回退
//! 「图片替换元素不叠作者 ::selection 底」旧口径；进程内 set_var 会与并行
//! 单测互踩，R3858 教训）。

use zero_engine::RenderPipeline;

#[test]
fn kill_switch_restores_untinted_image() {
    // Rust 2024：set_var/remove_var 为 unsafe（进程全局可变）。
    unsafe { std::env::set_var("ZW_SELECTION_IMG_BG", "0") };
    let html = r#"<html><head><style>img::selection { background: green; }</style></head><body style="margin:0"><p style="margin:0">A <img src="blue.png" style="width:15px;height:15px"> B</p></body></html>"#;
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("html".to_string(), 0, 100)]);
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    assert!(
        !p.fills.iter().any(|f| f.color.a == 204),
        "kill-switch 回退后不应叠图片 tint，实际 fills {:?}",
        p.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    unsafe { std::env::remove_var("ZW_SELECTION_IMG_BG") };
}

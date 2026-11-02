//! R5391 kill-switch 回退锚（独立测试进程——`ZW_NESTED_CLIP_INTERSECT=0` 回退
//! R3922「最近者优先覆盖」携带矩形旧口径；进程内 set_var 与并行单测互踩，R3858
//! 教训——本文件仅此一测，独占进程环境）。

use zero_engine::RenderPipeline;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;

#[test]
fn kill_switch_restores_nearest_wins_carried_rect() {
    // Rust 2024：set_var/remove_var 为 unsafe（进程全局可变）。
    unsafe { std::env::set_var("ZW_NESTED_CLIP_INTERSECT", "0") };
    let html = "<html><body style=\"margin:0\">\
<div style=\"width:100px; height:100px; overflow:hidden\">\
<div style=\"height:50px\"></div>\
<div style=\"width:100px; height:100px; overflow:hidden\">\
<div style=\"position:relative; width:100px; height:200px; background:#f00\"></div>\
</div></div></body></html>";
    let result = RenderPipeline::new(400.0, 300.0).render_html(html, "");
    let p = result.primitives();
    let red = Color::rgba(255, 0, 0, 255);
    assert!(
        p.fills
            .iter()
            .any(|f| f.color == red && f.rect == Rect::new(0.0, 50.0, 100.0, 100.0)),
        "开关关闭后携带矩形应回退最近者优先（内层视口 y∈[50,150)，fill 高 100、\
         y∈[100,150) 漏出外层带）；实际 fills {:?}",
        p.fills.iter().map(|f| (f.color, f.rect)).collect::<Vec<_>>()
    );
    unsafe { std::env::remove_var("ZW_NESTED_CLIP_INTERSECT") };
}

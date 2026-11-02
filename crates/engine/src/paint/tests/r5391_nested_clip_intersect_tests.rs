//! R5391（嵌套裁剪器携带矩形取交集）：R3922 携带矩形「最近者优先覆盖」使外层
//! overflow 裁剪对迟延 positioned 后代丢失——css-overflow-3 可视域 = 沿途裁剪
//! 矩形组合，normal flow 主循环路径逐层 clip 双裁都生效，携带矩形须同语义。
//! driving: clipped-scroller-add-content（外层 hidden 对滚动后 relative 内容漏裁
//! 20px 带 1.25%）；overflow-clip-margin-002 / single-axis-overflow-scroll-to-clip
//! 同窗连带翻绿。

use crate::pipeline::RenderPipeline;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;

/// 嵌套裁剪器：外层 hidden 100×100（y∈[0,100)）+ 50px spacer + 内层 hidden
/// 100×100（y∈[50,150)），内层 position:relative 红底 100×200。可见域 =
/// 交集 y∈[50,100)——fill 截断高 50；R3922 最近者优先时携带矩形 = 内层
/// y∈[50,150)，fill 高 100（y∈[100,150) 漏出外层带）。
#[test]
fn nested_clippers_intersect_carried_rect() {
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
            .any(|f| f.color == red && f.rect == Rect::new(0.0, 50.0, 100.0, 50.0)),
        "relative 子 fill 应截断到交集 y∈[50,100)（高 50）；实际 fills {:?}",
        p.fills.iter().map(|f| (f.color, f.rect)).collect::<Vec<_>>()
    );
    assert!(
        !p.fills.iter().any(|f| f.color == red && f.rect.bottom() > 100.5),
        "y≥100 带属外层 hidden 之外，不得有任何红色 fill 漏裁；实际 fills {:?}",
        p.fills.iter().map(|f| (f.color, f.rect)).collect::<Vec<_>>()
    );
}

/// 单裁剪器（无外层携带）：交集臂 None 分支语义不变——fill 截断到自身视口
/// y∈[0,100)（R3922 原口径，防交集臂误改简单路径）。
#[test]
fn single_clipper_keeps_self_rect() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"width:100px; height:100px; overflow:hidden\">\
<div style=\"position:relative; width:100px; height:200px; background:#f00\"></div>\
</div></body></html>";
    let result = RenderPipeline::new(400.0, 300.0).render_html(html, "");
    let p = result.primitives();
    let red = Color::rgba(255, 0, 0, 255);
    assert!(
        p.fills
            .iter()
            .any(|f| f.color == red && f.rect == Rect::new(0.0, 0.0, 100.0, 100.0)),
        "单裁剪器 relative 子 fill 应截断到自身视口 y∈[0,100)；实际 fills {:?}",
        p.fills.iter().map(|f| (f.color, f.rect)).collect::<Vec<_>>()
    );
}

/// 交集为空（内层视口完全在外层之外）：完全不可见——零面积矩形承载，红色 fill
/// 全清零；R3922 口径下内层带照常漏出（该形态由 kill-switch 锚钉）。
#[test]
fn empty_intersection_clips_everything() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"width:100px; height:30px; overflow:hidden\">\
<div style=\"height:50px\"></div>\
<div style=\"width:100px; height:100px; overflow:hidden\">\
<div style=\"position:relative; width:100px; height:200px; background:#f00\"></div>\
</div></div></body></html>";
    let result = RenderPipeline::new(400.0, 300.0).render_html(html, "");
    let p = result.primitives();
    let red = Color::rgba(255, 0, 0, 255);
    assert!(
        !p.fills
            .iter()
            .any(|f| f.color == red && f.rect.size.width > 0.0 && f.rect.size.height > 0.0),
        "交集为空时红色 fill 应全清零（完全不可见）；实际 fills {:?}",
        p.fills.iter().map(|f| (f.color, f.rect)).collect::<Vec<_>>()
    );
}

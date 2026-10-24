//! R5255（迟延 positioned 后代的 shaped clip edge）：R5201 圆角改写只及主循环
//! 窗口图元——position:relative 子经 scope flush 补裁（R3922），此前仅矩形裁剪
//! 时角部漏成方角。driving: overflow-clip-margin-visual-box-and-value-with-
//! border-radius（三容器外圈角部 993px；相对子为迟延路径独占形态）。

use crate::pipeline::RenderPipeline;
use zero_render_foundation::geometry::Rect;

/// overflow:clip + `overflow-clip-margin: border-box 5px` + border-radius:40px：
/// border box 100（border/padding 20、content 20），裁剪边 = border box 外扩 5 →
/// (−5,−5,110,110)，dx = −5 → 角半径 = 40+5 = 45。relative 子 fill 自 (40,40)
/// 铺开 200×200，补裁窗口截断到 (40,40,65,65)（触裁剪边右/下），右下角承接
/// 圆角矩形改写（R5201 半径模型对迟延路径同样生效；方角形态 = 修前签名）。
#[test]
fn r5255_deferred_positioned_child_rounded_clip() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:0\">\
<div style=\"overflow: clip; overflow-clip-margin: border-box 5px; border-radius: 40px; border: 20px solid transparent; padding: 20px; width: 20px; height: 20px\">\
<div style=\"position: relative; background: #000; width: 200px; height: 200px\"></div></div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    let converted = p
        .rounded_rects
        .iter()
        .find(|rr| rr.rect == Rect::new(40.0, 40.0, 65.0, 65.0))
        .expect("迟延 relative 子 fill 应在补裁窗口内改写为圆角矩形");
    assert!(
        converted.top_left_radius == 0.0
            && converted.top_right_radius == 0.0
            && (converted.bottom_right_radius - 45.0).abs() < 0.5
            && converted.bottom_left_radius == 0.0,
        "仅右下角承接半径 45（spec 40 + 外扩 5），got {:?}",
        converted
    );
    assert!(
        !p.fills.iter().any(|f| f.rect == Rect::new(40.0, 40.0, 65.0, 65.0)),
        "截断后的矩形 fill 应清零，不再以方角重绘"
    );
}

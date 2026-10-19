//! R5201（CSS Overflow 3 §3 裁剪区域圆角）：overflow:hidden/clip + border-radius
//! 的子内容圆角裁剪——触及裁剪边的 fill 改写圆角矩形（clip edge 形如外 box-shadow
//! spread，csswg #7103）。driving: overflow-clip-margin-border-radius-002
//! （5 子案 clip 尺寸全对、独缺角部圆整 0.53%）。

use crate::pipeline::RenderPipeline;
use zero_render_foundation::geometry::Rect;

/// overflow:clip（默认 padding-box 裁剪边）+ border-radius:40px：border box 100
///（border 20 + padding 20 + content 20），裁剪边 = padding box (20,20,60,60)，
/// dx = 20 → 角半径 = clamp0(40−20) = 20。子内容自 content 原点铺开，截断后
/// (40,40,40,40) 触及裁剪窗右/下边 → 右下角承接半径改写圆角矩形。
#[test]
fn r5201_overflow_clip_rounds_child_fill() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:0\">\
<div style=\"overflow: clip; border-radius: 40px; border: 20px solid transparent; padding: 20px; width: 20px; height: 20px\">\
<div style=\"background: #000; width: 200px; height: 200px\"></div></div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    let converted = p
        .rounded_rects
        .iter()
        .find(|rr| rr.rect == Rect::new(40.0, 40.0, 40.0, 40.0))
        .expect("触及裁剪角的子内容 fill 应改写为圆角矩形");
    assert!(
        converted.top_left_radius == 0.0
            && converted.top_right_radius == 0.0
            && (converted.bottom_right_radius - 20.0).abs() < 0.5
            && converted.bottom_left_radius == 0.0,
        "仅右下角承接半径 20（40−max(border 20)），got {:?}",
        converted
    );
    assert!(
        !p.fills.iter().any(|f| f.rect == Rect::new(40.0, 40.0, 40.0, 40.0)),
        "原 fill 应清零，不再以矩形重绘"
    );
}

/// overflow-clip-margin 扩展裁剪边（padding-box + 10px）：裁剪边 = padding box
/// 外扩 10 → (10,10,80,80)；外 box-shadow spread 模型以 padding-box 圆角为基
///（spec 40 − border 20 = 20，外扩 +10）→ 角半径 30。子内容截断后 (40,40,50,50)
/// 触及右/下边 → 右下角半径 30。
#[test]
fn r5201_clip_margin_expands_clip_edge_and_radius() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:0\">\
<div style=\"overflow: clip; overflow-clip-margin: 10px; border-radius: 40px; border: 20px solid transparent; padding: 20px; width: 20px; height: 20px\">\
<div style=\"background: #000; width: 200px; height: 200px\"></div></div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    let converted = p
        .rounded_rects
        .iter()
        .find(|rr| rr.rect == Rect::new(40.0, 40.0, 50.0, 50.0))
        .expect("margin 扩展后的裁剪角应承载圆角矩形改写");
    assert!(
        (converted.bottom_right_radius - 30.0).abs() < 0.5,
        "右下角半径应为 30（padding-box 圆角 20 + 外扩 10），got {:?}",
        converted
    );
}

/// overflow-clip-margin: content-box 基准内缩：dx = border 20 + padding 20 = 40
/// → 角半径 = clamp0(40−40) = 0（无圆角，矩形裁剪，零行为变更——driver ① 形）。
#[test]
fn r5201_content_box_base_full_inset_keeps_rect() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:0\">\
<div style=\"overflow: clip; overflow-clip-margin: content-box; border-radius: 40px; border: 20px solid transparent; padding: 20px; width: 20px; height: 20px\">\
<div style=\"background: #000; width: 200px; height: 200px\"></div></div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    assert!(
        p.rounded_rects
            .iter()
            .all(|rr| rr.rect != Rect::new(40.0, 40.0, 20.0, 20.0)),
        "全内缩圆角清零后不应改写圆角矩形，保持矩形裁剪"
    );
    assert!(
        p.fills.iter().any(|f| f.rect == Rect::new(40.0, 40.0, 20.0, 20.0)),
        "子内容 fill 应保持矩形（截断到 content box）"
    );
}

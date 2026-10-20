//! R5203（CSS Backgrounds 3 #corner-shaping）：① background-clip 内盒（padding/
//! content-box）的背景**色**圆角 = 外圈半径按越过的边宽同心内缩（内曲线同 §5.5
//! 模型）；② 单角百分比半径不按半短边钳制（§5.5 角重叠收缩才是规范口径）。
//! driving: background-clip-{content,padding}-box-with-border-radius-*。

use crate::pipeline::RenderPipeline;
use zero_render_foundation::geometry::Rect;

/// content-box clip（003 案构造）：border 25 transparent + padding 25 + radius
/// `100% 0 0 0`——% 相对 border-box 宽 200（100+50+50），单角无 §5.5 收缩 →
/// 外圈 TL 200；内圈 TL = 200 − (border 25 + padding 25) = 150，落在 content
/// box (58,58,100,100) 上——非外圈原半径（修前不自洽缺陷）。
#[test]
fn r5203_content_box_clip_insets_inner_curve() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:8px\">\
<div style=\"width:100px; height:100px; border:25px solid transparent; padding:25px;\
 border-radius:100% 0 0 0; background:#000; background-clip:content-box\"></div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    let rr = p
        .rounded_rects
        .iter()
        .find(|rr| rr.rect == Rect::new(58.0, 58.0, 100.0, 100.0))
        .expect("content-box 背景色应发圆角矩形于 content box");
    assert!(
        (rr.top_left_radius - 150.0).abs() < 0.5
            && rr.top_right_radius == 0.0
            && rr.bottom_right_radius == 0.0
            && rr.bottom_left_radius == 0.0,
        "内圈 TL 半径应为 150（外圈 200 同心内缩 border 25 + padding 25），got {:?}",
        rr
    );
}

/// padding-box clip（002 案构造）：padding 20 + border 20，均匀 `100%`（border-box
/// 100 宽 → 四角 100）先经 §5.5 收缩 → 外圈 50（内切圆）；内圈 = 50 − border 20
/// = 30，落在 padding box (28,28,60,60) 上——§5.5 收缩与同心内缩组合正确。
#[test]
fn r5203_padding_box_clip_insets_inner_curve() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:8px\">\
<div style=\"width:20px; height:20px; padding:20px; border:20px solid transparent;\
 border-radius:100%; background:#00f; background-clip:padding-box\"></div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    let rr = p
        .rounded_rects
        .iter()
        .find(|rr| rr.rect == Rect::new(28.0, 28.0, 60.0, 60.0))
        .expect("padding-box 背景色应发圆角矩形于 padding box");
    assert!(
        (rr.top_left_radius - 30.0).abs() < 0.5
            && (rr.top_right_radius - 30.0).abs() < 0.5
            && (rr.bottom_right_radius - 30.0).abs() < 0.5
            && (rr.bottom_left_radius - 30.0).abs() < 0.5,
        "内圈四角半径均应 30（外圈 50 − border 20），got {:?}",
        rr
    );
}

/// 单角百分比不按半短边钳制：100×100 无边框盒 `border-radius:100% 0 0 0` 与
/// `100px 0 0 0` 同形（均 TL 100 四分之一圆）——修前 % 被钳 50 与 px 不同形
///（002 案 %/px 两侧不对称实证）。
#[test]
fn r5203_percent_radius_matches_px_on_single_corner() {
    for radius in ["100%", "100px"] {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = format!(
            "<html><body style=\"margin:8px\">\
<div style=\"width:100px; height:100px; border-radius:{radius} 0 0 0; background:#000\"></div>\
</body></html>"
        );
        let result = pipeline.render_html(&html, "");
        let p = result.primitives();
        let rr = p
            .rounded_rects
            .iter()
            .find(|rr| rr.rect == Rect::new(8.0, 8.0, 100.0, 100.0))
            .unwrap_or_else(|| panic!("{radius}: 背景色圆角矩形应于 border box"));
        assert!(
            (rr.top_left_radius - 100.0).abs() < 0.5,
            "{radius}: TL 应为 100（%不按半短边钳制，与 px 同形），got {:?}",
            rr
        );
    }
}

/// §5.5 角重叠收缩维持：均匀超大半径（999px）在 100×100 盒上四角收缩到 50
///（f·r = 半边长）——去钳制不破坏重叠收缩路径（胶囊/圆形不受影响）。
#[test]
fn r5203_uniform_oversized_radius_still_shrinks() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:8px\">\
<div style=\"width:100px; height:100px; border-radius:999px; background:#000\"></div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    let rr = p
        .rounded_rects
        .iter()
        .find(|rr| rr.rect == Rect::new(8.0, 8.0, 100.0, 100.0))
        .expect("背景色圆角矩形应于 border box");
    assert!(
        (rr.top_left_radius - 50.0).abs() < 0.5
            && (rr.top_right_radius - 50.0).abs() < 0.5
            && (rr.bottom_right_radius - 50.0).abs() < 0.5
            && (rr.bottom_left_radius - 50.0).abs() < 0.5,
        "四角应收缩到 50（§5.5 均匀半径 f·r = 半边长），got {:?}",
        rr
    );
}

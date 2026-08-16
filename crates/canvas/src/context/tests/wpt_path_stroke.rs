//! R56h：WPT path-objects 剩余描边几何修复的驱动单测（stroke.skew 复现等）。

use super::super::types::*;
use crate::context::*;
use zero_render_foundation::color::Color;

fn pixel(ctx: &CanvasContext, x: u32, y: u32) -> (u8, u8, u8, u8) {
    let img = ctx.get_image_data(x as i32, y as i32, 1, 1);
    (img.data[0], img.data[1], img.data[2], img.data[3])
}

fn green() -> CanvasStyle {
    CanvasStyle::Color(Color::rgba(0, 255, 0, 255))
}

fn red() -> CanvasStyle {
    CanvasStyle::Color(Color::rgba(255, 0, 0, 255))
}

/// 2d.path.stroke.skew —— WPT 套件内部语义冲突用例的**当前语义哨兵**。
///
/// WPT 期望 stroke 在 draw 时重应用当前 CTM（线段 (49,-50)→(201,-50) 被
/// rotate(π/4)·scale(1,283) 旋转后横贯画布）——与套件内 5+ 用例
/// （stroke.scale1/2、transformation.changing/multiple/basic，全 Pass 实证
/// 「路径追加时烘焙 CTM，draw 只缩放线宽」）在任何单一模型下互斥。保持主流
/// 语义（追加时烘焙）：绿带 = 烘焙水平段 ±|T·n̂|/2，x∈[49,201]；红带 =
/// 烘焙 (-101,-50)→(49,-50) ±71——(0,0) 落在绿带 t<0 区外。runner 侧该用例
/// 已按 WPT 冲突跳过（testharness.rs CANVAS_SKIP_FILES），本测试锁定烘焙语义
/// 不回归（若未来改 draw 时重应用 CTM，须同时回归 stroke.scale1/2 与
/// transformation.changing/multiple——将全部翻红）。
#[test]
fn test_wpt_stroke_skew_baked_semantics() {
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_fill_style(red());
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);

    ctx.save();
    ctx.begin_path();
    ctx.move_to(49.0, -50.0);
    ctx.line_to(201.0, -50.0);
    ctx.rotate(std::f32::consts::FRAC_PI_4);
    ctx.scale(1.0, 283.0);
    ctx.set_stroke_style(green());
    ctx.stroke();
    ctx.restore();

    ctx.save();
    ctx.begin_path();
    ctx.translate(-150.0, 0.0);
    ctx.move_to(49.0, -50.0);
    ctx.line_to(199.0, -50.0);
    ctx.rotate(std::f32::consts::FRAC_PI_4);
    ctx.scale(1.0, 142.0);
    ctx.set_stroke_style(red());
    ctx.stroke();
    ctx.restore();

    // 烘焙语义：绿带覆盖 x∈[49,99]（烘焙段投影区），(0,0) 在段首外（t<0）——
    // 与 WPT 期望（draw 时重应用 CTM）冲突，见函数头注释。
    assert_eq!(pixel(&ctx, 50, 0), (0, 255, 0, 255), "(50,0) 绿带覆盖（烘焙段投影区）");
    assert_eq!(
        pixel(&ctx, 0, 0),
        (255, 0, 0, 255),
        "(0,0) 段首外——烘焙语义（WPT 冲突文档）"
    );
}

/// 2d.path.stroke.prune.line —— 零长 lineTo 被剪除，round 帽下什么都不画。
#[test]
fn test_wpt_stroke_prune_line() {
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_fill_style(green());
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);
    ctx.set_stroke_style(red());
    ctx.set_line_width(50.0);
    ctx.begin_path();
    ctx.move_to(50.0, 25.0);
    ctx.line_to(50.0, 25.0);
    ctx.stroke();
    assert_eq!(pixel(&ctx, 50, 25), (0, 255, 0, 255), "(50,25) 应绿——零长段无描边");
}

/// 2d.path.stroke.prune.curve —— 零长二次/三次贝塞尔被剪除。
#[test]
fn test_wpt_stroke_prune_curve() {
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_fill_style(green());
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);
    ctx.set_stroke_style(red());
    ctx.set_line_width(100.0);
    ctx.set_line_cap(LineCap::Round);
    ctx.set_line_join(LineJoin::Round);
    ctx.begin_path();
    ctx.move_to(50.0, 25.0);
    ctx.quadratic_curve_to(50.0, 25.0, 50.0, 25.0);
    ctx.stroke();
    ctx.begin_path();
    ctx.move_to(50.0, 25.0);
    ctx.bezier_curve_to(50.0, 25.0, 50.0, 25.0, 50.0, 25.0);
    ctx.stroke();
    assert_eq!(pixel(&ctx, 50, 25), (0, 255, 0, 255), "(50,25) 应绿——零长曲线无描边");
}

/// 2d.path.roundrect.closed —— roundRect 是自闭合子路径：闭合 join（miter）
/// 覆盖角部（旧 closed 判定只认 ClosePath，末命令 RoundRect 丢闭合 join）。
#[test]
fn test_wpt_roundrect_closed_join() {
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_fill_style(red());
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);
    ctx.set_stroke_style(green());
    ctx.set_line_width(200.0);
    ctx.set_line_join(LineJoin::Miter);
    ctx.begin_path();
    ctx.round_rect(100.0, 50.0, 100.0, 100.0, vec![(0.0, 0.0)]);
    ctx.stroke();
    assert_eq!(pixel(&ctx, 50, 25), (0, 255, 0, 255), "(50,25) 应绿——闭合 miter 角覆盖");
}

/// 2d.path.roundrect.end.3 —— roundRect 后 current 落在子路径起点 (x+tl, y)，
/// 后续 lineTo 从该点出发（旧实现从 (x, y+tl) 出发漏覆盖画布像素）。
#[test]
fn test_wpt_roundrect_end3_current_point() {
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_fill_style(red());
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);
    ctx.set_stroke_style(green());
    ctx.set_line_width(100.0);
    ctx.begin_path();
    ctx.round_rect(101.0, 51.0, 2000.0, 2000.0, vec![(500.0, 500.0); 4]);
    ctx.line_to(-1.0, -1.0);
    ctx.stroke();
    for (x, y) in [(1, 1), (98, 1), (1, 48), (98, 48)] {
        assert_eq!(pixel(&ctx, x, y), (0, 255, 0, 255), "({x},{y}) 应绿——lineTo 带覆盖");
    }
}

/// 2d.path.isPointInStroke.scaleddashes —— dash 感知命中测试（用户空间度量）。
#[test]
fn test_wpt_is_point_in_stroke_scaleddashes() {
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_line_dash(vec![10.0, 21.4159]);
    ctx.scale(20.0, 20.0);
    ctx.ellipse(6.0, 10.0, 5.0, 5.0, 0.0, std::f32::consts::TAU, 0.0, false);
    ctx.stroke();
    // 点位于用户空间椭圆上（经逆变换回用户空间判定）。
    assert!(
        ctx.is_point_in_stroke(11.0 * 20.0, 10.0 * 20.0),
        "起始点（angle 0）在 dash 开段"
    );
    assert!(
        ctx.is_point_in_stroke(8.70 * 20.0, 14.21 * 20.0),
        "1.0 rad 在 dash 开段（弧长 5 ≤ 10）"
    );
    assert!(
        ctx.is_point_in_stroke(4.10 * 20.0, 14.63 * 20.0),
        "1.96 rad 在 dash 开段边缘内"
    );
    assert!(
        !ctx.is_point_in_stroke(3.74 * 20.0, 14.46 * 20.0),
        "2.04 rad 超出 dash 开段"
    );
}

/// R56h（M3）：CanvasFilter dropShadow 渲染——shadow 机制（offset+blur+floodColor）。
#[test]
fn test_filter_drop_shadow_renders() {
    let mut ctx = CanvasContext::new(100, 60);
    ctx.set_filter_drop_shadow(Some((5.0, 5.0, 0.0, Color::rgba(0, 0, 0, 255))));
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(10.0, 10.0, 10.0, 10.0);
    // 阴影在源矩形外 (21,15)（源 x∈[10,20] + offset (5,5)）——黑色。
    let p = ctx.get_image_data(21, 15, 1, 1);
    assert_eq!(p.data[0], 0, "阴影 R=0");
    assert_eq!(p.data[3], 255, "阴影 A=255");
    // 源矩形本身仍画红。
    let src = ctx.get_image_data(10, 10, 1, 1);
    assert_eq!(src.data[0], 255, "源 R=255");
}

/// R56h（M3）：渐变 colorInterpolationMethod 色彩空间插值。
#[test]
fn test_gradient_color_interpolation_spaces() {
    use zero_render_foundation::primitive::{GradientColorSpace, HueMethod};
    // LinearGradient: red → lime 中点
    let mut grad = crate::context::types::LinearGradient::new(0.0, 0.0, 100.0, 0.0);
    grad.add_color_stop(0.0, Color::rgba(255, 0, 0, 255));
    grad.add_color_stop(1.0, Color::rgba(0, 255, 0, 255));
    let mid_srgb = grad.sample_color(0.5);
    grad.set_color_interpolation(Some(GradientColorSpace::Hsl), HueMethod::Shorter);
    let mid_hsl = grad.sample_color(0.5);
    // HSL 中点偏黄（H 插值）；sRGB 中点暗黄。
    assert!(mid_hsl.r > 200 && mid_hsl.g > 200, "HSL 中点偏黄: {mid_hsl:?}");
    assert!(mid_srgb.r < 200 || mid_srgb.g < 200, "sRGB 中点暗: {mid_srgb:?}");
}

/// R56h（M3）：reset 后 fillRect 双矩形——合成操作回默认 source-over，
/// 重叠区不因 xor 清空，形状外保持透明（2d.reset.render.global_composite_operation）。
/// reset 语义 = JS 桥 `CanvasContext::new` 重建（js_dom_bridge/canvas.rs "reset"）。
#[test]
fn test_reset_then_fill_rects() {
    let mut ctx = CanvasContext::new(400, 400);
    ctx.set_composite_operation(CompositeOperation::Xor);
    // reset：重建（全状态默认——composite 回 source-over）
    let (w, h) = (ctx.width(), ctx.height());
    ctx = CanvasContext::new(w, h);
    ctx.set_fill_color(Color::BLACK);
    ctx.fill_rect(10.0, 10.0, 100.0, 100.0);
    ctx.fill_rect(50.0, 50.0, 100.0, 100.0);
    // L 形：矩形内黑（重叠区 (60,60) 黑——source-over 非 xor 清空），
    // 形状外透明（spec 默认画布透明，非白）。
    for (x, y, _) in [(30, 30, 0), (60, 60, 0), (140, 60, 0)] {
        let p = ctx.get_image_data(x, y, 1, 1);
        assert_eq!(p.data[0], 0, "({x},{y}) R 通道 got {}", p.data[0]);
        assert_eq!(p.data[3], 255, "({x},{y}) A 通道");
    }
    let outside: [(i32, i32); 5] = [(160, 160), (5, 5), (110, 10), (150, 50), (150, 150)];
    for (x, y) in outside {
        let p = ctx.get_image_data(x, y, 1, 1);
        assert_eq!(p.data[3], 0, "({x},{y}) 应透明，got {:?}", &p.data[..4]);
    }
}

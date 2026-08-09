//! Canvas 上下文测试（edge 批次）。

use super::super::types::*;
use crate::context::*;
use crate::path::{Path2D, PathCommand};
use zero_render_foundation::color::Color;

///
/// 创建 10x8 的 ImageData，所有像素应为 rgba(0,0,0,0)。
#[test]
fn test_create_image_data_blank() {
    let ctx = CanvasContext::new(100, 100);
    let img = ctx.create_image_data(10, 8);
    assert_eq!(img.width, 10, "width should be 10");
    assert_eq!(img.height, 8, "height should be 8");
    assert_eq!(img.data.len(), 10 * 8 * 4, "data length should be 10*8*4 = 320");
    // 所有像素应为透明黑色
    for chunk in img.data.chunks_exact(4) {
        assert_eq!(chunk, &[0, 0, 0, 0], "pixel should be transparent black (rgba 0,0,0,0)");
    }
}

/// 测试默认变换矩阵为单位矩阵。
///
/// 新创建的 CanvasContext 的 get_transform() 应返回单位矩阵。
#[test]
fn test_get_transform_default_identity() {
    let ctx = CanvasContext::new(100, 100);
    let t = ctx.get_transform();
    assert!((t.a - 1.0).abs() < f32::EPSILON, "a should be 1.0");
    assert!((t.b).abs() < f32::EPSILON, "b should be 0.0");
    assert!((t.c).abs() < f32::EPSILON, "c should be 0.0");
    assert!((t.d - 1.0).abs() < f32::EPSILON, "d should be 1.0");
    assert!((t.e).abs() < f32::EPSILON, "e should be 0.0");
    assert!((t.f).abs() < f32::EPSILON, "f should be 0.0");
}

/// 测试执行 translate+rotate+scale 后 get_transform 返回正确矩阵。
///
/// 依次执行 translate(10,20)、rotate(π/2)、scale(2,3)，
/// 验证 get_transform() 返回的矩阵不等于单位矩阵，且为有限值。
#[test]
fn test_get_transform_after_ops() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.translate(10.0, 20.0);
    ctx.rotate(std::f32::consts::FRAC_PI_2);
    ctx.scale(2.0, 3.0);

    let t = ctx.get_transform();
    // 不应为单位矩阵
    assert!(
        (t.a - 1.0).abs() > 0.01 || (t.d - 1.0).abs() > 0.01 || (t.e).abs() > 0.01 || (t.f).abs() > 0.01,
        "transform after ops should not be identity"
    );
    // 所有元素应为有限值
    assert!(t.a.is_finite(), "a should be finite");
    assert!(t.b.is_finite(), "b should be finite");
    assert!(t.c.is_finite(), "c should be finite");
    assert!(t.d.is_finite(), "d should be finite");
    assert!(t.e.is_finite(), "e should be finite");
    assert!(t.f.is_finite(), "f should be finite");
}

/// 测试 transform() 方法是乘法叠加而非替换。
///
/// 先 scale(2,1) 再 transform(1,0,0,1,10,0)（即 translate(10,0)），
/// 验证 transform 是后乘叠加，结果不同于 set_transform 直接替换。
#[test]
fn test_transform_multiply_vs_set() {
    let mut ctx1 = CanvasContext::new(100, 100);
    ctx1.scale(2.0, 1.0);
    ctx1.transform(1.0, 0.0, 0.0, 1.0, 10.0, 0.0); // translate via transform()
    let t1 = ctx1.get_transform();

    // scale(2,1) * translate(10,0) 后乘：
    // [2 0 0]   [1 0 10]   [2 0 20]
    // [0 1 0] * [0 1  0] = [0 1  0]
    // [0 0 1]   [0 0  1]   [0 0  1]
    // 所以 a=2, d=1, e=20
    assert!((t1.a - 2.0).abs() < f32::EPSILON, "a should be 2.0");
    assert!((t1.d - 1.0).abs() < f32::EPSILON, "d should be 1.0");
    assert!((t1.e - 20.0).abs() < f32::EPSILON, "e should be 20.0 (2*10)");

    // 使用 set_transform 直接设置 a=2, d=1, e=10
    let mut ctx2 = CanvasContext::new(100, 100);
    ctx2.set_transform(2.0, 0.0, 0.0, 1.0, 10.0, 0.0);
    let t2 = ctx2.get_transform();

    // transform 叠加 vs set_transform 替换，结果应不同
    assert!(
        (t1.e - t2.e).abs() > 0.01,
        "transform multiply (e={}) should differ from set_transform (e={})",
        t1.e,
        t2.e
    );
}

/// 测试 miter_limit 默认值为 10.0。
///
/// 新创建的 CanvasContext 的 miter_limit() 应返回 10.0。
#[test]
fn test_miter_limit_default_value() {
    let ctx = CanvasContext::new(100, 100);
    assert!(
        (ctx.miter_limit() - 10.0).abs() < f32::EPSILON,
        "default miter_limit should be 10.0, got {}",
        ctx.miter_limit()
    );
}

/// 测试 miter_limit 在 save/restore 中正确保存和恢复。
///
/// 设置 miter_limit 为 5.0，save 后改为 15.0，restore 后应恢复 5.0。
#[test]
fn test_miter_limit_save_restore_value() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_miter_limit(5.0);
    ctx.save();
    ctx.set_miter_limit(15.0);
    assert!(
        (ctx.miter_limit() - 15.0).abs() < f32::EPSILON,
        "after save+set, miter_limit should be 15.0"
    );
    ctx.restore();
    assert!(
        (ctx.miter_limit() - 5.0).abs() < f32::EPSILON,
        "after restore, miter_limit should be back to 5.0"
    );
}

/// 测试 direction 默认值为 Inherit。
///
/// 新创建的 CanvasContext 的 direction() 应返回 TextDirection::Inherit。
#[test]
fn test_text_direction_default_value() {
    let ctx = CanvasContext::new(100, 100);
    assert_eq!(
        ctx.direction(),
        TextDirection::Inherit,
        "default direction should be Inherit"
    );
}

// ── CanvasStyle 测试 ──

/// 测试 CanvasStyle 默认为不透明黑色。
#[test]
fn test_canvas_style_default() {
    let ctx = CanvasContext::new(100, 100);
    assert_eq!(ctx.fill_color(), Color::BLACK);
    assert_eq!(ctx.stroke_color(), Color::BLACK);
    // 验证 fill_style/stroke_style 是 Color 变体
    assert!(matches!(ctx.fill_style(), CanvasStyle::Color(Color::BLACK)));
    assert!(matches!(ctx.stroke_style(), CanvasStyle::Color(Color::BLACK)));
}

/// 测试设置 fill 为线性渐变后 resolve_color 返回插值颜色。
#[test]
fn test_fill_style_gradient() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 200.0, 0.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(1.0, Color::BLUE);
    ctx.set_fill_style(CanvasStyle::LinearGradient(grad));
    // resolve_color 在 offset=0.5 处采样，应为红色和蓝色的中间值
    let resolved = ctx.fill_color();
    // 中间色：(128, 0, 128, 255)
    assert_eq!(resolved.r, 128);
    assert_eq!(resolved.g, 0);
    assert_eq!(resolved.b, 128);
    assert_eq!(resolved.a, 255);
}

/// 测试设置 stroke 为线性渐变后 resolve_color 返回插值颜色。
#[test]
fn test_stroke_style_gradient() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 200.0, 0.0);
    grad.add_color_stop(0.0, Color::BLACK);
    grad.add_color_stop(1.0, Color::WHITE);
    ctx.set_stroke_style(CanvasStyle::LinearGradient(grad));
    let resolved = ctx.stroke_color();
    // 中间色：(128, 128, 128, 255)
    assert_eq!(resolved.r, 128);
    assert_eq!(resolved.g, 128);
    assert_eq!(resolved.b, 128);
    assert_eq!(resolved.a, 255);
}

/// 测试渐变只有一个停止点时 sample_color 返回该停止点的颜色。
#[test]
fn test_gradient_sample_single_stop() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
    grad.add_color_stop(0.5, Color::GREEN);
    assert_eq!(grad.sample_color(0.0), Color::GREEN);
    assert_eq!(grad.sample_color(0.5), Color::GREEN);
    assert_eq!(grad.sample_color(1.0), Color::GREEN);
}

/// 测试渐变两个停止点时 sample_color 在各位置返回正确的插值颜色。
#[test]
fn test_gradient_sample_two_stops() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
    grad.add_color_stop(0.0, Color::BLACK);
    grad.add_color_stop(1.0, Color::WHITE);
    // offset=0.0: 黑色
    assert_eq!(grad.sample_color(0.0), Color::BLACK);
    // offset=1.0: 白色
    assert_eq!(grad.sample_color(1.0), Color::WHITE);
    // offset=0.5: 中间灰
    let mid = grad.sample_color(0.5);
    assert_eq!(mid.r, 128);
    assert_eq!(mid.g, 128);
    assert_eq!(mid.b, 128);
    assert_eq!(mid.a, 255);
}

/// 测试渐变 sample_color 在偏移量超出 [0,1] 范围时 clamp 到边界停止点颜色。
#[test]
fn test_gradient_sample_out_of_range() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(1.0, Color::BLUE);
    // 负偏移量 → clamp 到 0.0 → 红色
    assert_eq!(grad.sample_color(-1.0), Color::RED);
    // 超过 1.0 → clamp 到 1.0 → 蓝色
    assert_eq!(grad.sample_color(2.0), Color::BLUE);
}

/// 测试 set_fill_color 便捷方法仍然正常工作。
#[test]
fn test_set_fill_color_convenience() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_fill_color(Color::RED);
    assert_eq!(ctx.fill_color(), Color::RED);
    assert!(matches!(ctx.fill_style(), CanvasStyle::Color(Color::RED)));
    // 填充矩形应使用红色
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    let pixel = ctx.get_image_data(5, 5, 1, 1);
    assert_eq!(pixel.data[0..4], [255, 0, 0, 255]);
}

/// 测试 set_stroke_color 便捷方法仍然正常工作。
#[test]
fn test_set_stroke_color_convenience() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_stroke_color(Color::BLUE);
    assert_eq!(ctx.stroke_color(), Color::BLUE);
    assert!(matches!(ctx.stroke_style(), CanvasStyle::Color(Color::BLUE)));
    // 描边矩形应使用蓝色
    ctx.stroke_rect(10.0, 10.0, 20.0, 20.0);
    let pixel = ctx.get_image_data(10, 10, 1, 1);
    assert_eq!(pixel.data[0..4], [0, 0, 255, 255]);
}

/// 测试径向渐变 sample_color。
#[test]
fn test_radial_gradient_sample_color() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_radial_gradient(50.0, 50.0, 10.0, 50.0, 50.0, 100.0);
    grad.add_color_stop(0.0, Color::WHITE);
    grad.add_color_stop(1.0, Color::BLACK);
    // offset=0: 白色
    assert_eq!(grad.sample_color(0.0), Color::WHITE);
    // offset=1: 黑色
    assert_eq!(grad.sample_color(1.0), Color::BLACK);
    // offset=0.5: 中间灰
    let mid = grad.sample_color(0.5);
    assert_eq!(mid.r, 128);
}

/// 测试锥形渐变 sample_color。
#[test]
fn test_conic_gradient_sample_color() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_conic_gradient(0.0, 50.0, 50.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(0.5, Color::GREEN);
    grad.add_color_stop(1.0, Color::BLUE);
    // offset=0: 红色
    assert_eq!(grad.sample_color(0.0), Color::RED);
    // offset=0.25: 红绿中间
    let c = grad.sample_color(0.25);
    assert_eq!(c.r, 128);
    assert_eq!(c.g, 128);
}

/// 测试 CanvasStyle Pattern 变体 resolve_color 返回黑色。
#[test]
fn test_canvas_style_pattern_resolve() {
    let img = ImageData {
        width: 2,
        height: 2,
        data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
    };
    let pattern = CanvasPattern::new(img, PatternRepetition::Repeat);
    let style = CanvasStyle::Pattern(pattern);
    assert_eq!(style.resolve_color(), Color::BLACK);
}

/// 测试 save/restore 正确保存和恢复 CanvasStyle（渐变）。
#[test]
fn test_save_restore_gradient_style() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 200.0, 0.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(1.0, Color::BLUE);
    ctx.set_fill_style(CanvasStyle::LinearGradient(grad));
    ctx.save();
    ctx.set_fill_style(CanvasStyle::Color(Color::GREEN));
    assert_eq!(ctx.fill_color(), Color::GREEN);
    ctx.restore();
    // 恢复后应回到渐变样式
    let resolved = ctx.fill_color();
    assert_eq!(resolved.r, 128);
    assert_eq!(resolved.b, 128);
}

/// 测试无停止点的渐变 sample_color 返回黑色。
#[test]
fn test_gradient_sample_empty_stops() {
    let ctx = CanvasContext::new(200, 200);
    let grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
    assert_eq!(grad.sample_color(0.5), Color::BLACK);
}

/// 测试使用渐变 fill_style 绘制 fill_rect（逐像素光栅化，R3079）。
#[test]
fn test_fill_rect_with_gradient_style() {
    let mut ctx = CanvasContext::new(100, 100);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(1.0, Color::BLUE);
    ctx.set_fill_style(CanvasStyle::LinearGradient(grad));
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    // R3079：fill_rect 现逐像素采样渐变（旧 resolve_color flat-midpoint 近似已废弃）。
    // red→blue 沿 x=0..100，rect 0..50：左端 t≈0 偏红，右沿（x≈48, t≈0.48）偏紫——r 递减、b 递增。
    let left = ctx.get_image_data(2, 25, 1, 1);
    let right = ctx.get_image_data(48, 25, 1, 1);
    assert!(
        left.data[0] > right.data[0],
        "渐变左→右：r 递减（{} → {}）",
        left.data[0],
        right.data[0]
    );
    assert!(
        left.data[2] < right.data[2],
        "渐变左→右：b 递增（{} → {}）",
        left.data[2],
        right.data[2]
    );
    assert_eq!(left.data[3], 255, "alpha 应为 255");
}

/// 测试使用渐变 stroke_style 绘制 stroke_rect。
#[test]
fn test_stroke_rect_with_gradient_style() {
    let mut ctx = CanvasContext::new(100, 100);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
    grad.add_color_stop(0.0, Color::BLACK);
    grad.add_color_stop(1.0, Color::WHITE);
    ctx.set_stroke_style(CanvasStyle::LinearGradient(grad));
    ctx.stroke_rect(5.0, 5.0, 20.0, 20.0);
    // 描边像素应使用 offset=0.5 处的采样色 (128, 128, 128)
    let pixel = ctx.get_image_data(5, 5, 1, 1);
    assert_eq!(pixel.data[0], 128);
    assert_eq!(pixel.data[1], 128);
    assert_eq!(pixel.data[2], 128);
    assert_eq!(pixel.data[3], 255);
}

// ── 边界条件测试：渐变填充、描边连接、嵌套裁剪、ImageData 往返、globalAlpha、Path2D ──

/// 测试 fill_rect 使用 CanvasStyle::LinearGradient 作为填充样式。
/// 渐变从红色到蓝色，采样 offset=0.5 处应得到 (128, 0, 128) 左右的颜色。
#[test]
fn test_canvas_fill_rect_with_gradient_style() {
    let mut ctx = CanvasContext::new(100, 100);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(1.0, Color::BLUE);
    ctx.set_fill_style(CanvasStyle::LinearGradient(grad));
    ctx.fill_rect(0.0, 0.0, 100.0, 100.0);
    // fill_rect 使用 fill_style.resolve_color() 采样 offset=0.5
    // 红(255,0,0) 与 蓝(0,0,255) 在 0.5 处插值 ≈ (128, 0, 128)
    let pixel = ctx.get_image_data(50, 50, 1, 1);
    assert!(
        (pixel.data[0] as i32 - 128).abs() <= 2,
        "red channel should be ~128, got {}",
        pixel.data[0]
    );
    assert_eq!(pixel.data[1], 0, "green channel should be 0");
    assert!(
        (pixel.data[2] as i32 - 128).abs() <= 2,
        "blue channel should be ~128, got {}",
        pixel.data[2]
    );
    assert_eq!(pixel.data[3], 255, "alpha should be 255");
}

/// 测试 stroke_rect 使用 LineJoin::Round 不 panic 且生成描边图元。
#[test]
fn test_stroke_rect_with_round_join() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_join(LineJoin::Round);
    ctx.set_line_width(5.0);
    ctx.stroke_rect(10.0, 10.0, 30.0, 30.0);
    // stroke_rect 生成 4 条边的 fill 图元
    assert_eq!(ctx.primitives().fills.len(), 4);
    assert_eq!(ctx.line_join(), LineJoin::Round);
}

/// 测试嵌套 clip 操作：先裁剪到大矩形，再裁剪到小矩形，最终绘制范围应受限于交集。
#[test]
fn test_canvas_clip_nested() {
    let mut ctx = CanvasContext::new(200, 200);
    // 第一次裁剪：大矩形
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(100.0, 0.0);
    ctx.line_to(100.0, 100.0);
    ctx.line_to(0.0, 100.0);
    ctx.close_path();
    ctx.clip();
    assert_eq!(ctx.primitives().clips.len(), 1);
    // 第二次裁剪：小矩形（嵌套）
    ctx.begin_path();
    ctx.move_to(20.0, 20.0);
    ctx.line_to(60.0, 20.0);
    ctx.line_to(60.0, 60.0);
    ctx.line_to(20.0, 60.0);
    ctx.close_path();
    ctx.clip();
    assert_eq!(ctx.primitives().clips.len(), 2);
    // 后续绘制应受限于两个裁剪区域的交集
    ctx.fill_rect(0.0, 0.0, 200.0, 200.0);
    assert_eq!(ctx.primitives().fills.len(), 1);
}

/// 测试 put_image_data 后 get_image_data 能完整读回相同数据（往返一致性）。
#[test]
fn test_put_get_image_data_roundtrip() {
    let mut ctx = CanvasContext::new(20, 20);
    // 构造 4x4 的测试像素：每个像素不同的 RGBA 值
    let mut pixels = Vec::with_capacity(4 * 4 * 4);
    for i in 0u8..16 {
        pixels.extend_from_slice(&[i * 16, (255 - i * 16), i * 8, 255]);
    }
    let img = ImageData {
        width: 4,
        height: 4,
        data: pixels.clone(),
    };
    ctx.put_image_data(&img, 5, 5);
    let result = ctx.get_image_data(5, 5, 4, 4);
    assert_eq!(
        result.data, pixels,
        "get_image_data 应返回与 put_image_data 写入完全相同的数据"
    );
}

/// 测试 globalAlpha=0 时 fill_rect 产生完全透明的像素。
#[test]
fn test_global_alpha_zero() {
    let mut ctx = CanvasContext::new(50, 50);
    ctx.set_global_alpha(0.0);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    // 填充颜色的 alpha 应被 globalAlpha=0 清零
    let pixel = ctx.get_image_data(25, 25, 1, 1);
    assert_eq!(pixel.data[3], 0, "globalAlpha=0 时像素应完全透明");
    // 图元颜色 alpha 也应为 0
    let fill = &ctx.primitives().fills[0];
    assert_eq!(fill.color.a, 0, "图元颜色 alpha 应为 0");
}

// ── 边界条件测试（第八批）──

/// 测试 resize 到零尺寸后画布宽度高度为零，且不 panic。
#[test]
fn test_canvas_resize_to_zero() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    ctx.resize(0, 0);
    assert_eq!(ctx.width(), 0);
    assert_eq!(ctx.height(), 0);
}

/// 测试 fill_rect 零宽高不产生可见像素。
#[test]
fn test_canvas_fill_rect_zero_dimensions() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(10.0, 10.0, 0.0, 0.0);
    ctx.fill_rect(20.0, 20.0, 0.0, 50.0);
    ctx.fill_rect(30.0, 30.0, 50.0, 0.0);
    // 零宽/高的矩形不应写入像素
    let pixel = ctx.get_image_data(10, 10, 1, 1);
    assert_eq!(pixel.data[0..4], [0, 0, 0, 0], "零宽高 fill_rect 不应写入像素");
}

/// 测试 set_line_dash 传入单元素数组后自动加倍为双元素。
#[test]
fn test_line_dash_single_element_doubled() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_dash(vec![8.0]);
    // 奇数长度时 Canvas 规范要求复制拼接：[8] → [8, 8]
    assert_eq!(ctx.get_line_dash(), &[8.0, 8.0]);
}

/// 测试 stroke_rect 零尺寸（零宽零高）只生成 4 个退化图元且不 panic。
#[test]
fn test_canvas_stroke_rect_zero_size() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.stroke_rect(50.0, 50.0, 0.0, 0.0);
    // stroke_rect 始终生成 4 条边的 fill 图元，即使零尺寸
    assert_eq!(ctx.primitives().fills.len(), 4);
}

/// 测试深度嵌套 save/restore 正确恢复每一层状态。
#[test]
fn test_canvas_deep_nested_save_restore() {
    let mut ctx = CanvasContext::new(100, 100);
    // 层级 0: 红色填充
    ctx.set_fill_color(Color::RED);
    ctx.save();
    // 层级 1: 绿色填充
    ctx.set_fill_color(Color::GREEN);
    ctx.save();
    // 层级 2: 蓝色填充
    ctx.set_fill_color(Color::BLUE);
    ctx.save();
    // 层级 3: 白色填充
    ctx.set_fill_color(Color::WHITE);
    assert_eq!(ctx.fill_color(), Color::WHITE);

    // 逐层恢复
    ctx.restore();
    assert_eq!(ctx.fill_color(), Color::BLUE, "恢复到层级 2 应为蓝色");
    ctx.restore();
    assert_eq!(ctx.fill_color(), Color::GREEN, "恢复到层级 1 应为绿色");
    ctx.restore();
    assert_eq!(ctx.fill_color(), Color::RED, "恢复到层级 0 应为红色");
}

// ── 边界条件测试（第九批）──

/// 测试 resize 到更大尺寸后画布像素缓冲区重新分配，之前内容被清空。
#[test]
fn test_canvas_resize_larger_clears_pixels() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    // 确认红色已写入
    let before = ctx.get_image_data(0, 0, 5, 5);
    assert_eq!(before.data[0], 255);
    // resize 到更大尺寸
    ctx.resize(20, 20);
    assert_eq!(ctx.width(), 20);
    assert_eq!(ctx.height(), 20);
    // resize 后像素应全部清零
    let after = ctx.get_image_data(0, 0, 5, 5);
    assert_eq!(after.data[0..4], [0, 0, 0, 0], "resize 后像素应被清零");
}

/// 测试 set_fill_style 使用径向渐变后 fill_rect 逐像素光栅化（R3079）。
#[test]
fn test_fill_rect_with_radial_gradient_style() {
    let mut ctx = CanvasContext::new(100, 100);
    let mut grad = ctx.create_radial_gradient(50.0, 50.0, 0.0, 50.0, 50.0, 50.0);
    grad.add_color_stop(0.0, Color::WHITE);
    grad.add_color_stop(1.0, Color::BLACK);
    ctx.set_fill_style(CanvasStyle::RadialGradient(grad));
    ctx.fill_rect(0.0, 0.0, 100.0, 100.0);
    // R3079：径向渐变逐像素采样（旧 resolve_color flat-midpoint 近似已废弃）。
    // 圆心 (50,50) t=0 白；角点 (0,0) dist≈70 > r1=50 → t=1 黑（spec 钳制到末停止点）。
    let center = ctx.get_image_data(50, 50, 1, 1);
    let corner = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(center.data[0], 255, "圆心 t=0 白");
    assert_eq!(center.data[3], 255, "alpha 应为 255");
    assert_eq!(corner.data[0], 0, "角点 dist>r1 → t=1 黑");
}

/// 测试 set_stroke_style 使用锥形渐变后 stroke_rect 像素使用采样颜色。
#[test]
fn test_stroke_rect_with_conic_gradient_style() {
    let mut ctx = CanvasContext::new(100, 100);
    let mut grad = ctx.create_conic_gradient(0.0, 50.0, 50.0);
    grad.add_color_stop(0.0, Color::BLACK);
    grad.add_color_stop(1.0, Color::WHITE);
    ctx.set_stroke_style(CanvasStyle::ConicGradient(grad));
    ctx.stroke_rect(5.0, 5.0, 20.0, 20.0);
    // stroke_rect 使用 resolve_color()，ConicGradient 在 offset=0.0 处采样
    // offset=0.0 对应第一个 stop 的颜色：黑色
    let pixel = ctx.get_image_data(5, 5, 1, 1);
    assert_eq!(pixel.data[0], 0, "conic gradient sample at 0.0 应为黑色");
    assert_eq!(pixel.data[1], 0);
    assert_eq!(pixel.data[2], 0);
    assert_eq!(pixel.data[3], 255);
}

/// 测试 scale(0, 0) 后 fill_rect 不 panic 且变换结果退化。
#[test]
fn test_canvas_scale_zero_no_panic() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.scale(0.0, 0.0);
    // scale(0,0) 使变换矩阵退化为全零平移，fill_rect 应不 panic
    ctx.fill_rect(10.0, 10.0, 50.0, 50.0);
    // 退化矩阵下 transform_point 产生的矩形宽高为 0，不应写入像素
    let pixel = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(pixel.data[0..4], [0, 0, 0, 0], "退化矩阵不应写入像素");
}

/// 测试 measure_text 在 set_font 改变字体大小后返回不同的宽度值。
#[test]
fn test_measure_text_reflects_font_size_change() {
    let mut ctx = CanvasContext::new(100, 100);
    // 默认字体大小 10.0
    let m1 = ctx.measure_text("abc");
    let expected1 = 3.0 * 10.0 * 0.6; // 18.0
    assert!(
        (m1.width - expected1).abs() < f32::EPSILON,
        "默认大小 10.0 时宽度应为 {}",
        expected1
    );
    // 改为 20.0
    ctx.set_font(FontDescriptor {
        family: "sans-serif".to_string(),
        size: 20.0,
        weight: FontWeight::Normal,
        style: FontStyle::Normal,
    });
    let m2 = ctx.measure_text("abc");
    let expected2 = 3.0 * 20.0 * 0.6; // 36.0
    assert!(
        (m2.width - expected2).abs() < f32::EPSILON,
        "字体大小 20.0 时宽度应为 {}",
        expected2
    );
    // 两次测量应不同
    assert!((m1.width - m2.width).abs() > 1.0, "不同字体大小的测量结果应不同");
}

// ── 边界条件测试（第十批）──

/// 测试 clear_rect 使用负坐标和负尺寸时不 panic，且不破坏已有像素。
/// 负尺寸的 clear_rect 应视为空操作（不清除任何像素）。
#[test]
fn test_canvas_clear_rect_negative_dimensions() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 100.0, 100.0);
    // 负宽高的 clear_rect — 不应 panic
    ctx.clear_rect(10.0, 10.0, -20.0, -30.0);
    // 原有红色像素应保持不变
    let pixel = ctx.get_image_data(50, 50, 1, 1);
    assert_eq!(pixel.data[0..4], [255, 0, 0, 255], "负尺寸 clear_rect 不应破坏已有像素");
}

/// 测试 CanvasStyle::Pattern 作为 fill_style 绘制 fill_rect 时使用黑色回退色。
/// Pattern 的 resolve_color() 返回黑色，因此 fill_rect 应使用黑色绘制。
#[test]
fn test_canvas_fill_rect_with_pattern_style() {
    let mut ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 2,
        height: 2,
        data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
    };
    let pattern = ctx.create_pattern(img, PatternRepetition::Repeat);
    ctx.set_fill_style(CanvasStyle::Pattern(pattern));
    ctx.fill_rect(10.0, 10.0, 30.0, 30.0);
    // Pattern resolve_color 回退为黑色
    let pixel = ctx.get_image_data(20, 20, 1, 1);
    assert_eq!(pixel.data[0], 0, "pattern fill 应使用黑色回退色 r");
    assert_eq!(pixel.data[1], 0, "pattern fill 应使用黑色回退色 g");
    assert_eq!(pixel.data[2], 0, "pattern fill 应使用黑色回退色 b");
    assert_eq!(pixel.data[3], 255, "pattern fill alpha 应为 255");
}

/// 测试三色渐变在中间停止点偏移量处精确返回该停止点的颜色（无插值误差）。
#[test]
fn test_gradient_sample_three_stops_exact_boundary() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(0.5, Color::GREEN);
    grad.add_color_stop(1.0, Color::BLUE);
    // 在偏移量 0.0 处应为红色
    assert_eq!(grad.sample_color(0.0), Color::RED, "offset 0.0 应为红色");
    // 在偏移量 0.5 处应为绿色（精确命中停止点，无需插值）
    assert_eq!(grad.sample_color(0.5), Color::GREEN, "offset 0.5 应为绿色");
    // 在偏移量 1.0 处应为蓝色
    assert_eq!(grad.sample_color(1.0), Color::BLUE, "offset 1.0 应为蓝色");
}

/// 测试 save/restore 保存并恢复 text_align 和 text_baseline。
/// save 后修改文本对齐和基线，restore 后应恢复到 save 时的值。
#[test]
fn test_text_align_and_baseline_save_restore() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_text_align(TextAlign::Right);
    ctx.set_text_baseline(TextBaseline::Top);
    ctx.save();
    ctx.set_text_align(TextAlign::Center);
    ctx.set_text_baseline(TextBaseline::Bottom);
    assert_eq!(ctx.text_align(), TextAlign::Center);
    assert_eq!(ctx.text_baseline(), TextBaseline::Bottom);
    ctx.restore();
    assert_eq!(ctx.text_align(), TextAlign::Right, "restore 后 text_align 应为 Right");
    assert_eq!(
        ctx.text_baseline(),
        TextBaseline::Top,
        "restore 后 text_baseline 应为 Top"
    );
}

/// 测试 Path2D 连续添加多种子路径命令后 len() 正确递增。
/// 依次添加 move_to、line_to、quadratic_curve_to、bezier_curve_to、arc、close_path，
/// 验证每步后的命令数量。
#[test]
fn test_path2d_mixed_commands_count() {
    let mut p = Path2D::new();
    assert_eq!(p.len(), 0, "空路径应有 0 个命令");

    p.move_to(10.0, 20.0);
    assert_eq!(p.len(), 1, "move_to 后应有 1 个命令");

    p.line_to(30.0, 40.0);
    assert_eq!(p.len(), 2, "line_to 后应有 2 个命令");

    p.quadratic_curve_to(50.0, 60.0, 70.0, 80.0);
    assert_eq!(p.len(), 3, "quadratic_curve_to 后应有 3 个命令");

    p.bezier_curve_to(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
    assert_eq!(p.len(), 4, "bezier_curve_to 后应有 4 个命令");

    p.arc(0.0, 0.0, 10.0, 0.0, std::f32::consts::PI);
    assert_eq!(p.len(), 5, "arc 后应有 5 个命令");

    p.close_path();
    assert_eq!(p.len(), 6, "close_path 后应有 6 个命令");

    // 验证各命令类型正确
    assert!(matches!(p.commands()[0], PathCommand::MoveTo(10.0, 20.0)));
    assert!(matches!(p.commands()[1], PathCommand::LineTo(30.0, 40.0)));
    assert!(matches!(
        p.commands()[2],
        PathCommand::QuadraticCurveTo(50.0, 60.0, 70.0, 80.0)
    ));
    assert!(matches!(
        p.commands()[3],
        PathCommand::BezierCurveTo(1.0, 2.0, 3.0, 4.0, 5.0, 6.0)
    ));
    assert!(matches!(p.commands()[4], PathCommand::Arc(0.0, 0.0, 10.0, 0.0, _)));
    assert!(matches!(p.commands()[5], PathCommand::ClosePath));
}

// ── 边界条件测试（第十一批）──

/// 测试 miter_limit 设置/获取和 save/restore。
/// 默认值为 10.0，修改后 getter 应返回新值，restore 后恢复。
#[test]
fn test_miter_limit_set_get_and_save_restore() {
    let ctx = CanvasContext::new(100, 100);
    // 默认值为 10.0
    assert!(
        (ctx.miter_limit() - 10.0).abs() < f32::EPSILON,
        "miter_limit 默认应为 10.0"
    );

    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_miter_limit(5.0);
    assert!(
        (ctx.miter_limit() - 5.0).abs() < f32::EPSILON,
        "设置后 miter_limit 应为 5.0"
    );

    // save/restore 保存并恢复 miter_limit
    ctx.save();
    ctx.set_miter_limit(2.0);
    assert!(
        (ctx.miter_limit() - 2.0).abs() < f32::EPSILON,
        "save 后修改 miter_limit 应为 2.0"
    );
    ctx.restore();
    assert!(
        (ctx.miter_limit() - 5.0).abs() < f32::EPSILON,
        "restore 后 miter_limit 应恢复为 5.0"
    );
}

/// 测试 direction（文本方向）设置/获取和 save/restore。
/// 默认值为 TextDirection::Inherit，修改后 getter 应返回新值，restore 后恢复。
#[test]
fn test_direction_set_get_and_save_restore() {
    let ctx = CanvasContext::new(100, 100);
    assert_eq!(ctx.direction(), TextDirection::Inherit, "direction 默认应为 Inherit");

    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_direction(TextDirection::Rtl);
    assert_eq!(ctx.direction(), TextDirection::Rtl, "设置后 direction 应为 Rtl");

    // save/restore
    ctx.save();
    ctx.set_direction(TextDirection::Ltr);
    assert_eq!(ctx.direction(), TextDirection::Ltr, "save 后修改 direction 应为 Ltr");
    ctx.restore();
    assert_eq!(ctx.direction(), TextDirection::Rtl, "restore 后 direction 应恢复为 Rtl");
}

/// 测试 transform() 方法（矩阵后乘）与 set_transform 的区别。
/// transform() 是叠加乘法，set_transform() 是替换。
#[test]
fn test_transform_method_accumulates_vs_set_transform_replaces() {
    let mut ctx = CanvasContext::new(100, 100);
    // 使用 transform() 叠加两个平移
    ctx.transform(1.0, 0.0, 0.0, 1.0, 10.0, 0.0); // 平移 x+10
    ctx.transform(1.0, 0.0, 0.0, 1.0, 20.0, 0.0); // 平移 x+20（叠加）
    let p1 = ctx.transform.transform_point(0.0, 0.0);
    // 叠加后应平移 30.0
    assert!((p1.0 - 30.0).abs() < 0.01, "叠加两次平移后 x 应为 30.0，实际 {}", p1.0);

    // 使用 set_transform 替换（非叠加）
    ctx.set_transform(1.0, 0.0, 0.0, 1.0, 5.0, 0.0);
    let p2 = ctx.transform.transform_point(0.0, 0.0);
    assert!(
        (p2.0 - 5.0).abs() < 0.01,
        "set_transform 替换后 x 应为 5.0，实际 {}",
        p2.0
    );
}

/// 测试 fill_text 在不同字体大小时 glyph x 坐标按字体大小正确递进。
/// 默认字体大小 10.0 时 em_width = 6.0；改为 20.0 时 em_width = 12.0。
#[test]
fn test_fill_text_glyph_offset_scales_with_font_size() {
    // 字体大小 10.0（默认）
    let mut ctx_small = CanvasContext::new(200, 200);
    ctx_small.fill_text("AB", 0.0, 0.0);
    let glyphs_small = &ctx_small.primitives().glyphs;
    // 第二个字符的 x 应为 0.0 + 10.0 * 0.6 = 6.0
    assert!(
        (glyphs_small[1].x - 6.0).abs() < f32::EPSILON,
        "字体大小 10.0 时第二个 glyph x 应为 6.0，实际 {}",
        glyphs_small[1].x
    );

    // 字体大小 20.0
    let mut ctx_large = CanvasContext::new(200, 200);
    ctx_large.set_font(FontDescriptor {
        family: "sans-serif".to_string(),
        size: 20.0,
        weight: FontWeight::Normal,
        style: FontStyle::Normal,
    });
    ctx_large.fill_text("AB", 0.0, 0.0);
    let glyphs_large = &ctx_large.primitives().glyphs;
    // 第二个字符的 x 应为 0.0 + 20.0 * 0.6 = 12.0
    assert!(
        (glyphs_large[1].x - 12.0).abs() < f32::EPSILON,
        "字体大小 20.0 时第二个 glyph x 应为 12.0，实际 {}",
        glyphs_large[1].x
    );
}

/// 测试扫描线光栅化对三角形路径填充写入正确的像素。
/// 在三角形重心附近的像素应被写入填充色，三角形外部的像素应保持透明。
#[test]
fn test_scanline_rasterization_triangle_fill_pixels() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_fill_color(Color::RED);
    ctx.begin_path();
    // 等腰三角形：顶点 (30,10), (50,50), (10,50)
    ctx.move_to(30.0, 10.0);
    ctx.line_to(50.0, 50.0);
    ctx.line_to(10.0, 50.0);
    ctx.close_path();
    ctx.fill();

    // 三角形重心 ≈ (30, 36.7) — 应为红色
    let center = ctx.get_image_data(30, 36, 1, 1);
    assert_eq!(center.data[0..4], [255, 0, 0, 255], "三角形重心处应为红色");

    // 三角形外部 (80, 10) — 应为透明
    let outside = ctx.get_image_data(80, 10, 1, 1);
    assert_eq!(outside.data[0..4], [0, 0, 0, 0], "三角形外部应为透明");

    // 三角形下方 (30, 80) — 应为透明（低于底边）
    let below = ctx.get_image_data(30, 80, 1, 1);
    assert_eq!(below.data[0..4], [0, 0, 0, 0], "三角形底边下方应为透明");
}

// ── 边界条件测试（第十二批）──

/// 测试 Transform2D 旋转 2π（360°）后应近似回到单位矩阵。
/// 由于浮点精度，各元素与单位矩阵之差应极小（< 0.001）。
#[test]
fn test_transform_rotate_full_circle() {
    let rot = Transform2D::rotate(std::f32::consts::TAU); // 2π
    assert!((rot.a - 1.0).abs() < 0.001, "旋转 2π 后 a 应近似 1.0");
    assert!(rot.b.abs() < 0.001, "旋转 2π 后 b 应近似 0.0");
    assert!(rot.c.abs() < 0.001, "旋转 2π 后 c 应近似 0.0");
    assert!((rot.d - 1.0).abs() < 0.001, "旋转 2π 后 d 应近似 1.0");
    assert!(rot.e.abs() < f32::EPSILON, "旋转 2π 后 e 应为 0.0");
    assert!(rot.f.abs() < f32::EPSILON, "旋转 2π 后 f 应为 0.0");
}

/// 测试 ImageData clone 后修改克隆副本不影响原始数据。
/// 克隆一份包含非零像素的 ImageData，修改克隆副本的数据，验证原始数据不变。
#[test]
fn test_image_data_clone_independence() {
    let original = ImageData {
        width: 2,
        height: 2,
        data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
    };
    let mut cloned = original.clone();
    // 修改克隆副本的第一个像素为黑色
    cloned.data[0] = 0;
    cloned.data[1] = 0;
    cloned.data[2] = 0;
    cloned.data[3] = 0;
    // 原始数据的第一个像素应保持不变（红色）
    assert_eq!(original.data[0..4], [255, 0, 0, 255], "原始数据应不受克隆修改的影响");
    assert_eq!(cloned.data[0..4], [0, 0, 0, 0], "克隆副本应反映修改");
}

/// 测试连续两次 begin_path 调用后当前路径被正确清空。
/// 第一次 begin_path 前添加路径命令，第一次 begin_path 清空，
/// 再添加新命令，第二次 begin_path 再次清空，fill 应不产生图元。
#[test]
fn test_double_begin_path_clears() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(50.0, 50.0);
    // 第二次 begin_path 应清空当前路径
    ctx.begin_path();
    // 空路径 fill 应不产生填充图元
    ctx.fill();
    assert!(
        ctx.primitives().fills.is_empty(),
        "第二次 begin_path 后 fill 空路径不应产生填充图元"
    );
}

/// 测试 save/restore 对多次连续 save 的后进先出（LIFO）行为。
/// 依次 save 红色、绿色、蓝色，restore 后应按蓝色→绿色→红色顺序恢复。
#[test]
fn test_save_restore_lifo_order() {
    let mut ctx = CanvasContext::new(100, 100);
    // 层级 0：红色
    ctx.set_fill_color(Color::RED);
    ctx.save();
    // 层级 1：绿色
    ctx.set_fill_color(Color::GREEN);
    ctx.save();
    // 层级 2：蓝色
    ctx.set_fill_color(Color::BLUE);
    assert_eq!(ctx.fill_color(), Color::BLUE, "当前应为蓝色");

    // 第一次 restore：恢复到绿色
    ctx.restore();
    assert_eq!(ctx.fill_color(), Color::GREEN, "LIFO 第一次 restore 应恢复到绿色");

    // 第二次 restore：恢复到红色
    ctx.restore();
    assert_eq!(ctx.fill_color(), Color::RED, "LIFO 第二次 restore 应恢复到红色");
}

/// 测试设置负的 line_width 不 panic，且后续操作正常。
/// 虽然负线宽在 Canvas 规范中被忽略，但不应导致 panic。
#[test]
fn test_negative_line_width_no_panic() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_width(-5.0);
    // 负线宽设置后 getter 应返回设置的值
    assert!(
        (ctx.line_width() - (-5.0)).abs() < f32::EPSILON,
        "line_width getter 应返回设置值 -5.0"
    );
    // 描边矩形不应 panic
    ctx.set_stroke_color(Color::RED);
    ctx.stroke_rect(10.0, 10.0, 30.0, 30.0);
    // 验证描边图元已生成（即使线宽为负）
    assert!(!ctx.primitives().fills.is_empty(), "负线宽描边矩形仍应生成图元");
}

// ── 边界条件测试（第十三批）──

/// 测试 createRadialGradient 使用完全相同的内外圆（圆心和半径均相同）时不 panic。
/// 退化渐变应正常创建，停止点可正常添加。
#[test]
fn test_radial_gradient_identical_circles_no_panic() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_radial_gradient(50.0, 50.0, 25.0, 50.0, 50.0, 25.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(1.0, Color::BLUE);
    // 退化渐变不 panic，停止点数量正确
    assert_eq!(grad.stops.len(), 2);
    assert!((grad.x0 - 50.0).abs() < f32::EPSILON);
    assert!((grad.y0 - 50.0).abs() < f32::EPSILON);
    assert!((grad.r0 - 25.0).abs() < f32::EPSILON);
    assert!((grad.x1 - 50.0).abs() < f32::EPSILON);
    assert!((grad.y1 - 50.0).abs() < f32::EPSILON);
    assert!((grad.r1 - 25.0).abs() < f32::EPSILON);
}

/// 测试在路径构建过程中 resize 画布后路径仍然保留。
/// begin_path → move_to → line_to → resize → 更多路径操作 → fill，
/// resize 只清除像素缓冲区和已渲染图元，不清除当前路径。
#[test]
fn test_resize_during_path_construction_preserves_path() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(50.0, 10.0);
    // resize 会清除 pixel_buffer 和 primitives，但 current_path 不受影响
    ctx.resize(200, 200);
    // 继续路径操作
    ctx.line_to(50.0, 50.0);
    ctx.line_to(10.0, 50.0);
    ctx.close_path();
    ctx.fill();
    // 路径应保留，fill 应产生 path_fills 图元
    assert!(
        !ctx.primitives().path_fills.is_empty(),
        "resize 后路径应保留，fill 应产生填充图元"
    );
}

/// 测试 set_transform 使用全零退化矩阵时不 panic。
/// 全零矩阵将所有点映射到原点，reset_transform 后恢复正常绘制。
#[test]
fn test_set_transform_degenerate_all_zeros_then_reset() {
    let mut ctx = CanvasContext::new(100, 100);
    // 设置全零退化矩阵
    ctx.set_transform(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let (x, y) = ctx.transform.transform_point(50.0, 50.0);
    assert!((x).abs() < f32::EPSILON, "退化矩阵应将所有点映射到原点 x=0");
    assert!((y).abs() < f32::EPSILON, "退化矩阵应将所有点映射到原点 y=0");
    // 退化矩阵下 fill_rect 不 panic
    ctx.fill_rect(10.0, 10.0, 30.0, 30.0);
    // reset_transform 恢复单位矩阵后绘制正常
    ctx.reset_transform();
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    let pixel = ctx.get_image_data(5, 5, 1, 1);
    assert_eq!(pixel.data[0..4], [255, 0, 0, 255], "reset_transform 后应正常绘制");
}

/// 测试 put_image_data 使用超出 ImageData 数据范围的偏移参数时不 panic。
/// 将小尺寸 ImageData 放置到远超其范围的坐标上，不应导致越界访问。
#[test]
fn test_put_image_data_out_of_bounds_dirty_rect_no_panic() {
    let mut ctx = CanvasContext::new(50, 50);
    // 2x2 的 ImageData
    let img = ImageData {
        width: 2,
        height: 2,
        data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
    };
    // 放置在远超画布边界的位置 — 不应 panic
    ctx.put_image_data(&img, 100, 100);
    ctx.put_image_data(&img, 200, 200);
    ctx.put_image_data(&img, u32::MAX, u32::MAX);
    // 画布内像素应未被修改
    let pixel = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(pixel.data[0..4], [0, 0, 0, 0], "越界 put_image_data 不应写入画布内像素");
}

/// 测试 createLinearGradient 起点和终点完全相同（零长度渐变）时不 panic。
/// 零长度渐变应正常创建，可添加停止点，sample_color 不 panic。
#[test]
fn test_linear_gradient_zero_length_no_panic() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(75.0, 75.0, 75.0, 75.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(0.5, Color::GREEN);
    grad.add_color_stop(1.0, Color::BLUE);
    // 零长度渐变不 panic，停止点数量正确
    assert_eq!(grad.stops.len(), 3);
    // sample_color 不应 panic
    let c = grad.sample_color(0.5);
    assert_eq!(c, Color::GREEN, "零长度渐变在 offset=0.5 处应返回绿色");
}

/// 测试 restore 在没有对应 save 时不 panic。
/// Canvas 规范要求多余的 restore 静默忽略，不应导致崩溃。
#[test]
fn test_restore_without_matching_save_no_panic() {
    let mut ctx = CanvasContext::new(100, 100);
    // 栈为空时调用 restore — 不应 panic
    ctx.restore();
    ctx.restore();
    ctx.restore();
    // 画布状态应保持默认值不变
    assert_eq!(ctx.global_alpha(), 1.0, "多余 restore 不应改变 global_alpha");
    assert_eq!(ctx.line_width(), 1.0, "多余 restore 不应改变 line_width");
}

/// 测试 fill_text 传入空字符串时不 panic。
/// 空字符串没有字符需要渲染，应正常处理而不产生任何图元。
#[test]
fn test_fill_text_empty_string_no_panic() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.fill_text("", 10.0, 20.0);
    // 空字符串不应产生任何图元
    assert!(
        ctx.primitives().glyphs.is_empty(),
        "空字符串 fill_text 不应产生 glyph 图元"
    );
}

/// 测试 stroke_rect 使用零宽高时不 panic。
/// 零尺寸矩形的描边边框在数学上退化为点或线，实现应安全处理。
#[test]
fn test_stroke_rect_zero_width_height_no_panic() {
    let mut ctx = CanvasContext::new(100, 100);
    // 零宽高 — 不应 panic
    ctx.stroke_rect(50.0, 50.0, 0.0, 0.0);
    // 零宽度非零高度
    ctx.stroke_rect(25.0, 25.0, 0.0, 40.0);
    // 非零宽度零高度
    ctx.stroke_rect(25.0, 25.0, 40.0, 0.0);
}

/// 测试 create_radial_gradient 使用负半径（退化渐变）时不 panic。
/// 负半径在数学上无意义，但应能正常创建对象并添加停止点。
#[test]
fn test_radial_gradient_negative_radius_degenerate_no_panic() {
    let ctx = CanvasContext::new(100, 100);
    let mut grad = ctx.create_radial_gradient(50.0, 50.0, -10.0, 50.0, 50.0, -20.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(1.0, Color::BLUE);
    // 负半径渐变不 panic，停止点正确保存
    assert_eq!(grad.stops.len(), 2, "负半径渐变应能正常添加停止点");
    assert_eq!(grad.r0, -10.0, "内圆半径应保持负值不变");
    assert_eq!(grad.r1, -20.0, "外圆半径应保持负值不变");
    // sample_color 不应 panic（RED 和 BLUE 在 0.5 处插值为 (128,0,128)）
    let c = grad.sample_color(0.5);
    assert_eq!(c, Color::rgba(128, 0, 128, 255), "负半径渐变采样应返回正确插值颜色");
}

/// 测试 clip 在没有当前路径（空裁剪）时不 panic。
/// 没有构建路径时直接调用 clip，应静默忽略而非崩溃。
#[test]
fn test_clip_no_current_path_no_panic() {
    let mut ctx = CanvasContext::new(100, 100);
    // 没有构建任何路径，直接 clip — 不应 panic
    ctx.clip();
    ctx.clip();
    // 后续绘制操作应正常执行（裁剪区域未被设置）
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    // 画布内像素应有正常绘制结果
    let pixel = ctx.get_image_data(5, 5, 1, 1);
    assert_eq!(pixel.data[0..4], [255, 0, 0, 255], "空 clip 后 fill_rect 应正常绘制");
}

// ── 新增边界条件测试（5 个） ──

/// 测试 save/restore 在嵌套场景下正确保持 line_width 状态。
/// 外层设置 line_width=3，save 后改为 8，内层再 save 后改为 20，
/// 逐层 restore 后应依次恢复到 8 和 3。
#[test]
fn test_canvas_save_restore_line_width_nested() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_width(3.0);
    ctx.save();
    ctx.set_line_width(8.0);
    ctx.save();
    ctx.set_line_width(20.0);
    assert!(
        (ctx.line_width() - 20.0).abs() < f32::EPSILON,
        "内层 line_width 应为 20.0"
    );
    ctx.restore();
    assert!(
        (ctx.line_width() - 8.0).abs() < f32::EPSILON,
        "恢复到中层 line_width 应为 8.0"
    );
    ctx.restore();
    assert!(
        (ctx.line_width() - 3.0).abs() < f32::EPSILON,
        "恢复到外层 line_width 应为 3.0"
    );
}

/// 测试连续多次 arc() 调用不会 panic，且路径命令正确累积。
/// 模拟绘制一个由多段弧线组成的复杂路径场景。
#[test]
fn test_canvas_multiple_arc_paths() {
    let mut ctx = CanvasContext::new(400, 400);
    ctx.begin_path();
    ctx.arc(100.0, 100.0, 50.0, 0.0, std::f32::consts::PI);
    ctx.arc(200.0, 100.0, 30.0, 0.0, std::f32::consts::FRAC_PI_2);
    ctx.arc(300.0, 100.0, 40.0, 0.0, std::f32::consts::TAU);
    // 不应 panic
    ctx.fill();
    assert!(
        !ctx.primitives().path_fills.is_empty(),
        "多次 arc 后 fill 应生成路径填充图元"
    );
}

/// 测试 resize 到 0x0 尺寸不 panic，后续操作仍可安全执行。
#[test]
fn test_canvas_zero_size_resize() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    // resize 到零尺寸 — 不应 panic
    ctx.resize(0, 0);
    assert_eq!(ctx.width(), 0);
    assert_eq!(ctx.height(), 0);
    // 零尺寸画布上的绘制操作也不应 panic
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    ctx.stroke_rect(0.0, 0.0, 10.0, 10.0);
}

/// 测试对空路径同时调用 fill() 和 stroke() 均不 panic，且不生成任何图元。
#[test]
fn test_canvas_fill_stroke_empty_path() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    // 空路径上调用 fill 和 stroke — 不应 panic
    ctx.fill();
    ctx.stroke();
    assert_eq!(ctx.primitives().path_fills.len(), 0, "空路径 fill 不应生成图元");
    assert_eq!(ctx.primitives().path_strokes.len(), 0, "空路径 stroke 不应生成图元");
}

/// 测试 set_global_alpha 在边界值 0.0 和 1.0 时的行为正确。
/// 0.0 应使后续绘制完全透明，1.0 应保持完全不透明。
#[test]
fn test_canvas_global_alpha_boundary() {
    let mut ctx = CanvasContext::new(100, 100);
    // 设置 alpha 为 0.0 — 完全透明
    ctx.set_global_alpha(0.0);
    assert!(ctx.global_alpha().abs() < f32::EPSILON, "alpha=0.0 应精确返回 0.0");
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    let pixel_zero = ctx.get_image_data(25, 25, 1, 1);
    assert_eq!(pixel_zero.data[3], 0, "alpha=0.0 时绘制的像素应完全透明");

    // 设置 alpha 为 1.0 — 完全不透明
    ctx.set_global_alpha(1.0);
    assert!(
        (ctx.global_alpha() - 1.0).abs() < f32::EPSILON,
        "alpha=1.0 应精确返回 1.0"
    );
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    let pixel_one = ctx.get_image_data(25, 25, 1, 1);
    assert_eq!(
        pixel_one.data[0..4],
        [255, 0, 0, 255],
        "alpha=1.0 时绘制的像素应完全不透明"
    );
}

// ── 新增边界测试（第二批） ──

/// 测试多次 save/restore 后 fill_color 正确恢复（像素验证）。
#[test]
fn test_canvas_nested_save_restore_pixel_verify() {
    let mut ctx = CanvasContext::new(100, 100);

    ctx.set_fill_color(Color::rgba(255, 0, 0, 255)); // red
    ctx.save();
    ctx.set_fill_color(Color::rgba(0, 255, 0, 255)); // green
    ctx.save();
    ctx.set_fill_color(Color::rgba(0, 0, 255, 255)); // blue

    // 恢复到 green
    ctx.restore();
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    let pixel = ctx.get_image_data(5, 5, 1, 1);
    assert_eq!(pixel.data[1], 255, "restore 后应为绿色");

    // 恢复到 red
    ctx.restore();
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    let pixel = ctx.get_image_data(5, 5, 1, 1);
    assert_eq!(pixel.data[0], 255, "二次 restore 后应为红色");
}

/// 测试 clear_rect 清除指定区域像素。
#[test]
fn test_canvas_clear_rect_region() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_fill_color(Color::WHITE);
    ctx.fill_rect(0.0, 0.0, 100.0, 100.0);

    // 清除左上角 50x50 区域
    ctx.clear_rect(0.0, 0.0, 50.0, 50.0);

    let cleared = ctx.get_image_data(25, 25, 1, 1);
    let untouched = ctx.get_image_data(75, 75, 1, 1);

    // 被清除区域应为透明（全 0）
    assert_eq!(cleared.data[3], 0, "清除区域应为透明");
    // 未清除区域应保持白色
    assert_eq!(untouched.data[0], 255, "未清除区域应保持白色");
}

/// 测试 Canvas 2D 变换 translate + scale 组合。
#[test]
fn test_canvas_translate_scale_combined() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_fill_color(Color::RED);
    ctx.translate(50.0, 50.0);
    ctx.scale(2.0, 2.0);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);

    // 矩形在变换后应出现在 (50, 50) 位置，大小 20x20
    let pixel_inside = ctx.get_image_data(60, 60, 1, 1);
    assert_eq!(pixel_inside.data[0], 255, "变换后区域内部应有红色像素");
}

/// 测试 set_line_width 边界值。
#[test]
fn test_canvas_line_width_boundary_values() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_width(0.0);
    assert_eq!(ctx.line_width(), 0.0, "line_width=0 应被接受");

    ctx.set_line_width(0.5);
    assert!((ctx.line_width() - 0.5).abs() < f32::EPSILON, "line_width=0.5 应被接受");

    ctx.set_line_width(1000.0);
    assert_eq!(ctx.line_width(), 1000.0, "大 line_width 应被接受");
}

/// 测试 fillText 绘制不 panic。
#[test]
fn test_canvas_fill_text_no_panic() {
    let mut ctx = CanvasContext::new(200, 100);
    ctx.set_fill_color(Color::BLACK);
    // fillText 应不 panic（即使无字体加载）
    ctx.fill_text("Hello", 10.0, 50.0);
}

/// 测试 stroke_rect 零尺寸不 panic。
#[test]
fn test_canvas_stroke_rect_zero_dims() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_stroke_color(Color::BLACK);
    ctx.stroke_rect(50.0, 50.0, 0.0, 0.0);
    ctx.stroke_rect(0.0, 0.0, -10.0, -10.0);
}

/// 测试 putImageData 再 get_image_data 数据一致。
#[test]
fn test_canvas_put_get_roundtrip() {
    let mut ctx = CanvasContext::new(100, 100);
    let data = ImageData {
        width: 2,
        height: 2,
        data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 128, 128, 128, 255],
    };
    ctx.put_image_data(&data, 10, 10);

    let read = ctx.get_image_data(10, 10, 2, 2);
    assert_eq!(read.data[0..4], [255, 0, 0, 255], "像素 0 应一致");
    assert_eq!(read.data[4..8], [0, 255, 0, 255], "像素 1 应一致");
    assert_eq!(read.data[8..12], [0, 0, 255, 255], "像素 2 应一致");
    assert_eq!(read.data[12..16], [128, 128, 128, 255], "像素 3 应一致");
}

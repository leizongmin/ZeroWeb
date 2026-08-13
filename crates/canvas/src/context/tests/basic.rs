//! Canvas 上下文测试（basic 批次）。

use super::super::types::*;
use crate::context::*;
use zero_render_foundation::color::Color;

#[test]
fn test_canvas_new() {
    let ctx = CanvasContext::new(800, 600);
    assert_eq!(ctx.width(), 800);
    assert_eq!(ctx.height(), 600);
    assert_eq!(ctx.global_alpha(), 1.0);
    assert_eq!(ctx.line_width(), 1.0);
}

#[test]
fn test_canvas_fill_rect() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.fill_rect(10.0, 20.0, 30.0, 40.0);
    assert_eq!(ctx.primitives().fills.len(), 1);
    let fill = &ctx.primitives().fills[0];
    assert_eq!(fill.rect.origin.x, 10.0);
    assert_eq!(fill.rect.origin.y, 20.0);
    assert_eq!(fill.rect.size.width, 30.0);
    assert_eq!(fill.rect.size.height, 40.0);
}

#[test]
fn test_canvas_stroke_rect() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.stroke_rect(0.0, 0.0, 50.0, 50.0);
    // R34xx：stroke_rect 生成 1 个闭合周长路径描边图元（旧四边 fill 实现已废弃——
    // 负尺寸/零尺寸退化矩形与 join/cap 语义对齐 spec 需要路径描边）。
    assert_eq!(ctx.primitives().path_strokes.len(), 1);
}

#[test]
fn test_canvas_clear_rect() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.clear_rect(5.0, 5.0, 20.0, 20.0);
    assert_eq!(ctx.primitives().fills.len(), 1);
    let fill = &ctx.primitives().fills[0];
    assert_eq!(fill.color, Color::TRANSPARENT);
}

#[test]
fn test_canvas_fill_text() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.fill_text("hello", 10.0, 20.0, None);
    // 每个字符生成一个 glyph
    assert_eq!(ctx.primitives().glyphs.len(), 5);
}

#[test]
fn test_canvas_measure_text() {
    let ctx = CanvasContext::new(200, 200);
    let metrics = ctx.measure_text("abc");
    // 简化估算: 3 chars * 10.0 * 0.6 = 18.0
    assert!((metrics.width - 18.0).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_measure_text_full_fields_r3303() {
    // R3303：spec TextMetrics 全 10 字段。canvas crate 无真实字体度量，字段为 font.size 比例启发式
    // 近似（与既有 ascent=0.8em/descent=0.2em 一致）。断言字段集齐全 + 启发式一致性（防回归）。
    let ctx = CanvasContext::new(200, 200);
    let m = ctx.measure_text("hello");
    let size = 10.0_f32; // CanvasContext::new 默认 font.size
    // 与 measure_text 内同序计算（size*0.6 再乘字符数），避 f32 结合序差。
    let width = (size * 0.6) * 5.0;
    assert!((m.width - width).abs() < f32::EPSILON, "width = chars*em");
    assert!((m.actual_bounding_box_ascent - size * 0.8).abs() < f32::EPSILON);
    assert!((m.actual_bounding_box_descent - size * 0.2).abs() < f32::EPSILON);
    assert_eq!(m.actual_bounding_box_left, 0.0, "拉丁默认无左伸");
    assert!(
        (m.actual_bounding_box_right - width).abs() < f32::EPSILON,
        "right ≈ advance width"
    );
    assert!((m.font_bounding_box_ascent - size * 0.8).abs() < f32::EPSILON);
    assert!((m.font_bounding_box_descent - size * 0.2).abs() < f32::EPSILON);
    assert_eq!(m.alphabetic_baseline, 0.0, "默认基线即 alphabetic");
    assert!((m.hanging_baseline - size * 0.8).abs() < f32::EPSILON);
    assert!((m.ideographic_baseline - (-(size * 0.2))).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_save_restore() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_fill_color(Color::RED);
    ctx.save();
    ctx.set_fill_color(Color::BLUE);
    assert_eq!(ctx.fill_color(), Color::BLUE);
    ctx.restore();
    assert_eq!(ctx.fill_color(), Color::RED);
}

#[test]
fn test_canvas_set_fill_color() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_fill_color(Color::GREEN);
    assert_eq!(ctx.fill_color(), Color::GREEN);
}

#[test]
fn test_canvas_set_line_width() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_width(5.0);
    assert!((ctx.line_width() - 5.0).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_global_alpha() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_global_alpha(0.5);
    assert!((ctx.global_alpha() - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_translate() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.translate(10.0, 20.0);
    ctx.fill_rect(0.0, 0.0, 5.0, 5.0);
    let fill = &ctx.primitives().fills[0];
    assert!((fill.rect.origin.x - 10.0).abs() < f32::EPSILON);
    assert!((fill.rect.origin.y - 20.0).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_scale() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.scale(2.0, 3.0);
    ctx.fill_rect(10.0, 10.0, 10.0, 10.0);
    let fill = &ctx.primitives().fills[0];
    assert!((fill.rect.origin.x - 20.0).abs() < f32::EPSILON);
    assert!((fill.rect.origin.y - 30.0).abs() < f32::EPSILON);
    assert!((fill.rect.size.width - 20.0).abs() < f32::EPSILON);
    assert!((fill.rect.size.height - 30.0).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_rotate() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.rotate(std::f32::consts::FRAC_PI_2);
    // After 90-degree rotation, drawing at (1,0) should appear near (0,1)
    let (x, y) = ctx.transform.transform_point(1.0, 0.0);
    assert!((x).abs() < 0.001);
    assert!((y - 1.0).abs() < 0.001);
}

#[test]
fn test_canvas_set_transform() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_transform(2.0, 0.0, 0.0, 2.0, 10.0, 10.0);
    let (x, y) = ctx.transform.transform_point(5.0, 5.0);
    assert!((x - 20.0).abs() < f32::EPSILON);
    assert!((y - 20.0).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_reset_transform() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.translate(100.0, 100.0);
    ctx.reset_transform();
    let (x, y) = ctx.transform.transform_point(0.0, 0.0);
    assert!((x).abs() < f32::EPSILON);
    assert!((y).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_multiple_operations() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    ctx.stroke_rect(60.0, 60.0, 30.0, 30.0);
    ctx.fill_text("test", 0.0, 0.0, None);
    // fill_rect = 1, stroke_rect = 1 path_stroke（R34xx 旧四边 fill 已废弃）, fill_text = 4 glyphs
    assert_eq!(ctx.primitives().fills.len(), 1);
    assert_eq!(ctx.primitives().path_strokes.len(), 1);
    assert_eq!(ctx.primitives().glyphs.len(), 4);
}

#[test]
fn test_canvas_nested_save_restore() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_fill_color(Color::RED);
    ctx.save();
    ctx.set_fill_color(Color::GREEN);
    ctx.save();
    ctx.set_fill_color(Color::BLUE);
    assert_eq!(ctx.fill_color(), Color::BLUE);
    ctx.restore();
    assert_eq!(ctx.fill_color(), Color::GREEN);
    ctx.restore();
    assert_eq!(ctx.fill_color(), Color::RED);
}

#[test]
fn test_canvas_primitives_collected() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    ctx.fill_rect(20.0, 20.0, 10.0, 10.0);
    let primitives = ctx.into_primitives();
    assert_eq!(primitives.fills.len(), 2);
}

#[test]
fn test_image_data_new() {
    let ctx = CanvasContext::new(100, 100);
    let img = ctx.get_image_data(0, 0, 10, 10);
    assert_eq!(img.width, 10);
    assert_eq!(img.height, 10);
    assert_eq!(img.data.len(), 400); // 10 * 10 * 4
}

#[test]
fn test_transform_identity() {
    let t = Transform2D::identity();
    assert!((t.a - 1.0).abs() < f32::EPSILON);
    assert!((t.d - 1.0).abs() < f32::EPSILON);
    assert!((t.e).abs() < f32::EPSILON);
    assert!((t.f).abs() < f32::EPSILON);
}

#[test]
fn test_transform_multiply() {
    let a = Transform2D::translate(10.0, 20.0);
    let b = Transform2D::scale(2.0, 2.0);
    let c = a.multiply(&b);
    let (x, y) = c.transform_point(5.0, 5.0);
    // translate(10,20) * scale(2,2) applied to (5,5):
    // first scale: (10, 10), then translate: (20, 30)
    assert!((x - 20.0).abs() < f32::EPSILON);
    assert!((y - 30.0).abs() < f32::EPSILON);
}

#[test]
fn test_transform_point() {
    let t = Transform2D::translate(100.0, 200.0);
    let (x, y) = t.transform_point(10.0, 20.0);
    assert!((x - 110.0).abs() < f32::EPSILON);
    assert!((y - 220.0).abs() < f32::EPSILON);
}

// ── FontDescriptor / FontWeight / FontStyle ──

#[test]
fn test_font_descriptor_default() {
    let f = FontDescriptor::default();
    assert_eq!(f.family, "sans-serif");
    assert!((f.size - 10.0).abs() < f32::EPSILON);
    assert!(matches!(f.weight, FontWeight::Normal));
    assert!(matches!(f.style, FontStyle::Normal));
}

#[test]
fn test_font_descriptor_custom() {
    let f = FontDescriptor {
        family: "monospace".to_string(),
        size: 14.0,
        weight: FontWeight::Bold,
        style: FontStyle::Italic,
    };
    assert_eq!(f.family, "monospace");
    assert!(matches!(f.weight, FontWeight::Bold));
    assert!(matches!(f.style, FontStyle::Italic));
}

#[test]
fn test_canvas_set_font() {
    let mut ctx = CanvasContext::new(100, 100);
    let font = FontDescriptor {
        family: "serif".to_string(),
        size: 20.0,
        weight: FontWeight::Bold,
        style: FontStyle::Italic,
    };
    ctx.set_font(font);
    let metrics = ctx.measure_text("test");
    // 字体大小 20.0，4 字符 × 20.0 × 0.6 = 48.0
    assert!((metrics.width - 48.0).abs() < f32::EPSILON);
}

// ── stroke_color / stroke_text ──

#[test]
fn test_canvas_set_stroke_color() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_stroke_color(Color::BLUE);
    assert_eq!(ctx.stroke_color(), Color::BLUE);
}

#[test]
fn test_canvas_stroke_text() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.stroke_text("hello", 10.0, 20.0);
    // 每个字符生成一个 glyph
    assert_eq!(ctx.primitives().glyphs.len(), 5);
}

// ── 路径操作 ──

#[test]
fn test_canvas_begin_path_clears() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.move_to(10.0, 10.0);
    ctx.line_to(50.0, 50.0);
    ctx.begin_path();
    ctx.fill();
    // begin_path 清除路径，fill 空路径不生成图元
    assert_eq!(ctx.primitives().path_fills.len(), 0);
}

#[test]
fn test_canvas_move_to_line_to_fill() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.move_to(10.0, 10.0);
    ctx.line_to(100.0, 10.0);
    ctx.line_to(100.0, 100.0);
    ctx.fill();
    assert!(!ctx.primitives().path_fills.is_empty());
}

#[test]
fn test_canvas_stroke_path() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.move_to(10.0, 10.0);
    ctx.line_to(100.0, 10.0);
    ctx.line_to(100.0, 100.0);
    ctx.stroke();
    assert!(!ctx.primitives().path_strokes.is_empty());
}

#[test]
fn test_canvas_fill_empty_path() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.fill();
    assert_eq!(ctx.primitives().path_fills.len(), 0);
}

#[test]
fn test_canvas_stroke_empty_path() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.stroke();
    assert_eq!(ctx.primitives().path_strokes.len(), 0);
}

#[test]
fn test_canvas_quadratic_curve_to() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.move_to(10.0, 10.0);
    ctx.quadratic_curve_to(50.0, 0.0, 100.0, 50.0);
    ctx.fill();
    assert!(!ctx.primitives().path_fills.is_empty());
}

#[test]
fn test_canvas_bezier_curve_to() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.move_to(10.0, 10.0);
    ctx.bezier_curve_to(30.0, 0.0, 70.0, 100.0, 100.0, 50.0);
    ctx.fill();
    assert!(!ctx.primitives().path_fills.is_empty());
}

#[test]
fn test_canvas_close_path_on_context() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.move_to(10.0, 10.0);
    ctx.line_to(100.0, 10.0);
    ctx.line_to(100.0, 100.0);
    ctx.close_path();
    ctx.fill();
    assert!(!ctx.primitives().path_fills.is_empty());
}

#[test]
fn test_canvas_arc_on_context() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.arc(50.0, 50.0, 25.0, 0.0, std::f32::consts::PI, false);
    ctx.line_to(100.0, 100.0); // 确保有非弧线的路径点
    ctx.fill();
    assert!(!ctx.primitives().path_fills.is_empty());
}

// ── 边界条件 ──

#[test]
fn test_canvas_new_zero_size() {
    let ctx = CanvasContext::new(0, 0);
    assert_eq!(ctx.width(), 0);
    assert_eq!(ctx.height(), 0);
}

#[test]
fn test_canvas_global_alpha_clamp_high() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_global_alpha(2.0);
    assert!((ctx.global_alpha() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_global_alpha_clamp_negative() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_global_alpha(-1.0);
    assert!((ctx.global_alpha()).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_global_alpha_clamp_zero() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_global_alpha(0.0);
    assert!((ctx.global_alpha()).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_restore_empty_stack() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.restore(); // 不应 panic
    assert_eq!(ctx.fill_color(), Color::BLACK);
}

#[test]
fn test_canvas_measure_text_empty_string() {
    let ctx = CanvasContext::new(200, 200);
    let metrics = ctx.measure_text("");
    assert!((metrics.width).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_measure_text_unicode() {
    let ctx = CanvasContext::new(200, 200);
    let metrics = ctx.measure_text("日本語");
    // 3 个 char × 10.0 × 0.6 = 18.0（按 char 计数，非字节）
    assert!((metrics.width - 18.0).abs() < f32::EPSILON);
}

// ── save/restore 完整性 ──

#[test]
fn test_canvas_save_restore_stroke_color() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_stroke_color(Color::RED);
    ctx.save();
    ctx.set_stroke_color(Color::BLUE);
    assert_eq!(ctx.stroke_color(), Color::BLUE);
    ctx.restore();
    assert_eq!(ctx.stroke_color(), Color::RED);
}

#[test]
fn test_canvas_save_restore_line_width() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_width(5.0);
    ctx.save();
    ctx.set_line_width(10.0);
    ctx.restore();
    assert!((ctx.line_width() - 5.0).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_save_restore_global_alpha() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_global_alpha(0.5);
    ctx.save();
    ctx.set_global_alpha(0.8);
    ctx.restore();
    assert!((ctx.global_alpha() - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_save_restore_transform() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.translate(10.0, 20.0);
    ctx.save();
    ctx.translate(100.0, 200.0);
    ctx.restore();
    let (x, y) = ctx.transform.transform_point(0.0, 0.0);
    assert!((x - 10.0).abs() < 1.0);
    assert!((y - 20.0).abs() < 1.0);
}

#[test]
fn test_canvas_save_restore_font() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_font(FontDescriptor {
        family: "serif".to_string(),
        size: 16.0,
        weight: FontWeight::Bold,
        style: FontStyle::Normal,
    });
    ctx.save();
    ctx.set_font(FontDescriptor {
        family: "monospace".to_string(),
        size: 20.0,
        weight: FontWeight::Normal,
        style: FontStyle::Italic,
    });
    ctx.restore();
    let m = ctx.measure_text("x");
    // 应恢复到 serif 16pt: 1 × 16.0 × 0.6 = 9.6
    assert!((m.width - 9.6).abs() < f32::EPSILON);
}

// ── Transform 边界条件 ──

#[test]
fn test_transform_multiply_identity() {
    let t = Transform2D::translate(10.0, 20.0);
    let result = t.multiply(&Transform2D::identity());
    let (x, y) = result.transform_point(0.0, 0.0);
    assert!((x - 10.0).abs() < f32::EPSILON);
    assert!((y - 20.0).abs() < f32::EPSILON);
}

#[test]
fn test_transform_scale_negative() {
    let t = Transform2D::scale(-1.0, 1.0);
    let (x, y) = t.transform_point(5.0, 10.0);
    assert!((x - (-5.0)).abs() < f32::EPSILON);
    assert!((y - 10.0).abs() < f32::EPSILON);
}

#[test]
fn test_transform_scale_zero() {
    let t = Transform2D::scale(0.0, 0.0);
    let (x, y) = t.transform_point(5.0, 10.0);
    assert!((x).abs() < f32::EPSILON);
    assert!((y).abs() < f32::EPSILON);
}

#[test]
fn test_canvas_chained_transforms() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.translate(10.0, 0.0);
    ctx.scale(2.0, 1.0);
    let (x, y) = ctx.transform.transform_point(5.0, 5.0);
    // translate(10,0) 然后 scale(2,1): 先 scale 得 (10,5)，再 translate 得 (20,5)
    // 实际矩阵乘法顺序：scale 先应用
    assert!((x - 20.0).abs() < 0.01);
    assert!((y - 5.0).abs() < 0.01);
}

// ── alpha 应用 ──

#[test]
fn test_canvas_fill_rect_alpha_zero() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_global_alpha(0.0);
    ctx.fill_rect(10.0, 10.0, 50.0, 50.0);
    let fill = &ctx.primitives().fills[0];
    assert_eq!(fill.color.a, 0);
}

// ── put_image_data stub ──

#[test]
fn test_canvas_put_image_data_no_panic() {
    let mut ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 2,
        height: 2,
        data: vec![255; 16],
    };
    ctx.put_image_data(&img, 0, 0); // 不应 panic
}

// ── 边界条件补充测试 ──

/// 测试 arc 命令不 panic（路径简化实现只记录中心点）。
#[test]
fn test_canvas_arc_no_panic() {
    let mut ctx = CanvasContext::new(400, 400);
    ctx.begin_path();
    ctx.arc(200.0, 200.0, 50.0, 0.0, std::f32::consts::TAU, false);
    // arc 不 panic 即可
}

/// 测试 arc 部分弧线不 panic。
#[test]
fn test_canvas_arc_partial_no_panic() {
    let mut ctx = CanvasContext::new(400, 400);
    ctx.begin_path();
    ctx.arc(200.0, 200.0, 100.0, 0.0, std::f32::consts::PI, false);
    // arc 不 panic 即可
}

/// 测试多次 fill_rect 累积图元。
#[test]
fn test_canvas_fill_rect_accumulates() {
    let mut ctx = CanvasContext::new(400, 300);
    for i in 0..10 {
        ctx.fill_rect(i as f32 * 20.0, 0.0, 15.0, 15.0);
    }
    assert_eq!(ctx.primitives().fills.len(), 10);
}

/// 测试 fill_rect 负坐标。
#[test]
fn test_canvas_fill_rect_negative_coords() {
    let mut ctx = CanvasContext::new(400, 300);
    ctx.fill_rect(-50.0, -30.0, 100.0, 100.0);
    assert_eq!(ctx.primitives().fills.len(), 1);
}

/// 测试 stroke_rect 多次调用累积。
#[test]
fn test_canvas_stroke_rect_accumulates() {
    let mut ctx = CanvasContext::new(400, 300);
    ctx.stroke_rect(10.0, 10.0, 50.0, 50.0);
    ctx.stroke_rect(100.0, 100.0, 50.0, 50.0);
    // R34xx：每个 stroke_rect 生成 1 个周长路径描边图元（旧四边 fill 已废弃）
    assert_eq!(ctx.primitives().path_strokes.len(), 2);
}

/// 测试 into_primitives 消费上下文。
#[test]
fn test_canvas_into_primitives() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.fill_rect(0.0, 0.0, 100.0, 100.0);
    let count = ctx.primitives().fills.len();
    let primitives = ctx.into_primitives();
    assert_eq!(primitives.fills.len(), count);
}

/// 测试 get_image_data 各种尺寸。
#[test]
fn test_canvas_get_image_data_sizes() {
    let ctx = CanvasContext::new(200, 200);
    // 正常尺寸
    let img = ctx.get_image_data(0, 0, 10, 10);
    assert_eq!(img.width, 10);
    assert_eq!(img.height, 10);
    assert_eq!(img.data.len(), 400); // 10*10*4

    // 1x1
    let img1 = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(img1.width, 1);
    assert_eq!(img1.data.len(), 4);
}

/// 测试极端变换：大旋转角度。
#[test]
fn test_canvas_rotate_large_angle() {
    let mut ctx = CanvasContext::new(400, 400);
    ctx.rotate(100.0_f32.to_radians());
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    assert_eq!(ctx.primitives().fills.len(), 1);
}

/// 测试变换后 reset_transform 恢复。
#[test]
fn test_canvas_reset_after_transform() {
    let mut ctx = CanvasContext::new(400, 400);
    ctx.translate(100.0, 100.0);
    ctx.rotate(45.0_f32.to_radians());
    ctx.reset_transform();
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    // reset_transform 后应在原始坐标系绘制
    let fill = &ctx.primitives().fills[0];
    assert_eq!(fill.rect.origin.x, 0.0);
    assert_eq!(fill.rect.origin.y, 0.0);
}

/// 测试 fill_text 和 stroke_text 产生字形图元。
#[test]
fn test_canvas_text_produces_glyphs() {
    let mut ctx = CanvasContext::new(400, 300);
    ctx.fill_text("Hello", 10.0, 20.0, None);
    assert_eq!(ctx.primitives().glyphs.len(), 5); // 5 chars
    ctx.stroke_text("World", 10.0, 50.0);
    assert_eq!(ctx.primitives().glyphs.len(), 10); // 5 + 5 chars
}

/// 测试 set_font 影响文本度量。
#[test]
fn test_canvas_font_affects_measure() {
    let ctx = CanvasContext::new(400, 300);
    let small = ctx.measure_text("test");
    let mut ctx2 = CanvasContext::new(400, 300);
    ctx2.set_font(FontDescriptor {
        family: "serif".into(),
        size: 32.0,
        ..Default::default()
    });
    let large = ctx2.measure_text("test");
    assert!(large.width > small.width, "大字体应产生更宽的文本度量");
}

/// 测试 Path2D 连续操作。
#[test]
fn test_canvas_complex_path() {
    let mut ctx = CanvasContext::new(400, 400);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(100.0, 10.0);
    ctx.quadratic_curve_to(150.0, 50.0, 100.0, 100.0);
    ctx.bezier_curve_to(80.0, 120.0, 20.0, 120.0, 10.0, 100.0);
    ctx.close_path();
    ctx.fill();
    assert!(!ctx.primitives().path_fills.is_empty());
}

/// 测试 clear_rect 产生透明填充。
#[test]
fn test_canvas_clear_rect_transparent() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.clear_rect(10.0, 10.0, 50.0, 50.0);
    assert_eq!(ctx.primitives().fills.len(), 1);
    assert_eq!(ctx.primitives().fills[0].color.a, 0);
}

/// 测试 global_alpha 影响填充颜色透明度。
#[test]
fn test_canvas_alpha_affects_all_operations() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_global_alpha(0.5);
    ctx.fill_rect(0.0, 0.0, 100.0, 100.0);
    assert_eq!(ctx.primitives().fills[0].color.a, 127); // 255 * 0.5 ≈ 127
}

// ── put_image_data + get_image_data round-trip ──

/// 测试 put_image_data 写入像素后 get_image_data 能读回相同数据。
#[test]
fn test_put_get_image_data_round_trip() {
    let mut ctx = CanvasContext::new(10, 10);
    let pixels = vec![
        255, 0, 0, 255, // 红色
        0, 255, 0, 255, // 绿色
        0, 0, 255, 255, // 蓝色
        255, 255, 0, 255, // 黄色
    ];
    let img = ImageData {
        width: 2,
        height: 2,
        data: pixels.clone(),
    };
    ctx.put_image_data(&img, 0, 0);
    let result = ctx.get_image_data(0, 0, 2, 2);
    assert_eq!(result.data, pixels);
}

/// 测试 put_image_data 在偏移位置写入。
#[test]
fn test_put_image_data_with_offset() {
    let mut ctx = CanvasContext::new(10, 10);
    // 在 (5, 5) 位置写入 2x2 的红色像素
    let red = vec![255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255];
    let img = ImageData {
        width: 2,
        height: 2,
        data: red.clone(),
    };
    ctx.put_image_data(&img, 5, 5);
    // 读取偏移位置
    let result = ctx.get_image_data(5, 5, 2, 2);
    assert_eq!(result.data, red);
}

/// 测试 put_image_data 后 get_image_data 只读取写入的区域。
#[test]
fn test_get_image_data_reflects_put() {
    let mut ctx = CanvasContext::new(10, 10);
    // 先写入红色到 (0,0) - 2x2
    let red = ImageData {
        width: 2,
        height: 2,
        data: vec![255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255],
    };
    ctx.put_image_data(&red, 0, 0);
    // 再写入绿色到 (2,0) - 2x2
    let green = ImageData {
        width: 2,
        height: 2,
        data: vec![0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255],
    };
    ctx.put_image_data(&green, 2, 0);
    // 读取整个第一行 4 像素
    let result = ctx.get_image_data(0, 0, 4, 1);
    // 前 2 个红色，后 2 个绿色
    assert_eq!(result.data[0..4], [255, 0, 0, 255]); // 红
    assert_eq!(result.data[4..8], [255, 0, 0, 255]); // 红
    assert_eq!(result.data[8..12], [0, 255, 0, 255]); // 绿
    assert_eq!(result.data[12..16], [0, 255, 0, 255]); // 绿
}

/// 测试 get_image_data 在未写入区域返回零。
#[test]
fn test_get_image_data_unwritten_is_zeros() {
    let ctx = CanvasContext::new(10, 10);
    let result = ctx.get_image_data(5, 5, 2, 2);
    assert_eq!(result.data, vec![0u8; 16]);
}

/// 测试 fill_rect 后 get_image_data 反映绘制内容。
#[test]
fn test_get_image_data_after_fill_rect() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 3.0, 2.0);
    // 读取整个区域
    let result = ctx.get_image_data(0, 0, 10, 10);
    // (0,0) 应为红色
    assert_eq!(result.data[0..4], [255, 0, 0, 255]);
    // (3,0) 不应被填充
    assert_eq!(result.data[12..16], [0, 0, 0, 0]);
}

/// 测试 clear_rect 后 get_image_data 反映透明。
#[test]
fn test_get_image_data_after_clear_rect() {
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    ctx.clear_rect(2.0, 2.0, 3.0, 3.0);
    let result = ctx.get_image_data(2, 2, 1, 1);
    // 被清除的区域应透明
    assert_eq!(result.data[0..4], [0, 0, 0, 0]);
}

// ── Path fill/stroke shape correctness ──

/// 测试 fill() 生成 path_fills 而非 fills，且包含正确的顶点。
#[test]
fn test_fill_emits_path_fill_primitive() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(100.0, 10.0);
    ctx.line_to(100.0, 100.0);
    ctx.fill();
    // 应生成 path_fill 而非 fill
    assert_eq!(ctx.primitives().fills.len(), 0);
    assert_eq!(ctx.primitives().path_fills.len(), 1);
    let pf = &ctx.primitives().path_fills[0];
    // 三角形路径：2 条线段 = 4 个顶点对 (x1,y1,x2,y2)
    // line_to(10,10)->(100,10) 和 (100,10)->(100,100)
    assert!(pf.vertices.len() >= 8); // 至少 2 段 × 4 floats
    assert_eq!(pf.color, Color::BLACK);
}

/// 测试 stroke() 生成 path_stroke 图元，且包含正确的颜色和线宽。
#[test]
fn test_stroke_emits_path_stroke_primitive() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_line_width(3.0);
    ctx.set_stroke_color(Color::RED);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(100.0, 100.0);
    ctx.stroke();
    assert_eq!(ctx.primitives().fills.len(), 0);
    assert_eq!(ctx.primitives().path_strokes.len(), 1);
    let ps = &ctx.primitives().path_strokes[0];
    assert_eq!(ps.color, Color::RED);
    assert!((ps.line_width - 3.0).abs() < f32::EPSILON);
    assert!(!ps.vertices.is_empty());
}

/// 测试 fill() 三角形路径的顶点数量正确。
#[test]
fn test_fill_triangle_vertices() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(50.0, 0.0);
    ctx.line_to(25.0, 50.0);
    ctx.fill();
    let pf = &ctx.primitives().path_fills[0];
    // 2 条 LineTo 命令，每条生成 4 floats (x1,y1,x2,y2)
    assert_eq!(pf.vertices.len(), 8);
}

/// 测试 stroke() 的闭合标记。
#[test]
fn test_stroke_closed_flag() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(50.0, 0.0);
    ctx.line_to(50.0, 50.0);
    ctx.close_path();
    ctx.stroke();
    assert!(ctx.primitives().path_strokes[0].closed);
}

/// 测试 stroke() 无 close_path 时 closed=false。
#[test]
fn test_stroke_not_closed_flag() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(50.0, 0.0);
    ctx.stroke();
    assert!(!ctx.primitives().path_strokes[0].closed);
}

/// 测试 fill() 像素缓冲区写入（三角形应只覆盖部分像素）。
#[test]
fn test_fill_writes_pixels() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_fill_color(Color::RED);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(50.0, 0.0);
    ctx.line_to(25.0, 50.0);
    ctx.fill();
    // 检查三角形内部某点应为红色
    let result = ctx.get_image_data(10, 10, 1, 1);
    assert_eq!(result.data[0..4], [255, 0, 0, 255], "triangle interior should be red");
    // 检查三角形外部某点应为零
    let outside = ctx.get_image_data(40, 40, 1, 1);
    assert_eq!(outside.data[0..4], [0, 0, 0, 0], "outside triangle should be empty");
}

/// 测试 stroke() 像素缓冲区写入（描边线段应沿线覆盖像素）。
#[test]
fn test_stroke_writes_pixels() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_stroke_color(Color::BLUE);
    ctx.set_line_width(3.0);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(50.0, 0.0);
    ctx.stroke();
    // 沿描边路径上的像素应为蓝色
    let result = ctx.get_image_data(25, 0, 1, 1);
    assert_eq!(result.data[0..4], [0, 0, 255, 255], "stroke should be blue");
}

// ── fill_text / stroke_text per-character glyph ──

/// 测试 fill_text 每个字符的 glyph_id 等于 Unicode 码点。
#[test]
fn test_fill_text_glyph_ids_are_codepoints() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.fill_text("AB", 10.0, 20.0, None);
    let glyphs = &ctx.primitives().glyphs;
    assert_eq!(glyphs.len(), 2);
    assert_eq!(glyphs[0].glyph_id, 'A' as u32);
    assert_eq!(glyphs[1].glyph_id, 'B' as u32);
}

/// 测试 fill_text 每个字符水平偏移递增。
#[test]
fn test_fill_text_glyph_positions_offset() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.fill_text("abc", 10.0, 20.0, None);
    let glyphs = &ctx.primitives().glyphs;
    assert_eq!(glyphs.len(), 3);
    let em_width = 10.0 * 0.6; // font_size * 0.6
    assert!((glyphs[0].x - 10.0).abs() < f32::EPSILON);
    assert!((glyphs[1].x - (10.0 + em_width)).abs() < f32::EPSILON);
    assert!((glyphs[2].x - (10.0 + 2.0 * em_width)).abs() < f32::EPSILON);
}

/// 测试 stroke_text 使用描边颜色。
#[test]
fn test_stroke_text_uses_stroke_color() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_stroke_color(Color::RED);
    ctx.stroke_text("X", 10.0, 20.0);
    assert_eq!(ctx.primitives().glyphs[0].color, Color::RED);
}

/// 测试 fill_text 使用填充颜色。
#[test]
fn test_fill_text_uses_fill_color() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_fill_color(Color::GREEN);
    ctx.fill_text("X", 10.0, 20.0, None);
    assert_eq!(ctx.primitives().glyphs[0].color, Color::GREEN);
}

/// 测试空字符串不生成 glyph。
#[test]
fn test_fill_text_empty_string() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.fill_text("", 10.0, 20.0, None);
    assert_eq!(ctx.primitives().glyphs.len(), 0);
}

/// 测试 stroke_text 空字符串不生成 glyph。
#[test]
fn test_stroke_text_empty_string() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.stroke_text("", 10.0, 20.0);
    assert_eq!(ctx.primitives().glyphs.len(), 0);
}

/// 测试 Unicode 文本的 glyph_id 正确。
#[test]
fn test_fill_text_unicode_glyph_ids() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.fill_text("日本", 10.0, 20.0, None);
    let glyphs = &ctx.primitives().glyphs;
    assert_eq!(glyphs.len(), 2);
    assert_eq!(glyphs[0].glyph_id, '日' as u32);
    assert_eq!(glyphs[1].glyph_id, '本' as u32);
}

// ── Quadratic/Bezier curve flattening ──

/// 测试二次贝塞尔曲线填充生成正确的段数。
#[test]
fn test_quadratic_curve_flattening() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.quadratic_curve_to(50.0, 100.0, 100.0, 0.0);
    ctx.fill();
    let pf = &ctx.primitives().path_fills[0];
    // 8 段细分 × 4 floats = 32
    assert_eq!(pf.vertices.len(), 32);
}

/// 测试三次贝塞尔曲线填充生成正确的段数。
#[test]
fn test_bezier_curve_flattening() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.bezier_curve_to(25.0, 100.0, 75.0, 100.0, 100.0, 0.0);
    ctx.fill();
    let pf = &ctx.primitives().path_fills[0];
    // 8 段细分 × 4 floats = 32
    assert_eq!(pf.vertices.len(), 32);
}

/// 测试圆弧填充生成正确的段数。
#[test]
fn test_arc_flattening() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.arc(50.0, 50.0, 25.0, 0.0, std::f32::consts::PI, false);
    ctx.fill();
    let pf = &ctx.primitives().path_fills[0];
    // 16 段细分 × 4 floats = 64
    assert_eq!(pf.vertices.len(), 64);
}

// ── clip() 测试 ──

/// 测试 clip() 从三角形路径生成裁剪图元。
#[test]
fn test_clip_triangle() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(100.0, 10.0);
    ctx.line_to(50.0, 100.0);
    ctx.close_path();
    ctx.clip();
    assert_eq!(ctx.primitives().clips.len(), 1);
    let clip = &ctx.primitives().clips[0];
    // 裁剪区域应是路径的包围盒
    assert!(clip.rect.origin.x <= 10.0);
    assert!(clip.rect.origin.y <= 10.0);
    assert!(clip.rect.size.width >= 90.0);
    assert!(clip.rect.size.height >= 90.0);
}

/// 测试 clip() 空路径不生成裁剪图元。
#[test]
fn test_clip_empty_path() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.begin_path();
    ctx.clip();
    assert_eq!(ctx.primitives().clips.len(), 0);
}

/// 测试 clip() 矩形路径生成精确的裁剪矩形。
#[test]
fn test_clip_rectangular_path() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(20.0, 30.0);
    ctx.line_to(80.0, 30.0);
    ctx.line_to(80.0, 70.0);
    ctx.line_to(20.0, 70.0);
    ctx.close_path();
    ctx.clip();
    assert_eq!(ctx.primitives().clips.len(), 1);
    let clip = &ctx.primitives().clips[0];
    assert!((clip.rect.origin.x - 20.0).abs() < f32::EPSILON);
    assert!((clip.rect.origin.y - 30.0).abs() < f32::EPSILON);
    assert!((clip.rect.size.width - 60.0).abs() < f32::EPSILON);
    assert!((clip.rect.size.height - 40.0).abs() < f32::EPSILON);
}

/// 测试 clip() 后绘制操作仍然正常。
#[test]
fn test_clip_then_draw() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(100.0, 0.0);
    ctx.line_to(100.0, 100.0);
    ctx.close_path();
    ctx.clip();
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    assert_eq!(ctx.primitives().clips.len(), 1);
    assert_eq!(ctx.primitives().fills.len(), 1);
}

/// 测试多次 clip() 调用累积裁剪区域。
#[test]
fn test_clip_multiple() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(50.0, 0.0);
    ctx.line_to(50.0, 50.0);
    ctx.close_path();
    ctx.clip();
    ctx.begin_path();
    ctx.move_to(25.0, 25.0);
    ctx.line_to(75.0, 25.0);
    ctx.line_to(75.0, 75.0);
    ctx.close_path();
    ctx.clip();
    assert_eq!(ctx.primitives().clips.len(), 2);
}

// ── CompositeOperation 测试 ──

/// 测试默认合成操作模式为 SourceOver。
#[test]
fn test_composite_operation_default() {
    let ctx = CanvasContext::new(100, 100);
    assert_eq!(ctx.composite_operation(), CompositeOperation::SourceOver);
}

/// 测试设置和获取合成操作模式。
#[test]
fn test_composite_operation_set_get() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_composite_operation(CompositeOperation::Multiply);
    assert_eq!(ctx.composite_operation(), CompositeOperation::Multiply);
    ctx.set_composite_operation(CompositeOperation::Screen);
    assert_eq!(ctx.composite_operation(), CompositeOperation::Screen);
}

/// 测试合成操作模式在 save/restore 中正确保存和恢复。
#[test]
fn test_composite_operation_save_restore() {
    let mut ctx = CanvasContext::new(100, 100);
    assert_eq!(ctx.composite_operation(), CompositeOperation::SourceOver);
    ctx.save();
    ctx.set_composite_operation(CompositeOperation::Lighter);
    assert_eq!(ctx.composite_operation(), CompositeOperation::Lighter);
    ctx.restore();
    assert_eq!(ctx.composite_operation(), CompositeOperation::SourceOver);
}

/// 测试所有合成操作模式变体可以正确设置。
#[test]
fn test_composite_operation_all_variants() {
    let mut ctx = CanvasContext::new(100, 100);
    let ops = [
        CompositeOperation::SourceOver,
        CompositeOperation::DestinationOver,
        CompositeOperation::DestinationOut,
        CompositeOperation::DestinationAtop,
        CompositeOperation::DestinationIn,
        CompositeOperation::SourceIn,
        CompositeOperation::SourceAtop,
        CompositeOperation::Lighter,
        CompositeOperation::Copy,
        CompositeOperation::Xor,
        CompositeOperation::Multiply,
        CompositeOperation::Screen,
        CompositeOperation::Overlay,
        CompositeOperation::Darken,
        CompositeOperation::Lighten,
        CompositeOperation::ColorDodge,
        CompositeOperation::ColorBurn,
        CompositeOperation::HardLight,
        CompositeOperation::SoftLight,
        CompositeOperation::Difference,
        CompositeOperation::Exclusion,
        CompositeOperation::Hue,
        CompositeOperation::Saturation,
        CompositeOperation::Color,
        CompositeOperation::Luminosity,
    ];
    for op in &ops {
        ctx.set_composite_operation(*op);
        assert_eq!(ctx.composite_operation(), *op);
    }
}

/// 测试合成操作模式 save/restore 嵌套。
#[test]
fn test_composite_operation_nested_save_restore() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_composite_operation(CompositeOperation::SourceOver);
    ctx.save();
    ctx.set_composite_operation(CompositeOperation::Multiply);
    ctx.save();
    ctx.set_composite_operation(CompositeOperation::Screen);
    assert_eq!(ctx.composite_operation(), CompositeOperation::Screen);
    ctx.restore();
    assert_eq!(ctx.composite_operation(), CompositeOperation::Multiply);
    ctx.restore();
    assert_eq!(ctx.composite_operation(), CompositeOperation::SourceOver);
}

// ── createLinearGradient 测试 ──

/// 测试创建线性渐变。
#[test]
fn test_create_linear_gradient() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 200.0, 0.0);
    assert!((grad.x0).abs() < f32::EPSILON);
    assert!((grad.y0).abs() < f32::EPSILON);
    assert!((grad.x1 - 200.0).abs() < f32::EPSILON);
    assert!((grad.y1).abs() < f32::EPSILON);
    assert!(grad.stops.is_empty());
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(1.0, Color::BLUE);
    assert_eq!(grad.stops.len(), 2);
    assert_eq!(grad.stops[0].color, Color::RED);
    assert_eq!(grad.stops[1].color, Color::BLUE);
}

/// 测试线性渐变多色停止点。
#[test]
fn test_linear_gradient_multiple_stops() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 100.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(0.5, Color::GREEN);
    grad.add_color_stop(1.0, Color::BLUE);
    assert_eq!(grad.stops.len(), 3);
    assert!((grad.stops[1].offset - 0.5).abs() < f32::EPSILON);
    assert_eq!(grad.stops[1].color, Color::GREEN);
}

/// 测试线性渐变起点和终点相同（退化情况不 panic）。
#[test]
fn test_linear_gradient_degenerate() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(50.0, 50.0, 50.0, 50.0);
    grad.add_color_stop(0.0, Color::RED);
    assert_eq!(grad.stops.len(), 1);
}

// ── createRadialGradient 测试 ──

/// 测试创建径向渐变。
#[test]
fn test_create_radial_gradient() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_radial_gradient(50.0, 50.0, 10.0, 50.0, 50.0, 100.0);
    assert!((grad.x0 - 50.0).abs() < f32::EPSILON);
    assert!((grad.y0 - 50.0).abs() < f32::EPSILON);
    assert!((grad.r0 - 10.0).abs() < f32::EPSILON);
    assert!((grad.x1 - 50.0).abs() < f32::EPSILON);
    assert!((grad.y1 - 50.0).abs() < f32::EPSILON);
    assert!((grad.r1 - 100.0).abs() < f32::EPSILON);
    assert!(grad.stops.is_empty());
    grad.add_color_stop(0.0, Color::WHITE);
    grad.add_color_stop(1.0, Color::BLACK);
    assert_eq!(grad.stops.len(), 2);
}

/// 测试径向渐变多色停止点。
#[test]
fn test_radial_gradient_multiple_stops() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_radial_gradient(0.0, 0.0, 0.0, 100.0, 100.0, 50.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(0.33, Color::GREEN);
    grad.add_color_stop(0.66, Color::BLUE);
    grad.add_color_stop(1.0, Color::WHITE);
    assert_eq!(grad.stops.len(), 4);
}

/// 测试径向渐变偏心圆（圆心不同）。
#[test]
fn test_radial_gradient_eccentric() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_radial_gradient(0.0, 0.0, 5.0, 200.0, 200.0, 50.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(1.0, Color::BLUE);
    assert_eq!(grad.stops.len(), 2);
}

// ── createPattern 测试 ──

/// 测试从 ImageData 创建图案。
#[test]
fn test_create_pattern() {
    let ctx = CanvasContext::new(200, 200);
    let img = ImageData {
        width: 2,
        height: 2,
        data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
    };
    let pattern = ctx.create_pattern(img.clone(), PatternRepetition::Repeat);
    assert_eq!(pattern.image_data.width, 2);
    assert_eq!(pattern.image_data.height, 2);
    assert_eq!(pattern.repetition, PatternRepetition::Repeat);
}

/// 测试图案重复模式 NoRepeat。
#[test]
fn test_create_pattern_no_repeat() {
    let ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 1,
        height: 1,
        data: vec![255, 0, 0, 255],
    };
    let pattern = ctx.create_pattern(img, PatternRepetition::NoRepeat);
    assert_eq!(pattern.repetition, PatternRepetition::NoRepeat);
}

/// 测试图案重复模式 RepeatX / RepeatY。
#[test]
fn test_create_pattern_repeat_variants() {
    let ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 1,
        height: 1,
        data: vec![0; 4],
    };

    let p1 = ctx.create_pattern(img.clone(), PatternRepetition::RepeatX);
    assert_eq!(p1.repetition, PatternRepetition::RepeatX);

    let p2 = ctx.create_pattern(img, PatternRepetition::RepeatY);
    assert_eq!(p2.repetition, PatternRepetition::RepeatY);
}

/// 测试图案默认重复模式为 Repeat。
#[test]
fn test_pattern_repetition_default() {
    assert_eq!(PatternRepetition::default(), PatternRepetition::Repeat);
}

// ── isPointInPath 测试 ──

/// 测试点在三角形路径内部。
#[test]
fn test_is_point_in_path_inside_triangle() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(100.0, 0.0);
    ctx.line_to(50.0, 100.0);
    ctx.close_path();
    // 质心 (50, 33.3) 应在三角形内
    assert!(ctx.is_point_in_path(50.0, 30.0));
}

/// 测试点在三角形路径外部。
#[test]
fn test_is_point_in_path_outside_triangle() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(100.0, 0.0);
    ctx.line_to(50.0, 100.0);
    ctx.close_path();
    // 点 (200, 200) 应在三角形外
    assert!(!ctx.is_point_in_path(200.0, 200.0));
}

/// 测试空路径上所有点都不在路径内。
#[test]
fn test_is_point_in_path_empty_path() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    assert!(!ctx.is_point_in_path(50.0, 50.0));
}

/// 测试点在矩形路径上。
#[test]
fn test_is_point_in_path_rectangle() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(90.0, 10.0);
    ctx.line_to(90.0, 90.0);
    ctx.line_to(10.0, 90.0);
    ctx.close_path();
    // 中心点应在矩形内
    assert!(ctx.is_point_in_path(50.0, 50.0));
    // 角落外的点应不在矩形内
    assert!(!ctx.is_point_in_path(5.0, 5.0));
}

/// 测试点恰好在地面上。
#[test]
fn test_is_point_in_path_on_edge() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(100.0, 0.0);
    ctx.line_to(100.0, 100.0);
    ctx.close_path();
    // 边界上的点行为取决于射线法实现（不确定在内还是外）
    // 主要验证不 panic
    let _ = ctx.is_point_in_path(0.0, 0.0);
    let _ = ctx.is_point_in_path(50.0, 0.0);
}

/// 测试 isPointInPath 对仅有 MoveTo 的路径返回 false。
#[test]
fn test_is_point_in_path_move_to_only() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(50.0, 50.0);
    // 只有 MoveTo，没有闭合区域
    assert!(!ctx.is_point_in_path(50.0, 50.0));
}

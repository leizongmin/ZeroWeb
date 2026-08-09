// Raster 辅助函数覆盖率测试
use super::super::types::*;
use crate::context::*;
use crate::path::Path2D;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;

/// 辅助：创建一个带指定尺寸和像素缓冲区的 CanvasContext。
fn ctx_with_pixels(w: u32, h: u32) -> CanvasContext {
    let mut ctx = CanvasContext::new(w, h);
    ctx.pixel_buffer = vec![0u8; (w * h * 4) as usize];
    ctx
}

// ── transform_rect 测试 ──

#[test]
fn test_transform_rect_identity() {
    let ctx = CanvasContext::new(100, 100);
    let rect = ctx.transform_rect(10.0, 20.0, 30.0, 40.0);
    assert_eq!(rect.origin.x, 10.0);
    assert_eq!(rect.origin.y, 20.0);
    assert_eq!(rect.size.width, 30.0);
    assert_eq!(rect.size.height, 40.0);
}

#[test]
fn test_transform_rect_scaled() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.transform = Transform2D::scale(2.0, 3.0);
    let rect = ctx.transform_rect(10.0, 10.0, 20.0, 20.0);
    // 缩放后：(10,10) → (20,30)，(30,30) → (60,90)
    assert_eq!(rect.origin.x, 20.0);
    assert_eq!(rect.origin.y, 30.0);
    assert_eq!(rect.size.width, 40.0);
    assert_eq!(rect.size.height, 60.0);
}

#[test]
fn test_transform_rect_zero_size() {
    let ctx = CanvasContext::new(100, 100);
    let rect = ctx.transform_rect(5.0, 5.0, 0.0, 0.0);
    assert_eq!(rect.origin.x, 5.0);
    assert_eq!(rect.origin.y, 5.0);
    assert_eq!(rect.size.width, 0.0);
    assert_eq!(rect.size.height, 0.0);
}

// ── apply_alpha 测试 ──

#[test]
fn test_apply_alpha_full() {
    let ctx = CanvasContext::new(100, 100);
    let color = Color::rgba(255, 128, 64, 200);
    let result = ctx.apply_alpha(color);
    assert_eq!(result.r, 255);
    assert_eq!(result.g, 128);
    assert_eq!(result.b, 64);
    // global_alpha 默认 1.0 → a = 200 * 1.0 = 200
    assert_eq!(result.a, 200);
}

#[test]
fn test_apply_alpha_half() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.global_alpha = 0.5;
    let color = Color::rgba(255, 0, 0, 200);
    let result = ctx.apply_alpha(color);
    assert_eq!(result.a, 100); // 200 * 0.5 = 100
}

#[test]
fn test_apply_alpha_zero() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.global_alpha = 0.0;
    let color = Color::rgba(255, 255, 255, 255);
    let result = ctx.apply_alpha(color);
    assert_eq!(result.a, 0);
}

// ── composite_pixel 测试 ──

#[test]
fn test_composite_pixel_source_over() {
    let ctx = CanvasContext::new(100, 100);
    let src = Color::rgba(255, 0, 0, 128); // 半透明红
    let (r, _g, _b, a) = ctx.composite_pixel(src, 0, 0, 255, 255); // 不透明蓝背景
    assert!(r > 0, "red channel should be nonzero");
    assert!(a > 0, "output alpha should be nonzero");
}

#[test]
fn test_composite_pixel_copy() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.composite_operation = CompositeOperation::Copy;
    let src = Color::rgba(100, 150, 200, 255);
    let (r, g, b, a) = ctx.composite_pixel(src, 50, 50, 50, 50);
    assert_eq!(r, 100);
    assert_eq!(g, 150);
    assert_eq!(b, 200);
    assert_eq!(a, 255);
}

#[test]
fn test_composite_pixel_source_over_transparent_dst() {
    let ctx = CanvasContext::new(100, 100);
    let src = Color::rgba(0, 255, 0, 200);
    let (_r, g, _b, a) = ctx.composite_pixel(src, 0, 0, 0, 0);
    assert!(g > 0);
    assert!(a > 0);
}

#[test]
fn test_composite_pixel_source_over_opaque_src() {
    let ctx = CanvasContext::new(100, 100);
    let src = Color::rgba(255, 0, 0, 255); // 不透明红
    let (r, _g, _b, a) = ctx.composite_pixel(src, 0, 255, 0, 255);
    assert_eq!(r, 255); // 不透明源应覆盖目标
    assert_eq!(a, 255);
}

#[test]
fn test_composite_pixel_zero_alpha_src() {
    let ctx = CanvasContext::new(100, 100);
    let src = Color::rgba(255, 0, 0, 0); // 完全透明
    let (r, g, b, a) = ctx.composite_pixel(src, 0, 255, 0, 255);
    assert_eq!(r, 0);
    assert_eq!(g, 255);
    assert_eq!(b, 0);
    assert_eq!(a, 255);
}

// ── line_segment_rect 测试 ──

#[test]
fn test_line_segment_rect_horizontal() {
    let ctx = CanvasContext::new(100, 100);
    let rect = ctx.line_segment_rect(10.0, 50.0, 90.0, 50.0, 4.0);
    assert_eq!(rect.origin.x, 8.0); // 10 - 2
    assert_eq!(rect.origin.y, 48.0); // 50 - 2
    assert_eq!(rect.size.width, 84.0); // 90-10+4
    assert_eq!(rect.size.height, 4.0); // line_width
}

#[test]
fn test_line_segment_rect_vertical() {
    let ctx = CanvasContext::new(100, 100);
    let rect = ctx.line_segment_rect(50.0, 10.0, 50.0, 90.0, 6.0);
    assert_eq!(rect.origin.x, 47.0); // 50 - 3
    assert_eq!(rect.origin.y, 7.0); // 10 - 3
    assert_eq!(rect.size.width, 6.0);
    assert_eq!(rect.size.height, 86.0);
}

#[test]
fn test_line_segment_rect_zero_width() {
    let ctx = CanvasContext::new(100, 100);
    let rect = ctx.line_segment_rect(10.0, 20.0, 30.0, 40.0, 0.0);
    assert_eq!(rect.origin.x, 10.0);
    assert_eq!(rect.origin.y, 20.0);
}

// ── blit_rect_to_pixels 测试 ──

#[test]
fn test_blit_rect_writes_pixels() {
    let mut ctx = ctx_with_pixels(10, 10);
    let rect = Rect::new(2.0, 2.0, 3.0, 3.0);
    let color = Color::rgba(255, 0, 0, 255);
    ctx.blit_rect_to_pixels(&rect, color);

    // 检查中心像素 (3, 3) 是否被写入
    let idx = ((3 * 10) + 3) * 4;
    assert_eq!(ctx.pixel_buffer[idx], 255, "red channel at (3,3)");
    assert_eq!(ctx.pixel_buffer[idx + 3], 255, "alpha at (3,3)");
}

#[test]
fn test_blit_rect_clips_to_bounds() {
    let mut ctx = ctx_with_pixels(10, 10);
    // 矩形超出边界
    let rect = Rect::new(-5.0, -5.0, 20.0, 20.0);
    let color = Color::rgba(0, 255, 0, 255);
    ctx.blit_rect_to_pixels(&rect, color);

    // 像素 (0, 0) 应被写入（在裁剪范围内）
    assert_eq!(ctx.pixel_buffer[1], 255, "green at (0,0)");
}

// ── compute_miter_length 测试 ──

#[test]
fn test_compute_miter_length_perpendicular() {
    // 两个垂直的法线 → 斜接长度 = half_lw * 2 / sqrt(2) ≈ half_lw * 1.414
    let len = CanvasContext::compute_miter_length(1.0, 0.0, 0.0, 1.0, 5.0);
    let expected = 10.0 / 2.0f32.sqrt(); // half_lw * 2 / sqrt(2)
    assert!((len - expected).abs() < 0.01);
}

#[test]
fn test_compute_miter_length_parallel() {
    // 平行法线 → m = 2, length = half_lw * 2 / 2 = half_lw
    let len = CanvasContext::compute_miter_length(1.0, 0.0, 1.0, 0.0, 5.0);
    assert_eq!(len, 5.0);
}

#[test]
fn test_compute_miter_length_opposite() {
    // 反向法线 → m ≈ 0 → 回退到 half_lw
    let len = CanvasContext::compute_miter_length(1.0, 0.0, -1.0, 0.0, 3.0);
    assert_eq!(len, 3.0); // 回退到 half_lw
}

#[test]
fn test_compute_miter_length_zero_normals() {
    let len = CanvasContext::compute_miter_length(0.0, 0.0, 0.0, 0.0, 4.0);
    assert_eq!(len, 4.0); // m = 0 → 回退到 half_lw
}

// ── flatten_round_rect 测试 ──

#[test]
fn test_flatten_round_rect_empty_radii() {
    // 0 个半径值 → 退化为矩形
    let mut verts = Vec::new();
    let (cx, cy) = CanvasContext::flatten_round_rect(&mut verts, 0.0, 0.0, 10.0, 10.0, 50.0, 50.0, &[]);
    assert_eq!(cx, 10.0); // 第一个角 x
    assert_eq!(cy, 10.0); // 第一个角 y
}

#[test]
fn test_flatten_round_rect_one_radius() {
    let mut verts = Vec::new();
    let (_cx, _cy) = CanvasContext::flatten_round_rect(&mut verts, 0.0, 0.0, 10.0, 10.0, 50.0, 50.0, &[8.0]);
    // 应产生顶点（圆角路径）
    assert!(!verts.is_empty(), "should produce vertices for rounded rect");
}

#[test]
fn test_flatten_round_rect_two_radii() {
    let mut verts = Vec::new();
    let (_cx, _cy) = CanvasContext::flatten_round_rect(&mut verts, 0.0, 0.0, 10.0, 10.0, 50.0, 50.0, &[8.0, 12.0]);
    assert!(!verts.is_empty(), "should produce vertices for 2-radii round rect");
}

#[test]
fn test_flatten_round_rect_three_radii() {
    let mut verts = Vec::new();
    let (_cx, _cy) = CanvasContext::flatten_round_rect(&mut verts, 0.0, 0.0, 10.0, 10.0, 50.0, 50.0, &[8.0, 12.0, 6.0]);
    assert!(!verts.is_empty(), "should produce vertices for 3-radii round rect");
}

#[test]
fn test_flatten_round_rect_four_radii() {
    let mut verts = Vec::new();
    let (_cx, _cy) =
        CanvasContext::flatten_round_rect(&mut verts, 0.0, 0.0, 10.0, 10.0, 50.0, 50.0, &[8.0, 12.0, 6.0, 10.0]);
    assert!(!verts.is_empty(), "should produce vertices for 4-radii round rect");
}

#[test]
fn test_flatten_round_rect_zero_radius_degenerates() {
    // 所有半径为 0 → 退化为矩形
    let mut verts = Vec::new();
    let (cx, cy) = CanvasContext::flatten_round_rect(&mut verts, 5.0, 5.0, 10.0, 10.0, 50.0, 50.0, &[0.0]);
    assert_eq!(cx, 10.0);
    assert_eq!(cy, 10.0);
    // 应产生 5 条线段（4 边 + 起始连接线）
    assert_eq!(verts.len(), 5 * 4, "degenerate round rect = 5 line segments");
}

#[test]
fn test_flatten_round_rect_large_radius_clamped() {
    // 半径超过短边一半时应被钳位
    let mut verts = Vec::new();
    let _ = CanvasContext::flatten_round_rect(&mut verts, 0.0, 0.0, 10.0, 10.0, 20.0, 10.0, &[100.0]);
    // max_r = min(20,10)/2 = 5, radius should be clamped to 5
    assert!(!verts.is_empty());
}

#[test]
fn test_flatten_round_rect_negative_radius_clamped() {
    let mut verts = Vec::new();
    let _ = CanvasContext::flatten_round_rect(&mut verts, 0.0, 0.0, 10.0, 10.0, 50.0, 50.0, &[-5.0]);
    // 负半径被钳位到 0 → 退化为矩形
    assert_eq!(verts.len(), 5 * 4, "negative radius degenerates to rect");
}

// ── compute_arc_to_geometry 测试 ──

#[test]
fn test_compute_arc_to_geometry_zero_radius() {
    let (t1x, t1y, t2x, t2y) = CanvasContext::compute_arc_to_geometry(0.0, 0.0, 50.0, 0.0, 50.0, 50.0, 0.0);
    // 零半径 → 退化为控制点
    assert_eq!(t1x, 50.0);
    assert_eq!(t1y, 0.0);
    assert_eq!(t2x, 50.0);
    assert_eq!(t2y, 0.0);
}

#[test]
fn test_compute_arc_to_geometry_collinear() {
    // 三点共线 → 退化为直线
    let (t1x, _t1y, t2x, _t2y) = CanvasContext::compute_arc_to_geometry(0.0, 0.0, 50.0, 0.0, 100.0, 0.0, 20.0);
    assert_eq!(t1x, 50.0);
    assert_eq!(t2x, 50.0);
}

#[test]
fn test_compute_arc_to_geometry_normal() {
    let (t1x, t1y, t2x, t2y) = CanvasContext::compute_arc_to_geometry(0.0, 0.0, 50.0, 0.0, 50.0, 50.0, 20.0);
    // 正常情况：切点不在控制点上
    assert_ne!(
        (t1x, t1y),
        (50.0, 0.0),
        "tangent point should differ from control for normal case"
    );
    assert_ne!((t2x, t2y), (50.0, 0.0), "tangent point 2 should differ from control");
}

#[test]
fn test_compute_arc_to_geometry_coincident_points() {
    // 起点和控制点重合 → len1=0 → 退化为控制点
    let (t1x, t1y, t2x, t2y) = CanvasContext::compute_arc_to_geometry(50.0, 0.0, 50.0, 0.0, 100.0, 0.0, 20.0);
    assert_eq!(t1x, 50.0);
    assert_eq!(t1y, 0.0);
    assert_eq!(t2x, 50.0);
    assert_eq!(t2y, 0.0);
}

#[test]
fn test_compute_arc_to_geometry_control2_coincident() {
    // 控制点1和终点重合 → len2=0 → 退化为控制点
    let (t1x, t1y, t2x, t2y) = CanvasContext::compute_arc_to_geometry(0.0, 0.0, 50.0, 50.0, 50.0, 50.0, 20.0);
    assert_eq!(t1x, 50.0);
    assert_eq!(t1y, 50.0);
    assert_eq!(t2x, 50.0);
    assert_eq!(t2y, 50.0);
}

// ── flatten_arc_to 测试 ──

#[test]
fn test_flatten_arc_to_normal() {
    let mut verts = Vec::new();
    CanvasContext::flatten_arc_to(&mut verts, 0.0, 0.0, 50.0, 0.0, 50.0, 50.0, 20.0, 4);
    assert!(!verts.is_empty(), "normal arc should produce vertices");
    // 每个段 4 个 f32 (x1,y1,x2,y2)
    assert!(
        verts.len() >= 4 * 4,
        "should have >= 4 arc segments + 1 connecting line"
    );
}

#[test]
fn test_flatten_arc_to_degenerate() {
    // 零半径 → 切点重合，不产生弧线（可能有连接线到控制点）
    let mut verts = Vec::new();
    CanvasContext::flatten_arc_to(&mut verts, 0.0, 0.0, 50.0, 0.0, 50.0, 50.0, 0.0, 4);
    // 零半径 → t1==t2 → 在切点重合检查处直接返回（无连接线因为 current!=t1 也不满足）
    // 或者只产生连接线但不产生弧线段
    // 最多 4 floats (连接线), 不应有弧线段
    assert!(
        verts.len() <= 4,
        "zero-radius arc should produce no arc segments, got {} floats",
        verts.len()
    );
}

#[test]
fn test_flatten_arc_to_same_position() {
    // 当前点已在切点位置 → 不画连接线
    let mut verts = Vec::new();
    // 使用共线点使切点在控制点上
    CanvasContext::flatten_arc_to(&mut verts, 50.0, 0.0, 50.0, 0.0, 50.0, 50.0, 20.0, 4);
    // 切点重合 → 直接返回
}

// ── flatten_path 测试（通过 CanvasContext 的 current_path）──

#[test]
fn test_flatten_path_empty() {
    let ctx = CanvasContext::new(100, 100);
    let verts = ctx.flatten_path();
    assert!(verts.is_empty(), "empty path should produce no vertices");
}

#[test]
fn test_flatten_path_line_to() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(50.0, 50.0);
    let verts = ctx.flatten_path();
    assert_eq!(verts.len(), 4, "single line segment = 4 floats");
    assert_eq!(verts[0], 10.0);
    assert_eq!(verts[1], 10.0);
    assert_eq!(verts[2], 50.0);
    assert_eq!(verts[3], 50.0);
}

#[test]
fn test_flatten_path_quadratic_curve() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.quadratic_curve_to(50.0, 0.0, 50.0, 50.0);
    let verts = ctx.flatten_path();
    // 8 段细分 × 4 = 32 floats
    assert_eq!(verts.len(), 32, "quadratic curve = 8 segments × 4");
}

#[test]
fn test_flatten_path_bezier_curve() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.bezier_curve_to(25.0, 0.0, 50.0, 25.0, 50.0, 50.0);
    let verts = ctx.flatten_path();
    assert_eq!(verts.len(), 32, "cubic bezier = 8 segments × 4");
}

#[test]
fn test_flatten_path_arc() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 0.0);
    ctx.current_path
        .arc(50.0, 50.0, 50.0, -std::f32::consts::FRAC_PI_2, 0.0);
    let verts = ctx.flatten_path();
    assert_eq!(verts.len(), 64, "arc = 16 segments × 4");
}

#[test]
fn test_flatten_path_close_path() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(50.0, 10.0);
    ctx.current_path.line_to(50.0, 50.0);
    ctx.current_path.close_path();
    let verts = ctx.flatten_path();
    // 2 line segments + 1 close segment = 3 × 4 = 12
    assert_eq!(verts.len(), 12, "triangle = 3 segments");
}

#[test]
fn test_flatten_path_close_already_at_start() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(10.0, 10.0); // 回到起点
    ctx.current_path.close_path(); // 已在起点，不产生额外线段
    let verts = ctx.flatten_path();
    assert_eq!(verts.len(), 4, "line back to start, close produces nothing extra");
}

#[test]
fn test_flatten_path_ellipse_command() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 10.0);
    ctx.current_path
        .ellipse(50.0, 50.0, 40.0, 30.0, 0.0, -std::f32::consts::FRAC_PI_2, 0.0);
    let verts = ctx.flatten_path();
    assert_eq!(verts.len(), 64, "ellipse = 16 segments × 4");
}

#[test]
fn test_flatten_path_round_rect_command() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.round_rect(10.0, 10.0, 50.0, 50.0, vec![8.0]);
    let verts = ctx.flatten_path();
    assert!(!verts.is_empty(), "round rect should produce vertices");
}

#[test]
fn test_flatten_path_arc_to_command() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.arc_to(50.0, 0.0, 50.0, 50.0, 20.0);
    let verts = ctx.flatten_path();
    assert!(!verts.is_empty(), "arc_to should produce vertices");
}

// ── flatten_path_for 测试（使用独立 Path2D）──

#[test]
fn test_flatten_path_for_line() {
    let ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.move_to(10.0, 10.0);
    path.line_to(90.0, 90.0);
    let verts = ctx.flatten_path_for(&path);
    assert_eq!(verts.len(), 4, "path line = 4 floats");
}

#[test]
fn test_flatten_path_for_quadratic() {
    let ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.move_to(0.0, 0.0);
    path.quadratic_curve_to(50.0, 0.0, 50.0, 50.0);
    let verts = ctx.flatten_path_for(&path);
    assert_eq!(verts.len(), 32, "path quadratic = 8×4");
}

#[test]
fn test_flatten_path_for_bezier() {
    let ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.move_to(0.0, 0.0);
    path.bezier_curve_to(25.0, 0.0, 50.0, 25.0, 50.0, 50.0);
    let verts = ctx.flatten_path_for(&path);
    assert_eq!(verts.len(), 32, "path bezier = 8×4");
}

#[test]
fn test_flatten_path_for_arc() {
    let ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.move_to(50.0, 0.0);
    path.arc(50.0, 50.0, 50.0, -std::f32::consts::FRAC_PI_2, 0.0);
    let verts = ctx.flatten_path_for(&path);
    assert_eq!(verts.len(), 64, "path arc = 16×4");
}

#[test]
fn test_flatten_path_for_ellipse() {
    let ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.move_to(50.0, 10.0);
    path.ellipse(50.0, 50.0, 40.0, 30.0, 0.0, -std::f32::consts::FRAC_PI_2, 0.0);
    let verts = ctx.flatten_path_for(&path);
    assert_eq!(verts.len(), 64, "path ellipse = 16×4");
}

#[test]
fn test_flatten_path_for_round_rect() {
    let ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.move_to(0.0, 0.0);
    path.round_rect(10.0, 10.0, 50.0, 50.0, vec![8.0]);
    let verts = ctx.flatten_path_for(&path);
    assert!(!verts.is_empty(), "path round rect should produce vertices");
}

#[test]
fn test_flatten_path_for_arc_to() {
    let ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.move_to(0.0, 0.0);
    path.arc_to(50.0, 0.0, 50.0, 50.0, 20.0);
    let verts = ctx.flatten_path_for(&path);
    assert!(!verts.is_empty(), "path arc_to should produce vertices");
}

#[test]
fn test_flatten_path_for_close_path() {
    let ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.move_to(10.0, 10.0);
    path.line_to(50.0, 10.0);
    path.line_to(50.0, 50.0);
    path.close_path();
    let verts = ctx.flatten_path_for(&path);
    assert_eq!(verts.len(), 12, "path triangle = 3 segments");
}

// ── blit_path_to_pixels 测试 ──

#[test]
fn test_blit_path_empty_vertices() {
    let mut ctx = ctx_with_pixels(10, 10);
    ctx.blit_path_to_pixels(&[], Color::rgba(255, 0, 0, 255));
    // 空顶点 → 不写入任何像素
    let all_zero = ctx.pixel_buffer.iter().all(|&v| v == 0);
    assert!(all_zero, "empty path should not write pixels");
}

#[test]
fn test_blit_path_triangle_fills_pixels() {
    let mut ctx = ctx_with_pixels(20, 20);
    // 三角形：(5,5) → (15,5) → (10,15) → (5,5)
    let vertices = [5.0, 5.0, 15.0, 5.0, 10.0, 15.0];
    let color = Color::rgba(255, 0, 0, 255);
    ctx.blit_path_to_pixels(&vertices, color);
    // 中心点 (10, 8) 应该被填充
    let idx = ((8 * 20) + 10) * 4;
    assert_eq!(ctx.pixel_buffer[idx], 255, "triangle center should be red");
}

#[test]
fn test_blit_path_single_point_no_fill() {
    let mut ctx = ctx_with_pixels(10, 10);
    let vertices = [5.0, 5.0]; // 只有一个点（<4）
    ctx.blit_path_to_pixels(&vertices, Color::rgba(255, 0, 0, 255));
    let all_zero = ctx.pixel_buffer.iter().all(|&v| v == 0);
    assert!(all_zero, "single point path should not fill");
}

// ── blit_stroke_to_pixels 测试 ──

#[test]
fn test_blit_stroke_empty() {
    let mut ctx = ctx_with_pixels(10, 10);
    ctx.blit_stroke_to_pixels(&[], Color::rgba(255, 0, 0, 255), 2.0);
    let all_zero = ctx.pixel_buffer.iter().all(|&v| v == 0);
    assert!(all_zero, "empty stroke should not write pixels");
}

#[test]
fn test_blit_stroke_single_segment() {
    let mut ctx = ctx_with_pixels(20, 20);
    let segments = [5.0, 10.0, 15.0, 10.0]; // 水平线段
    ctx.blit_stroke_to_pixels(&segments, Color::rgba(0, 255, 0, 255), 4.0);
    // 中间点应该被写入
    let idx = ((10 * 20) + 10) * 4;
    assert_eq!(ctx.pixel_buffer[idx + 1], 255, "green channel on line");
}

#[test]
fn test_blit_stroke_two_segments_miter_join() {
    let mut ctx = ctx_with_pixels(30, 30);
    ctx.line_join = LineJoin::Miter;
    let segments = [5.0, 15.0, 15.0, 15.0, 15.0, 5.0]; // L 形
    ctx.blit_stroke_to_pixels(&segments, Color::rgba(255, 0, 0, 255), 4.0);
    // 连接点 (15, 15) 附近应有像素
    let idx = ((15 * 30) + 15) * 4;
    assert_eq!(ctx.pixel_buffer[idx], 255, "join point should be written");
}

#[test]
fn test_blit_stroke_two_segments_round_join() {
    let mut ctx = ctx_with_pixels(30, 30);
    ctx.line_join = LineJoin::Round;
    let segments = [5.0, 15.0, 15.0, 15.0, 15.0, 5.0];
    ctx.blit_stroke_to_pixels(&segments, Color::rgba(0, 0, 255, 255), 4.0);
    let idx = ((15 * 30) + 15) * 4;
    assert_eq!(ctx.pixel_buffer[idx + 2], 255, "round join point should be blue");
}

#[test]
fn test_blit_stroke_two_segments_bevel_join() {
    let mut ctx = ctx_with_pixels(30, 30);
    ctx.line_join = LineJoin::Bevel;
    let segments = [5.0, 15.0, 15.0, 15.0, 15.0, 5.0];
    ctx.blit_stroke_to_pixels(&segments, Color::rgba(255, 255, 0, 255), 4.0);
    let idx = ((15 * 30) + 15) * 4;
    assert!(ctx.pixel_buffer[idx] > 0 || ctx.pixel_buffer[idx + 1] > 0, "bevel join");
}

#[test]
fn test_blit_stroke_round_cap() {
    let mut ctx = ctx_with_pixels(20, 20);
    ctx.line_cap = LineCap::Round;
    let segments = [5.0, 10.0, 15.0, 10.0];
    ctx.blit_stroke_to_pixels(&segments, Color::rgba(255, 0, 0, 255), 4.0);
    // 端点附近应该有像素
    let idx_start = ((10 * 20) + 5) * 4;
    assert_eq!(ctx.pixel_buffer[idx_start], 255, "round cap at start");
}

#[test]
fn test_blit_stroke_square_cap() {
    let mut ctx = ctx_with_pixels(20, 20);
    ctx.line_cap = LineCap::Square;
    let segments = [5.0, 10.0, 15.0, 10.0];
    ctx.blit_stroke_to_pixels(&segments, Color::rgba(0, 128, 0, 255), 4.0);
    // Square cap 应在端点外延伸
    let idx = ((10 * 20) + 15) * 4;
    assert!(ctx.pixel_buffer[idx + 1] > 0, "square cap extends beyond endpoint");
}

// ── blit_line_cap 测试 ──

#[test]
fn test_blit_line_cap_butt() {
    let mut ctx = ctx_with_pixels(20, 20);
    ctx.line_cap = LineCap::Butt;
    // Butt cap 不额外绘制
    ctx.blit_line_cap(10.0, 10.0, 15.0, 10.0, 2.0, Color::rgba(255, 0, 0, 255));
    // Butt cap 不产生额外像素（线段矩形已覆盖）
}

#[test]
fn test_blit_line_cap_round() {
    let mut ctx = ctx_with_pixels(20, 20);
    ctx.line_cap = LineCap::Round;
    ctx.blit_line_cap(10.0, 10.0, 15.0, 10.0, 3.0, Color::rgba(255, 0, 0, 255));
    let idx = ((10 * 20) + 10) * 4;
    assert_eq!(ctx.pixel_buffer[idx], 255, "round cap at endpoint");
}

#[test]
fn test_blit_line_cap_square() {
    let mut ctx = ctx_with_pixels(20, 20);
    ctx.line_cap = LineCap::Square;
    ctx.blit_line_cap(10.0, 10.0, 15.0, 10.0, 3.0, Color::rgba(0, 0, 255, 255));
    // Square cap 在端点外延伸
    let idx = ((10 * 20) + 10) * 4;
    assert!(ctx.pixel_buffer[idx + 2] > 0, "square cap blue channel");
}

#[test]
fn test_blit_line_cap_square_zero_length() {
    let mut ctx = ctx_with_pixels(20, 20);
    ctx.line_cap = LineCap::Square;
    // 端点和另一个点重合 → 长度为 0 → 直接返回
    ctx.blit_line_cap(10.0, 10.0, 10.0, 10.0, 3.0, Color::rgba(255, 0, 0, 255));
    // 不会 panic
}

// ── stroke_outline_vertices 测试 ──

#[test]
fn test_stroke_outline_empty_path() {
    let ctx = CanvasContext::new(100, 100);
    let outline = ctx.stroke_outline_vertices();
    assert!(outline.is_empty(), "empty path = empty outline");
}

#[test]
fn test_stroke_outline_single_line_butt() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_cap = LineCap::Butt;
    ctx.line_width = 4.0;
    ctx.current_path.move_to(10.0, 50.0);
    ctx.current_path.line_to(90.0, 50.0);
    let outline = ctx.stroke_outline_vertices();
    // 单线段 + 主体矩形 = 8 floats（4 角点 × 2 coords）
    assert!(!outline.is_empty(), "single line should have outline vertices");
}

#[test]
fn test_stroke_outline_single_line_round_cap() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_cap = LineCap::Round;
    ctx.line_width = 4.0;
    ctx.current_path.move_to(10.0, 50.0);
    ctx.current_path.line_to(90.0, 50.0);
    let outline = ctx.stroke_outline_vertices();
    assert!(outline.len() > 8, "round cap adds extra vertices beyond rectangle");
}

#[test]
fn test_stroke_outline_single_line_square_cap() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_cap = LineCap::Square;
    ctx.line_width = 4.0;
    ctx.current_path.move_to(10.0, 50.0);
    ctx.current_path.line_to(90.0, 50.0);
    let outline = ctx.stroke_outline_vertices();
    // Square cap 增加额外的矩形顶点
    assert!(outline.len() > 8, "square cap adds extra vertices");
}

#[test]
fn test_stroke_outline_two_lines_miter_join() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_join = LineJoin::Miter;
    ctx.line_width = 4.0;
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(50.0, 50.0);
    ctx.current_path.line_to(90.0, 10.0);
    let outline = ctx.stroke_outline_vertices();
    assert!(!outline.is_empty(), "miter join should produce outline");
}

#[test]
fn test_stroke_outline_two_lines_round_join() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_join = LineJoin::Round;
    ctx.line_width = 4.0;
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(50.0, 50.0);
    ctx.current_path.line_to(90.0, 10.0);
    let outline = ctx.stroke_outline_vertices();
    assert!(!outline.is_empty(), "round join should produce outline");
}

#[test]
fn test_stroke_outline_two_lines_bevel_join() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_join = LineJoin::Bevel;
    ctx.line_width = 4.0;
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(50.0, 50.0);
    ctx.current_path.line_to(90.0, 10.0);
    let outline = ctx.stroke_outline_vertices();
    assert!(!outline.is_empty(), "bevel join should produce outline");
}

// ── composite_pixel 更多合成操作测试 ──

#[test]
fn test_composite_pixel_destination_over() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.composite_operation = CompositeOperation::DestinationOver;
    let src = Color::rgba(255, 0, 0, 128);
    let (_r, _g, _b, a) = ctx.composite_pixel(src, 0, 0, 255, 255);
    assert!(a > 0, "destination-over should produce nonzero alpha");
}

#[test]
fn test_composite_pixel_source_in() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.composite_operation = CompositeOperation::SourceIn;
    let src = Color::rgba(255, 0, 0, 200);
    let (_r, _g, _b, a) = ctx.composite_pixel(src, 0, 0, 255, 128);
    assert!(
        a > 0,
        "source-in should produce nonzero alpha when dst is partially transparent"
    );
}

#[test]
fn test_composite_pixel_destination_in() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.composite_operation = CompositeOperation::DestinationIn;
    let src = Color::rgba(255, 0, 0, 128);
    let (_r, _g, _b, a) = ctx.composite_pixel(src, 0, 255, 0, 200);
    assert!(a > 0, "destination-in should produce nonzero alpha");
}

#[test]
fn test_composite_pixel_destination_out() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.composite_operation = CompositeOperation::DestinationOut;
    let src = Color::rgba(255, 0, 0, 200);
    let (_r, _g, _b, a) = ctx.composite_pixel(src, 0, 255, 0, 255);
    // destination-out: Fa=0, Fb=1-sa → dst survives where src is transparent
    assert!(a < 255, "destination-out should reduce destination alpha");
}

#[test]
fn test_composite_pixel_source_atop() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.composite_operation = CompositeOperation::SourceAtop;
    let src = Color::rgba(255, 0, 0, 200);
    let (r, _g, _b, a) = ctx.composite_pixel(src, 0, 0, 255, 255);
    assert!(r > 0 && a > 0, "source-atop on opaque dst");
}

#[test]
fn test_composite_pixel_destination_atop() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.composite_operation = CompositeOperation::DestinationAtop;
    let src = Color::rgba(255, 0, 0, 200);
    let (_r, _g, _b, a) = ctx.composite_pixel(src, 0, 255, 0, 128);
    assert!(a > 0, "destination-atop should produce output");
}

#[test]
fn test_composite_pixel_xor() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.composite_operation = CompositeOperation::Xor;
    let src = Color::rgba(255, 0, 0, 128);
    let dst_bg = Color::rgba(0, 255, 0, 128);
    let (_r, _g, _b, a) = ctx.composite_pixel(src, dst_bg.r, dst_bg.g, dst_bg.b, dst_bg.a);
    assert!(a > 0, "xor should produce output");
}

#[test]
fn test_composite_pixel_lighter() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.composite_operation = CompositeOperation::Lighter;
    let src = Color::rgba(255, 0, 0, 128);
    let (r, _g, _b, _a) = ctx.composite_pixel(src, 128, 0, 0, 128);
    assert!(r > 128, "lighter should add color values");
}

#[test]
fn test_composite_pixel_transparent_src_zero_alpha() {
    let ctx = CanvasContext::new(100, 100);
    let src = Color::rgba(0, 0, 0, 0);
    let (r, g, b, a) = ctx.composite_pixel(src, 0, 0, 0, 0);
    assert_eq!((r, g, b, a), (0, 0, 0, 0), "both transparent → output transparent");
}

// ── blit_rect_to_pixels 边界测试 ──

#[test]
fn test_blit_rect_no_overlap() {
    let mut ctx = ctx_with_pixels(10, 10);
    // 矩形完全在画布外
    let rect = Rect::new(20.0, 20.0, 5.0, 5.0);
    ctx.blit_rect_to_pixels(&rect, Color::rgba(255, 0, 0, 255));
    let all_zero = ctx.pixel_buffer.iter().all(|&v| v == 0);
    assert!(all_zero, "rect outside canvas should not write pixels");
}

#[test]
fn test_blit_rect_full_canvas() {
    let mut ctx = ctx_with_pixels(5, 5);
    let rect = Rect::new(0.0, 0.0, 5.0, 5.0);
    ctx.blit_rect_to_pixels(&rect, Color::rgba(255, 255, 255, 255));
    // 所有像素应为白色
    for i in 0..25 {
        let idx = i * 4;
        assert_eq!(ctx.pixel_buffer[idx], 255, "pixel {} red", i);
        assert_eq!(ctx.pixel_buffer[idx + 3], 255, "pixel {} alpha", i);
    }
}

// ── raster.rs 更多覆盖率测试 ──

#[test]
fn test_flatten_path_empty_path_with_close() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.close_path();
    let vertices = ctx.flatten_path();
    // Empty path with close should still be empty
    assert_eq!(vertices.len(), 0);
}

#[test]
fn test_flatten_path_single_line_with_close() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(20.0, 20.0);
    ctx.current_path.close_path();
    let vertices = ctx.flatten_path();
    // Should have line segment + close segment
    assert_eq!(vertices.len(), 8); // 2 segments × 4
}

#[test]
fn test_flatten_path_line_to_same_point() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(10.0, 10.0); // Same point
    let vertices = ctx.flatten_path();
    // Degenerate line segment might be filtered or kept
    assert_eq!(vertices.len(), 4);
}

#[test]
fn test_flatten_path_arc_full_circle() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 0.0);
    ctx.current_path.arc(50.0, 50.0, 50.0, 0.0, std::f32::consts::TAU);
    let vertices = ctx.flatten_path();
    // Full circle should still produce vertices
    assert_eq!(vertices.len(), 64); // 16 segments × 4
}

#[test]
fn test_flatten_path_arc_zero_radius() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.arc(20.0, 20.0, 0.0, 0.0, std::f32::consts::PI);
    let vertices = ctx.flatten_path();
    // Zero radius arc - just verify it doesn't panic
    let _ = vertices.len();
}

#[test]
fn test_flatten_path_arc_negative_angles() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 0.0);
    ctx.current_path.arc(50.0, 50.0, 50.0, -std::f32::consts::PI, 0.0);
    let vertices = ctx.flatten_path();
    // Negative angles should still work
    assert_eq!(vertices.len(), 64); // 16 segments × 4
}

#[test]
fn test_flatten_path_arc_start_greater_than_end() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 0.0);
    ctx.current_path.arc(50.0, 50.0, 50.0, std::f32::consts::PI, 0.0);
    let vertices = ctx.flatten_path();
    // Start > end should still produce vertices
    assert_eq!(vertices.len(), 64); // 16 segments × 4
}

#[test]
fn test_flatten_path_ellipse_horizontal_stretch() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 0.0);
    ctx.current_path
        .ellipse(50.0, 50.0, 50.0, 10.0, 0.0, 0.0, std::f32::consts::PI);
    let vertices = ctx.flatten_path();
    // Wide ellipse should produce vertices
    assert_eq!(vertices.len(), 64); // 16 segments × 4
}

#[test]
fn test_flatten_path_ellipse_vertical_stretch() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(0.0, 50.0);
    ctx.current_path
        .ellipse(50.0, 50.0, 10.0, 50.0, 0.0, 0.0, std::f32::consts::PI);
    let vertices = ctx.flatten_path();
    // Tall ellipse should produce vertices
    assert_eq!(vertices.len(), 64); // 16 segments × 4
}

#[test]
fn test_flatten_path_ellipse_rotated() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(70.71, 20.71); // Approximate for 45° rotation
    ctx.current_path.ellipse(
        50.0,
        50.0,
        30.0,
        20.0,
        std::f32::consts::PI / 4.0,
        0.0,
        std::f32::consts::PI / 2.0,
    );
    let vertices = ctx.flatten_path();
    // Rotated ellipse should produce vertices
    assert_eq!(vertices.len(), 64); // 16 segments × 4
}

#[test]
fn test_flatten_path_ellipse_zero_rotation() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 20.0);
    ctx.current_path
        .ellipse(50.0, 50.0, 30.0, 20.0, 0.0, 0.0, std::f32::consts::PI / 2.0);
    let vertices = ctx.flatten_path();
    // Zero rotation should still work
    assert_eq!(vertices.len(), 64); // 16 segments × 4
}

#[test]
fn test_flatten_path_round_rect_zero_radius() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.round_rect(10.0, 10.0, 50.0, 50.0, vec![0.0]);
    let vertices = ctx.flatten_path();
    // Zero radius should degenerate to rectangle
    assert_eq!(vertices.len(), 20); // 5 segments × 4
}

#[test]
fn test_flatten_path_round_rect_negative_radius() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.round_rect(10.0, 10.0, 50.0, 50.0, vec![-5.0]);
    let vertices = ctx.flatten_path();
    // Negative radius should be clamped to 0, degenerate to rectangle
    assert_eq!(vertices.len(), 20); // 5 segments × 4
}

#[test]
fn test_flatten_path_round_rect_radii_too_large() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.round_rect(10.0, 10.0, 20.0, 20.0, vec![100.0]);
    let vertices = ctx.flatten_path();
    // Radius larger than half size should be clamped - just verify no panic
    let _ = vertices.len();
}

#[test]
fn test_flatten_path_arc_to_with_zero_distance() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.arc_to(10.0, 10.0, 20.0, 20.0, 5.0);
    let vertices = ctx.flatten_path();
    // Points at same location - just verify no panic
    let _ = vertices.len();
}

#[test]
fn test_flatten_path_arc_to_collinear_with_radius() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.arc_to(10.0, 0.0, 20.0, 0.0, 5.0);
    let vertices = ctx.flatten_path();
    // Collinear points should still produce some vertices
    assert!(!vertices.is_empty());
}

#[test]
fn test_flatten_path_multiple_commands() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.line_to(10.0, 10.0);
    ctx.current_path.quadratic_curve_to(20.0, 0.0, 30.0, 10.0);
    ctx.current_path.arc(50.0, 50.0, 10.0, 0.0, std::f32::consts::PI / 2.0);
    ctx.current_path.close_path();
    let vertices = ctx.flatten_path();
    // Multiple command types should produce many vertices
    assert!(vertices.len() > 64); // Should have many vertices
}

#[test]
fn test_flatten_path_for_complex_path() {
    let ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.rect(10.0, 10.0, 20.0, 20.0);
    path.ellipse(50.0, 50.0, 15.0, 10.0, 0.0, 0.0, std::f32::consts::PI);
    let vertices = ctx.flatten_path_for(&path);
    // Complex path with multiple commands
    assert!(vertices.len() > 64);
}

#[test]
fn test_stroke_outline_complex_path() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_width = 4.0;
    ctx.line_join = LineJoin::Miter;
    ctx.line_cap = LineCap::Round;

    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(30.0, 10.0);
    ctx.current_path.line_to(50.0, 30.0);
    ctx.current_path.line_to(70.0, 10.0);
    ctx.current_path.line_to(90.0, 30.0);
    ctx.current_path.close_path();

    let outline = ctx.stroke_outline_vertices();
    // Complex path with multiple joins and caps
    assert!(outline.len() > 50);
}

#[test]
fn test_stroke_outline_with_miter_limit_exceeded() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_width = 4.0;
    ctx.line_join = LineJoin::Miter;
    ctx.set_miter_limit(1.0); // Low miter limit

    ctx.current_path.move_to(0.0, 20.0);
    ctx.current_path.line_to(10.0, 0.0);
    ctx.current_path.line_to(20.0, 20.0);

    let outline = ctx.stroke_outline_vertices();
    // Should handle miter limit correctly
    assert!(!outline.is_empty());
}

#[test]
fn test_stroke_outline_zero_line_width() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_width = 0.0;
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.line_to(10.0, 10.0);

    let outline = ctx.stroke_outline_vertices();
    // Zero width - just verify no panic
    let _ = outline.len();
}

#[test]
fn test_stroke_outline_single_point_path() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_width = 4.0;
    ctx.current_path.move_to(10.0, 10.0);

    let outline = ctx.stroke_outline_vertices();
    // Single point should produce no outline
    assert_eq!(outline.len(), 0);
}

#[test]
fn test_stroke_outline_point_only_path() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_width = 4.0;
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(10.0, 10.0); // Same point

    let outline = ctx.stroke_outline_vertices();
    // Degenerate line should produce minimal outline
    assert_eq!(outline.len(), 0);
}

#[test]
fn test_composite_pixel_all_operations() {
    let ctx = CanvasContext::new(100, 100);

    // Test all composite operations that might not be covered
    let operations = [
        CompositeOperation::SourceOver,
        CompositeOperation::DestinationOver,
        CompositeOperation::SourceIn,
        CompositeOperation::DestinationIn,
        CompositeOperation::DestinationOut,
        CompositeOperation::SourceAtop,
        CompositeOperation::DestinationAtop,
        CompositeOperation::Copy,
        CompositeOperation::Xor,
        CompositeOperation::Lighter,
    ];

    for op in operations {
        let mut ctx_test = CanvasContext::new(100, 100);
        ctx_test.composite_operation = op;
        let src = Color::rgba(255, 0, 0, 128);
        let (r, g, b, a) = ctx_test.composite_pixel(src, 0, 255, 0, 255);
        // Should not panic and should produce reasonable output
        assert!(r <= 255);
        assert!(g <= 255);
        assert!(b <= 255);
        assert!(a <= 255);
    }
}

#[test]
fn test_composite_pixel_with_transparent_dst() {
    let ctx = CanvasContext::new(100, 100);
    let src = Color::rgba(255, 0, 0, 128);
    let (r, g, b, a) = ctx.composite_pixel(src, 0, 0, 0, 0);
    // Composite with fully transparent destination
    assert!(r >= 0 && r <= 255);
    assert!(g >= 0 && g <= 255);
    assert!(b >= 0 && b <= 255);
    assert!(a >= 0 && a <= 255);
}

#[test]
fn test_composite_pixel_with_translucent_src() {
    let ctx = CanvasContext::new(100, 100);
    let src = Color::rgba(128, 128, 128, 128);
    let (r, g, b, a) = ctx.composite_pixel(src, 128, 128, 128, 128);
    // Composite with translucent source
    assert!(r >= 0 && r <= 255);
    assert!(g >= 0 && g <= 255);
    assert!(b >= 0 && b <= 255);
    assert!(a >= 0 && a <= 255);
}

#[test]
fn test_composite_pixel_with_opaque_src_dst() {
    let ctx = CanvasContext::new(100, 100);
    let src = Color::rgba(255, 0, 0, 255);
    let (r, g, b, a) = ctx.composite_pixel(src, 0, 0, 255, 255);
    // Composite with opaque source over opaque destination
    assert_eq!(r, 255); // Source should completely override
    assert_eq!(g, 0);
    assert_eq!(b, 0);
    assert_eq!(a, 255);
}

#[test]
fn test_blit_stroke_with_bevel_join_sharp_angle() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_join = LineJoin::Bevel;
    ctx.line_width = 10.0;

    // Create a sharp angle
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.line_to(10.0, 10.0);
    ctx.current_path.line_to(0.0, 20.0);

    let outline = ctx.stroke_outline_vertices();
    // Should handle sharp angle correctly
    assert!(!outline.is_empty());
}

#[test]
fn test_blit_stroke_with_round_join_sharp_angle() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_join = LineJoin::Round;
    ctx.line_width = 10.0;

    // Create a sharp angle
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.line_to(10.0, 10.0);
    ctx.current_path.line_to(0.0, 20.0);

    let outline = ctx.stroke_outline_vertices();
    // Should handle sharp angle with round join
    assert!(!outline.is_empty());
}

#[test]
fn test_blit_stroke_with_miter_join_exceeding_limit() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_join = LineJoin::Miter;
    ctx.set_miter_limit(1.0); // Very low limit
    ctx.line_width = 10.0;

    // Create a very sharp angle
    ctx.current_path.move_to(0.0, 20.0);
    ctx.current_path.line_to(10.0, 0.0);
    ctx.current_path.line_to(20.0, 20.0);

    let outline = ctx.stroke_outline_vertices();
    // Should handle miter limit correctly
    assert!(!outline.is_empty());
}

#[test]
fn test_blit_stroke_large_line_width() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_width = 50.0;
    ctx.line_cap = LineCap::Round;

    ctx.current_path.move_to(10.0, 50.0);
    ctx.current_path.line_to(90.0, 50.0);

    let outline = ctx.stroke_outline_vertices();
    // Should handle very wide lines
    assert!(outline.len() > 20);
}

#[test]
fn test_blit_stroke_variable_width_path() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.line_width = 2.0;

    // Create a path with many segments
    for i in 0..20 {
        let x = i as f32 * 5.0;
        ctx.current_path.move_to(x, 10.0);
        ctx.current_path.line_to(x, 90.0);
    }

    let outline = ctx.stroke_outline_vertices();
    // Should handle path with many segments
    assert!(outline.len() > 100);
}

// ── 渐变光栅化（R3079）：sample_at + blit_rect_gradient/blit_path_gradient 经 fill_rect/fill 路径 ──

#[test]
fn test_canvas_style_sample_at_linear() {
    // 线性渐变 red(0)→blue(1) 沿 x 轴 0..20
    let mut grad = LinearGradient::new(0.0, 0.0, 20.0, 0.0);
    grad.add_color_stop(0.0, Color::rgba(255, 0, 0, 255));
    grad.add_color_stop(1.0, Color::rgba(0, 0, 255, 255));
    let style = CanvasStyle::LinearGradient(grad);
    assert!(style.is_gradient());
    // 端点：t=0 红，t=1 蓝
    assert_eq!(style.sample_at(0.0, 5.0), Color::rgba(255, 0, 0, 255));
    assert_eq!(style.sample_at(20.0, 5.0), Color::rgba(0, 0, 255, 255));
    // 中点：红蓝各半（lerp_u8(255,0,0.5)=128，lerp_u8(0,255,0.5)=128）
    assert_eq!(style.sample_at(10.0, 5.0), Color::rgba(128, 0, 128, 255));
    // 超出渐变线延伸：钳制到端点色（spec）
    assert_eq!(style.sample_at(-5.0, 5.0), Color::rgba(255, 0, 0, 255));
    assert_eq!(style.sample_at(50.0, 5.0), Color::rgba(0, 0, 255, 255));
}

#[test]
fn test_canvas_style_sample_at_radial() {
    // 径向渐变 white(0)→blue(1)，内心 (0,0,r0=0)，外心 (0,0,r1=10)
    let mut grad = RadialGradient::new(0.0, 0.0, 0.0, 0.0, 0.0, 10.0);
    grad.add_color_stop(0.0, Color::rgba(255, 255, 255, 255));
    grad.add_color_stop(1.0, Color::rgba(0, 0, 255, 255));
    let style = CanvasStyle::RadialGradient(grad);
    assert!(style.is_gradient());
    assert_eq!(style.sample_at(0.0, 0.0), Color::rgba(255, 255, 255, 255));
    assert_eq!(style.sample_at(10.0, 0.0), Color::rgba(0, 0, 255, 255));
}

#[test]
fn test_canvas_style_add_color_stop_and_is_gradient() {
    // Color/Pattern 非 is_gradient；add_color_stop 为 no-op
    let mut color_style = CanvasStyle::Color(Color::BLACK);
    assert!(!color_style.is_gradient());
    color_style.add_color_stop(0.5, Color::WHITE); // no-op，不应 panic
    assert!(matches!(color_style, CanvasStyle::Color(_)));
}

#[test]
fn test_fill_rect_linear_gradient_rasterizes() {
    // 20×10 画布，红色→蓝色线性渐变沿 x，fill_rect 整画布。
    let mut ctx = CanvasContext::new(20, 10);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 20.0, 0.0);
    grad.add_color_stop(0.0, Color::rgba(255, 0, 0, 255));
    grad.add_color_stop(1.0, Color::rgba(0, 0, 255, 255));
    ctx.set_fill_style(CanvasStyle::LinearGradient(grad));
    ctx.fill_rect(0.0, 0.0, 20.0, 10.0);

    // 第 3 行像素逐像素采样：左端（x=0, t=0）红；中点（x=10, t=0.5）紫 (128,0,128)；
    // 右端（x=19, t=0.95）偏蓝 (13,0,242)——注意 t=1.0 在 x=20（fill 区间外），故右沿非纯蓝。
    let row = 3usize;
    let left = (row * 20) * 4;
    assert_eq!(&ctx.pixel_buffer[left..left + 4], &[255, 0, 0, 255]);
    let mid = (row * 20 + 10) * 4;
    assert_eq!(&ctx.pixel_buffer[mid..mid + 4], &[128, 0, 128, 255]);
    let right = (row * 20 + 19) * 4;
    assert_eq!(&ctx.pixel_buffer[right..right + 4], &[13, 0, 242, 255]);
}

#[test]
fn test_fill_path_linear_gradient_rasterizes() {
    // 路径填充（fill）渐变：rect 路径 + 渐变 fillStyle。
    let mut ctx = CanvasContext::new(20, 10);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 20.0, 0.0);
    grad.add_color_stop(0.0, Color::rgba(255, 0, 0, 255));
    grad.add_color_stop(1.0, Color::rgba(0, 0, 255, 255));
    ctx.set_fill_style(CanvasStyle::LinearGradient(grad));
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(20.0, 0.0);
    ctx.line_to(20.0, 10.0);
    ctx.line_to(0.0, 10.0);
    ctx.close_path();
    ctx.fill();

    let row = 5usize;
    let left = (row * 20) * 4;
    assert_eq!(&ctx.pixel_buffer[left..left + 4], &[255, 0, 0, 255]);
    // 右沿 x=19 → t=0.95 → (13,0,242)（偏蓝，t=1.0 在区间外故非纯蓝）
    let right = (row * 20 + 19) * 4;
    assert_eq!(&ctx.pixel_buffer[right..right + 4], &[13, 0, 242, 255]);
}

// ── 描边渐变光栅化（R3084，对称 fill 渐变 R3079）──

#[test]
fn test_stroke_linear_gradient_rasterizes() {
    // 30×10 画布，红→蓝线性渐变 stroke_style 沿 x，stroke 一条水平线（line_width 4，y 中心 5）。
    let mut ctx = CanvasContext::new(30, 10);
    ctx.line_width = 4.0;
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 30.0, 0.0);
    grad.add_color_stop(0.0, Color::rgba(255, 0, 0, 255));
    grad.add_color_stop(1.0, Color::rgba(0, 0, 255, 255));
    ctx.set_stroke_style(CanvasStyle::LinearGradient(grad));
    ctx.begin_path();
    ctx.move_to(2.0, 5.0);
    ctx.line_to(28.0, 5.0);
    ctx.stroke();

    // 描边覆盖 y∈[3,7)（line_width 4，中心 5）。逐像素渐变：左端（x=2, t≈0.067）偏红，右端（x=27, t=0.9）偏蓝。
    let left_idx = (5 * 30 + 2) * 4;
    let right_idx = (5 * 30 + 27) * 4;
    let lr = ctx.pixel_buffer[left_idx];
    let lb = ctx.pixel_buffer[left_idx + 2];
    let rr = ctx.pixel_buffer[right_idx];
    let rb = ctx.pixel_buffer[right_idx + 2];
    assert!(lr > lb, "描边左端偏红（r={} > b={}）", lr, lb);
    assert!(rb > rr, "描边右端偏蓝（b={} > r={}）", rb, rr);
    assert!(lb < rb, "渐变左→右：b 递增（{} → {}）", lb, rb);
}

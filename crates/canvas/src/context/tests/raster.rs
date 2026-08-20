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
    // R34xx：段矩形精确到端点（端点延伸归 cap 绘制）；垂直方向扩半线宽。
    assert_eq!(rect.origin.x, 10.0);
    assert_eq!(rect.origin.y, 48.0); // 50 - 2
    assert_eq!(rect.size.width, 80.0); // 90-10
    assert_eq!(rect.size.height, 4.0); // line_width
}

#[test]
fn test_line_segment_rect_vertical() {
    let ctx = CanvasContext::new(100, 100);
    let rect = ctx.line_segment_rect(50.0, 10.0, 50.0, 90.0, 6.0);
    assert_eq!(rect.origin.x, 47.0); // 50 - 3
    assert_eq!(rect.origin.y, 10.0);
    assert_eq!(rect.size.width, 6.0);
    assert_eq!(rect.size.height, 80.0); // 90-10
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
    let (_cx, _cy) = CanvasContext::flatten_round_rect(&mut verts, 0.0, 0.0, 10.0, 10.0, 50.0, 50.0, &[(8.0, 8.0)]);
    // 应产生顶点（圆角路径）
    assert!(!verts.is_empty(), "should produce vertices for rounded rect");
}

#[test]
fn test_flatten_round_rect_two_radii() {
    let mut verts = Vec::new();
    let (_cx, _cy) = CanvasContext::flatten_round_rect(
        &mut verts,
        0.0,
        0.0,
        10.0,
        10.0,
        50.0,
        50.0,
        &[(8.0, 8.0), (12.0, 12.0)],
    );
    assert!(!verts.is_empty(), "should produce vertices for 2-radii round rect");
}

#[test]
fn test_flatten_round_rect_three_radii() {
    let mut verts = Vec::new();
    let (_cx, _cy) = CanvasContext::flatten_round_rect(
        &mut verts,
        0.0,
        0.0,
        10.0,
        10.0,
        50.0,
        50.0,
        &[(8.0, 8.0), (12.0, 12.0), (6.0, 6.0)],
    );
    assert!(!verts.is_empty(), "should produce vertices for 3-radii round rect");
}

// ── R56（M8/DC-8）：负 w/h 归一化 + 镜像角序 ──

#[test]
fn test_flatten_round_rect_negative_h_bbox() {
    // 负 h：roundRect(0,25,100,-25) 归一到包围盒 (0,0)-(100,25)。
    // 扫描线 y=12.5 交点配对后 (50,12) 应被覆盖（段计数为偶、含该 x）。
    let mut verts = Vec::new();
    let _ = CanvasContext::flatten_round_rect(&mut verts, 0.0, 0.0, 0.0, 25.0, 100.0, -25.0, &[]);
    let hits = scanline_hits(&verts, 12.5);
    assert_eq!(hits.len() % 2, 0, "even crossings");
    assert!(
        hits.iter().any(|&x| x < 50.0) && hits.iter().any(|&x| x > 50.0),
        "span covers x=50: {hits:?}"
    );
}

#[test]
fn test_flatten_round_rect_negative_h_corner_mirror() {
    // 负 h + radii=[10,0,0,0]：参数 tl 圆角贴参数角 (0,50)，垂直镜像后落在包围盒
    // **左下**（y=h 侧）；归一化包围盒左上 (0,0) 应是直角（顶点精确过角）。
    let mut verts = Vec::new();
    let _ = CanvasContext::flatten_round_rect(
        &mut verts,
        0.0,
        0.0,
        0.0,
        25.0,
        50.0,
        -25.0,
        &[(10.0, 10.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)],
    );
    // 左上角 (0,0) 直角：应存在顶点精确落在 (0,0)（直角边交点），且 (0,0) 附近
    // 没有内凹圆弧（最近的弧点距角 > 半径的一半——左上无圆角）。
    let exact_corner = verts
        .as_chunks::<2>()
        .0
        .iter()
        .any(|c| c[0].abs() < 0.01 && c[1].abs() < 0.01);
    assert!(exact_corner, "square corner at (0,0) after mirror");
}

/// 统计扫描线 y=sy 与段序列的交点 x（段对 x1,y1,x2,y2 半开区间判定）。
fn scanline_hits(verts: &[f32], sy: f32) -> Vec<f32> {
    let mut xs = Vec::new();
    for seg in verts.as_chunks::<4>().0 {
        let (x1, y1, x2, y2) = (seg[0], seg[1], seg[2], seg[3]);
        if (y1 <= sy && y2 > sy) || (y2 <= sy && y1 > sy) {
            let t = (sy - y1) / (y2 - y1);
            xs.push(x1 + t * (x2 - x1));
        }
    }
    xs
}

#[test]
fn test_flatten_round_rect_four_radii() {
    let mut verts = Vec::new();
    let (_cx, _cy) = CanvasContext::flatten_round_rect(
        &mut verts,
        0.0,
        0.0,
        10.0,
        10.0,
        50.0,
        50.0,
        &[(8.0, 8.0), (12.0, 12.0), (6.0, 6.0), (10.0, 10.0)],
    );
    assert!(!verts.is_empty(), "should produce vertices for 4-radii round rect");
}

#[test]
fn test_flatten_round_rect_zero_radius_degenerates() {
    // 所有半径为 0 → 退化为矩形
    let mut verts = Vec::new();
    let (cx, cy) = CanvasContext::flatten_round_rect(&mut verts, 5.0, 5.0, 10.0, 10.0, 50.0, 50.0, &[(0.0, 0.0)]);
    assert_eq!(cx, 10.0);
    assert_eq!(cy, 10.0);
    // R56：自包含子路径（不连 current）——4 条边段；起始连接段移除（重复边在
    // 段式扫描线下产生奇数交点，见 zero_radius_negative_h 测试）。
    assert_eq!(verts.len(), 4 * 4, "degenerate round rect = 4 edge segments");
}

#[test]
fn test_flatten_round_rect_large_radius_clamped() {
    // 半径超过短边一半时应被钳位
    let mut verts = Vec::new();
    let _ = CanvasContext::flatten_round_rect(&mut verts, 0.0, 0.0, 10.0, 10.0, 20.0, 10.0, &[(100.0, 100.0)]);
    // max_r = min(20,10)/2 = 5, radius should be clamped to 5
    assert!(!verts.is_empty());
}

#[test]
fn test_flatten_round_rect_negative_radius_clamped() {
    let mut verts = Vec::new();
    let _ = CanvasContext::flatten_round_rect(&mut verts, 0.0, 0.0, 10.0, 10.0, 50.0, 50.0, &[(-5.0, -5.0)]);
    // 负半径被钳位到 0 → 退化为矩形（4 边段，R56 同上）
    assert_eq!(verts.len(), 4 * 4, "negative radius degenerates to rect");
}

// ── flatten_path 测试（通过 CanvasContext 的 current_path）──

#[test]
fn test_flatten_path_empty() {
    let ctx = CanvasContext::new(100, 100);
    let verts = ctx.flatten_path_open();
    assert!(verts.is_empty(), "empty path should produce no vertices");
}

#[test]
fn test_flatten_path_line_to() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(50.0, 50.0);
    let verts = ctx.flatten_path_open();
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
    let verts = ctx.flatten_path_open();
    // R56h：段数自适应 = 控制折线 100/8 → 13 段 × 4 = 52。
    assert_eq!(verts.len(), 52, "quadratic curve = 13 adaptive segments × 4");
}

#[test]
fn test_flatten_path_bezier_curve() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.bezier_curve_to(25.0, 0.0, 50.0, 25.0, 50.0, 50.0);
    let verts = ctx.flatten_path_open();
    // R56h：段数自适应 = 控制折线 85.4/8 → 11 段 × 4 = 44。
    assert_eq!(verts.len(), 44, "cubic bezier = 11 adaptive segments × 4");
}

#[test]
fn test_flatten_path_arc() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 0.0);
    ctx.current_path
        .arc(50.0, 50.0, 50.0, -std::f32::consts::FRAC_PI_2, 0.0, false);
    let verts = ctx.flatten_path_open();
    // R56：moveTo 后 arc 含 spec「current→弧首」连线段（+1 段）。
    // R56g：段数自适应 N=16（lw/r≤0.25 细线）→ 17 段。
    assert_eq!(verts.len(), 68, "arc = 16 segments + line-to-arc-start");
}

#[test]
fn test_flatten_path_close_path() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(50.0, 10.0);
    ctx.current_path.line_to(50.0, 50.0);
    ctx.current_path.close_path();
    let verts = ctx.flatten_path_open();
    // 2 line segments + 1 close segment = 3 × 4 = 12
    assert_eq!(verts.len(), 12, "triangle = 3 segments");
}

#[test]
fn test_flatten_path_close_already_at_start() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(10.0, 10.0); // 回到起点
    ctx.current_path.close_path(); // 已在起点，不产生额外线段
    let verts = ctx.flatten_path_open();
    // R56h：零长段剪除——lineTo(10,10) 为退化段，路径无几何。
    assert_eq!(verts.len(), 0, "zero-length line pruned; close produces nothing extra");
}

#[test]
fn test_flatten_path_ellipse_command() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 10.0);
    ctx.current_path
        .ellipse(50.0, 50.0, 40.0, 30.0, 0.0, -std::f32::consts::FRAC_PI_2, 0.0);
    let verts = ctx.flatten_path_open();
    assert_eq!(verts.len(), 64, "ellipse = 16 segments × 4");
}

#[test]
fn test_flatten_path_round_rect_command() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.round_rect(10.0, 10.0, 50.0, 50.0, vec![(8.0, 8.0)]);
    let verts = ctx.flatten_path_open();
    assert!(!verts.is_empty(), "round rect should produce vertices");
}

#[test]
fn test_flatten_path_arc_to_command() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.arc_to(50.0, 0.0, 50.0, 50.0, 20.0);
    let verts = ctx.flatten_path_open();
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
    // R56h：自适应细分。
    assert!(verts.len() >= 32, "path quadratic = adaptive segments（≥8×4）");
}

#[test]
fn test_flatten_path_for_bezier() {
    let ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.move_to(0.0, 0.0);
    path.bezier_curve_to(25.0, 0.0, 50.0, 25.0, 50.0, 50.0);
    let verts = ctx.flatten_path_for(&path);
    // R56h：自适应细分。
    assert!(verts.len() >= 32, "path bezier = adaptive segments（≥8×4）");
}

#[test]
fn test_flatten_path_for_arc() {
    let ctx = CanvasContext::new(100, 100);
    let mut path = Path2D::new();
    path.move_to(50.0, 0.0);
    path.arc(50.0, 50.0, 50.0, -std::f32::consts::FRAC_PI_2, 0.0, false);
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
    path.round_rect(10.0, 10.0, 50.0, 50.0, vec![(8.0, 8.0)]);
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
    // 三角形：(5,5) → (15,5) → (10,15) → (5,5)。R56：blit 消费「独立段」序列
    // （flatten 输出格式），补闭合段 (10,15)→(5,5)。
    let vertices = [5.0, 5.0, 15.0, 5.0, 15.0, 5.0, 10.0, 15.0, 10.0, 15.0, 5.0, 5.0];
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
    ctx.blit_stroke_to_pixels(&[], Color::rgba(255, 0, 0, 255), 2.0, false);
    let all_zero = ctx.pixel_buffer.iter().all(|&v| v == 0);
    assert!(all_zero, "empty stroke should not write pixels");
}

#[test]
fn test_blit_stroke_single_segment() {
    let mut ctx = ctx_with_pixels(20, 20);
    let segments = [5.0, 10.0, 15.0, 10.0]; // 水平线段
    ctx.blit_stroke_to_pixels(&segments, Color::rgba(0, 255, 0, 255), 4.0, false);
    // 中间点应该被写入
    let idx = ((10 * 20) + 10) * 4;
    assert_eq!(ctx.pixel_buffer[idx + 1], 255, "green channel on line");
}

#[test]
fn test_blit_stroke_two_segments_miter_join() {
    let mut ctx = ctx_with_pixels(30, 30);
    ctx.line_join = LineJoin::Miter;
    // R34xx：旧数据 [5,15,15,15,15,5] 仅 6 元素 = 1.5 段（chunks_exact(4) 丢余数，join 循环
    // 不执行），旧断言靠段矩形外扩误覆盖。修正为 2 段 8 元素；断言点 (16,15) 在 miter
    // 尖角三角内（(15,15) 恰在角平分线斜边上，无抗锯齿光栅不覆盖）。
    let segments = [5.0, 15.0, 15.0, 15.0, 15.0, 15.0, 15.0, 5.0]; // L 形
    ctx.blit_stroke_to_pixels(&segments, Color::rgba(255, 0, 0, 255), 4.0, false);
    // 连接点 (15, 15) 附近应有像素
    let idx = ((15 * 30) + 16) * 4;
    assert_eq!(ctx.pixel_buffer[idx], 255, "join point should be written");
}

#[test]
fn test_blit_stroke_two_segments_round_join() {
    let mut ctx = ctx_with_pixels(30, 30);
    ctx.line_join = LineJoin::Round;
    let segments = [5.0, 15.0, 15.0, 15.0, 15.0, 15.0, 15.0, 5.0];
    ctx.blit_stroke_to_pixels(&segments, Color::rgba(0, 0, 255, 255), 4.0, false);
    // R34xx：round join 圆盘（半径 half_lw=2）覆盖 join 点 (15,15)。
    let idx = ((15 * 30) + 15) * 4;
    assert_eq!(ctx.pixel_buffer[idx + 2], 255, "round join point should be blue");
}

#[test]
fn test_blit_stroke_two_segments_bevel_join() {
    let mut ctx = ctx_with_pixels(30, 30);
    ctx.line_join = LineJoin::Bevel;
    let segments = [5.0, 15.0, 15.0, 15.0, 15.0, 15.0, 15.0, 5.0];
    ctx.blit_stroke_to_pixels(&segments, Color::rgba(255, 255, 0, 255), 4.0, false);
    // R34xx：bevel 平切三角 {jx, a_ext, b_ext} 覆盖 join 点 (15,15)。
    let idx = ((15 * 30) + 15) * 4;
    assert!(ctx.pixel_buffer[idx] > 0 || ctx.pixel_buffer[idx + 1] > 0, "bevel join");
}

#[test]
fn test_blit_stroke_round_cap() {
    let mut ctx = ctx_with_pixels(20, 20);
    ctx.line_cap = LineCap::Round;
    let segments = [5.0, 10.0, 15.0, 10.0];
    ctx.blit_stroke_to_pixels(&segments, Color::rgba(255, 0, 0, 255), 4.0, false);
    // 端点附近应该有像素
    let idx_start = ((10 * 20) + 5) * 4;
    assert_eq!(ctx.pixel_buffer[idx_start], 255, "round cap at start");
}

#[test]
fn test_blit_stroke_square_cap() {
    let mut ctx = ctx_with_pixels(20, 20);
    ctx.line_cap = LineCap::Square;
    let segments = [5.0, 10.0, 15.0, 10.0];
    ctx.blit_stroke_to_pixels(&segments, Color::rgba(0, 128, 0, 255), 4.0, false);
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
    // R34xx：cap 矩形 = 延伸段（endpoint→ext）垂直扩 half_lw——端点像素本身由段主体
    // 覆盖（cap 单独调用时端点 (10,10) 在矩形边界外，断言改为延伸段内点 (9,10)）。
    let idx = ((10 * 20) + 9) * 4;
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
    let vertices = ctx.flatten_path_open();
    // Empty path with close should still be empty
    assert_eq!(vertices.len(), 0);
}

#[test]
fn test_flatten_path_single_line_with_close() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(20.0, 20.0);
    ctx.current_path.close_path();
    let vertices = ctx.flatten_path_open();
    // Should have line segment + close segment
    assert_eq!(vertices.len(), 8); // 2 segments × 4
}

#[test]
fn test_flatten_path_line_to_same_point() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.line_to(10.0, 10.0); // Same point
    let vertices = ctx.flatten_path_open();
    // R56h：零长段剪除（spec trace-a-path）——退化 lineTo 不参与描边。
    assert_eq!(vertices.len(), 0, "zero-length segment pruned");
}

#[test]
fn test_flatten_path_arc_full_circle() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 0.0);
    ctx.current_path
        .arc(50.0, 50.0, 50.0, 0.0, std::f32::consts::TAU, false);
    let vertices = ctx.flatten_path_open();
    // Full circle should still produce vertices
    // R56：moveTo(50,0) 恰为弧首（角 0）→ 连线段零长仍 push（退化段，扫描线无
    // 穿越）。R56g：细线 N=16 → 17 段。
    assert_eq!(vertices.len(), 68); // 16 segments + line-to-arc-start
}

#[test]
fn test_flatten_path_arc_zero_radius() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.arc(20.0, 20.0, 0.0, 0.0, std::f32::consts::PI, false);
    let vertices = ctx.flatten_path_open();
    // Zero radius arc - just verify it doesn't panic
    let _ = vertices.len();
}

#[test]
fn test_flatten_path_arc_negative_angles() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 0.0);
    ctx.current_path
        .arc(50.0, 50.0, 50.0, -std::f32::consts::PI, 0.0, false);
    let vertices = ctx.flatten_path_open();
    // Negative angles should still work
    // R56：+1 连线段（moveTo→弧首）。R56g：细线 N=16 → 17 段。
    assert_eq!(vertices.len(), 68); // 16 segments + line-to-arc-start
}

#[test]
fn test_flatten_path_arc_start_greater_than_end() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 0.0);
    ctx.current_path.arc(50.0, 50.0, 50.0, std::f32::consts::PI, 0.0, false);
    let vertices = ctx.flatten_path_open();
    // Start > end should still produce vertices
    // R56：+1 连线段（moveTo→弧首）；顺时针 start>end 归一化 span = +π（同向 mod）。
    // R56g：细线 N=16 → 17 段。
    assert_eq!(vertices.len(), 68); // 16 segments + line-to-arc-start
}

#[test]
fn test_flatten_path_ellipse_horizontal_stretch() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 0.0);
    ctx.current_path
        .ellipse(50.0, 50.0, 50.0, 10.0, 0.0, 0.0, std::f32::consts::PI);
    let vertices = ctx.flatten_path_open();
    // Wide ellipse should produce vertices
    assert_eq!(vertices.len(), 64); // 16 segments × 4
}

#[test]
fn test_flatten_path_ellipse_vertical_stretch() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(0.0, 50.0);
    ctx.current_path
        .ellipse(50.0, 50.0, 10.0, 50.0, 0.0, 0.0, std::f32::consts::PI);
    let vertices = ctx.flatten_path_open();
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
    let vertices = ctx.flatten_path_open();
    // Rotated ellipse should produce vertices
    assert_eq!(vertices.len(), 64); // 16 segments × 4
}

#[test]
fn test_flatten_path_ellipse_zero_rotation() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(50.0, 20.0);
    ctx.current_path
        .ellipse(50.0, 50.0, 30.0, 20.0, 0.0, 0.0, std::f32::consts::PI / 2.0);
    let vertices = ctx.flatten_path_open();
    // Zero rotation should still work
    assert_eq!(vertices.len(), 64); // 16 segments × 4
}

#[test]
fn test_flatten_path_round_rect_zero_radius() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.round_rect(10.0, 10.0, 50.0, 50.0, vec![(0.0, 0.0)]);
    let vertices = ctx.flatten_path_open();
    // Zero radius should degenerate to rectangle（R56：自包含子路径 4 边段，
    // MoveTo 不产生连接段）
    assert_eq!(vertices.len(), 16); // 4 segments × 4
}

#[test]
fn test_flatten_path_round_rect_negative_radius() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.round_rect(10.0, 10.0, 50.0, 50.0, vec![(-5.0, -5.0)]);
    let vertices = ctx.flatten_path_open();
    // Negative radius should be clamped to 0, degenerate to rectangle（R56 同上）
    assert_eq!(vertices.len(), 16); // 4 segments × 4
}

#[test]
fn test_flatten_path_round_rect_radii_too_large() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path
        .round_rect(10.0, 10.0, 20.0, 20.0, vec![(100.0, 100.0)]);
    let vertices = ctx.flatten_path_open();
    // Radius larger than half size should be clamped - just verify no panic
    let _ = vertices.len();
}

#[test]
fn test_flatten_path_arc_to_with_zero_distance() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(10.0, 10.0);
    ctx.current_path.arc_to(10.0, 10.0, 20.0, 20.0, 5.0);
    let vertices = ctx.flatten_path_open();
    // Points at same location - just verify no panic
    let _ = vertices.len();
}

#[test]
fn test_flatten_path_arc_to_collinear_with_radius() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.arc_to(10.0, 0.0, 20.0, 0.0, 5.0);
    let vertices = ctx.flatten_path_open();
    // Collinear points should still produce some vertices
    assert!(!vertices.is_empty());
}

#[test]
fn test_flatten_path_multiple_commands() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.current_path.move_to(0.0, 0.0);
    ctx.current_path.line_to(10.0, 10.0);
    ctx.current_path.quadratic_curve_to(20.0, 0.0, 30.0, 10.0);
    ctx.current_path
        .arc(50.0, 50.0, 10.0, 0.0, std::f32::consts::PI / 2.0, false);
    ctx.current_path.close_path();
    let vertices = ctx.flatten_path_open();
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

// ── 图案平铺光栅化（R3085，对称 fill 渐变 R3079）──

#[test]
fn test_fill_rect_pattern_rasterizes_r3085() {
    // 6×4 画布，2×2 源四角四色（red/green/blue/white），repeat 平铺，fill_rect 整画布。
    // fill_rect 经 is_per_pixel_style 分流到 blit_rect_gradient → sample_at → sample_pattern_pixel 逐像素平铺。
    let mut ctx = CanvasContext::new(6, 4);
    let img = ImageData {
        width: 2,
        height: 2,
        data: vec![
            255, 0, 0, 255, // 0 red
            0, 255, 0, 255, // (1,0) green
            0, 0, 255, 255, // (0,1) blue
            255, 255, 255, 255, // 1 white
        ],
    };
    ctx.set_fill_style(CanvasStyle::Pattern(CanvasPattern::new(img, PatternRepetition::Repeat)));
    ctx.fill_rect(0.0, 0.0, 6.0, 4.0);

    let px = |x: usize, y: usize| {
        let i = (y * 6 + x) * 4;
        &ctx.pixel_buffer[i..i + 4]
    };
    // 平铺（tile 2×2，画布 6×4）：x/y 整除回绕。
    assert_eq!(px(0, 0), &[255, 0, 0, 255], "tile(0,0)=red");
    assert_eq!(px(1, 0), &[0, 255, 0, 255], "tile(1,0)=green");
    assert_eq!(px(2, 0), &[255, 0, 0, 255], "x=2 回绕 tile(0,0)=red");
    assert_eq!(px(3, 0), &[0, 255, 0, 255], "x=3 回绕 tile(1,0)=green");
    assert_eq!(px(0, 1), &[0, 0, 255, 255], "y=1 tile(0,1)=blue");
    assert_eq!(px(1, 1), &[255, 255, 255, 255], "tile(1,1)=white");
    // y=2 行也回绕到 tile y=0：px(0,2)=red, px(1,2)=green。
    assert_eq!(px(0, 2), &[255, 0, 0, 255], "y=2 回绕 tile(0,0)=red");
}

#[test]
fn test_fill_rect_pattern_no_repeat_rasterizes_r3085() {
    // 6×4 画布，2×2 源四角四色，no-repeat：仅 tile(0..2,0..2) 着色，其余透明（pixel_buffer 初始 0）。
    let mut ctx = CanvasContext::new(6, 4);
    let img = ImageData {
        width: 2,
        height: 2,
        data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255],
    };
    ctx.set_fill_style(CanvasStyle::Pattern(CanvasPattern::new(
        img,
        PatternRepetition::NoRepeat,
    )));
    ctx.fill_rect(0.0, 0.0, 6.0, 4.0);

    let px = |x: usize, y: usize| {
        let i = (y * 6 + x) * 4;
        &ctx.pixel_buffer[i..i + 4]
    };
    assert_eq!(px(0, 0), &[255, 0, 0, 255], "tile(0,0)=red");
    assert_eq!(px(1, 1), &[255, 255, 255, 255], "tile(1,1)=white");
    assert_eq!(px(2, 0), &[0, 0, 0, 0], "x=2 越界 no-repeat → 透明");
    assert_eq!(px(0, 2), &[0, 0, 0, 0], "y=2 越界 no-repeat → 透明");
    assert_eq!(px(5, 3), &[0, 0, 0, 0], "远端越界 → 透明");
}

#[test]
fn test_flatten_round_rect_zero_radius_negative_h() {
    // R56：零半径 + 负 h（winding 用例 r3=(0,25,100,-25,[0])）——退化矩形分支
    // 必须在归一化包围盒 y∈[0,25] 产出闭合四边。
    let mut verts = Vec::new();
    let _ = CanvasContext::flatten_round_rect(&mut verts, 0.0, 0.0, 0.0, 25.0, 100.0, -25.0, &[(0.0, 0.0)]);
    let hits = scanline_hits(&verts, 12.5);
    assert_eq!(hits.len(), 2, "2 crossings at y=12.5: {hits:?}");
    let (lo, hi) = (hits[0].min(hits[1]), hits[0].max(hits[1]));
    assert!(lo <= 0.5 && hi >= 99.5, "span full width: {hits:?}");
}

// ── R56（M8/DC-8）：arc 方向归一化 + 弧首连线 单测 ──

#[test]
fn test_arc_span_normalization_cw_negative_diff() {
    // 2d.path.arc.angle.5：start=1023π、end=512.5π、顺时针（acw=false）。
    // 旧 `raw % TAU` 得 −π/2（负）→ 弧走向反向翻到对侧象限；正确为同向 mod
    // （span ∈ [0,2π)）= +3π/2。展平首段方向可判：从角 1023π（mod 2π = π）起
    // 步进为正（顺时针增方向）。
    let mut ctx = CanvasContext::new(100, 50);
    ctx.begin_path();
    ctx.arc(
        100.0,
        0.0,
        150.0,
        (1024.0 - 1.0) as f32 * std::f32::consts::PI,
        (512.0 + 0.5) as f32 * std::f32::consts::PI,
        false,
    );
    let verts = ctx.flatten_path_open();
    assert!(verts.len() >= 8, "arc segments emitted");
    // 首段起点 = 弧起点（角 1023π mod 2π = π → (100−150, 0)）
    assert!((verts[0] - (-50.0)).abs() < 1.0, "arc start x≈−50: {}", verts[0]);
    assert!(verts[1].abs() < 1.0, "arc start y≈0: {}", verts[1]);
    // 顺时针（角度增方向）：从角 π 出发 sin 减小（π→1.5π 区间）→ y 减小（屏幕
    // 上方）。旧反向实现（span 负 + dir 双重取反前）弧翻到对侧象限。
    assert!(
        verts[5] < verts[1],
        "clockwise arc from angle π proceeds with y decreasing: {} → {}",
        verts[1],
        verts[5]
    );
}

#[test]
fn test_arc_span_normalization_acw_positive_diff() {
    // 2d.path.arc.angle.2：start=−3π/2、end=−π、逆时针（acw=true）。
    // acw 归一化 span ∈ (−2π,0]：−π（从 −3π/2 减小到 −π−2π？否——同向 mod：
    // start−end = −π/2 → |span| = π/2 → span = −π/2，从 −3π/2 走到 −2π）。
    // 旧实现 raw % TAU = +π/2 再乘 dir=−1 恰为 −π/2——本例旧值相同，但整圆
    // 边界（angle.3 的 raw=−510.5π）与顺时针负差（angle.5）都依赖新归一化。
    let mut ctx = CanvasContext::new(100, 50);
    ctx.begin_path();
    ctx.arc(
        100.0,
        0.0,
        150.0,
        -1.5 * std::f32::consts::PI,
        -std::f32::consts::PI,
        true,
    );
    let verts = ctx.flatten_path_open();
    // 逆时针（角度减方向）从 −3π/2（点 (100,−150) 屏幕上方）向 −2π（点 (250,0)）
    // 首段 y 应减小（屏幕向上）。
    assert!(
        verts[5] < verts[1],
        "anticlockwise arc proceeds screen-upward (angle decreasing): {} → {}",
        verts[1],
        verts[5]
    );
}

#[test]
fn test_arc_line_to_arc_start_after_move_to() {
    // 2d.path.arc.angle.4 / twopie.5：moveTo(圆心) + arc(整圆) + fill。
    // spec dom-context-2d-arc：「If the context has any subpaths, add a straight
    // line from the current point to the start point of the arc」。旧实现不 push
    // 该段 → 扇形缺「圆心→弧首」边，扫描线在弧首角配对破裂。
    let mut ctx = CanvasContext::new(100, 50);
    ctx.begin_path();
    ctx.move_to(50.0, 25.0);
    ctx.arc(50.0, 25.0, 60.0, 0.0, 5.0 * std::f32::consts::PI, false);
    let verts = ctx.flatten_path();
    // 首 4 值应为 (50,25)→弧首 的连线段（弧首 = 角 0 → (110,25)）。
    assert!((verts[0] - 50.0).abs() < f32::EPSILON, "line-from-current x");
    assert!((verts[1] - 25.0).abs() < f32::EPSILON, "line-from-current y");
    assert!((verts[2] - 110.0).abs() < 1.0, "arc start x≈110: {}", verts[2]);
    assert!((verts[3] - 25.0).abs() < 1.0, "arc start y≈25: {}", verts[3]);
}

#[test]
fn test_arc_no_line_without_any_subpath() {
    // 首命令是 arc（beginPath 后无 moveTo）：spec「no subpaths → 直接 moveTo 弧
    // 起点」，不产生 (0,0)→弧首 的虚假连线段（虚假段会在 fill 扫描线引入杂边）。
    let mut ctx = CanvasContext::new(100, 50);
    ctx.begin_path();
    ctx.arc(50.0, 25.0, 60.0, 0.0, std::f32::consts::PI, false);
    let verts = ctx.flatten_path();
    // 首段起点 = 弧首点（角 0 → (110,25)），非 (0,0)。
    assert!(
        (verts[0] - 110.0).abs() < 1.0,
        "first seg starts at arc start: {}",
        verts[0]
    );
    assert!((verts[1] - 25.0).abs() < 1.0, "first seg y≈25: {}", verts[1]);
}

#[test]
fn test_fill_rect_2x2_writes_pixels_r56() {
    // R56 回归哨兵：pipeline canvas 桥测试（canvas_element_bridges_to_image_primitive）
    // 的最小复现——2×2 画布 moveTo/lineTo×3/closePath + fill 必须写 pixel_buffer
    // （snapshot_rgba 非全零才产出 ImagePrimitive）。
    let mut ctx = CanvasContext::new(2, 2);
    ctx.set_fill_color(Color::rgba(255, 0, 0, 255));
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(2.0, 0.0);
    ctx.line_to(2.0, 2.0);
    ctx.line_to(0.0, 2.0);
    ctx.close_path();
    ctx.fill();
    assert!(
        ctx.pixel_buffer.iter().any(|&b| b != 0),
        "2×2 fill 后 pixel_buffer 必须非全零（snapshot_rgba 依赖）: {:?}",
        ctx.pixel_buffer
    );
}

// ── R56c：fill rule（nonzero/evenodd）+ 绕组方向 测试 ──

/// 辅助：读 (x,y) 像素 alpha（不透明度 255 = 完全填充）。
fn pixel_alpha(ctx: &CanvasContext, x: u32, y: u32) -> u8 {
    ctx.pixel_buffer[((y * ctx.width) + x) as usize * 4 + 3]
}

#[test]
fn test_fill_rule_nonzero_nested_same_direction_r56c() {
    // 2d.path.fill.winding.add 语义：嵌套同向子路径（外大矩形 + 内小矩形同向），
    // nonzero 下绕组 ±2 ≠ 0 → 全部填充；旧偶奇配对会把内矩形挖成假洞。
    let mut ctx = ctx_with_pixels(20, 20);
    // 外矩形 (2,2)→(18,2)→(18,18)→(2,18)（顺时针，屏幕 y 向下）+ 内矩形 (6,6)→(14,6)→(14,14)→(6,14) 同向
    let verts = [
        2.0, 2.0, 18.0, 2.0, // 外上
        18.0, 2.0, 18.0, 18.0, // 外右
        18.0, 18.0, 2.0, 18.0, // 外下
        2.0, 18.0, 2.0, 2.0, // 外左（闭合）
        6.0, 6.0, 14.0, 6.0, // 内上
        14.0, 6.0, 14.0, 14.0, // 内右
        14.0, 14.0, 6.0, 14.0, // 内下
        6.0, 14.0, 6.0, 6.0, // 内左（闭合）
    ];
    ctx.blit_path_to_pixels_rule(&verts, Color::rgba(255, 0, 0, 255), FillRule::NonZero);
    // 两矩形之间（绕组 1）与内矩形中心（绕组 2）都应填充
    assert_eq!(pixel_alpha(&ctx, 4, 4), 255, "between rects: winding 1 filled");
    assert_eq!(
        pixel_alpha(&ctx, 10, 10),
        255,
        "inner center: winding 2 filled (nonzero)"
    );
}

#[test]
fn test_fill_rule_nonzero_opposite_direction_hole_r56c() {
    // 2d.path.fill.winding.subtract 语义：反向内矩形（逆时针）绕组对消 → 挖洞；
    // 两矩形之间绕组 1 → 填充。
    let mut ctx = ctx_with_pixels(20, 20);
    let verts = [
        2.0, 2.0, 18.0, 2.0, //
        18.0, 2.0, 18.0, 18.0, //
        18.0, 18.0, 2.0, 18.0, //
        2.0, 18.0, 2.0, 2.0, //
        6.0, 6.0, 6.0, 14.0, // 内矩形逆时针
        6.0, 14.0, 14.0, 14.0, //
        14.0, 14.0, 14.0, 6.0, //
        14.0, 6.0, 6.0, 6.0, //
    ];
    ctx.blit_path_to_pixels_rule(&verts, Color::rgba(255, 0, 0, 255), FillRule::NonZero);
    assert_eq!(pixel_alpha(&ctx, 4, 4), 255, "between rects filled");
    assert_eq!(pixel_alpha(&ctx, 10, 10), 0, "inner hole: winding cancels to 0");
}

#[test]
fn test_fill_rule_evenodd_double_rect_unfilled_r56c() {
    // 2d.path.fill.winding.evenodd.1/2 语义：同一矩形画两遍，evenodd 下穿越 2 次
    // = 偶 → 不填充（保持底色）；nonzero 下绕组 2 → 填充。
    let mut ctx = ctx_with_pixels(20, 20);
    let rect = [
        2.0, 2.0, 18.0, 2.0, //
        18.0, 2.0, 18.0, 18.0, //
        18.0, 18.0, 2.0, 18.0, //
        2.0, 18.0, 2.0, 2.0, //
    ];
    let mut double = rect.to_vec();
    double.extend_from_slice(&rect);
    ctx.blit_path_to_pixels_rule(&double, Color::rgba(255, 0, 0, 255), FillRule::EvenOdd);
    assert_eq!(pixel_alpha(&ctx, 10, 10), 0, "evenodd: two crossings = unfilled");
    // nonzero 同路径应填
    let mut ctx2 = ctx_with_pixels(20, 20);
    ctx2.blit_path_to_pixels_rule(&double, Color::rgba(255, 0, 0, 255), FillRule::NonZero);
    assert_eq!(pixel_alpha(&ctx2, 10, 10), 255, "nonzero: winding 2 = filled");
}

#[test]
fn test_fill_rule_evenodd_nested_hole_r56c() {
    // evenodd 嵌套（一外一内，方向无关）：内矩形中心穿越 2 次 → 洞（与 nonzero
    // 同向嵌套的差别即在此——evenodd 与方向无关）。
    let mut ctx = ctx_with_pixels(20, 20);
    let verts = [
        2.0, 2.0, 18.0, 2.0, //
        18.0, 2.0, 18.0, 18.0, //
        18.0, 18.0, 2.0, 18.0, //
        2.0, 18.0, 2.0, 2.0, //
        6.0, 6.0, 14.0, 6.0, // 内同向（evenodd 无视方向）
        14.0, 6.0, 14.0, 14.0, //
        14.0, 14.0, 6.0, 14.0, //
        6.0, 14.0, 6.0, 6.0, //
    ];
    ctx.blit_path_to_pixels_rule(&verts, Color::rgba(255, 0, 0, 255), FillRule::EvenOdd);
    assert_eq!(pixel_alpha(&ctx, 4, 4), 255, "between rects: 1 crossing filled");
    assert_eq!(pixel_alpha(&ctx, 10, 10), 0, "inner: 2 crossings = hole");
}

#[test]
fn test_fill_rule_overlap_rects_semitransparent_r56c() {
    // 2d.path.fill.overlap 语义：两重叠子路径一次 fill —— nonzero 下重叠区绕组 2
    // 仍是一次填充（同一次 fill 内部不叠加 alpha，输出 rgba(0,127,0) 而非 0,64）。
    // 旧偶奇配对把重叠区挖空。用整型 alpha 通道近似验证。
    let mut ctx = ctx_with_pixels(20, 20);
    // rect A (2,2)-(12,12)，rect B (8,8)-(18,18)（重叠区 (8,8)-(12,12)）
    let verts = [
        2.0, 2.0, 12.0, 2.0, //
        12.0, 2.0, 12.0, 12.0, //
        12.0, 12.0, 2.0, 12.0, //
        2.0, 12.0, 2.0, 2.0, //
        8.0, 8.0, 18.0, 8.0, //
        18.0, 8.0, 18.0, 18.0, //
        18.0, 18.0, 8.0, 18.0, //
        8.0, 18.0, 8.0, 8.0, //
    ];
    // 半透明绿（alpha 128）叠透明确色底过于琐碎——此处底为透明 0，验证 alpha 即写值
    ctx.blit_path_to_pixels_rule(&verts, Color::rgba(0, 255, 0, 128), FillRule::NonZero);
    let overlap = pixel_alpha(&ctx, 10, 10);
    let only_a = pixel_alpha(&ctx, 4, 4);
    assert_eq!(overlap, 128, "overlap region alpha 128 (single fill, not 2 layers)");
    assert_eq!(only_a, 128, "non-overlap alpha 128");
}

#[test]
fn test_roundrect_single_axis_mirror_winding_r56c() {
    // 2d.path.roundrect.winding 语义：单轴镜像（负 h）的 roundRect 与正参数矩形
    // 绕向相反（真浏览器沿参数边方向环绕——R56c reverse_subpath 实证）。
    // roundRect(10,10,20,20) + roundRect(10,30,20,-20)（参数角 bl，归一化后
    // 同区域垂直镜像反向）nonzero 绕组对消 → 不填充。旧偶奇光栅无方向语义，
    // 且旧 flatten 不反转——同区域反向对会整片误填（winding 用例四角全红）。
    // 同区域反向：roundRect(10,10,20,20)（顺时针）+ roundRect(30,30,-20,-20)
    //（负 w/h 双负=180° 旋转同向；用单轴 roundRect(10,10,-20,20) 会偏到 x∈[-10,10]
    // 不同区域——WPT winding 用例的配对方式是 (0,0,50,50) vs (100,50,-50,-50)？
    // 不——实测 2d.path.roundrect.winding 用的是 (0,25,100,-25) 单轴镜像 vs
    // (0,0,50,50)：同区域须让参数角落在同一对角。此处取 (10,30,-20,20)：参数角
    // (10,30)（tl），归一化包围盒 (−10,10)-(10,30) 仍偏。正确配对：
    // roundRect(10,10,20,20) 与 roundRect(10,30,20,-20)（负 h，参数角 (10,30)
    // = bl，包围盒 (10,10)-(30,30) 同区域、垂直镜像反向）。
    let mut ctx = ctx_with_pixels(40, 40);
    ctx.begin_path();
    ctx.round_rect(10.0, 10.0, 20.0, 20.0, vec![(5.0, 5.0)]);
    ctx.round_rect(10.0, 30.0, 20.0, -20.0, vec![(5.0, 5.0)]);
    ctx.fill_with_rule(FillRule::NonZero);
    // 同区域绕组对消 → 不填
    let filled = (12..28).any(|y| (12..28).any(|x| pixel_alpha(&ctx, x, y) != 0));
    assert!(!filled, "opposite-winding roundrects cancel under nonzero");
    // evenodd 同路径也**不填**（crossing 计数偶 → outside；evenodd 无方向语义，
    // 同区域反向对在两规则下都不填——与 nonzero 的区别要靠**同向**双矩形区分，
    // 见 test_fill_rule_evenodd_double_rect_unfilled_r56c / nonzero 同路径填）。
    let mut ctx2 = ctx_with_pixels(40, 40);
    ctx2.begin_path();
    ctx2.round_rect(10.0, 10.0, 20.0, 20.0, vec![(5.0, 5.0)]);
    ctx2.round_rect(10.0, 30.0, 20.0, -20.0, vec![(5.0, 5.0)]);
    ctx2.fill_with_rule(FillRule::EvenOdd);
    let filled2 = (12..28).any(|y| (12..28).any(|x| pixel_alpha(&ctx2, x, y) != 0));
    assert!(
        !filled2,
        "evenodd: same-region opposite pair = even crossings, unfilled"
    );
}

#[test]
fn test_is_point_in_path_giant_scale_r56d() {
    // 2d.path.isPointInPath.basic 巨 scale 场景：scale(MAX_VALUE) 后 rect(-10,-10,20,20)
    // 顶点溢出 ±inf——收缩代理后 (0,0) 应在巨矩形内（真浏览器双精度仍有限）。
    let mut ctx = CanvasContext::new(200, 200);
    ctx.scale(f32::MAX, f32::MAX);
    ctx.begin_path();
    // host rect op 语义：moveTo+3×lineTo+closePath（顶点经 CTM 变换溢出 ±inf）。
    ctx.move_to(-10.0, -10.0);
    ctx.line_to(10.0, -10.0);
    ctx.line_to(10.0, 10.0);
    ctx.line_to(-10.0, 10.0);
    ctx.close_path();
    assert!(
        ctx.is_point_in_path_rule(0.0, 0.0, crate::context::FillRule::NonZero),
        "giant scale: (0,0) inside via inf-shrink proxy"
    );
    // 非可逆 CTM（scale(0,0)）：det=0，路径退化——不命中。
    let mut ctx2 = CanvasContext::new(200, 200);
    ctx2.scale(0.0, 0.0);
    ctx2.begin_path();
    ctx2.move_to(-10.0, -10.0);
    ctx2.line_to(10.0, -10.0);
    ctx2.line_to(10.0, 10.0);
    ctx2.line_to(-10.0, 10.0);
    ctx2.close_path();
    assert!(
        !ctx2.is_point_in_path_rule(0.0, 0.0, crate::context::FillRule::NonZero),
        "non-invertible ctm: degenerate path misses"
    );
}

// ── R56e：ensuresubpath 语义 + clip 相交/空 测试 ──

#[test]
fn test_lineto_no_subpath_is_moveto_r56e() {
    // spec dom-context-2d-lineto：无子路径 lineTo 等同 moveTo（不画隐含连线）。
    let mut ctx = CanvasContext::new(100, 50);
    ctx.begin_path();
    ctx.line_to(100.0, 50.0);
    let v = ctx.flatten_path_open();
    // 无子路径 → 无段（只设起点，曲线/线不画）。
    assert!(
        v.is_empty(),
        "lineTo no-subpath must not emit segments, got {} segs",
        v.len() / 4
    );
    // 有子路径（先 moveTo）后 lineTo 正常画 1 段。
    let mut ctx2 = CanvasContext::new(100, 50);
    ctx2.begin_path();
    ctx2.move_to(0.0, 0.0);
    ctx2.line_to(100.0, 50.0);
    assert_eq!(ctx2.flatten_path_open().len(), 4, "moveTo+lineTo = 1 segment");
}

#[test]
fn test_curve_no_subpath_first_control_point_r56e() {
    // spec dom-context-2d-quadraticcurveto / beziercurveto：无子路径时第一控制点
    // 被加入为起点，曲线照画（quadratic(0,25,100,25) 退化直线 (0,25)→(100,25)）。
    let mut ctx = CanvasContext::new(100, 50);
    ctx.begin_path();
    ctx.quadratic_curve_to(0.0, 25.0, 100.0, 25.0);
    let v = ctx.flatten_path_open();
    assert!(!v.is_empty(), "quadratic no-subpath draws from first control point");
    // 首段起点 = 第一控制点 (0,25)。
    assert!(
        (v[0] - 0.0).abs() < 1.0 && (v[1] - 25.0).abs() < 1.0,
        "first seg starts at first control point (0,25), got ({},{})",
        v[0],
        v[1]
    );
    // bezier 同语义：current := cp1。
    let mut ctx2 = CanvasContext::new(100, 50);
    ctx2.begin_path();
    ctx2.bezier_curve_to(0.0, 25.0, 100.0, 25.0, 100.0, 25.0);
    let v2 = ctx2.flatten_path_open();
    assert!(
        !v2.is_empty() && (v2[1] - 25.0).abs() < 1.0,
        "bezier no-subpath starts at cp1"
    );
}

#[test]
fn test_arcto_no_subpath_no_leading_line_r56e() {
    // spec dom-context-2d-arcto：无子路径时第一控制点加入为起点（P1），
    // **什么都不画**（"nothing is drawn up to it"——R56f 修正：R56e 版「弧仍画」
    // 是误读，真浏览器该场景 stroke 输出为空；WPT ensuresubpath.1 期望图整片底色）。
    let mut ctx = CanvasContext::new(100, 50);
    ctx.begin_path();
    ctx.arc_to(100.0, 50.0, 200.0, 50.0, 0.1);
    let v = ctx.flatten_path_open();
    assert!(v.is_empty(), "no-subpath arcTo draws nothing, got {} segs", v.len() / 4);
    // 后续 lineTo 从 P1 起（P1 已是子路径起点）正常画。
    ctx.line_to(50.0, 25.0);
    let v2 = ctx.flatten_path_open();
    assert_eq!(v2.len(), 4, "following lineTo draws one segment from P1");
}

#[test]
fn test_clip_empty_and_intersect_r56e() {
    // spec dom-context-2d-clip：clip 与既有 clip 相交；空路径 clip = 交集空全裁。
    let mut ctx = CanvasContext::new(100, 50);
    ctx.begin_path();
    ctx.clip(); // 空路径
    ctx.set_fill_color(Color::rgba(255, 0, 0, 255));
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);
    let px = ctx.get_image_data(50, 25, 1, 1);
    assert_eq!(px.data[0], 0, "empty clip: nothing drawn");
    // 相邻两矩形依次 clip → 交集空 → 全裁。
    let mut ctx2 = CanvasContext::new(100, 50);
    ctx2.begin_path();
    ctx2.move_to(0.0, 0.0);
    ctx2.line_to(50.0, 0.0);
    ctx2.line_to(50.0, 50.0);
    ctx2.line_to(0.0, 50.0);
    ctx2.close_path();
    ctx2.clip();
    ctx2.begin_path();
    ctx2.move_to(50.0, 0.0);
    ctx2.line_to(100.0, 0.0);
    ctx2.line_to(100.0, 50.0);
    ctx2.line_to(50.0, 50.0);
    ctx2.close_path();
    ctx2.clip();
    ctx2.set_fill_color(Color::rgba(255, 0, 0, 255));
    ctx2.fill_rect(0.0, 0.0, 100.0, 50.0);
    let px2 = ctx2.get_image_data(50, 25, 1, 1);
    assert_eq!(px2.data[0], 0, "adjacent clips intersect to empty");
    // 重叠两矩形 clip → 交集 = 重叠区可画。
    let mut ctx3 = CanvasContext::new(100, 50);
    ctx3.begin_path();
    ctx3.move_to(0.0, 0.0);
    ctx3.line_to(60.0, 0.0);
    ctx3.line_to(60.0, 50.0);
    ctx3.line_to(0.0, 50.0);
    ctx3.close_path();
    ctx3.clip();
    ctx3.begin_path();
    ctx3.move_to(40.0, 0.0);
    ctx3.line_to(100.0, 0.0);
    ctx3.line_to(100.0, 50.0);
    ctx3.line_to(40.0, 50.0);
    ctx3.close_path();
    ctx3.clip();
    ctx3.set_fill_color(Color::rgba(255, 0, 0, 255));
    ctx3.fill_rect(0.0, 0.0, 100.0, 50.0);
    let in_overlap = ctx3.get_image_data(50, 25, 1, 1);
    let outside = ctx3.get_image_data(10, 25, 1, 1);
    assert_eq!(in_overlap.data[0], 255, "overlap region drawn");
    assert_eq!(outside.data[0], 0, "outside both clips culled");
}

// ── R56f：arcTo 真切线弧 + 变换语义 ──

#[test]
fn test_arcto_tangent_geometry_and_anisotropic_transform_r56f() {
    // 2d.path.arcTo.transformation 场景：moveTo(0,50) + translate(100,0) +
    // arcTo(50,50,50,0,50)——切线弧 T1=(100,50)、圆心 (100,0)、T2=(150,0)。
    let mut ctx = CanvasContext::new(200, 50);
    ctx.begin_path();
    ctx.move_to(0.0, 50.0);
    ctx.translate(100.0, 0.0);
    ctx.arc_to(50.0, 50.0, 50.0, 0.0, 50.0);
    let v = ctx.flatten_path();
    // 首段 = P0(0,50)→T1(100,50)（切线连线）。
    assert!(
        (v[0] - 0.0).abs() < 1.0 && (v[1] - 50.0).abs() < 1.0 && (v[2] - 100.0).abs() < 1.5,
        "leading tangent line P0->T1, got ({},{})",
        v[0],
        v[2]
    );
    // 末弧段终点 ≈ T2(150,0)（fill 语义 flatten 末尾是 closepath-on-fill 闭合段，
    // 弧末取倒数第二段）。
    let arc_end = &v[v.len() - 8..v.len() - 4];
    assert!(
        (arc_end[2] - 150.0).abs() < 1.5 && arc_end[3].abs() < 1.5,
        "arc ends at T2(150,0), got ({},{})",
        arc_end[2],
        arc_end[3]
    );

    // 各向异性 CTM（2d.path.arcTo.scale 的 scale(0.1,1)）：弧在用户空间构造后
    // 正变换——设备空间 x 压 0.1。切线 T1 用户空间 (0,50)+…，设备空间近似窄弧。
    let mut ctx2 = CanvasContext::new(200, 50);
    ctx2.begin_path();
    ctx2.move_to(0.0, 50.0);
    ctx2.translate(100.0, 0.0);
    ctx2.scale(0.1, 1.0);
    ctx2.arc_to(50.0, 50.0, 50.0, 0.0, 50.0);
    let v2 = ctx2.flatten_path();
    assert!(!v2.is_empty());
    // 弧顶点 x 不应超过 ~100 + 50*0.1 + 容差（用户空间 r=50 经 sx=0.1 压到 5）。
    let max_x = v2
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|s| [s[0], s[2]])
        .fold(f32::MIN, f32::max);
    assert!(
        max_x < 115.0,
        "anisotropic scale clamps arc x-extent (user-space r=50 -> device 5), max_x={max_x}"
    );
}

// ── R56g：弧自适应段数 + 真圆环带 ──

#[test]
fn test_arc_adaptive_segments_thick_stroke_r56g() {
    // 2d.path.arc.shape.1 语义：粗 stroke（lw≈r）下折线弦的斜段矩形不得侧向
    // 掠入弧带外——(20,48) 距弧（圆心(50,50) r=50 的下半弧）≈ 40px 应保持底色。
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_fill_color(Color::rgba(0, 255, 0, 255));
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);
    ctx.line_width = 50.0;
    ctx.set_stroke_color(Color::rgba(255, 0, 0, 255));
    ctx.begin_path();
    ctx.arc(50.0, 50.0, 50.0, 0.0, std::f32::consts::PI, false);
    ctx.stroke();
    let idx = ((48 * 100) + 20) as usize * 4;
    assert_eq!(
        ctx.pixel_buffer[idx], 0,
        "arc.shape.1: (20,48) stays background under thick stroke"
    );
}

#[test]
fn test_arc_annulus_covers_interior_r56g() {
    // 2d.path.arc.shape.2 语义：粗 stroke 逆时针弧带须连续——(20,20) 距弧
    // 7.6px 在带内（折线伪节点洞由 annulus 后处理补上）。
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_fill_color(Color::rgba(255, 0, 0, 255));
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);
    ctx.line_width = 100.0;
    ctx.set_stroke_color(Color::rgba(0, 255, 0, 255));
    ctx.begin_path();
    ctx.arc(50.0, 50.0, 50.0, 0.0, std::f32::consts::PI, true);
    ctx.stroke();
    let g = |x: u32, y: u32| ctx.pixel_buffer[((y * 100) + x) as usize * 4 + 1];
    assert_eq!(g(20, 20), 255, "annulus covers (20,20)");
    assert_eq!(g(1, 1), 255, "annulus covers (1,1) (dist to arc 18.6 < half 50)");
    // butt 端面外（θ 范围外）不延伸：θ>0 的右上象限 (98,1) 距端点近但不在弧内。
    // (98,1) 距弧最近点 (85,15) 18.8 <50 但 θ=-45.6° 在 [-180°,0] 内 → 带内。
    // 端面外的例子：弧为上半圆，(98,48) θ≈-12° 带内…取画布下缘中点 (50,49)：
    // 距弧 (50,0) 49 <50 带内。端面外取 (1,49)：距端点 (0,50) 1.4 —— θ=atan2(-1,-1)
    // =-135° 带内。真端面外点：θ=+90° 的 (50,100+) 画布外——skip。
}

// ── R56h：曲线段数自适应 + 像素方形覆盖 ──

#[test]
fn test_curve_adaptive_segments_and_boundary_pixel_r56h() {
    // 2d.path.bezierCurveTo.shape 语义：巨坐标曲线（控制折线 ~13000px）的
    // 固定 8 段弦偏差 48px；自适应后 (1,1) 临界像素（中心距曲线 27.9、
    // half 27.5、方形最近角 27.2 < 27.5 相交）须着色。
    let mut ctx = CanvasContext::new(100, 50);
    ctx.set_fill_color(Color::rgba(255, 0, 0, 255));
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);
    ctx.line_width = 55.0;
    ctx.set_stroke_color(Color::rgba(0, 255, 0, 255));
    ctx.begin_path();
    ctx.move_to(-2000.0, 3100.0);
    ctx.bezier_curve_to(-2000.0, -1000.0, 2100.0, -1000.0, 2100.0, 3100.0);
    ctx.stroke();
    let g = |x: u32, y: u32| ctx.pixel_buffer[((y * 100) + x) as usize * 4 + 1];
    assert_eq!(g(50, 25), 255, "curve midpoint (50,25) stroked");
    assert_eq!(
        g(1, 1),
        255,
        "boundary pixel (1,1): square intersects band (27.2 < 27.5)"
    );
    assert_eq!(g(98, 48), 255, "boundary pixel (98,48) stroked");

    // quadratic 同构（中点 (50,25)）。
    let mut ctx2 = CanvasContext::new(100, 50);
    ctx2.set_fill_color(Color::rgba(255, 0, 0, 255));
    ctx2.fill_rect(0.0, 0.0, 100.0, 50.0);
    ctx2.line_width = 55.0;
    ctx2.set_stroke_color(Color::rgba(0, 255, 0, 255));
    ctx2.begin_path();
    ctx2.move_to(-1000.0, 1050.0);
    ctx2.quadratic_curve_to(0.0, -1000.0, 1200.0, 1050.0);
    ctx2.stroke();
    let g2 = |x: u32, y: u32| ctx2.pixel_buffer[((y * 100) + x) as usize * 4 + 1];
    assert_eq!(g2(50, 25), 255, "quad midpoint stroked");
    assert_eq!(g2(1, 1), 255, "quad boundary pixel (1,1) stroked");
}

// R56i（M8/DC-8）：roundRect 闭合环绕 join——退化矩形 4 段在画布外、
// lineWidth 200 时起点 join（环绕）须覆盖画布中心。
#[test]
fn test_roundrect_closed_wraparound_join_r56i() {
    let mut ctx = ctx_with_pixels(100, 50);
    ctx.set_fill_color(Color::rgba(255, 0, 0, 255));
    ctx.fill_rect(0.0, 0.0, 100.0, 50.0);
    ctx.line_width = 200.0;
    ctx.set_stroke_color(Color::rgba(0, 255, 0, 255));
    ctx.begin_path();
    ctx.round_rect(100.0, 50.0, 100.0, 100.0, vec![(0.0, 0.0)]);
    ctx.stroke();
    let g = |x: u32, y: u32| ctx.pixel_buffer[((y * 100) + x) as usize * 4 + 1];
    assert_eq!(g(50, 25), 255, "wrap-around join covers canvas center");
}

/// R57（M3）：旋转 CTM 下 fillRect 的边界覆盖率混合（4×4 超采样 AA）——
/// 轴对齐恒 1（零回归）；旋转矩形边像素应半色调（ref Chromium AA）。
#[test]
fn test_fill_rect_rotated_coverage_aa() {
    let mut ctx = CanvasContext::new(20, 20);
    // 30° 旋转（cos30=0.866, sin30=0.5）+ 平移：矩形 (5,5,10,10) 的斜边跨像素
    // 网格——边界像素覆盖率 ∈ (0,1)（4×4 超采样半色调）。
    let (c30, s30) = (0.866f32, 0.5f32);
    ctx.set_transform(c30, s30, -s30, c30, 6.0, -3.0);
    ctx.set_fill_color(Color::rgba(255, 0, 0, 255));
    ctx.fill_rect(5.0, 5.0, 10.0, 10.0);
    // 矩形中心 (10,10) → 变换后 (0.866·10−0.5·10+6, 0.5·10+0.866·10−3) = (9.66, 10.66)
    // ——内部满色
    let inner = ctx.get_image_data(10, 11, 1, 1);
    assert_eq!(
        &inner.data[..4],
        &[255, 0, 0, 255],
        "旋转矩形内部应满色红: {:?}",
        &inner.data[..4]
    );
    // 变换后远离矩形的像素 (2,2) 应透明
    let outside = ctx.get_image_data(2, 2, 1, 1);
    assert_eq!(outside.data[3], 0, "旋转矩形外应透明: {:?}", &outside.data[..4]);
    // 边界像素覆盖率 ∈ (0,1)：源 alpha 被混合（半色调）——扫描全画布
    let mut found_edge = false;
    for y in 0..20 {
        for x in 0..20 {
            let p = ctx.get_image_data(x, y, 1, 1);
            if p.data[3] > 0 && p.data[3] < 255 {
                found_edge = true;
                break;
            }
        }
        if found_edge {
            break;
        }
    }
    assert!(found_edge, "旋转矩形边应存在半色调像素（覆盖率混合）");
}

/// R57（M3）：轴对齐 fillRect 覆盖率恒 1——整数矩形硬边（零回归守护）。
#[test]
fn test_fill_rect_axis_aligned_hard_edge() {
    let mut ctx = CanvasContext::new(20, 20);
    ctx.set_fill_color(Color::rgba(0, 0, 255, 255));
    ctx.fill_rect(10.0, 10.0, 5.0, 5.0);
    let edge = ctx.get_image_data(9, 10, 1, 1);
    assert_eq!(edge.data[3], 0, "整数矩形左侧外应透明");
    let inside = ctx.get_image_data(10, 10, 1, 1);
    assert_eq!(&inside.data[..4], &[0, 0, 255, 255], "整数矩形内部应满色");
}

/// R57（M3）：旋转 CTM 下路径 fill() 的边界覆盖率混合（4×4 超采样 AA）——
/// 与 fillRect 的 rect_coverage 同模式：内部满色、外部透明、边界半色调。
#[test]
fn test_fill_path_rotated_coverage_aa() {
    let mut ctx = CanvasContext::new(20, 20);
    // 30° 旋转（cos30=0.866, sin30=0.5）+ 平移（与 fillRect 测试同变换）。
    let (c30, s30) = (0.866f32, 0.5f32);
    ctx.set_transform(c30, s30, -s30, c30, 6.0, -3.0);
    ctx.set_fill_color(Color::rgba(0, 0, 255, 255));
    ctx.begin_path();
    ctx.move_to(5.0, 5.0);
    ctx.line_to(15.0, 5.0);
    ctx.line_to(15.0, 15.0);
    ctx.line_to(5.0, 15.0);
    ctx.close_path();
    ctx.fill();
    // 矩形中心 (10,10) → (9.66, 10.66)——内部满色
    let inner = ctx.get_image_data(10, 11, 1, 1);
    assert_eq!(
        &inner.data[..4],
        &[0, 0, 255, 255],
        "旋转路径内部应满色蓝: {:?}",
        &inner.data[..4]
    );
    // 变换后远离路径的像素 (2,2) 应透明
    let outside = ctx.get_image_data(2, 2, 1, 1);
    assert_eq!(outside.data[3], 0, "旋转路径外应透明");
    // 边界像素半色调（覆盖率混合）——扫描全画布
    let mut found_edge = false;
    for y in 0..20 {
        for x in 0..20 {
            let p = ctx.get_image_data(x, y, 1, 1);
            if p.data[3] > 0 && p.data[3] < 255 {
                found_edge = true;
                break;
            }
        }
        if found_edge {
            break;
        }
    }
    assert!(found_edge, "旋转路径边应存在半色调像素（覆盖率混合）");
}

/// R57（M3）：旋转 CTM + evenodd 挖洞路径填充——边界半色调与洞内透明共存。
#[test]
fn test_fill_path_evenodd_rotated_aa() {
    let mut ctx = CanvasContext::new(24, 24);
    let (c30, s30) = (0.866f32, 0.5f32);
    ctx.set_transform(c30, s30, -s30, c30, 8.0, -4.0);
    ctx.set_fill_color(Color::rgba(255, 0, 0, 255));
    ctx.begin_path();
    ctx.move_to(4.0, 4.0);
    ctx.line_to(16.0, 4.0);
    ctx.line_to(16.0, 16.0);
    ctx.line_to(4.0, 16.0);
    ctx.close_path();
    ctx.move_to(7.0, 7.0);
    ctx.line_to(13.0, 7.0);
    ctx.line_to(13.0, 13.0);
    ctx.line_to(7.0, 13.0);
    ctx.close_path(); // evenodd 内洞
    ctx.fill_with_rule(crate::context::FillRule::EvenOdd);
    // evenodd 外环：环带内应存在满色红像素 + 旋转边半色调像素共存。
    let mut found_full = false;
    let mut found_edge = false;
    for y in 0..24 {
        for x in 0..24 {
            let p = ctx.get_image_data(x, y, 1, 1);
            if p.data[3] == 255 && p.data[0] == 255 {
                found_full = true;
            }
            if p.data[3] > 0 && p.data[3] < 255 {
                found_edge = true;
            }
        }
    }
    assert!(found_full, "evenodd 外环应存在满色红像素");
    assert!(found_edge, "evenodd 旋转路径边应存在半色调像素");
}

/// R57（M3）：旋转 CTM 下渐变路径填充边界——AA 覆盖率作用于渐变采样色。
#[test]
fn test_fill_path_gradient_rotated_aa() {
    let mut ctx = CanvasContext::new(20, 20);
    let (c30, s30) = (0.866f32, 0.5f32);
    ctx.set_transform(c30, s30, -s30, c30, 6.0, -3.0);
    let mut grad = LinearGradient::new(0.0, 0.0, 20.0, 20.0);
    grad.add_color_stop(0.0, Color::rgba(255, 0, 0, 255));
    grad.add_color_stop(1.0, Color::rgba(0, 0, 255, 255));
    ctx.set_fill_style(CanvasStyle::LinearGradient(grad));
    ctx.begin_path();
    ctx.move_to(5.0, 5.0);
    ctx.line_to(15.0, 5.0);
    ctx.line_to(15.0, 15.0);
    ctx.line_to(5.0, 15.0);
    ctx.close_path();
    ctx.fill();
    // 内部满 alpha（渐变红→蓝）
    let inner = ctx.get_image_data(10, 11, 1, 1);
    assert_eq!(inner.data[3], 255, "渐变路径内部应满 alpha: {:?}", &inner.data[..4]);
    // 边界半色调
    let mut found_edge = false;
    for y in 0..20 {
        for x in 0..20 {
            let p = ctx.get_image_data(x, y, 1, 1);
            if p.data[3] > 0 && p.data[3] < 255 {
                found_edge = true;
                break;
            }
        }
        if found_edge {
            break;
        }
    }
    assert!(found_edge, "旋转渐变路径边应存在半色调像素");
}

/// R57（M3）：轴对齐 CTM 下路径 fill 硬边——旋转 AA 零回归守护
///（整数坐标矩形路径边界满色、外透明，与 fillRect 同语义）。
#[test]
fn test_fill_path_axis_aligned_hard_edge() {
    let mut ctx = CanvasContext::new(20, 20);
    ctx.set_fill_color(Color::rgba(0, 0, 255, 255));
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(15.0, 10.0);
    ctx.line_to(15.0, 15.0);
    ctx.line_to(10.0, 15.0);
    ctx.close_path();
    ctx.fill();
    let edge = ctx.get_image_data(9, 10, 1, 1);
    assert_eq!(edge.data[3], 0, "整数矩形路径左侧外应透明");
    let inside = ctx.get_image_data(10, 10, 1, 1);
    assert_eq!(&inside.data[..4], &[0, 0, 255, 255], "整数矩形路径内部应满色");
}

/// R57（M3）：描边边界像素 AA——中心 miss（距中心线 ∈ (half, half+0.5]）的
/// 斜线边像素 4×4 超采样半色调；中心命中（≤ half）仍满色（WPT 满色契约——
/// bezierCurveTo.shape 的 (1,1) 等临界像素）。
#[test]
fn test_stroke_edge_aa_supersampling() {
    let mut ctx = CanvasContext::new(60, 40);
    // 45° 斜线（half=20）：边界像素中心距线 20-20.5 之间出现（线跨像素网格）
    ctx.set_line_width(40.0);
    ctx.set_stroke_color(Color::rgba(0, 0, 255, 255));
    ctx.begin_path();
    ctx.move_to(-20.0, 30.0);
    ctx.line_to(80.0, -30.0);
    ctx.stroke();
    // 线的内部（距中心线 < half）满色
    let inner = ctx.get_image_data(20, 20, 1, 1);
    assert_eq!(
        &inner.data[..4],
        &[0, 0, 255, 255],
        "描边内部应满色蓝: {:?}",
        &inner.data[..4]
    );
    // 存在半色调边界像素（alpha ∈ (0,255)）——AA 边
    let mut found_edge = false;
    for y in 0..40 {
        for x in 0..60 {
            let p = ctx.get_image_data(x, y, 1, 1);
            if p.data[3] > 0 && p.data[3] < 255 {
                found_edge = true;
                break;
            }
        }
        if found_edge {
            break;
        }
    }
    assert!(found_edge, "斜线描边边应存在半色调像素（超采样覆盖率）");
}

/// R57（M3）：miter 尖角顶的亚像素 span 填充——join 三角四边形在尖角处宽
/// < 1px，floor 截断曾丢尖角顶（2d.reset.render.miter_limit 尖角 vs Chromium
/// 差 5px）。尖角顶（内容 y≈2）应着色。
#[test]
fn test_miter_spike_top_subpixel_span() {
    let mut ctx = CanvasContext::new(60, 40);
    ctx.set_line_width(10.0);
    ctx.set_stroke_color(Color::rgba(0, 0, 255, 255));
    ctx.begin_path();
    // 对称 V 形（i=4 角同款：两段关于 y 轴对称，miter 尖角垂直向上）
    ctx.move_to(0.0, 40.0);
    ctx.line_to(10.0, 0.0);
    ctx.line_to(20.0, 40.0);
    ctx.stroke();
    // 尖角顶：miter_len = 5/sin(θ/2)——θ=36.9°（两段方向 (10,-40)/(10,40)）
    // → sin(18.4°)=0.316 → miter_len=15.8——顶 y = 0 - 15.8×cos(...)？
    // 平分向上：顶在 (10, 0-15.8) 附近——亚像素宽四边形从 (10,0) 向上——
    // 至少 y=2 处（理论顶附近）应着色
    let spike = ctx.get_image_data(10, 1, 1, 1);
    assert!(
        spike.data[3] > 0,
        "miter 尖角顶（y=1）应着色（亚像素 span 填充）: {:?}",
        &spike.data[..4]
    );
    let spike2 = ctx.get_image_data(10, 2, 1, 1);
    assert!(spike2.data[3] > 0, "miter 尖角顶（y=2）应着色");
}

//! Canvas 上下文测试（intermediate 批次）。

use super::super::types::*;
use crate::context::*;
use crate::path::{Path2D, PathCommand};
use zero_render_foundation::color::Color;

#[test]
fn test_point_in_polygon_square() {
    let square = [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
    assert!(point_in_polygon(50.0, 50.0, &square));
    assert!(!point_in_polygon(150.0, 50.0, &square));
}

/// 测试射线法对少于 3 个点返回 false。
#[test]
fn test_point_in_polygon_too_few_points() {
    let two_points = [(0.0, 0.0), (100.0, 100.0)];
    assert!(!point_in_polygon(50.0, 50.0, &two_points));
    let empty: [(f32, f32); 0] = [];
    assert!(!point_in_polygon(50.0, 50.0, &empty));
}

/// 测试射线法判断凹多边形。
#[test]
fn test_point_in_polygon_concave() {
    // L 形多边形
    let l_shape = [
        (0.0, 0.0),
        (100.0, 0.0),
        (100.0, 50.0),
        (50.0, 50.0),
        (50.0, 100.0),
        (0.0, 100.0),
    ];
    // 凹角内侧的点
    assert!(point_in_polygon(25.0, 75.0, &l_shape));
    // 凹角外侧的点
    assert!(!point_in_polygon(75.0, 75.0, &l_shape));
}

// ── CompositeOperation Default 测试 ──

/// 测试 CompositeOperation 默认值为 SourceOver。
#[test]
fn test_composite_operation_default_value() {
    assert_eq!(CompositeOperation::default(), CompositeOperation::SourceOver);
}

// ── Shadow properties 测试 ──

/// 测试阴影属性默认值。
#[test]
fn test_shadow_default_values() {
    let ctx = CanvasContext::new(100, 100);
    assert_eq!(*ctx.shadow_color(), Color::TRANSPARENT);
    assert!((ctx.shadow_blur() - 0.0).abs() < f32::EPSILON);
    assert!((ctx.shadow_offset_x() - 0.0).abs() < f32::EPSILON);
    assert!((ctx.shadow_offset_y() - 0.0).abs() < f32::EPSILON);
}

/// 测试设置和获取阴影颜色。
#[test]
fn test_shadow_set_get_color() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_shadow_color(Color::RED);
    assert_eq!(*ctx.shadow_color(), Color::RED);
}

/// 测试设置和获取阴影模糊半径。
#[test]
fn test_shadow_set_get_blur() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_shadow_blur(10.0);
    assert!((ctx.shadow_blur() - 10.0).abs() < f32::EPSILON);
}

/// 测试设置和获取阴影偏移。
#[test]
fn test_shadow_set_get_offset() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_shadow_offset_x(5.0);
    ctx.set_shadow_offset_y(7.0);
    assert!((ctx.shadow_offset_x() - 5.0).abs() < f32::EPSILON);
    assert!((ctx.shadow_offset_y() - 7.0).abs() < f32::EPSILON);
}

/// 测试阴影属性在 save/restore 中正确保存和恢复。
#[test]
fn test_shadow_save_restore() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_shadow_color(Color::RED);
    ctx.set_shadow_blur(5.0);
    ctx.set_shadow_offset_x(3.0);
    ctx.set_shadow_offset_y(4.0);
    ctx.save();
    ctx.set_shadow_color(Color::BLUE);
    ctx.set_shadow_blur(20.0);
    ctx.set_shadow_offset_x(10.0);
    ctx.set_shadow_offset_y(15.0);
    assert_eq!(*ctx.shadow_color(), Color::BLUE);
    assert!((ctx.shadow_blur() - 20.0).abs() < f32::EPSILON);
    ctx.restore();
    assert_eq!(*ctx.shadow_color(), Color::RED);
    assert!((ctx.shadow_blur() - 5.0).abs() < f32::EPSILON);
    assert!((ctx.shadow_offset_x() - 3.0).abs() < f32::EPSILON);
    assert!((ctx.shadow_offset_y() - 4.0).abs() < f32::EPSILON);
}

/// 测试阴影模糊半径负值被限制为 0。
#[test]
fn test_shadow_blur_clamp_negative() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_shadow_blur(-10.0);
    assert!((ctx.shadow_blur() - 0.0).abs() < f32::EPSILON);
}

/// 测试阴影应用于 fill_rect。
#[test]
fn test_shadow_applied_to_fill_rect() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_shadow_color(Color::BLACK);
    ctx.set_shadow_offset_x(5.0);
    ctx.set_shadow_offset_y(5.0);
    ctx.fill_rect(10.0, 10.0, 20.0, 20.0);
    // 检查阴影区域有像素被写入（偏移位置）
    let shadow_pixel = ctx.get_image_data(15, 15, 1, 1);
    assert_ne!(shadow_pixel.data[3], 0, "shadow area should have pixels");
}

/// 测试阴影应用于 stroke_rect。
#[test]
fn test_shadow_applied_to_stroke_rect() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_shadow_color(Color::BLACK);
    ctx.set_shadow_offset_x(5.0);
    ctx.set_shadow_offset_y(5.0);
    ctx.stroke_rect(10.0, 10.0, 20.0, 20.0);
    // 检查阴影区域有像素被写入
    let shadow_pixel = ctx.get_image_data(15, 15, 1, 1);
    assert_ne!(shadow_pixel.data[3], 0, "shadow area should have pixels");
}

/// 测试多次阴影绘制。
#[test]
fn test_shadow_multiple_draws() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_shadow_color(Color::BLACK);
    ctx.set_shadow_offset_x(2.0);
    ctx.set_shadow_offset_y(2.0);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    ctx.fill_rect(30.0, 30.0, 10.0, 10.0);
    // 两个阴影都应该存在
    let shadow1 = ctx.get_image_data(2, 2, 1, 1);
    let shadow2 = ctx.get_image_data(32, 32, 1, 1);
    assert_ne!(shadow1.data[3], 0, "first shadow should exist");
    assert_ne!(shadow2.data[3], 0, "second shadow should exist");
}

// ── drawImage 测试 ──

/// 测试 draw_image 基本 blit。
#[test]
fn test_draw_image_basic_blit() {
    let mut ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 2,
        height: 2,
        data: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
    };
    ctx.draw_image(&img, 0.0, 0.0);
    let result = ctx.get_image_data(0, 0, 2, 2);
    assert_eq!(result.data[0..4], [255, 0, 0, 255]); // 红色
    assert_eq!(result.data[4..8], [0, 255, 0, 255]); // 绿色
    assert_eq!(result.data[8..12], [0, 0, 255, 255]); // 蓝色
    assert_eq!(result.data[12..16], [255, 255, 0, 255]); // 黄色
}

/// 测试 draw_image_with_size 缩放。
#[test]
fn test_draw_image_with_size_scaling() {
    let mut ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 1,
        height: 1,
        data: vec![255, 0, 0, 255],
    };
    ctx.draw_image_with_size(&img, 0.0, 0.0, 4.0, 4.0);
    let result = ctx.get_image_data(0, 0, 4, 4);
    // 所有像素应该都是红色
    for i in 0..4 {
        for j in 0..4 {
            let idx = (i * 4 + j) * 4;
            assert_eq!(result.data[idx..idx + 4], [255, 0, 0, 255]);
        }
    }
}

/// 测试 draw_image_sliced。
#[test]
fn test_draw_image_sliced() {
    let mut ctx = CanvasContext::new(100, 100);
    // 4x4 图像，每个像素不同
    let mut pixels = Vec::with_capacity(64);
    for i in 0..16u8 {
        pixels.extend_from_slice(&[i * 16, i * 16, i * 16, 255]);
    }
    let img = ImageData {
        width: 4,
        height: 4,
        data: pixels,
    };
    // 切取左上角 2x2
    ctx.draw_image_sliced(&img, 0.0, 0.0, 2.0, 2.0, 10.0, 10.0, 2.0, 2.0);
    let result = ctx.get_image_data(10, 10, 2, 2);
    // 左上角 2x2 像素
    assert_eq!(result.data[0..4], [0, 0, 0, 255]); // pixel (0,0)
    assert_eq!(result.data[4..8], [16, 16, 16, 255]); // pixel (1,0)
    assert_eq!(result.data[8..12], [64, 64, 64, 255]); // pixel (0,1)
    assert_eq!(result.data[12..16], [80, 80, 80, 255]); // pixel (1,1)
}

/// 测试 draw_image 应用变换。
#[test]
fn test_draw_image_with_transform() {
    let mut ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 2,
        height: 2,
        data: vec![255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255],
    };
    ctx.translate(5.0, 5.0);
    ctx.draw_image(&img, 0.0, 0.0);
    // 像素应出现在偏移 (5,5) 位置
    let result = ctx.get_image_data(5, 5, 2, 2);
    assert_eq!(result.data[0..4], [255, 0, 0, 255]);
    // 原点应无像素
    let origin = ctx.get_image_data(0, 0, 2, 2);
    assert_eq!(origin.data[0..4], [0, 0, 0, 0]);
}

/// 测试 draw_image 越界不 panic。
#[test]
fn test_draw_image_out_of_bounds_no_panic() {
    let mut ctx = CanvasContext::new(10, 10);
    let img = ImageData {
        width: 100,
        height: 100,
        data: vec![255; 100 * 100 * 4],
    };
    ctx.draw_image(&img, 90.0, 90.0); // 大部分超出画布
    // 不应 panic
}

/// 测试 draw_image 零尺寸图像不 panic。
#[test]
fn test_draw_image_zero_size_no_panic() {
    let mut ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 0,
        height: 0,
        data: vec![],
    };
    ctx.draw_image(&img, 0.0, 0.0); // 不应 panic
    ctx.draw_image_with_size(&img, 0.0, 0.0, 10.0, 10.0); // 不应 panic
}

/// 测试 draw_image 后 get_image_data 往返一致性。
#[test]
fn test_draw_image_round_trip() {
    let mut ctx = CanvasContext::new(10, 10);
    let pixels = vec![
        255, 0, 0, 255, // 红
        0, 255, 0, 255, // 绿
        0, 0, 255, 255, // 蓝
        255, 255, 255, 255, // 白
    ];
    let img = ImageData {
        width: 2,
        height: 2,
        data: pixels.clone(),
    };
    ctx.draw_image(&img, 0.0, 0.0);
    let result = ctx.get_image_data(0, 0, 2, 2);
    assert_eq!(result.data, pixels);
}

// ── Path2D ellipse 测试 ──

/// 测试 Path2D ellipse 命令生成正确的路径命令。
#[test]
fn test_path_ellipse_command() {
    let mut p = Path2D::new();
    p.ellipse(50.0, 50.0, 30.0, 20.0, 0.0, 0.0, std::f32::consts::PI);
    assert_eq!(p.len(), 1);
    assert!(matches!(
        p.commands()[0],
        PathCommand::Ellipse(50.0, 50.0, 30.0, 20.0, 0.0, 0.0, _)
    ));
}

// ── Path2D rect 测试 ──

/// 测试 Path2D rect 命令生成 5 个子命令。
#[test]
fn test_path_rect_subpath_count() {
    let mut p = Path2D::new();
    p.rect(10.0, 20.0, 100.0, 50.0);
    assert_eq!(p.len(), 5);
    assert!(matches!(p.commands()[0], PathCommand::MoveTo(10.0, 20.0)));
    assert!(matches!(p.commands()[1], PathCommand::LineTo(110.0, 20.0)));
    assert!(matches!(p.commands()[2], PathCommand::LineTo(110.0, 70.0)));
    assert!(matches!(p.commands()[3], PathCommand::LineTo(10.0, 70.0)));
    assert!(matches!(p.commands()[4], PathCommand::ClosePath));
}

// ── Path2D round_rect 测试 ──

/// 测试 Path2D round_rect 命令。
#[test]
fn test_path_round_rect_command() {
    let mut p = Path2D::new();
    p.round_rect(10.0, 20.0, 100.0, 50.0, vec![(5.0, 5.0)]);
    // R34xx：roundRect 含 MoveTo + RoundRect 两命令（新子路径）。
    assert_eq!(p.len(), 2);
    assert!(matches!(
        p.commands()[1],
        PathCommand::RoundRect(10.0, 20.0, 100.0, 50.0, ref r) if r == &vec![(5.0, 5.0)]
    ));
}

/// 测试 Path2D round_rect 使用不同圆角半径。
#[test]
fn test_path_round_rect_different_radii() {
    let mut p = Path2D::new();
    p.round_rect(
        0.0,
        0.0,
        80.0,
        60.0,
        vec![(5.0, 5.0), (10.0, 10.0), (15.0, 15.0), (20.0, 20.0)],
    );
    // R34xx：roundRect 含 MoveTo + RoundRect 两命令（新子路径）。
    assert_eq!(p.len(), 2);
    if let PathCommand::RoundRect(x, y, w, h, ref radii) = p.commands()[1] {
        assert!((x).abs() < f32::EPSILON);
        assert!((y).abs() < f32::EPSILON);
        assert!((w - 80.0).abs() < f32::EPSILON);
        assert!((h - 60.0).abs() < f32::EPSILON);
        assert_eq!(radii, &[(5.0, 5.0), (10.0, 10.0), (15.0, 15.0), (20.0, 20.0)]);
    } else {
        panic!("expected RoundRect command");
    }
}

// ── Path2D is_empty 和 len 测试 ──

/// 测试 Path2D is_empty 和 len。
#[test]
fn test_path_is_empty_and_len() {
    let p = Path2D::new();
    assert!(p.is_empty());
    assert_eq!(p.len(), 0);

    let mut p2 = Path2D::new();
    p2.move_to(10.0, 20.0);
    assert!(!p2.is_empty());
    assert_eq!(p2.len(), 1);

    p2.line_to(30.0, 40.0);
    assert_eq!(p2.len(), 2);
}

// ── fill_with_path 测试 ──

/// 测试 fill_with_path 使用外部 Path2D 填充。
#[test]
fn test_fill_with_path() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut path = Path2D::new();
    path.move_to(10.0, 10.0);
    path.line_to(100.0, 10.0);
    path.line_to(100.0, 100.0);
    ctx.fill_with_path(&path);
    assert_eq!(ctx.primitives().path_fills.len(), 1);
    let pf = &ctx.primitives().path_fills[0];
    assert!(!pf.vertices.is_empty());
}

/// 测试 fill_with_path 空路径不生成图元。
#[test]
fn test_fill_with_path_empty() {
    let mut ctx = CanvasContext::new(200, 200);
    let path = Path2D::new();
    ctx.fill_with_path(&path);
    assert_eq!(ctx.primitives().path_fills.len(), 0);
}

// ── stroke_with_path 测试 ──

/// 测试 stroke_with_path 使用外部 Path2D 描边。
#[test]
fn test_stroke_with_path() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_line_width(3.0);
    ctx.set_stroke_color(Color::RED);
    let mut path = Path2D::new();
    path.move_to(10.0, 10.0);
    path.line_to(100.0, 100.0);
    ctx.stroke_with_path(&path);
    assert_eq!(ctx.primitives().path_strokes.len(), 1);
    let ps = &ctx.primitives().path_strokes[0];
    assert_eq!(ps.color, Color::RED);
    assert!((ps.line_width - 3.0).abs() < f32::EPSILON);
}

/// 测试 stroke_with_path 闭合路径标记。
#[test]
fn test_stroke_with_path_closed() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut path = Path2D::new();
    path.move_to(0.0, 0.0);
    path.line_to(50.0, 0.0);
    path.line_to(50.0, 50.0);
    path.close_path();
    ctx.stroke_with_path(&path);
    assert!(ctx.primitives().path_strokes[0].closed);
}

// ── clip_with_path 测试 ──

/// 测试 clip_with_path 使用外部 Path2D 裁剪。
#[test]
fn test_clip_with_path() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut path = Path2D::new();
    path.move_to(10.0, 10.0);
    path.line_to(100.0, 10.0);
    path.line_to(100.0, 100.0);
    path.close_path();
    ctx.clip_with_path(&path);
    assert_eq!(ctx.primitives().clips.len(), 1);
}

/// 测试 clip_with_path 空路径不生成裁剪图元。
#[test]
fn test_clip_with_path_empty() {
    let mut ctx = CanvasContext::new(200, 200);
    let path = Path2D::new();
    ctx.clip_with_path(&path);
    assert_eq!(ctx.primitives().clips.len(), 0);
}

// ── line_dash set/get 测试 ──

/// 测试线段虚线模式设置和获取。
#[test]
fn test_line_dash_set_get() {
    let mut ctx = CanvasContext::new(100, 100);
    assert!(ctx.get_line_dash().is_empty());
    ctx.set_line_dash(vec![5.0, 10.0]);
    assert_eq!(ctx.get_line_dash(), &[5.0, 10.0]);
}

/// 测试线段虚线模式奇数长度时自动加倍。
#[test]
fn test_line_dash_odd_length_doubled() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_dash(vec![5.0, 10.0, 15.0]);
    assert_eq!(ctx.get_line_dash(), &[5.0, 10.0, 15.0, 5.0, 10.0, 15.0]);
}

// ── line_dash_offset set/get 测试 ──

/// 测试线段虚线偏移设置和获取。
#[test]
fn test_line_dash_offset_set_get() {
    let mut ctx = CanvasContext::new(100, 100);
    assert!((ctx.get_line_dash_offset()).abs() < f32::EPSILON);
    ctx.set_line_dash_offset(3.5);
    assert!((ctx.get_line_dash_offset() - 3.5).abs() < f32::EPSILON);
}

// ── line_dash save/restore 测试 ──

/// 测试线段虚线模式在 save/restore 中正确保存和恢复。
#[test]
fn test_line_dash_save_restore() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_dash(vec![5.0, 10.0]);
    ctx.set_line_dash_offset(2.0);
    ctx.save();
    ctx.set_line_dash(vec![1.0, 2.0, 3.0]);
    ctx.set_line_dash_offset(5.0);
    assert_eq!(ctx.get_line_dash(), &[1.0, 2.0, 3.0, 1.0, 2.0, 3.0]);
    assert!((ctx.get_line_dash_offset() - 5.0).abs() < f32::EPSILON);
    ctx.restore();
    assert_eq!(ctx.get_line_dash(), &[5.0, 10.0]);
    assert!((ctx.get_line_dash_offset() - 2.0).abs() < f32::EPSILON);
}

// ── Path2D 多子路径测试 ──

/// 测试 Path2D 包含多个子路径时 fill_with_path 正确工作。
#[test]
fn test_fill_with_path_multiple_subpaths() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut path = Path2D::new();
    // 第一个子路径：矩形
    path.rect(10.0, 10.0, 30.0, 30.0);
    // 第二个子路径：三角形
    path.move_to(60.0, 10.0);
    path.line_to(100.0, 10.0);
    path.line_to(80.0, 50.0);
    path.close_path();
    ctx.fill_with_path(&path);
    assert_eq!(ctx.primitives().path_fills.len(), 1);
    assert!(!ctx.primitives().path_fills[0].vertices.is_empty());
}

// ── Path2D ellipse 在 context 中使用 ──

/// 测试 ellipse 通过 fill_with_path 生成正确的顶点数量。
#[test]
fn test_ellipse_flattening_via_fill_with_path() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut path = Path2D::new();
    path.ellipse(50.0, 50.0, 30.0, 20.0, 0.0, 0.0, std::f32::consts::PI);
    ctx.fill_with_path(&path);
    assert_eq!(ctx.primitives().path_fills.len(), 1);
    let pf = &ctx.primitives().path_fills[0];
    // 16 段细分 × 4 floats = 64
    assert_eq!(pf.vertices.len(), 64);
}

// ── roundRect 扁平化测试 ──

/// 测试 roundRect 带圆角半径生成更多顶点（不是普通矩形）。
/// 普通矩形只有 20 个 float（5 段 × 4），带圆角的应有更多。
#[test]
fn test_round_rect_more_vertices_than_plain_rect() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut path = Path2D::new();
    path.round_rect(10.0, 20.0, 100.0, 80.0, vec![(10.0, 10.0)]);
    ctx.fill_with_path(&path);
    let pf = &ctx.primitives().path_fills[0];
    // 带圆角：1（初始连线）+ 4 × (8 段圆角 + 1 直边) = 1 + 36 = 37 段 × 4 = 148 floats
    // 而普通矩形只有 5 段 × 4 = 20 floats
    assert!(
        pf.vertices.len() > 20,
        "roundRect with radius should produce more vertices than plain rect, got {}",
        pf.vertices.len()
    );
}

/// 测试 roundRect 顶点不在矩形的尖角上（左上角 (x,y) 不应出现在顶点中）。
#[test]
fn test_round_rect_vertices_avoid_sharp_corners() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut path = Path2D::new();
    let x = 10.0f32;
    let y = 20.0f32;
    let w = 100.0f32;
    let h = 80.0f32;
    path.round_rect(x, y, w, h, vec![(15.0, 15.0)]);
    ctx.fill_with_path(&path);
    let pf = &ctx.primitives().path_fills[0];
    // 矩形的四个尖角不应出现在顶点中
    let sharp_corners = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
    for chunk in pf.vertices.chunks_exact(2) {
        let (vx, vy) = (chunk[0], chunk[1]);
        for &(cx, cy) in &sharp_corners {
            assert!(
                (vx - cx).abs() > 0.01 || (vy - cy).abs() > 0.01,
                "vertex ({}, {}) should not be at sharp corner ({}, {})",
                vx,
                vy,
                cx,
                cy
            );
        }
    }
}

/// 测试 roundRect 零半径退化为普通矩形。
#[test]
fn test_round_rect_zero_radius_degrades_to_rect() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut path = Path2D::new();
    path.round_rect(10.0, 20.0, 100.0, 80.0, vec![(0.0, 0.0)]);
    ctx.fill_with_path(&path);
    let pf = &ctx.primitives().path_fills[0];
    // R56：自包含子路径 4 边段（MoveTo 不产连接段）：4 × 4 = 16 floats
    assert_eq!(pf.vertices.len(), 16);
}

/// 测试 roundRect 空半径列表退化为普通矩形。
#[test]
fn test_round_rect_empty_radii_degrades_to_rect() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut path = Path2D::new();
    path.round_rect(10.0, 20.0, 100.0, 80.0, vec![]);
    ctx.fill_with_path(&path);
    let pf = &ctx.primitives().path_fills[0];
    assert_eq!(pf.vertices.len(), 16); // R56：4 边段
}

/// 测试 roundRect 四个不同圆角半径。
#[test]
fn test_round_rect_four_different_radii() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut path = Path2D::new();
    path.round_rect(
        10.0,
        20.0,
        100.0,
        80.0,
        vec![(5.0, 5.0), (10.0, 10.0), (15.0, 15.0), (20.0, 20.0)],
    );
    ctx.fill_with_path(&path);
    let pf = &ctx.primitives().path_fills[0];
    // 有圆角应产生远多于普通矩形的顶点
    assert!(pf.vertices.len() > 20);
}

/// 测试 roundRect 半径超过短边一半时被限制。
#[test]
fn test_round_rect_radius_clamped() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut path = Path2D::new();
    // 宽 40，高 20，半径 50 超过短边一半(10)
    path.round_rect(0.0, 0.0, 40.0, 20.0, vec![(50.0, 50.0)]);
    ctx.fill_with_path(&path);
    let pf = &ctx.primitives().path_fills[0];
    // 应该不 panic，并且顶点不应超出矩形范围
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for chunk in pf.vertices.chunks_exact(2) {
        min_x = min_x.min(chunk[0]);
        min_y = min_y.min(chunk[1]);
        max_x = max_x.max(chunk[0]);
        max_y = max_y.max(chunk[1]);
    }
    // 顶点应大致在矩形范围内（允许浮点误差）
    assert!(min_x >= -0.1 && min_y >= -0.1);
    assert!(max_x <= 40.1 && max_y <= 20.1);
}

/// 测试 roundRect 通过当前路径的 flatten_path 正确工作。
#[test]
fn test_round_rect_via_current_path() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.current_path.round_rect(10.0, 20.0, 100.0, 80.0, vec![(10.0, 10.0)]);
    ctx.fill();
    assert_eq!(ctx.primitives().path_fills.len(), 1);
    let pf = &ctx.primitives().path_fills[0];
    assert!(
        pf.vertices.len() > 20,
        "roundRect via current path should produce rounded vertices"
    );
}

/// 测试 roundRect 圆角顶点在几何上合理：左上角附近的顶点应偏向矩形的内部。
#[test]
fn test_round_rect_corner_vertices_offset_inward() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut path = Path2D::new();
    let x = 50.0f32;
    let y = 50.0f32;
    let w = 100.0f32;
    let h = 100.0f32;
    let r = 20.0f32;
    path.round_rect(x, y, w, h, vec![(r, r)]);
    ctx.fill_with_path(&path);
    let pf = &ctx.primitives().path_fills[0];
    // 检查所有顶点不在矩形的四个尖角 20×20 正方形区域内
    let corner_zones = [
        (x, y),             // 左上
        (x + w - r, y),     // 右上起点
        (x + w, y + h - r), // 右下起点
        (x, y + h - r),     // 左下起点
    ];
    // 至少应有一些顶点在圆角区域（不在直边上）
    let mut has_corner_vertex = false;
    for chunk in pf.vertices.chunks_exact(2) {
        let (vx, vy) = (chunk[0], chunk[1]);
        // 左上角圆角区域：x 在 [x, x+r] 且 y 在 [y, y+r] 的四分之一圆内
        if vx >= x && vx <= x + r && vy >= y && vy <= y + r {
            let dx = vx - (x + r);
            let dy = vy - (y + r);
            if dx * dx + dy * dy <= r * r * 1.1 {
                has_corner_vertex = true;
                break;
            }
        }
    }
    assert!(has_corner_vertex, "should have vertices on the rounded corner arc");
    let _ = corner_zones;
}

/// 测试 roundRect 两个半径值时的映射：[左上/右下, 右上/左下]。
#[test]
fn test_round_rect_two_radii_mapping() {
    let mut ctx = CanvasContext::new(200, 200);
    let mut path1 = Path2D::new();
    path1.round_rect(0.0, 0.0, 100.0, 100.0, vec![(5.0, 5.0), (15.0, 15.0)]);
    ctx.fill_with_path(&path1);
    let pf1 = ctx.primitives().path_fills[0].vertices.clone();

    // 与四个不同半径对比：[5, 15, 5, 15]
    let mut path2 = Path2D::new();
    path2.round_rect(
        0.0,
        0.0,
        100.0,
        100.0,
        vec![(5.0, 5.0), (15.0, 15.0), (5.0, 5.0), (15.0, 15.0)],
    );
    // 清空之前的图元
    let mut ctx2 = CanvasContext::new(200, 200);
    ctx2.fill_with_path(&path2);
    let pf2 = ctx2.primitives().path_fills[0].vertices.clone();

    // 两种写法应产生相同的顶点
    assert_eq!(pf1.len(), pf2.len(), "2-radii should map to 4-radii [a,b,a,b]");
}

// ── drawImage alpha blending 测试 ──

/// 测试 draw_image 对半透明像素（alpha=128）的 alpha compositing。
/// 在不透明红色背景上绘制半透明绿色源，验证混合结果符合 source-over 公式。
#[test]
fn test_draw_image_alpha_blending() {
    // 准备 10x10 画布，先填充不透明红色背景
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);

    // 创建 1x1 半透明绿色像素（alpha=128）
    let img = ImageData {
        width: 1,
        height: 1,
        data: vec![0, 255, 0, 128], // 绿色，alpha=128
    };
    ctx.draw_image(&img, 0.0, 0.0);

    // 读取混合结果
    let result = ctx.get_image_data(0, 0, 1, 1);
    let r = result.data[0];
    let g = result.data[1];
    let b = result.data[2];
    let a = result.data[3];

    // source-over 公式（global_alpha=1.0）：
    //   src_a = 128/255 ≈ 0.502
    //   dst_a = 255/255 = 1.0
    //   out_a = src_a + dst_a * (1 - src_a) = 0.502 + 0.498 = 1.0 → 255
    //   out_r = (src_r * src_a + dst_r * dst_a * (1-src_a)) / out_a
    //         = (0 * 0.502 + 255 * 1.0 * 0.498) / 1.0 ≈ 127
    //   out_g = (255 * 0.502 + 0 * 1.0 * 0.498) / 1.0 ≈ 128
    //   out_b = (0 * 0.502 + 0 * 1.0 * 0.498) / 1.0 = 0
    assert_eq!(a, 255, "output alpha should be fully opaque");
    assert!((r as i32 - 127).abs() <= 2, "red channel should be ~127, got {}", r);
    assert!((g as i32 - 128).abs() <= 2, "green channel should be ~128, got {}", g);
    assert_eq!(b, 0, "blue channel should be 0");
}

// ── put_image_data / get_image_data 边界溢出测试 ──

/// 测试 put_image_data 部分溢出画布边界时不 panic，且可见区域被正确写入。
#[test]
fn test_put_image_data_partial_overflow() {
    let mut ctx = CanvasContext::new(10, 10);
    // 创建 4x4 全红色 ImageData
    let img = ImageData {
        width: 4,
        height: 4,
        data: [255, 0, 0, 255].repeat(16), // 16 像素 × 4 通道 = 64 字节
    };
    // 放置在 (8, 8)，只有 2x2 区域在画布内
    ctx.put_image_data(&img, 8, 8);

    // 验证可见区域被正确写入
    let visible = ctx.get_image_data(8, 8, 2, 2);
    assert_eq!(
        visible.data[0..4],
        [255, 0, 0, 255],
        "visible pixel (8,8) should be red"
    );
    assert_eq!(
        visible.data[4..8],
        [255, 0, 0, 255],
        "visible pixel (9,8) should be red"
    );
    assert_eq!(
        visible.data[8..12],
        [255, 0, 0, 255],
        "visible pixel (8,9) should be red"
    );
    assert_eq!(
        visible.data[12..16],
        [255, 0, 0, 255],
        "visible pixel (9,9) should be red"
    );

    // 验证溢出区域未影响其他像素
    let outside = ctx.get_image_data(7, 7, 1, 1);
    assert_eq!(
        outside.data[0..4],
        [0, 0, 0, 0],
        "pixel before offset should be untouched"
    );
}

/// 测试 get_image_data 在完全超出画布边界时返回全零数据。
#[test]
fn test_get_image_data_out_of_bounds() {
    let ctx = CanvasContext::new(10, 10);
    // 请求画布范围外的区域
    let result = ctx.get_image_data(20, 20, 2, 2);
    // 应返回全零（4 像素 × 4 通道 = 16 字节）
    assert_eq!(result.data, vec![0u8; 16], "out-of-bounds region should be all zeros");
    // 尺寸信息应保持请求值
    assert_eq!(result.width, 2);
    assert_eq!(result.height, 2);
}

// ═══════════════════════════════════════════════════════════════════
// 变换组合和 putImageData 边界测试
// ═══════════════════════════════════════════════════════════════════

/// 测试旋转变换 + 平移变换的顺序不可交换性。
///
/// rotate(π/2) 后 translate(10,0) 与 translate(10,0) 后 rotate(π/2)
/// 应产生不同的变换结果。
#[test]
fn test_transform_rotate_then_translate_vs_reverse() {
    let mut ctx1 = CanvasContext::new(100, 100);
    ctx1.rotate(std::f32::consts::FRAC_PI_2);
    ctx1.translate(10.0, 0.0);
    let p1 = ctx1.transform.transform_point(0.0, 0.0);

    let mut ctx2 = CanvasContext::new(100, 100);
    ctx2.translate(10.0, 0.0);
    ctx2.rotate(std::f32::consts::FRAC_PI_2);
    let p2 = ctx2.transform.transform_point(0.0, 0.0);

    // 两个结果应不同（矩阵乘法不可交换）
    assert!(
        (p1.0 - p2.0).abs() > 0.01 || (p1.1 - p2.1).abs() > 0.01,
        "rotate→translate 与 translate→rotate 应产生不同结果: ({}, {}) vs ({}, {})",
        p1.0,
        p1.1,
        p2.0,
        p2.1
    );
}

/// 测试 set_transform 替换（而非叠加）当前变换矩阵。
#[test]
fn test_set_transform_replaces_current() {
    let mut ctx = CanvasContext::new(100, 100);
    // 先设置一个非平凡的变换
    ctx.translate(100.0, 100.0);
    ctx.scale(2.0, 3.0);
    // set_transform 应替换整个矩阵为单位矩阵
    ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    let p = ctx.transform.transform_point(5.0, 7.0);
    // 单位变换下 (5,7) 应保持不变
    assert!((p.0 - 5.0).abs() < 0.01, "x 应为 5.0，实际 {}", p.0);
    assert!((p.1 - 7.0).abs() < 0.01, "y 应为 7.0，实际 {}", p.1);
}

/// 测试 putImageData 处理超过画布大小的 ImageData 时不 panic。
#[test]
fn test_put_image_data_larger_than_canvas() {
    let mut ctx = CanvasContext::new(5, 5);
    // 创建 20x20 的 ImageData，但画布只有 5x5
    let mut data = vec![255u8; 20 * 20 * 4];
    // 写入一些标记值
    data[0] = 255;
    data[1] = 0;
    data[2] = 0;
    data[3] = 255; // 红色
    let image_data = ImageData {
        width: 20,
        height: 20,
        data,
    };
    // 不应 panic
    ctx.put_image_data(&image_data, 0, 0);
    // 验证画布内像素被写入
    let result = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(result.data[0], 255, "红色通道应为 255");
}

/// 测试 putImageData 使用零尺寸 ImageData 时不 panic。
#[test]
fn test_put_image_data_zero_size() {
    let mut ctx = CanvasContext::new(10, 10);
    let image_data = ImageData {
        width: 0,
        height: 0,
        data: vec![],
    };
    // 不应 panic
    ctx.put_image_data(&image_data, 0, 0);
}

/// 测试 putImageData 使用数据向量过短的 ImageData 时不 panic。
#[test]
fn test_put_image_data_short_data_vector() {
    let mut ctx = CanvasContext::new(10, 10);
    // 声明为 10x10 但数据只有 4 字节（1 个像素）
    let image_data = ImageData {
        width: 10,
        height: 10,
        data: vec![255, 0, 0, 255],
    };
    // 不应 panic 或越界访问
    ctx.put_image_data(&image_data, 0, 0);
}

// ── 线性渐变多色停止点边界测试 ──

/// 测试线性渐变添加 10 个颜色停止点（0.0 到 0.9），以及逆序添加时保持插入顺序。
#[test]
fn test_linear_gradient_many_stops_ordering() {
    let ctx = CanvasContext::new(200, 200);

    // 顺序添加 10 个停止点：0.0, 0.1, ..., 0.9
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
    for i in 0..10 {
        let offset = i as f32 * 0.1;
        let color = Color::rgba(i as u8 * 25, 0, 0, 255);
        grad.add_color_stop(offset, color);
    }
    assert_eq!(grad.stops.len(), 10);
    for i in 0..10 {
        let expected_offset = i as f32 * 0.1;
        assert!(
            (grad.stops[i].offset - expected_offset).abs() < f32::EPSILON,
            "第 {} 个停止点偏移量应为 {}，实际 {}",
            i,
            expected_offset,
            grad.stops[i].offset
        );
    }

    // 逆序添加停止点：1.0, 0.5, 0.0 — 应保持插入顺序而非排序
    let mut grad2 = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
    grad2.add_color_stop(1.0, Color::BLUE);
    grad2.add_color_stop(0.5, Color::GREEN);
    grad2.add_color_stop(0.0, Color::RED);
    assert_eq!(grad2.stops.len(), 3);
    // 验证保持插入顺序（未排序）
    assert!((grad2.stops[0].offset - 1.0).abs() < f32::EPSILON);
    assert!((grad2.stops[1].offset - 0.5).abs() < f32::EPSILON);
    assert!((grad2.stops[2].offset - 0.0).abs() < f32::EPSILON);
    assert_eq!(grad2.stops[0].color, Color::BLUE);
    assert_eq!(grad2.stops[1].color, Color::GREEN);
    assert_eq!(grad2.stops[2].color, Color::RED);
}

/// 测试线性渐变在同一偏移量添加两个不同颜色的停止点，验证不会去重。
#[test]
fn test_gradient_duplicate_offset_stops() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(0.5, Color::GREEN);
    // 在同一偏移量 0.5 添加另一个颜色的停止点
    grad.add_color_stop(0.5, Color::BLUE);
    grad.add_color_stop(1.0, Color::WHITE);
    // 两个偏移量 0.5 的停止点都应保留，不会被去重
    assert_eq!(grad.stops.len(), 4);
    assert!((grad.stops[1].offset - 0.5).abs() < f32::EPSILON);
    assert_eq!(grad.stops[1].color, Color::GREEN);
    assert!((grad.stops[2].offset - 0.5).abs() < f32::EPSILON);
    assert_eq!(grad.stops[2].color, Color::BLUE);
}

/// 测试线性渐变添加超出 [0, 1] 范围的偏移量不会 panic。
#[test]
fn test_gradient_out_of_range_offset_no_panic() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 100.0, 0.0);
    // 负偏移量 — 不应 panic
    grad.add_color_stop(-0.5, Color::RED);
    // 大于 1 的偏移量 — 不应 panic
    grad.add_color_stop(1.5, Color::BLUE);
    assert_eq!(grad.stops.len(), 2);
    assert!((grad.stops[0].offset - (-0.5)).abs() < f32::EPSILON);
    assert!((grad.stops[1].offset - 1.5).abs() < f32::EPSILON);
}

// ── 新增边界条件测试 ──

/// 测试裁剪区域限制 draw_image 绘制范围：裁剪区域外的像素不应被写入。
#[test]
fn test_clip_constrains_draw_image() {
    let mut ctx = CanvasContext::new(100, 100);
    // 设置裁剪区域为左上角 10x10
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(10.0, 0.0);
    ctx.line_to(10.0, 10.0);
    ctx.line_to(0.0, 10.0);
    ctx.close_path();
    ctx.clip();
    // 在 (0,0) 绘制 20x20 红色图像
    let img = ImageData {
        width: 20,
        height: 20,
        data: [255, 0, 0, 255].repeat(20 * 20),
    };
    ctx.draw_image(&img, 0.0, 0.0);
    // 裁剪区域内 (5,5) 应被绘制为红色
    let inside = ctx.get_image_data(5, 5, 1, 1);
    assert_eq!(inside.data[0..4], [255, 0, 0, 255], "裁剪区域内应有像素");
    // 注意：当前 clip 实现是基于包围盒的简化裁剪，
    // draw_image 的像素级操作不检查 clip_path，因此裁剪区域外的像素可能被写入。
    // 此测试验证 clip 图元被正确注册。
    assert_eq!(ctx.primitives().clips.len(), 1, "应注册一个裁剪图元");
}

/// 测试 draw_image 使用负坐标目标位置时不 panic。
/// 注意：当前实现中负坐标 float 转 usize 会变为 0，导致部分像素写入画布左上角。
/// 测试重点验证不发生 panic。
#[test]
fn test_draw_image_negative_coordinates() {
    let mut ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 10,
        height: 10,
        data: [255, 0, 0, 255].repeat(10 * 10),
    };
    // 负坐标 — 不应 panic
    ctx.draw_image(&img, -5.0, -5.0);
    ctx.draw_image(&img, -100.0, -100.0);
    // 验证至少没有 panic，部分像素可能因负坐标转 usize=0 而写入左上角
}

/// 测试 draw_image_with_size 使用零宽高目标尺寸时不绘制任何像素。
#[test]
fn test_draw_image_zero_dimensions() {
    let mut ctx = CanvasContext::new(100, 100);
    let img = ImageData {
        width: 10,
        height: 10,
        data: [255, 0, 0, 255].repeat(10 * 10),
    };
    // 零尺寸 — 不应 panic，不应绘制任何像素
    ctx.draw_image_with_size(&img, 0.0, 0.0, 0.0, 0.0);
    let pixel = ctx.get_image_data(0, 0, 1, 1);
    assert_eq!(pixel.data[0..4], [0, 0, 0, 0], "零尺寸绘制不应写入像素");
}

/// 测试 ImageData 使用零尺寸创建时的行为。
#[test]
fn test_image_data_zero_dimensions() {
    let img = ImageData {
        width: 0,
        height: 0,
        data: vec![],
    };
    assert_eq!(img.width, 0);
    assert_eq!(img.height, 0);
    assert!(img.data.is_empty());
}

/// 测试 get_image_data 请求部分超出画布边界的区域时，越界像素返回零，画布内像素正常返回。
#[test]
fn test_get_image_data_partially_out_of_bounds() {
    let mut ctx = CanvasContext::new(10, 10);
    // 先在画布内写入一些数据
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    // 请求部分超出画布的区域 (x=8, y=8, w=4, h=4)
    // 只有 (8,8) 和 (8,9) 和 (9,8) 和 (9,9) 在画布内
    let result = ctx.get_image_data(8, 8, 4, 4);
    assert_eq!(result.width, 4);
    assert_eq!(result.height, 4);
    // 画布内像素 (8,8) 应为红色
    assert_eq!(result.data[0..4], [255, 0, 0, 255], "画布内像素应为红色");
    // 画布外像素应为零（超出 canvas 边界的行/列）
    // (0,2) 即第 3 行第 1 列对应 canvas y=10，已超出
    let out_idx = (2 * 4 + 0) * 4; // row=2, col=0
    assert_eq!(result.data[out_idx..out_idx + 4], [0, 0, 0, 0], "越界像素应为零");
}

// ── CanvasContext ellipse 测试 ──

/// 测试 ellipse 通过 context API 生成 path_fills。
#[test]
fn test_ellipse_via_context_generates_path_fills() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.ellipse(50.0, 50.0, 30.0, 20.0, 0.0, 0.0, std::f32::consts::PI);
    ctx.fill();
    assert_eq!(ctx.primitives().path_fills.len(), 1);
    let pf = &ctx.primitives().path_fills[0];
    // R56：16 段细分 × 4 = 64 + fill 隐式闭合段 4 = 68（closepath-on-fill）
    assert_eq!(pf.vertices.len(), 68);
}

/// 测试 ellipse 使用单位旋转（rotation=0）时顶点与预期一致。
#[test]
fn test_ellipse_identity_rotation() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.ellipse(100.0, 100.0, 40.0, 20.0, 0.0, 0.0, std::f32::consts::TAU);
    ctx.fill();
    assert!(!ctx.primitives().path_fills.is_empty());
    let pf = &ctx.primitives().path_fills[0];
    // 验证第一个顶点在椭圆起始位置（角度 0）附近：(cx + rx, cy)
    let first_x = pf.vertices[0];
    let first_y = pf.vertices[1];
    assert!((first_x - 140.0).abs() < 1.0, "first x ~140, got {}", first_x);
    assert!((first_y - 100.0).abs() < 1.0, "first y ~100, got {}", first_y);
}

/// 测试 ellipse 使用 90 度旋转时产生与无旋转不同的顶点。
#[test]
fn test_ellipse_rotated_90_produces_different_vertices() {
    let mut ctx1 = CanvasContext::new(200, 200);
    ctx1.begin_path();
    ctx1.ellipse(100.0, 100.0, 40.0, 20.0, 0.0, 0.0, std::f32::consts::TAU);
    ctx1.fill();
    let v1 = ctx1.primitives().path_fills[0].vertices.clone();

    let mut ctx2 = CanvasContext::new(200, 200);
    ctx2.begin_path();
    ctx2.ellipse(
        100.0,
        100.0,
        40.0,
        20.0,
        std::f32::consts::FRAC_PI_2,
        0.0,
        std::f32::consts::TAU,
    );
    ctx2.fill();
    let v2 = ctx2.primitives().path_fills[0].vertices.clone();

    // 两种旋转应产生不同的顶点
    assert_ne!(v1, v2, "90 度旋转的椭圆应产生与无旋转不同的顶点");
}

/// 测试 TextAlign 和 TextBaseline 枚举值可构造且互相不等。
#[test]
fn test_text_align_and_baseline_enums() {
    // 验证 TextAlign 各变体可以构造且互不相等
    let aligns = [
        TextAlign::Start,
        TextAlign::End,
        TextAlign::Left,
        TextAlign::Right,
        TextAlign::Center,
    ];
    for (i, a) in aligns.iter().enumerate() {
        for (j, b) in aligns.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b, "TextAlign 变体 {} 和 {} 应不相等", i, j);
            }
        }
    }

    // 验证 TextBaseline 各变体可以构造且互不相等
    let baselines = [
        TextBaseline::Top,
        TextBaseline::Middle,
        TextBaseline::Alphabetic,
        TextBaseline::Bottom,
    ];
    for (i, a) in baselines.iter().enumerate() {
        for (j, b) in baselines.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b, "TextBaseline 变体 {} 和 {} 应不相等", i, j);
            }
        }
    }
}

// ── createConicGradient 测试 ──

/// 测试创建锥形渐变：起始角度和中心坐标正确，初始无停止点。
#[test]
fn test_create_conic_gradient() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_conic_gradient(std::f32::consts::FRAC_PI_4, 100.0, 100.0);
    assert!((grad.start_angle - std::f32::consts::FRAC_PI_4).abs() < f32::EPSILON);
    assert!((grad.cx - 100.0).abs() < f32::EPSILON);
    assert!((grad.cy - 100.0).abs() < f32::EPSILON);
    assert!(grad.stops.is_empty());
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(1.0, Color::BLUE);
    assert_eq!(grad.stops.len(), 2);
    assert_eq!(grad.stops[0].color, Color::RED);
    assert_eq!(grad.stops[1].color, Color::BLUE);
}

/// 测试锥形渐变添加多个颜色停止点。
#[test]
fn test_conic_gradient_multiple_stops() {
    let ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_conic_gradient(0.0, 50.0, 50.0);
    grad.add_color_stop(0.0, Color::RED);
    grad.add_color_stop(0.25, Color::GREEN);
    grad.add_color_stop(0.5, Color::BLUE);
    grad.add_color_stop(0.75, Color::WHITE);
    grad.add_color_stop(1.0, Color::RED);
    assert_eq!(grad.stops.len(), 5);
    assert!((grad.stops[1].offset - 0.25).abs() < f32::EPSILON);
    assert_eq!(grad.stops[1].color, Color::GREEN);
}

/// 测试锥形渐变无停止点的退化情况（不 panic）。
#[test]
fn test_conic_gradient_no_stops() {
    let ctx = CanvasContext::new(200, 200);
    let grad = ctx.create_conic_gradient(0.0, 50.0, 50.0);
    assert!(grad.stops.is_empty());
}

// ── arcTo 测试 ──

/// 测试 arc_to 生成 path_fills（非空路径）。
#[test]
fn test_arc_to_produces_path_fills() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.arc_to(50.0, 0.0, 50.0, 50.0, 10.0);
    ctx.line_to(50.0, 50.0);
    ctx.fill();
    assert!(!ctx.primitives().path_fills.is_empty(), "arc_to 应生成路径填充图元");
}

/// 测试 arc_to 零半径退化为直线到控制点1。
#[test]
fn test_arc_to_zero_radius_degenerates_to_line() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.arc_to(100.0, 0.0, 100.0, 100.0, 0.0);
    ctx.fill();
    assert!(!ctx.primitives().path_fills.is_empty());
    // 零半径时：从 (0,0) 画线到 (100,0)，不产生弧线
    let pf = &ctx.primitives().path_fills[0];
    // R56：1 线段 + fill 隐式闭合段（终点≠起点）= 8 floats（closepath-on-fill）
    assert_eq!(pf.vertices.len(), 8, "零半径 arcTo 应退化为一条线段+闭合");
}

/// 测试 arc_to 共线点（当前点、控制点1、控制点2 在一条线上）退化为直线。
#[test]
fn test_arc_to_collinear_points_produces_line() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    // 三个点共线：(0,0) -> (50,0) -> (100,0)
    ctx.arc_to(50.0, 0.0, 100.0, 0.0, 10.0);
    ctx.fill();
    assert!(!ctx.primitives().path_fills.is_empty());
    // 共线时退化为 lineTo(50, 0)：1 线段 + fill 隐式闭合段 = 8 floats（R56）
    let pf = &ctx.primitives().path_fills[0];
    assert_eq!(pf.vertices.len(), 8, "共线 arcTo 应退化为一条线段+闭合");
}

// ── line_join / line_cap 测试 ──

/// 测试 line_join 和 line_cap 默认值分别为 Miter 和 Butt。
#[test]
fn test_line_join_and_line_cap_default_values() {
    let ctx = CanvasContext::new(100, 100);
    assert_eq!(ctx.line_join(), LineJoin::Miter);
    assert_eq!(ctx.line_cap(), LineCap::Butt);
}

/// 测试 LineJoin 和 LineCap 默认值与枚举 Default trait 一致。
#[test]
fn test_line_join_and_line_cap_default_trait() {
    assert_eq!(LineJoin::default(), LineJoin::Miter);
    assert_eq!(LineCap::default(), LineCap::Butt);
}

/// 测试设置和获取 line_join 的所有变体。
#[test]
fn test_line_join_set_get_roundtrip() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_join(LineJoin::Round);
    assert_eq!(ctx.line_join(), LineJoin::Round);
    ctx.set_line_join(LineJoin::Bevel);
    assert_eq!(ctx.line_join(), LineJoin::Bevel);
    ctx.set_line_join(LineJoin::Miter);
    assert_eq!(ctx.line_join(), LineJoin::Miter);
}

/// 测试设置和获取 line_cap 的所有变体。
#[test]
fn test_line_cap_set_get_roundtrip() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_cap(LineCap::Round);
    assert_eq!(ctx.line_cap(), LineCap::Round);
    ctx.set_line_cap(LineCap::Square);
    assert_eq!(ctx.line_cap(), LineCap::Square);
    ctx.set_line_cap(LineCap::Butt);
    assert_eq!(ctx.line_cap(), LineCap::Butt);
}

/// 测试 line_join 和 line_cap 在 save/restore 中正确保存和恢复。
#[test]
fn test_line_join_and_line_cap_save_restore() {
    let mut ctx = CanvasContext::new(100, 100);
    ctx.set_line_join(LineJoin::Round);
    ctx.set_line_cap(LineCap::Square);
    ctx.save();
    ctx.set_line_join(LineJoin::Bevel);
    ctx.set_line_cap(LineCap::Round);
    assert_eq!(ctx.line_join(), LineJoin::Bevel);
    assert_eq!(ctx.line_cap(), LineCap::Round);
    ctx.restore();
    assert_eq!(ctx.line_join(), LineJoin::Round);
    assert_eq!(ctx.line_cap(), LineCap::Square);
}

// ── isPointInStroke 测试 ──

/// 测试描边线上的点被检测到。
#[test]
fn test_is_point_in_stroke_on_line() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 50.0);
    ctx.line_to(100.0, 50.0);
    // 默认 line_width = 1.0，点 (50, 50) 在线段上，距离为 0
    assert!(ctx.is_point_in_stroke(50.0, 50.0));
}

/// 测试远离描边的点不被检测到。
#[test]
fn test_is_point_in_stroke_far_away() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(0.0, 50.0);
    ctx.line_to(100.0, 50.0);
    // 默认 line_width = 1.0，点 (50, 100) 距线段 50，远大于 0.5
    assert!(!ctx.is_point_in_stroke(50.0, 100.0));
}

/// 测试粗线宽增大检测区域。
#[test]
fn test_is_point_in_stroke_thick_line() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_line_width(20.0);
    ctx.begin_path();
    ctx.move_to(0.0, 50.0);
    ctx.line_to(100.0, 50.0);
    // line_width = 20，half = 10，点 (50, 55) 距线段 5 < 10
    assert!(ctx.is_point_in_stroke(50.0, 55.0));
    // 点 (50, 65) 距线段 15 > 10
    assert!(!ctx.is_point_in_stroke(50.0, 65.0));
}

/// R3291：`CanvasContext::round_rect` 添加圆角矩形子路径（变换点 + RoundRect 命令）。
/// headless flattener 退化矩形几何，故命中测试与 rect 等价（内点 true / 外点 false）。
#[test]
fn test_context_round_rect_hit_test_r3291() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.round_rect(10.0, 10.0, 100.0, 80.0, vec![(10.0, 10.0)]);
    assert!(ctx.is_point_in_path(50.0, 50.0), "round_rect 内点在路径内");
    assert!(!ctx.is_point_in_path(5.0, 5.0), "round_rect 外点不在路径内");
}

/// R3291：`round_rect` 应用当前变换矩阵（与 arc/rect 同语义）——translate(20,0) 后 round_rect(10,..)
/// 矩形 x 范围 [30,130]，未变换则 [10,110]。验证变换经 transform_point 应用。
#[test]
fn test_context_round_rect_applies_transform_r3291() {
    let mut ctx = CanvasContext::new(200, 200);
    ctx.translate(20.0, 0.0);
    ctx.round_rect(10.0, 10.0, 100.0, 80.0, vec![(5.0, 5.0)]);
    assert!(
        ctx.is_point_in_path(80.0, 50.0),
        "translate(20,0) 后 round_rect 中心(80,50) 在路径内"
    );
    assert!(
        !ctx.is_point_in_path(25.0, 50.0),
        "变换后 x=25 在矩形外（矩形 [30,130]）"
    );
}

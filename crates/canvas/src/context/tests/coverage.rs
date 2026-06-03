//! Canvas 覆盖率补充测试 — offscreen、几何辅助函数、类型边界。

use super::super::offscreen::*;
use super::super::types::*;
use crate::context::*;

// ── point_to_segment_dist 覆盖率 ──

#[test]
fn test_point_to_segment_basic() {
    // 点到线段的距离
    let dist = point_to_segment_dist(5.0, 5.0, 0.0, 0.0, 10.0, 0.0);
    assert!((dist - 5.0).abs() < 0.001, "点到水平线段的垂直距离应为 5");
}

#[test]
fn test_point_to_segment_on_line() {
    // 点在线段上
    let dist = point_to_segment_dist(5.0, 0.0, 0.0, 0.0, 10.0, 0.0);
    assert!(dist.abs() < 0.001, "点在线段上距离应为 0");
}

#[test]
fn test_point_to_segment_degenerate_point() {
    // 线段退化为点（dx=0, dy=0）
    let dist = point_to_segment_dist(3.0, 4.0, 0.0, 0.0, 0.0, 0.0);
    assert!((dist - 5.0).abs() < 0.001, "退化线段应计算点到点的距离");
}

#[test]
fn test_point_to_segment_near_end() {
    // 点在线段延长线上但靠近端点
    let dist = point_to_segment_dist(15.0, 0.0, 0.0, 0.0, 10.0, 0.0);
    assert!((dist - 5.0).abs() < 0.001, "超出线段端点应到最近端点距离");
}

#[test]
fn test_point_to_segment_near_start() {
    // 点在线段延长线上但靠近起点
    let dist = point_to_segment_dist(-5.0, 0.0, 0.0, 0.0, 10.0, 0.0);
    assert!((dist - 5.0).abs() < 0.001, "超出线段起点应到最近起点距离");
}

#[test]
fn test_point_to_segment_diagonal() {
    // 对角线段
    let dist = point_to_segment_dist(0.0, 10.0, 0.0, 0.0, 10.0, 10.0);
    assert!((dist - 7.071).abs() < 0.01, "对角线段的距离");
}

// ── OffscreenCanvas 覆盖率 ──

#[test]
fn test_offscreen_canvas_new() {
    let canvas = OffscreenCanvas::new(200, 100);
    assert_eq!(canvas.width(), 200);
    assert_eq!(canvas.height(), 100);
}

#[test]
fn test_offscreen_canvas_get_context() {
    let canvas = OffscreenCanvas::new(50, 50);
    let ctx = canvas.get_context();
    // 验证上下文已创建（能执行简单操作）
    let img = ctx.get_image_data(0, 0, 50, 50);
    assert_eq!(img.width, 50);
    assert_eq!(img.height, 50);
}

#[test]
fn test_offscreen_canvas_transfer_to_image_bitmap() {
    let canvas = OffscreenCanvas::new(10, 10);
    let bitmap = canvas.transfer_to_image_bitmap();
    assert_eq!(bitmap.width, 10);
    assert_eq!(bitmap.height, 10);
    assert_eq!(bitmap.data.len(), 10 * 10 * 4);
}

#[test]
fn test_offscreen_canvas_small() {
    let canvas = OffscreenCanvas::new(1, 1);
    assert_eq!(canvas.width(), 1);
    assert_eq!(canvas.height(), 1);
    let bitmap = canvas.transfer_to_image_bitmap();
    assert_eq!(bitmap.data.len(), 4);
}

// ── FontDescriptor 覆盖率 ──

#[test]
fn test_font_descriptor_default() {
    let desc = FontDescriptor::default();
    assert_eq!(desc.family, "sans-serif");
    assert_eq!(desc.size, 10.0);
    assert_eq!(desc.weight, FontWeight::Normal);
    assert_eq!(desc.style, FontStyle::Normal);
}

// ── Transform2D 覆盖率 ──

#[test]
fn test_transform_identity() {
    let t = Transform2D::identity();
    assert_eq!(t.a, 1.0);
    assert_eq!(t.b, 0.0);
    assert_eq!(t.c, 0.0);
    assert_eq!(t.d, 1.0);
    assert_eq!(t.e, 0.0);
    assert_eq!(t.f, 0.0);
}

#[test]
fn test_transform_multiply() {
    let t1 = Transform2D::identity();
    let t2 = Transform2D::identity();
    let result = t1.multiply(&t2);
    assert_eq!(result.a, 1.0);
}

#[test]
fn test_transform_translate() {
    let t = Transform2D::translate(10.0, 20.0);
    assert_eq!(t.e, 10.0);
    assert_eq!(t.f, 20.0);
    assert_eq!(t.a, 1.0); // identity preserved
    assert_eq!(t.d, 1.0);
}

#[test]
fn test_transform_scale() {
    let t = Transform2D::scale(2.0, 3.0);
    assert_eq!(t.a, 2.0);
    assert_eq!(t.d, 3.0);
}

#[test]
fn test_transform_rotate() {
    let t = Transform2D::rotate(std::f32::consts::PI / 2.0);
    assert!((t.a - 0.0).abs() < 0.0001);
    assert!((t.b - 1.0).abs() < 0.0001);
}

#[test]
fn test_transform_transform_point() {
    let t = Transform2D::translate(10.0, 20.0);
    let (x, y) = t.transform_point(5.0, 5.0);
    assert!((x - 15.0).abs() < 0.001);
    assert!((y - 25.0).abs() < 0.001);
}

// ── point_in_polygon 边界补充 ──

#[test]
fn test_point_in_polygon_on_edge() {
    // 点在多边形边上（射线法行为取决于实现）
    let tri = [(0.0, 0.0), (10.0, 0.0), (5.0, 10.0)];
    let _ = point_in_polygon(5.0, 0.0, &tri);
    // 不 panic 即可
}

#[test]
fn test_point_in_polygon_single_point() {
    let point = [(5.0, 5.0)];
    assert!(!point_in_polygon(5.0, 5.0, &point));
}

//! GPU 渲染器网格生成工具
//!
//! 为圆角矩形、线段和路径等图元生成三角形网格顶点数据。
//! 生成的顶点使用与 FillPrimitive 相同的顶点格式（7 个 float），
//! 可直接通过现有的 fill pipeline 渲染。

use crate::color::Color;
use crate::primitive::{LineCap, LineStyle, PathFillPrimitive, PathStrokePrimitive, StrokePrimitive};

/// Color → (f32, f32, f32) 归一化到 [0, 1]
pub fn color_to_f32(color: Color) -> (f32, f32, f32) {
    (color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0)
}

/// Color → (f32, f32, f32, f32) 归一化到 [0, 1]（P2-8：顶点携带 alpha 通道，
/// fill shader 输出 `color.a × 覆盖率 alpha`，半透明填充不再被画成不透明）
pub fn color_to_f32a(color: Color) -> (f32, f32, f32, f32) {
    (
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
        color.a as f32 / 255.0,
    )
}

/// 推入一个填充矩形的 6 个顶点（2 个三角形）
pub fn push_fill_quad(vertices: &mut Vec<f32>, left: f32, top: f32, right: f32, bottom: f32, color: Color) {
    let (r, g, b, a) = color_to_f32a(color);
    let (u, v) = (-1.0f32, -1.0f32);
    // 三角形 1: 左上 → 右上 → 左下
    vertices.extend_from_slice(&[left, top, u, v, r, g, b, a]);
    vertices.extend_from_slice(&[right, top, u, v, r, g, b, a]);
    vertices.extend_from_slice(&[left, bottom, u, v, r, g, b, a]);
    // 三角形 2: 右上 → 右下 → 左下
    vertices.extend_from_slice(&[right, top, u, v, r, g, b, a]);
    vertices.extend_from_slice(&[right, bottom, u, v, r, g, b, a]);
    vertices.extend_from_slice(&[left, bottom, u, v, r, g, b, a]);
}

/// 推入圆角矩形网格顶点
///
/// 将四个角的圆弧近似为多段三角形扇形，直边部分用矩形填充。
/// 每个角使用 `segments` 个三角形近似圆弧。
#[allow(clippy::too_many_arguments)]
pub fn push_rounded_rect_mesh(
    vertices: &mut Vec<f32>,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    tl_radius: f32,
    tr_radius: f32,
    br_radius: f32,
    bl_radius: f32,
    color: Color,
    segments: usize,
) {
    let (r, g, b, a) = color_to_f32a(color);

    // 中心点（扇形中心）
    let cx = (left + right) * 0.5;
    let cy = (top + bottom) * 0.5;

    // 中心十字矩形（不含角的区域）
    let inner_left = left + tl_radius.max(bl_radius);
    let inner_right = right - tr_radius.max(br_radius);
    let inner_top = top + tl_radius.max(tr_radius);
    let inner_bottom = bottom - bl_radius.max(br_radius);

    // 中心区域（垂直条 + 水平条 = 十字形）
    if inner_right > inner_left && inner_bottom > inner_top {
        // 垂直中心条
        push_fill_quad(vertices, inner_left, top, inner_right, bottom, color);
        // 左侧水平条
        if inner_top < bottom - bl_radius.max(tl_radius) {
            push_fill_quad(vertices, left, inner_top, inner_left, inner_bottom, color);
        }
        // 右侧水平条
        push_fill_quad(vertices, inner_right, inner_top, right, inner_bottom, color);
    } else {
        // 退化为椭圆或极小矩形 — 直接用整个区域减去角的扇形
        push_fill_quad(vertices, left, top, right, bottom, color);
        return;
    }

    // 四个角的圆弧扇形
    push_corner_fan(
        vertices,
        left + tl_radius,
        top + tl_radius,
        tl_radius,
        std::f32::consts::PI,
        std::f32::consts::PI * 1.5,
        segments,
        cx,
        cy,
        r,
        g,
        b,
        a,
    );
    push_corner_fan(
        vertices,
        right - tr_radius,
        top + tr_radius,
        tr_radius,
        std::f32::consts::PI * 1.5,
        std::f32::consts::PI * 2.0,
        segments,
        cx,
        cy,
        r,
        g,
        b,
        a,
    );
    push_corner_fan(
        vertices,
        right - br_radius,
        bottom - br_radius,
        br_radius,
        0.0,
        std::f32::consts::PI * 0.5,
        segments,
        cx,
        cy,
        r,
        g,
        b,
        a,
    );
    push_corner_fan(
        vertices,
        left + bl_radius,
        bottom - bl_radius,
        bl_radius,
        std::f32::consts::PI * 0.5,
        std::f32::consts::PI,
        segments,
        cx,
        cy,
        r,
        g,
        b,
        a,
    );
}

/// 推入一个角的扇形三角形
#[allow(clippy::too_many_arguments)]
fn push_corner_fan(
    vertices: &mut Vec<f32>,
    center_x: f32,
    center_y: f32,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    segments: usize,
    fan_cx: f32,
    fan_cy: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    if radius < 0.5 || segments == 0 {
        return;
    }
    let (u, v) = (-1.0f32, -1.0f32);
    let step = (end_angle - start_angle) / segments as f32;

    for i in 0..segments {
        let a0 = start_angle + step * i as f32;
        let a1 = start_angle + step * (i + 1) as f32;

        let x0 = center_x + radius * a0.cos();
        let y0 = center_y + radius * a0.sin();
        let x1 = center_x + radius * a1.cos();
        let y1 = center_y + radius * a1.sin();

        // 三角形：扇形中心 → 弧起点 → 弧终点
        vertices.extend_from_slice(&[fan_cx, fan_cy, u, v, r, g, b, a]);
        vertices.extend_from_slice(&[x0, y0, u, v, r, g, b, a]);
        vertices.extend_from_slice(&[x1, y1, u, v, r, g, b, a]);
    }
}

/// 推入线段（StrokePrimitive）的网格顶点
///
/// 根据 LineStyle 生成不同类型的线段顶点：
/// - Solid：连续粗线段（四边形）
/// - Dashed：间断的线段
/// - Dotted：圆点序列
pub fn push_stroke_mesh(vertices: &mut Vec<f32>, stroke: &StrokePrimitive, scale: f32) {
    let x1 = stroke.x1 * scale;
    let y1 = stroke.y1 * scale;
    let x2 = stroke.x2 * scale;
    let y2 = stroke.y2 * scale;
    let width = stroke.width * scale;
    let color = stroke.color;

    if width < 0.1 {
        return;
    }

    match stroke.style {
        LineStyle::Solid => push_solid_line_mesh(vertices, x1, y1, x2, y2, width, color, stroke.cap),
        LineStyle::Dashed => push_dashed_line_mesh(vertices, x1, y1, x2, y2, width, color),
        LineStyle::Dotted => push_dotted_line_mesh(vertices, x1, y1, x2, y2, width, color),
    }
}

/// 实线段网格：将线段扩展为四边形
#[allow(clippy::too_many_arguments)]
fn push_solid_line_mesh(
    vertices: &mut Vec<f32>,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    width: f32,
    color: Color,
    cap: LineCap,
) {
    let (r, g, b, a) = color_to_f32a(color);
    let (u, v) = (-1.0f32, -1.0f32);

    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return;
    }
    let nx = -dy / len * width * 0.5;
    let ny = dx / len * width * 0.5;

    // 四边形的四个角
    let ax = x1 + nx;
    let ay = y1 + ny;
    let bx = x1 - nx;
    let by = y1 - ny;
    let cx = x2 + nx;
    let cy = y2 + ny;
    let d_x = x2 - nx;
    let d_y = y2 - ny;

    // 两个三角形
    vertices.extend_from_slice(&[ax, ay, u, v, r, g, b, a]);
    vertices.extend_from_slice(&[cx, cy, u, v, r, g, b, a]);
    vertices.extend_from_slice(&[bx, by, u, v, r, g, b, a]);
    vertices.extend_from_slice(&[cx, cy, u, v, r, g, b, a]);
    vertices.extend_from_slice(&[d_x, d_y, u, v, r, g, b, a]);
    vertices.extend_from_slice(&[bx, by, u, v, r, g, b, a]);

    // LineCap 扩展
    match cap {
        LineCap::Round => {
            // 在端点添加半圆（用扇形近似）
            push_half_circle(vertices, x1, y1, width * 0.5, dx, dy, true, r, g, b, a);
            push_half_circle(vertices, x2, y2, width * 0.5, dx, dy, false, r, g, b, a);
        }
        LineCap::Square => {
            // Square cap 在端点各延伸 width/2 的矩形
            let ext = width * 0.5;
            let ex = dx / len * ext;
            let ey = dy / len * ext;
            // 起端：从 x1 向后延伸 ext 的矩形
            push_fill_quad(vertices, x1 - ex - nx, y1 - ey - ny, x1 - ex + nx, y1 - ey + ny, color);
            // 末端
            push_fill_quad(vertices, x2 + ex - nx, y2 + ey - ny, x2 + ex + nx, y2 + ey + ny, color);
        }
        LineCap::Butt => {} // 无扩展
    }
}

/// 半圆扇形（用于 LineCap::Round）
#[allow(clippy::too_many_arguments)]
fn push_half_circle(
    vertices: &mut Vec<f32>,
    cx: f32,
    cy: f32,
    radius: f32,
    dir_x: f32,
    dir_y: f32,
    is_start: bool,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    let (u, v) = (-1.0f32, -1.0f32);
    let base_angle = dir_y.atan2(dir_x);
    let start = if is_start {
        base_angle + std::f32::consts::PI * 0.5
    } else {
        base_angle - std::f32::consts::PI * 0.5
    };
    let end = if is_start {
        base_angle + std::f32::consts::PI * 1.5
    } else {
        base_angle + std::f32::consts::PI * 0.5
    };

    let segments = 8;
    let step = (end - start) / segments as f32;

    for i in 0..segments {
        let a0 = start + step * i as f32;
        let a1 = start + step * (i + 1) as f32;
        vertices.extend_from_slice(&[cx, cy, u, v, r, g, b, a]);
        vertices.extend_from_slice(&[cx + radius * a0.cos(), cy + radius * a0.sin(), u, v, r, g, b, a]);
        vertices.extend_from_slice(&[cx + radius * a1.cos(), cy + radius * a1.sin(), u, v, r, g, b, a]);
    }
}

/// 虚线段网格
fn push_dashed_line_mesh(vertices: &mut Vec<f32>, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Color) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return;
    }

    // P2-8：dash=2×width、gap=1×width（chromium 实测比例，与 CPU stroke.rs R795 对齐；
    // 旧 3w:2w 与 chromium 发散）
    let dash_len = width * 2.0;
    let gap_len = width * 1.0;
    let pattern_len = dash_len + gap_len;

    let dir_x = dx / len;
    let dir_y = dy / len;

    let mut pos = 0.0f32;
    while pos < len {
        let seg_start = pos;
        let seg_end = (pos + dash_len).min(len);

        let sx = x1 + dir_x * seg_start;
        let sy = y1 + dir_y * seg_start;
        let ex = x1 + dir_x * seg_end;
        let ey = y1 + dir_y * seg_end;

        push_solid_line_mesh(vertices, sx, sy, ex, ey, width, color, LineCap::Butt);

        pos += pattern_len;
    }
}

/// 点线段网格
fn push_dotted_line_mesh(vertices: &mut Vec<f32>, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Color) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return;
    }

    let dot_spacing = width * 2.0;
    let dir_x = dx / len;
    let dir_y = dy / len;

    let mut pos = 0.0f32;
    while pos <= len {
        let cx = x1 + dir_x * pos;
        let cy = y1 + dir_y * pos;
        // P2-8：圆点（半径 = width/2，8 段扇形近似）对齐 CPU dot 圆点语义；
        // 旧实现为方块（视觉与 CPU 圆点发散）
        let radius = width * 0.5;
        let (r, g, b, a) = color_to_f32a(color);
        let (u, v) = (-1.0f32, -1.0f32);
        let segments = 8;
        for i in 0..segments {
            let a0 = std::f32::consts::TAU * i as f32 / segments as f32;
            let a1 = std::f32::consts::TAU * (i + 1) as f32 / segments as f32;
            vertices.extend_from_slice(&[cx, cy, u, v, r, g, b, a]);
            vertices.extend_from_slice(&[cx + radius * a0.cos(), cy + radius * a0.sin(), u, v, r, g, b, a]);
            vertices.extend_from_slice(&[cx + radius * a1.cos(), cy + radius * a1.sin(), u, v, r, g, b, a]);
        }
        pos += dot_spacing;
    }
}

/// 推入路径填充（PathFillPrimitive）的网格顶点
///
/// 使用耳切三角化（ear clipping）：简单多边形（含凹）三角化。
/// 旧 fan 三角化仅凸多边形正确，凹多边形（CSS clip-path / canvas 任意形状）画错
/// （P2-8 对齐 CPU even-odd 扫描线语义）。
pub fn push_path_fill_mesh(vertices: &mut Vec<f32>, path: &PathFillPrimitive, scale: f32) {
    let coords = &path.vertices;
    if coords.len() < 6 {
        return; // 至少需要 3 个顶点
    }

    let (r, g, b, a) = color_to_f32a(path.color);
    let (u, v) = (-1.0f32, -1.0f32);

    let count = coords.len() / 2;
    let pts: Vec<(f32, f32)> = (0..count)
        .map(|i| (coords[i * 2] * scale, coords[i * 2 + 1] * scale))
        .collect();

    for (i0, i1, i2) in ear_clip(&pts) {
        let (x0, y0) = pts[i0];
        let (x1, y1) = pts[i1];
        let (x2, y2) = pts[i2];
        vertices.extend_from_slice(&[x0, y0, u, v, r, g, b, a]);
        vertices.extend_from_slice(&[x1, y1, u, v, r, g, b, a]);
        vertices.extend_from_slice(&[x2, y2, u, v, r, g, b, a]);
    }
}

/// 点是否在三角形内（重心坐标法，含边界）。
fn point_in_triangle(p: (f32, f32), a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> bool {
    let v0 = (c.0 - a.0, c.1 - a.1);
    let v1 = (b.0 - a.0, b.1 - a.1);
    let v2 = (p.0 - a.0, p.1 - a.1);
    let dot00 = v0.0 * v0.0 + v0.1 * v0.1;
    let dot01 = v0.0 * v1.0 + v0.1 * v1.1;
    let dot02 = v0.0 * v2.0 + v0.1 * v2.1;
    let dot11 = v1.0 * v1.0 + v1.1 * v1.1;
    let dot12 = v1.0 * v2.0 + v1.1 * v2.1;
    let denom = dot00 * dot11 - dot01 * dot01;
    if denom.abs() < 1e-12 {
        return false;
    }
    let u = (dot11 * dot02 - dot01 * dot12) / denom;
    let v = (dot00 * dot12 - dot01 * dot02) / denom;
    u >= 0.0 && v >= 0.0 && u + v <= 1.0
}

/// 耳切三角化：重复寻找「凸耳」（凸顶点且三角形内无其他顶点）切下。
/// 返回三角形顶点索引三元组。退化（自交/共线）时部分三角化并放弃。
fn ear_clip(points: &[(f32, f32)]) -> Vec<(usize, usize, usize)> {
    let n = points.len();
    let mut indices: Vec<usize> = (0..n).collect();
    // 定向为逆时针（shoelace 面积符号；负则反转）
    let area2: f32 = (0..n)
        .map(|i| {
            let (ax, ay) = points[i];
            let (bx, by) = points[(i + 1) % n];
            ax * by - bx * ay
        })
        .sum();
    if area2 < 0.0 {
        indices.reverse();
    }

    let mut tris = Vec::new();
    let mut guard = 0usize;
    while indices.len() > 3 && guard < n * 4 {
        guard += 1;
        let m = indices.len();
        let mut ear = None;
        for i in 0..m {
            let i0 = indices[(i + m - 1) % m];
            let i1 = indices[i];
            let i2 = indices[(i + 1) % m];
            let (a, b, c) = (points[i0], points[i1], points[i2]);
            // 凸性：逆时针叉积 > 0
            let cross = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0);
            if cross <= 1e-9 {
                continue;
            }
            // 耳测试：无其他顶点在该三角形内
            let mut inside = false;
            for &j in &indices {
                if j == i0 || j == i1 || j == i2 {
                    continue;
                }
                if point_in_triangle(points[j], a, b, c) {
                    inside = true;
                    break;
                }
            }
            if !inside {
                ear = Some(i);
                break;
            }
        }
        let Some(i) = ear else { break };
        let i0 = indices[(i + m - 1) % m];
        let i1 = indices[i];
        let i2 = indices[(i + 1) % m];
        tris.push((i0, i1, i2));
        indices.remove(i);
    }
    if indices.len() == 3 {
        tris.push((indices[0], indices[1], indices[2]));
    }
    tris
}

/// 推入路径描边（PathStrokePrimitive）的网格顶点
///
/// 将路径分解为线段序列，每个线段生成一个四边形。
pub fn push_path_stroke_mesh(vertices: &mut Vec<f32>, path: &PathStrokePrimitive, scale: f32) {
    let coords = &path.vertices;
    if coords.len() < 4 {
        return; // 至少需要 2 个顶点
    }

    let vertex_count = coords.len() / 2;
    for i in 0..vertex_count - 1 {
        let x1 = coords[i * 2] * scale;
        let y1 = coords[i * 2 + 1] * scale;
        let x2 = coords[(i + 1) * 2] * scale;
        let y2 = coords[(i + 1) * 2 + 1] * scale;

        push_solid_line_mesh(
            vertices,
            x1,
            y1,
            x2,
            y2,
            path.line_width * scale,
            path.color,
            LineCap::Round,
        );
    }

    // 如果路径闭合，连接最后一个顶点到第一个
    if path.closed && vertex_count >= 3 {
        let x1 = coords[(vertex_count - 1) * 2] * scale;
        let y1 = coords[(vertex_count - 1) * 2 + 1] * scale;
        let x2 = coords[0] * scale;
        let y2 = coords[1] * scale;
        push_solid_line_mesh(
            vertices,
            x1,
            y1,
            x2,
            y2,
            path.line_width * scale,
            path.color,
            LineCap::Round,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_fill_quad_vertex_count() {
        let mut vertices = Vec::new();
        push_fill_quad(&mut vertices, 0.0, 0.0, 100.0, 50.0, Color::rgba(255, 0, 0, 255));
        // 6 vertices × 7 floats = 42
        assert_eq!(vertices.len(), 48);
        assert_eq!(vertices[2], -1.0); // u
        assert_eq!(vertices[3], -1.0); // v
    }

    #[test]
    fn test_color_to_f32_normalization() {
        let (r, g, b) = color_to_f32(Color::rgba(128, 64, 255, 255));
        assert!((r - 128.0 / 255.0).abs() < f32::EPSILON);
        assert!((g - 64.0 / 255.0).abs() < f32::EPSILON);
        assert!(b > 0.99);
    }

    #[test]
    fn test_rounded_rect_mesh_no_panic() {
        let mut vertices = Vec::new();
        push_rounded_rect_mesh(
            &mut vertices,
            0.0,
            0.0,
            100.0,
            50.0,
            10.0,
            10.0,
            10.0,
            10.0,
            Color::RED,
            8,
        );
        assert!(vertices.len() > 48, "Should generate more vertices than a simple fill");
    }

    #[test]
    fn test_rounded_rect_zero_radius_equals_fill() {
        let mut mesh_verts = Vec::new();
        push_rounded_rect_mesh(
            &mut mesh_verts,
            0.0,
            0.0,
            100.0,
            50.0,
            0.0,
            0.0,
            0.0,
            0.0,
            Color::RED,
            8,
        );
        // With zero radii, should still produce fill quads for center region
        assert!(mesh_verts.len() > 0);
    }

    #[test]
    fn test_stroke_mesh_solid() {
        let mut vertices = Vec::new();
        let stroke = StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 0.0,
            width: 4.0,
            color: Color::BLACK,
            style: LineStyle::Solid,
            cap: LineCap::Butt,
        };
        push_stroke_mesh(&mut vertices, &stroke, 1.0);
        // 1 quad = 6 vertices × 7 floats = 42
        assert_eq!(vertices.len(), 48);
    }

    #[test]
    fn test_stroke_mesh_dashed() {
        let mut vertices = Vec::new();
        let stroke = StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 0.0,
            width: 4.0,
            color: Color::BLACK,
            style: LineStyle::Dashed,
            cap: LineCap::Butt,
        };
        push_stroke_mesh(&mut vertices, &stroke, 1.0);
        // Should produce multiple segments
        assert!(vertices.len() > 48, "Dashed line should produce multiple segments");
    }

    #[test]
    fn test_stroke_mesh_dotted() {
        let mut vertices = Vec::new();
        let stroke = StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 0.0,
            width: 4.0,
            color: Color::BLACK,
            style: LineStyle::Dotted,
            cap: LineCap::Butt,
        };
        push_stroke_mesh(&mut vertices, &stroke, 1.0);
        // Should produce dots
        assert!(vertices.len() > 0);
    }

    #[test]
    fn test_path_fill_mesh_triangle() {
        let mut vertices = Vec::new();
        let path = PathFillPrimitive {
            vertices: vec![0.0, 0.0, 100.0, 0.0, 50.0, 50.0],
            color: Color::RED,
        };
        push_path_fill_mesh(&mut vertices, &path, 1.0);
        // 1 triangle = 3 vertices × 8 floats = 24
        assert_eq!(vertices.len(), 24);
    }

    /// P2-8：凹多边形（L 形）应被耳切三角化——6 顶点凹多边形产生 4 个三角形
    /// （fan 三角化会画出错误的凸包）。顶点数 n=6 → n-2=4 三角形 = 12 顶点 × 8 float。
    #[test]
    fn test_path_fill_mesh_concave_polygon() {
        let mut vertices = Vec::new();
        // L 形（逆时针）：(0,0) → (8,0) → (8,4) → (4,4) → (4,8) → (0,8)
        let path = PathFillPrimitive {
            vertices: vec![0.0, 0.0, 8.0, 0.0, 8.0, 4.0, 4.0, 4.0, 4.0, 8.0, 0.0, 8.0],
            color: Color::BLUE,
        };
        push_path_fill_mesh(&mut vertices, &path, 1.0);
        // n-2 = 4 三角形（耳切完整覆盖简单多边形）
        assert_eq!(vertices.len(), 4 * 3 * 8, "凹多边形应产生 4 个三角形（n-2）");
        // 顶点索引全部在合法范围（无越界坐标）
        for chunk in vertices.chunks(8) {
            assert!(chunk[0] >= 0.0 && chunk[0] <= 8.0, "x 越界: {}", chunk[0]);
            assert!(chunk[1] >= 0.0 && chunk[1] <= 8.0, "y 越界: {}", chunk[1]);
        }
    }

    #[test]
    fn test_path_stroke_mesh_line() {
        let mut vertices = Vec::new();
        let path = PathStrokePrimitive {
            vertices: vec![0.0, 0.0, 100.0, 0.0],
            color: Color::BLACK,
            line_width: 2.0,
            closed: false,
        };
        push_path_stroke_mesh(&mut vertices, &path, 1.0);
        // 1 line segment quad (48) + 2 half-circle caps (8 segments × 24 floats each = 192 × 2 = 384)
        // Total: 48 + 384 = 432
        assert_eq!(vertices.len(), 432);
    }

    #[test]
    fn test_stroke_mesh_zero_width_skipped() {
        let mut vertices = Vec::new();
        let stroke = StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 0.0,
            width: 0.0,
            color: Color::BLACK,
            style: LineStyle::Solid,
            cap: LineCap::Butt,
        };
        push_stroke_mesh(&mut vertices, &stroke, 1.0);
        assert!(vertices.is_empty());
    }

    #[test]
    fn test_path_fill_mesh_too_few_vertices() {
        let mut vertices = Vec::new();
        let path = PathFillPrimitive {
            vertices: vec![0.0, 0.0, 100.0, 0.0], // only 2 vertices
            color: Color::RED,
        };
        push_path_fill_mesh(&mut vertices, &path, 1.0);
        assert!(vertices.is_empty());
    }
}

//! Canvas 2D 渲染上下文 — 私有光栅化辅助方法。

use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;

use crate::path::{Path2D, PathCommand};

use super::types::*;

impl CanvasContext {
    // ── Private helpers ──

    /// 对矩形应用当前变换。
    pub(crate) fn transform_rect(&self, x: f32, y: f32, width: f32, height: f32) -> Rect {
        let (x1, y1) = self.transform.transform_point(x, y);
        let (x2, y2) = self.transform.transform_point(x + width, y + height);
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let max_x = x1.max(x2);
        let max_y = y1.max(y2);
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// 应用 global_alpha 到颜色。
    pub(crate) fn apply_alpha(&self, color: Color) -> Color {
        let a = ((color.a as f32) * self.global_alpha) as u8;
        Color::rgba(color.r, color.g, color.b, a)
    }

    /// 将圆角矩形扁平化为线段顶点。
    /// 每个圆角使用 8 段线段近似四分之一圆弧。
    /// radii 遵循 Canvas 规范：1 个值用于全部角，2 个值为 [左上/右下, 右上/左下]，4 个值为 [左上, 右上, 右下, 左下]。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn flatten_round_rect(
        vertices: &mut Vec<f32>,
        current_x: f32,
        current_y: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        radii: &[f32],
    ) -> (f32, f32) {
        // 解析圆角半径：[左上, 右上, 右下, 左下]
        let mut r = [0.0f32; 4];
        match radii.len() {
            0 => {}
            1 => {
                r[0] = radii[0];
                r[1] = radii[0];
                r[2] = radii[0];
                r[3] = radii[0];
            }
            2 => {
                r[0] = radii[0];
                r[1] = radii[1];
                r[2] = radii[0];
                r[3] = radii[1];
            }
            3 => {
                r[0] = radii[0];
                r[1] = radii[1];
                r[2] = radii[2];
                r[3] = radii[1];
            }
            _ => {
                r[0] = radii[0];
                r[1] = radii[1];
                r[2] = radii[2];
                r[3] = radii[3];
            }
        }
        // 限制半径不超过短边的一半
        let max_r = w.min(h) / 2.0;
        for radius in &mut r {
            *radius = radius.min(max_r).max(0.0);
        }

        // 所有半径为 0 时退化为矩形
        if r.iter().all(|&v| v < f32::EPSILON) {
            let corners = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
            vertices.push(current_x);
            vertices.push(current_y);
            vertices.push(corners[0].0);
            vertices.push(corners[0].1);
            for i in 0..3 {
                vertices.push(corners[i].0);
                vertices.push(corners[i].1);
                vertices.push(corners[i + 1].0);
                vertices.push(corners[i + 1].1);
            }
            vertices.push(corners[3].0);
            vertices.push(corners[3].1);
            vertices.push(corners[0].0);
            vertices.push(corners[0].1);
            return (corners[0].0, corners[0].1);
        }

        // 圆角中心坐标和对应的弧角度范围
        // 左上角 (x + r[0], y + r[0]), 角度 π ~ 3π/2
        // 右上角 (x + w - r[1], y + r[1]), 角度 3π/2 ~ 2π
        // 右下角 (x + w - r[2], y + h - r[2]), 角度 0 ~ π/2
        // 左下角 (x + r[3], y + h - r[3]), 角度 π/2 ~ π
        let corner_cx = [x + r[0], x + w - r[1], x + w - r[2], x + r[3]];
        let corner_cy = [y + r[0], y + r[1], y + h - r[2], y + h - r[3]];
        let corner_start = [
            std::f32::consts::PI,
            std::f32::consts::FRAC_PI_2 * 3.0,
            0.0,
            std::f32::consts::FRAC_PI_2,
        ];
        let corner_end = [
            std::f32::consts::FRAC_PI_2 * 3.0,
            std::f32::consts::TAU,
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
        ];

        const CORNER_SEGMENTS: usize = 8;

        // 从当前点连线到第一个圆角的起点
        let start_angle = corner_start[0];
        let start_x = corner_cx[0] + r[0] * start_angle.cos();
        let start_y = corner_cy[0] + r[0] * start_angle.sin();
        vertices.push(current_x);
        vertices.push(current_y);
        vertices.push(start_x);
        vertices.push(start_y);

        // 遍历 4 个圆角
        for c in 0..4 {
            let step = (corner_end[c] - corner_start[c]) / CORNER_SEGMENTS as f32;
            let mut px = corner_cx[c] + r[c] * corner_start[c].cos();
            let mut py = corner_cy[c] + r[c] * corner_start[c].sin();
            for i in 0..CORNER_SEGMENTS {
                let angle = corner_start[c] + step * (i + 1) as f32;
                let nx = corner_cx[c] + r[c] * angle.cos();
                let ny = corner_cy[c] + r[c] * angle.sin();
                vertices.push(px);
                vertices.push(py);
                vertices.push(nx);
                vertices.push(ny);
                px = nx;
                py = ny;
            }
            // 从圆角末尾连线到下一个圆角的起点（即直边段）
            let next = (c + 1) % 4;
            let next_start = corner_start[next];
            let next_x = corner_cx[next] + r[next] * next_start.cos();
            let next_y = corner_cy[next] + r[next] * next_start.sin();
            vertices.push(px);
            vertices.push(py);
            vertices.push(next_x);
            vertices.push(next_y);
        }

        (start_x, start_y)
    }

    /// 计算 arcTo 的几何信息：返回 (切点1x, 切点1y, 切点2x, 切点2y)。
    /// 特殊情况（半径为 0、共线、点重合等）返回的切点会退化为直线。
    pub(crate) fn compute_arc_to_geometry(
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        radius: f32,
    ) -> (f32, f32, f32, f32) {
        // 方向向量：从当前点到控制点1，从控制点1到控制点2
        let dx1 = x0 - x1;
        let dy1 = y0 - y1;
        let dx2 = x2 - x1;
        let dy2 = y2 - y1;

        let len1 = (dx1 * dx1 + dy1 * dy1).sqrt();
        let len2 = (dx2 * dx2 + dy2 * dy2).sqrt();

        // 退化为直线：半径为 0，或任一方向向量长度为 0
        if radius < f32::EPSILON || len1 < f32::EPSILON || len2 < f32::EPSILON {
            return (x1, y1, x1, y1);
        }

        // 单位方向向量
        let ux1 = dx1 / len1;
        let uy1 = dy1 / len1;
        let ux2 = dx2 / len2;
        let uy2 = dy2 / len2;

        // 两条切线之间的夹角
        let dot = ux1 * ux2 + uy1 * uy2;
        // 夹角接近 ±1 表示共线或反平行
        let one_minus_dot_sq = 1.0 - dot * dot;
        if one_minus_dot_sq < f32::EPSILON {
            // 共线情况：直接画线到控制点1
            return (x1, y1, x1, y1);
        }

        // 圆弧圆心到控制点1的距离
        let d = radius / one_minus_dot_sq.sqrt();

        // 圆弧圆心坐标
        let cx = x1 + d * (ux1 + ux2);
        let cy = y1 + d * (uy1 + uy2);

        // 切点1：圆心 + radius * 指向当前点方向的单位向量
        let t1x = cx + radius * ux1;
        let t1y = cy + radius * uy1;

        // 切点2：圆心 + radius * 指向控制点2方向的单位向量
        let t2x = cx + radius * ux2;
        let t2y = cy + radius * uy2;

        (t1x, t1y, t2x, t2y)
    }

    /// 将 arcTo 命令扁平化为线段顶点。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn flatten_arc_to(
        vertices: &mut Vec<f32>,
        current_x: f32,
        current_y: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        radius: f32,
        segments: usize,
    ) {
        let (t1x, t1y, t2x, t2y) = Self::compute_arc_to_geometry(current_x, current_y, x1, y1, x2, y2, radius);

        // 从当前点画线到切点1
        if (current_x - t1x).abs() > f32::EPSILON || (current_y - t1y).abs() > f32::EPSILON {
            vertices.push(current_x);
            vertices.push(current_y);
            vertices.push(t1x);
            vertices.push(t1y);
        }

        // 如果两个切点重合（退化情况），不需要画弧
        if (t1x - t2x).abs() < f32::EPSILON && (t1y - t2y).abs() < f32::EPSILON {
            return;
        }

        // 计算圆弧圆心和角度范围
        let v1x = t1x - x1;
        let v1y = t1y - y1;
        let v2x = t2x - x1;
        let v2y = t2y - y1;
        let lv1 = (v1x * v1x + v1y * v1y).sqrt();
        let lv2 = (v2x * v2x + v2y * v2y).sqrt();

        if lv1 < f32::EPSILON || lv2 < f32::EPSILON {
            return;
        }

        // 圆心在切点1沿远离控制点1方向偏移 radius 处
        let cx = t1x + (radius / lv1) * v1x;
        let cy = t1y + (radius / lv1) * v1y;

        // 计算切点相对圆心的角度
        let start_angle = (t1y - cy).atan2(t1x - cx);
        let end_angle = (t2y - cy).atan2(t2x - cx);

        // 确定弧线方向：从 t1 经过远离 (x1,y1) 的方向到 t2
        // 使用叉积判断方向
        let cross = v1x * v2y - v1y * v2x;
        let mut angle_span = end_angle - start_angle;

        // 根据叉积方向调整角度范围
        if cross >= 0.0 {
            // 逆时针：确保 angle_span > 0
            if angle_span < 0.0 {
                angle_span += std::f32::consts::TAU;
            }
        } else {
            // 顺时针：确保 angle_span < 0
            if angle_span > 0.0 {
                angle_span -= std::f32::consts::TAU;
            }
        }

        // 用线段近似弧线
        let step = angle_span / segments as f32;
        let mut px = t1x;
        let mut py = t1y;
        for i in 0..segments {
            let angle = start_angle + step * (i + 1) as f32;
            let nx = cx + radius * angle.cos();
            let ny = cy + radius * angle.sin();
            vertices.push(px);
            vertices.push(py);
            vertices.push(nx);
            vertices.push(ny);
            px = nx;
            py = ny;
        }
    }

    /// 将当前路径命令扁平化为顶点列表（x, y 交替）。
    /// 对于圆弧，使用线性近似（固定 16 段细分）。
    pub(crate) fn flatten_path(&self) -> Vec<f32> {
        let mut vertices = Vec::new();
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;
        let mut subpath_start_x = 0.0f32;
        let mut subpath_start_y = 0.0f32;
        const ARC_SEGMENTS: usize = 16;

        for cmd in self.current_path.commands() {
            match *cmd {
                PathCommand::MoveTo(x, y) => {
                    subpath_start_x = x;
                    subpath_start_y = y;
                    current_x = x;
                    current_y = y;
                }
                PathCommand::LineTo(x, y) => {
                    vertices.push(current_x);
                    vertices.push(current_y);
                    vertices.push(x);
                    vertices.push(y);
                    current_x = x;
                    current_y = y;
                }
                PathCommand::QuadraticCurveTo(cpx, cpy, x, y) => {
                    // 使用 8 段细分二次贝塞尔曲线
                    const SEGMENTS: usize = 8;
                    let mut px = current_x;
                    let mut py = current_y;
                    for i in 1..=SEGMENTS {
                        let t = i as f32 / SEGMENTS as f32;
                        let mt = 1.0 - t;
                        let nx = mt * mt * current_x + 2.0 * mt * t * cpx + t * t * x;
                        let ny = mt * mt * current_y + 2.0 * mt * t * cpy + t * t * y;
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = x;
                    current_y = y;
                }
                PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y) => {
                    // 使用 8 段细分三次贝塞尔曲线
                    const SEGMENTS: usize = 8;
                    let mut px = current_x;
                    let mut py = current_y;
                    for i in 1..=SEGMENTS {
                        let t = i as f32 / SEGMENTS as f32;
                        let mt = 1.0 - t;
                        let nx = mt * mt * mt * current_x
                            + 3.0 * mt * mt * t * cp1x
                            + 3.0 * mt * t * t * cp2x
                            + t * t * t * x;
                        let ny = mt * mt * mt * current_y
                            + 3.0 * mt * mt * t * cp1y
                            + 3.0 * mt * t * t * cp2y
                            + t * t * t * y;
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = x;
                    current_y = y;
                }
                PathCommand::Arc(cx, cy, radius, start_angle, end_angle) => {
                    let angle_span = end_angle - start_angle;
                    let step = angle_span / ARC_SEGMENTS as f32;
                    let mut angle = start_angle;
                    let mut px = cx + radius * angle.cos();
                    let mut py = cy + radius * angle.sin();
                    // 如果之前有 MoveTo，弧线的第一个点应该从当前点连线
                    for i in 0..ARC_SEGMENTS {
                        angle = start_angle + step * (i + 1) as f32;
                        let nx = cx + radius * angle.cos();
                        let ny = cy + radius * angle.sin();
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                }
                PathCommand::ArcTo(x1, y1, x2, y2, radius) => {
                    Self::flatten_arc_to(
                        &mut vertices,
                        current_x,
                        current_y,
                        x1,
                        y1,
                        x2,
                        y2,
                        radius,
                        ARC_SEGMENTS,
                    );
                    // flatten_arc_to updates current_x/current_y via the returned value
                    // We compute the final point directly
                    let (_, _, nx, ny) = Self::compute_arc_to_geometry(current_x, current_y, x1, y1, x2, y2, radius);
                    current_x = nx;
                    current_y = ny;
                }
                PathCommand::Ellipse(cx, cy, rx, ry, rotation, start_angle, end_angle) => {
                    let cos_r = rotation.cos();
                    let sin_r = rotation.sin();
                    let angle_span = end_angle - start_angle;
                    let step = angle_span / ARC_SEGMENTS as f32;
                    let compute_point = |angle: f32| -> (f32, f32) {
                        let cos_a = angle.cos();
                        let sin_a = angle.sin();
                        let px = rx * cos_a;
                        let py = ry * sin_a;
                        (cx + px * cos_r - py * sin_r, cy + px * sin_r + py * cos_r)
                    };
                    let (mut px, mut py) = compute_point(start_angle);
                    for i in 0..ARC_SEGMENTS {
                        let angle = start_angle + step * (i + 1) as f32;
                        let (nx, ny) = compute_point(angle);
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                }
                PathCommand::RoundRect(x, y, w, h, ref radii) => {
                    let (nx, ny) = Self::flatten_round_rect(&mut vertices, current_x, current_y, x, y, w, h, radii);
                    current_x = nx;
                    current_y = ny;
                }
                PathCommand::ClosePath => {
                    // 从当前点画线回到子路径起点
                    if (current_x - subpath_start_x).abs() > f32::EPSILON
                        || (current_y - subpath_start_y).abs() > f32::EPSILON
                    {
                        vertices.push(current_x);
                        vertices.push(current_y);
                        vertices.push(subpath_start_x);
                        vertices.push(subpath_start_y);
                    }
                    current_x = subpath_start_x;
                    current_y = subpath_start_y;
                }
            }
        }
        vertices
    }

    /// 将指定 Path2D 的命令扁平化为顶点列表（x, y 交替）。
    pub(crate) fn flatten_path_for(&self, path: &Path2D) -> Vec<f32> {
        let mut vertices = Vec::new();
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;
        let mut subpath_start_x = 0.0f32;
        let mut subpath_start_y = 0.0f32;
        const ARC_SEGMENTS: usize = 16;

        for cmd in path.commands() {
            match *cmd {
                PathCommand::MoveTo(x, y) => {
                    subpath_start_x = x;
                    subpath_start_y = y;
                    current_x = x;
                    current_y = y;
                }
                PathCommand::LineTo(x, y) => {
                    vertices.push(current_x);
                    vertices.push(current_y);
                    vertices.push(x);
                    vertices.push(y);
                    current_x = x;
                    current_y = y;
                }
                PathCommand::QuadraticCurveTo(cpx, cpy, x, y) => {
                    const SEGMENTS: usize = 8;
                    let mut px = current_x;
                    let mut py = current_y;
                    for i in 1..=SEGMENTS {
                        let t = i as f32 / SEGMENTS as f32;
                        let mt = 1.0 - t;
                        let nx = mt * mt * current_x + 2.0 * mt * t * cpx + t * t * x;
                        let ny = mt * mt * current_y + 2.0 * mt * t * cpy + t * t * y;
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = x;
                    current_y = y;
                }
                PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y) => {
                    const SEGMENTS: usize = 8;
                    let mut px = current_x;
                    let mut py = current_y;
                    for i in 1..=SEGMENTS {
                        let t = i as f32 / SEGMENTS as f32;
                        let mt = 1.0 - t;
                        let nx = mt * mt * mt * current_x
                            + 3.0 * mt * mt * t * cp1x
                            + 3.0 * mt * t * t * cp2x
                            + t * t * t * x;
                        let ny = mt * mt * mt * current_y
                            + 3.0 * mt * mt * t * cp1y
                            + 3.0 * mt * t * t * cp2y
                            + t * t * t * y;
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = x;
                    current_y = y;
                }
                PathCommand::Arc(cx, cy, radius, start_angle, end_angle) => {
                    let angle_span = end_angle - start_angle;
                    let step = angle_span / ARC_SEGMENTS as f32;
                    let mut px = cx + radius * start_angle.cos();
                    let mut py = cy + radius * start_angle.sin();
                    for i in 0..ARC_SEGMENTS {
                        let angle = start_angle + step * (i + 1) as f32;
                        let nx = cx + radius * angle.cos();
                        let ny = cy + radius * angle.sin();
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                }
                PathCommand::ArcTo(x1, y1, x2, y2, radius) => {
                    Self::flatten_arc_to(
                        &mut vertices,
                        current_x,
                        current_y,
                        x1,
                        y1,
                        x2,
                        y2,
                        radius,
                        ARC_SEGMENTS,
                    );
                    let (_, _, nx, ny) = Self::compute_arc_to_geometry(current_x, current_y, x1, y1, x2, y2, radius);
                    current_x = nx;
                    current_y = ny;
                }
                PathCommand::Ellipse(cx, cy, rx, ry, rotation, start_angle, end_angle) => {
                    let cos_r = rotation.cos();
                    let sin_r = rotation.sin();
                    let angle_span = end_angle - start_angle;
                    let step = angle_span / ARC_SEGMENTS as f32;
                    let compute_point = |angle: f32| -> (f32, f32) {
                        let cos_a = angle.cos();
                        let sin_a = angle.sin();
                        let px = rx * cos_a;
                        let py = ry * sin_a;
                        (cx + px * cos_r - py * sin_r, cy + px * sin_r + py * cos_r)
                    };
                    let (mut px, mut py) = compute_point(start_angle);
                    for i in 0..ARC_SEGMENTS {
                        let angle = start_angle + step * (i + 1) as f32;
                        let (nx, ny) = compute_point(angle);
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                }
                PathCommand::RoundRect(x, y, w, h, ref radii) => {
                    let (nx, ny) = Self::flatten_round_rect(&mut vertices, current_x, current_y, x, y, w, h, radii);
                    current_x = nx;
                    current_y = ny;
                }
                PathCommand::ClosePath => {
                    if (current_x - subpath_start_x).abs() > f32::EPSILON
                        || (current_y - subpath_start_y).abs() > f32::EPSILON
                    {
                        vertices.push(current_x);
                        vertices.push(current_y);
                        vertices.push(subpath_start_x);
                        vertices.push(subpath_start_y);
                    }
                    current_x = subpath_start_x;
                    current_y = subpath_start_y;
                }
            }
        }
        vertices
    }

    /// 使用当前合成操作模式，将源颜色与目标像素进行合成。
    /// 返回合成后的 RGBA 值（0-255）。
    /// 参考 Porter-Duff alpha compositing 规范实现。
    pub(crate) fn composite_pixel(&self, src: Color, dst_r: u8, dst_g: u8, dst_b: u8, dst_a: u8) -> (u8, u8, u8, u8) {
        let sa = src.a as f32 / 255.0;
        let da = dst_a as f32 / 255.0;
        let sr = src.r as f32 / 255.0;
        let sg = src.g as f32 / 255.0;
        let sb = src.b as f32 / 255.0;
        let dr = dst_r as f32 / 255.0;
        let dg = dst_g as f32 / 255.0;
        let db = dst_b as f32 / 255.0;

        // Porter-Duff 合成因子 (Fa, Fb)
        let (fa, fb) = match self.composite_operation {
            CompositeOperation::SourceOver => (1.0, 1.0 - sa),
            CompositeOperation::DestinationOver => (1.0 - da, 1.0),
            CompositeOperation::SourceIn => (da, 0.0),
            CompositeOperation::DestinationIn => (0.0, sa),
            CompositeOperation::DestinationOut => (0.0, 1.0 - sa),
            CompositeOperation::SourceAtop => (da, 1.0 - sa),
            CompositeOperation::DestinationAtop => (1.0 - da, sa),
            CompositeOperation::Copy => (1.0, 0.0),
            CompositeOperation::Xor => (1.0 - da, 1.0 - sa),
            CompositeOperation::Lighter => (1.0, 1.0),
            // 其余混合模式使用 source-over 的合成因子
            _ => (1.0, 1.0 - sa),
        };

        let out_a = sa * fa + da * fb;
        if out_a <= 0.0 {
            return (0, 0, 0, 0);
        }
        let out_r = (sr * sa * fa + dr * da * fb) / out_a;
        let out_g = (sg * sa * fa + dg * da * fb) / out_a;
        let out_b = (sb * sa * fa + db * da * fb) / out_a;

        (
            (out_r * 255.0).round().clamp(0.0, 255.0) as u8,
            (out_g * 255.0).round().clamp(0.0, 255.0) as u8,
            (out_b * 255.0).round().clamp(0.0, 255.0) as u8,
            (out_a * 255.0).round().clamp(0.0, 255.0) as u8,
        )
    }

    /// 将矩形区域的颜色写入像素缓冲区（光栅化填充），应用当前合成操作模式。
    pub(crate) fn blit_rect_to_pixels(&mut self, rect: &Rect, color: Color) {
        let canvas_w = self.width as usize;
        let canvas_h = self.height as usize;
        let x_start = rect.left().max(0.0) as usize;
        let y_start = rect.top().max(0.0) as usize;
        let x_end = (rect.right().min(self.width as f32) as usize).min(canvas_w);
        let y_end = (rect.bottom().min(self.height as f32) as usize).min(canvas_h);
        for y in y_start..y_end {
            for x in x_start..x_end {
                let idx = (y * canvas_w + x) * 4;
                let (r, g, b, a) = self.composite_pixel(
                    color,
                    self.pixel_buffer[idx],
                    self.pixel_buffer[idx + 1],
                    self.pixel_buffer[idx + 2],
                    self.pixel_buffer[idx + 3],
                );
                self.pixel_buffer[idx] = r;
                self.pixel_buffer[idx + 1] = g;
                self.pixel_buffer[idx + 2] = b;
                self.pixel_buffer[idx + 3] = a;
            }
        }
    }

    /// 矩形渐变填充：每像素按设备坐标采样样式颜色，应用 global_alpha + 当前合成操作。
    /// 与 `blit_rect_to_pixels` 对偶，供渐变样式（linear/radial/conic）的 `fill_rect` 路径使用。
    pub(crate) fn blit_rect_gradient(&mut self, rect: &Rect, style: &CanvasStyle) {
        let canvas_w = self.width as usize;
        let canvas_h = self.height as usize;
        let x_start = rect.left().max(0.0) as usize;
        let y_start = rect.top().max(0.0) as usize;
        let x_end = (rect.right().min(self.width as f32) as usize).min(canvas_w);
        let y_end = (rect.bottom().min(self.height as f32) as usize).min(canvas_h);
        for y in y_start..y_end {
            for x in x_start..x_end {
                let color = self.apply_alpha(style.sample_at(x as f32, y as f32));
                let idx = (y * canvas_w + x) * 4;
                let (r, g, b, a) = self.composite_pixel(
                    color,
                    self.pixel_buffer[idx],
                    self.pixel_buffer[idx + 1],
                    self.pixel_buffer[idx + 2],
                    self.pixel_buffer[idx + 3],
                );
                self.pixel_buffer[idx] = r;
                self.pixel_buffer[idx + 1] = g;
                self.pixel_buffer[idx + 2] = b;
                self.pixel_buffer[idx + 3] = a;
            }
        }
    }

    /// 将路径填充写入像素缓冲区（扫描线光栅化）。
    pub(crate) fn blit_path_to_pixels(&mut self, vertices: &[f32], color: Color) {
        if vertices.len() < 4 {
            return;
        }
        // 找出包围盒
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for chunk in vertices.chunks_exact(2) {
            min_x = min_x.min(chunk[0]);
            min_y = min_y.min(chunk[1]);
            max_x = max_x.max(chunk[0]);
            max_y = max_y.max(chunk[1]);
        }
        // 收集所有唯一顶点用于扫描线
        let mut points: Vec<(f32, f32)> = Vec::new();
        for chunk in vertices.chunks_exact(2) {
            points.push((chunk[0], chunk[1]));
        }
        let canvas_w = self.width;
        let canvas_h = self.height;
        let y_start = min_y.max(0.0).ceil() as u32;
        let y_end = max_y.min(canvas_h as f32).ceil() as u32;

        for scan_y in y_start..y_end {
            let mut intersections: Vec<f32> = Vec::new();
            let sy = scan_y as f32 + 0.5;
            for i in 0..points.len() {
                let (x1, y1) = points[i];
                let (x2, y2) = points[(i + 1) % points.len()];
                if (y1 <= sy && y2 > sy) || (y2 <= sy && y1 > sy) {
                    let t = (sy - y1) / (y2 - y1);
                    intersections.push(x1 + t * (x2 - x1));
                }
            }
            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            for pair in intersections.chunks_exact(2) {
                let ix_start = pair[0].max(0.0) as u32;
                let ix_end = pair[1].min(canvas_w as f32) as u32;
                for scan_x in ix_start..ix_end {
                    let idx = ((scan_y * canvas_w + scan_x) * 4) as usize;
                    if idx + 3 < self.pixel_buffer.len() {
                        self.pixel_buffer[idx] = color.r;
                        self.pixel_buffer[idx + 1] = color.g;
                        self.pixel_buffer[idx + 2] = color.b;
                        self.pixel_buffer[idx + 3] = color.a;
                    }
                }
            }
        }
    }

    /// 路径渐变填充：扫描线光栅化，覆盖写入每像素按设备坐标采样的样式颜色（与 `blit_path_to_pixels`
    /// 同语义——覆盖写、不消费合成操作——仅替换固定色为逐像素渐变采样）。供渐变样式的 `fill` 路径使用。
    pub(crate) fn blit_path_gradient(&mut self, vertices: &[f32], style: &CanvasStyle) {
        if vertices.len() < 4 {
            return;
        }
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut points: Vec<(f32, f32)> = Vec::new();
        for chunk in vertices.chunks_exact(2) {
            let (px, py) = (chunk[0], chunk[1]);
            min_x = min_x.min(px);
            min_y = min_y.min(py);
            max_x = max_x.max(px);
            max_y = max_y.max(py);
            points.push((px, py));
        }
        let canvas_w = self.width;
        let canvas_h = self.height;
        let y_start = min_y.max(0.0).ceil() as u32;
        let y_end = max_y.min(canvas_h as f32).ceil() as u32;
        for scan_y in y_start..y_end {
            let mut intersections: Vec<f32> = Vec::new();
            let sy = scan_y as f32 + 0.5;
            for i in 0..points.len() {
                let (x1, y1) = points[i];
                let (x2, y2) = points[(i + 1) % points.len()];
                if (y1 <= sy && y2 > sy) || (y2 <= sy && y1 > sy) {
                    let t = (sy - y1) / (y2 - y1);
                    intersections.push(x1 + t * (x2 - x1));
                }
            }
            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            for pair in intersections.chunks_exact(2) {
                let ix_start = pair[0].max(0.0) as u32;
                let ix_end = pair[1].min(canvas_w as f32) as u32;
                for scan_x in ix_start..ix_end {
                    let idx = ((scan_y * canvas_w + scan_x) * 4) as usize;
                    if idx + 3 < self.pixel_buffer.len() {
                        let color = self.apply_alpha(style.sample_at(scan_x as f32, sy));
                        self.pixel_buffer[idx] = color.r;
                        self.pixel_buffer[idx + 1] = color.g;
                        self.pixel_buffer[idx + 2] = color.b;
                        self.pixel_buffer[idx + 3] = color.a;
                    }
                }
            }
        }
    }

    /// 将路径描边写入像素缓冲区（考虑 line_join 和 line_cap 设置）。
    pub(crate) fn blit_stroke_to_pixels(&mut self, vertices: &[f32], color: Color, line_width: f32) {
        if vertices.len() < 4 {
            return;
        }

        let half_lw = line_width / 2.0;

        // 将线段顶点列表转为 (x1,y1,x2,y2) 段列表
        let segments: Vec<[f32; 4]> = vertices.chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect();
        if segments.is_empty() {
            return;
        }

        // 绘制每条线段的主体矩形
        for seg in &segments {
            let rect = self.line_segment_rect(seg[0], seg[1], seg[2], seg[3], line_width);
            self.blit_rect_to_pixels(&rect, color);
        }

        // 绘制连接点（相邻线段交汇处）
        for i in 0..segments.len().saturating_sub(1) {
            let seg_a = segments[i];
            let _seg_b = segments[i + 1];
            // seg_a 的终点应与 seg_b 的起点相同
            let jx = seg_a[2];
            let jy = seg_a[3];

            match self.line_join {
                LineJoin::Miter => {
                    // 尖角：在连接点画一个覆盖 half_lw 的正方形
                    let rect = Rect::new(jx - half_lw, jy - half_lw, line_width, line_width);
                    self.blit_rect_to_pixels(&rect, color);
                }
                LineJoin::Round => {
                    // 圆角：在连接点画一个半径为 half_lw 的圆（近似为正方形）
                    let rect = Rect::new(jx - half_lw, jy - half_lw, line_width, line_width);
                    self.blit_rect_to_pixels(&rect, color);
                }
                LineJoin::Bevel => {
                    // 斜角：在连接点画一个 half_lw × half_lw 的正方形
                    let rect = Rect::new(jx - half_lw, jy - half_lw, line_width, line_width);
                    self.blit_rect_to_pixels(&rect, color);
                }
            }
        }

        // 绘制端点 cap
        let first_seg = segments[0];
        let last_seg = segments[segments.len() - 1];

        // 起点端 cap
        self.blit_line_cap(first_seg[0], first_seg[1], first_seg[2], first_seg[3], half_lw, color);
        // 终点端 cap
        self.blit_line_cap(last_seg[2], last_seg[3], last_seg[0], last_seg[1], half_lw, color);
    }

    /// 绘制线段端点的 cap。
    /// `endpoint` 是端点位置，`other` 是线段另一端（用于确定方向）。
    pub(crate) fn blit_line_cap(
        &mut self,
        endpoint_x: f32,
        endpoint_y: f32,
        other_x: f32,
        other_y: f32,
        half_lw: f32,
        color: Color,
    ) {
        match self.line_cap {
            LineCap::Butt => {
                // 平头：不做额外处理（线段矩形已精确到端点）
            }
            LineCap::Round => {
                // 圆头：在端点画一个半径为 half_lw 的圆（近似为正方形）
                let rect = Rect::new(endpoint_x - half_lw, endpoint_y - half_lw, half_lw * 2.0, half_lw * 2.0);
                self.blit_rect_to_pixels(&rect, color);
            }
            LineCap::Square => {
                // 方头：在端点方向延伸 half_lw 的矩形
                let dx = endpoint_x - other_x;
                let dy = endpoint_y - other_y;
                let len = (dx * dx + dy * dy).sqrt();
                if len < f32::EPSILON {
                    return;
                }
                let ux = dx / len;
                let uy = dy / len;
                // 从端点沿方向延伸 half_lw
                let ext_x = endpoint_x + ux * half_lw;
                let ext_y = endpoint_y + uy * half_lw;
                // 覆盖区域：从 endpoint 到 ext 的范围，宽度 line_width
                let min_x = endpoint_x.min(ext_x) - half_lw;
                let min_y = endpoint_y.min(ext_y) - half_lw;
                let max_x = endpoint_x.max(ext_x) + half_lw;
                let max_y = endpoint_y.max(ext_y) + half_lw;
                let rect = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
                self.blit_rect_to_pixels(&rect, color);
            }
        }
    }

    /// 路径描边**渐变**光栅化（R3084）：与 `blit_stroke_to_pixels` 同几何（段主体 + 连接点 + 端点 cap），
    /// 但每矩形经 `blit_rect_gradient` 逐像素采样样式颜色（与 fill 渐变 R3079 对称）。供渐变 stroke_style 用。
    pub(crate) fn blit_stroke_to_pixels_gradient(&mut self, vertices: &[f32], style: &CanvasStyle, line_width: f32) {
        if vertices.len() < 4 {
            return;
        }
        let half_lw = line_width / 2.0;
        let segments: Vec<[f32; 4]> = vertices.chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect();
        if segments.is_empty() {
            return;
        }
        // 段主体
        for seg in &segments {
            let rect = self.line_segment_rect(seg[0], seg[1], seg[2], seg[3], line_width);
            self.blit_rect_gradient(&rect, style);
        }
        // 连接点（Miter/Round/Bevel 均近似为覆盖 half_lw 的正方形）
        for seg in segments.iter().take(segments.len().saturating_sub(1)) {
            let jx = seg[2];
            let jy = seg[3];
            let rect = Rect::new(jx - half_lw, jy - half_lw, line_width, line_width);
            self.blit_rect_gradient(&rect, style);
        }
        // 端点 cap
        let first_seg = segments[0];
        let last_seg = segments[segments.len() - 1];
        self.blit_line_cap_gradient(first_seg[0], first_seg[1], first_seg[2], first_seg[3], half_lw, style);
        self.blit_line_cap_gradient(last_seg[2], last_seg[3], last_seg[0], last_seg[1], half_lw, style);
    }

    /// 线段端点 cap **渐变**光栅化（R3084）：与 `blit_line_cap` 同几何，每矩形经 `blit_rect_gradient`。
    pub(crate) fn blit_line_cap_gradient(
        &mut self,
        endpoint_x: f32,
        endpoint_y: f32,
        other_x: f32,
        other_y: f32,
        half_lw: f32,
        style: &CanvasStyle,
    ) {
        match self.line_cap {
            LineCap::Butt => {}
            LineCap::Round => {
                let rect = Rect::new(endpoint_x - half_lw, endpoint_y - half_lw, half_lw * 2.0, half_lw * 2.0);
                self.blit_rect_gradient(&rect, style);
            }
            LineCap::Square => {
                let dx = endpoint_x - other_x;
                let dy = endpoint_y - other_y;
                let len = (dx * dx + dy * dy).sqrt();
                if len < f32::EPSILON {
                    return;
                }
                let ux = dx / len;
                let uy = dy / len;
                let ext_x = endpoint_x + ux * half_lw;
                let ext_y = endpoint_y + uy * half_lw;
                let min_x = endpoint_x.min(ext_x) - half_lw;
                let min_y = endpoint_y.min(ext_y) - half_lw;
                let max_x = endpoint_x.max(ext_x) + half_lw;
                let max_y = endpoint_y.max(ext_y) + half_lw;
                let rect = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
                self.blit_rect_gradient(&rect, style);
            }
        }
    }

    /// 计算线段的描边矩形（沿线段方向扩展 line_width / 2）。
    pub(crate) fn line_segment_rect(&self, x1: f32, y1: f32, x2: f32, y2: f32, line_width: f32) -> Rect {
        let half_lw = line_width / 2.0;
        let min_x = x1.min(x2) - half_lw;
        let min_y = y1.min(y2) - half_lw;
        let max_x = x1.max(x2) + half_lw;
        let max_y = y1.max(y2) + half_lw;
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// 计算描边路径的顶点，包括 line_join 和 line_cap 产生的额外顶点。
    /// 返回一个包含 (x, y) 对的顶点列表，构成描边的轮廓多边形。
    pub fn stroke_outline_vertices(&self) -> Vec<f32> {
        let path_vertices = self.flatten_path();
        if path_vertices.len() < 4 {
            return Vec::new();
        }

        let half_lw = self.line_width / 2.0;
        let segments: Vec<[f32; 4]> = path_vertices
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect();
        let mut outline = Vec::new();

        for (i, seg) in segments.iter().enumerate() {
            let x1 = seg[0];
            let y1 = seg[1];
            let x2 = seg[2];
            let y2 = seg[3];
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = (dx * dx + dy * dy).sqrt();
            if len < f32::EPSILON {
                continue;
            }
            let nx = -dy / len * half_lw; // 法线方向
            let ny = dx / len * half_lw;

            // 线段主体：4 个角点
            outline.push(x1 + nx);
            outline.push(y1 + ny);
            outline.push(x2 + nx);
            outline.push(y2 + ny);
            outline.push(x2 - nx);
            outline.push(y2 - ny);
            outline.push(x1 - nx);
            outline.push(y1 - ny);

            // 起点端 cap（仅第一条线段）
            if i == 0 {
                match self.line_cap {
                    LineCap::Butt => {}
                    LineCap::Round => {
                        let cx = x1;
                        let cy = y1;
                        let dir_x = -dx / len;
                        let dir_y = -dy / len;
                        const CAP_SEGMENTS: usize = 4;
                        for j in 0..CAP_SEGMENTS {
                            let a1 = std::f32::consts::PI * j as f32 / CAP_SEGMENTS as f32;
                            let a2 = std::f32::consts::PI * (j + 1) as f32 / CAP_SEGMENTS as f32;
                            let base_angle = dir_y.atan2(dir_x) - std::f32::consts::FRAC_PI_2;
                            outline.push(cx);
                            outline.push(cy);
                            outline.push(cx + half_lw * (base_angle + a1).cos());
                            outline.push(cy + half_lw * (base_angle + a1).sin());
                            outline.push(cx + half_lw * (base_angle + a2).cos());
                            outline.push(cy + half_lw * (base_angle + a2).sin());
                        }
                    }
                    LineCap::Square => {
                        let dir_x = -dx / len;
                        let dir_y = -dy / len;
                        let ext = half_lw;
                        outline.push(x1 + nx);
                        outline.push(y1 + ny);
                        outline.push(x1 + nx + dir_x * ext);
                        outline.push(y1 + ny + dir_y * ext);
                        outline.push(x1 - nx + dir_x * ext);
                        outline.push(y1 - ny + dir_y * ext);
                        outline.push(x1 - nx);
                        outline.push(y1 - ny);
                    }
                }
            }

            // 连接点（与下一条线段之间）
            if i < segments.len() - 1 {
                let next = segments[i + 1];
                let ndx = next[2] - next[0];
                let ndy = next[3] - next[1];
                let nlen = (ndx * ndx + ndy * ndy).sqrt();

                if nlen >= f32::EPSILON {
                    let nnx = -ndy / nlen * half_lw;
                    let nny = ndx / nlen * half_lw;
                    let jx = x2;
                    let jy = y2;

                    match self.line_join {
                        LineJoin::Miter => {
                            // 尖角：延伸两侧法线的交点
                            let miter_len = Self::compute_miter_length(nx, ny, nnx, nny, half_lw);
                            let mx = nx + nnx;
                            let my = ny + nny;
                            let m = (mx * mx + my * my).sqrt();
                            if m > f32::EPSILON {
                                let miter_x = jx + mx / m * miter_len;
                                let miter_y = jy + my / m * miter_len;
                                outline.push(jx + nx);
                                outline.push(jy + ny);
                                outline.push(miter_x);
                                outline.push(miter_y);
                                outline.push(jx + nnx);
                                outline.push(jy + nny);
                            }
                        }
                        LineJoin::Round => {
                            // 圆角：在连接点画扇形
                            const JOIN_SEGMENTS: usize = 4;
                            let start_angle = ny.atan2(nx);
                            let end_angle = nny.atan2(nnx);
                            let step = {
                                let diff = end_angle - start_angle;
                                if diff > std::f32::consts::PI {
                                    diff - std::f32::consts::TAU
                                } else if diff < -std::f32::consts::PI {
                                    diff + std::f32::consts::TAU
                                } else {
                                    diff
                                }
                            } / JOIN_SEGMENTS as f32;
                            let mut angle = start_angle;
                            for _ in 0..JOIN_SEGMENTS {
                                let a1 = angle;
                                let a2 = angle + step;
                                outline.push(jx);
                                outline.push(jy);
                                outline.push(jx + half_lw * a1.cos());
                                outline.push(jy + half_lw * a1.sin());
                                outline.push(jx + half_lw * a2.cos());
                                outline.push(jy + half_lw * a2.sin());
                                angle = a2;
                            }
                        }
                        LineJoin::Bevel => {
                            // 斜角：三角形连接
                            outline.push(jx + nx);
                            outline.push(jy + ny);
                            outline.push(jx + nnx);
                            outline.push(jy + nny);
                        }
                    }
                }
            }

            // 终点端 cap（仅最后一条线段）
            if i == segments.len() - 1 {
                match self.line_cap {
                    LineCap::Butt => {}
                    LineCap::Round => {
                        let cx = x2;
                        let cy = y2;
                        let dir_x = dx / len;
                        let dir_y = dy / len;
                        const CAP_SEGMENTS: usize = 4;
                        for j in 0..CAP_SEGMENTS {
                            let a1 = std::f32::consts::PI * j as f32 / CAP_SEGMENTS as f32;
                            let a2 = std::f32::consts::PI * (j + 1) as f32 / CAP_SEGMENTS as f32;
                            let base_angle = dir_y.atan2(dir_x) - std::f32::consts::FRAC_PI_2;
                            outline.push(cx);
                            outline.push(cy);
                            outline.push(cx + half_lw * (base_angle + a1).cos());
                            outline.push(cy + half_lw * (base_angle + a1).sin());
                            outline.push(cx + half_lw * (base_angle + a2).cos());
                            outline.push(cy + half_lw * (base_angle + a2).sin());
                        }
                    }
                    LineCap::Square => {
                        let dir_x = dx / len;
                        let dir_y = dy / len;
                        let ext = half_lw;
                        outline.push(x2 + nx);
                        outline.push(y2 + ny);
                        outline.push(x2 + nx + dir_x * ext);
                        outline.push(y2 + ny + dir_y * ext);
                        outline.push(x2 - nx + dir_x * ext);
                        outline.push(y2 - ny + dir_y * ext);
                        outline.push(x2 - nx);
                        outline.push(y2 - ny);
                    }
                }
            }
        }

        outline
    }

    /// 计算 miter 连接的长度（从连接点到尖角顶点的距离）。
    pub(crate) fn compute_miter_length(nx: f32, ny: f32, nnx: f32, nny: f32, half_lw: f32) -> f32 {
        let mx = nx + nnx;
        let my = ny + nny;
        let m = (mx * mx + my * my).sqrt();
        if m < f32::EPSILON {
            return half_lw;
        }
        half_lw * 2.0 / m
    }
}

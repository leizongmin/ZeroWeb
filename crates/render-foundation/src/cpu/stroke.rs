//! 渲染线段和路径图元 — StrokePrimitive、PathFillPrimitive、PathStrokePrimitive。

use crate::color::Color;
use crate::primitive::{LineCap, LineStyle, PathFillPrimitive, PathStrokePrimitive, StrokePrimitive};
use crate::surface::FrameBuffer;

/// 渲染线段图元到帧缓冲。
pub fn render_stroke(fb: &mut FrameBuffer, stroke: &StrokePrimitive, scale: f32) {
    let x1 = stroke.x1 * scale;
    let y1 = stroke.y1 * scale;
    let x2 = stroke.x2 * scale;
    let y2 = stroke.y2 * scale;
    let half_w = stroke.width * scale * 0.5;

    // R1909：防御非有限坐标（如 vertical-mode border 生成的 y2=inf）。
    // 非有限端点会使包围盒 clamp 异常，且 render_dotted_line 的 `while d <= total_len`
    //（total_len=inf）永不终止 → 渲染卡死（text-underline-position-001a >75s hang 根因）。
    // 跳过退化线段（源头几何 inf 是独立的 painter/vertical 度量 bug，此处仅防卡死）。
    if !x1.is_finite() || !y1.is_finite() || !x2.is_finite() || !y2.is_finite() {
        return;
    }

    // 计算包围盒
    let min_x = x1.min(x2) - half_w - 1.0;
    let min_y = y1.min(y2) - half_w - 1.0;
    let max_x = x1.max(x2) + half_w + 1.0;
    let max_y = y1.max(y2) + half_w + 1.0;

    let left = min_x.floor().max(0.0) as u32;
    let top = min_y.floor().max(0.0) as u32;
    let right = max_x.ceil().min(fb.width as f32) as u32;
    let bottom = max_y.ceil().min(fb.height as f32) as u32;

    if left >= right || top >= bottom {
        return;
    }

    // 线段方向向量
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len_sq = dx * dx + dy * dy;
    let len = len_sq.sqrt();

    // 单位法线
    let (nx, ny) = if len > 1e-10 {
        (-dy / len, dx / len)
    } else {
        (0.0, -1.0)
    };

    // 判断是否需要虚线/点线处理
    match stroke.style {
        LineStyle::Solid => {
            render_solid_line(
                fb,
                x1,
                y1,
                x2,
                y2,
                half_w,
                nx,
                ny,
                left,
                top,
                right,
                bottom,
                stroke.color,
                stroke.cap,
            );
        }
        LineStyle::Dashed => {
            if len > 1e-10 {
                render_dashed_line(
                    fb,
                    x1,
                    y1,
                    x2,
                    y2,
                    dx / len,
                    dy / len,
                    half_w,
                    nx,
                    ny,
                    left,
                    top,
                    right,
                    bottom,
                    stroke.color,
                    stroke.cap,
                    // R795：dash/gap 相对 border 厚度（chromium 实测：8px 边框 dash=16/gap=8，
                    // 即 dash=2×width、gap=1×width）。原 6.0*scale/4.0*scale 是固定 6:4 与
                    // 厚度无关，致 dashed 边框与 chromium 发散（root-element 等）。stroke.width*scale
                    // = 像素边框厚度（=2×half_w）。
                    2.0 * stroke.width * scale,
                    stroke.width * scale,
                );
            }
        }
        LineStyle::Dotted => {
            if len > 1e-10 {
                render_dotted_line(
                    fb,
                    x1,
                    y1,
                    x2,
                    y2,
                    dx / len,
                    dy / len,
                    half_w,
                    left,
                    top,
                    right,
                    bottom,
                    stroke.color,
                    // R796：dot 间距（圆心到圆心）= 2× border 厚度（chromium 实测：8px 边框
                    // 圆点直径=8、gap=8、圆心距=16=2×width）。原 2.0*scale 固定 2px 致圆点
                    //（半径 half_w=width/2）严重重叠成实线。dot_radius=half_w 已正确（=width/2）。
                    2.0 * stroke.width * scale,
                );
            }
        }
    }
}

/// 渲染实线。
#[allow(clippy::too_many_arguments)]
fn render_solid_line(
    fb: &mut FrameBuffer,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    half_w: f32,
    _nx: f32,
    _ny: f32,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
    color: Color,
    cap: LineCap,
) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len_sq = dx * dx + dy * dy;
    let len = len_sq.sqrt();

    for y in top..bottom {
        for x in left..right {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;

            // 计算到线段的距离
            let px = fx - x1;
            let py = fy - y1;

            let dist = if len_sq < 1e-10 {
                (px * px + py * py).sqrt()
            } else {
                let proj = (px * dx + py * dy) / len_sq;
                let proj = proj.clamp(0.0, 1.0);
                let closest_x = x1 + proj * dx;
                let closest_y = y1 + proj * dy;
                let ddx = fx - closest_x;
                let ddy = fy - closest_y;
                (ddx * ddx + ddy * ddy).sqrt()
            };

            if dist <= half_w {
                fb.set_pixel(x, y, [color.r, color.g, color.b, 255]);
            } else if cap != LineCap::Butt {
                // 检查端点帽
                let cap_extra = match cap {
                    LineCap::Round => 0.0,
                    LineCap::Square => half_w,
                    LineCap::Butt => continue,
                };
                let cap_r = half_w + cap_extra;

                // 到起点/终点的距离
                let d_start = ((fx - x1) * (fx - x1) + (fy - y1) * (fy - y1)).sqrt();
                let d_end = ((fx - x2) * (fx - x2) + (fy - y2) * (fy - y2)).sqrt();

                // 只在端点附近才检查
                if d_start <= cap_r || d_end <= cap_r {
                    // 检查是否在线段方向范围内
                    let proj = if len > 1e-10 { (px * dx + py * dy) / len_sq } else { 0.0 };
                    if (proj < 0.0 && d_start <= cap_r) || (proj > 1.0 && d_end <= cap_r) {
                        fb.set_pixel(x, y, [color.r, color.g, color.b, 255]);
                    }
                }
            }
        }
    }
}

/// 渲染虚线。
#[allow(clippy::too_many_arguments)]
fn render_dashed_line(
    fb: &mut FrameBuffer,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    dir_x: f32,
    dir_y: f32,
    half_w: f32,
    nx: f32,
    ny: f32,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
    color: Color,
    _cap: LineCap,
    dash_len: f32,
    gap_len: f32,
) {
    let total_len = ((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)).sqrt();
    let pattern_len = dash_len + gap_len;

    for y in top..bottom {
        for x in left..right {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;

            // 沿线方向的投影
            let px = fx - x1;
            let py = fy - y1;
            let proj = px * dir_x + py * dir_y;

            // 到线段的垂直距离
            let perp_dist = (px * nx.abs() + py * ny.abs()).abs();

            if perp_dist > half_w {
                continue;
            }

            // 检查是否在虚线段上
            if proj >= 0.0 && proj <= total_len {
                let pos_in_pattern = proj % pattern_len;
                if pos_in_pattern <= dash_len {
                    fb.set_pixel(x, y, [color.r, color.g, color.b, 255]);
                }
            }
        }
    }
}

/// 渲染点线。
#[allow(clippy::too_many_arguments)]
fn render_dotted_line(
    fb: &mut FrameBuffer,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    dir_x: f32,
    dir_y: f32,
    half_w: f32,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
    color: Color,
    dot_spacing: f32,
) {
    let total_len = ((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)).sqrt();
    let dot_radius = half_w;

    // R1909：dot_spacing 非正或非有限（width=0 退化时 = 0）会使 d 永不增长 → while 死循环。
    // 端点有限性已由 render_stroke 入口守卫保证，故 total_len 此处恒有限，无需再 cap
    //（cap 到 bbox 对角线会漏绘「起点在 fb 外」的点线可见段，引入回归）。
    if !dot_spacing.is_finite() || dot_spacing <= 0.0 {
        return;
    }

    // 沿线段方向逐点放置圆点
    let mut d = 0.0;
    while d <= total_len {
        let cx = x1 + dir_x * d;
        let cy = y1 + dir_y * d;

        // 渲染圆点
        let dot_left = (cx - dot_radius - 1.0).floor().max(left as f32) as u32;
        let dot_top = (cy - dot_radius - 1.0).floor().max(top as f32) as u32;
        let dot_right = (cx + dot_radius + 1.0).ceil().min(right as f32) as u32;
        let dot_bottom = (cy + dot_radius + 1.0).ceil().min(bottom as f32) as u32;

        for y in dot_top..dot_bottom {
            for x in dot_left..dot_right {
                let fx = x as f32 + 0.5;
                let fy = y as f32 + 0.5;
                let dx = fx - cx;
                let dy = fy - cy;
                if dx * dx + dy * dy <= dot_radius * dot_radius {
                    fb.set_pixel(x, y, [color.r, color.g, color.b, 255]);
                }
            }
        }

        d += dot_spacing;
    }
}

/// 渲染路径填充 — 使用扫描线算法填充多边形。
pub fn render_path_fill(fb: &mut FrameBuffer, path: &PathFillPrimitive, scale: f32) {
    let vertices = &path.vertices;
    if vertices.len() < 6 {
        return; // 至少需要 3 个顶点（6 个 f32）
    }

    let n = vertices.len() / 2;
    let mut scaled: Vec<(f32, f32)> = Vec::with_capacity(n);
    for i in 0..n {
        scaled.push((vertices[i * 2] * scale, vertices[i * 2 + 1] * scale));
    }

    // 计算包围盒
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for &(x, y) in &scaled {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    let left = min_x.floor().max(0.0) as u32;
    let top = min_y.floor().max(0.0) as u32;
    let right = max_x.ceil().min(fb.width as f32) as u32;
    let bottom = max_y.ceil().min(fb.height as f32) as u32;

    if left >= right || top >= bottom {
        return;
    }

    let color = path.color;

    // 扫描线填充 — even-odd 规则
    for y in top..bottom {
        let fy = y as f32 + 0.5;
        let mut intersections: Vec<f32> = Vec::new();

        // 计算所有边与扫描线的交点
        for i in 0..n {
            let j = (i + 1) % n;
            let (x0, y0) = scaled[i];
            let (x1, y1) = scaled[j];

            if (y0 <= fy && y1 > fy) || (y1 <= fy && y0 > fy) {
                let t = (fy - y0) / (y1 - y0);
                intersections.push(x0 + t * (x1 - x0));
            }
        }

        // 排序交点
        intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // 填充交点对之间的像素
        let mut idx = 0;
        while idx + 1 < intersections.len() {
            let fill_left = intersections[idx].floor().max(left as f32) as u32;
            let fill_right = intersections[idx + 1].ceil().min(right as f32) as u32;

            for x in fill_left..fill_right {
                fb.set_pixel(x, y, [color.r, color.g, color.b, 255]);
            }
            idx += 2;
        }
    }
}

/// 渲染路径描边。
pub fn render_path_stroke(fb: &mut FrameBuffer, path: &PathStrokePrimitive, scale: f32) {
    let vertices = &path.vertices;
    if vertices.len() < 4 {
        return; // 至少需要 2 个顶点（4 个 f32）
    }

    let n = vertices.len() / 2;
    let mut scaled: Vec<(f32, f32)> = Vec::with_capacity(n);
    for i in 0..n {
        scaled.push((vertices[i * 2] * scale, vertices[i * 2 + 1] * scale));
    }

    let half_w = path.line_width * scale * 0.5;
    let color = path.color;

    // 将路径描边分解为线段
    for i in 0..n - 1 {
        let (x1, y1) = scaled[i];
        let (x2, y2) = scaled[i + 1];
        render_thick_line(fb, x1, y1, x2, y2, half_w, color);
    }

    // 如果闭合路径，连接首尾
    if path.closed && n > 2 {
        let (x1, y1) = scaled[n - 1];
        let (x2, y2) = scaled[0];
        render_thick_line(fb, x1, y1, x2, y2, half_w, color);
    }
}

/// 渲染有宽度的线段（粗线段）。
fn render_thick_line(fb: &mut FrameBuffer, x1: f32, y1: f32, x2: f32, y2: f32, half_w: f32, color: Color) {
    let min_x = x1.min(x2) - half_w - 1.0;
    let min_y = y1.min(y2) - half_w - 1.0;
    let max_x = x1.max(x2) + half_w + 1.0;
    let max_y = y1.max(y2) + half_w + 1.0;

    let left = min_x.floor().max(0.0) as u32;
    let top = min_y.floor().max(0.0) as u32;
    let right = max_x.ceil().min(fb.width as f32) as u32;
    let bottom = max_y.ceil().min(fb.height as f32) as u32;

    if left >= right || top >= bottom {
        return;
    }

    let dx = x2 - x1;
    let dy = y2 - y1;
    let len_sq = dx * dx + dy * dy;

    for y in top..bottom {
        for x in left..right {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;

            let dist = if len_sq < 1e-10 {
                let ddx = fx - x1;
                let ddy = fy - y1;
                (ddx * ddx + ddy * ddy).sqrt()
            } else {
                let proj = ((fx - x1) * dx + (fy - y1) * dy) / len_sq;
                let proj = proj.clamp(0.0, 1.0);
                let closest_x = x1 + proj * dx;
                let closest_y = y1 + proj * dy;
                let ddx = fx - closest_x;
                let ddy = fy - closest_y;
                (ddx * ddx + ddy * ddy).sqrt()
            };

            if dist <= half_w {
                fb.set_pixel(x, y, [color.r, color.g, color.b, 255]);
            }
        }
    }
}

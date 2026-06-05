//! 标签栏 Chrome 风格形状与 loading 指示器。

use zero_render_foundation::color::Color;
use zero_render_foundation::primitive::FillPrimitive;

use crate::layout;

/// 绘制非激活标签（仅顶部圆角）。
pub fn push_inactive_tab_fill(fills: &mut Vec<FillPrimitive>, x: f32, w: f32, h: f32, scale: f32, color: Color) {
    push_top_rounded_rect_fill(fills, x, 0.0, w, h, layout::TAB_TOP_RADIUS * scale, color);
}

/// 绘制 Chrome 风格激活标签（与标签栏同高；底角二次曲线过渡到水平底边）。
pub fn push_active_tab_fill(fills: &mut Vec<FillPrimitive>, x: f32, w: f32, h: f32, scale: f32, color: Color) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r_top = (layout::TAB_TOP_RADIUS * scale).min(w * 0.5).min(h);
    let r_foot = (layout::TAB_FOOT_RADIUS * scale).min(w * 0.5).min(h * 0.5);
    let bottom_y = h;
    let foot_top = bottom_y - r_foot;
    let r_top_sq = r_top * r_top;

    let min_y = 0;
    let max_y = bottom_y.ceil() as i32;

    for row in min_y..max_y {
        let yf = row as f32 + 0.5;
        if yf >= bottom_y {
            continue;
        }

        let mut x_start = x;
        let mut x_end = x + w;

        if yf < r_top && r_top > f32::EPSILON {
            let dy = r_top - yf;
            let dx = (r_top_sq - dy * dy).max(0.0).sqrt();
            x_start = x + r_top - dx;
            x_end = x + w - r_top + dx;
        }

        if yf >= foot_top && r_foot > f32::EPSILON {
            // 扫描线中心在 bottom_y - 0.5，将 progress 归一化到该行为 1.0
            let foot_span = (bottom_y - foot_top - 0.5).max(f32::EPSILON);
            let progress = ((yf - foot_top) / foot_span).clamp(0.0, 1.0);
            let foot_extend = r_foot * progress * progress;
            x_start -= foot_extend;
            x_end += foot_extend;
        }

        if x_end > x_start {
            fills.push(rect_fill(x_start, row as f32, x_end - x_start, 1.0, color));
        }
    }
}

/// 标签 loading 旋转环（`angle` 为弧度）。
pub fn push_loading_spinner(fills: &mut Vec<FillPrimitive>, cx: f32, cy: f32, size: f32, angle: f32, color: Color) {
    let stroke = (1.4 * size / 16.0).max(1.0);
    let radius = size * 0.38;
    let span = std::f32::consts::TAU * 0.72;
    let steps = 28;
    for i in 0..steps {
        let t0 = i as f32 / steps as f32;
        let t1 = (i + 1) as f32 / steps as f32;
        let a0 = angle + span * t0;
        let a1 = angle + span * t1;
        push_stroke_segment(
            fills,
            cx + radius * a0.cos(),
            cy + radius * a0.sin(),
            cx + radius * a1.cos(),
            cy + radius * a1.sin(),
            stroke,
            color,
        );
    }
}

fn push_top_rounded_rect_fill(
    fills: &mut Vec<FillPrimitive>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    color: Color,
) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = radius.min(w * 0.5).min(h);
    if r <= f32::EPSILON {
        fills.push(rect_fill(x, y, w, h, color));
        return;
    }

    let min_y = y.floor() as i32;
    let max_y = (y + h).ceil() as i32;
    let r_sq = r * r;

    for row in min_y..max_y {
        let yf = row as f32 + 0.5;
        if yf >= y + h {
            break;
        }
        let mut x_start = x;
        let mut x_end = x + w;
        if yf < y + r {
            let dy = (y + r) - yf;
            let dx = (r_sq - dy * dy).max(0.0).sqrt();
            x_start = x + r - dx;
            x_end = x + w - r + dx;
        }
        if x_end > x_start {
            fills.push(rect_fill(x_start, row as f32, x_end - x_start, 1.0, color));
        }
    }
}

fn push_stroke_segment(
    fills: &mut Vec<FillPrimitive>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    thickness: f32,
    color: Color,
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    if len < f32::EPSILON {
        return;
    }
    let nx = -dy / len * thickness * 0.5;
    let ny = dx / len * thickness * 0.5;
    let ax = x0 + nx;
    let ay = y0 + ny;
    let bx = x1 + nx;
    let by = y1 + ny;
    let cxp = x1 - nx;
    let cyp = y1 - ny;
    let dxp = x0 - nx;
    let dyp = y0 - ny;

    let min_y = ay.min(by).min(cyp).min(dyp).floor() as i32;
    let max_y = ay.max(by).max(cyp).max(dyp).ceil() as i32;

    for row in min_y..=max_y {
        let yf = row as f32 + 0.5;
        let mut xs = Vec::new();
        for (x1e, y1e, x2e, y2e) in [
            (ax, ay, bx, by),
            (bx, by, cxp, cyp),
            (cxp, cyp, dxp, dyp),
            (dxp, dyp, ax, ay),
        ] {
            if let Some(x) = edge_x_at_y(x1e, y1e, x2e, y2e, yf) {
                xs.push(x);
            }
        }
        if xs.len() < 2 {
            continue;
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let x_start = xs[0];
        let x_end = xs[xs.len() - 1];
        if x_end > x_start {
            fills.push(rect_fill(x_start, row as f32, x_end - x_start, 1.0, color));
        }
    }
}

fn edge_x_at_y(x0: f32, y0: f32, x1: f32, y1: f32, y: f32) -> Option<f32> {
    if (y0 - y).abs() < 0.001 {
        return Some(x0);
    }
    if (y1 - y).abs() < 0.001 {
        return Some(x1);
    }
    if (y0 - y) * (y1 - y) > 0.0 {
        return None;
    }
    if (y1 - y0).abs() < f32::EPSILON {
        return None;
    }
    Some(x0 + (y - y0) * (x1 - x0) / (y1 - y0))
}

fn rect_fill(x: f32, y: f32, w: f32, h: f32, color: Color) -> FillPrimitive {
    FillPrimitive {
        rect: zero_render_foundation::geometry::Rect::new(x, y, w, h),
        color,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_render_foundation::color::Color;

    fn test_color() -> Color {
        Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    fn max_fill_bottom(fills: &[FillPrimitive]) -> f32 {
        fills
            .iter()
            .map(|f| f.rect.origin.y + f.rect.size.height)
            .fold(0.0, f32::max)
    }

    fn chrome_active_tab_left_edge_at_y(x: f32, bottom_y: f32, r_foot: f32, y: f32) -> f32 {
        if r_foot <= f32::EPSILON || y < bottom_y - r_foot {
            return x;
        }
        let foot_top = bottom_y - r_foot;
        let foot_span = (bottom_y - foot_top - 0.5).max(f32::EPSILON);
        let progress = ((y - foot_top) / foot_span).clamp(0.0, 1.0);
        x - r_foot * progress * progress
    }

    #[test]
    fn active_tab_stays_within_tab_bar_height() {
        let mut fills = Vec::new();
        let h = 36.0;
        push_active_tab_fill(&mut fills, 0.0, 200.0, h, 1.0, test_color());
        assert!(
            max_fill_bottom(&fills) <= h + 0.1,
            "active tab must not extend below tab bar, got {}",
            max_fill_bottom(&fills)
        );
    }

    #[test]
    fn active_tab_bottom_feet_reach_outside_corners() {
        let w = 200.0;
        let h = 36.0;
        let r = layout::TAB_FOOT_RADIUS;
        let left = chrome_active_tab_left_edge_at_y(0.0, h, r, h - 0.5);
        assert!(
            (left + r).abs() < 0.2,
            "bottom left foot should extend to x=-r, got {left}"
        );
    }

    #[test]
    fn active_tab_side_ends_before_bottom_for_concave_transition() {
        let h = 36.0;
        let r = layout::TAB_FOOT_RADIUS;
        let mid_foot_y = h - r * 0.5;
        let left = chrome_active_tab_left_edge_at_y(0.0, h, r, mid_foot_y);
        assert!(
            left > -r * 0.5,
            "foot curve should transition gradually, not bulge early, got {left}"
        );
        assert!(
            left < -0.1,
            "foot curve should extend slightly outside at mid height, got {left}"
        );
    }

    #[test]
    fn active_tab_feet_extend_outside_horizontally() {
        let mut fills = Vec::new();
        let x = 100.0;
        let w = 200.0;
        let h = 36.0;
        push_active_tab_fill(&mut fills, x, w, h, 1.0, test_color());
        assert!(fills.iter().any(|f| f.rect.origin.x < x - 0.1));
        assert!(fills.iter().any(|f| f.rect.origin.x + f.rect.size.width > x + w + 0.1));
    }
}

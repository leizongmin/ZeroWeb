//! 渲染渐变图元 — 线性、径向、锥形渐变。

use crate::color::Color;
use crate::primitive::{GradientKind, GradientPrimitive, GradientStop};
use crate::surface::FrameBuffer;

/// 渲染渐变图元到帧缓冲。
pub fn render_gradient(fb: &mut FrameBuffer, gradient: &GradientPrimitive, scale: f32) {
    if gradient.stops.is_empty() {
        return;
    }

    let left = (gradient.rect.left() * scale).floor().max(0.0) as u32;
    let top = (gradient.rect.top() * scale).floor().max(0.0) as u32;
    let right = (gradient.rect.right() * scale).ceil().min(fb.width as f32) as u32;
    let bottom = (gradient.rect.bottom() * scale).ceil().min(fb.height as f32) as u32;

    if left >= right || top >= bottom {
        return;
    }

    for y in top..bottom {
        for x in left..right {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;

            let mut t = compute_gradient_t(fx, fy, gradient, scale);
            if gradient.repeating {
                // 重复渐变：色标区间 [first, last] 为一个周期，t 折叠回该区间循环。
                // period = last - first（非 last 本身——否则 first≠0 时周期错）。
                // 折叠后直接用原 offset 采样（不再 / period：旧代码 t/=period 后用 [0,1) 采
                // [0,period] offset，致 t≥period 的像素被 sample_gradient_color 钳到最后色标，
                // repeating-linear(red 0,blue 25%) 等子域重复渲染为大片末色，比例错误）。
                // driving: css-images repeating-conic-gradient（black 0 25%, white 25% 50%）。
                let first = gradient.stops.first().map(|s| s.offset).unwrap_or(0.0);
                let last = gradient.stops.last().map(|s| s.offset).unwrap_or(1.0);
                let period = last - first;
                if period > 1e-6 {
                    let mut local = (t - first) % period;
                    if local < 0.0 {
                        local += period;
                    }
                    t = first + local;
                } else {
                    t = first;
                }
            } else {
                // 非重复：钳到 [0,1]（原 compute_gradient_t 的 clamp 语义，byte-identical）
                t = t.clamp(0.0, 1.0);
            }
            let color = sample_gradient_color(t, &gradient.stops, gradient.interpolation);
            let [r, g, b, _] = blend_with_fb(fb, x, y, color);
            fb.set_pixel(x, y, [r, g, b, 255]);
        }
    }
}

/// 计算像素在渐变中的位置参数 t ∈ [0, 1]。
fn compute_gradient_t(fx: f32, fy: f32, gradient: &GradientPrimitive, scale: f32) -> f32 {
    match &gradient.kind {
        GradientKind::Linear { x0, y0, x1, y1 } => {
            let sx0 = x0 * scale;
            let sy0 = y0 * scale;
            let sx1 = x1 * scale;
            let sy1 = y1 * scale;

            let dx = sx1 - sx0;
            let dy = sy1 - sy0;
            let len_sq = dx * dx + dy * dy;

            if len_sq < 1e-10 {
                return 0.0;
            }

            // 投影到渐变线上的参数 t（不钳——重复渐变需 t 超出 [0,1] 以折叠回周期；
            // 非重复的钳制在 render_gradient 调用方做，保持 byte-identical）
            ((fx - sx0) * dx + (fy - sy0) * dy) / len_sq
        }
        GradientKind::Radial {
            cx,
            cy,
            inner_radius,
            outer_radius,
        } => {
            let scx = cx * scale;
            let scy = cy * scale;
            let sir = inner_radius * scale;
            let sor = outer_radius * scale;

            let dx = fx - scx;
            let dy = fy - scy;
            let dist = (dx * dx + dy * dy).sqrt();

            let range = sor - sir;
            if range.abs() < 1e-10 {
                if dist <= sor { 1.0 } else { 0.0 }
            } else {
                // 不钳——重复径向需 t>1（outer 之外）以折叠回周期（非重复钳制在调用方）
                (dist - sir) / range
            }
        }
        GradientKind::Conic { cx, cy, start_angle } => {
            let scx = cx * scale;
            let scy = cy * scale;

            let dx = fx - scx;
            let dy = fy - scy;

            let angle = dy.atan2(dx);
            let mut t = (angle - start_angle) / (2.0 * std::f32::consts::PI);
            // 归一化到 [0, 1]
            t %= 1.0;
            if t < 0.0 {
                t += 1.0;
            }
            t
        }
    }
}

/// 根据位置参数 t 采样渐变颜色。
fn sample_gradient_color(
    t: f32,
    stops: &[GradientStop],
    interpolation: crate::primitive::GradientInterpolation,
) -> Color {
    if stops.is_empty() {
        return Color::TRANSPARENT;
    }
    if stops.len() == 1 {
        return stops[0].color;
    }

    // 找到 t 所在的两个色标之间
    if t <= stops[0].offset {
        return stops[0].color;
    }
    if t >= stops[stops.len() - 1].offset {
        return stops[stops.len() - 1].color;
    }

    for i in 0..stops.len() - 1 {
        let s0 = &stops[i];
        let s1 = &stops[i + 1];

        if t >= s0.offset && t <= s1.offset {
            let range = s1.offset - s0.offset;
            if range.abs() < 1e-10 {
                return s0.color;
            }
            let local_t = (t - s0.offset) / range;

            return crate::color_space::interp_pair(
                s0.color,
                s1.color,
                local_t as f64,
                interpolation.space,
                interpolation.hue,
            );
        }
    }

    stops[stops.len() - 1].color
}

/// 将渐变颜色与帧缓冲像素混合。
fn blend_with_fb(fb: &mut FrameBuffer, x: u32, y: u32, color: Color) -> [u8; 4] {
    let dst = fb.get_pixel(x, y);
    let src_a = color.a as f32 / 255.0;
    if src_a >= 1.0 {
        [color.r, color.g, color.b, 255]
    } else if src_a > 0.0 {
        let inv_a = 1.0 - src_a;
        [
            (color.r as f32 * src_a + dst[0] as f32 * inv_a).round() as u8,
            (color.g as f32 * src_a + dst[1] as f32 * inv_a).round() as u8,
            (color.b as f32 * src_a + dst[2] as f32 * inv_a).round() as u8,
            255,
        ]
    } else {
        dst
    }
}

//! 渲染阴影图元 — box-shadow 高斯模糊近似。

use crate::primitive::ShadowPrimitive;
use crate::surface::FrameBuffer;

/// 渲染 box-shadow。
///
/// 使用三遍 box-blur 近似高斯模糊：
/// 1. 计算阴影矩形（原始矩形 + spread + offset）
/// 2. 对阴影区域进行模糊
/// 3. 将模糊结果合成到帧缓冲
pub fn render_shadow(fb: &mut FrameBuffer, shadow: &ShadowPrimitive, scale: f32) {
    let blur_r = shadow.blur_radius * scale;
    let spread = shadow.spread_radius * scale;

    // 阴影矩形（扩展 + 偏移）
    let sx = shadow.rect.left() * scale - spread + shadow.offset_x * scale;
    let sy = shadow.rect.top() * scale - spread + shadow.offset_y * scale;
    let sw = shadow.rect.size.width * scale + spread * 2.0;
    let sh = shadow.rect.size.height * scale + spread * 2.0;

    if sw <= 0.0 || sh <= 0.0 {
        return;
    }

    // 计算需要渲染的区域（包含模糊扩展）
    let blur_extent = blur_r * 3.0; // 3σ 覆盖 99.7% 的高斯分布
    let area_left = (sx - blur_extent).floor().max(0.0) as u32;
    let area_top = (sy - blur_extent).floor().max(0.0) as u32;
    let area_right = (sx + sw + blur_extent).ceil().min(fb.width as f32) as u32;
    let area_bottom = (sy + sh + blur_extent).ceil().min(fb.height as f32) as u32;

    if area_left >= area_right || area_top >= area_bottom {
        return;
    }

    let area_w = (area_right - area_left) as usize;
    let area_h = (area_bottom - area_top) as usize;

    if area_w == 0 || area_h == 0 {
        return;
    }

    // 步骤 1：生成阴影 alpha 蒙版
    let mut alpha_mask = vec![0.0f32; area_w * area_h];

    let shadow_color = [shadow.color.r as f32, shadow.color.g as f32, shadow.color.b as f32];
    let shadow_alpha = shadow.color.a as f32 / 255.0;

    // 对阴影矩形区域内的像素设置初始 alpha
    for y in area_top..area_bottom {
        for x in area_left..area_right {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;

            if fx >= sx && fx <= sx + sw && fy >= sy && fy <= sy + sh {
                let idx = (y - area_top) as usize * area_w + (x - area_left) as usize;
                alpha_mask[idx] = shadow_alpha;
            }
        }
    }

    // 步骤 2：三遍 box-blur（近似高斯模糊）
    if blur_r > 0.5 {
        let radius = blur_r.ceil() as usize;
        if radius > 0 {
            box_blur(&mut alpha_mask, area_w, area_h, radius);
            box_blur(&mut alpha_mask, area_w, area_h, radius);
            box_blur(&mut alpha_mask, area_w, area_h, radius);
        }
    }

    // 步骤 3：合成到帧缓冲
    for y in area_top..area_bottom {
        for x in area_left..area_right {
            let idx = (y - area_top) as usize * area_w + (x - area_left) as usize;
            let alpha = alpha_mask[idx];

            if alpha < 0.001 {
                continue;
            }

            let dst = fb.get_pixel(x, y);
            let inv_a = 1.0 - alpha;
            let r = shadow_color[0] * alpha + dst[0] as f32 * inv_a;
            let g = shadow_color[1] * alpha + dst[1] as f32 * inv_a;
            let b = shadow_color[2] * alpha + dst[2] as f32 * inv_a;
            fb.set_pixel(x, y, [r.round() as u8, g.round() as u8, b.round() as u8, 255]);
        }
    }
}

/// 单遍 box-blur — 对 alpha 蒙版进行水平+垂直模糊。
fn box_blur(data: &mut [f32], width: usize, height: usize, radius: usize) {
    if width == 0 || height == 0 || radius == 0 {
        return;
    }

    let kernel_size = radius * 2 + 1;
    let inv_size = 1.0 / kernel_size as f32;

    // 水平模糊
    let mut temp = vec![0.0f32; data.len()];
    for y in 0..height {
        let mut sum = 0.0;
        // 初始化窗口
        for dx in 0..=radius.min(width - 1) {
            sum += data[y * width + dx];
        }
        // 如果左边界外有值，用边界值填充
        for _ in 0..radius.saturating_sub(0) {
            // 已在初始化中处理
        }

        for x in 0..width {
            let right = x + radius;
            let left = if x > radius { x - radius - 1 } else { 0 };

            if right < width {
                sum += data[y * width + right];
            }
            if x > radius {
                sum -= data[y * width + left];
            }

            temp[y * width + x] = sum * inv_size;
        }
    }

    // 垂直模糊
    for x in 0..width {
        let mut sum = 0.0;
        for dy in 0..=radius.min(height - 1) {
            sum += temp[dy * width + x];
        }

        for y in 0..height {
            let bottom = y + radius;
            let top = if y > radius { y - radius - 1 } else { 0 };

            if bottom < height {
                sum += temp[bottom * width + x];
            }
            if y > radius {
                sum -= temp[top * width + x];
            }

            data[y * width + x] = sum * inv_size;
        }
    }
}

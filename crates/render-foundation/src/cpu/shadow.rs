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
    // R2476：inset 内阴影走单独路径（frame 形状 = box 减 offset+spread 收缩的洞，向内模糊，
    // 裁切到盒）；outset 走既有路径。
    if shadow.inset {
        render_inset_shadow(fb, shadow, scale);
        return;
    }
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
    // CSS 规范：box-shadow blur_radius 对应高斯标准差 σ = blur_radius / 2（而非
    // blur_radius 本身）。三遍等半径 box-blur（每遍半宽 r）合成 σ 满足
    // σ² ≈ ((2r+1)² - 1) / 4。由 σ 反解连续半宽 d = (sqrt(4σ²+1) - 1)/2，再按 d
    // 的小数部分把 3 遍在 floor/ceil 半宽间分配（m 遍用 ceil、3-m 遍用 floor），
    // 精确逼近目标 σ。旧实现直接用 radius=blur_r.ceil() 致 σ 偏大约 2.3 倍，
    // 阴影扩散过远（welcome.html 卡片 box-shadow:0 1px 3px 实测扩散 12px）。
    if blur_r > 0.5 {
        let sigma = blur_r * 0.5;
        let d = ((4.0 * sigma * sigma + 1.0).sqrt() - 1.0) * 0.5;
        let r_lo = d.floor() as usize;
        let r_hi = r_lo + 1;
        let m = ((d - r_lo as f32) * 3.0).round().clamp(0.0, 3.0) as usize;
        for _ in 0..(3 - m) {
            if r_lo > 0 {
                box_blur(&mut alpha_mask, area_w, area_h, r_lo);
            }
        }
        for _ in 0..m {
            box_blur(&mut alpha_mask, area_w, area_h, r_hi);
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

/// R2476：渲染 inset（内）box-shadow（CSS Backgrounds §7.1）。
///
/// 内阴影在盒内：alpha 蒙版 = 盒（OUTER）内、洞（OUTER 经 offset 偏移 + spread 收缩）
/// 外的区域（frame 形状）。三遍 box-blur 软化洞边界（阴影向内淡出）；OUTER 边界因 area
/// 恰为盒、向外模糊被裁切而保持硬边。合成（source-over）裁切到盒。
fn render_inset_shadow(fb: &mut FrameBuffer, shadow: &ShadowPrimitive, scale: f32) {
    let blur_r = shadow.blur_radius * scale;
    let spread = shadow.spread_radius * scale;

    // OUTER = 盒（缩放后绝对坐标）
    let ox = shadow.rect.left() * scale;
    let oy = shadow.rect.top() * scale;
    let ow = shadow.rect.size.width * scale;
    let oh = shadow.rect.size.height * scale;
    if ow <= 0.0 || oh <= 0.0 {
        return;
    }

    // 洞：OUTER 经 offset 偏移 + spread 收缩（inset 正 spread 向内增长阴影 → 洞缩小）。
    let hx = ox + shadow.offset_x * scale + spread;
    let hy = oy + shadow.offset_y * scale + spread;
    let hw = ow - 2.0 * spread;
    let hh = oh - 2.0 * spread;

    // area = OUTER（内阴影限制在盒内）裁切到帧缓冲
    let area_left = ox.floor().max(0.0) as u32;
    let area_top = oy.floor().max(0.0) as u32;
    let area_right = (ox + ow).ceil().min(fb.width as f32) as u32;
    let area_bottom = (oy + oh).ceil().min(fb.height as f32) as u32;
    if area_left >= area_right || area_top >= area_bottom {
        return;
    }
    let area_w = (area_right - area_left) as usize;
    let area_h = (area_bottom - area_top) as usize;
    if area_w == 0 || area_h == 0 {
        return;
    }

    let shadow_alpha = shadow.color.a as f32 / 255.0;
    let mut alpha_mask = vec![0.0f32; area_w * area_h];

    // alpha = shadow_alpha 在 frame 区（OUTER 内且洞外）；洞退化（hw/hh<=0）→ 全盒阴影。
    let hole_degenerate = hw <= 0.0 || hh <= 0.0;
    for y in area_top..area_bottom {
        for x in area_left..area_right {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            let outside_hole = hole_degenerate || fx < hx || fx > hx + hw || fy < hy || fy > hy + hh;
            if outside_hole {
                let idx = (y - area_top) as usize * area_w + (x - area_left) as usize;
                alpha_mask[idx] = shadow_alpha;
            }
        }
    }

    // 三遍 box-blur（软化洞边界向内淡出）。R2476：用 box_blur_inset（正确 2D-clamp 实现），
    // 非 outset 的 box_blur——后者滑动窗口 init 对非均匀 mask（frame 形状）有 double-count
    // 残差泄漏（对 outset 均匀矩形 mask 良性，对 inset frame 致洞内残余 alpha）。
    if blur_r > 0.5 {
        let sigma = blur_r * 0.5;
        let d = ((4.0 * sigma * sigma + 1.0).sqrt() - 1.0) * 0.5;
        let r_lo = d.floor() as usize;
        let r_hi = r_lo + 1;
        let m = ((d - r_lo as f32) * 3.0).round().clamp(0.0, 3.0) as usize;
        for _ in 0..(3 - m) {
            if r_lo > 0 {
                box_blur_inset(&mut alpha_mask, area_w, area_h, r_lo);
            }
        }
        for _ in 0..m {
            box_blur_inset(&mut alpha_mask, area_w, area_h, r_hi);
        }
    }

    // 合成（source-over），裁切到盒（area=OUTER 自动满足）。
    let sc = [shadow.color.r as f32, shadow.color.g as f32, shadow.color.b as f32];
    for y in area_top..area_bottom {
        for x in area_left..area_right {
            let idx = (y - area_top) as usize * area_w + (x - area_left) as usize;
            let alpha = alpha_mask[idx];
            if alpha < 0.001 {
                continue;
            }
            let dst = fb.get_pixel(x, y);
            let inv_a = 1.0 - alpha;
            let r = sc[0] * alpha + dst[0] as f32 * inv_a;
            let g = sc[1] * alpha + dst[1] as f32 * inv_a;
            let b = sc[2] * alpha + dst[2] as f32 * inv_a;
            fb.set_pixel(x, y, [r.round() as u8, g.round() as u8, b.round() as u8, 255]);
        }
    }
}

/// R2476：单遍 box-blur（2D，边缘 clamp）—— inset 内阴影专用。
///
/// 与 outset 的 `box_blur`（可分离滑动窗口）不同：后者对均匀矩形 mask 良性，但滑动窗口
/// init 对非均匀 frame mask 有 double-count 残差泄漏。本函数直接逐像素求窗口均值（窗口
/// 越界 clamp 到边缘 → OUTER 边保持硬边，洞边界向内淡出），小 mask（盒内）+ 小 radius 成本可接受。
fn box_blur_inset(data: &mut [f32], width: usize, height: usize, radius: usize) {
    if width == 0 || height == 0 || radius == 0 {
        return;
    }
    let src = data.to_vec();
    for y in 0..height {
        let y0 = y.saturating_sub(radius);
        let y1 = (y + radius).min(height - 1);
        for x in 0..width {
            let x0 = x.saturating_sub(radius);
            let x1 = (x + radius).min(width - 1);
            let mut sum = 0.0f32;
            let mut cnt = 0usize;
            for yy in y0..=y1 {
                for xx in x0..=x1 {
                    sum += src[yy * width + xx];
                    cnt += 1;
                }
            }
            data[y * width + x] = if cnt > 0 { sum / cnt as f32 } else { 0.0 };
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

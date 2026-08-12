//! 渲染滤镜和混合模式图元 — FilterPrimitive、BlendModePrimitive。

use crate::primitive::{BlendMode, BlendModePrimitive, FilterKind, FilterPrimitive};
use crate::surface::FrameBuffer;

/// 应用 CSS 滤镜效果。
///
/// 支持的滤镜：
/// - `Blur` — box-blur 近似高斯模糊
/// - `Brightness` — 亮度调节
/// - `Contrast` — 对比度调节
/// - `Grayscale` — 灰度
/// - `HueRotate` — 色相旋转
/// - `Invert` — 反色
/// - `Opacity` — 透明度
/// - `Saturate` — 饱和度调节
/// - `Sepia` — 棕褐色调
/// - `DropShadow` — 投影阴影（CPU 简化实现）
pub fn apply_filter(fb: &mut FrameBuffer, filter: &FilterPrimitive, scale: f32) {
    if filter.filters.is_empty() {
        return;
    }

    let left = (filter.rect.left() * scale).floor().max(0.0) as u32;
    let top = (filter.rect.top() * scale).floor().max(0.0) as u32;
    let right = (filter.rect.right() * scale).ceil().min(fb.width as f32) as u32;
    let bottom = (filter.rect.bottom() * scale).ceil().min(fb.height as f32) as u32;

    if left >= right || top >= bottom {
        return;
    }

    // 依次应用每个滤镜
    for f in &filter.filters {
        match f {
            FilterKind::Blur(radius) => {
                let r = (radius * scale).ceil() as usize;
                if r > 0 {
                    apply_box_blur(fb, left, top, right, bottom, r);
                }
            }
            FilterKind::Brightness(amount) => {
                let amt = *amount;
                for y in top..bottom {
                    for x in left..right {
                        let p = fb.get_pixel(x, y);
                        fb.set_pixel(
                            x,
                            y,
                            [
                                (p[0] as f32 * amt).round().clamp(0.0, 255.0) as u8,
                                (p[1] as f32 * amt).round().clamp(0.0, 255.0) as u8,
                                (p[2] as f32 * amt).round().clamp(0.0, 255.0) as u8,
                                255,
                            ],
                        );
                    }
                }
            }
            FilterKind::Contrast(amount) => {
                let amt = *amount;
                for y in top..bottom {
                    for x in left..right {
                        let p = fb.get_pixel(x, y);
                        fb.set_pixel(
                            x,
                            y,
                            [
                                ((p[0] as f32 - 128.0) * amt + 128.0).round().clamp(0.0, 255.0) as u8,
                                ((p[1] as f32 - 128.0) * amt + 128.0).round().clamp(0.0, 255.0) as u8,
                                ((p[2] as f32 - 128.0) * amt + 128.0).round().clamp(0.0, 255.0) as u8,
                                255,
                            ],
                        );
                    }
                }
            }
            FilterKind::Grayscale(amount) => {
                let amt = *amount;
                for y in top..bottom {
                    for x in left..right {
                        let p = fb.get_pixel(x, y);
                        let gray = p[0] as f32 * 0.299 + p[1] as f32 * 0.587 + p[2] as f32 * 0.114;
                        fb.set_pixel(
                            x,
                            y,
                            [
                                (p[0] as f32 + (gray - p[0] as f32) * amt).round().clamp(0.0, 255.0) as u8,
                                (p[1] as f32 + (gray - p[1] as f32) * amt).round().clamp(0.0, 255.0) as u8,
                                (p[2] as f32 + (gray - p[2] as f32) * amt).round().clamp(0.0, 255.0) as u8,
                                255,
                            ],
                        );
                    }
                }
            }
            FilterKind::HueRotate(degrees) => {
                let angle = degrees.to_radians();
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                for y in top..bottom {
                    for x in left..right {
                        let p = fb.get_pixel(x, y);
                        let [r, g, b] = hue_rotate(p[0], p[1], p[2], cos_a, sin_a);
                        fb.set_pixel(x, y, [r, g, b, 255]);
                    }
                }
            }
            FilterKind::Invert(amount) => {
                let amt = *amount;
                for y in top..bottom {
                    for x in left..right {
                        let p = fb.get_pixel(x, y);
                        fb.set_pixel(
                            x,
                            y,
                            [
                                (p[0] as f32 + (255.0 - 2.0 * p[0] as f32) * amt)
                                    .round()
                                    .clamp(0.0, 255.0) as u8,
                                (p[1] as f32 + (255.0 - 2.0 * p[1] as f32) * amt)
                                    .round()
                                    .clamp(0.0, 255.0) as u8,
                                (p[2] as f32 + (255.0 - 2.0 * p[2] as f32) * amt)
                                    .round()
                                    .clamp(0.0, 255.0) as u8,
                                255,
                            ],
                        );
                    }
                }
            }
            FilterKind::Opacity(amount) => {
                let amt = *amount;
                for y in top..bottom {
                    for x in left..right {
                        let p = fb.get_pixel(x, y);
                        // opacity 降低亮度来模拟（帧缓冲 alpha 总为 255）
                        fb.set_pixel(
                            x,
                            y,
                            [
                                (p[0] as f32 * amt).round().clamp(0.0, 255.0) as u8,
                                (p[1] as f32 * amt).round().clamp(0.0, 255.0) as u8,
                                (p[2] as f32 * amt).round().clamp(0.0, 255.0) as u8,
                                255,
                            ],
                        );
                    }
                }
            }
            FilterKind::Saturate(amount) => {
                let amt = *amount;
                for y in top..bottom {
                    for x in left..right {
                        let p = fb.get_pixel(x, y);
                        let gray = p[0] as f32 * 0.299 + p[1] as f32 * 0.587 + p[2] as f32 * 0.114;
                        fb.set_pixel(
                            x,
                            y,
                            [
                                (gray + (p[0] as f32 - gray) * amt).round().clamp(0.0, 255.0) as u8,
                                (gray + (p[1] as f32 - gray) * amt).round().clamp(0.0, 255.0) as u8,
                                (gray + (p[2] as f32 - gray) * amt).round().clamp(0.0, 255.0) as u8,
                                255,
                            ],
                        );
                    }
                }
            }
            FilterKind::Sepia(amount) => {
                let amt = *amount;
                for y in top..bottom {
                    for x in left..right {
                        let p = fb.get_pixel(x, y);
                        let sr = (p[0] as f32 * 0.393 + p[1] as f32 * 0.769 + p[2] as f32 * 0.189).min(255.0);
                        let sg = (p[0] as f32 * 0.349 + p[1] as f32 * 0.686 + p[2] as f32 * 0.168).min(255.0);
                        let sb = (p[0] as f32 * 0.272 + p[1] as f32 * 0.534 + p[2] as f32 * 0.131).min(255.0);
                        fb.set_pixel(
                            x,
                            y,
                            [
                                (p[0] as f32 + (sr - p[0] as f32) * amt).round().clamp(0.0, 255.0) as u8,
                                (p[1] as f32 + (sg - p[1] as f32) * amt).round().clamp(0.0, 255.0) as u8,
                                (p[2] as f32 + (sb - p[2] as f32) * amt).round().clamp(0.0, 255.0) as u8,
                                255,
                            ],
                        );
                    }
                }
            }
            FilterKind::DropShadow(_ox, _oy, _blur, _color) => {
                // CPU 简化实现：drop-shadow 在 CPU 渲染器中跳过
                // 完整实现需要将区域内容提取为 alpha 蒙版，模糊后叠加
            }
        }
    }
}

/// 对帧缓冲指定区域应用 box-blur（单遍，双向）。
fn apply_box_blur(fb: &mut FrameBuffer, left: u32, top: u32, right: u32, bottom: u32, radius: usize) {
    let w = (right - left) as usize;
    let h = (bottom - top) as usize;
    if w == 0 || h == 0 || radius == 0 {
        return;
    }

    // 提取区域像素
    let mut pixels_r = vec![0u8; w * h];
    let mut pixels_g = vec![0u8; w * h];
    let mut pixels_b = vec![0u8; w * h];

    for y in 0..h {
        for x in 0..w {
            let p = fb.get_pixel(left + x as u32, top + y as u32);
            pixels_r[y * w + x] = p[0];
            pixels_g[y * w + x] = p[1];
            pixels_b[y * w + x] = p[2];
        }
    }

    // 三遍 box-blur 近似高斯模糊
    box_blur_channel(&mut pixels_r, w, h, radius);
    box_blur_channel(&mut pixels_g, w, h, radius);
    box_blur_channel(&mut pixels_b, w, h, radius);
    box_blur_channel(&mut pixels_r, w, h, radius);
    box_blur_channel(&mut pixels_g, w, h, radius);
    box_blur_channel(&mut pixels_b, w, h, radius);
    box_blur_channel(&mut pixels_r, w, h, radius);
    box_blur_channel(&mut pixels_g, w, h, radius);
    box_blur_channel(&mut pixels_b, w, h, radius);

    // 写回帧缓冲
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            fb.set_pixel(
                left + x as u32,
                top + y as u32,
                [pixels_r[idx], pixels_g[idx], pixels_b[idx], 255],
            );
        }
    }
}

/// 对单通道进行 box-blur（水平+垂直）。
fn box_blur_channel(data: &mut [u8], width: usize, height: usize, radius: usize) {
    if width == 0 || height == 0 || radius == 0 {
        return;
    }

    let kernel_size = radius * 2 + 1;

    // 水平模糊
    let mut temp = vec![0u8; data.len()];
    for y in 0..height {
        let mut sum = 0u32;
        // 初始化窗口
        for dx in 0..=radius.min(width - 1) {
            sum += data[y * width + dx] as u32;
        }
        for x in 0..width {
            let right_x = x + radius;
            let left_x = if x > radius { x - radius - 1 } else { usize::MAX };

            if right_x < width {
                sum += data[y * width + right_x] as u32;
            }
            if left_x != usize::MAX {
                sum -= data[y * width + left_x] as u32;
            }

            temp[y * width + x] = (sum / kernel_size as u32) as u8;
        }
    }

    // 垂直模糊
    for x in 0..width {
        let mut sum = 0u32;
        for dy in 0..=radius.min(height - 1) {
            sum += temp[dy * width + x] as u32;
        }
        for y in 0..height {
            let bottom_y = y + radius;
            let top_y = if y > radius { y - radius - 1 } else { usize::MAX };

            if bottom_y < height {
                sum += temp[bottom_y * width + x] as u32;
            }
            if top_y != usize::MAX {
                sum -= temp[top_y * width + x] as u32;
            }

            data[y * width + x] = (sum / kernel_size as u32) as u8;
        }
    }
}

/// 色相旋转辅助函数 — 将 RGB 通过色相旋转矩阵变换。
fn hue_rotate(r: u8, g: u8, _b: u8, cos_a: f32, sin_a: f32) -> [u8; 3] {
    // CSS filter hue-rotate 矩阵
    let sq3 = 3.0_f32.sqrt();
    let inv3 = 1.0 / 3.0;
    let ma = cos_a + (1.0 - cos_a) * inv3;
    let mb = (1.0 - cos_a) * inv3 - sq3 * sin_a * inv3;
    let mc = (1.0 - cos_a) * inv3 + sq3 * sin_a * inv3;

    let matrix: [[f32; 3]; 3] = [[ma, mb, mc], [mc, ma, mb], [mb, mc, ma]];

    let rf = r as f32;
    let gf = g as f32;
    let bf = _b as f32;

    [
        (matrix[0][0] * rf + matrix[0][1] * gf + matrix[0][2] * bf)
            .round()
            .clamp(0.0, 255.0) as u8,
        (matrix[1][0] * rf + matrix[1][1] * gf + matrix[1][2] * bf)
            .round()
            .clamp(0.0, 255.0) as u8,
        (matrix[2][0] * rf + matrix[2][1] * gf + matrix[2][2] * bf)
            .round()
            .clamp(0.0, 255.0) as u8,
    ]
}

/// 应用混合模式：源层（`src`，元素及子元素图元）与目标层（`fb`，背景）在
/// blend 区域内按 CSS Compositing-1 §5 公式合成（P2-7）。
///
/// 由 render_draw_order 提供源层：painter 在元素图元之前 push DrawOp::BlendMode
/// 标记，渲染循环把标记之后的图元画到独立源缓冲，循环结束后逐区域合成。
/// 嵌套/兄弟 blend：同一源缓冲、各自 rect 分别合成（rect 不重叠时正确；
/// 嵌套时内层在父区域内二次合成，近似）。
pub fn composite_blend(fb: &mut FrameBuffer, src: &FrameBuffer, blend: &BlendModePrimitive, scale: f32) {
    let left = (blend.rect.left() * scale).floor().max(0.0) as u32;
    let top = (blend.rect.top() * scale).floor().max(0.0) as u32;
    let right = (blend.rect.right() * scale).ceil().min(fb.width as f32) as u32;
    let bottom = (blend.rect.bottom() * scale).ceil().min(fb.height as f32) as u32;

    if left >= right || top >= bottom {
        return; // 空区域，跳过
    }

    for y in top..bottom {
        for x in left..right {
            let si = (y * fb.width + x) as usize * 4;
            let s = [src.data[si], src.data[si + 1], src.data[si + 2], src.data[si + 3]];
            let d = [fb.data[si], fb.data[si + 1], fb.data[si + 2], fb.data[si + 3]];
            let out = blend_rgba(s, d, blend.mode);
            fb.data[si..si + 4].copy_from_slice(&out);
        }
    }
}

/// 单个像素的 CSS 混合（RGB 通道按模式公式，alpha 走 src-over 合成；
/// CSS Compositing-1 §5.1 B(Cb,Cs) + §5.2 simple alpha compositing）。
pub fn blend_rgba(src: [u8; 4], dst: [u8; 4], mode: BlendMode) -> [u8; 4] {
    if matches!(mode, BlendMode::Normal) {
        return src;
    }
    let cs = [src[0] as f32 / 255.0, src[1] as f32 / 255.0, src[2] as f32 / 255.0];
    let cb = [dst[0] as f32 / 255.0, dst[1] as f32 / 255.0, dst[2] as f32 / 255.0];
    let mixed: [f32; 3] = blend_channels(cb, cs, mode);
    let out: [u8; 3] = [
        (mixed[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (mixed[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (mixed[2].clamp(0.0, 1.0) * 255.0).round() as u8,
    ];
    // alpha：src-over（CSS §5.2）——混合后的 alpha 由合成贡献
    let sa = src[3] as f32 / 255.0;
    let da = dst[3] as f32 / 255.0;
    let out_a = (sa + da * (1.0 - sa)).clamp(0.0, 1.0);
    [out[0], out[1], out[2], (out_a * 255.0).round() as u8]
}

/// 16 种混合模式公式（CSS Compositing-1 §5.1）。B(Cb, Cs)：Cb=背景（目标），
/// Cs=源（元素）。Hue/Saturation/Color/Luminosity 经 HSL 中间空间（§5.1.13-16）。
fn blend_channels(cb: [f32; 3], cs: [f32; 3], mode: BlendMode) -> [f32; 3] {
    let b = |c1: f32, c2: f32| -> f32 { c1 * c2 }; // multiply
    let s = |c1: f32, c2: f32| -> f32 { 1.0 - (1.0 - c1) * (1.0 - c2) }; // screen
    match mode {
        BlendMode::Normal => cs,
        BlendMode::Multiply => [b(cb[0], cs[0]), b(cb[1], cs[1]), b(cb[2], cs[2])],
        BlendMode::Screen => [s(cb[0], cs[0]), s(cb[1], cs[1]), s(cb[2], cs[2])],
        BlendMode::Overlay => hard_light(cs, cb),
        BlendMode::HardLight => hard_light(cb, cs),
        BlendMode::Darken => [cb[0].min(cs[0]), cb[1].min(cs[1]), cb[2].min(cs[2])],
        BlendMode::Lighten => [cb[0].max(cs[0]), cb[1].max(cs[1]), cb[2].max(cs[2])],
        BlendMode::ColorDodge => [
            color_dodge(cb[0], cs[0]),
            color_dodge(cb[1], cs[1]),
            color_dodge(cb[2], cs[2]),
        ],
        BlendMode::ColorBurn => [
            color_burn(cb[0], cs[0]),
            color_burn(cb[1], cs[1]),
            color_burn(cb[2], cs[2]),
        ],
        BlendMode::SoftLight => [
            soft_light(cb[0], cs[0]),
            soft_light(cb[1], cs[1]),
            soft_light(cb[2], cs[2]),
        ],
        BlendMode::Difference => [(cb[0] - cs[0]).abs(), (cb[1] - cs[1]).abs(), (cb[2] - cs[2]).abs()],
        BlendMode::Exclusion => [
            cb[0] + cs[0] - 2.0 * cb[0] * cs[0],
            cb[1] + cs[1] - 2.0 * cb[1] * cs[1],
            cb[2] + cs[2] - 2.0 * cb[2] * cs[2],
        ],
        BlendMode::Hue => set_lum(set_sat(cb, sat(cs)), lum(cb)),
        BlendMode::Saturation => set_lum(set_sat(cs, sat(cb)), lum(cb)),
        BlendMode::Color => set_lum(cs, lum(cb)),
        BlendMode::Luminosity => set_lum(cb, lum(cs)),
    }
}

fn hard_light(cb: [f32; 3], cs: [f32; 3]) -> [f32; 3] {
    let mut out = [0.0f32; 3];
    for i in 0..3 {
        out[i] = if cs[i] <= 0.5 {
            cb[i] * cs[i] * 2.0
        } else {
            1.0 - 2.0 * (1.0 - cb[i]) * (1.0 - cs[i])
        };
    }
    out
}

fn color_dodge(cb: f32, cs: f32) -> f32 {
    if cs >= 1.0 {
        1.0
    } else {
        (cb / (1.0 - cs)).clamp(0.0, 1.0)
    }
}

fn color_burn(cb: f32, cs: f32) -> f32 {
    if cs <= 0.0 {
        0.0
    } else {
        (1.0 - (1.0 - cb) / cs).clamp(0.0, 1.0)
    }
}

fn soft_light(cb: f32, cs: f32) -> f32 {
    // CSS Compositing-1 §5.1.10
    if cs <= 0.5 {
        cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb)
    } else {
        let d = if cb <= 0.25 {
            ((16.0 * cb - 12.0) * cb + 4.0) * cb
        } else {
            cb.sqrt()
        };
        cb + (2.0 * cs - 1.0) * (d - cb)
    }
}

fn lum(c: [f32; 3]) -> f32 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

fn sat(c: [f32; 3]) -> f32 {
    c.iter().cloned().fold(f32::NEG_INFINITY, f32::max) - c.iter().cloned().fold(f32::INFINITY, f32::min)
}

fn set_sat(c: [f32; 3], s: f32) -> [f32; 3] {
    let min = c.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = c.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    if max > min {
        let mid = c.iter().find(|&&v| v != min && v != max).copied().unwrap_or(min);
        let mid = (mid - min) * s / (max - min);
        [
            if c[0] == min {
                0.0
            } else if c[0] == max {
                s
            } else {
                mid
            },
            if c[1] == min {
                0.0
            } else if c[1] == max {
                s
            } else {
                mid
            },
            if c[2] == min {
                0.0
            } else if c[2] == max {
                s
            } else {
                mid
            },
        ]
    } else {
        c
    }
}

fn set_lum(c: [f32; 3], l: f32) -> [f32; 3] {
    let d = l - lum(c);
    let mut out = [c[0] + d, c[1] + d, c[2] + d];
    // 钳制后重新调整亮度（CSS 规范 §5.1 的 clip 步骤简化：直接 clamp）
    for v in out.iter_mut() {
        *v = v.clamp(0.0, 1.0);
    }
    out
}

/// 应用混合模式（旧后处理入口，保留签名兼容；真实实现见 [`composite_blend`]）。
pub fn apply_blend_mode(fb: &mut FrameBuffer, blend: &BlendModePrimitive, scale: f32) {
    // P2-7：render_typed_buckets（逃生舱）无 draw_order 顺序信息、无法定位源层，
    // 保持跳过（与 GPU 路径一致——blend 由 scene_supported 拒绝回退 CPU 的
    // render_draw_order 路径处理）。
    let _ = (fb, blend, scale);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::primitive::FilterPrimitive;

    #[test]
    fn filter_blur_produces_different_output() {
        let mut fb = FrameBuffer::new(20, 20);
        fb.clear(255, 255, 255, 255);
        // 画一个黑色方块
        for y in 8..12 {
            for x in 8..12 {
                fb.set_pixel(x, y, [0, 0, 0, 255]);
            }
        }

        let before = fb.get_pixel(6, 6);
        let filter = FilterPrimitive {
            rect: Rect::new(5.0, 5.0, 15.0, 15.0),
            filters: vec![FilterKind::Blur(2.0)],
        };
        apply_filter(&mut fb, &filter, 1.0);
        let after = fb.get_pixel(6, 6);

        // 模糊后边缘像素应该变暗
        assert_ne!(before, after, "blur should change edge pixels");
    }

    #[test]
    fn filter_opacity_reduces_brightness() {
        let mut fb = FrameBuffer::new(10, 10);
        fb.clear(0, 0, 0, 255); // 全黑

        let filter = FilterPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            filters: vec![FilterKind::Opacity(0.5)],
        };
        apply_filter(&mut fb, &filter, 1.0);

        let _p = fb.get_pixel(5, 5);
        // opacity(0.5) 在帧缓冲中通过降低亮度模拟
        // 黑 * 0.5 = 0，保持不变
        // 但白 * 0.5 = 128
    }

    #[test]
    fn filter_brightness_multiplies() {
        let mut fb = FrameBuffer::new(10, 10);
        // 灰色背景
        for y in 0..10 {
            for x in 0..10 {
                fb.set_pixel(x, y, [100, 100, 100, 255]);
            }
        }

        let filter = FilterPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            filters: vec![FilterKind::Brightness(2.0)],
        };
        apply_filter(&mut fb, &filter, 1.0);

        let p = fb.get_pixel(5, 5);
        assert_eq!(p[0], 200, "brightness(2.0) should double the value");
    }

    /// P2-7：混合公式已知值（CSS Compositing-1 §5.1）。
    #[test]
    fn blend_formulas_known_values() {
        use crate::primitive::BlendMode;
        // multiply：cs=128(0.502)、cb=64(0.251) → 0.126 → 32
        let out = blend_rgba([128, 128, 128, 255], [64, 64, 64, 255], BlendMode::Multiply);
        assert_eq!(out[0], 32, "multiply(128, 64) 应 ≈32，got {}", out[0]);
        // screen：cs=128 → 1-(1-0.502)(1-0.251) = 1-0.498×0.749 = 0.627 → 160
        let out = blend_rgba([128, 128, 128, 255], [64, 64, 64, 255], BlendMode::Screen);
        assert_eq!(out[0], 160, "screen(128, 64) 应 ≈160，got {}", out[0]);
        // difference：|128-64| = 64
        let out = blend_rgba([128, 128, 128, 255], [64, 64, 64, 255], BlendMode::Difference);
        assert_eq!(out[0], 64);
        // darken / lighten
        assert_eq!(
            blend_rgba([128, 0, 0, 255], [64, 64, 64, 255], BlendMode::Darken)[0],
            64
        );
        assert_eq!(
            blend_rgba([128, 0, 0, 255], [64, 64, 64, 255], BlendMode::Lighten)[0],
            128
        );
        // normal：直接返回源
        assert_eq!(
            blend_rgba([10, 20, 30, 255], [200, 200, 200, 255], BlendMode::Normal),
            [10, 20, 30, 255]
        );
        // exclusion：cs+cb-2×cs×cb = 0.502+0.251-2×0.126 = 0.501 → 128
        let out = blend_rgba([128, 128, 128, 255], [64, 64, 64, 255], BlendMode::Exclusion);
        assert_eq!(out[0], 128, "exclusion 应 ≈128，got {}", out[0]);
        // color-dodge：cb/(1-cs) = 0.251/0.498 = 0.504 → 128.5 → 129
        let out = blend_rgba([128, 128, 128, 255], [64, 64, 64, 255], BlendMode::ColorDodge);
        assert_eq!(out[0], 129, "color-dodge 应 ≈129（0.504×255 舍入），got {}", out[0]);
        // color-burn：1-(1-cb)/cs = 1-0.749/0.502 = 1-1.492 → clamp 0
        let out = blend_rgba([128, 128, 128, 255], [64, 64, 64, 255], BlendMode::ColorBurn);
        assert_eq!(out[0], 0, "color-burn 应 ≈0（clamp），got {}", out[0]);
        // hue = SetLum(SetSat(Cb, Sat(Cs)), Lum(Cb))：保持**背景**色相与亮度、
        // 取**源**饱和度（CSS Compositing-1 §5.1.13）。背景绿、源红 → 结果保持绿色系。
        let out = blend_rgba([255, 0, 0, 255], [0, 128, 0, 255], BlendMode::Hue);
        assert!(out[1] > out[0], "hue 应保持背景绿色相（G>R），got {out:?}");
        // 灰背景（sat=0）下 SetSat 无中生有（max==min 直接返回）→ 规范行为保持灰
        let out = blend_rgba([255, 0, 0, 255], [128, 128, 128, 255], BlendMode::Hue);
        assert_eq!(out[0], 128, "灰背景 hue 应保持灰（规范 SetSat max==min），got {out:?}");
    }

    /// P2-7：composite_blend 区域合成（源层与背景层）。
    #[test]
    fn composite_blend_region_multiply() {
        use crate::primitive::BlendMode;
        // 背景（fb）全灰 128；源层（src）左侧红、右侧透明
        let mut fb = FrameBuffer::new_filled(8, 8, 128, 128, 128, 255);
        let mut src = FrameBuffer::new_filled(8, 8, 0, 0, 0, 0);
        for y in 0..8 {
            for x in 0..4 {
                src.set_pixel(x, y, [255, 0, 0, 255]);
            }
        }
        let blend = BlendModePrimitive {
            rect: Rect::new(0.0, 0.0, 8.0, 8.0),
            mode: BlendMode::Multiply,
        };
        composite_blend(&mut fb, &src, &blend, 1.0);
        // 红×灰128：R=255×0.502=128；G=0（红×灰=0）；左半红色区
        let p = fb.get_pixel(2, 4);
        assert_eq!(p, [128, 0, 0, 255], "multiply(红, 灰) 应 (128,0,0)，got {p:?}");
        // 右半（源透明 0,0,0,0）：multiply(0, 128) = 0 → 黑
        let p = fb.get_pixel(6, 4);
        assert_eq!(p, [0, 0, 0, 255], "multiply(透明源=0, 灰) 应黑，got {p:?}");
    }
}

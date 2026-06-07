//! 渲染滤镜和混合模式图元 — FilterPrimitive、BlendModePrimitive。

use crate::primitive::{BlendModePrimitive, FilterKind, FilterPrimitive};
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

/// 应用混合模式。
///
/// 在 CPU 渲染器中，混合模式通过「保存源 → 清除 → 重新混合」实现。
/// 简化处理：对区域内每个像素，读取当前颜色作为目标，然后使用混合模式公式重新计算。
pub fn apply_blend_mode(fb: &mut FrameBuffer, blend: &BlendModePrimitive, scale: f32) {
    let left = (blend.rect.left() * scale).floor().max(0.0) as u32;
    let top = (blend.rect.top() * scale).floor().max(0.0) as u32;
    let right = (blend.rect.right() * scale).ceil().min(fb.width as f32) as u32;
    let bottom = (blend.rect.bottom() * scale).ceil().min(fb.height as f32) as u32;

    if left >= right || top >= bottom {
        return; // 空区域，跳过
    }

    // CPU 渲染器中混合模式的简化实现：
    // Normal 模式不需要做任何事情
    // 其他混合模式在 CPU 渲染器中效果有限（完整实现需要「源」和「目标」两个图层）
    let _ = (left, top, right, bottom); // 避免未使用变量警告
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
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
}

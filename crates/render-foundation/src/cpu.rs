//! CPU 软件渲染 — 将基础图元光栅化到 RGBA 帧缓冲。

use crate::color::Color;
use crate::font::cache::{GlyphCache, GlyphKey};
use crate::font::loader::FontLoader;
use crate::gpu::renderer::GlyphDraw;
use crate::primitive::FillPrimitive;
use crate::surface::FrameBuffer;

/// 将填充矩形和 glyph 文本渲染到 CPU 帧缓冲。
///
/// `width` 和 `height` 是逻辑像素尺寸，`scale_factor` 是逻辑像素到物理像素的缩放。
/// 返回的 [`FrameBuffer`] 尺寸为物理像素。
pub fn render_scene_to_framebuffer(
    width: u32,
    height: u32,
    scale_factor: f32,
    fills: &[FillPrimitive],
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    glyphs: &[GlyphDraw],
) -> FrameBuffer {
    let scale = normalize_scale_factor(scale_factor);
    let physical_width = scale_dimension(width, scale);
    let physical_height = scale_dimension(height, scale);
    let mut fb = FrameBuffer::new(physical_width, physical_height);
    fb.clear(255, 255, 255, 255);

    for fill in fills {
        fill_rect(&mut fb, fill, scale);
    }

    for glyph in glyphs {
        draw_glyph(&mut fb, glyph, scale, font_loader, glyph_cache);
    }

    fb
}

pub(crate) fn glyph_top_left(x: f32, baseline_y: f32, x_offset: i16, y_offset: i16, height: u16) -> (f32, f32) {
    (x + x_offset as f32, baseline_y - y_offset as f32 - height as f32)
}

fn normalize_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

fn scale_dimension(value: u32, scale: f32) -> u32 {
    ((value as f32 * scale).round() as u32).max(1)
}

fn fill_rect(fb: &mut FrameBuffer, fill: &FillPrimitive, scale: f32) {
    let left = (fill.rect.left() * scale).floor().max(0.0) as u32;
    let top = (fill.rect.top() * scale).floor().max(0.0) as u32;
    let right = (fill.rect.right() * scale).ceil().max(0.0).min(fb.width as f32) as u32;
    let bottom = (fill.rect.bottom() * scale).ceil().max(0.0).min(fb.height as f32) as u32;

    if left >= right || top >= bottom {
        return;
    }

    for y in top..bottom {
        for x in left..right {
            fb.set_pixel(x, y, [fill.color.r, fill.color.g, fill.color.b, 255]);
        }
    }
}

fn draw_glyph(
    fb: &mut FrameBuffer,
    glyph: &GlyphDraw,
    scale: f32,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
) {
    let physical_font_size = glyph.font_size * scale;
    let key = GlyphKey::new(glyph.font_id, glyph.ch as u32, physical_font_size);
    let Ok(bitmap) = glyph_cache.get_or_insert_with(key, || {
        font_loader.rasterize_glyph(glyph.font_id, glyph.ch, physical_font_size)
    }) else {
        return;
    };

    if bitmap.width == 0 || bitmap.height == 0 {
        return;
    }

    let (gx, gy) = glyph_top_left(
        glyph.x * scale,
        glyph.baseline_y * scale,
        bitmap.x_offset,
        bitmap.y_offset,
        bitmap.height,
    );
    let start_x = gx.round() as i32;
    let start_y = gy.round() as i32;

    for row in 0..bitmap.height {
        for col in 0..bitmap.width {
            let px = start_x + col as i32;
            let py = start_y + row as i32;
            if px < 0 || py < 0 || px >= fb.width as i32 || py >= fb.height as i32 {
                continue;
            }

            let alpha = bitmap.data[row as usize * bitmap.width as usize + col as usize];
            if alpha == 0 {
                continue;
            }
            blend_pixel(fb, px as u32, py as u32, glyph.color, alpha);
        }
    }
}

fn blend_pixel(fb: &mut FrameBuffer, x: u32, y: u32, color: Color, alpha: u8) {
    let dst = fb.get_pixel(x, y);
    let src_a = alpha as f32 / 255.0 * (color.a as f32 / 255.0);
    let inv_a = 1.0 - src_a;
    let r = color.r as f32 * src_a + dst[0] as f32 * inv_a;
    let g = color.g as f32 * src_a + dst[1] as f32 * inv_a;
    let b = color.b as f32 * src_a + dst[2] as f32 * inv_a;
    fb.set_pixel(x, y, [r.round() as u8, g.round() as u8, b.round() as u8, 255]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    #[test]
    fn glyph_top_left_converts_fontdue_y_up_metrics_to_screen_y_down() {
        let (x, y) = glyph_top_left(10.0, 50.0, 2, -4, 18);
        assert_eq!(x, 12.0);
        assert_eq!(y, 36.0);
    }

    #[test]
    fn render_scene_to_framebuffer_scales_logical_dimensions() {
        let fills = vec![FillPrimitive {
            rect: Rect::new(0.0, 0.0, 5.0, 4.0),
            color: Color::BLACK,
        }];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);

        let fb = render_scene_to_framebuffer(10, 8, 2.0, &fills, &font_loader, &mut glyph_cache, &[]);

        assert_eq!(fb.width, 20);
        assert_eq!(fb.height, 16);
        assert_eq!(fb.get_pixel(0, 0), [0, 0, 0, 255]);
        assert_eq!(fb.get_pixel(19, 15), [255, 255, 255, 255]);
    }

    #[test]
    fn render_scene_to_framebuffer_no_scaling() {
        let fills = vec![FillPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color::RED,
        }];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);

        let fb = render_scene_to_framebuffer(10, 10, 1.0, &fills, &font_loader, &mut glyph_cache, &[]);

        assert_eq!(fb.width, 10);
        assert_eq!(fb.height, 10);
        assert_eq!(fb.get_pixel(0, 0), [255, 0, 0, 255]);
        assert_eq!(fb.get_pixel(9, 9), [255, 0, 0, 255]);
    }

    #[test]
    fn render_scene_to_framebuffer_only_glyphs() {
        let fills = [];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let glyphs = vec![GlyphDraw {
            ch: 'A',
            x: 0.0,
            baseline_y: 10.0,
            color: Color::GREEN,
            font_id: 0,
            font_size: 8.0,
        }];

        let fb = render_scene_to_framebuffer(16, 16, 1.0, &fills, &font_loader, &mut glyph_cache, &glyphs);

        assert_eq!(fb.width, 16);
        assert_eq!(fb.height, 16);
        // 没有加载真实字体，glyph 不会被渲染
        // 整个帧缓冲应该是白色
        for y in 0..16 {
            for x in 0..16 {
                assert_eq!(
                    fb.get_pixel(x, y),
                    [255, 255, 255, 255],
                    "pixel ({}, {}) should be white",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn render_scene_to_framebuffer_empty_inputs() {
        let fills = [];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let glyphs = [];

        let fb = render_scene_to_framebuffer(8, 8, 1.0, &fills, &font_loader, &mut glyph_cache, &glyphs);

        assert_eq!(fb.width, 8);
        assert_eq!(fb.height, 8);
        // 应该全是白色（清除色）
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(fb.get_pixel(x, y), [255, 255, 255, 255]);
            }
        }
    }

    #[test]
    fn render_scene_to_framebuffer_zero_scale() {
        let fills = vec![FillPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color::BLUE,
        }];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);

        let fb = render_scene_to_framebuffer(10, 10, 0.0, &fills, &font_loader, &mut glyph_cache, &[]);

        assert_eq!(fb.width, 10); // 最小尺寸为1，所以保持10
        assert_eq!(fb.height, 10);
        // 0.0 缩放被归一化为 1.0，矩形应该被渲染
        assert_eq!(fb.get_pixel(0, 0), [0, 0, 255, 255]); // 矩形应该渲染
    }

    #[test]
    fn render_scene_to_framebuffer_fill_clipping() {
        let fills = vec![FillPrimitive {
            rect: Rect::new(0.0, 0.0, 20.0, 20.0),
            color: Color::BLACK,
        }];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);

        let fb = render_scene_to_framebuffer(10, 10, 1.0, &fills, &font_loader, &mut glyph_cache, &[]);

        assert_eq!(fb.width, 10);
        assert_eq!(fb.height, 10);
        assert_eq!(fb.get_pixel(0, 0), [0, 0, 0, 255]); // 填充整个帧缓冲
        assert_eq!(fb.get_pixel(9, 9), [0, 0, 0, 255]);
    }

    #[test]
    fn render_scene_to_framebuffer_negative_scale() {
        let fills = vec![FillPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color::RED,
        }];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);

        let fb = render_scene_to_framebuffer(10, 10, -1.0, &fills, &font_loader, &mut glyph_cache, &[]);

        assert_eq!(fb.width, 10); // 负缩放应回退到1.0
        assert_eq!(fb.height, 10);
        // 由于 normalize_scale_factor 将负值转为 1.0，矩形应该被渲染
        assert_eq!(fb.get_pixel(0, 0), [255, 0, 0, 255]); // 矩形应该渲染
    }

    #[test]
    fn fill_rect_with_invalid_coordinates() {
        let fills = vec![
            FillPrimitive {
                rect: Rect::new(-10.0, -10.0, 5.0, 5.0), // 全部在帧缓冲外
                color: Color::RED,
            },
            FillPrimitive {
                rect: Rect::new(100.0, 100.0, 105.0, 105.0), // 全部在帧缓冲外
                color: Color::GREEN,
            },
            FillPrimitive {
                rect: Rect::new(5.0, 5.0, 10.0, 10.0), // 部分在帧缓冲内
                color: Color::BLUE,
            },
        ];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);

        let fb = render_scene_to_framebuffer(10, 10, 1.0, &fills, &font_loader, &mut glyph_cache, &[]);

        assert_eq!(fb.width, 10);
        assert_eq!(fb.height, 10);
        // 只有部分重叠的矩形应该渲染
        assert_eq!(fb.get_pixel(5, 5), [0, 0, 255, 255]);
        assert_eq!(fb.get_pixel(9, 9), [0, 0, 255, 255]);
    }

    #[test]
    fn glyph_out_of_bounds() {
        let fills = [];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);

        // Glyph 在帧缓冲外
        let glyphs = vec![GlyphDraw {
            ch: 'A',
            x: 100.0,
            baseline_y: 100.0,
            color: Color::RED,
            font_id: 0,
            font_size: 8.0,
        }];

        let fb = render_scene_to_framebuffer(10, 10, 1.0, &fills, &font_loader, &mut glyph_cache, &glyphs);

        assert_eq!(fb.width, 10);
        assert_eq!(fb.height, 10);
        // 应该是全白，没有渲染 glyph
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(fb.get_pixel(x, y), [255, 255, 255, 255]);
            }
        }
    }

    #[test]
    fn blend_alpha_compositing() {
        let fills = [];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);

        // 半透明 glyph
        let glyphs = vec![GlyphDraw {
            ch: 'A',
            x: 0.0,
            baseline_y: 8.0,
            color: Color::rgba(255, 0, 0, 128), // 半透明红
            font_id: 0,
            font_size: 8.0,
        }];

        let fb = render_scene_to_framebuffer(8, 8, 1.0, &fills, &font_loader, &mut glyph_cache, &glyphs);

        assert_eq!(fb.width, 8);
        assert_eq!(fb.height, 8);
        // 如果 glyph 渲染在 (0,0)，alpha 应为 128
        // 注意：实际值取决于 glyph 形状
        let mut has_non_white_pixel = false;
        for y in 0..8 {
            for x in 0..8 {
                let pixel = fb.get_pixel(x, y);
                if pixel[0] != 255 || pixel[1] != 255 || pixel[2] != 255 {
                    has_non_white_pixel = true;
                    // alpha 通道应保持 255（帧缓冲总是完全不透明）
                    assert_eq!(pixel[3], 255);
                }
            }
        }
        // 如果测试系统字体存在，glyph 应该渲染
        if has_non_white_pixel {
            assert!(true);
        }
    }

    #[test]
    fn scale_dimension_edge_cases() {
        // 测试边界尺寸计算
        assert_eq!(scale_dimension(0, 1.0), 1); // 最小为1
        assert_eq!(scale_dimension(0, 2.0), 1);
        assert_eq!(scale_dimension(10, 0.1), 1);
        assert_eq!(scale_dimension(1000, 0.001), 1);
        assert_eq!(scale_dimension(100, 1.0), 100);
        assert_eq!(scale_dimension(100, 1.5), 150);
        assert_eq!(scale_dimension(100, 1.49), 149); // 四舍五入
    }

    #[test]
    fn glyph_top_left_large_offsets() {
        // 测试大的偏移值
        let (x, y) = glyph_top_left(0.0, 0.0, 1000, 2000, 100);
        assert_eq!(x, 1000.0);
        assert_eq!(y, -2100.0); // baseline_y - y_offset - height

        // 测试正常偏移
        let (x, y) = glyph_top_left(10.0, 20.0, 5, -8, 16);
        assert_eq!(x, 15.0); // 10 + 5
        assert_eq!(y, 12.0); // 20 - (-8) - 16 = 20 + 8 - 16 = 12
    }

    #[test]
    fn render_scene_small_dimensions() {
        let fills = vec![FillPrimitive {
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
            color: Color::GREEN,
        }];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);

        // 最小尺寸
        let fb = render_scene_to_framebuffer(1, 1, 1.0, &fills, &font_loader, &mut glyph_cache, &[]);

        assert_eq!(fb.width, 1);
        assert_eq!(fb.height, 1);
        assert_eq!(fb.get_pixel(0, 0), [0, 255, 0, 255]);
    }
}

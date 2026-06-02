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
}

//! C3：compositor 进程内 GPU 光栅化（`ZW_COMPOSITOR_GPU=1`）。
//!
//! 页面 GPU 上下文仅允许在 `zero-compositor` 内创建；renderer 不持有 wgpu 设备。
//! 初始化失败或 readback 失败时由调用方回退 CPU 路径。

use crate::recovery;
use zero_render_foundation::font::{FontLoader, GlyphCache};
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::gpu::renderer::GpuRenderer;
use zero_render_foundation::image_cache::ImageCache;
use zero_render_foundation::primitive::RenderPrimitives;
use zero_render_foundation::surface::FrameBuffer;

/// 尝试 GPU 全量光栅化并写入 `back`；成功返回 true，失败返回 false（调用方走 CPU）。
#[allow(clippy::too_many_arguments)] // renderer state + frame inputs + two persistent caches + output
pub fn try_rasterize_fills_into_back(
    gpu_renderer: &mut Option<GpuRenderer>,
    width: u32,
    height: u32,
    primitives: &RenderPrimitives,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    image_cache: &mut ImageCache,
    back: &mut FrameBuffer,
    scale_factor: f32,
) -> bool {
    rasterize_gpu(
        gpu_renderer,
        width,
        height,
        primitives,
        font_loader,
        glyph_cache,
        image_cache,
        back,
        None,
        scale_factor,
    )
}

/// C3-S2：部分 dirty 时按脏区 clip 光栅并 blit 到 `back`（调用方须已 `copy_front_to_back`）。
#[allow(clippy::too_many_arguments)]
pub fn try_rasterize_partial_into_back(
    gpu_renderer: &mut Option<GpuRenderer>,
    width: u32,
    height: u32,
    primitives: &RenderPrimitives,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    image_cache: &mut ImageCache,
    back: &mut FrameBuffer,
    dirty_rects: &[(f32, f32, f32, f32)],
    scale_factor: f32,
) -> bool {
    if dirty_rects.is_empty() {
        return false;
    }
    for &(x, y, rw, rh) in dirty_rects {
        if rw <= 0.0 || rh <= 0.0 {
            continue;
        }
        let region = Rect::new(x * scale_factor, y * scale_factor, rw * scale_factor, rh * scale_factor);
        if !rasterize_gpu(
            gpu_renderer,
            width,
            height,
            primitives,
            font_loader,
            glyph_cache,
            image_cache,
            back,
            Some(region),
            scale_factor,
        ) {
            return false;
        }
    }
    true
}

#[allow(clippy::too_many_arguments)]
fn rasterize_gpu(
    gpu_renderer: &mut Option<GpuRenderer>,
    width: u32,
    height: u32,
    primitives: &RenderPrimitives,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    image_cache: &mut ImageCache,
    back: &mut FrameBuffer,
    clip: Option<Rect>,
    scale_factor: f32,
) -> bool {
    if recovery::take_simulated_device_lost(gpu_renderer) {
        return false;
    }
    // R3281（#3）：真实设备丢失（wgpu 回调置位）→ 丢弃上下文，本帧 CPU 回退、下帧重建
    if recovery::take_real_device_lost(gpu_renderer) {
        return false;
    }
    if gpu_renderer.is_none() {
        *gpu_renderer = GpuRenderer::new_headless(width, height).ok();
    }
    let Some(gpu) = gpu_renderer.as_mut() else {
        return false;
    };
    gpu.configure_surface(width, height);
    if let Some(clip_rect) = clip {
        if !primitives.images.is_empty() {
            return false;
        }
        gpu.render_scene_with_clip_scaled(
            &primitives.fills,
            font_loader,
            glyph_cache,
            &[],
            &[],
            &[],
            Some(clip_rect),
            scale_factor,
        );
    } else {
        // P0-1：GPU 生产路径未实现的特性（clips/blend/半透明/带模糊阴影）时
        // render_full_scene_gpu 返回 false → 本函数返回 false → 调用方回退 CPU 栅格化。
        if !gpu.render_full_scene_gpu(
            primitives,
            font_loader,
            glyph_cache,
            Some(image_cache),
            &[],
            &[],
            &[],
            &[],
            scale_factor,
        ) {
            return false;
        }
    }
    let Some(pixels) = gpu.read_pixels() else {
        return false;
    };
    if let Some(region) = clip {
        blit_region_from_rgba(back, &pixels, width, region);
    } else {
        let len = back.data.len().min(pixels.len());
        back.data[..len].copy_from_slice(&pixels[..len]);
    }
    true
}

fn blit_region_from_rgba(dst: &mut FrameBuffer, src: &[u8], width: u32, region: Rect) {
    let x0 = region.origin.x.max(0.0).floor() as u32;
    let y0 = region.origin.y.max(0.0).floor() as u32;
    let x1 = (region.origin.x + region.size.width).ceil() as u32;
    let y1 = (region.origin.y + region.size.height).ceil() as u32;
    let x1 = x1.min(width);
    let row = (width * 4) as usize;
    for y in y0..y1 {
        let off = (y as usize * row) + (x0 as usize * 4);
        let len = ((x1 - x0) * 4) as usize;
        if off + len <= src.len() && off + len <= dst.data.len() {
            dst.data[off..off + len].copy_from_slice(&src[off..off + len]);
        }
    }
}

#[cfg(test)]
mod tests {
    use zero_render_foundation::color::Color;
    use zero_render_foundation::geometry::Rect;
    use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};

    use super::*;

    /// C3：headless GPU 纯色 fill 可读回（GPU 不可用时跳过）。
    #[test]
    fn gpu_fill_produces_expected_solid_color() {
        let mut gpu: Option<GpuRenderer> = None;
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let mut primitives = RenderPrimitives::new();
        primitives.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 32.0, 32.0),
            color: Color::rgb(255, 0, 0),
        });

        let mut gpu_back = FrameBuffer::new(32, 32);
        let mut image_cache = ImageCache::new(8, 1 << 20);
        if !try_rasterize_fills_into_back(
            &mut gpu,
            32,
            32,
            &primitives,
            &font_loader,
            &mut glyph_cache,
            &mut image_cache,
            &mut gpu_back,
            1.0,
        ) {
            return;
        }

        assert_eq!(&gpu_back.data[..4], &[255, 0, 0, 255]);
    }

    /// C3-S2：GPU 脏区光栅保留区外像素（GPU 不可用时跳过）。
    #[test]
    fn gpu_partial_dirty_preserves_outside_pixels() {
        let mut gpu: Option<GpuRenderer> = None;
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let mut primitives = RenderPrimitives::new();
        primitives.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 16.0, 16.0),
            color: Color::rgb(255, 0, 0),
        });

        let mut back = FrameBuffer::new_filled(32, 32, 0, 0, 255, 255);
        let mut image_cache = ImageCache::new(8, 1 << 20);
        if !try_rasterize_partial_into_back(
            &mut gpu,
            32,
            32,
            &primitives,
            &font_loader,
            &mut glyph_cache,
            &mut image_cache,
            &mut back,
            &[(0.0, 0.0, 16.0, 16.0)],
            1.0,
        ) {
            return;
        }

        assert_eq!(&back.data[..4], &[255, 0, 0, 255], "脏区内应为红");
        let outside = (20 * 32 + 20) * 4;
        assert_eq!(back.data[outside], 0, "脏区外 R 应保留蓝");
        assert_eq!(back.data[outside + 2], 255, "脏区外 B 应保留蓝");
    }
}

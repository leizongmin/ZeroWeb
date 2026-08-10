//! C3：compositor 进程内 GPU 光栅化（`ZW_COMPOSITOR_GPU=1`）。
//!
//! 页面 GPU 上下文仅允许在 `zero-compositor` 内创建；renderer 不持有 wgpu 设备。
//! 初始化失败或 readback 失败时由调用方回退 CPU 路径。

use zero_render_foundation::font::{FontLoader, GlyphCache};
use zero_render_foundation::gpu::renderer::GpuRenderer;
use zero_render_foundation::primitive::RenderPrimitives;
use zero_render_foundation::surface::FrameBuffer;

/// 尝试 GPU 全量光栅化并写入 `back`；成功返回 true，失败返回 false（调用方走 CPU）。
pub fn try_rasterize_fills_into_back(
    gpu_renderer: &mut Option<GpuRenderer>,
    width: u32,
    height: u32,
    primitives: &RenderPrimitives,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    back: &mut FrameBuffer,
) -> bool {
    if gpu_renderer.is_none() {
        *gpu_renderer = GpuRenderer::new_headless(width, height).ok();
    }
    let Some(gpu) = gpu_renderer.as_mut() else {
        return false;
    };
    gpu.configure_surface(width, height);
    gpu.render_scene_ext(&primitives.fills, font_loader, glyph_cache, &[], &[], &[]);
    let Some(pixels) = gpu.read_pixels() else {
        return false;
    };
    let len = back.data.len().min(pixels.len());
    back.data[..len].copy_from_slice(&pixels[..len]);
    true
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
        if !try_rasterize_fills_into_back(
            &mut gpu,
            32,
            32,
            &primitives,
            &font_loader,
            &mut glyph_cache,
            &mut gpu_back,
        ) {
            return; // 无 GPU 后端（CI/headless）时跳过
        }

        assert_eq!(&gpu_back.data[..4], &[255, 0, 0, 255]);
    }
}

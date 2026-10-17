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
    // T34：脏区 clip 路径只有 fill+glyph 管线；帧内出现其他图元类型（渐变/圆角/
    // 描边/阴影/图片/滤镜等）或 z 序需 fill 盖 glyph 时整帧拒绝，由调用方回退 CPU
    // 脏区渲染（慢但对）。此前 glyphs 在此被静默丢弃（clip 分支传空数组）——脏区内
    // 文字整块消失。
    if !zero_render_foundation::gpu::scene_support::clip_scene_supported(primitives) {
        return false;
    }
    for &(x, y, rw, rh) in dirty_rects {
        if rw <= 0.0 || rh <= 0.0 {
            continue;
        }
        // clip 传逻辑坐标：renderer 内部按 scale 换算 scissor（此前传物理坐标被再次
        // 缩放，scale≠1 时 scissor 偏移 ×scale² 漂移）；blit 用物理 region，在
        // rasterize_gpu 内换算。
        let clip_logical = Rect::new(x, y, rw, rh);
        if !rasterize_gpu(
            gpu_renderer,
            width,
            height,
            primitives,
            font_loader,
            glyph_cache,
            image_cache,
            back,
            Some(clip_logical),
            scale_factor,
        ) {
            return false;
        }
    }
    true
}

/// `clip` 为脏区逻辑坐标（renderer 内部换算 scissor，blit 换算物理 region）；`None` 走全量。
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
    if let Some(clip_logical) = clip {
        gpu.render_page_scene_with_clip_scaled(primitives, font_loader, glyph_cache, clip_logical, scale_factor);
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
    if let Some(clip_logical) = clip {
        // 调用方传入的 scale_factor 已按 rasterize::device_scale_factor 归一化。
        let region = Rect::new(
            clip_logical.origin.x * scale_factor,
            clip_logical.origin.y * scale_factor,
            clip_logical.size.width * scale_factor,
            clip_logical.size.height * scale_factor,
        );
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
    use zero_render_foundation::primitive::{FillPrimitive, FontId, GlyphPrimitive, RenderPrimitives};

    use super::*;

    /// 带系统字体的 loader（FontLoader::new 为空，glyph 光栅化需真实字体）。
    fn font_loader_with_system_fonts() -> FontLoader {
        zero_render_foundation::font::system::load_platform_fonts().loader
    }

    /// 白底 + 黑色 'A'（baseline 32px @ (8,32)），无其他图元。
    fn fills_plus_glyph_primitives(width: f32, height: f32) -> RenderPrimitives {
        let mut primitives = RenderPrimitives::new();
        primitives.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, width, height),
            color: Color::rgb(255, 255, 255),
        });
        primitives.glyphs.push(GlyphPrimitive {
            x: 8.0,
            y: 32.0,
            font_size: 32.0,
            color: Color::rgb(0, 0, 0),
            glyph_id: 'A' as u32,
            font_glyph_index: None,
            source: None,
            font_id: FontId(0),
            font_variation_id: None,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        });
        primitives
    }

    /// 矩形区域内深色像素计数（glyph 笔画核心，含抗锯齿边缘余量）。
    fn dark_pixel_count(back: &FrameBuffer, x0: u32, y0: u32, x1: u32, y1: u32) -> usize {
        let mut count = 0;
        for y in y0..y1 {
            for x in x0..x1 {
                let off = ((y * back.width + x) * 4) as usize;
                let (r, g, b) = (back.data[off], back.data[off + 1], back.data[off + 2]);
                if r < 100 && g < 100 && b < 100 {
                    count += 1;
                }
            }
        }
        count
    }

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

    /// T34：GPU 脏区路径须与 fills 同等地位绘制 glyphs——脏区覆盖文字时不得整块
    /// 抹掉（此前 clip 分支传空 glyphs 数组，文字消失）。GPU 不可用时跳过。
    /// serial：同 [`gpu_partial_dirty_fractional_edge_no_clear_color_leak`]。
    #[test]
    #[serial_test::serial(gpu_env)]
    fn gpu_partial_dirty_draws_glyphs() {
        // 先探测 GPU 可用性（fill 全量路径），不可用则本测无意义。
        let mut gpu: Option<GpuRenderer> = None;
        {
            let probe = RenderPrimitives::new();
            let mut back = FrameBuffer::new(4, 4);
            let font_loader = font_loader_with_system_fonts();
            let mut glyph_cache = GlyphCache::new(64);
            let mut image_cache = ImageCache::new(8, 1 << 20);
            if !try_rasterize_fills_into_back(
                &mut gpu,
                4,
                4,
                &probe,
                &font_loader,
                &mut glyph_cache,
                &mut image_cache,
                &mut back,
                1.0,
            ) {
                return;
            }
        }

        let font_loader = font_loader_with_system_fonts();
        let mut glyph_cache = GlyphCache::new(1024);
        let mut image_cache = ImageCache::new(8, 1 << 20);
        let primitives = fills_plus_glyph_primitives(48.0, 48.0);
        // back 预填蓝（脏区外保留），脏区 (0,0,40,40) 覆盖字形。
        let mut back = FrameBuffer::new_filled(48, 48, 0, 0, 255, 255);
        assert!(
            try_rasterize_partial_into_back(
                &mut gpu,
                48,
                48,
                &primitives,
                &font_loader,
                &mut glyph_cache,
                &mut image_cache,
                &mut back,
                &[(0.0, 0.0, 40.0, 40.0)],
                1.0,
            ),
            "fills+glyphs 场景脏区 GPU 光栅化必须成功"
        );
        let dark = dark_pixel_count(&back, 0, 0, 40, 40);
        assert!(dark >= 10, "脏区内应保留字形笔画深色像素，got {dark}");
        let outside = ((44 * 48 + 44) * 4) as usize;
        assert_eq!(back.data[outside], 0, "脏区外 R 应保留蓝");
        assert_eq!(back.data[outside + 2], 255, "脏区外 B 应保留蓝");
    }

    /// T60：小数坐标脏区右/底边不得泄漏清屏色。此前 scissor 右/下边缘取 trunc 而
    /// blit 取 ceil，小数脏区（真实页面百分比宽度常态）最后一列/行从未被绘制、
    /// 却把 `LoadOp::Clear(WHITE)` 的像素整条拷回 back（脏区边缘 1px 白条残影，
    /// CPU 路径无此问题）。修复后 scissor 与 blit 同取 floor(left)/ceil(right)。
    /// GPU 不可用时跳过。serial：与 recovery 环境变量模拟丢设备测试互斥。
    #[test]
    #[serial_test::serial(gpu_env)]
    fn gpu_partial_dirty_fractional_edge_no_clear_color_leak() {
        let mut gpu: Option<GpuRenderer> = None;
        let font_loader = font_loader_with_system_fonts();
        let mut glyph_cache = GlyphCache::new(64);
        let mut image_cache = ImageCache::new(8, 1 << 20);
        let mut primitives = RenderPrimitives::new();
        // 全视口红底：脏区内任何未被绘制的像素都会保留 Clear(WHITE) 白色。
        primitives.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 64.0, 64.0),
            color: Color::rgb(255, 0, 0),
        });
        // 预填白底即清屏色——只有真正画上红才可见（区分"没画"与"画了"）。
        let mut back = FrameBuffer::new_filled(64, 64, 255, 255, 255, 255);
        assert!(
            try_rasterize_partial_into_back(
                &mut gpu,
                64,
                64,
                &primitives,
                &font_loader,
                &mut glyph_cache,
                &mut image_cache,
                &mut back,
                // 右边 28.6、底边 24.4 均为小数：blit 区域 [8,29)×[8,25)。
                &[(8.0, 8.0, 20.6, 16.4)],
                1.0,
            ),
            "纯 fill 场景脏区 GPU 光栅化必须成功"
        );
        for y in 8u32..25 {
            for x in 8u32..29 {
                let off = ((y * 64 + x) * 4) as usize;
                assert_eq!(
                    (back.data[off], back.data[off + 1], back.data[off + 2]),
                    (255, 0, 0),
                    "脏区 ({x},{y}) 应为红（clear 白色泄漏即此回归）"
                );
            }
        }
    }

    /// T60：同场景同脏区下 GPU 脏区光栅与 CPU 区域光栅一致（脏区内最大通道差
    /// ≤2，含字形 AA 的混合舍入容差）——两臂消费同一脏区归一是合成正确性契约。
    /// GPU 不可用时跳过。serial：同 [`gpu_partial_dirty_fractional_edge_no_clear_color_leak`]。
    #[test]
    #[serial_test::serial(gpu_env)]
    fn gpu_partial_matches_cpu_region_render() {
        let mut gpu: Option<GpuRenderer> = None;
        {
            let probe = RenderPrimitives::new();
            let mut back = FrameBuffer::new(4, 4);
            let font_loader = font_loader_with_system_fonts();
            let mut glyph_cache = GlyphCache::new(64);
            let mut image_cache = ImageCache::new(8, 1 << 20);
            if !try_rasterize_fills_into_back(
                &mut gpu,
                4,
                4,
                &probe,
                &font_loader,
                &mut glyph_cache,
                &mut image_cache,
                &mut back,
                1.0,
            ) {
                return;
            }
        }

        let font_loader = font_loader_with_system_fonts();
        let primitives = fills_plus_glyph_primitives(48.0, 48.0);
        let dirty = [(8.0, 8.0, 24.0, 24.0)];

        let mut gpu_back = FrameBuffer::new_filled(48, 48, 0, 0, 255, 255);
        let mut glyph_cache_gpu = GlyphCache::new(1024);
        let mut image_cache_gpu = ImageCache::new(8, 1 << 20);
        assert!(try_rasterize_partial_into_back(
            &mut gpu,
            48,
            48,
            &primitives,
            &font_loader,
            &mut glyph_cache_gpu,
            &mut image_cache_gpu,
            &mut gpu_back,
            &dirty,
            1.0,
        ));

        // CPU 区域光栅（compositor rasterize_into_back 非线程分支同款调用）。
        let mut cpu_back = FrameBuffer::new_filled(48, 48, 0, 0, 255, 255);
        let mut glyph_cache_cpu = GlyphCache::new(1024);
        let mut image_cache_cpu = ImageCache::new(8, 1 << 20);
        zero_render_foundation::cpu::render_full_scene_region_into(
            &mut cpu_back,
            &primitives,
            &font_loader,
            &mut glyph_cache_cpu,
            Some(&mut image_cache_cpu),
            &[],
            &[],
            &[],
            &[],
            Some(Rect::new(8.0, 8.0, 24.0, 24.0)),
            1.0,
        );

        let mut max_diff = 0i32;
        for (g, c) in gpu_back.data.iter().zip(cpu_back.data.iter()) {
            max_diff = max_diff.max((*g as i32 - *c as i32).abs());
        }
        assert!(
            max_diff <= 2,
            "GPU 与 CPU 脏区光栅应逐像素一致（AA 舍入容差 ≤2），got max {max_diff}"
        );
    }

    /// T34：脏区路径对 fills/glyphs 之外的图元类型（此处渐变）整帧拒绝（返回
    /// false → 调用方回退 CPU），不再静默丢弃脏区内内容。GPU 不可用时跳过。
    #[test]
    fn gpu_partial_dirty_rejects_unsupported_primitive_types() {
        let mut gpu: Option<GpuRenderer> = None;
        let font_loader = font_loader_with_system_fonts();
        let mut glyph_cache = GlyphCache::new(64);
        let mut image_cache = ImageCache::new(8, 1 << 20);
        {
            let mut probe = RenderPrimitives::new();
            probe.fills.push(FillPrimitive {
                rect: Rect::new(0.0, 0.0, 4.0, 4.0),
                color: Color::rgb(255, 0, 0),
            });
            let mut back = FrameBuffer::new(4, 4);
            if !try_rasterize_fills_into_back(
                &mut gpu,
                4,
                4,
                &probe,
                &font_loader,
                &mut glyph_cache,
                &mut image_cache,
                &mut back,
                1.0,
            ) {
                return;
            }
        }

        let mut primitives = fills_plus_glyph_primitives(32.0, 32.0);
        primitives
            .gradients
            .push(zero_render_foundation::primitive::GradientPrimitive {
                rect: Rect::new(0.0, 0.0, 16.0, 16.0),
                kind: zero_render_foundation::primitive::GradientKind::Linear {
                    x0: 0.0,
                    y0: 0.0,
                    x1: 16.0,
                    y1: 16.0,
                },
                stops: vec![
                    zero_render_foundation::primitive::GradientStop {
                        offset: 0.0,
                        color: Color::rgb(255, 0, 0),
                    },
                    zero_render_foundation::primitive::GradientStop {
                        offset: 1.0,
                        color: Color::rgb(0, 0, 255),
                    },
                ],
                repeating: false,
                interpolation: zero_render_foundation::primitive::GradientInterpolation::default(),
                clip: None,
            });
        let mut back = FrameBuffer::new(32, 32);
        assert!(
            !try_rasterize_partial_into_back(
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
            ),
            "含渐变的帧脏区 GPU 路径必须拒绝（回退 CPU），不得静默丢内容"
        );
    }
}

//! CPU↔GPU 全帧像素对照测试（P0-2）。
//!
//! 同一 `RenderPrimitives` 分别走 CPU 软件光栅与 GPU 无头渲染，全帧逐像素对比。
//! 把 CPU/GPU 双链路的功能分叉（docs/learnings/bugs/cpu-gpu-path-divergence.md）
//! 变成可量化失败清单：GPU 生产路径支持子集（无 clip/blend/半透明/带模糊阴影）
//! 必须与 CPU 输出一致；`scene_supported` 拒绝的子集由回退机制兜底。
//!
//! 格式语义见 `compare_frames`：场景图元用 sRGB 传递函数不动点色规避 headless
//! `Rgba8UnormSrgb` 目标的编码效应；中间色 fill 的对齐是 P2-8 独立任务。

use super::*;
use serial_test::serial;

use crate::cpu;
use crate::image_cache::{ImageCache, ImageData};
use crate::primitive::{
    FillPrimitive, GradientKind, GradientPrimitive, GradientStop, ImagePrimitive, RoundedRectPrimitive,
    ShadowPrimitive, StrokePrimitive,
};
use crate::surface::FrameBuffer;

/// 全帧逐像素对比：GPU 读回字节 vs CPU 原值字节，直接比较。
///
/// 格式语义：headless 目标是 `Rgba8UnormSrgb`——fill/图片/stroke/圆角的中间色会被
/// sRGB 编码（byte/255 当 linear 输出），故场景图元使用 sRGB 传递函数不动点色
/// （0 或 255）规避；渐变走「sRGB 纹理 decode → sRGB target encode」近似恒等链，
/// 中间色无损，可直接比。中间色 fill 的对齐是 P2-8 独立任务。
/// A 通道直通（sRGB 格式 alpha 不编码）。返回 (通道数超差, 最大通道差)。
fn compare_frames(cpu: &[u8], gpu: &[u8], tolerance: u8) -> (usize, u8) {
    assert_eq!(cpu.len(), gpu.len(), "帧尺寸不一致");
    let mut over = 0;
    let mut max_diff = 0u8;
    for (i, (&c, &g)) in cpu.iter().zip(gpu.iter()).enumerate() {
        let diff = c.abs_diff(g);
        max_diff = max_diff.max(diff);
        if diff > tolerance {
            over += 1;
        }
    }
    (over, max_diff)
}

/// 构造 GPU 生产路径支持子集的混合场景（全不透明、无 clip/blend/filter/transform）。
fn build_basic_scene() -> (RenderPrimitives, ImageCache) {
    let mut p = RenderPrimitives::default();
    // 0. 白底覆盖上半帧（GPU clear 为白、CPU framebuffer 初始为黑——底色须对齐；
    //    高度 40 让下半帧留白给阴影区域，避免背景盖掉阴影——
    //    CSS 语义 box-shadow 画在背景之下，背景覆盖阴影是正确行为）
    p.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 56.0, 40.0),
        color: Color::rgba(255, 255, 255, 255),
    });
    // 1. 纯色填充（红）
    p.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 12.0, 12.0),
        color: Color::rgba(255, 0, 0, 255),
    });
    // 2. 圆角矩形（蓝，左上角半径 4）
    p.rounded_rects.push(RoundedRectPrimitive {
        rect: Rect::new(12.0, 12.0, 16.0, 16.0),
        color: Color::rgba(0, 0, 255, 255),
        top_left_radius: 4.0,
        top_right_radius: 4.0,
        bottom_right_radius: 4.0,
        bottom_left_radius: 4.0,
    });
    // 3. 线性渐变（黑→白，水平）
    p.gradients.push(GradientPrimitive {
        rect: Rect::new(28.0, 0.0, 28.0, 16.0),
        kind: GradientKind::Linear {
            x0: 28.0,
            y0: 8.0,
            x1: 56.0,
            y1: 8.0,
        },
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: Color::rgba(0, 0, 0, 255),
            },
            GradientStop {
                offset: 1.0,
                color: Color::rgba(255, 255, 255, 255),
            },
        ],
        repeating: false,
        interpolation: crate::primitive::GradientInterpolation {
            space: crate::primitive::GradientColorSpace::Srgb,
            hue: crate::primitive::HueMethod::Shorter,
        },
    });
    // 4. 描边线段（绿，4px 实线；宽度避开像素中心边界——
    //    CPU 像素中心判定含边界、GPU 光栅化 top-left 规则不含右/下边缘）
    p.strokes.push(StrokePrimitive {
        x1: 0.0,
        y1: 30.0,
        x2: 56.0,
        y2: 30.0,
        width: 4.0,
        color: Color::rgba(0, 255, 0, 255),
        style: crate::primitive::LineStyle::Solid,
        cap: crate::primitive::LineCap::Butt,
    });
    // 5. 无模糊阴影（黑，offset (2,0)，blur=0 spread=0 outset，位于白底覆盖区之外
    //    y=44..54）——GPU/CPU 一致子集；黑是 sRGB 不动点（中间色会被 headless
    //    sRGB target 编码，P2-8 对齐）
    p.shadows.push(ShadowPrimitive {
        rect: Rect::new(4.0, 44.0, 16.0, 10.0),
        color: Color::rgba(0, 0, 0, 255),
        offset_x: 2.0,
        offset_y: 0.0,
        blur_radius: 0.0,
        spread_radius: 0.0,
        inset: false,
    });
    // 6. 1×1 红图放大到 12×12
    let mut image_cache = ImageCache::new(16, 1 << 20);
    let key = image_cache.insert(ImageData::from_rgba(vec![255, 0, 0, 255], 1, 1).expect("red image"));
    p.images.push(ImagePrimitive {
        rect: Rect::new(44.0, 18.0, 12.0, 12.0),
        image_key: key,
        clip: None,
    });
    (p, image_cache)
}

fn render_cpu(width: u32, height: u32, p: &RenderPrimitives, image_cache: Option<&mut ImageCache>) -> FrameBuffer {
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    cpu::render_full_scene(
        width,
        height,
        1.0,
        p,
        &font_loader,
        &mut glyph_cache,
        image_cache,
        &[],
        &[],
        &[],
        &[],
    )
}

fn render_gpu(width: u32, height: u32, p: &RenderPrimitives, image_cache: Option<&mut ImageCache>) -> Vec<u8> {
    let mut renderer = GpuRenderer::new_headless(width, height).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let rendered =
        renderer.render_full_scene_gpu(p, &font_loader, &mut glyph_cache, image_cache, &[], &[], &[], &[], 1.0);
    assert!(rendered, "GPU 应支持本场景（不透明、无 clip/blend/滤镜）");
    renderer.read_pixels().expect("read_pixels")
}

/// 混合场景：CPU 与 GPU 输出全帧一致（RGB 经 sRGB 编码对照，容差 ±5）。
#[serial]
#[test]
fn parity_basic_scene_matches() {
    let (p, mut image_cache) = build_basic_scene();
    let (w, h) = (56, 56);
    let cpu_fb = render_cpu(w, h, &p, Some(&mut image_cache));
    let gpu_px = render_gpu(w, h, &p, Some(&mut image_cache));
    // 阴影区域（rect 4..20×44..54 + offset(2,0) → 6..22×44..54）应画黑——确认
    // 阴影渲染没有被背景覆盖而「两边一致地白」（CSS 语义下阴影在背景之下）。
    for &(px, py) in &[(10usize, 48usize), (18, 50)] {
        let b = (py * w as usize + px) * 4;
        assert_eq!(&cpu_fb.data[b..b + 3], &[0, 0, 0], "CPU 阴影 ({px},{py}) 应为黑");
        assert_eq!(&gpu_px[b..b + 3], &[0, 0, 0], "GPU 阴影 ({px},{py}) 应为黑");
    }
    let (over, max_diff) = compare_frames(&cpu_fb.data, &gpu_px, 5);
    assert_eq!(
        over, 0,
        "CPU/GPU 对照差异：{over} 个通道超差（容差 5），最大通道差 {max_diff}"
    );
}

/// scene_supported：GPU 生产路径未实现子集必须被拒绝（触发回退而非画错）。
#[serial]
#[test]
fn parity_scene_supported_rejects_unimplemented() {
    let (p, _) = build_basic_scene();
    let empty: &[GlyphDraw] = &[];
    let headless = true;

    // clips 非空
    let mut with_clip = p.clone();
    with_clip.clips.push(crate::primitive::ClipPrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 8.0),
    });
    assert!(!crate::gpu::scene_support::scene_supported(
        &with_clip,
        empty,
        &[],
        empty,
        &[],
        headless
    ));

    // blend_modes 非空
    let mut with_blend = p.clone();
    with_blend.blend_modes.push(crate::primitive::BlendModePrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 8.0),
        mode: crate::primitive::BlendMode::Multiply,
    });
    assert!(!crate::gpu::scene_support::scene_supported(
        &with_blend,
        empty,
        &[],
        empty,
        &[],
        headless
    ));

    // 半透明填充
    let mut with_alpha = p.clone();
    with_alpha.fills[0].color = Color::rgba(255, 0, 0, 128);
    assert!(!crate::gpu::scene_support::scene_supported(
        &with_alpha,
        empty,
        &[],
        empty,
        &[],
        headless
    ));

    // 带模糊阴影
    let mut with_shadow_blur = p.clone();
    with_shadow_blur.shadows[0].blur_radius = 4.0;
    assert!(!crate::gpu::scene_support::scene_supported(
        &with_shadow_blur,
        empty,
        &[],
        empty,
        &[],
        headless
    ));

    // 窗口模式（headless=false）+ filter
    let mut with_filter = p.clone();
    with_filter.filters.push(crate::primitive::FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 8.0),
        filters: vec![crate::primitive::FilterKind::Opacity(0.5)],
    });
    assert!(!crate::gpu::scene_support::scene_supported(
        &with_filter,
        empty,
        &[],
        empty,
        &[],
        false
    ));
    // headless 下 filter 支持 → 不拒绝
    assert!(crate::gpu::scene_support::scene_supported(
        &with_filter,
        empty,
        &[],
        empty,
        &[],
        true
    ));

    // 支持子集 → 接受
    assert!(crate::gpu::scene_support::scene_supported(
        &p,
        empty,
        &[],
        empty,
        &[],
        headless
    ));
}

/// P2-6：超过 adapter max_texture_dimension_2d 的图片必须触发回退（返回 false），
/// 而非上传校验失败 panic。上限随驱动不同（llvmpipe≈8192 / Intel Arc≈16384）。
#[serial]
#[test]
fn parity_oversize_image_gpu_returns_false() {
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let max_tex = renderer.device_limits().max_texture_dimension_2d;
    // 超限图（max_tex+1 宽 × 1 高；纯色像素数据量小）
    let mut px = Vec::with_capacity((max_tex as usize + 1) * 4);
    px.resize((max_tex as usize + 1) * 4, 255);
    let mut image_cache = ImageCache::new(16, 1 << 30);
    let key = image_cache.insert(ImageData::from_rgba(px, max_tex + 1, 1).expect("big image"));
    let mut primitives = RenderPrimitives::default();
    primitives.images.push(ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 16.0),
        image_key: key,
        clip: None,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let rendered = renderer.render_full_scene_gpu(
        &primitives,
        &font_loader,
        &mut glyph_cache,
        Some(&mut image_cache),
        &[],
        &[],
        &[],
        &[],
        1.0,
    );
    assert!(!rendered, "超限图应返回 false 触发 CPU 回退");
    // 正常尺寸图不受影响（对照组）
    let mut image_cache2 = ImageCache::new(16, 1 << 20);
    let key2 = image_cache2.insert(ImageData::from_rgba(vec![255, 0, 0, 255], 1, 1).expect("small image"));
    let mut small = RenderPrimitives::default();
    small.images.push(ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 16.0),
        image_key: key2,
        clip: None,
    });
    let rendered_ok = renderer.render_full_scene_gpu(
        &small,
        &font_loader,
        &mut glyph_cache,
        Some(&mut image_cache2),
        &[],
        &[],
        &[],
        &[],
        1.0,
    );
    assert!(rendered_ok, "正常尺寸图应正常渲染");
}

/// 半透明场景 GPU 渲染必须返回 false（触发回退），而非静默画错。
#[serial]
#[test]
fn parity_semitransparent_gpu_returns_false() {
    let (mut p, mut image_cache) = build_basic_scene();
    p.fills[0].color = Color::rgba(255, 0, 0, 128);
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let rendered = renderer.render_full_scene_gpu(
        &p,
        &font_loader,
        &mut glyph_cache,
        Some(&mut image_cache),
        &[],
        &[],
        &[],
        &[],
        1.0,
    );
    assert!(!rendered, "半透明场景应返回 false 触发 CPU 回退");
}

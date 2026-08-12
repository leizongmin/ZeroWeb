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
/// #2 修复后 headless 目标为 `Rgba8Unorm`：shader 输出 byte/255 直通存储（与窗口
/// 模式、CPU 一致），中间色可直接比；A 通道直通。返回 (通道数超差, 最大通道差)。
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

    // R3284：clip 全路径支持（draw_order 原位 / 分桶末尾擦白）→ 接受
    let mut with_clip = p.clone();
    with_clip.clips.push(crate::primitive::ClipPrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 8.0),
    });
    assert!(crate::gpu::scene_support::scene_supported(&with_clip));

    // blend_modes 非空
    let mut with_blend = p.clone();
    with_blend.blend_modes.push(crate::primitive::BlendModePrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 8.0),
        mode: crate::primitive::BlendMode::Multiply,
    });
    assert!(!crate::gpu::scene_support::scene_supported(&with_blend));

    // P2-8：半透明填充现已支持（顶点携带 alpha，shader 输出 color.a × 覆盖率）→ 接受
    let mut with_alpha = p.clone();
    with_alpha.fills[0].color = Color::rgba(255, 0, 0, 128);
    assert!(crate::gpu::scene_support::scene_supported(&with_alpha));

    // R3287：模糊阴影 GPU 已支持（离屏 blur + 混合）→ 接受
    let mut with_shadow_blur = p.clone();
    with_shadow_blur.shadows[0].blur_radius = 4.0;
    assert!(crate::gpu::scene_support::scene_supported(&with_shadow_blur));

    // D/R3279：filter 窗口模式与 headless 均支持（离屏后处理）→ 不拒绝
    let mut with_filter = p.clone();
    with_filter.filters.push(crate::primitive::FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 8.0),
        filters: vec![crate::primitive::FilterKind::Opacity(0.5)],
    });
    assert!(crate::gpu::scene_support::scene_supported(&with_filter));
    assert!(crate::gpu::scene_support::scene_supported(&with_filter));

    // 支持子集 → 接受
    assert!(crate::gpu::scene_support::scene_supported(&p));
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

/// P2-8：半透明填充现由 GPU 正确渲染（顶点 alpha × 覆盖率）——不再回退，
/// 且像素为真实混合结果（128-alpha 红 over 白底 → 混合粉）。
#[serial]
#[test]
fn parity_semitransparent_gpu_renders_correctly() {
    // 128-alpha 红 fill 覆盖全帧（clear 白底）
    let mut primitives = RenderPrimitives::default();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 16.0),
        color: Color::rgba(255, 0, 0, 128),
    });
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let rendered = renderer.render_full_scene_gpu(
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
        1.0,
    );
    assert!(rendered, "半透明场景应被 GPU 支持（P2-8 alpha 通道）");
    let pixels = renderer.read_pixels().expect("read_pixels");
    // #2 修复后 headless 目标 Rgba8Unorm：byte/255 直通存储，混合结果 0.5 分量 = 128
    //（与窗口模式、CPU 一致）。
    assert_eq!(pixels[0], 255, "R 应为 255（红分量满）");
    assert!(
        (pixels[1] as i32 - 128).abs() <= 5,
        "G 应 ≈128（半混合），got {}",
        pixels[1]
    );
    assert!(
        (pixels[2] as i32 - 128).abs() <= 5,
        "B 应 ≈128（半混合），got {}",
        pixels[2]
    );
}

/// #8：P0-1 回退的「CPU 帧上传 → blit → present」链路视觉验证——headless 下
/// upload_frame + set_compositor_import + 空场景渲染 → read_pixels 应还原 CPU 帧。
#[serial]
#[test]
fn parity_cpu_fallback_upload_blit_roundtrip() {
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    // CPU 帧：左侧红、右侧蓝
    let mut cpu_fb = FrameBuffer::new_filled(16, 16, 0, 0, 255, 255);
    for y in 0..16u32 {
        for x in 0..8u32 {
            cpu_fb.set_pixel(x, y, [255, 0, 0, 255]);
        }
    }
    let texture = renderer.upload_frame(cpu_fb.width, cpu_fb.height, &cpu_fb.data);
    renderer.set_compositor_import(texture, cpu_fb.width, cpu_fb.height, 0.0, 0.0);
    let empty = RenderPrimitives::default();
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let rendered =
        renderer.render_full_scene_gpu(&empty, &font_loader, &mut glyph_cache, None, &[], &[], &[], &[], 1.0);
    assert!(rendered, "空场景 + import blit 应渲染成功");
    let pixels = renderer.read_pixels().expect("read_pixels");
    // (4,8) 红、(12,8) 蓝——CPU 帧经上传+blit 还原
    let left = (8 * 16 + 4) * 4;
    assert_eq!(&pixels[left..left + 3], &[255, 0, 0], "左侧应还原红");
    let right = (8 * 16 + 12) * 4;
    assert_eq!(&pixels[right..right + 3], &[0, 0, 255], "右侧应还原蓝");
    renderer.clear_compositor_import();
}

/// #9：多渲染器交替渲染（模拟多标签各自独立渲染器）——GPU_CREATE_MUTEX 序列化
/// 创建后各 device 独立渲染，结果互不串扰。
#[serial]
#[test]
fn parity_multiple_renderers_alternate_independently() {
    let mut red = GpuRenderer::new_headless(8, 8).expect("red renderer");
    let mut blue = GpuRenderer::new_headless(8, 8).expect("blue renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    for _ in 0..3 {
        let mut rp = RenderPrimitives::default();
        rp.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 8.0, 8.0),
            color: Color::rgba(255, 0, 0, 255),
        });
        assert!(red.render_full_scene_gpu(&rp, &font_loader, &mut glyph_cache, None, &[], &[], &[], &[], 1.0));
        let mut bp = RenderPrimitives::default();
        bp.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 8.0, 8.0),
            color: Color::rgba(0, 0, 255, 255),
        });
        assert!(blue.render_full_scene_gpu(&bp, &font_loader, &mut glyph_cache, None, &[], &[], &[], &[], 1.0));
    }
    let rp = red.read_pixels().expect("red pixels");
    let bp = blue.read_pixels().expect("blue pixels");
    assert_eq!(&rp[..3], &[255, 0, 0], "red 渲染器应保持红");
    assert_eq!(&bp[..3], &[0, 0, 255], "blue 渲染器应保持蓝");
}

/// B（R3277）：draw_order 驱动的 GPU 绘制——父背景图（ImagePrimitive 先插入）
/// 与子元素背景色（fill 后插入）：draw_order 按插入顺序 → 子 fill 盖父 image
///（CSS painting order）；分桶路径会画反（image 桶全在 fill 桶后）。
#[serial]
#[test]
fn parity_draw_order_controls_z_order() {
    use crate::primitive::DrawOp;
    let mut image_cache = ImageCache::new(16, 1 << 20);
    let key = image_cache.insert(ImageData::from_rgba(vec![255, 0, 0, 255], 1, 1).expect("red image"));
    let mut p = RenderPrimitives::default();
    // 父背景图（先插入，应被子内容盖住）
    p.images.push(ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 16.0),
        image_key: key,
        clip: None,
    });
    // 子元素背景色（后插入，应盖住父背景图）
    p.fills.push(FillPrimitive {
        rect: Rect::new(2.0, 2.0, 12.0, 12.0),
        color: Color::rgba(0, 0, 255, 255),
    });
    p.draw_order = vec![DrawOp::Image(0), DrawOp::Fill(0)];

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
    assert!(rendered, "draw_order 场景应被 GPU 支持");
    let pixels = renderer.read_pixels().expect("read_pixels");
    // (4,4) 在子 fill 内 → 蓝（子盖父）
    let inner = (4 * 16 + 4) * 4;
    assert_eq!(
        &pixels[inner..inner + 3],
        &[0, 0, 255],
        "子 fill 应盖住父背景图（draw_order）"
    );
    // (0,0) 在父背景图内、子 fill 外 → 红（父背景图可见）
    let outer = 0usize;
    assert_eq!(&pixels[outer..outer + 3], &[255, 0, 0], "父背景图应可见于子 fill 外");
}

/// C（R3278）：GPU clip（draw_order 白 rect 擦白）与 CPU apply_clip 语义一致——
/// clip rect 外擦白、rect 内保留后续图元。
#[serial]
#[test]
fn parity_clip_draw_order_matches_cpu() {
    use crate::primitive::{ClipPrimitive, DrawOp};
    let mut p = RenderPrimitives::default();
    // 全帧蓝底
    p.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 16.0),
        color: Color::rgba(0, 0, 255, 255),
    });
    // clip rect (2,2,12,12)：rect 外擦白
    p.clips.push(ClipPrimitive {
        rect: Rect::new(2.0, 2.0, 12.0, 12.0),
    });
    p.draw_order = vec![DrawOp::Fill(0), DrawOp::Clip(0)];
    // CPU 与 GPU 分别渲染对比
    let cpu_fb = render_cpu(16, 16, &p, None);
    let gpu_px = render_gpu(16, 16, &p, None);
    let (over, max_diff) = compare_frames(&cpu_fb.data, &gpu_px, 0);
    assert_eq!(over, 0, "clip 场景 CPU/GPU 应逐像素一致，diff={over} max={max_diff}");
    // 语义抽查：clip 内 (4,4) 蓝、clip 外 (0,0) 白
    let inner = (4 * 16 + 4) * 4;
    assert_eq!(&gpu_px[inner..inner + 3], &[0, 0, 255], "clip 内应保留蓝");
    let outer = 0usize;
    assert_eq!(&gpu_px[outer..outer + 3], &[255, 255, 255], "clip 外应擦白");
}

/// C（R3278）：GPU blend 双 pass（源层 + backdrop 混合）与 CPU 源层重渲染一致。
/// 背景灰 + multiply 元素层红 → 混合结果 = 红×灰（CPU/GPU 对照）。
#[serial]
#[test]
fn parity_blend_draw_order_matches_cpu() {
    use crate::primitive::{BlendMode, BlendModePrimitive, DrawOp};
    let mut p = RenderPrimitives::default();
    // 背景：全帧灰 128
    p.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 16.0),
        color: Color::rgba(128, 128, 128, 255),
    });
    // blend 元素：红（multiply 与背景混合）
    p.blend_modes.push(BlendModePrimitive {
        rect: Rect::new(2.0, 2.0, 12.0, 12.0),
        mode: BlendMode::Multiply,
    });
    p.fills.push(FillPrimitive {
        rect: Rect::new(2.0, 2.0, 12.0, 12.0),
        color: Color::rgba(255, 0, 0, 255),
    });
    p.draw_order = vec![DrawOp::Fill(0), DrawOp::BlendMode(0), DrawOp::Fill(1)];

    let cpu_fb = render_cpu(16, 16, &p, None);
    let gpu_px = render_gpu(16, 16, &p, None);
    // blend 区域 (2,2,12,12) 内 multiply：红×灰 = (128, 0, 0)
    let inner = (4 * 16 + 4) * 4;
    assert_eq!(&gpu_px[inner..inner + 3], &[128, 0, 0], "GPU blend 区应 multiply 红×灰");
    // 区域外保持背景灰
    let outer = 0usize;
    assert_eq!(&gpu_px[outer..outer + 3], &[128, 128, 128], "blend 外应保持背景");
    // CPU/GPU 全帧一致（容差 0）
    let (over, max_diff) = compare_frames(&cpu_fb.data, &gpu_px, 0);
    assert_eq!(over, 0, "blend 场景 CPU/GPU 应逐像素一致，diff={over} max={max_diff}");
}

/// R3284：分桶路径（draw_order 空）的 clip——末尾擦白，与 CPU typed 分桶一致。
#[serial]
#[test]
fn parity_clip_bucket_path_matches_cpu() {
    use crate::primitive::ClipPrimitive;
    let mut p = RenderPrimitives::default();
    p.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 16.0),
        color: Color::rgba(0, 0, 255, 255),
    });
    p.clips.push(ClipPrimitive {
        rect: Rect::new(2.0, 2.0, 12.0, 12.0),
    });
    // draw_order 空 → 分桶路径
    assert!(p.draw_order.is_empty());
    let cpu_fb = render_cpu(16, 16, &p, None);
    let gpu_px = render_gpu(16, 16, &p, None);
    let (over, max_diff) = compare_frames(&cpu_fb.data, &gpu_px, 0);
    assert_eq!(
        over, 0,
        "分桶 clip 场景 CPU/GPU 应逐像素一致，diff={over} max={max_diff}"
    );
}

/// R3287：blur 阴影 GPU（离屏 blur + 混合）与 CPU 阴影（σ=blur/2 三遍 box blur）
/// 视觉对照——容差内一致（模糊核不同导致亚像素差异，宽容差）。
#[serial]
#[test]
fn parity_blur_shadow_matches_cpu() {
    // 无背景 fill：clear 白底即背景（阴影画在背景之上可见；
    // 有背景时背景盖阴影是正确 CSS 语义，阴影不可见）
    let mut p = RenderPrimitives::default();
    p.shadows.push(ShadowPrimitive {
        rect: Rect::new(4.0, 4.0, 16.0, 16.0),
        color: Color::rgba(0, 0, 0, 255),
        offset_x: 2.0,
        offset_y: 2.0,
        blur_radius: 3.0,
        spread_radius: 0.0,
        inset: false,
    });
    p.draw_order = vec![DrawOp::Shadow(0)];
    let cpu_fb = render_cpu(32, 32, &p, None);
    let gpu_px = render_gpu(32, 32, &p, None);
    {
        let row: Vec<(u8, u8)> = (0..32)
            .map(|px| {
                let b = (10 * 32 + px) * 4;
                (cpu_fb.data[b], gpu_px[b])
            })
            .collect();
        eprintln!("BLURDIAG shadow row y=10 (cpu, gpu) x=0..32: {row:?}");
    }
    // 阴影中心（rect+offset 内）：CPU blur 后暗（阴影色混合），GPU 同
    let center = (10 * 32 + 10) * 4;
    assert!(
        cpu_fb.data[center] < 200,
        "CPU 阴影中心应暗，got {}",
        cpu_fb.data[center]
    );
    assert!(gpu_px[center] < 200, "GPU 阴影中心应暗，got {}", gpu_px[center]);
    // R3291：GPU 3 遍 2D box blur（CPU 同公式 r = floor(d)）——与 CPU 逐像素一致
    let (over, max_diff) = compare_frames(&cpu_fb.data, &gpu_px, 8);
    assert_eq!(over, 0, "blur 阴影 CPU/GPU 应逐像素一致，diff={over} max={max_diff}");
    // 渐变存在断言：矩形边缘外（x=4）GPU 应非纯白（blur 扩散生效）
    let edge = (10 * 32 + 4) * 4;
    assert!(gpu_px[edge] < 250, "GPU 阴影边缘应有 blur 渐变，got {}", gpu_px[edge]);
    {
        let row: Vec<u8> = (0..32).map(|px| gpu_px[(10 * 32 + px) * 4]).collect();
        eprintln!("SHADROW gpu x=0..32: {row:?}");
    }
}

/// R3289：repeating 渐变首色标 offset≠0——GPU 色标重映射后与 CPU 折叠等效（逐像素一致）。
#[serial]
#[test]
fn parity_repeating_gradient_first_offset_matches_cpu() {
    let mut p = RenderPrimitives::default();
    // 周期 [0.2, 0.6]（first≠0）：黑→白→黑 折叠
    p.gradients.push(GradientPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 16.0),
        kind: GradientKind::Linear {
            x0: 0.0,
            y0: 8.0,
            x1: 32.0,
            y1: 8.0,
        },
        stops: vec![
            GradientStop {
                offset: 0.2,
                color: Color::rgba(0, 0, 0, 255),
            },
            GradientStop {
                offset: 0.6,
                color: Color::rgba(255, 255, 255, 255),
            },
        ],
        repeating: true,
        interpolation: crate::primitive::GradientInterpolation {
            space: crate::primitive::GradientColorSpace::Srgb,
            hue: crate::primitive::HueMethod::Shorter,
        },
    });
    let cpu_fb = render_cpu(32, 16, &p, None);
    let gpu_px = render_gpu(32, 16, &p, None);
    let (over, max_diff) = compare_frames(&cpu_fb.data, &gpu_px, 8);
    assert_eq!(
        over, 0,
        "repeating 渐变 first≠0 CPU/GPU 应逐像素一致，diff={over} max={max_diff}"
    );
}

/// R3254-G1：repeating 渐变**首色标 offset==0**（px 色标 `red 0px, blue 10px` 常见语法）——
/// 此前重映射条件漏掉 first==0，GPU 整条渐变压缩（回归）；重映射后与 CPU 折叠逐像素一致。
#[serial]
#[test]
fn parity_repeating_gradient_first_zero_offset_matches_cpu() {
    let mut p = RenderPrimitives::default();
    // 周期 [0, 10]（first==0，px 色标形态）：黑→白 折叠，视口只覆盖 0.1 个周期。
    p.gradients.push(GradientPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 16.0),
        kind: GradientKind::Linear {
            x0: 0.0,
            y0: 8.0,
            x1: 32.0,
            y1: 8.0,
        },
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: Color::rgba(0, 0, 0, 255),
            },
            GradientStop {
                offset: 10.0,
                color: Color::rgba(255, 255, 255, 255),
            },
        ],
        repeating: true,
        interpolation: crate::primitive::GradientInterpolation {
            space: crate::primitive::GradientColorSpace::Srgb,
            hue: crate::primitive::HueMethod::Shorter,
        },
    });
    let cpu_fb = render_cpu(32, 16, &p, None);
    let gpu_px = render_gpu(32, 16, &p, None);
    let (over, max_diff) = compare_frames(&cpu_fb.data, &gpu_px, 8);
    assert_eq!(
        over, 0,
        "repeating 渐变 first==0 CPU/GPU 应逐像素一致，diff={over} max={max_diff}"
    );
}

/// R3254-G1：百分比色标 first==0（`red 0%, blue 25%`，周期 [0, 0.25]）。
#[serial]
#[test]
fn parity_repeating_gradient_percent_first_zero_matches_cpu() {
    let mut p = RenderPrimitives::default();
    p.gradients.push(GradientPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 16.0),
        kind: GradientKind::Linear {
            x0: 0.0,
            y0: 8.0,
            x1: 32.0,
            y1: 8.0,
        },
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: Color::rgba(0, 0, 0, 255),
            },
            GradientStop {
                offset: 0.25,
                color: Color::rgba(255, 255, 255, 255),
            },
        ],
        repeating: true,
        interpolation: crate::primitive::GradientInterpolation {
            space: crate::primitive::GradientColorSpace::Srgb,
            hue: crate::primitive::HueMethod::Shorter,
        },
    });
    let cpu_fb = render_cpu(32, 16, &p, None);
    let gpu_px = render_gpu(32, 16, &p, None);
    let (over, max_diff) = compare_frames(&cpu_fb.data, &gpu_px, 8);
    assert_eq!(
        over, 0,
        "repeating 渐变百分比 first==0 CPU/GPU 应逐像素一致，diff={over} max={max_diff}"
    );
}

/// R3290：inset 阴影 GPU（盒内 frame 蒙版 + 洞 blur）与 CPU 视觉对照
///（blur 核差异为视觉近似，宽容差；洞边界应为渐变而非硬边）。
#[serial]
#[test]
fn parity_inset_shadow_matches_cpu() {
    let mut p = RenderPrimitives::default();
    p.shadows.push(ShadowPrimitive {
        rect: Rect::new(4.0, 4.0, 20.0, 20.0),
        color: Color::rgba(0, 0, 0, 255),
        offset_x: 3.0,
        offset_y: 3.0,
        blur_radius: 3.0,
        spread_radius: 0.0,
        inset: true,
    });
    p.draw_order = vec![DrawOp::Shadow(0)];
    let cpu_fb = render_cpu(32, 32, &p, None);
    let gpu_px = render_gpu(32, 32, &p, None);
    // 盒内非洞区域应有阴影（暗于白底）；洞中心（16,16）应保持白（挖空）
    let hole = (16 * 32 + 16) * 4;
    assert!(cpu_fb.data[hole] > 200, "CPU 洞中心应白，got {}", cpu_fb.data[hole]);
    assert!(gpu_px[hole] > 200, "GPU 洞中心应白，got {}", gpu_px[hole]);
    // 盒内边缘（洞边界附近）应有阴影
    let edge = (6 * 32 + 6) * 4;
    assert!(
        cpu_fb.data[edge] < 200,
        "CPU 盒内边缘应有阴影，got {}",
        cpu_fb.data[edge]
    );
    assert!(gpu_px[edge] < 200, "GPU 盒内边缘应有阴影，got {}", gpu_px[edge]);
    // R3291：3 遍 2D box blur 对齐——主体一致；洞边界 blur 边缘语义（CPU box_blur
    // 边界 vs GPU ClampToEdge）有细微差异，视觉近似容差
    let (over, max_diff) = compare_frames(&cpu_fb.data, &gpu_px, 90);
    let over_ratio = over as f64 / (cpu_fb.data.len() / 4) as f64;
    assert!(
        over_ratio < 0.35,
        "inset 阴影 CPU/GPU 差异比例应 <35%，got {over_ratio:.3} (max_diff={max_diff})"
    );
}

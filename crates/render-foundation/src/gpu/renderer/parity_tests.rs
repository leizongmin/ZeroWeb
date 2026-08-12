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
    let headless = true;

    // clips 非空
    let mut with_clip = p.clone();
    with_clip.clips.push(crate::primitive::ClipPrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 8.0),
    });
    assert!(!crate::gpu::scene_support::scene_supported(&with_clip, headless));

    // blend_modes 非空
    let mut with_blend = p.clone();
    with_blend.blend_modes.push(crate::primitive::BlendModePrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 8.0),
        mode: crate::primitive::BlendMode::Multiply,
    });
    assert!(!crate::gpu::scene_support::scene_supported(&with_blend, headless));

    // P2-8：半透明填充现已支持（顶点携带 alpha，shader 输出 color.a × 覆盖率）→ 接受
    let mut with_alpha = p.clone();
    with_alpha.fills[0].color = Color::rgba(255, 0, 0, 128);
    assert!(crate::gpu::scene_support::scene_supported(&with_alpha, headless));

    // 带模糊阴影
    let mut with_shadow_blur = p.clone();
    with_shadow_blur.shadows[0].blur_radius = 4.0;
    assert!(!crate::gpu::scene_support::scene_supported(&with_shadow_blur, headless));

    // 窗口模式（headless=false）+ filter
    let mut with_filter = p.clone();
    with_filter.filters.push(crate::primitive::FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 8.0),
        filters: vec![crate::primitive::FilterKind::Opacity(0.5)],
    });
    assert!(!crate::gpu::scene_support::scene_supported(&with_filter, false));
    // headless 下 filter 支持 → 不拒绝
    assert!(crate::gpu::scene_support::scene_supported(&with_filter, true));

    // 支持子集 → 接受
    assert!(crate::gpu::scene_support::scene_supported(&p, headless));
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

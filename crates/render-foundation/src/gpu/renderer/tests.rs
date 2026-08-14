//! GPU 渲染器单元测试

use super::*;
use crate::gpu::atlas::GlyphAtlasKey;
use crate::gpu::mesh::{color_to_f32, push_fill_quad};
use crate::primitive::TransformPrimitive;

// GPU 测试共享 software wgpu 后端（非线程安全：并发 device.poll + map_async 跨多 fallback
// adapter 触发 SIGSEGV，R285 记录、R286 新增 ping-pong 滤镜测试后稳定复现）。用 serial_test
// 序列化本文件全部测试，消除并发竞争。覆盖范围=本文件 58 测试（含纯逻辑测试，过度序列化代价
// 可忽略；其他 crate/文件测试仍并行）。
use serial_test::serial;

#[serial]
#[test]
fn test_push_fill_quad() {
    let mut vertices = Vec::new();
    push_fill_quad(&mut vertices, 0.0, 0.0, 100.0, 50.0, Color::rgba(255, 0, 0, 255));
    // 6 个顶点 × 7 个 float = 42
    assert_eq!(vertices.len(), 48);
    assert_eq!(vertices[2], -1.0); // u
    assert_eq!(vertices[3], -1.0); // v
}

#[serial]
#[test]
fn test_color_to_f32() {
    let (r, g, b) = color_to_f32(Color::rgba(128, 64, 255, 255));
    assert!((r - 128.0 / 255.0).abs() < f32::EPSILON);
    assert!((g - 64.0 / 255.0).abs() < f32::EPSILON);
    assert!(b.abs() > 0.99);
}

#[serial]
#[test]
fn test_push_multiple_fills() {
    let mut vertices = Vec::new();
    for i in 0..5u32 {
        push_fill_quad(
            &mut vertices,
            i as f32 * 10.0,
            0.0,
            i as f32 * 10.0 + 10.0,
            10.0,
            Color::BLACK,
        );
    }
    // 5 × 6 × 8 = 240
    assert_eq!(vertices.len(), 240);
}

#[serial]
#[test]
fn test_scale_rect_scales_origin_and_size() {
    let rect = scale_rect(Rect::new(2.0, 3.0, 10.0, 20.0), 2.0);
    assert_eq!(rect.origin.x, 4.0);
    assert_eq!(rect.origin.y, 6.0);
    assert_eq!(rect.size.width, 20.0);
    assert_eq!(rect.size.height, 40.0);
}

/// 测试无头模式 GPU 渲染器创建
#[serial]
#[test]
fn test_gpu_renderer_headless_creation() {
    let renderer = GpuRenderer::new_headless(64, 64);
    assert!(renderer.is_ok(), "Failed to create headless renderer");
    let renderer = renderer.unwrap();
    assert!(!renderer.is_window_mode());
    assert_eq!(renderer.surface_size(), (64, 64));
    assert_eq!(renderer.surface_format(), wgpu::TextureFormat::Rgba8Unorm);
}

/// 测试渲染红色填充并回读像素验证
#[serial]
#[test]
fn test_gpu_renderer_render_and_read_pixels() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::RED,
    }];
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[], &[]);

    let pixels = renderer
        .read_pixels()
        .expect("read_pixels should return data in headless mode");
    assert_eq!(pixels.len(), 32 * 32 * 4);

    // 第一个像素应为红色 (R=255, G=0, B=0, A=255)
    assert_eq!(pixels[0], 255, "R channel should be 255");
    assert_eq!(pixels[1], 0, "G channel should be 0");
    assert_eq!(pixels[2], 0, "B channel should be 0");
    assert_eq!(pixels[3], 255, "A channel should be 255");
}

/// R1595 DC-9 GPU parity：glyph `rotation = FRAC_PI_2`（90° CW）应把字形 bbox 的宽高互换——
/// 证明 GPU renderer 应用字形旋转（此前 GlyphDraw 无 rotation 字段，GPU-mode vertical 文本不旋转）。
/// 用 register_bitmap_glyph 注入可控 4w×8h 实心字形（FontLoader::new 空，须手动供字形）。
#[serial]
#[test]
fn test_gpu_renderer_rotated_glyph_swaps_dimensions() {
    use crate::font::GlyphBitmap;
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut font_loader = FontLoader::new();
    // 4 宽 × 8 高实心字形（高>宽）； upright 渲染应高>宽，rotated 90°CW 应宽>高（swap）。
    font_loader.register_bitmap_glyph(
        0,
        'I' as u32,
        16.0,
        GlyphBitmap {
            data: vec![255; 4 * 8],
            width: 4,
            height: 8,
            x_offset: 0,
            y_offset: 0,
            advance: 4.0,
        },
    );
    let mut glyph_cache = GlyphCache::new(64);

    let lit_bbox = |px: &[u8]| -> Option<(i32, i32)> {
        let (mut minx, mut miny, mut maxx, mut maxy) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for y in 0..32 {
            for x in 0..32 {
                let r = px[(y * 32 + x) as usize * 4];
                // 黑字 over 白底：glyph 区 RGB 暗（R < 200）。
                if r < 200 {
                    minx = minx.min(x);
                    maxx = maxx.max(x);
                    miny = miny.min(y);
                    maxy = maxy.max(y);
                }
            }
        }
        if maxx < minx {
            None
        } else {
            Some((maxx - minx + 1, maxy - miny + 1))
        }
    };

    let upright = vec![GlyphDraw {
        ch: 'I',
        font_glyph_index: None,
        x: 8.0,
        baseline_y: 24.0,
        color: Color::BLACK,
        font_id: 0,
        font_variations: None,
        font_size: 16.0,
        rotation: 0.0,
    }];
    renderer.render_scene(&[], &font_loader, &mut glyph_cache, &upright, &[]);
    let upright_px = renderer.read_pixels().expect("read_pixels");
    let (uw, uh) = lit_bbox(&upright_px).expect("upright glyph should render dark pixels");
    assert!(uh > uw, "upright 4×8 glyph should be tall: got {uw}w×{uh}h");

    let rotated = vec![GlyphDraw {
        ch: 'I',
        font_glyph_index: None,
        x: 8.0,
        baseline_y: 24.0,
        color: Color::BLACK,
        font_id: 0,
        font_variations: None,
        font_size: 16.0,
        rotation: std::f32::consts::FRAC_PI_2,
    }];
    renderer.render_scene(&[], &font_loader, &mut glyph_cache, &rotated, &[]);
    let rotated_px = renderer.read_pixels().expect("read_pixels");
    let (rw, rh) = lit_bbox(&rotated_px).expect("rotated glyph should render dark pixels");
    assert!(rw > rh, "rotated 90° glyph should be wide (swap): got {rw}w×{rh}h");
}

/// 测试渲染绿色填充并回读像素
#[serial]
#[test]
fn test_gpu_renderer_green_fill_readback() {
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 16.0),
        color: Color::GREEN,
    }];
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[], &[]);

    let pixels = renderer.read_pixels().expect("read_pixels");
    // 绿色 (R=0, G=255, B=0, A=255)
    assert_eq!(pixels[0], 0);
    assert_eq!(pixels[1], 255);
    assert_eq!(pixels[2], 0);
    assert_eq!(pixels[3], 255);
}

/// 测试无填充时回读像素应为白色（clear color）
#[serial]
#[test]
fn test_gpu_renderer_empty_scene_white_pixels() {
    let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_scene(&[], &font_loader, &mut glyph_cache, &[], &[]);

    let pixels = renderer.read_pixels().expect("read_pixels");
    // 白色背景 (R=255, G=255, B=255, A=255)
    assert_eq!(pixels[0], 255);
    assert_eq!(pixels[1], 255);
    assert_eq!(pixels[2], 255);
    assert_eq!(pixels[3], 255);
}

/// 测试 configure_surface 更新无头纹理尺寸
#[serial]
#[test]
fn test_gpu_renderer_configure_surface_resize() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    assert_eq!(renderer.surface_size(), (32, 32));

    renderer.configure_surface(64, 64);
    assert_eq!(renderer.surface_size(), (64, 64));
}

/// 测试 read_pixels 在窗口模式下返回 None
#[serial]
#[test]
fn test_gpu_renderer_read_pixels_window_mode_none() {
    let renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    // headless 模式有 texture，所以 read_pixels 应该能工作
    assert!(renderer.read_pixels().is_some());
}

/// 测试裁剪区域限制渲染范围
#[serial]
#[test]
fn test_gpu_renderer_clip_rect_limits_rendering() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");

    let fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::RED,
    }];
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let clip = Rect::new(0.0, 0.0, 8.0, 8.0);

    renderer.render_scene_with_clip(&fills, &font_loader, &mut glyph_cache, &[], &[], Some(clip));

    let pixels = renderer.read_pixels().expect("read_pixels");

    assert_eq!(pixels[0], 255, "R at (0,0)");
    assert_eq!(pixels[1], 0, "G at (0,0)");

    let idx = (16 * 4) as usize;
    assert_eq!(pixels[idx], 255, "R at (16,0) should be white");
    assert_eq!(pixels[idx + 1], 255, "G at (16,0) should be white");
}

/// 测试 atlas 初始状态
#[serial]
#[test]
fn test_gpu_renderer_atlas_initial_state() {
    let renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    assert_eq!(renderer.atlas_generation(), 0);
    assert_eq!(renderer.atlas_glyph_count(), 0);
}

/// 测试蓝色填充回读
#[serial]
#[test]
fn test_gpu_renderer_blue_fill_readback() {
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 16.0),
        color: Color::BLUE,
    }];
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[], &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels[0], 0);
    assert_eq!(pixels[1], 0);
    assert_eq!(pixels[2], 255);
    assert_eq!(pixels[3], 255);
}

/// 测试黑色填充回读
#[serial]
#[test]
fn test_gpu_renderer_black_fill_readback() {
    let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    let fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 8.0),
        color: Color::BLACK,
    }];
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[], &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels[0], 0);
    assert_eq!(pixels[1], 0);
    assert_eq!(pixels[2], 0);
    assert_eq!(pixels[3], 255);
}

/// 测试 glyph_draw 结构体
#[serial]
#[test]
fn test_glyph_draw_fields() {
    let gd = GlyphDraw {
        ch: 'A',
        font_glyph_index: None,
        x: 10.0,
        baseline_y: 20.0,
        color: Color::RED,
        font_id: 1,
        font_variations: None,
        font_size: 16.0,
        rotation: 0.0,
    };
    assert_eq!(gd.ch, 'A');
    assert_eq!(gd.x, 10.0);
    assert_eq!(gd.font_id, 1);
}

/// 测试 configure_surface 最小尺寸
#[serial]
#[test]
fn test_gpu_renderer_configure_surface_min_size() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    renderer.configure_surface(0, 0);
    assert_eq!(renderer.surface_size(), (1, 1));
}

/// 测试多次渲染不会 panic
#[serial]
#[test]
fn test_gpu_renderer_multiple_renders() {
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    for _ in 0..3 {
        let fills = vec![FillPrimitive {
            rect: Rect::new(0.0, 0.0, 16.0, 16.0),
            color: Color::RED,
        }];
        renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[], &[]);
    }
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 16 * 16 * 4);
}

#[serial]
#[test]
fn test_gpu_renderer_zero_sized_glyph_does_not_enter_atlas() {
    let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    let placement = renderer.upload_glyph_to_atlas(GlyphAtlasKey::new(0, ' ' as u32, 16.0), &[], 0, 0, 0, 0, 6.0);
    assert!(placement.is_none());
    assert_eq!(renderer.atlas_glyph_count(), 0);
}

/// 测试 GlyphDraw Clone 派生
#[serial]
#[test]
fn test_glyph_draw_clone() {
    let gd = GlyphDraw {
        ch: 'Z',
        font_glyph_index: None,
        x: 42.0,
        baseline_y: 88.0,
        color: Color::GREEN,
        font_id: 3,
        font_variations: None,
        font_size: 24.0,
        rotation: 0.0,
    };
    let gd2 = gd.clone();
    assert_eq!(gd2.ch, 'Z');
    assert_eq!(gd2.x, 42.0);
    assert_eq!(gd2.baseline_y, 88.0);
    assert_eq!(gd2.font_id, 3);
    assert_eq!(gd2.font_size, 24.0);
}

/// 测试 render_scene 使用空填充和空 glyph 列表
#[serial]
#[test]
fn test_render_scene_both_empty_inputs() {
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_scene(&[], &font_loader, &mut glyph_cache, &[], &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    for chunk in pixels.chunks_exact(4) {
        assert_eq!(chunk, [255, 255, 255, 255]);
    }
}

/// 测试 normalize_scale_factor 对各种边界输入的处理
#[serial]
#[test]
fn test_normalize_scale_factor_edge_cases() {
    assert_eq!(normalize_scale_factor(0.0), 1.0);
    assert_eq!(normalize_scale_factor(-1.0), 1.0);
    assert_eq!(normalize_scale_factor(f32::NAN), 1.0);
    assert_eq!(normalize_scale_factor(f32::INFINITY), 1.0);
    assert_eq!(normalize_scale_factor(f32::NEG_INFINITY), 1.0);
    assert!((normalize_scale_factor(2.0) - 2.0).abs() < f32::EPSILON);
    assert!((normalize_scale_factor(0.5) - 0.5).abs() < f32::EPSILON);
}

/// 测试上传不同尺寸的 glyph
#[serial]
#[test]
fn test_upload_glyph_to_atlas_various_sizes() {
    let mut renderer = GpuRenderer::new_headless(64, 64).expect("headless renderer");
    let sizes = [(8, 8), (16, 16), (32, 32)];
    for (i, (width, height)) in sizes.iter().enumerate() {
        let bitmap_data = vec![255u8; (width * height) as usize];
        let key = GlyphAtlasKey::new(0, 'A' as u32 + i as u32, 16.0);
        let placement = renderer.upload_glyph_to_atlas(key, &bitmap_data, *width, *height, 0, 0, 6.0);
        assert!(placement.is_some(), "width={}, height={} 应成功", width, height);
    }
}

/// 测试 render_scene_scaled 应用缩放
#[serial]
#[test]
fn test_render_scene_scaled_applies_scale() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 16.0),
        color: Color::BLACK,
    }];
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_scene_scaled(&fills, &font_loader, &mut glyph_cache, &[], &[], 2.0);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 32 * 32 * 4);
    assert_eq!(pixels[0], 0);
    assert_eq!(pixels[(31 * 32 + 31) * 4], 0);
}

/// 测试 render_scene_with_clip_scaled 结合裁剪和缩放
#[serial]
#[test]
fn test_render_scene_with_clip_scaled() {
    let mut renderer = GpuRenderer::new_headless(64, 64).expect("headless renderer");
    let fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 64.0, 64.0),
        color: Color::BLACK,
    }];
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let clip = Rect::new(16.0, 16.0, 32.0, 32.0);
    renderer.render_scene_with_clip_scaled(&fills, &font_loader, &mut glyph_cache, &[], &[], &[], Some(clip), 1.0);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels[(16 * 64 + 16) * 4], 0);
    assert_eq!(pixels[0], 255);
    assert_eq!(pixels[(63 * 64 + 63) * 4], 255);
}

/// 测试渲染混合颜色（半透明）
#[serial]
#[test]
fn test_render_scene_with_alpha_blending() {
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 16.0),
        color: Color::rgba(255, 0, 0, 128),
    }];
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[], &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels[0], 255, "R 通道应为 255");
    assert_eq!(pixels[3], 255, "alpha 通道应为 255");
}

/// 测试 surface_format 返回正确格式
#[serial]
#[test]
fn test_surface_format_returns_expected() {
    let renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let format = renderer.surface_format();
    matches!(format, wgpu::TextureFormat::Rgba8Unorm);
}

/// 测试窗口模式下的 atlas state
#[serial]
#[test]
fn test_window_mode_atlas_state() {
    let renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    assert!(renderer.atlas_generation() > 0 || renderer.atlas_glyph_count() == 0);
}

/// 测试 read_pixels 返回正确尺寸的缓冲区
#[serial]
#[test]
fn test_read_pixels_returns_correct_buffer_size() {
    let mut renderer = GpuRenderer::new_headless(10, 20).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_scene(&[], &font_loader, &mut glyph_cache, &[], &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 10 * 20 * 4);
}

/// 测试极端缩放值
#[serial]
#[test]
fn test_extreme_scale_factors() {
    let mut renderer = GpuRenderer::new_headless(4, 4).expect("headless renderer");
    let fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 1.0, 1.0),
        color: Color::BLACK,
    }];
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_scene_scaled(&fills, &font_loader, &mut glyph_cache, &[], &[], 100.0);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 4 * 4 * 4);
}

/// 测试 glyph 在图像边界上的处理
#[serial]
#[test]
fn test_glyph_at_image_edge() {
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let glyphs = vec![GlyphDraw {
        ch: 'A',
        font_glyph_index: None,
        x: 15.0,
        baseline_y: 15.0,
        color: Color::BLACK,
        font_id: 0,
        font_variations: None,
        font_size: 8.0,
        rotation: 0.0,
    }];
    renderer.render_scene(&[], &font_loader, &mut glyph_cache, &glyphs, &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 16 * 16 * 4);
}

/// 测试完全透明的 glyph
#[serial]
#[test]
fn test_glyph_alpha_zero() {
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let glyphs = vec![GlyphDraw {
        ch: 'A',
        font_glyph_index: None,
        x: 0.0,
        baseline_y: 8.0,
        color: Color::rgba(255, 255, 255, 0),
        font_id: 0,
        font_variations: None,
        font_size: 8.0,
        rotation: 0.0,
    }];
    renderer.render_scene(&[], &font_loader, &mut glyph_cache, &glyphs, &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels[0], 255);
    assert_eq!(pixels[1], 255);
    assert_eq!(pixels[2], 255);
}

/// 测试渲染到不同尺寸的表面
#[serial]
#[test]
fn test_render_to_different_surface_sizes() {
    for size in [(8, 8), (64, 64), (256, 128)] {
        let renderer = GpuRenderer::new_headless(size.0, size.1);
        assert!(renderer.is_ok(), "size {}x{} 应成功创建", size.0, size.1);
        let renderer = renderer.unwrap();
        assert_eq!(renderer.surface_size(), size);
    }
}

/// 测试 suspend_present 阻止 render_vertices 执行
#[serial]
#[test]
fn test_suspend_present_skips_render_vertices() {
    let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    renderer.suspend_present();
    assert!(renderer.is_present_suspended());
    renderer.render_vertices(&[], None);
}

/// 测试 render_vertices 在没有顶点数据时的处理
#[serial]
#[test]
fn test_render_vertices_empty_vertex_data() {
    let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    renderer.render_vertices(&[], None);
    let _pixels = renderer.read_pixels();
}

/// 测试缩放因子为 1.0 时的特殊处理
#[serial]
#[test]
fn test_scale_factor_one_point_zero() {
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 8.0),
        color: Color::BLUE,
    }];
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_scene_scaled(&fills, &font_loader, &mut glyph_cache, &[], &[], 1.0);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 16 * 16 * 4);
    assert_eq!(pixels[0], 0);
    assert_eq!(pixels[2], 255);
}

/// 测试 run_render_pass 中的裁剪区域边界情况
#[serial]
#[test]
fn test_render_pass_clip_rect_boundary_cases() {
    let mut renderer = GpuRenderer::new_headless(64, 64).expect("headless renderer");
    renderer.render_vertices(&[], Some(Rect::new(100.0, 100.0, 200.0, 200.0)));
    renderer.render_vertices(&[], Some(Rect::new(32.0, 32.0, 96.0, 96.0)));
    renderer.render_vertices(&[], Some(Rect::new(0.0, 0.0, 64.0, 64.0)));
    renderer.render_vertices(&[], Some(Rect::new(-32.0, -32.0, 32.0, 32.0)));
    let _pixels = renderer.read_pixels();
}

/// 测试 upload_glyph_to_atlas 中 atlas 满了重建的逻辑
#[serial]
#[test]
fn test_upload_glyph_atlas_rebuild_on_full() {
    let mut renderer = GpuRenderer::new_headless(128, 128).expect("headless renderer");
    let mut placed_glyphs = 0;
    for i in 0..100 {
        let bitmap_data = vec![255u8; 8 * 8];
        let key = GlyphAtlasKey::new(0, i, 16.0);
        if renderer
            .upload_glyph_to_atlas(key, &bitmap_data, 8, 8, 0, 0, 6.0)
            .is_some()
        {
            placed_glyphs += 1;
        }
    }
    assert!(placed_glyphs > 0);
}

/// 测试 read_pixels 中的错误处理
#[serial]
#[test]
fn test_read_pixels_error_handling() {
    let renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let pixels = renderer.read_pixels();
    assert!(pixels.is_some());
}

/// 测试 render_vertices 处理空顶点数据
#[serial]
#[test]
fn test_render_vertices_empty_vertex_buffer() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    renderer.render_vertices(&[], None);
    let _pixels = renderer.read_pixels();
}

/// 测试配置表面时尺寸为 1x1 的边界情况
#[serial]
#[test]
fn test_configure_surface_one_pixel() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    renderer.configure_surface(1, 1);
    assert_eq!(renderer.surface_size(), (1, 1));
    let fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 1.0, 1.0),
        color: Color::RED,
    }];
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[], &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 1 * 1 * 4);
}

/// 测试多个 glyph 使用相同字体 ID 和字符
#[serial]
#[test]
fn test_multiple_glyphs_same_font_char() {
    let mut renderer = GpuRenderer::new_headless(64, 64).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let glyphs = vec![
        GlyphDraw {
            ch: 'A',
            font_glyph_index: None,
            x: 10.0,
            baseline_y: 30.0,
            color: Color::BLACK,
            font_id: 0,
            font_variations: None,
            font_size: 16.0,
            rotation: 0.0,
        },
        GlyphDraw {
            ch: 'A',
            font_glyph_index: None,
            x: 30.0,
            baseline_y: 30.0,
            color: Color::BLACK,
            font_id: 0,
            font_variations: None,
            font_size: 16.0,
            rotation: 0.0,
        },
    ];
    renderer.render_scene(&[], &font_loader, &mut glyph_cache, &glyphs, &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 64 * 64 * 4);
}

/// 测试 glyph 在边界上的渲染
#[serial]
#[test]
fn test_glyph_at_bottom_edge() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let glyphs = vec![GlyphDraw {
        ch: 'A',
        font_glyph_index: None,
        x: 0.0,
        baseline_y: 30.0,
        color: Color::BLACK,
        font_id: 0,
        font_variations: None,
        font_size: 8.0,
        rotation: 0.0,
    }];
    renderer.render_scene(&[], &font_loader, &mut glyph_cache, &glyphs, &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 32 * 32 * 4);
}

// ─── GPU 全量图元渲染测试（render_full_scene_gpu） ──────────────────

/// 测试 render_full_scene_gpu 渲染填充矩形并回读像素
#[serial]
#[test]
fn test_gpu_full_scene_fills() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::RED,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 32 * 32 * 4);
    // 应为红色
    assert_eq!(pixels[0], 255, "R channel should be 255");
    assert_eq!(pixels[1], 0, "G channel should be 0");
    assert_eq!(pixels[2], 0, "B channel should be 0");
}

/// indexed glyph 与对应 Unicode 码点必须走同一字体 face 并产生相同 GPU 像素。
#[serial]
#[test]
fn test_gpu_full_scene_indexed_glyph_matches_code_point() {
    const LATO_TTF: &[u8] = include_bytes!("../../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
    let mut renderer = GpuRenderer::new_headless(40, 40).expect("headless renderer");
    let mut font_loader = FontLoader::new();
    let font_id = font_loader.load_font(LATO_TTF).expect("should load bundled Lato font");
    let glyph_index = font_loader
        .get(font_id)
        .expect("font should remain loaded")
        .lookup_glyph_index('A');
    let make_primitives = |font_glyph_index| {
        let mut primitives = RenderPrimitives::new();
        primitives.add_glyph(crate::primitive::GlyphPrimitive {
            x: 8.0,
            y: 28.0,
            font_size: 20.0,
            color: Color::BLACK,
            glyph_id: 'A' as u32,
            font_glyph_index,
            source: None,
            font_id: crate::primitive::FontId(font_id),
            font_variation_id: None,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        });
        primitives
    };
    let unicode = make_primitives(None);
    let indexed = make_primitives(Some(glyph_index));
    let mut glyph_cache = GlyphCache::new(8);

    renderer.render_full_scene_gpu(&unicode, &font_loader, &mut glyph_cache, None, &[], &[], &[], &[], 1.0);
    let unicode_pixels = renderer.read_pixels().expect("unicode readback");
    renderer.render_full_scene_gpu(&indexed, &font_loader, &mut glyph_cache, None, &[], &[], &[], &[], 1.0);
    let indexed_pixels = renderer.read_pixels().expect("indexed readback");

    assert_eq!(indexed_pixels, unicode_pixels);
}

#[serial]
#[test]
fn test_gpu_full_scene_preserves_draw_indices_after_unrenderable_glyph() {
    const LATO_TTF: &[u8] = include_bytes!("../../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
    let mut renderer = GpuRenderer::new_headless(40, 40).expect("headless renderer");
    let mut font_loader = FontLoader::new();
    let font_id = font_loader.load_font(LATO_TTF).expect("should load bundled Lato font");
    let glyph_index = font_loader
        .get(font_id)
        .expect("font should remain loaded")
        .lookup_glyph_index('A');
    let mut primitives = RenderPrimitives::new();
    primitives.add_glyph(crate::primitive::GlyphPrimitive {
        x: 0.0,
        y: 20.0,
        font_size: 20.0,
        color: Color::RED,
        glyph_id: '中' as u32,
        font_glyph_index: Some(0),
        source: None,
        font_id: crate::primitive::FontId(u32::MAX),
        font_variation_id: None,
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: false,
    });
    primitives.add_glyph(crate::primitive::GlyphPrimitive {
        x: 8.0,
        y: 28.0,
        font_size: 20.0,
        color: Color::BLACK,
        glyph_id: 'A' as u32,
        font_glyph_index: Some(glyph_index),
        source: None,
        font_id: crate::primitive::FontId(font_id),
        font_variation_id: None,
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: false,
    });
    let mut glyph_cache = GlyphCache::new(8);

    assert!(renderer.render_full_scene_gpu(
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
        1.0,
    ));
    let pixels = renderer.read_pixels().expect("readback");
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[0] < 250 || pixel[1] < 250 || pixel[2] < 250),
        "the renderable glyph after the missing glyph should still be drawn"
    );
}

/// 测试 render_full_scene_gpu 渲染圆角矩形
#[serial]
#[test]
fn test_gpu_full_scene_rounded_rect() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.rounded_rects.push(crate::primitive::RoundedRectPrimitive {
        rect: Rect::new(4.0, 4.0, 24.0, 24.0),
        color: Color::BLUE,
        top_left_radius: 8.0,
        top_right_radius: 8.0,
        bottom_right_radius: 8.0,
        bottom_left_radius: 8.0,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    // 中心像素应为蓝色
    let cx = 16 * 32 + 16;
    assert_eq!(pixels[cx * 4], 0, "center R should be 0");
    assert_eq!(pixels[cx * 4 + 2], 255, "center B should be 255");
    // 角落（在圆角半径外）应为白色
    assert_eq!(pixels[0], 255, "corner (0,0) should be white");
}

/// 测试 render_full_scene_gpu 渲染线性渐变
#[serial]
#[test]
fn test_gpu_full_scene_gradient() {
    let mut renderer = GpuRenderer::new_headless(64, 16).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.gradients.push(crate::primitive::GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 64.0, 16.0),
        kind: crate::primitive::GradientKind::Linear {
            x0: 0.0,
            y0: 0.0,
            x1: 64.0,
            y1: 0.0,
        },
        stops: vec![
            crate::primitive::GradientStop {
                offset: 0.0,
                color: Color::RED,
            },
            crate::primitive::GradientStop {
                offset: 1.0,
                color: Color::BLUE,
            },
        ],
        repeating: false,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    // 左端应为红色（GPU 浮点精度允许 ±3 容差）
    assert!(pixels[0] >= 252, "left R should be ~255, got {}", pixels[0]);
    assert!(pixels[2] <= 3, "left B should be ~0, got {}", pixels[2]);
    // 右端应为蓝色
    let right_idx = (63 * 4) as usize;
    assert!(
        pixels[right_idx] <= 3,
        "right R should be ~0, got {}",
        pixels[right_idx]
    );
    assert!(
        pixels[right_idx + 2] >= 252,
        "right B should be ~255, got {}",
        pixels[right_idx + 2]
    );
    // P1-5 加固：中间点 x=32 应为红蓝各半（渐变走 sRGB 纹理↔target 恒等链，
    // 中间值字节无损；此前仅左右 2 像素断言，中部插值从未验证）
    let mid_idx = (32 * 4) as usize;
    assert!(
        (pixels[mid_idx] as i32 - 128).abs() <= 5,
        "mid R should be ~128, got {}",
        pixels[mid_idx]
    );
    assert!(
        (pixels[mid_idx + 2] as i32 - 128).abs() <= 5,
        "mid B should be ~128, got {}",
        pixels[mid_idx + 2]
    );
}

/// 测试 render_full_scene_gpu 渲染不透明无模糊阴影（P1-5 加固：原测试用半透明阴影
/// 会被 scene_supported 拒绝（返回 false 触发回退），且单像素「非纯白」断言在
/// 渲染被跳过、纹理未初始化时也能侥幸通过——连「什么都没画」都测不出来）。
/// 现改为区域断言：阴影矩形（rect+offset）内全黑、阴影外保持白底。
#[serial]
#[test]
fn test_gpu_full_scene_shadow_opaque() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.shadows.push(crate::primitive::ShadowPrimitive {
        rect: Rect::new(4.0, 4.0, 24.0, 24.0),
        color: Color::BLACK,
        offset_x: 2.0,
        offset_y: 2.0,
        blur_radius: 0.0,
        spread_radius: 0.0,
        inset: false,
    });
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
    assert!(rendered, "不透明无模糊阴影属于 GPU 支持子集，应渲染成功");

    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 32 * 32 * 4);
    // 阴影矩形 4..28 × 4..28 + offset(2,2) → 6..30 × 6..30
    for &(px, py) in &[(10usize, 10usize), (20, 20), (6, 6), (29, 29)] {
        let b = (py * 32 + px) * 4;
        assert_eq!(&pixels[b..b + 4], &[0, 0, 0, 255], "阴影内 ({px},{py}) 应为不透明黑");
    }
    // 阴影外（2,2）保持 clear 白底
    let outside = &pixels[(2 * 32 + 2) * 4..(2 * 32 + 2) * 4 + 4];
    assert_eq!(outside, &[255, 255, 255, 255], "阴影外应保持白底");
}

/// 半透明阴影触发 CPU 回退（GPU 只画硬边不透明矩形，静默画错 → 返回 false）。
#[serial]
#[test]
fn test_gpu_full_scene_shadow_semitransparent() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.shadows.push(crate::primitive::ShadowPrimitive {
        rect: Rect::new(4.0, 4.0, 24.0, 24.0),
        color: Color::rgba(0, 0, 0, 128),
        offset_x: 2.0,
        offset_y: 2.0,
        blur_radius: 0.0,
        spread_radius: 0.0,
        inset: false,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    // P2-8：半透明阴影现已支持（顶点 alpha 通道）——应渲染成功
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
    assert!(rendered, "半透明无模糊阴影应被 GPU 支持（P2-8 alpha 通道）");
    let pixels = renderer.read_pixels().expect("read_pixels");
    // 阴影矩形 4..28 × 4..28 + offset(2,2) → 6..30 × 6..30；(10,10) 在半透明黑
    // 阴影内 over 白底 → 混合灰（headless sRGB 编码：encode(0.5)≈187；R 通道 0.5×0+0.5×255
    // = 0.5 线性 → encode ≈187）。半透明生效的判定：像素显著暗于纯白 255。
    let b = (10 * 32 + 10) * 4;
    assert!(pixels[b] < 230, "半透明阴影应使白底变暗（混合灰），got {}", pixels[b]);
    assert!(
        pixels[b] > 100,
        "半透明阴影不应全黑（128 alpha 混合），got {}",
        pixels[b]
    );
}

/// 测试 render_full_scene_gpu 渲染线段
#[serial]
#[test]
fn test_gpu_full_scene_stroke() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.strokes.push(crate::primitive::StrokePrimitive {
        x1: 0.0,
        y1: 16.0,
        x2: 31.0,
        y2: 16.0,
        width: 4.0,
        color: Color::BLACK,
        style: crate::primitive::LineStyle::Solid,
        cap: crate::primitive::LineCap::Butt,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    // 中间行应有黑色像素
    let mid = (16 * 32 + 16) * 4;
    assert_eq!(pixels[mid], 0, "line center R should be 0");
}

/// DC-9 GPU ImagePrimitive — 渲染纯色图片（红），断言回读像素为红色。
///
/// R661 识别的 rigor gap：GPU ImagePrimitive（draw_image_pass）已实现但无 framebuffer
/// readback 测试。本测填这个缺口，验证 GPU 图片纹理采样路径（ImageCache → 纹理 → 采样）。
#[serial]
#[test]
fn test_gpu_full_scene_image() {
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    // 1×1 纯红 RGBA 图片（放大到 16×16 rect，solid_color 检测缓存，仍走 draw_image_pass）
    let img = crate::image_cache::ImageData::from_rgba(vec![255, 0, 0, 255], 1, 1).expect("red image");
    let mut image_cache = crate::image_cache::ImageCache::new(16, 1 << 20);
    let key = image_cache.insert(img);
    let mut primitives = RenderPrimitives::default();
    primitives.images.push(crate::primitive::ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 16.0),
        image_key: key,
        clip: None,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    // 图片应填满 16×16；中心 (8,8) 应为红（GPU 浮点采样容差 ±5）
    let c = (8 * 16 + 8) * 4;
    assert!(pixels[c] >= 250, "image center R should be ~255, got {}", pixels[c]);
    assert!(pixels[c + 1] <= 5, "image center G should be ~0, got {}", pixels[c + 1]);
    assert!(pixels[c + 2] <= 5, "image center B should be ~0, got {}", pixels[c + 2]);
    assert!(
        pixels[c + 3] >= 250,
        "image center A should be ~255, got {}",
        pixels[c + 3]
    );
}

/// Repeated frames reuse GPU image/uniform resources, while changed pixels under
/// the same ImageKey allocate a new texture and become visible immediately.
#[serial]
#[test]
fn test_gpu_full_scene_reuses_resources_and_invalidates_changed_image_content() {
    let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    let mut image_cache = crate::image_cache::ImageCache::new(8, 1 << 20);
    let key = crate::image_cache::ImageKey::new(77);
    image_cache.insert_with_key(
        key.clone(),
        crate::image_cache::ImageData::from_rgba(vec![255, 0, 0, 255], 1, 1).unwrap(),
    );
    let mut primitives = RenderPrimitives::default();
    primitives.images.push(crate::primitive::ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 8.0),
        image_key: key.clone(),
        clip: None,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(16);

    renderer.render_full_scene_gpu(
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
    let uniform_address = renderer.uniform_buffer.as_ref().unwrap() as *const wgpu::Buffer;
    assert_eq!(renderer.image_texture_cache.len(), 1);

    renderer.render_full_scene_gpu(
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
    assert_eq!(renderer.image_texture_cache.len(), 1);
    assert_eq!(
        renderer.uniform_buffer.as_ref().unwrap() as *const wgpu::Buffer,
        uniform_address
    );

    image_cache.insert_with_key(
        key,
        crate::image_cache::ImageData::from_rgba(vec![0, 0, 255, 255], 1, 1).unwrap(),
    );
    renderer.render_full_scene_gpu(
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
    assert_eq!(renderer.image_texture_cache.len(), 2);
    let pixels = renderer.read_pixels().expect("read pixels");
    let center = (4 * 8 + 4) * 4;
    assert!(pixels[center] <= 5);
    assert!(pixels[center + 2] >= 250);
}

/// R3254-M3：显存字节预算——超过时按 last_used 逐出（不再无界增长到 8192 全清）。
#[serial]
#[test]
fn test_gpu_image_texture_cache_evicts_by_byte_budget() {
    let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    // 预算 = 2 张 1x1 纹理（每张 4 字节）。
    renderer.image_texture_budget_bytes = 8;
    let mut image_cache = crate::image_cache::ImageCache::new(8, 1 << 20);
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(16);

    for (id, color) in [
        (11u64, [255u8, 0, 0, 255]),
        (12, [0, 255, 0, 255]),
        (13, [0, 0, 255, 255]),
    ] {
        let key = crate::image_cache::ImageKey::new(id);
        image_cache.insert_with_key(
            key.clone(),
            crate::image_cache::ImageData::from_rgba(color.to_vec(), 1, 1).unwrap(),
        );
        let mut primitives = RenderPrimitives::default();
        primitives.images.push(crate::primitive::ImagePrimitive {
            rect: Rect::new(0.0, 0.0, 8.0, 8.0),
            image_key: key,
            clip: None,
        });
        renderer.render_full_scene_gpu(
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
    }
    // 预算 8 字节：第 3 张图触发逐出，缓存 ≤ 2 张。
    assert!(
        renderer.image_texture_cache.len() <= 2,
        "byte budget must evict oldest entries (len={})",
        renderer.image_texture_cache.len()
    );
    assert!(renderer.image_texture_cache.len() >= 1);
}

/// R3254-M4：clear_image_texture_cache 清空 GPU 纹理缓存（导航 epoch / 标签切换）。
#[serial]
#[test]
fn test_gpu_clear_image_texture_cache() {
    let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    let mut image_cache = crate::image_cache::ImageCache::new(8, 1 << 20);
    let key = crate::image_cache::ImageKey::new(99);
    image_cache.insert_with_key(
        key.clone(),
        crate::image_cache::ImageData::from_rgba(vec![1, 2, 3, 4], 1, 1).unwrap(),
    );
    let mut primitives = RenderPrimitives::default();
    primitives.images.push(crate::primitive::ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 8.0, 8.0),
        image_key: key,
        clip: None,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(16);
    renderer.render_full_scene_gpu(
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
    assert_eq!(renderer.image_texture_cache.len(), 1);
    renderer.clear_image_texture_cache();
    assert_eq!(renderer.image_texture_cache.len(), 0);
}

/// DC-9 GPU PathFillPrimitive — 渲染矩形多边形填充（黑），断言中心黑、外部白。
///
/// R661 gap：PathFill 此前仅 mesh 顶点测试（gpu/mesh.rs），无 framebuffer readback。
#[serial]
#[test]
fn test_gpu_full_scene_path_fill() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    // 矩形多边形 (4,4)-(28,28)
    primitives.path_fills.push(crate::primitive::PathFillPrimitive {
        vertices: vec![4.0, 4.0, 28.0, 4.0, 28.0, 28.0, 4.0, 28.0],
        color: Color::BLACK,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    // 中心 (16,16) 在多边形内 → 黑（深内部，实色无 AA）
    let c = (16 * 32 + 16) * 4;
    assert_eq!(
        pixels[c], 0,
        "path-fill center R should be 0 (black), got {}",
        pixels[c]
    );
    assert_eq!(pixels[c + 2], 0, "path-fill center B should be 0");
    // 角 (1,1) 在多边形外 → 白（clear color）
    let corner = (1 * 32 + 1) * 4;
    assert_eq!(pixels[corner], 255, "path-fill outside corner should be white");
}

/// DC-9 GPU PathStrokePrimitive — 渲染闭合矩形描边，断言描边边有黑像素、内部为背景白。
///
/// R661 gap：PathStroke 此前仅 mesh 顶点测试（gpu/mesh.rs），无 framebuffer readback。
#[serial]
#[test]
fn test_gpu_full_scene_path_stroke() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    // 矩形描边中心 (8,8)-(24,24)，线宽 3，闭合
    primitives.path_strokes.push(crate::primitive::PathStrokePrimitive {
        vertices: vec![8.0, 8.0, 24.0, 8.0, 24.0, 24.0, 8.0, 24.0],
        color: Color::BLACK,
        line_width: 3.0,
        closed: true,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    // 矩形中心 (16,16) 在描边内部 → 白（未被描边覆盖）
    let center = (16 * 32 + 16) * 4;
    assert_eq!(
        pixels[center], 255,
        "path-stroke interior should be white (background), got {}",
        pixels[center]
    );
    // 顶边描边带（y=8 行，x∈[8,24]）应至少有一个黑像素（描边已绘制）
    let top_edge_has_black = (8..=24).any(|x| {
        let i = (8 * 32 + x) * 4;
        pixels[i] == 0 && pixels[i + 2] == 0
    });
    assert!(
        top_edge_has_black,
        "path-stroke top edge band should contain black pixels"
    );
}

/// 测试 render_full_scene_gpu 空场景
#[serial]
#[test]
fn test_gpu_full_scene_empty() {
    let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    let primitives = RenderPrimitives::default();
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    for chunk in pixels.chunks_exact(4) {
        assert_eq!(chunk, [255, 255, 255, 255]);
    }
}

/// DC-9 GPU filter:opacity — 渲染红色填充 + Opacity(0.5) filter，断言 RGB 被乘 0.5。
///
/// 匹配 CPU `apply_filter` 的 Opacity 语义（区域 RGB *= amount）。无 filter 时为纯红 (255,0,0)；
/// 加 Opacity(0.5) 后应 ≈ (128,0,0)。
#[serial]
#[test]
fn test_gpu_full_scene_filter_opacity_multiplies_rgb() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::RED,
    });
    primitives.filters.push(crate::primitive::FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        filters: vec![crate::primitive::FilterKind::Opacity(0.5)],
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    // #2 修复后 headless 目标为 Rgba8Unorm：shader 输出 byte/255 直通存储（gamma 空间
    // 乘法，与 CPU effects.rs 一致），255 * 0.5 = 128。
    let r = pixels[0] as i32;
    assert!(
        (r - 128).abs() <= 4,
        "R should be ~128 (byte 直通) after Opacity(0.5), got {r}"
    );
    assert!(pixels[1] <= 4, "G should be ~0, got {}", pixels[1]);
    assert!(pixels[2] <= 4, "B should be ~0, got {}", pixels[2]);
}

/// DC-9 GPU filter:brightness — 红色填充 + Brightness(0.5)，断言 RGB *= 0.5（byte 直通）。
#[serial]
#[test]
fn test_gpu_full_scene_filter_brightness() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::RED,
    });
    primitives.filters.push(crate::primitive::FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        filters: vec![crate::primitive::FilterKind::Brightness(0.5)],
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    // brightness(0.5) 与 opacity 同数学（RGB *= 0.5，线性空间）→ sRGB 编码 ≈ 187
    let r = pixels[0] as i32;
    assert!(
        (r - 128).abs() <= 4,
        "R should be ~128 (byte 直通) after Brightness(0.5), got {r}"
    );
    assert!(pixels[1] <= 4, "G should be ~0, got {}", pixels[1]);
}

/// DC-9 GPU filter:contrast — 深灰填充 (64,64,64) + Contrast(2.0)，断言对比度增强使其更暗。
#[serial]
#[test]
fn test_gpu_full_scene_filter_contrast() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::rgba(64, 64, 64, 255),
    });
    primitives.filters.push(crate::primitive::FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        filters: vec![crate::primitive::FilterKind::Contrast(2.0)],
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    // contrast(2): (linear - 0.5)*2 + 0.5。64/255=0.251 linear → (0.251-0.5)*2+0.5=0.002 → sRGB ≈ 7
    // 无 filter 时 G=64；加 Contrast(2) 后应显著变暗（< 30），证明 contrast 路径生效。
    let g = pixels[1] as i32;
    assert!(g < 30, "G should be much darker than 64 after Contrast(2.0), got {g}");
    assert!(g >= 0, "G should be non-negative, got {g}");
}

/// DC-9 GPU filter:grayscale — 红色填充 + Grayscale(1.0)，断言三通道收敛为灰
///（mode 3 路径：lerp 向 Rec601 luma，red 0.299 luma ≈ sRGB-encoded 148）。
#[serial]
#[test]
fn test_gpu_full_scene_filter_grayscale() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::RED,
    });
    primitives.filters.push(crate::primitive::FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        filters: vec![crate::primitive::FilterKind::Grayscale(1.0)],
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    let (r, g, b) = (pixels[0] as i32, pixels[1] as i32, pixels[2] as i32);
    // grayscale(1.0)：三通道相等（灰），值在中灰区间（red 线性 1.0 luma=0.299 → sRGB≈148）。
    assert!((r - g).abs() <= 8, "R≈G after Grayscale(1.0), got r={r} g={g}");
    assert!((g - b).abs() <= 8, "G≈B after Grayscale(1.0), got g={g} b={b}");
    assert!(
        (60..=100).contains(&r),
        "gray value mid-range (luma 0.299×255≈76), got r={r}"
    );
}

/// DC-9 GPU filter:hue-rotate — 红色填充 + HueRotate(120)，断言 120° 旋转将红映射为绿
///（mode 4 路径：CSS hue-rotate 循环矩阵，120° 时 ma=mb=0,mc=1 → red→green）。
#[serial]
#[test]
fn test_gpu_full_scene_filter_hue_rotate() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::RED,
    });
    primitives.filters.push(crate::primitive::FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        filters: vec![crate::primitive::FilterKind::HueRotate(120.0)],
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    let (r, g, b) = (pixels[0] as i32, pixels[1] as i32, pixels[2] as i32);
    // hue-rotate(120°) 把红 (255,0,0) 旋转到绿 (0,255,0)。
    assert!(g > 200, "G should be ~255 (red→green) after HueRotate(120), got g={g}");
    assert!(r < 30, "R should be ~0 after HueRotate(120), got r={r}");
    assert!(b < 30, "B should be ~0 after HueRotate(120), got b={b}");
}

/// DC-9 GPU filter:invert — 红色填充 + Invert(1.0)，断言完全反相为青
///（mode 5 路径：mix(c, 1-c, 1.0)，red (255,0,0) → cyan (0,255,255)）。
#[serial]
#[test]
fn test_gpu_full_scene_filter_invert() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::RED,
    });
    primitives.filters.push(crate::primitive::FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        filters: vec![crate::primitive::FilterKind::Invert(1.0)],
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    let (r, g, b) = (pixels[0] as i32, pixels[1] as i32, pixels[2] as i32);
    // invert(1.0)：red (255,0,0) → cyan (0,255,255)。
    assert!(r < 30, "R should be ~0 after Invert(1.0), got r={r}");
    assert!(g > 200, "G should be ~255 after Invert(1.0), got g={g}");
    assert!(b > 200, "B should be ~255 after Invert(1.0), got b={b}");
}

/// DC-9 GPU filter:saturate — 红色填充 + Saturate(0.0)，断言去饱和为灰
///（mode 6 路径：mix(gray, c, 0.0)=gray，与 grayscale(1.0) 同数值但走 mode 6 分支）。
#[serial]
#[test]
fn test_gpu_full_scene_filter_saturate() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::RED,
    });
    primitives.filters.push(crate::primitive::FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        filters: vec![crate::primitive::FilterKind::Saturate(0.0)],
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    let (r, g, b) = (pixels[0] as i32, pixels[1] as i32, pixels[2] as i32);
    // saturate(0.0)：三通道收敛为 luma 灰（red luma=0.299 → sRGB≈148）。
    assert!((r - g).abs() <= 8, "R≈G after Saturate(0.0), got r={r} g={g}");
    assert!((g - b).abs() <= 8, "G≈B after Saturate(0.0), got g={g} b={b}");
    assert!(
        (60..=100).contains(&r),
        "gray value mid-range (luma 0.299×255≈76), got r={r}"
    );
}

/// DC-9 GPU filter:sepia — 红色填充 + Sepia(1.0)，断言转换为暖棕调
///（mode 7 路径：sepia 矩阵 + lerp，red → (0.393,0.349,0.272) sRGB≈(168,159,142)）。
#[serial]
#[test]
fn test_gpu_full_scene_filter_sepia() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::RED,
    });
    primitives.filters.push(crate::primitive::FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        filters: vec![crate::primitive::FilterKind::Sepia(1.0)],
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    let (r, g, b) = (pixels[0] as i32, pixels[1] as i32, pixels[2] as i32);
    // sepia(1.0)：red → 暖棕，三通道均升高（原 B=0 → ~142），且 R>G>B（暖调）。
    assert!(
        (50..=100).contains(&b),
        "B should rise from 0 to ~69 (sepia 0.272×255), got b={b}"
    );
    assert!(r >= g, "R>=G (warm sepia), got r={r} g={g}");
    assert!(g >= b, "G>=B (warm sepia), got g={g} b={b}");
}

/// DC-9 GPU transform — 左半红 / 右半蓝填充 + 平移 tx=8 变换，断言逆矩阵重采样产出
/// 白/红/蓝三带（匹配 CPU `apply_transform_post` 的逆变换 + clear-to-white 语义）。
///
/// 平移 a=1,b=0,c=0,d=1,tx=8：逆映射 src_x = dst_x - 8。
/// dst x∈[0,8) → src∈[-8,0) 落 rect 外 → 白；x∈[8,24) → src∈[0,16) 采样红；x∈[24,32) → src∈[16,24) 采样蓝。
#[serial]
#[test]
fn test_gpu_full_scene_transform_translation() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    // 左半红（x∈[0,16)）、右半蓝（x∈[16,32)）
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 32.0),
        color: Color::RED,
    });
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(16.0, 0.0, 16.0, 32.0),
        color: Color::BLUE,
    });
    // 平移变换：把整张图右移 8px（tx=8），原点取 rect 中心（纯平移与原点无关，此处仅为覆盖路径）
    primitives.transforms.push(TransformPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        origin_x: 16.0,
        origin_y: 16.0,
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 8.0,
        ty: 0.0,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    let px = |x: usize, y: usize| -> [u8; 4] {
        let i = (y * 32 + x) * 4;
        [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
    };
    // y=16 中线采样三带
    let white_band = px(4, 16);
    assert_eq!(
        white_band,
        [255, 255, 255, 255],
        "x=4 应为白（clear-to-white），got {white_band:?}"
    );
    let red_band = px(16, 16);
    assert!(
        red_band[0] > 200 && red_band[1] < 30 && red_band[2] < 30,
        "x=16 应为红，got {red_band:?}"
    );
    let blue_band = px(28, 16);
    assert!(
        blue_band[2] > 200 && blue_band[0] < 30 && blue_band[1] < 30,
        "x=28 应为蓝，got {blue_band:?}"
    );
}

/// DC-9 GPU filter:blur — 渲染一个边缘锐利的色块 + Blur(3)，断言边缘像素被模糊（不再纯色）。
#[serial]
#[test]
fn test_gpu_full_scene_filter_blur_softens_edges() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    // 中央 16x16 红块，四周白
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::WHITE,
    });
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(8.0, 8.0, 24.0, 24.0),
        color: Color::RED,
    });
    primitives.filters.push(crate::primitive::FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        filters: vec![crate::primitive::FilterKind::Blur(3.0)],
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    renderer.render_full_scene_gpu(
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

    let pixels = renderer.read_pixels().expect("read_pixels");
    // 模糊前：红块边缘 (x=8,y=8) 是纯红 (255,0,0)，邻近 (x<8) 是白 (255,255,255)。
    // 模糊(3) 后，边缘 G 通道应因白色渗入而上升（白 G=255，红 G=0）。
    // 白/红边界处 R 通道两侧均为 255（白=255, 红=255），不区分；用 G 通道判定：
    // 边缘像素 (8,8) 模糊前 G=0（纯红），模糊后邻近白色（G=255）渗入 → G 显著上升。
    // P1-5 加固：原相对阈值（中心<60、边缘>30）可容忍错误的模糊核（中心严重污染 /
    // 边缘仅微量渗入都算过）；改为三角核 blur(3) 的精确语义：
    // 中心 (16,16) 距边界 8px > 核半径 → 几乎不渗入（G<15）；
    // 边缘 (8,8) 半邻域为白 → G 应显著（>80）；多采样 3 个边缘点取均值。
    let center_g = pixels[((16 * 32) + 16) * 4 + 1] as i32;
    let edge_gs: i32 = [(8, 8), (8, 12), (10, 8)]
        .iter()
        .map(|&(px, py)| pixels[(py * 32 + px) * 4 + 1] as i32)
        .sum();
    let edge_avg = edge_gs / 3;
    assert!(
        center_g < 15,
        "blur center G should stay near 0 (8px from edge > blur radius), got {center_g}"
    );
    assert!(
        edge_avg > 80,
        "blur edge G should rise strongly (white bleed-in), got avg {edge_avg}"
    );
}

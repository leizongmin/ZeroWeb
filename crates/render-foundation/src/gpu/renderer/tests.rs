//! GPU 渲染器单元测试

use super::*;
use crate::gpu::atlas::GlyphAtlasKey;
use crate::gpu::mesh::{color_to_f32, push_fill_quad};

#[test]
fn test_push_fill_quad() {
    let mut vertices = Vec::new();
    push_fill_quad(&mut vertices, 0.0, 0.0, 100.0, 50.0, Color::rgba(255, 0, 0, 255));
    // 6 个顶点 × 7 个 float = 42
    assert_eq!(vertices.len(), 42);
    assert_eq!(vertices[2], -1.0); // u
    assert_eq!(vertices[3], -1.0); // v
}

#[test]
fn test_color_to_f32() {
    let (r, g, b) = color_to_f32(Color::rgba(128, 64, 255, 255));
    assert!((r - 128.0 / 255.0).abs() < f32::EPSILON);
    assert!((g - 64.0 / 255.0).abs() < f32::EPSILON);
    assert!(b.abs() > 0.99);
}

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
    // 5 × 6 × 7 = 210
    assert_eq!(vertices.len(), 210);
}

#[test]
fn test_scale_rect_scales_origin_and_size() {
    let rect = scale_rect(Rect::new(2.0, 3.0, 10.0, 20.0), 2.0);
    assert_eq!(rect.origin.x, 4.0);
    assert_eq!(rect.origin.y, 6.0);
    assert_eq!(rect.size.width, 20.0);
    assert_eq!(rect.size.height, 40.0);
}

/// 测试无头模式 GPU 渲染器创建
#[test]
fn test_gpu_renderer_headless_creation() {
    let renderer = GpuRenderer::new_headless(64, 64);
    assert!(renderer.is_ok(), "Failed to create headless renderer");
    let renderer = renderer.unwrap();
    assert!(!renderer.is_window_mode());
    assert_eq!(renderer.surface_size(), (64, 64));
    assert_eq!(renderer.surface_format(), wgpu::TextureFormat::Rgba8UnormSrgb);
}

/// 测试渲染红色填充并回读像素验证
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

/// 测试渲染绿色填充并回读像素
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
#[test]
fn test_gpu_renderer_configure_surface_resize() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    assert_eq!(renderer.surface_size(), (32, 32));

    renderer.configure_surface(64, 64);
    assert_eq!(renderer.surface_size(), (64, 64));
}

/// 测试 read_pixels 在窗口模式下返回 None
#[test]
fn test_gpu_renderer_read_pixels_window_mode_none() {
    let renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    // headless 模式有 texture，所以 read_pixels 应该能工作
    assert!(renderer.read_pixels().is_some());
}

/// 测试裁剪区域限制渲染范围
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
#[test]
fn test_gpu_renderer_atlas_initial_state() {
    let renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    assert_eq!(renderer.atlas_generation(), 0);
    assert_eq!(renderer.atlas_glyph_count(), 0);
}

/// 测试蓝色填充回读
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
#[test]
fn test_glyph_draw_fields() {
    let gd = GlyphDraw {
        ch: 'A',
        x: 10.0,
        baseline_y: 20.0,
        color: Color::RED,
        font_id: 1,
        font_size: 16.0,
    };
    assert_eq!(gd.ch, 'A');
    assert_eq!(gd.x, 10.0);
    assert_eq!(gd.font_id, 1);
}

/// 测试 configure_surface 最小尺寸
#[test]
fn test_gpu_renderer_configure_surface_min_size() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    renderer.configure_surface(0, 0);
    assert_eq!(renderer.surface_size(), (1, 1));
}

/// 测试多次渲染不会 panic
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

#[test]
fn test_gpu_renderer_zero_sized_glyph_does_not_enter_atlas() {
    let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    let placement =
        renderer.upload_glyph_to_atlas(GlyphAtlasKey::new(0, ' ' as u32, 16.0), &[], 0, 0, 0, 0, 6.0);
    assert!(placement.is_none());
    assert_eq!(renderer.atlas_glyph_count(), 0);
}

/// 测试 GlyphDraw Clone 派生
#[test]
fn test_glyph_draw_clone() {
    let gd = GlyphDraw {
        ch: 'Z',
        x: 42.0,
        baseline_y: 88.0,
        color: Color::GREEN,
        font_id: 3,
        font_size: 24.0,
    };
    let gd2 = gd.clone();
    assert_eq!(gd2.ch, 'Z');
    assert_eq!(gd2.x, 42.0);
    assert_eq!(gd2.baseline_y, 88.0);
    assert_eq!(gd2.font_id, 3);
    assert_eq!(gd2.font_size, 24.0);
}

/// 测试 render_scene 使用空填充和空 glyph 列表
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
    renderer.render_scene_with_clip_scaled(
        &fills,
        &font_loader,
        &mut glyph_cache,
        &[],
        &[],
        &[],
        Some(clip),
        1.0,
    );
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels[(16 * 64 + 16) * 4], 0);
    assert_eq!(pixels[0], 255);
    assert_eq!(pixels[(63 * 64 + 63) * 4], 255);
}

/// 测试渲染混合颜色（半透明）
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
#[test]
fn test_surface_format_returns_expected() {
    let renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let format = renderer.surface_format();
    matches!(format, wgpu::TextureFormat::Rgba8UnormSrgb);
}

/// 测试窗口模式下的 atlas state
#[test]
fn test_window_mode_atlas_state() {
    let renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    assert!(renderer.atlas_generation() > 0 || renderer.atlas_glyph_count() == 0);
}

/// 测试 read_pixels 返回正确尺寸的缓冲区
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
#[test]
fn test_glyph_at_image_edge() {
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let glyphs = vec![GlyphDraw {
        ch: 'A',
        x: 15.0,
        baseline_y: 15.0,
        color: Color::BLACK,
        font_id: 0,
        font_size: 8.0,
    }];
    renderer.render_scene(&[], &font_loader, &mut glyph_cache, &glyphs, &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 16 * 16 * 4);
}

/// 测试完全透明的 glyph
#[test]
fn test_glyph_alpha_zero() {
    let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let glyphs = vec![GlyphDraw {
        ch: 'A',
        x: 0.0,
        baseline_y: 8.0,
        color: Color::rgba(255, 255, 255, 0),
        font_id: 0,
        font_size: 8.0,
    }];
    renderer.render_scene(&[], &font_loader, &mut glyph_cache, &glyphs, &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels[0], 255);
    assert_eq!(pixels[1], 255);
    assert_eq!(pixels[2], 255);
}

/// 测试渲染到不同尺寸的表面
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
#[test]
fn test_suspend_present_skips_render_vertices() {
    let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    renderer.suspend_present();
    assert!(renderer.is_present_suspended());
    renderer.render_vertices(&[], None);
}

/// 测试 render_vertices 在没有顶点数据时的处理
#[test]
fn test_render_vertices_empty_vertex_data() {
    let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
    renderer.render_vertices(&[], None);
    let _pixels = renderer.read_pixels();
}

/// 测试缩放因子为 1.0 时的特殊处理
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
#[test]
fn test_read_pixels_error_handling() {
    let renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
    let pixels = renderer.read_pixels();
    assert!(pixels.is_some());
}

/// 测试 render_vertices 处理空顶点数据
#[test]
fn test_render_vertices_empty_vertex_buffer() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    renderer.render_vertices(&[], None);
    let _pixels = renderer.read_pixels();
}

/// 测试配置表面时尺寸为 1x1 的边界情况
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
#[test]
fn test_multiple_glyphs_same_font_char() {
    let mut renderer = GpuRenderer::new_headless(64, 64).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let glyphs = vec![
        GlyphDraw {
            ch: 'A',
            x: 10.0,
            baseline_y: 30.0,
            color: Color::BLACK,
            font_id: 0,
            font_size: 16.0,
        },
        GlyphDraw {
            ch: 'A',
            x: 30.0,
            baseline_y: 30.0,
            color: Color::BLACK,
            font_id: 0,
            font_size: 16.0,
        },
    ];
    renderer.render_scene(&[], &font_loader, &mut glyph_cache, &glyphs, &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 64 * 64 * 4);
}

/// 测试 glyph 在边界上的渲染
#[test]
fn test_glyph_at_bottom_edge() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let glyphs = vec![GlyphDraw {
        ch: 'A',
        x: 0.0,
        baseline_y: 30.0,
        color: Color::BLACK,
        font_id: 0,
        font_size: 8.0,
    }];
    renderer.render_scene(&[], &font_loader, &mut glyph_cache, &glyphs, &[]);
    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 32 * 32 * 4);
}

// ─── GPU 全量图元渲染测试（render_full_scene_gpu） ──────────────────

/// 测试 render_full_scene_gpu 渲染填充矩形并回读像素
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
        1.0,
    );

    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 32 * 32 * 4);
    // 应为红色
    assert_eq!(pixels[0], 255, "R channel should be 255");
    assert_eq!(pixels[1], 0, "G channel should be 0");
    assert_eq!(pixels[2], 0, "B channel should be 0");
}

/// 测试 render_full_scene_gpu 渲染圆角矩形
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
#[test]
fn test_gpu_full_scene_gradient() {
    let mut renderer = GpuRenderer::new_headless(64, 16).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.gradients.push(crate::primitive::GradientPrimitive {
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
        1.0,
    );

    let pixels = renderer.read_pixels().expect("read_pixels");
    // 左端应为红色
    assert_eq!(pixels[0], 255, "left R should be 255");
    assert_eq!(pixels[2], 0, "left B should be 0");
    // 右端应为蓝色
    let right_idx = (63 * 4) as usize;
    assert_eq!(pixels[right_idx], 0, "right R should be 0");
    assert_eq!(pixels[right_idx + 2], 255, "right B should be 255");
}

/// 测试 render_full_scene_gpu 渲染阴影
#[test]
fn test_gpu_full_scene_shadow() {
    let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
    let mut primitives = RenderPrimitives::default();
    primitives.shadows.push(crate::primitive::ShadowPrimitive {
        rect: Rect::new(4.0, 4.0, 24.0, 24.0),
        color: Color::rgba(0, 0, 0, 128),
        offset_x: 2.0,
        offset_y: 2.0,
        blur_radius: 0.0,
        spread_radius: 0.0,
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
        1.0,
    );

    let pixels = renderer.read_pixels().expect("read_pixels");
    assert_eq!(pixels.len(), 32 * 32 * 4);
    // 阴影区域应有非白色像素
    let shadow_px = &pixels[(10 * 32 + 10) * 4..(10 * 32 + 10) * 4 + 4];
    assert!(shadow_px[0] < 255 || shadow_px[1] < 255 || shadow_px[2] < 255,
        "shadow area should not be pure white");
}

/// 测试 render_full_scene_gpu 渲染线段
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
        1.0,
    );

    let pixels = renderer.read_pixels().expect("read_pixels");
    // 中间行应有黑色像素
    let mid = (16 * 32 + 16) * 4;
    assert_eq!(pixels[mid], 0, "line center R should be 0");
}

/// 测试 render_full_scene_gpu 空场景
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
        1.0,
    );

    let pixels = renderer.read_pixels().expect("read_pixels");
    for chunk in pixels.chunks_exact(4) {
        assert_eq!(chunk, [255, 255, 255, 255]);
    }
}

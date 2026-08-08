//! render-foundation 基准测试
//!
//! Glyph 渲染吞吐量、脏区域检测耗时、全场景 CPU 光栅吞吐量

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use zero_render_foundation::color::Color;
use zero_render_foundation::font::GlyphBitmap;
use zero_render_foundation::font::cache::{GlyphCache, GlyphKey};
use zero_render_foundation::geometry::{DamageTracker, Rect, Size};
use zero_render_foundation::primitive::RenderPrimitives;
use zero_render_foundation::surface::FrameBuffer;

fn bench_damage_tracker_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("damage_tracker");
    for size in [100usize, 500, 1000] {
        group.bench_with_input(BenchmarkId::new("add_damage", size), &size, |b, &size| {
            b.iter(|| {
                let mut tracker = DamageTracker::new();
                for i in 0..size {
                    let x = (i % 100) as f32 * 10.0;
                    let y = (i / 100) as f32 * 10.0;
                    tracker.add_damage(Rect::new(x, y, 10.0, 10.0));
                }
                black_box(&tracker);
            });
        });
    }
    group.finish();
}

fn bench_damage_tracker_damage_all(c: &mut Criterion) {
    c.bench_function("damage_tracker/damage_all", |b| {
        let mut tracker = DamageTracker::new();
        b.iter(|| {
            tracker.damage_all(Size::new(1920.0, 1080.0));
            tracker.clear();
        });
    });
}

fn bench_glyph_cache_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("glyph_cache");
    group.bench_function("insert", |b| {
        b.iter(|| {
            let mut cache = GlyphCache::new(1024);
            for i in 0..256u32 {
                let key = GlyphKey::new(0, i, 16.0);
                let bitmap = GlyphBitmap {
                    data: vec![128u8; 256],
                    width: 16,
                    height: 16,
                    x_offset: 0,
                    y_offset: 0,
                    advance: 10.0,
                };
                cache.insert(key, bitmap);
            }
            black_box(&cache);
        });
    });
    group.finish();
}

fn bench_frame_buffer_clear(c: &mut Criterion) {
    c.bench_function("frame_buffer/clear_1080p", |b| {
        let mut fb = FrameBuffer::new(1920, 1080);
        b.iter(|| {
            fb.clear(255, 255, 255, 255);
        });
    });
}

fn bench_primitives_build(c: &mut Criterion) {
    c.bench_function("primitives/build_1000_fills", |b| {
        b.iter(|| {
            let mut p = RenderPrimitives::new();
            for i in 0..1000u32 {
                let x = (i % 50) as f32 * 20.0;
                let y = (i / 50) as f32 * 20.0;
                p.add_fill(Rect::new(x, y, 18.0, 18.0), Color::rgb((i % 256) as u8, 128, 64));
            }
            black_box(&p);
        });
    });
}

/// 全场景 CPU 光栅吞吐量（性能门禁优化 S4 的可门禁指标，2026-08-08）。
///
/// 模拟浏览器每帧合成：1000 不透明 fills + 500 半透明 fills 全帧光栅化
/// （1080p）。fills-only 场景无需字体加载（空 FontLoader 即可）。
fn bench_full_scene_raster(c: &mut Criterion) {
    use zero_render_foundation::cpu::render_full_scene;
    use zero_render_foundation::font::loader::FontLoader;

    let mut scene = RenderPrimitives::new();
    for i in 0..1000u32 {
        let x = (i % 40) as f32 * 48.0;
        let y = (i / 40) as f32 * 48.0;
        scene.add_fill(Rect::new(x, y, 46.0, 46.0), Color::rgb((i % 256) as u8, 128, 64));
    }
    for i in 0..500u32 {
        let x = (i % 25) as f32 * 72.0 + 20.0;
        let y = (i / 25) as f32 * 72.0 + 20.0;
        scene.add_fill(Rect::new(x, y, 40.0, 40.0), Color::rgba(64, 96, 200, 128));
    }

    let font_loader = FontLoader::new();
    c.bench_function("full_scene/raster_1500_fills_1080p", move |b| {
        b.iter(|| {
            let mut glyph_cache = GlyphCache::new(1024);
            let fb = render_full_scene(
                1920,
                1080,
                1.0,
                &scene,
                &font_loader,
                &mut glyph_cache,
                None,
                &[],
                &[],
                &[],
                &[],
            );
            black_box(fb);
        });
    });
}

criterion_group!(
    benches,
    bench_damage_tracker_add,
    bench_damage_tracker_damage_all,
    bench_glyph_cache_insert,
    bench_frame_buffer_clear,
    bench_primitives_build,
    bench_full_scene_raster,
);
criterion_main!(benches);

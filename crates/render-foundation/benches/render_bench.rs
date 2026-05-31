//! render-foundation 基准测试
//!
//! Glyph 渲染吞吐量、脏区域检测耗时

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

criterion_group!(
    benches,
    bench_damage_tracker_add,
    bench_damage_tracker_damage_all,
    bench_glyph_cache_insert,
    bench_frame_buffer_clear,
    bench_primitives_build,
);
criterion_main!(benches);

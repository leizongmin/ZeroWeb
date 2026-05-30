//! Canvas 2D 性能基准测试。

use criterion::{Criterion, criterion_group, criterion_main};
use zero_canvas::{CanvasContext, Path2D};

/// 基准：绘制 1000 个矩形
fn bench_fill_rect_1000(c: &mut Criterion) {
    c.bench_function("fill_rect_1000", |b| {
        b.iter(|| {
            let mut ctx = CanvasContext::new(800, 600);
            for i in 0..1000u32 {
                let x = (i % 40) as f32 * 20.0;
                let y = (i / 40) as f32 * 15.0;
                ctx.fill_rect(x, y, 18.0, 13.0);
            }
        })
    });
}

/// 基准：绘制 1000 个描边矩形
fn bench_stroke_rect_1000(c: &mut Criterion) {
    c.bench_function("stroke_rect_1000", |b| {
        b.iter(|| {
            let mut ctx = CanvasContext::new(800, 600);
            for i in 0..1000u32 {
                let x = (i % 40) as f32 * 20.0;
                let y = (i / 40) as f32 * 15.0;
                ctx.stroke_rect(x, y, 18.0, 13.0);
            }
        })
    });
}

/// 基准：路径操作（move_to + line_to）
fn bench_path_operations(c: &mut Criterion) {
    c.bench_function("path_ops_1000_segments", |b| {
        b.iter(|| {
            let mut path = Path2D::new();
            path.move_to(0.0, 0.0);
            for i in 1..=1000 {
                let x = (i as f32) * 0.5;
                let y = (i as f32).sin() * 100.0;
                path.line_to(x, y);
            }
        })
    });
}

/// 基准：save/restore 嵌套
fn bench_save_restore_nested(c: &mut Criterion) {
    c.bench_function("save_restore_100_deep", |b| {
        b.iter(|| {
            let mut ctx = CanvasContext::new(800, 600);
            for _ in 0..100 {
                ctx.save();
                ctx.translate(1.0, 1.0);
            }
            for _ in 0..100 {
                ctx.restore();
            }
        })
    });
}

/// 基准：变换矩阵操作
fn bench_transform_chain(c: &mut Criterion) {
    c.bench_function("transform_chain_100", |b| {
        b.iter(|| {
            let mut ctx = CanvasContext::new(800, 600);
            for i in 0..100u32 {
                let angle = (i as f32).to_radians();
                ctx.rotate(angle);
                ctx.translate(1.0, 0.0);
            }
        })
    });
}

criterion_group!(
    benches,
    bench_fill_rect_1000,
    bench_stroke_rect_1000,
    bench_path_operations,
    bench_save_restore_nested,
    bench_transform_chain,
);
criterion_main!(benches);

//! Host Runtime 性能基准测试。

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use zero_host_runtime::window::{HostRuntime, WindowConfig};

/// 基准：创建默认窗口配置
fn bench_window_config_default(c: &mut Criterion) {
    c.bench_function("window_config_default", |b| {
        b.iter(|| {
            let _config = WindowConfig::new(black_box("Benchmark Window"));
        })
    });
}

/// 基准：创建自定义窗口配置（builder 链式调用）
fn bench_window_config_builder(c: &mut Criterion) {
    c.bench_function("window_config_builder_chain", |b| {
        b.iter(|| {
            let _config = WindowConfig::new(black_box("Bench"))
                .with_size(black_box(1920), black_box(1080))
                .with_resizable(black_box(false));
        })
    });
}

/// 基准：创建 HostRuntime 实例
fn bench_host_runtime_new(c: &mut Criterion) {
    let config = WindowConfig::new("Bench").with_size(800, 600);
    c.bench_function("host_runtime_new", |b| {
        b.iter(|| {
            let _rt = HostRuntime::new(black_box(config.clone()));
        })
    });
}

/// 基准：大量配置创建（模拟多标签页场景）
fn bench_many_configs(c: &mut Criterion) {
    c.bench_function("create_100_window_configs", |b| {
        b.iter(|| {
            let configs: Vec<WindowConfig> = (0..100)
                .map(|i| {
                    WindowConfig::new(format!("Tab {i}"))
                        .with_size(800 + (i as u32 % 5) * 100, 600)
                })
                .collect();
            black_box(configs);
        })
    });
}

criterion_group!(
    benches,
    bench_window_config_default,
    bench_window_config_builder,
    bench_host_runtime_new,
    bench_many_configs,
);
criterion_main!(benches);

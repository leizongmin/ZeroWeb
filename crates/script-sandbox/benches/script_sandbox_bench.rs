//! script-sandbox 性能基准测试。
//!
//! 测量 V8 沙箱的关键操作吞吐量：
//! - 脚本编译执行（简单表达式 vs 复杂计算）
//! - JSON 序列化开销
//! - 沙箱创建开销
//! - 多次执行吞吐量

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use zero_script_sandbox::{V8Sandbox, SandboxConfig};

/// 基准 1: 简单表达式执行吞吐量。
fn bench_simple_expression(c: &mut Criterion) {
    let mut sandbox = V8Sandbox::new().unwrap();
    c.bench_function("execute_simple_expression", |b| {
        b.iter(|| {
            sandbox.execute(black_box("1 + 1")).unwrap()
        })
    });
}

/// 基准 2: 字符串操作执行。
fn bench_string_operations(c: &mut Criterion) {
    let mut sandbox = V8Sandbox::new().unwrap();
    c.bench_function("execute_string_ops", |b| {
        b.iter(|| {
            sandbox
                .execute(black_box("'hello'.split('').reverse().join(',')"))
                .unwrap()
        })
    });
}

/// 基准 3: 循环计算执行。
fn bench_loop_computation(c: &mut Criterion) {
    let mut sandbox = V8Sandbox::new().unwrap();
    c.bench_function("execute_loop_1k", |b| {
        b.iter(|| {
            sandbox
                .execute(black_box("var s=0; for(var i=0;i<1000;i++) s+=i; s"))
                .unwrap()
        })
    });
}

/// 基准 4: JSON 序列化（execute_json）。
fn bench_json_serialize(c: &mut Criterion) {
    let mut sandbox = V8Sandbox::new().unwrap();
    c.bench_function("execute_json_object", |b| {
        b.iter(|| {
            sandbox
                .execute_json(black_box("({a:1,b:[2,3],c:{d:4}})"))
                .unwrap()
        })
    });
}

/// 基准 5: 沙箱创建开销。
fn bench_sandbox_creation(c: &mut Criterion) {
    c.bench_function("sandbox_creation", |b| {
        b.iter(|| {
            let mut sandbox = V8Sandbox::new().unwrap();
            sandbox.execute("42").unwrap()
        })
    });
}

/// 基准 6: 自定义配置创建。
fn bench_sandbox_with_config(c: &mut Criterion) {
    c.bench_function("sandbox_with_config", |b| {
        b.iter(|| {
            let config = SandboxConfig {
                heap_limit: 8 * 1024 * 1024,
                timeout_ms: 5000,
            };
            let mut sandbox = V8Sandbox::with_config(config).unwrap();
            sandbox.execute("42").unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_simple_expression,
    bench_string_operations,
    bench_loop_computation,
    bench_json_serialize,
    bench_sandbox_creation,
    bench_sandbox_with_config,
);
criterion_main!(benches);

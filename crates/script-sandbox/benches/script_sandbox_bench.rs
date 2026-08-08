//! script-sandbox 性能基准测试。
//!
//! 测量 V8 沙箱的关键操作吞吐量：
//! - 脚本编译执行（简单表达式 vs 复杂计算）
//! - JSON 序列化开销
//! - 沙箱创建开销
//! - 多次执行吞吐量
//! - ES Module 执行开销
//! - Worker 创建和通信开销
//!
//! 基准对象 `V8Sandbox`/`WorkerRuntime` 仅在 v8 feature 下导出；本 bench 在
//! Cargo.toml 声明了 `required-features = ["v8"]`，quickjs 模式下被 cargo
//! 整体跳过（`--all-targets` 编译矩阵不受 bench 阻断）。

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use zero_script_sandbox::{EsModuleSandbox, SandboxConfig, V8Sandbox, WorkerRuntime};

/// 基准 1: 简单表达式执行吞吐量。
fn bench_simple_expression(c: &mut Criterion) {
    let mut sandbox = V8Sandbox::new().unwrap();
    c.bench_function("execute_simple_expression", |b| {
        b.iter(|| sandbox.execute(black_box("1 + 1")).unwrap())
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
        b.iter(|| sandbox.execute_json(black_box("({a:1,b:[2,3],c:{d:4}})")).unwrap())
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
                persistent_context: false,
                ..Default::default()
            };
            let mut sandbox = V8Sandbox::with_config(config).unwrap();
            sandbox.execute("42").unwrap()
        })
    });
}

/// 基准 7: ES Module 简单导出执行。
fn bench_es_module_simple(c: &mut Criterion) {
    let mut sandbox = EsModuleSandbox::new().unwrap();
    c.bench_function("es_module_simple_export", |b| {
        b.iter(|| {
            sandbox
                .execute_module(black_box("export const x = 42; export default x * 2;"), None)
                .unwrap()
        })
    });
}

/// 基准 8: ES Module 带依赖导入。
fn bench_es_module_with_deps(c: &mut Criterion) {
    let mut sandbox = EsModuleSandbox::new().unwrap();
    sandbox.register_module("./math.js", "export const PI = 3.14; export const E = 2.72;");
    c.bench_function("es_module_with_import", |b| {
        b.iter(|| {
            sandbox
                .execute_module(black_box("import { PI } from './math.js'; export default PI;"), None)
                .unwrap()
        })
    });
}

/// 基准 9: Worker 创建和终止。
fn bench_worker_lifecycle(c: &mut Criterion) {
    c.bench_function("worker_create_terminate", |b| {
        b.iter(|| {
            let mut worker = WorkerRuntime::new("var x = 1;", SandboxConfig::default()).unwrap();
            worker.terminate();
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
    bench_es_module_simple,
    bench_es_module_with_deps,
    bench_worker_lifecycle,
);
criterion_main!(benches);

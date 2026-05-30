//! WASM 沙箱性能基准测试。

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use zero_wasm_sandbox::{WasmSandbox, WasmValue};

/// 辅助函数：编译 WAT 文本为 WASM 字节
fn wat_to_wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).expect("invalid WAT")
}

/// 基准：WASM 模块编译
fn bench_module_compile(c: &mut Criterion) {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.add)
        )"#,
    );

    c.bench_function("wasm_module_compile", |b| {
        b.iter(|| {
            let _ = black_box(sandbox.compile(&wasm));
        })
    });
}

/// 基准：WASM 模块实例化
fn bench_module_instantiate(c: &mut Criterion) {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.add)
        )"#,
    );
    let module = sandbox.compile(&wasm).unwrap();

    c.bench_function("wasm_module_instantiate", |b| {
        b.iter(|| {
            let _ = black_box(module.instantiate(&sandbox));
        })
    });
}

/// 基准：WASM 函数调用（i32 加法）
fn bench_function_call_i32(c: &mut Criterion) {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.add)
        )"#,
    );
    let module = sandbox.compile(&wasm).unwrap();
    let mut instance = module.instantiate(&sandbox).unwrap();

    c.bench_function("wasm_call_add_i32_1000", |b| {
        b.iter(|| {
            for i in 0..1000u32 {
                let _ =
                    black_box(instance.call("add", &[WasmValue::I32(i as i32), WasmValue::I32(1)]));
            }
        })
    });
}

/// 基准：WASM 递归函数（阶乘）
fn bench_recursive_factorial(c: &mut Criterion) {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (func $fac (export "factorial") (param i32) (result i32)
                local.get 0
                i32.eqz
                if (result i32) i32.const 1
                else
                    local.get 0
                    local.get 0 i32.const 1 i32.sub
                    call $fac
                    i32.mul
                end)
        )"#,
    );
    let module = sandbox.compile(&wasm).unwrap();
    let mut instance = module.instantiate(&sandbox).unwrap();

    c.bench_function("wasm_factorial_100_calls", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = black_box(instance.call("factorial", &[WasmValue::I32(10)]));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_module_compile,
    bench_module_instantiate,
    bench_function_call_i32,
    bench_recursive_factorial,
);
criterion_main!(benches);

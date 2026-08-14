# ZeroWeb WASM Sandbox (`zero-wasm-sandbox`)

> WASM 沙箱运行时，wasmi / wasmtime 双后端，用于插件、扩展和受控计算任务

## 概述

`ZeroWeb WASM Sandbox` (`zero-wasm-sandbox`) 是 ZeroWeb 的 WASM 运行时模块，承担非页面侧的 WebAssembly 执行职责，为插件系统、扩展能力或受控计算任务提供安全的沙箱环境。支持两种后端：

- **wasmi**（默认 feature）— 纯 Rust 解释器，适用于插件和扩展
- **wasmtime**（可选 feature）— JIT 编译器，适用于页面级 WASM 执行

两者都启用时使用 wasmtime 后端（JIT 性能更优）；两者都未启用时退化为占位实现（`stub_backend`），确保编译通过。

## 主要功能

- WASM 模块编译与实例化（wasmi / wasmtime 统一 API）
- 导出函数调用，支持 i32/i64/f32/f64 四种值类型
- 线性内存读写，支持边界检查
- 导出项查询（函数、内存是否存在）
- 统一的 `WasmError` 错误类型
- feature gate：`default = ["wasmi"]`，`wasmi` / `wasmtime` 可选，均未启用时走 stub

## 使用示例

```rust
use zero_wasm_sandbox::{WasmSandbox, WasmValue};

let sandbox = WasmSandbox::new();

// 编译 WASM 模块
let wasm_bytes: &[u8] = /* 你的 .wasm 文件内容 */;
let module = sandbox.compile(wasm_bytes).expect("编译失败");

// 实例化并调用导出函数
let mut instance = module.instantiate(&sandbox).expect("实例化失败");
let result = instance.call("add", &[WasmValue::I32(10), WasmValue::I32(20)])
    .expect("调用失败");

// 读写线性内存
instance.write_memory("memory", 0, b"hello").unwrap();
let data = instance.read_memory("memory", 0, 5).unwrap();
assert_eq!(&data, b"hello");
```

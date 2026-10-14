# 非页面 WASM 沙箱与页面 WASM 的边界

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

区分可嵌入的 `WasmSandbox`、WebView `execute_script_with_dom` 自动桥接与页面引擎原生 WebAssembly。crate 用途说明和旧“页面级”注释不足以确定消费者，必须沿具体执行入口判断。

### User Task Navigation

| 入口 | 核对位置及边界 |
| --- | --- |
| Rust 直接 WASM 执行 | `wasm-sandbox/src/lib.rs`、后端与 `tests/module_instance.rs` |
| WebView 自动桥接 | `webview.rs::execute_script_with_dom` / `process_wasm_bridge`，`tests/wasm_bridge.rs` 钉结果、导入与错误 |
| 页面 WASM WPT | `testharness.rs::run_wasm_cases` → `run_testharness_html_inner` 的页面 execute_script；未经过上述自动桥接入口 |
| 需求与 feature 分支 | `docs/specs/page-wasm-host-function-spec-rfc.md`，[脚本 feature](../runtime/conditions/script-features.md) |

## Gotchas
- 默认后端为 wasmi，wasmtime 是显式 feature，双 feature 由 lib 门控选择；没有后端仍有 stub API 但返回 Unsupported，不能把编译成功当执行能力存在。 (`crates/wasm-sandbox/Cargo.toml`, `crates/wasm-sandbox/src/lib.rs`, `crates/wasm-sandbox/src/stub_backend.rs`)
- WebView `execute_script_with_dom` 的 polyfill 会覆盖原生 WebAssembly 为桥对象，已带 `_modules` 才幂等复用；普通页面 `execute_script`/WPT 不走此覆盖入口，V8 页面原生能力与该桥接测试是两条面。 (`crates/engine/src/dom_bridge.rs`, `crates/webview/src/webview.rs`, `tests/wpt-runner/src/testharness.rs`)
- process_wasm_bridge 先处理导出调用队列再消费 `_pendingBridge`，识别 compile/instantiate 前缀；桥接不是在 JS 调用瞬间直接同步完成，不能把普通 JS 返回值当 WASM 已实例化。 (`crates/webview/src/webview.rs`, `crates/engine/src/dom_bridge.rs`)
- JS importObject 只为函数导入建 linker，签名从模块 import_signatures 读取；非函数导入不会靠 JS 对象形状隐式补齐，缺失/不匹配实例化错误映射 LinkError。 (`crates/webview/src/webview.rs`, `crates/wasm-sandbox/src/types.rs`)
- HostFunction 的 results_out 初始为空，回调必须按声明顺序 push，索引赋值会吞掉结果或越界；多结果数量不足在桥层显式报错。 (`crates/webview/src/webview.rs`, `crates/wasm-sandbox/src/wasmi_backend.rs`, `crates/wasm-sandbox/src/wasmtime_backend.rs`)
- WASM→JS 导入使用同线程重入作用域守卫，覆盖实例化/start 期间调用并在错误注入前结束；否则重借 js_sandbox 或跨线程回调会破坏引擎借用与亲和。 (`crates/webview/src/webview.rs`)
- i64 wire 使用十进制字符串并在 JS 侧还原 BigInt，浮点特殊值另编组；全部经 JSON number 会丢 i64 精度，不能复用 i32 的默认转换。 (`crates/webview/src/webview.rs`, `crates/wasm-sandbox/src/types.rs`)
- bridge 的 memory.buffer 大小取真实导出内存字节数，不从最多 256 字节探测推算页数；探测长度曾让多页模块总暴露为一页。 (`crates/webview/src/webview.rs`, `crates/webview/src/tests/wasm_bridge.rs`)

## Architecture
- 纯 Rust compile/instantiate/call 的执行限制属于所选后端配置；它不会自动限制 V8 原生页面 WASM，更不能从 backend fuel 配置推断整个页面 JS 的资源上界。 (`crates/wasm-sandbox/src/lib.rs`, `crates/wasm-sandbox/src/wasmi_backend.rs`, `crates/wasm-sandbox/src/wasmtime_backend.rs`, `crates/script-sandbox/src/v8_runtime.rs`)
- 桥接实例缓存归 WebView，导出描述供 JS wrapper 类型转换；页面自动调用 `_start` 优先于 `_initialize` 的约定仅属该桥，不能推广为全部 WebAssembly.instantiate 的标准步骤。 (`crates/webview/src/webview.rs`, `crates/webview/src/tests/wasm_bridge.rs`)

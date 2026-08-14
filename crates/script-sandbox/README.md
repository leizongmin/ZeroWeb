# ZeroWeb Script Sandbox (`zero-script-sandbox`)

> JavaScript 脚本执行沙箱，通过 feature gate 选择 V8 或 QuickJS 后端引擎

## 概述

`ZeroWeb Script Sandbox` (`zero-script-sandbox`) 为 ZeroWeb 提供安全的 JavaScript 执行环境，用于运行扩展脚本、用户脚本、页面脚本与自动化脚本。通过 Rust feature gate 机制，用户可在 V8（`v8` crate）和 QuickJS（`rquickjs`）两种后端引擎之间切换，兼顾性能与嵌入体积。该 crate 位于渲染管线的脚本执行层，是浏览器核心与 JS 运行时之间的桥梁。

## 主要功能

- **`Sandbox` trait 抽象** — `V8Sandbox` 与 `QuickJSSandbox` 都实现 `Sandbox`，调用方以 `Box<dyn Sandbox>` 持有引擎无关的沙箱实例（cfg 选 V8/QuickJS，默认 `v8`）
- **脚本执行** — `execute`（返回字符串结果）与 `execute_json`（`JSON.stringify` 包装）；支持编译/运行时错误与超时（`set_timeout_ms`）
- **宿主回调** — `register_callback` 把 Rust 闭包挂为 JS 全局函数 `name(...)`，参数/返回经字符串桥；`resolve_async_callback` 支持 P1b 异步回调 resolve（V8 后端）
- **持久化 Context** — `SandboxConfig::persistent_context` 复用 V8 全局 Context，`reset_context` 清空 JS 状态
- **Dedicated Worker** — 独立线程 V8 持久上下文 + postMessage/onmessage 通道（`worker.rs` / `quickjs_worker.rs`）
- **ES Modules** — 源码转换式 import/export 支持、`import.meta.url`、链式依赖解析（`es_module.rs`）
- **P1b 原生绑定 escape-hatch** — `install_native_bindings` 在持久 V8 Context 内安装 `ObjectTemplate`/`FunctionTemplate`/accessor 等原生 DOM 绑定（仅 V8 后端，QuickJS 降级 no-op）

## 使用示例

```rust
use zero_script_sandbox::{Sandbox, V8Sandbox};

// 创建 V8 沙箱实例（默认 feature v8；quickjs feature 下用 QuickJSSandbox）
let mut sandbox = V8Sandbox::new()?;

// 执行 JavaScript 代码
let result = sandbox.execute("1 + 2")?;
println!("{}", result.value);

// 注册宿主回调（JS 侧调 hello(...) 触发 Rust 闭包）
sandbox.register_callback("hello", Box::new(|args| {
    format!("Hello, {}!", args.first().map(String::as_str).unwrap_or("world"))
}));
let result = sandbox.execute("hello('ZeroWeb')")?;
println!("{}", result.value);
```

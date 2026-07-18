# ZeroWeb Script Sandbox (`zero-script-sandbox`)

> JavaScript 脚本执行沙箱，通过 feature gate 选择 V8 或 QuickJS 后端引擎

## 概述

`ZeroWeb Script Sandbox` (`zero-script-sandbox`) 为 ZeroWeb 提供安全的 JavaScript 执行环境，用于运行扩展脚本、用户脚本和自动化脚本。通过 Rust feature gate 机制，用户可在 V8（`v8` crate）和 QuickJS（`rquickjs`）两种后端引擎之间切换，兼顾性能与嵌入体积。该 crate 位于渲染管线的脚本执行层，是浏览器核心与 JS 运行时之间的桥梁。

## 主要功能

- Feature gate 选择后端引擎：`v8`（高性能）或 `quickjs`（轻量嵌入）
- 扩展脚本、用户脚本和自动化脚本的沙箱隔离执行
- 错误处理基于 `thiserror`，提供结构化的脚本异常类型
- 与 `zero-engine` 协作，对接页面脚本生命周期

## 使用示例

```rust
use zero_script_sandbox::ScriptSandbox;

// 创建沙箱实例（需启用 v8 或 quickjs feature）
let sandbox = ScriptSandbox::new();

// 执行 JavaScript 代码
let result = sandbox.eval("1 + 2")?;
```

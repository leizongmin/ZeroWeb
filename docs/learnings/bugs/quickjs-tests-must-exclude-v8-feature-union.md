# QuickJS 专属测试必须排除 V8 feature union

**日期**: 2026-08-17
**相关模块**: `zero-webview`、workspace 测试矩阵

## 问题

QuickJS-only native DOM 测试独立运行通过，但 `make test` 的多包 QuickJS 阶段稳定失败。
失败时 `zero-webview` 同时编译了 602 个测试，独立 QuickJS-only 仅有 554 个。

## 根因

Cargo 对同一依赖执行 feature union。多包命令中的其他包通过默认依赖启用了
`zero-webview/v8`，同时顶层命令启用 `quickjs`，最终形成 `v8+quickjs`。WebView 在该组合态
选择 V8 后端，但 `#[cfg(feature = "quickjs")]` 仍会编译 QuickJS 专属断言。

## 解决

后端专属测试使用 `#[cfg(all(feature = "quickjs", not(feature = "v8")))]`，与生产 QuickJS
install/Drop 门控一致。验证必须同时覆盖 feature union 下不编译、QuickJS-only 下正常执行。

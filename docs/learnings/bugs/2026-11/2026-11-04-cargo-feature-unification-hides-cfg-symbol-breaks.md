---
date: 2026-11-04
modules: engine, 构建系统
---

# workspace 特性统一掩盖 cfg 门控符号的未门控引用断裂

## 问题描述

`make browser-build`（`cargo build --release -p zero-browser`，默认特性无 v8/quickjs）自 commit `90572cc78` 起编译失败：`rect_bridge.rs` 未门控地引用 `crate::js_dom_bridge::applied_mutations_cursor` / `MUT_DRAIN_GEN`，而这两项随 R2976 拆分落在 `#[cfg(feature = "script-runtime")]` 门控的 `callbacks` 子模块里，E0425。断裂存在约一天未被任何轮次发现。

## 根因分析

1. workspace 声明 `zero-engine = { path = ..., default-features = false }`，`zero-browser` default 仅 `windows-console`——`-p zero-browser` 单包构建解析出**无 JS 运行时**的 engine 变体。
2. 而 CI/日常验证全走 workspace 级构建或测试（renderer/page-runtime/webview 默认 `v8`），cargo 特性统一后 zero-engine 恒带 `script-runtime`，门控符号恒存在——**构建组合的多样性只在单包 `-p` 构建时暴露**。
3. 写 M3 修复时只验证了 workspace/测试入口（green），未跑过 `-p zero-browser`，未查引用目标的 cfg 门。

## 解决方案

按仓内既有门控模式（callbacks 模块同款）：`rect_bridge.rs` 的 pending 机制（kill-switch 函数、`RECT_LAYOUT_CACHE`、增强工厂）整体加 `#[cfg(feature = "script-runtime")]`，并补 `#[cfg(not(feature = "script-runtime"))]` 回落工厂（返回纯 snapshot handler——无 JS 侧样式写源，pending 尾恒空，快路径即完整语义）。共享 helper（`resolve_selector_node_cached` 等）保持未门控。

## 如何避免

- 引用门控模块的符号时，先查目标符号的 `#[cfg(...)]` 归属；给引用方加同款门控，或把符号下沉到未门控层。
- 改动有特性门的 crate 后，除了 workspace/test 入口，至少跑一次 `cargo check -p <binary-crate>`（不带特性覆盖）验证最薄特性组合——特性统一是"绿灯掩盖断裂"的高发区。
- 修共享 crate 的 no-JS 变体语义时，优先用「无 JS ⇒ 无样式写源 ⇒ 快路径即真值」论证回落行为，避免为门控新造平行为。

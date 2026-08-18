---
date: 2026-08-15
modules: scripts/test-guard.rs, Makefile（make test）, apps/browser, render-foundation 测试
---

# test-guard --compile-first 直接执行测试二进制的 cwd 语义

## 问题描述

远端提交 d2d47a1a 让 `make test` 走 `test-guard --compile-first`（先编译、再逐个
直接执行测试二进制，让 test-guard 只监管运行阶段）。此后本地 `make test` 出现
两类此前从未失败的测试错误：

1. `apps/browser/src/headless.rs` 的 welcome oracle 测试：`assets/welcome.html`
   NotFound（`Os { code: 2 }`）。
2. `render-foundation` 的 `test_ahem_font_detection`：加载到 Ahem.ttf 后
   `is_ahem(0)` 断言失败。

## 根因分析

**cwd 差异**：

- `cargo test` 原生模式：测试进程 cwd = **package root**（如 `apps/browser/`、
  `crates/render-foundation/`）。
- `test-guard --compile-first` 模式：编译产物清单出来后**逐个直接执行二进制**，
  cwd = **运行 test-guard 的目录 = workspace root**。

于是：

- welcome oracle 的相对路径 `assets/welcome.html`、`../../docs/...` 在 workspace
  root 下解析到错误位置 → NotFound。
- `load_ahem()` 的 `tests/wpt-runner/fonts/Ahem.ttf` 在 workspace root 下**恰好
  存在**（cargo 模式下反而不可达 → 测试一直 skip）——首次真实执行暴露测试自身
  的假设缺陷：`assert!(!loader.is_ahem(0))` 假设 id 0 非 Ahem，但 `load_ahem`
  是全新 loader（Ahem 即首个字体 id=0，`ahem_font_id=Some(0)`）→ 断言必然失败。

## 解决方案

1. 测试路径**不依赖 cwd**：用 `concat!(env!("CARGO_MANIFEST_DIR"), "/assets/...")`
   拼接（CLAUDE.md 路径通用化准则）。
2. 修正测试假设：删除"id 0 非 Ahem"的错误断言（`load_ahem` 场景 Ahem 就是 id 0），
   保留 `is_ahem(999)`（不存在字体不误判）覆盖原意图。

## 如何避免

- 测试读写 fixture 一律用 `CARGO_MANIFEST_DIR` 相对路径，不要用进程 cwd。
- 测试被"文件不可达"跳过时，要意识到该测试**从未真实执行过**——环境变化让它
  首次运行时，先怀疑测试自身假设而非新代码。
- test-guard 的 `--compile-first` 与 cargo 原生模式的 cwd 语义不同，涉及相对路径
  的测试在两种模式下都要能过。

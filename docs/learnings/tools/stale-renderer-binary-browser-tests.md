# browser 多进程测试 spawn 陈旧 renderer 二进制导致误判回归

**日期**：2026-08-12
**相关模块**：`apps/browser`（多进程 GUI 测试）、`apps/renderer`、`crates/protocol/src/process.rs`
**触发**：form_fixture 测试莫名失败（renderer 收不到 LoadHtml），误判为回归

## 问题描述

`cargo test -p zero-browser` 的多进程 GUI 测试（form_fixture 等）spawn `zero-renderer`
子进程（`RendererHandle::spawn` 经 `resolve_renderer_binary` 找 `target/debug/zero-renderer`）。
`zero-renderer` 是**独立 bin 包**（不在 zero-browser 的依赖树里）——`cargo test -p zero-browser`
**不会重编它**。改 renderer 代码后不重编，测试 spawn 的是陈旧二进制：
- 行为与当前 browser/协议不匹配 → form 页面加载失败（LoadHtml 静默丢弃/处理异常）
- 症状伪装成回归（测试稳定失败，排查方向全错）

## 根因分析

- `RendererHandle::spawn`（crates/protocol/src/process.rs:82）用 `resolve_renderer_binary`
  找二进制（当前 exe 上溯 target/debug 目录）——测试进程的 current_exe 是
  `target/debug/deps/zero_browser-xxx` → 找到 `target/debug/zero-renderer`（陈旧）。
- 排查陷阱：向 renderer 加 stdin 日志后测试仍无输出——因为 spawn 的二进制没包含日志
  （`cargo build -p zero-renderer` 后日志才出现，测试即通过）。

## 解决方案

改 `apps/renderer` 代码后、跑 browser 多进程测试前，先：
```bash
cargo build -p zero-renderer
```
（compositor 同理：`cargo build -p zero-compositor`——frame_flow 测试 spawn 它。）

## 如何复用

- browser 多进程测试失败时，先 `cargo build -p zero-renderer -p zero-compositor` 再复现，
  排除陈旧二进制因素再排查代码。
- 任何「spawn 独立 bin 的测试」都有此坑（frame_flow spawn zero-compositor 同款）。

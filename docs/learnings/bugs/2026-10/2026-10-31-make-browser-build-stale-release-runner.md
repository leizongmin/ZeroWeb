---
date: 2026-10-31
modules: layout-engine, wpt-runner
---

# make browser-build 在 Linux 是 powershell 假入口——release runner 静默陈旧

## 问题描述

R5366 轮排查 containing-block-020 时，修改 `crates/layout-engine/src/engine/postprocess.rs` 后执行
`make browser-build` 再跑 `./target/release/zero-wpt-runner reftest-upstream`，行为毫无变化；
连 `LAYOUT_DUMP` 布局树都维持旧几何，一度误判为「修复未生效、有其他 pass 覆盖」，多花两轮探针排查。

## 根因分析

Makefile 的 `browser-build` 目标在 Linux 分支执行的是
`powershell -NoProfile -ExecutionPolicy Bypass -File scripts\browser.ps1 -BuildOnly`——
Windows 专用入口，Linux 无 powershell 时静默失败/无操作，**不构建任何 Rust 产物**。
而 reftest 跑的是 `target/release/zero-wpt-runner`，其时间戳停在改动之前（16:11 vs 编辑 16:2x），
形成「改了代码 → 跑了构建 → 测了旧二进制」的静默陈旧链。stash A/B 单案复核若依赖同一假入口，
同样存在测错侧的风险（全量 corpus 走 `make reftest-upstream` 依赖
`zero-wpt-runner-release` 目标，不受影响——这是此前 A/B 结论仍正确的唯一原因）。

## 解决方案

- 重建 release runner 一律走 `make zero-wpt-runner-release`（= `cargo build --release --bin zero-wpt-runner`），
  或 `cargo build -p zero-wpt-runner --release`。
- 判据口诀：**改完代码跑 reftest 前看一眼 `ls -la target/release/zero-wpt-runner` 时间戳**，
  或干脆养成 `make zero-wpt-runner-release && make reftest-upstream FILTER=...` 的组合习惯。
- `make browser-build` / `make browser` 仅在 Windows 分支可用；Linux 起浏览器用 `make browser`
  的 Linux 分支（如存在），构建验证用 cargo 目标。

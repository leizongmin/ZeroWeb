# ZeroWeb WPT Runner (`zero-wpt-runner`)

> WPT 测试运行器 — 加载与执行 Web Platform Tests（WPT）、reftest 渲染对比与兼容性工具

## 概述

`ZeroWeb WPT Runner` (`zero-wpt-runner`) 是 ZeroWeb 的规范兼容性测试工具集：加载和执行 Web Platform Tests（WPT）测试用例，运行 WPT reftest（渲染对比测试）与上游 wpt-data 的 reftest，并提供 layout-dump 布局对比、chromium oracle 像素对比、结构扫描、产品静态 fixture 渲染与页面级性能基准等兼容性工具。它是 `make reftest` 的执行入口，导入的上游测试记录在 `imported-tests.txt` / `imported-testharness.txt` / `imported-resources.txt` 账本中。

## 主要功能

- **WPT testharness** — `run [filter]` 执行内置 testharness 用例（可选分类/模式过滤），`list` 列出用例，`summary` 仅输出汇总；`--json` / `--tap` / `--junit` 输出格式
- **专项 testharness 套件** — `testharness-html`（media/forms/focus/input-event）、`testharness-canvas`（Canvas 2D M1）、`testharness-dom`（js-dom M4 / DC-3）
- **reftest** — `reftest` 运行常驻 reftest 断言集（`reftest-skip-list.txt` / `reftest-smoke.txt` 分层控制）；`reftest-upstream` 运行上游 wpt-data reftest；`--media print|screen` 切换渲染媒体
- **布局对比** — `layout-dump [filter]` 对上游测试页 dump 布局树与 golden 对比（`layout-golden/`，配套 `../../scripts/run-layout-golden.sh`）
- **oracle 对比** — `reftest-oracle [filter]` 渲染上游测试页与 chromium oracle 截图对比（`oracle-shots/`），`product-smoke <html>` 渲染产品 fixture 到 CPU PNG 对比；`compare-png` 像素级对比（max-diff / channel-diff / pixel-radius）
- **结构扫描** — `struct-sweep [filter]` 对上游测试页做 sibling-overlap 结构检查
- **性能基准** — `perf` 页面级性能基准（perf-gate 场景，`--scenario id:path` 可重复执行）
- **并行执行** — rayon 并行测试作业（`--jobs`，默认 min(CPU-1, 8)）；外部 WPT MANIFEST.json 支持（`--manifest`）

## 运行方式

```bash
# reftest（release + test-guard 包裹；等价于 make reftest）
cargo run --release --bin zero-wpt-runner -- reftest

# 运行指定分类的 testharness 用例
cargo run --bin zero-wpt-runner -- run forms

# 上游 reftest（wpt-data 目录）
cargo run --release --bin zero-wpt-runner -- reftest-upstream

# layout dump 对比（仓库根 scripts/ 目录）
../../scripts/run-layout-golden.sh

# 产品 fixture 渲染 + oracle 像素对比
cargo run --release --bin zero-wpt-runner -- product-smoke res/xx.html --oracle oracle.png --out out.png

# 性能基准
cargo run --release --bin zero-wpt-runner -- perf --scenario home:path/to/page.html
```

## 相关文档

- `make reftest` 入口与测试分层：`docs/rally/run-rules.md`
- 导入测试账本：`imported-tests.txt` / `imported-testharness.txt` / `imported-resources.txt`

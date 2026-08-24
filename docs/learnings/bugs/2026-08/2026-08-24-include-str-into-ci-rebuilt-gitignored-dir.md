---
date: 2026-08-24
modules: zero-engine,zero-wpt-runner,ci
---

# include_str! 引用 gitignored 且被 CI 重建的目录导致 clippy 全平台失败

## 问题描述

`crates/engine/src/js_dom_bridge_tests/part21.rs`（js-dom R216 起）用
`include_str!("../../../../tests/wpt-runner/wpt-data/dom/common.js")` 等编译期嵌入 WPT
dom 子集文件。`make fetch-wpt-data`（CI 每个 job 都会执行）会 `rm -rf` 并重新 clone
`tests/wpt-runner/wpt-data/`（整体 gitignored，来自 zeroweb-wpt-data v1.10 仓库），该仓库
**不含 `dom/` 子集**（`dom/` 由独立的本地流程 `make fetch-wpt-dom` 拉取）。结果：
CI 的 `cargo clippy --all-targets` 在全部 v8 平台（linux x2 / macos x2 / windows x2）
编译期找不到文件直接失败；quickjs 矩阵因测试模块 `#[cfg(feature = "v8")]` 门控不受影响。
本机开发环境因曾手动 fetch 过 `dom/` 而未暴露（或有 v8 缓存），CI 全平台红态才显现。

## 根因分析

1. `include_str!` 是**编译期**依赖——文件必须在编译时存在，且 CI 每次 checkout 是干净
   工作树，任何「先拉数据再编译」的步骤缺失都会失败。
2. `tests/wpt-runner/wpt-data/` 整体 gitignored，且 `make fetch-wpt-data` 的目标设计为
   「reftest 数据源」，每次执行都 `rm -rf` 重建——**即使把文件 git add -f 进该目录，
   也会被 CI 的 fetch 步骤删掉**（`.cache-storage-window-root` 能存活是因为它在
   fetch 数据源的 clone 内容里，而非靠 git 跟踪）。
3. 「本地能编译」假象：本地开发机跑过 `make fetch-wpt-dom` 后目录齐全，clippy/test 都
   正常；CI 没有这个步骤，于是只有 CI 红。

## 解决方案

按仓库既有 vendor 先例（`crates/engine/tests/fixtures/dompurify.js`、
`wpt-data/.cache-storage-window-root`），把编译期依赖的文件**vendor 进 git 跟踪的目录**：

1. 以 `fetch-dom-subset.sh` 的 `WPT_REV` pin 从上游 WPT 拉取 `dom/common.js` 与
   `dom/ranges/Range-test-iframe.html`，放入 `crates/engine/tests/fixtures/wpt-dom/`。
2. `part21.rs` 的 5 处 `include_str!` 改指新路径；fixtures README 记录来源/许可证/更新方式
   （先更新 fetch 脚本的 WPT_REV 再同步 vendor 文件，保持同 rev）。
3. 本地验证：`cargo check -p zero-engine --tests` + workspace clippy `-D warnings` +
   相关测试 + `make test`（Xvfb）。

## 如何避免

- **铁律**：`include_str!` / `include_bytes!` / build.rs 等编译期路径只能指向
  **git 跟踪的仓库内文件**（或 Cargo 依赖），不得指向 gitignored、按需 fetch、
  或 CI 会重建的目录。
- 新增编译期测试资产时对照本仓两条既有路径：小文件 vendor 到
  `crates/engine/tests/fixtures/`；大套件由 wpt-runner 运行时读取（非 include_str）。
- 涉及「fetch 后编译」的新步骤，先在 CI 等价环境（干净 checkout + 仅跑 fetch 步骤）
  验证，再提交。

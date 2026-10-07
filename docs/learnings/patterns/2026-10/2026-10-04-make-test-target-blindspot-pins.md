---
date: 2026-10-04
modules: apps/renderer, apps/browser, tests/wpt-runner
---

# make test 锚算术对 zero-renderer/zero-browser 测试结构性失明——回归钉的落点必须核对 target 归属

## 问题描述

slice27（2026-10-04）在 `apps/renderer/src/js_worker.rs`（属 zero-renderer **lib** target，见 `src/lib.rs` 的 `pub mod js_worker`）新增回归钉 `renderer_js_worker_named_access_registers_after_snapshot_s27`，交付时汇报「make test 19614P 全绿」作为门禁证据。事后核查发现：该钉从未在 make test 中运行过——门禁日志（68 个 `test result:` 行合计 19614P）全文没有任何 `js_worker::tests` 条目。钉的 GREEN 与 RED 判别当时均无归档证据，属门禁证据缺口。

## 根因分析

`Makefile` 的 `make test` 组合是：

1. workspace 腿：`cargo test --workspace --exclude zero-browser --exclude zero-renderer`（多进程单测需先刷新 standalone 二进制，browser/renderer 两个 member 被整体排除）；
2. renderer 专项腿：`cargo test -p zero-renderer --bin zero-renderer`（**只跑 bin target**）+ quickjs feature 变体同样只跑 `--bin`。

因此落在 zero-renderer lib target（如 `js_worker.rs`、`lib.rs` 下各模块）或 zero-browser 任意 target 的 `#[test]`，无论多少个，都不影响 make test 总计行——锚算术（如 19614=19606+4+3+1）对它们结构性失明。本次缺陷（首载 Window named access 未注册）恰落在盲区内：make test 全绿的同时活体缺陷真实存在，最终靠集成验收的活体探针才暴露。

## 解决方案

- **放钉前先核对 target 归属**：确认该 `#[test]` 所在文件被哪个 target 编译（`cargo tree -p <crate>` 或查 `Cargo.toml` 的 `[[bin]]`/`[lib]` 与 `mod` 声明链），并确认 `make test` 实际运行该 target。
- **盲区 target 的钉必须显式专项运行并归档**：如 `cargo test -p zero-renderer --lib -- <钉名过滤>`；RED→GREEN 判别（回退修复只留钉 → 钉须红）同样要在归档里可追溯。
- **策略级建议**（待定）：给 make test 增加 `-p zero-renderer --lib` 专项腿消除盲区，或约定回归钉优先放在 make test 可见的 target（如 engine lib）。

## 如何避免

门禁声明（如「make test 19614P」）只对其确实运行的目标集合成立。汇报前用门禁日志 grep 钉名验证「钉真的跑过」——日志里没有钉名 = 没跑过，与总计行数字无关。

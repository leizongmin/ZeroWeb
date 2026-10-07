---
date: 2026-10-07
modules: ci,build
---

# workspace 根 `--bin` 构建的全成员 feature 统一解析把 v8 拖进 quickjs 臂

## 问题描述

`weekly.yml` 引擎拆分矩阵（v8/quickjs 双臂）2026-08-15 首跑起连续 8 周失败：全部 6 个
quickjs 臂在 `v8 crate build.rs` panic（`copy_archive` 找不到 `.cargo/rusty_v8/archive`），
`Publish weekly release` 恒被跳过。ci.yml 全绿（单包 `-p` 检查覆盖不到该场景），
ci-watchdog 只盯 ci.yml，故障对全部自动化不可见。

## 根因分析

quickjs 臂构建命令 `cargo build --release --bin zero-browser`（不带 `-p`）在 workspace
根执行时，cargo 对**全部默认成员**做一次统一 feature 解析：`zero-wpt-runner` 的
`default = ["v8"]` 点亮 `zero-engine/v8`，统一解析产出的 zero-engine 单元携带 v8——
依赖它的 zero-browser 也链接 v8。v8 的 build.rs 依 `.cargo/config.toml` 的
`[env] RUSTY_V8_ARCHIVE` 找本地 archive；v8 臂有预下载步骤所以能过，quickjs 臂没有
该步骤即 panic。带 `-p` 的构建（ci.yml 的单包检查、`cargo tree -p`）解析范围限定到
目标包，看不到 v8——这就是"本地查无 v8、CI 必炸"的认知错位来源。

## 解决方案

三条 CI 构建命令全部加 `-p`（zero-browser / zero-renderer / `-p zero-compositor -p
zero-image-decoder`），把解析范围钉死到目标包；browser 的引擎 feature 显式传递
（`--features v8` / `--features quickjs`，默认 feature 本无引擎）。weekly.yml 与
release.yml 同步（两处命令完全相同，release 也实际失败过一次）。apps/browser 的
守卫测试 `browser_build_and_release_entries_include_compositor` 断言构建字面，命令
形态变更须同步更新。

教训：**per-engine 产物构建必须 `-p` 隔离解析范围**——Makefile/scripts 早已用
`-p zero-renderer -p zero-compositor` 拆行（627afe21d "isolate release feature
resolution"），CI workflow 是漏网之鱼。诊断同类问题时，`cargo tree -p X -i Y` 与
`cargo build --bin X` 的解析范围不同，前者看不到后者实际执行的统一解析结果。

---
date: 2026-11-06
modules: make,target-disk-guard,test-guard
---

# 并发重型 make 入口时 target-disk-guard 三级全清连坐兄弟门禁腿

## 问题描述

同一 clone 里并发跑两个重型 make 入口（`make test` enginelib 门禁腿 + `make testharness-editing-run-one` 跑批腿），enginelib 腿在 apps/media-host 真进程测试处莫名失败：`spawn production zero-media-host binary: NotFound`（CARGO_BIN_EXE 路径指向的 target/debug/zero-media-host 不存在），cargo test 默认 fail-fast 随即中止，engine lib（3,700+ tests）根本没跑到，门禁结果作废。而同刻 target/debug/ 下明明刚由 make test 自身的 prebuild `cargo build -p zero-media-host` 报告 Finished。

## 根因分析

1. 跑批腿的 make 依赖链里有 `target-disk-guard`：target/ 当时 91GB 超过 50GB 阈值，一级清理（只删 `target/*/incremental`，5GB）不够，触发三级 `rm -rf target/` 全删（清理日志落在跑批腿输出里，grep `target-disk-guard` 可见）。
2. 全删发生时 enginelib 腿已经编译完 dev 测试二进制并开始执行：已 exec 的进程靠 inode 继续跑，表象正常；但 media-host 真进程测试 spawn 的是**按路径新开**的 `target/debug/zero-media-host`，文件已被删 → spawn NotFound。
3. cargo test 对失败的测试二进制默认 fail-fast（不传 `--no-fail-fast` 时中止其余二进制），workspace 成员序靠前的 apps/media-host 一倒，靠后的 crates/zero-engine 永远轮不到。
4. 附带陷阱：cargo 对「输出产物被删但 fingerprint 仍 fresh」不会重编——prebuild 的 `cargo build -p zero-media-host` 在产物缺失时静默跳过。冷 target（全删后重建）反而没有这个坑。

## 解决方案

- 重型 make 入口按 clone **串行**执行：跑批脚本里前一条腿退出后再起下一条；不要在同 clone 并发两个会触发 target-disk-guard / cargo 构建的入口。跨 clone 并行（双流各用独立 clone）不受影响。
- 见到兄弟腿莫名 `CARGO_BIN_EXE_* NotFound` + fail-fast 早退，先 grep 该腿输出里的 `target-disk-guard` 清理日志确认是否被全清连坐，不要按环境回归排查；冷 target 重跑即恢复。
- 需要强制重建「指纹 fresh 但产物被删」的 bin：touch 其入口源文件后再 build。

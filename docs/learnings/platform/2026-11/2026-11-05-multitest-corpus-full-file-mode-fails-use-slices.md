---
date: 2026-11-05
modules: zero-web-runner,editing-compat
---

# multitest 语料 full-file 单发模式失败——必须走切片协议

## 问题描述

`tests/wpt-runner/wpt-data/editing/run/multitest.html`（10 个 variant meta，约 5,460 case / 10,092 断言行）在 zero-wpt-runner 的 `testharness-editing-run` 通道下，**整文件单发运行必然失败**：runner 以 ~100% CPU 空转 19 分钟后退出，产出 0 行断言、日志仅 ~3.5KB（构建前导 + `make: Error 1`），无任何 runner 输出。该失败形状在本仓已三次复现：fix6 时代 `fix6-multitest-full.log`/`full2.log`（2026-11-05 凌晨）、fix10 轮 `fix10-multitest.log.fullfile-failed-090414`。根因未定位（FIXME：runner 对多 variant 大文件的枚举/加载路径疑似卡死在首 variant 页处理前，test-guard 4G/8G 阈值未触发）。

## 根因分析

multitest.html 每页加载时由 `data/multitest.js` + `include/tests.js` 动态生成约 5,460 个 case 并注册为 testharness 测试。切片协议（`slice-one.sh` 把 pristine 语料裁成单 variant meta）将单页 case 数降到 ~1,000，runner 可在 ~100s 内正常完成并产出恰好 1,000/1,092 行；整文件 10 variant 同页数量级时 runner 在产出任何行之前挂死。

## 解决方案

multitest 门禁一律走 10 切片逐片协议（每片 `TIME_LIMIT=1200`，fix9/fix10 轮实证总耗时 ~19-21 分钟）：

1. 每片前 `slice-one.sh <variant>` 把语料裁成单 variant，片后从 pristine 恢复（trap 兜底）；
2. 逐片 `make testharness-editing-run-one FILTER=editing/run/multitest.html`；
3. 各片日志合并后对基线做行名元组判读（`?1-1000` 等 variant key 在两种模式下行名同形，可跨模式比较）；
4. 并行多轮环境下脚本必须加 `flock` 单实例锁——双实例同写语料文件会互踩（fix10 轮实证：两个实例同时切片+同秒双 runner 启动，双方产出全废）；
5. 与其它墙钟敏感跑批严格 solo，防止并发 cargo build 毒化计时（既有教训）。

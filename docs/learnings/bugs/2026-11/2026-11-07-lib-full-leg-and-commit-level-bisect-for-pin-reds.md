---
date: 2026-11-07
modules: layout-engine, testing
---

# 单测钉红的归因必须 bisect 到提交；改动域轮次必须跑改动 crate 的 lib 全腿

## 问题描述

`cargo test -p X` 多包腿运行中，先跑的包 lib 失败会中止后续目标（无 `--no-fail-fast`），改动 crate 的 lib 全腿因此常被跳过。连续三轮叠加后果：

1. R5436 落 beside-shrink 臂（6acbeea9a）当轮翻红 r1721/r5213 两钉——该轮门禁只跑了新钉 + engine 腿，未察觉；
2. R5441 同模式（4 新钉绿、lib 双钉红未察觉）；
3. R5442 发现双钉红后，仅做「stash 后 HEAD 复核同败」——这只证明**HEAD 时已存在**，不证明引入点在 HEAD——遂误归因 R5441 并写入控制面，下轮才用提交级 bisect 勘误（引入点实为 R5436）。

## 根因分析

- lib 全腿（2400+ 测试）比新钉慢得多，轮次门禁倾向只跑新钉与受影响腿；但 lib 内既有钉正是既有语义的守卫，改动触碰到它们的判据域时必然翻红，跳过即漏检。
- 「HEAD 存在性复核」与「引入点归因」是两件事：存在性复核只圈定 `≤HEAD`，不定位提交；不 bisect 就把最近一次改动域相同的提交当引入点，是归因先验替代证据。
- kill-switch 单变量定界（`ZW_TABLE_FLOAT_BESIDE_SHRINK=0` 双钉复绿）比 checkout bisect 便宜一个量级，应作为第一步。

## 解决方案

- 改动了某 crate 的轮次，门禁必须包含该 crate 的 **lib 全腿**（test-guard 包裹；多包腿加 `--no-fail-fast` 防中止）。
- 钉红归因固定顺序：①先翻 kill-switch 定域（秒级）②`git checkout <suspect-commit>` 复跑 bisect 到具体提交 ③才写控制面归因；「stash HEAD 同败」只能表述为「HEAD 既有」，不得直接写「X 遗留/引入」。

## 证据

docs/goal/rendering-compat/evidence/r5443-beside-table-pin-oracle-2026-11-07.md（bisect 链 + chromium oracle 定谳 + 改钉）。

---
date: 2026-11-10
modules: zero-wpt-runner,wptfyi-runtime
---

# weekly coverage 快照组级摆动的产物法证——页超时截断三征定谳

## 问题描述

weekly `wptfyi-coverage` 快照的组级 `zw_passes` 在两次周跑之间会无引擎
改动地摆动：2026-11-09 两次 run（`a73228e22` 16:12Z cron → `4482feb56`
19:16Z）间 shadow-dom/declarative −491、editing/run −651、
css-writing-modes −1、dom/ranges +37，而 HEAD 差内引擎提交无一可达这些
面。若按「组级回退 → 查引擎回归」走，渲染/editing 流会被无债挂账
（DC-3 evidence 第四节初版即如此误挂）。

## 根因分析

runner 对大文件以 testharness 页超时收尾（message "testharness page
timeout completed the file with N test(s) unreported"）：页超时预算内跑
多少子测试算多少，未跑完的尾部不产出结果。GitHub hosted runner 速度
run 间方差（实测 html-sem job 26:01 → 28:38）直接移动截断点 → 同文件
子测试数摆动 → 组级账面波动。这是环境方差，不是行为翻转；与
7a5e6ecd9 在册的 r5270↔r5296 env 竞态、11-08 shadow-dom 掉零前科同根。

## 解决方案：截断三征 + 产物对拍法证流程

组级摆动先做产物级定谳再挂账，流程：

1. **对拍产物**：`gh run download <run-id> -R <repo> -n zw-results-<分片>`
   取两 run 同名 artifact，逐文件比子测试数与状态。先 `md5sum` 全量
   互拍——同 run 兄弟产物逐字节相同（本例 html-semantics md5 一致）
   即基本排除引擎面。
2. **截断三征**（全部满足 → env 方差）：
   - 子测试总数变少（或变多），Pass 数同落同涨（无单点翻转）；
   - 前缀状态逐位一致，唯一差异位恰为文件级 Timeout 条目本身；
   - 尾部条目 message 命中 "testharness page timeout … unreported"。
3. **可达性核对**：HEAD 差的引擎提交对摆动面无路径可达，作独立旁证
   而非主证（速度方差不需要引擎改动就能摆动）。

两个易混点：

- universe 外结果不进组账面：`zz-probe-*` 探针页（zw_only）原始
  pass 数大幅变化（−651）不影响快照组差，只能作 runner 方差佐证。
- 聚合口径本身会移动组间增幅（`wptfyi_gap.py` 分片收录映射
  3c4cd6772：wpt.fyi 对超大文件按 `?N-M` 分片收录、ZW 整文件结果
  此前落 zw_only，纳入后分子大增而分母口径零变化）——跨脚本版本比
  快照时先分离「记账修正」与「真实 pass 变化」（本例 +51,637 总差中
  ~+49,465 为收录修正，产物级仅 +2,172）。

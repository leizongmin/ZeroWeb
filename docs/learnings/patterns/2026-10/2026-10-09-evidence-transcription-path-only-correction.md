---
date: 2026-10-09
modules: .agents/skills/zeroweb-site-optimizer
---

# 已落账证据的不可变性与纯路径转写勘误规程

## 问题描述

网站任务验收流中，审查角色产出的 role JSON（记录报告文件与其 artifacts 的路径+SHA-256）在落账后需要修正引用路径。这些 JSON 的哈希已被记进 `workflow.json` 的 op.result 和审查汇总回执，直接重写会破坏「已哈希入账的证据不可变」约定，让已完成的操作变成不可验证的历史。slice48 运行中两份 role JSON（`pr119-defect-role.json`/`pr119-testeff-role.json`）就因内部 artifacts 引用了裸文件名（未带运行根相对前缀），在证据归档到 `archive/` 子目录后哈希全部对不上。

## 根因分析

两层原因叠加：

1. **创作期未约束路径形态**：审查 agent 写 role JSON 时以裸文件名引用同目录 artifacts，事后证据挪位（归档）即失配。
2. **账本不可变与勘误需求冲突**：哈希入账后文件不能动，但路径错误又必须修——缺一个不破坏证据身份的修正规程。

## 解决方案

转写勘误五步（slice48 rev465 实操验证）：

1. **失证原件字节级重建取证**：从归档前快照或截断点复原原件，对复原文件重算全文件 SHA-256，与账本记录哈希比对——**全哈希命中即证明重建件与原落账件字节同一**（slice48 两件：pre-mutation 快照 = archive 件去 INVALID 头；readcode md = 截断于追加更正块前）。
2. **纯路径修正**：role JSON 只改 artifacts 的 `path` 字段与补充说明字段，所有 evidence `sha256` 保持原值不动。
3. **原件归档**：修正前原件存 `review/legacy/`，保全过程可溯。
4. **账本同步**：`workflow.json` 与 prev 快照在同一修订内 patch 受影响 op.result 的哈希（否则 load(prev) 的 checkpoint_ref/evidence 校验失败）；被替换的 workflow 文件归档为 `workflow-legacy-revNNN.json`。
5. **run.md 记录**：转写动机、重建取证方式、哈希对账结论全部落账。

避免：审查角色产出 JSON 时就使用运行根相对路径；落账前先跑校验器实跑一遍，见同日 `../../bugs/2026-10/2026-10-09-lazy-validator-first-binding-exposure.md`。

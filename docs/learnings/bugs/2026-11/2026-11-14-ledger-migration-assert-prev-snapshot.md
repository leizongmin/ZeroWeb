---
date: 2026-11-14
modules: site_optimizer
---

# 账本 migration rev 的「intended 恰 N」断言必须查 prev 快照——连撞六撞后固化

## 问题描述

账本重放脚本的 verify 段对 migration rev（intent→completed 同段脚本）写前提
断言时，多次把「migration 前 intended op 恰 N 个」查到了**当前 completed 快照**
上——该快照里 intended 已被迁移为 completed，断言恒空，首跑即炸或更糟：
quietly 通过。T68 链内 rev1129、rev1131、rev1133、rev1138 连续四撞（前两撞
被修正模板救回，后两撞换汤未换药），加上更早两撞共六撞。

## 根因分析

模板复制时变量指向错了快照：migration rev 的「migration 前」状态只存在于
**prev 快照**（即 intent 所在的那个 rev 的落盘档），当前快照已经承载迁移后的
状态。断言查错对象不是笔误，而是模板语义没有把「时间轴」写进变量名——
`a`/`b`/`c` 这种字母命名让 prev/current 的指向全靠记忆。

## 解决方案

1. 模板固化：migration rev 的 intended 前提断言一律从 prev 快照（intent 所在
   rev）查；变量命名带语义（`prev_intended`，而非复用当前 rev 的列表变量）。
2. 炸断后的处置纪律：**先手工核验已落盘快照是否合法**（快照无恙、断言错），
   再 Edit 修断言，verify-only 重跑（importlib 加载调 verify，不重写落盘档）。
   编译式重跑会撞 `assert not exists` 或重写时间戳。
3. 打印行也算函数链一环：verify 的 f-string 引用不存在于 prev 快照的任务 ID
   会 IndexError——账本脚本「定义≠调用」自检要覆盖打印路径（done 前快照抽检，
   不信 exit 0）。

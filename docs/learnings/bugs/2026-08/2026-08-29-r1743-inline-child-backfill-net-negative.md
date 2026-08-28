---
date: 2026-08-29
modules: layout-engine,rendering-compat
---

# R1743 父高回填纳入 inline 子底边两次判据收窄均 net-negative（014 负结果记录）

## 问题描述

line-clamp-014（`.block > span.clamp` 5 行文本，ref 期望 `.block` 高 160px）暴露：
span 被 taffy 提升为独立子盒后，R1743 父高回填只累计 **block-level** 子盒底边，
inline 子盒（自身持 160px IFC）不计入 → `.block` 停在 strut 32px，黄底只盖 1 行。

直觉修法是把 R1743 fold 的 `child.is_block_level` 条件放宽纳入 inline 子。两次
判据收窄实现后全量 corpus A/B 均 net≤0，已全部 revert。

## 根因分析

- **尝试 1**：判据 = 父 `inline_layout` 为空/缺失（"父无自有行盒 → inline 子自包含"）。
  **不成立**：非 pure-Ahem 容器全部不存 `inline_layout`（R84 store gate），条件几乎
  恒真 → 29 案回归（quotes/counter/ruby/contain/generated-content 全族，净 -18）。
- **尝试 2**：判据 = 父无直接 DOM Text 子。仍 5 案回归（`after/before-inheritable-002`、
  `line-height-205`、`ruby-dynamic-insertion-005`、`word-break-break-all-062`，净 0）——
  这些父也无直接文本子，但其 inline 子底边不应计入（generated content 行、ruby 注音、
  断词残段等由其他 pass 管）。

根本困难：`has_inline_content` measure 路径把 inline 子文本**重复**归入父 IFC 与
子盒两处（同一文本既撑父 strut 测量又被 span 盒自己 remeasure），"inline 子是否
自包含 IFC"无法从 LayoutBox 局部信息（有无文本子/inline_layout）可靠判定——
需要 taffy 测量宽（max-content 252 vs 父宽 784）与最终 IFC 行数的差值信号。

## 解决方案

revert，保持 R1743 只认 block-level 子。line-clamp-014 保持红（21.06%），记为
**已知结构残案**：正确修法应改 R1024 leaf 判据的 `has_text_child`（把「唯一元素子
为 inline 且无元素孙」的容器也作 leaf → span 文本回归父 IFC → 父高自然正确），
但该判据经多轮 A/B 调优（R1024/R1025/R1494），任何放宽须按惯例做全量 10 目录
A/B + welcome/legacy 字节对比后才可落地，不宜夹带。

## 如何避免

- 放宽「权威高度来源」类 fold 的成员资格前，先用**全量 corpus**（而非目标簇）验证：
  inline 子出现在 quote/marker/ruby/generated-content 等十余种结构里，单簇绿色
  完全不表征风险面。
- `inline_layout` 的**存在性**不能当「父是否拥有行内容」判据——R84 store gate 使
  非 Ahem 容器恒为 None；结构判据要用 DOM 形状（直接文本子、元素子类型）。
- 负结果同样值得落档：本记录可防止后续轮次重走「inline 子计入父高」两次弯路。

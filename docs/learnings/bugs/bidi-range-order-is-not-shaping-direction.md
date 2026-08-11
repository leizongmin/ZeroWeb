# BiDi range 顺序不能替代 shaping direction

- 日期：2026-08-11
- 相关模块：`layout-engine/inline`、`engine/paint`

## 问题描述

为了让 LTR 容器内的 RTL fragment 使用逻辑文本 shaping，曾尝试把
visual-to-logical byte range 全降序视为 RTL direction。

## 根因分析

range 降序只说明视觉字符与逻辑源码顺序相反，不等于该 fragment 可以作为单一 RTL
shaping run。CSS `unicode-bidi: bidi-override` 可让 Latin、Hebrew 或拆分后的单个 script
fragment 呈降序，但真正的 shaping direction 由 Unicode BiDi Algorithm 的 resolved
embedding level 决定。

81 个 Chromium Oracle 案 A/B 中，range-order 覆盖使 rounded diff sum 从 `133.75`
回归到 `133.84`；加入脚本初筛后仍回归到 `133.86`。

## 解决方案

撤回 range-order direction 覆盖。mixed script、数字和 bidi 控制码先按
`unicode_bidi::BidiClass` fail-closed；后续若需 per-fragment direction，必须在
`visual_runs()` 阶段保存 resolved level，并随 visual-to-logical map 一起贯通到 paint。

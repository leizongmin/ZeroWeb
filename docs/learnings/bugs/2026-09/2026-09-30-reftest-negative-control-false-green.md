---
date: 2026-09-30
modules: wpt-runner,engine,layout-engine
---

# reftest 负控制假绿：断言「不绘制」的最小复现须把泄漏信号放大过分类容差

## 问题描述

为 display:none 文本泄漏绘制修复（paint 趟整子树跳过 + Path B 注入 `display_none_walk_nodes`）补了本地 reftest 资产（`css-display/none-text-not-painted`）。首版 test 页隐藏文本共 21 字符（`LEAK-BLOCK`/`LEAK-INLINE`）。修复态 PASS 后做负控制（stash 修复重跑）预期 FAIL，实际仍 PASS（Failed: 0）——reftest 恒真，无回归捕获能力。

## 根因分析

1. 本地 `InlineReftestDef` 无 per-case 阈值：`fuzzy_override` 仅来自 WPT MANIFEST.json，本地条目走 `ReftestCategory` 默认容差——Layout 类 `default_max_diff_ratio` = **1%**（reftest.rs:86）。
2. 泄漏信号量级估算缺失：21 字符隐藏文本在 800×600 视口泄漏墨迹仅 0.12%（576px），远低于 1% 容差 → pre-fix 也 PASS。
3. 首轮负控制输出被 grep 过滤只看了 Failed/Pass Rate，掩盖了 `✓ id (0.12%)` 里的关键信息——先看逐例 diff 率再看汇总，是负控制的基本动作。

## 解决方案

- **放大信号而不是放宽阈值**：本地条目无法覆盖阈值（改全局阈值会波及全 suite）。把隐藏文本加长到 ~700 字符（泄漏 2.67% = 12,796px）。注意 Path B 空 styles 重跑 IFC 按 UA 默认字号布局，CSS font-size 对泄漏墨迹无效，只能加字量。
- **双向实测定裕度**：post-fix 0.00% PASS / pre-fix 2.67% FAIL，对 1% 阈值裕度 2.67×。中间版（350 字符，1.33%）裕度不足已弃——跨平台字体回退差异可吃掉 33% 裕度。
- **负控制流程**：stash 修复 → `cargo build --release` 显式重建（确认 Compiling 行含受影响 crate）→ 全量输出跑 reftest → 确认 Total=1 且逐例 diff 率 > 容差 → stash pop。

## 如何避免

凡断言「不绘制/不可见/不渲染」的 reftest，泄漏信号默认是不可见的小墨迹——写完先做负控制实测 diff 率，与分类默认容差对比；diff 率不足时放大测试页信号量（加字量/加大色块），留 ≥2× 裕度。负控制输出看逐例 diff 率，不看过滤后的汇总行。

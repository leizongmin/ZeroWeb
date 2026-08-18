---
date: 2026-08-14
modules: zero-layout-engine::inline, zero-engine::paint
---

# Synthetic small-caps 需要在断行前展开

## 问题描述

`small-caps-letter-spacing-001` 的 self-source 差异为 `0.13%`，Chromium Oracle 差异为 `1.85%`。用例使用不含 OpenType `smcp` 的 Ahem，并要求 `ß` 合成 small-caps 后展开为 `SS`，两个 glyph之间仍应用 letter-spacing。

## 根因分析

ZeroWeb 目前只为 `font-variant-caps:small-caps` 注入 OpenType `smcp`。字体缺少该 feature时不会执行 synthetic small-caps，也不会发生 `ß→SS` 扩展。

该缺口不是 paint 侧多加一次 spacing即可修复。case expansion会改变字符数、advance、断行位置和 source range；若只在 paint阶段处理，layout宽度与 selection mapping都会失真。

## 解决方案

synthetic small-caps 必须在 inline line breaking之前完成 case expansion和字号缩放，并保留一对多 source mapping。layout、shaping与paint应消费同一展开结果。

在这条共享契约完成前，不要为 Ahem或 `ß` 添加专用 paint补丁。验收应同时覆盖该 WPT、换行边界、selection source range和 Chromium Oracle。

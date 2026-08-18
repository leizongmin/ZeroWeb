---
date: 2026-08-13
modules: zero-layout-engine, zero-wpt-runner
---

# Nonspacing mark 不应贡献独立 layout advance

## 问题描述

Arabic `NBSP + combining marks` 经 shaping 后总 advance 只有 `26.48px`，layout
estimator 却把两个 mark 各按 `0.5em` 计宽，将 fragment 放大到 `180px`。背景盒与相邻
文本因此整体错位，即使 mark glyph 自身的 shaping offset 正确。

## 根因分析

字符宽度启发式把所有非 ASCII、非 CJK 字符归入默认宽度，没有区分 Unicode
nonspacing mark。mark 的位置由 shaper 相对 base glyph 决定，不应再贡献独立 inline
advance。

## 解决方案

复用 `unicode-bidi` 的 `BidiClass::NSM` 分类，在通用估算入口返回零宽。该判断位于
Ahem 特判之前，因此测试字体也不会把 mark 当成方框。spacing mark 和其他 Unicode
字符保持原有估算。

排查 combining mark 几何时应同时比较 layout estimate、shaped advance 和 glyph
offset；只修 glyph offset 无法消除 fragment 与 sibling 的位置偏差。

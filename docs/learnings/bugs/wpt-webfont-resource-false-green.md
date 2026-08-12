# WPT Webfont Resource False Green

- 日期：2026-08-13
- 相关模块：WPT reftest 资产、字体加载、Chromium Oracle

## 问题描述

已导入的 `font-size-adjust-ic-height` test/ref 缺少其固定 Noto CJK 字体。资源缺失时 self-source 差异为 2.60%，看似接近通过；补齐真实字体后反而升至 4.60%，但 Chromium Oracle 从 2.85% 改善到 2.44%。

## 根因分析

test 与 reference 都引用同一缺失资源，但两页声明的字号和 `font-size-adjust` 不同，因此会分别落入不同 fallback 几何。test-vs-ref 的低差异只是两个错误 fallback 偶然接近，不能证明规范行为正确。

## 解决方案

通过标准 WPT importer 同时登记 test、reference 和所有固定字体资源，让 `imported-resources.txt` 成为 fresh checkout 的可再生依赖账本。收益以完整目录 Chromium Oracle A/B 裁决。

## 如何避免

分析 webfont 用例前先核对每个 `src: url(...)` 文件存在且可解析。资源缺失时，无论 self-source 数字多低，都应先恢复资产再判断引擎行为。

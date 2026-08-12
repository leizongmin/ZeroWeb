# First Available Font Metrics

- 日期：2026-08-13
- 相关模块：`css-parser`、`style-system`、`render-foundation`、`webview`

## 问题描述

`first-available-font-001/002` self-source reftest 接近 0%，但 Chromium Oracle 分别为
4.42% 和 1.85%。ZeroWeb 把 `ex` 在解析期固定转换为 `0.8em`，把 `ch` 在计算值阶段固定
为 `0.5em`，因此无法按 CSS font list 与 `unicode-range` 选择字体度量。

## 根因分析

CSS Fonts 将 first available font 定义为首个可匹配 U+0020 的 face。test 与 ref 都走相同
常量近似时会互相抵消，self-source 无法暴露错误。另一个关联缺口是 `font-size-adjust`：
`em` 使用 computed font-size，但 `ex/ch` 必须使用调整后的 used font size。

## 解决方案

保留结构化 `Ex` 长度值；FontLoader 按 U+0020 与 `unicode-range` 生成 family metric map，
并提供真实 x-height 与 zero advance aspect。StyleSystem 按 CSS family 顺序选择首个有
metric 的 family，再用 used font scale 解析 `ex/ch`。`ZW_FIRST_AVAILABLE_FONT_METRICS=0`
可回退旧常量路径。

验证必须使用 Chromium Oracle；self-source 仅用于确认 test/ref 内部一致。

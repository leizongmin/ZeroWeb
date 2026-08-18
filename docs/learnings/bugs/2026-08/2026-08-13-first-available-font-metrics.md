---
date: 2026-08-13
modules: css-parser, style-system, render-foundation, webview
---

# First Available Font Metrics

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

## 后续：缺失 sxHeight 与 normal 行盒

部分旧字体 OS/2 表没有 `sxHeight`。若 shaping 在这种情况下放弃 `font-size-adjust`，而
style-system 已用 fallback aspect 放大 `ch`，会出现盒宽正确、glyph 仍为 specified size
的分裂。可从 `x` glyph bbox 的 `yMax / unitsPerEm` 推导 x-height fallback。

glyph 使用 adjusted size 后，`line-height: normal` 也必须基于 used primary size，否则
放大的 glyph 会在仍按 specified size 生成的行盒中重叠。此处只调整活跃
`font-size-adjust` 的 normal 行高；普通页面继续保持历史常数路径，不能借机全局启用
per-font ascent/descent/gap。

## 后续：generic paint compatibility 分支

generic family 的 paint compatibility 分支会以逐字符 paint width 替换 shaping advance。
该策略只适用于 layout 仍使用 legacy estimate 的普通 generic 文本。若
`font-size-adjust` 已让 layout 使用 adjusted shaped advance，paint 再替换一次会使
fragment 总宽正确、内部 glyph 位置错误。

排查时应同时对账 `fragment_width`、shaped advance 总和与 `paint_consumed`。三者不一致
说明是 layout/paint 契约问题；三者已在浮点误差内一致后，剩余 Oracle 差异应归因于字体
选择或 raster，不应继续调 fragment 空白。

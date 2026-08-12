# Font Face Size Adjust Advance

- 日期：2026-08-13
- 相关模块：css-parser、render-foundation/font、layout advance、engine paint

## 问题描述

实现 `@font-face size-adjust` 时，只把 descriptor scale 应用于 glyph shaping 和 raster，会让字形变大但 fragment 仍保留旧字号的 legacy advance。`size-adjust-03` 中 60px glyph 落在 40px fragment 上，Chromium Oracle 从 15.46% 恶化到 16.03%。

## 根因分析

generic/custom alias 可能仍命中 legacy contextual advance 分支。该分支只把 CSS `font-size-adjust` property 视为 used-size adjustment，不知道 face descriptor 已改变 `ShapedGlyph.font_size`。trace 中 `Quick` 的 glyph used size 为 60px，但 layout width 为 112px，paint 实际消费 170px。

## 解决方案

以 shaping 输出为权威：只要任一 `ShapedGlyph.font_size` 偏离 specified font size，layout 和 paint 都使用 absolute shaped advance。`font-size-adjust` property 活跃时完全覆盖 face descriptor，禁止两种 scale 相乘。修复后 css-fonts Oracle 2 改善、0 回归，总改善 2.33pp。

## 如何避免

任何改变 used font size 的功能都必须同时核对 shaping size、layout fragment width、paint consumed advance 和 raster size。先用 advance trace 验证四者闭合，再判断像素收益。

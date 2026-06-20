# 归档：M6 初始 Reftest 通过率数据（2026-06-07，inline 100% 明细）

**归档日期**：2026-06-22（R398 doc-maintenance 治理轮）
**归档原因**：本节为 M6（2026-06-07）「685 内联 reftest 100% 通过」的历史基线明细。该 685 内联 reftest 自 DC-14 确立起已明确**不计达标分母**（goal line 323/844；master.md DC-2~5 各节均标「内联 smoke 100% 不计达标」），故此 100% 明细仅作历史记录，不再需要占据 master.md 控制面板。达标口径的真基线见 master.md「上游真实 WPT Reftest 通过率」节（self-source 443/490 / chromium-Oracle ~42%）。

> 本文件为 master.md `## 初始 Reftest 通过率数据` 节的verbatim 迁出，只追加、不修改。

---

## 初始 Reftest 通过率数据

**日期**: 2026-06-07（M6 DC-5 全目录达标）
**总用例**: 685（内联 reftest）
**运行用例**: 685
**通过**: 685
**失败**: 0
**通过率**: 100.0%
**渲染模式**: CPU 软件渲染
**视口**: 800×600

### 按分类

| 分类 | 通过/总数 | 通过率 |
|------|-----------|--------|
| Layout | 484/484 | 100.0% |
| Text | 201/201 | 100.0% |

### 按 WPT 目录

| 目录 | 数量 | 通过率 | ≥50 达标 |
|------|------|--------|----------|
| css21/ | 78 | 100.0% | ✅ |
| css-box/ | 54 | 100.0% | ✅ |
| css-text/ | 51 | 100.0% | ✅ |
| css-grid/ | 51 | 100.0% | ✅ |
| css-flexbox/ | 51 | 100.0% | ✅ |
| css-fonts/ | 50 | 100.0% | ✅ |
| css-position/ | 50 | 100.0% | ✅ |
| css-display/ | 50 | 100.0% | ✅ |
| css-text-decor/ | 50 | 100.0% | ✅ |
| css-writing-modes/ | 50 | 100.0% | ✅ |
| css-multicol/ | 50 | 100.0% | ✅ |
| css-float/ | 50 | 100.0% | ✅ |
| css-table/ | 50 | 100.0% | ✅ |

### 覆盖范围

- 颜色 (5): 命名色 vs hex, 命名色 vs rgb, 不同颜色 mismatch
- 背景 (5): 多色背景, 百分比尺寸, 不同背景 mismatch
- 边框 (10): 等价边框声明, 不同边框颜色 mismatch, 边框方向, solid 等价, 不同边宽度, padding+border, box-sizing
- 盒模型基础 (5): margin, padding, 等价盒模型, 不同 padding mismatch
- 定位基础 (5): absolute, relative, 不同定位 mismatch, bottom/right
- 显示基础 (5): display:none, display:block, visibility, 显示隐藏 mismatch
- 尺寸 (5): 固定尺寸, 百分比尺寸, 不同尺寸 mismatch
- Flexbox 基础 (10): row, column, row-vs-block, grow, wrap, justify, align, gap, nested, basis
- Flexbox 进阶 (10): grow-proportional, grow-with-base, wrap-multi-line, align-center, justify-space-between, shrink-overflow, column-direction, gap-between-items, order-reorder, basis-0-grow
- Flexbox 边界 case (10): align-self-flex-end, flex-basis-auto-with-width, nowrap-overflow, justify-flex-end, justify-center, wrap-reverse, shrink-ratio, min-width-constraint, max-width-constraint, nested-flex-wrap
- Grid 基础 (10): 固定列, fr, 2x2, gap, auto-rows, mixed-fr-px, vs-block, 三列, row/col gap, nested
- Grid 进阶 (11): fr-unit-proportional, mixed-fr-px-proportional, auto-placement-3x2, gap-rows-columns, nested-grid-in-flex, minmax-column, repeat-auto-fill, grid-in-grid, justify-items-stretch, flex-in-grid-item, shorthand-gap
- Grid 边界 case (10): auto-rows-minmax, justify-content-center, align-content-center, implicit-rows, place-items-center, grid-auto-columns, named-grid-area-simple, fr-with-percentage, empty-tracks, percentage-track-sizing
- 定位进阶 (15): absolute-top-left, shift-mismatch, relative-offset, vs-no-position, in-flow, bottom-right, stacking, z-index, overlap-mismatch, multiple-relatives, absolute-in-relative, absolute-right-bottom, relative-offset-no-layout, z-index-stacking-order, absolute-overlaps-static
- 文本排版 (10): 颜色, align, whitespace, line-height, letter-spacing, word-spacing, text-indent, transform, flex-container, vs-background
- 盒模型进阶 (10): margin-collapse, box-sizing, border-colors, overflow-hidden, overflow-visible, max-width, min-height, percentage-width, auto-margin-center, negative-margin
- 显示进阶 (10): none-removes-layout, inline-block, visibility-hidden, nested-inline-block, none-vs-visible, flex-item-none, grid-item-none, nested-flex-grid, block-100pct, body-background
- 嵌套/复杂 (5): 三层嵌套, 不同内部尺寸 mismatch, 兄弟排序, float 布局
- Overflow (5): hidden clips, visible no-clip, hidden vs visible mismatch, nested overflow, overflow with margin child
- Margin 折叠 (5): sibling collapse, parent-child collapse, BFC no-collapse, auto center, body reset
- Quirks mode (5): hashless color, numeric color, unitless width, unitless padding, table height as min-height
- Table 布局 (9): basic-2col, basic-3col, multi-row, with-tbody, auto-width-equal-cols, row-tallest-cell, thead-tbody-tfoot, th-td-mixed, single-column
- Multi-column 布局 (10): column-count-2, column-count-3, column-width-auto, column-gap, columns-shorthand, balanced-4-children, uneven-heights, with-column-rule, mismatch-column-count, no-columns
- 文字排版 (51): text-align (justify/center/right/multiline), word-spacing (normal/large), text-decoration (underline/overline/line-through/dashed), text-transform (uppercase/lowercase/capitalize/none), white-space (pre/pre-wrap/pre-line/nowrap), line-height (double/tight/mismatch), font-size (large/mismatch), text-color (green), text-indent (50px/percent), letter-spacing (4px/2px), word-break (break-all/keep-all), overflow-wrap (break-word/long-url), CJK (line-break/mixed-wrap), tab-size, text-in-flex, text-in-grid, vertical-align (top/middle), 颜色/align/whitespace/line-height/letter-spacing/word-spacing/text-indent/transform/flex-container/vs-background (15 个 css21 基础)

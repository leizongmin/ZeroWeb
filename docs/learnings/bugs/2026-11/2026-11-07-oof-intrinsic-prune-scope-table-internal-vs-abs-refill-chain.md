---
date: 2026-11-07
modules: layout-engine, testing
---

# OOF 固有宽剪除的作用域：table 内部盒可剪，普通块盒剪不得（R5300 补测链依赖）

## 问题描述

R5444 修 s-11-1-1b-005（td 唯一 abs 子 → 列/表被 abs 文本 max-content 撑破，cell 黑底带 279.5 宽）
时，第一版把「OOF（absolute/fixed）文本不入固有文本测量」普适到共享 walker
`text_max_width_walk` 的全部消费方——lib 全腿立即翻红 R5300 两钉
（`r5300_three_level_abspos_chain_propagates_cb_width` /
`r5300_nested_abspos_branch_text_shrinks_like_top_level`），且不是钉过时而是真回归：普通块盒
「仅 abs 子塌 0 补测」链依赖 abs 文本宽度量，剪掉后 wrapper 塌 0，abs 内容被压成逐字竖排。

## 根因分析

ZW 的 abs 定位模型与 CSS §10.1 有系统偏差（探针 F 实证：100px 普通父内 abs-auto 叶，ZW 按
**父盒内容宽** sizing（96），chrome 按 ICB（280））。在此模型下，普通块盒的固有宽**必须**计入
abs 后代文本，否则 R5300 的塌 0 补测链（`apply_abspos_shrink_to_fit_width`）拿不到尺寸源——
两套语义在普通块盒上互为前提。而 table 内部盒（§17.5.2）的规范语义明确「固有宽 = 流内内容」，
且 corpus oracle 实证 cell 不计 abs 才是 chrome 行为；cell 域的 abs 子在 ZW 里另有 taffy 首趟
退化可用宽的独立 sizing（塌 min-content），不依赖 cell 固有宽记账。

## 解决方案

- 剪除作用域收窄到 **table 内部盒**：`box_content_max_width_inner` 叶分支按 `display` 判
  table-internal 才走 `text_content_max_width_scoped(.., true, false)`（R5243 离流剪复用）；
  `compute_cell_intrinsic_width_impl` 子循环三臂 + `collect_text_length` 同开关收窄在 cell 域。
- 语义界线表述：**cell/table 内部盒固有宽 = 流内内容（§17.5.2，chrome 一致）；普通块盒固有宽
  计 abs 文本（ZW 内部模型自洽面，chrome §10.1 偏差另案挂账）**。
- 新增跨域语义改动（本例：OOF 剪普适化）必须先跑受影响链的**既有钉**（R5300 补测链钉就在
  lib 里），lib 全腿是唯一能当场接住这类「剪除面过宽」回归的门。

## 证据

docs/goal/rendering-compat/evidence/r5444-cell-intrinsic-oof-skip-2026-11-07.md（探针矩阵 +
双态 ×3 washing：ON ✓0.46% ×3 / OFF ✗1.52% ×3；R5300 第一版红、收窄后绿）。

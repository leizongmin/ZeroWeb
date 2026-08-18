---
date: 2026-08-13
modules: style-system UA declarations, layout final IFC, table paint
---

# Table Text Metadata And UA Defaults

## 问题描述

表格单元格可按继承字号完成几何布局，但直接文本没有 `text_node_font_sizes` metadata，paint IFC 会回退 16px。补齐 metadata 后 glyph 尺寸正确，却可能放大尚未闭合的 feature/raster 与 table geometry 误差。

## 根因分析

TableCell 不属于 block-level，final IFC eligibility 在写入 paint metadata 前提前返回。与此同时，HTML UA 的默认 `border-spacing: 2px` 和 cell `padding: 1px` 尚未注入。三项各自规范正确，但当前管线不同阶段不一致，单独开启会把 test/ref 或 Chromium 差异推向相反方向。

## 解决方案

不要单独提交任一项。先用 kill switch 分别测 cell metadata、table spacing、cell padding，再同时覆盖 css-fonts Chromium Oracle 与 css-tables self-source。只有协同方案在两个目录均净非负时才落地。

## 如何避免

表格文本字号问题必须同时检查 computed style、cell geometry、stored text metadata、paint fragment size、UA spacing/padding。单看 glyph 大小或单个目录会得出错误结论。

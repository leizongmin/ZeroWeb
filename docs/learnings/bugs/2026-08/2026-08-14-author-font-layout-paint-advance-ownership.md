---
date: 2026-08-14
modules: layout-engine/inline, engine/paint, engine/pipeline
---

# Author font layout and paint must share ordered face advances

## 问题描述

显式 `@font-face` 文本在初次布局中可以取得真实 shaping advance，但 paint IFC 重建片段时仍使用逐字符估算宽度。partial webfont 与 fallback face 混排后，布局 fragment 宽度和最终 glyph 消费宽度相差十余像素，形成明显词间空洞。

## 根因分析

paint IFC 的字体 override 以 text node ID 为键，但文本收集路径用 parent element ID 查询。即使键修正，`shaping_font_id_for_style()` 也只识别单值 `font_id_overrides`，没有从 ordered `font_ids_overrides` 取首 face，因此 run 的 `font_id` 保持 `None`，`advance_run_width()` 必然退回估算路径。

全局启用 shaped layout 会让 generic/system 文本也进入 rustybuzz，历史上造成 37 倍布局耗时回归。正确边界不是关闭所有真实 advance，而是只允许已经解析到非 generic author face 的 run消费 ordered face advance。

## 解决方案

文本 run 使用自身 node ID读取 override；无 style 的 paint IFC 可从 ordered face list首项恢复主 face。`ZW_AUTHOR_SHAPED_LAYOUT` 默认开启并可用 `=0` 回滚，只对实际解析到显式 author face 的普通 IFC、paint IFC和匿名 flex/grid文本启用同源测量；generic/system run继续走既有估算路径。诊断时让 `ZW_SHAPED_ADVANCE_TRACE=1` 同时输出无 `font-size-adjust` 的 run，避免变量字体和普通 webfont被静默过滤。

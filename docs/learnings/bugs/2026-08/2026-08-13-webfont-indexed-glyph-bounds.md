---
date: 2026-08-13
modules: zero-render-foundation, zero-wpt-runner
---

# Webfont indexed glyph 必须按 face 边界校验

## 问题描述

启用 WOFF2 后运行完整 css-text，外部字体产生的 indexed glyph 触发 fontdue
`rasterize_indexed` 数组越界 panic，导致整个并行 reftest 进程退出。

## 根因分析

OpenType glyph index 只在产生它的字体 face 内有效。渲染入口此前直接把 IPC/shaping
携带的 `u16` index 交给 FreeType/fontdue，没有验证它小于该 face 的 `numGlyphs`。
字体 surface 扩大后，错误 face 归属或畸形 index 不再只表现为空字形，而会越过 fontdue
内部数组边界。

## 解决方案

在共享 `FontLoader::rasterize_glyph_index` 入口解析同一 font bytes 的 sfnt
`numGlyphs`，越界时返回既有 `GlyphNotFound`。CPU/GPU 调用方均已对该错误 fail-closed，
因此无需在多个后端重复 guard。

任何新增字体容器或 shaping 路径，都应以完整字体目录运行一次，而不只验证解码成功；
扩大可达输入面可能暴露此前被资源缺失掩盖的后端边界问题。

---
date: 2026-08-11
modules: layout-engine, engine/paint
---

# 行内片段元数据必须贯通 stored paint 路径

## 问题描述

IFC 的 `TextFragment` 增加 BiDi logical source 后，fallback paint 路径可以读取该字段，
但默认 stored paint 仍拿不到元数据。

## 根因分析

行内布局结果会从 `TextFragment` 复制到 `InlineLayoutFragment`，paint 再复制到局部
`PaintFragment`。这些都是显式字段转换；新增字段不会自动穿过快照边界。只测试
`all_fragments_with_line_y()` 会遗漏默认 `use_stored` 路径。

## 解决方案

新增行内片段字段时同步审计并测试以下链路：

1. `TextFragment`
2. `InlineLayoutFragment`
3. paint 局部快照类型
4. stored 与 fallback 两个消费入口

元数据必须在每次转换时显式 clone 或重建；行为 gate 应放在最终消费点，避免存储路径和
fallback 路径产生不同语义。

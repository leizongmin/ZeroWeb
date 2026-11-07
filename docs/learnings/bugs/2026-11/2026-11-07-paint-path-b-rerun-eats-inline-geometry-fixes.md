---
date: 2026-11-07
modules: layout-engine, engine
---

# layout 期行内几何修复不经 paint Path B 等价注入，像素必然不变

## 问题描述

line-clamp 末保留行再对齐的 ellipsis 预留宽修成真面度量（layout 期容器 face 捕获，
`container_font_id`）后，runner 环境 block-ellipsis-032 的 reftest 差异百分比纹丝
不动（0.50%），且新旧 PNG 逐字节相同——layout 侧确认落位已改变（kept [8,216)），
但渲染像素零变化。

## 根因分析

paint 侧对带 inline 内容的盒不是直接消费 layout 期 IFC 结果，而是 **Path B 独立
重排**：`paint_text`（engine/paint/painter/text.rs）以**空 styles HashMap** 重跑
`ctx.layout(doc, node_id, &HashMap::new())`，再按盒存储的 `line_clamp_cap` 重放
`apply_line_clamp_cap`。glyph 位置以这次重排为准。

于是同一容器的 `apply_line_clamp_cap` 在一帧内被调 3 次：intrinsic 宽度测量、
layout 期真布局（有 styles，捕获生效）、paint Path B（空 styles，捕获臂空转回
estimate）。修 layout 期那一次，paint 那一次照旧用旧度量 → 像素必然不变。凡
`layout()` 里靠 styles 捕获的量（容器 face id、字号、Ahem 判定……）在 Path B 都
拿不到，必须经 builder 通道或专用字段从 layout 期存储显式注入。

## 解决方案

新字段 `clamp_ellipsis_advance: Option<f32>`（Default None），paint 侧在
`apply_line_clamp_cap` 前用与 ellipsis 截断门**同一个表达式**
（`resolve_style_font_id` + `measure_char_cached`）计算并注入；realign 消费序
注入值 → 容器 face 捕获 → estimate。两个注意点：

- 不要顺手改共享字段（`container_font_size`/`container_strut_is_ahem`）：它们同时
  供 Path B 存活的 strut half-leading / vertical-align baseline pin 路径消费，
  改写会扰动无关行定位。先 grep 全部消费点再决定是加字段还是复用。
- 诊断入口：怀疑「修复没生效」时先数 `apply_line_clamp_cap` 每容器被调几次
  （env 探针打印 font_id/落位），3 次 = intrinsic/layout/paint 三 pass 结构，
  而不是怀疑修复本身。

同型风险面：`inline_metric_storage.rs` 等注释里标注「Path B 镜像」的既有通道都是
同一结构的产物——layout 期加了 styles 依赖的捕获后，要主动检查 Path B 是否需要
镜像注入，而不是等 reftest 像素不动再发现。

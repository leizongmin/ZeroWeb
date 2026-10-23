---
date: 2026-10-23
modules: layout-engine/inline, engine/paint
---

# IFC collect 期 styles 依赖判定必须有存储信号镜像（paint Path B 空 styles）

## 问题描述

layout 期与 paint 期对同一 DOM 的 inline item 收集结果劈叉：`<span style="position:relative">PRE<img>POST</span>`（img 32×32）在 layout 趟 items = text×3 + InlineBlock×1、行高 32 正确；paint 重跑趟 img 项整个消失，img 后文本少推进一个原子宽与方框叠盖。单测断言 `div.inline_layout` 却三臂全过——存储路径无恙。

## 根因分析

paint 对非存储容器走 Path B：`ctx.layout(doc, node_id, &HashMap::new())` 空 styles 重跑 IFC（R72 裁决，不可改为传真实 styles，BFC-004 等 4 案回归）。collect 期任何依赖 `styles.get(&id)` 的分叉，在 Path B 下因查表恒空而塌到固定侧：`inline_elem_has_nested_inline_block` 的 BFS 逐节点 `styles.get(&cid)` 失败即 continue、恒 false → span 恒落 FLAT_CHILD_WALK，walk leaf 对无文本无 frame 的 img `continue` 静默丢弃。

单测全过与像素错误并存的原因：`inline_layout` 存储有 pure-Ahem 门控（store-gate），存储路径（Path A）有真实 styles、判定正确；真实页面默认字体不存储、走 Path B 才触雷。排查靠 `ZW_DEBUG_MEASURE`（paint_text 分发记账）+ `ZW_DEBUG_IFC`（items 构成，n_ib=0 时无 items 行、行高异常是旁证）双探针对比 layout 趟与 paint 趟的 items。

## 解决方案

双通道存储信号（已四例同构：R4310 vertical_walk_nodes、R4312 block_child_walk_nodes、R5177 color_divergent、R5235 nested_atomic）：

1. layout 期判定为真**即记录**进 `NodeIdSet`（挂在 IFC 上下文 RefCell 上——记录点在 `&self` 方法内时不可改 `&mut self` 签名）；
2. 容器收尾时存进 `LayoutBox` 新字段（store-gate return **之前**——Path B 容器恰是非存储容器）；
3. collect 分叉改双通道：`styles.get(&id).map(直判).unwrap_or_else(|| 信号集.contains(&id))`；
4. paint Path B 经既有 builder 通道注入信号集，逐位镜像同一分叉。

关键点：**触发即记录**优于在信号侧重扫 DOM——layout 判真即记录天然覆盖所有门控组合，paint 只需镜像。嵌套递归（clone 子 ctx）时子层触发要 merge-back 回容器 ctx。跨 crate 消费走 `pub(crate)` 转发函数，不加宽模块可见性；配 `ZW_*` kill-switch（runtime_flags OnceLock 单读，不进热路径）。

## 避免方式

在 collect/finalize 路径新增任何 `styles.get` 依赖分叉时先问「Path B（空 styles）怎么复现这个判定」。单测断言 `inline_layout` 前先查 store-gate（pure-Ahem 门）——存储缺失 ≠ 布局错误；纯 Ahem 单测绿不能证明真实页面 paint 正确，需另建空 styles + 注入信号的镜像臂断言。

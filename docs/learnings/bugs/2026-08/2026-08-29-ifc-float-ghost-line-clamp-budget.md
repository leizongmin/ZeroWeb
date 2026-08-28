---
date: 2026-08-29
modules: zero-layout-engine
---

# IFC 中 float 子的幽灵空行吃掉 line-clamp 行预算

## 问题描述

`line-clamp-with-floats-001`（float:left div + 5 行 pre 文本 + line-clamp:4）clamp 后只有 3 行文本可见，容器高 116px（≠ 4×32=128），且第 4 行位置前有一个 20px 空档。无 clamp 对照渲染 5 行全对——行计数看似正确，但 clamp 预算里混入了一个不可见行。

## 根因分析

三层叠加：

1. **collect_items 的 block-level 分支对 float 子发 `InlineItem::Br`**（R57 M3 为兼容 r1733 float-avoidance 保留旧 strut 语义；in-flow block 子早已改发无 strut 的 `BlockBreak`）。
2. **R1286 给 Br 结束的空行赋 strut 高度**（20px）——真 `<br>` 的「空行占一行 line-height」语义。
3. **line-clamp cap 按 `lines.len()` 截断**——幽灵空行占据 cap=4 的一个名额，4 行真文本被裁剩 3 行。

诊断关键：`ZW_DEBUG_IFC=1` 只 dump 行盒高度数组 `heights=[20,32,32,32,32,32]`——**首行 20px（默认 strut）而后续行 32px（line-height:32px 声明）** 是幽灵行的指纹。配合 item 序列 dump（临时 eprintln item 枚举）可直接看到 `[Br, Text(...)]` 中的 Br 来自 float。

注意 probe 阶段的坑：仅把 float 的 Br 换成 BlockBreak 还不够——BlockBreak 分支照样 push 行盒，只是变成 **0 高幽灵行**（`heights=[0,32×5]`），cap 计数不变。行盒的**存在**本身就是预算占用，与高度无关。

## 解决方案

- collect_items：float 子改发 `BlockBreak`（CSS2 §9.5：float 脱离常规流不产生行盒，其后行经 float exclusion 缩宽即可）。kill-switch `ZW_FLOAT_NO_GHOST_LINE=0`。
- break_lines：`BlockBreak` 在行首空行上**不 push 行盒**（只推进 current_x/current_y 光标）。真 `<br>` 的 R1286 strut 语义保持不变。

## 如何避免

- 「行数不等于内容行数」类 bug（clamp 计数、行号定位、nth-line 断言）：先用 `ZW_DEBUG_IFC=1` 看 heights 数组——出现与声明 line-height 不符的小行（尤其 20.0 = default strut）优先排查 block-level/float 子代理断行条目。
- IFC 占位条目（Br/BlockBreak）与行盒产生是两个独立决策：条目只表示「此处断行」，行盒是否入账要看是否有内容（runs 非空或 height>0）。
- float 的全部行内效果都应经 `effective_content_area`（exclusion 缩宽）表达，不应有任何行盒占位。

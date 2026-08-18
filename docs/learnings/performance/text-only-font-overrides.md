# Text-Only Font Overrides

日期：2026-08-18

相关模块：`crates/layout-engine/src/font_resolution.rs`、`crates/layout-engine/src/inline/mod.rs`、`crates/layout-engine/src/inline_finalization.rs`

## 问题描述

`collect_font_overrides` 会为整棵 DOM 中每个带样式的 element 和 text node 写入 ordered face IDs、`font-size-adjust` 与 variation context。实际消费点只按文本节点 `NodeId` 查询：TextRun 构造、`advance_run_width` 和匿名 flex/grid 文本测量都使用 text node key。element key 没有消费方，却会参与 HashMap 写入、Vec clone 和默认 context 判断。

## 根因分析

R3424-F 将 font overrides 提升为 pass 级共享后，避免了每个 IFC 重复全树收集，但单次收集仍沿用早期“所有 styled node 都物化”的宽口径。后续消费路径已经收敛到 text node ownership，未同步缩小 producer 的物化范围。

一个先试的替代方案是把 resolve cache key 从 owned `Vec<String>` 换成 borrowed slice。medium A/B 中 layout p95 `245.79→253.59ms`、total p95 `426.82→432.65ms`，说明减少 clone 不足以形成稳定收益，且会引入更复杂的生命周期约束。

## 解决方案

默认只为文本节点物化三张 override map，遍历仍覆盖整棵树，并继续使用父元素 computed style 为文本节点解析字体上下文。保留 `ZW_FONT_OVERRIDE_TEXT_ONLY=0` 回滚旧 all-node 写入。

测试需要同时锁住两件事：

+ 默认模式不再为 element key 写入 override。
+ all-node 与 text-only 模式对文本节点输出完全一致，包括非默认 `font-size-adjust` 与 variation context。

## 避免方式

优化共享 map 时先追踪所有消费点的 key ownership。若消费方只读取 text node key，producer 不应继续为 element key 物化数据。先缩小写入范围，再考虑 key 表示或 hasher 这类局部微优化。

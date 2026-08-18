---
date: 2026-08-17
modules: zero-layout-engine DOM-to-taffy 构树
---

# 热路径映射应先确认消费方

## 问题描述

`BuildContext.node_map` 为每个 DOM 节点记录 DOM NodeId 到 taffy NodeId 的映射。perf 中 NodeId SipHash 及其 write 路径占 medium 页面约 4.2% CPU。

## 根因分析

全仓检索显示该 map 只有初始化和两处 insert，没有任何读取，也不在构树返回值中。真实反向映射 `taffy_to_dom` 是独立字段。旧 map 因而只消耗 HashMap 扩容、SipHash、内存和 drop，不能影响布局或后续阶段。

## 解决方案

默认不创建和写入 `node_map`，字段改为 `Option<HashMap<...>>`。`ZW_TREE_NODE_MAP_RECORD=1` 可恢复旧诊断记录，用于 A/B 或临时调查。`taffy_to_dom` 保持原样。

通用规则：优化哈希器前先审计 map 的读写闭包；零读取的数据结构应先停写，而不是让无用工作更快。

## 验证

两组反序 medium A/B 均改善：layout p95 `407→378ms` 与 `382→373ms`，total p95 `703→626ms` 与 `644→618ms`。RSS 两组方向不一致，不作为收益证据。reftest 687/687，welcome 16.61%，所有结构门通过。

# 连续 last-write-wins 写入可延迟去重

- 日期：2026-08-17
- 相关模块：`zero-layout-engine`

## 问题描述

行断会把同一文本节点拆成连续的 word/line fragments。最终 IFC 原本为每个 fragment 向多张 NodeId map 重复写入字体、行高、间距和 paint fallback 度量，并重复 clone font-family。frame-pointer profile 中 NodeId SipHash 与多类 map insert 成为稳定热点。

## 根因分析

这些输出 map 对同一 NodeId 都采用 last-write-wins。连续 fragments 的中间值不会被消费，但旧实现仍完整执行每次 hash、lookup、insert 和 clone。

用 `HashSet` 记录已见 NodeId 看似直接，却会在正要消除的 NodeId hash 热点上再增加一张表。反向遍历全局去重还会改变不同键的提交顺序。

## 解决方案

前向遍历时只延迟一个 fragment。下一个 fragment 的 NodeId 相同时覆盖 pending；NodeId 改变时提交 pending。这样只合并连续同键段，不需要额外容器。对 `A,B,A` 等非连续输入仍按原顺序提交三次，因此完整保留全局 last-write-wins。

默认启用该路径，`ZW_IFC_METRIC_DEDUP=0` 恢复逐 fragment 写入。永久测试同时覆盖连续重复和 `A,B,A` 非连续重复，并逐项比较全部输出 map。

## 验证

固定 CPU 模块 microbench 使用 100 个相邻同节点 fragment、20,000 次存储，反序结果从 `537.8/593.2ms` 降至 `15.6/15.8ms`。真实 medium profile 中 `NodeId→f32 insert` self 从0.52%降至0.29%，font-size-adjust insert 从0.31%降至0.21%。整页时钟受共享主机频率漂移影响，不作为收益证据。

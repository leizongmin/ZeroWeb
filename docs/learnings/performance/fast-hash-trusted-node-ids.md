# 受信任的 NodeId 热点可使用专用哈希

- 日期：2026-08-18
- 相关模块：`zero-layout-engine`、`zero-engine`

## 问题描述

布局和绘制之间用多张 map 传递字体、行高、间距与 BiDi 度量。它们的 key 全部是 DOM slotmap 生成的 `NodeId`，但标准 `HashMap` 每次仍执行面向通用不可信输入的随机 SipHash。frame-pointer profile 中 layout 的 NodeId hash 与 SipHasher 合计成为稳定热点。

## 根因分析

`slotmap::KeyData::hash` 已把 index + generation 合成单个 `u64` 并调用 `write_u64`。对这类内部整数 key，重复执行 SipHash 的抗碰撞成本没有对应信任边界收益。

直接把原始 `u64` 当 hash 也不够严谨。常见 NodeId 的 generation 高位相同，会让 hashbrown 的 h2 fingerprint 缺少区分。专用快路仍需做低成本全位混合。

## 解决方案

提供 `NodeIdMap/NodeIdSet`，默认用 SplitMix64 混合 slotmap 的单个 `u64`。迁移范围只覆盖 LayoutBox paint fallback 度量集合及对应 IFC override，styles、DOM 和外部输入 map 保持标准随机哈希。

`ZW_NODE_ID_FAST_HASH=0` 为每张 map 构造独立 `RandomState`，恢复标准 SipHash，供同一 release 二进制跨进程 A/B 和紧急回滚。Hasher 的通用 `write` 路径保持增量 FNV 状态，避免未来多次 write 丢失前缀。

## 验证

当前代码反序 medium A/B 的可比 pair 中，layout p50 改善3.2%，paint p50 改善1.8%，total p50 改善2.4%。frame-pointer profile 中 layout SipHasher self 从1.77%降至1.13%，专用 fallback `NodeIdHasher::finish` self 从0.30%降到报告阈值0.08%以下。layout `1393/1393`、reftest `687/687`、产品 smoke 和性能绝对门均通过。

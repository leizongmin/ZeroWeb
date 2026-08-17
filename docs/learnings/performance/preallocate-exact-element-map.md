# 大值映射应按精确键空间预留

- 日期：2026-08-17
- 相关模块：`zero-dom`、`zero-style-system`

## 问题描述

样式系统逐元素向 `HashMap<NodeId, ComputedStyle>` 插入大对象。medium 页面中，扩容 rehash 的搬运约占全帧 1.37%，并放大哈希、分配和峰值内存。

## 根因分析

结果 map 的键空间是元素节点，不是全部 DOM 节点。按 `node_count()` 预留会把文本、注释和文档节点算入，早期实验因此过度分配并变慢；不预留则在递归插入期间反复扩容和搬运完整 `ComputedStyle`。

## 解决方案

DOM 暴露精确 `element_count()`，样式计算按该数量一次预留结果 map。`ZW_STYLE_MAP_EXACT_CAPACITY=0` 可恢复旧增长路径。对大值容器做容量优化时，必须使用真实键空间，不能用方便但偏大的代理计数。

## 验证

固定100轮 probe checksum均为40300，task-clock 两组下降3.8%和9.2%。medium style p95 两组从 `84.78→50.22ms`、`71.85→63.11ms`，单场景 RSS 约 `153→136MiB`；profile 中 style insert 从约3.0%降至0.56%，`reserve_rehash` 热路径消失。reftest `687/687`，产品与完整性能门通过。

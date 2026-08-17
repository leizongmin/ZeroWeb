# 默认上下文应使用稀疏映射

- 日期：2026-08-17
- 相关模块：`zero-layout-engine` 字体解析与行内布局

## 问题描述

pass 级 font overrides 为每个元素和文本节点记录有序字体 ID、`font-size-adjust` 和 `font-variation-settings`。medium 页面的大多数节点使用后两项的 CSS 初始值，仍会产生两次 NodeId 哈希、map 插入、扩容和销毁。

## 根因分析

行内 advance 消费端在 map 缺键时已经分别回退 `font-size-adjust: none` 和 `font-variation-settings: normal`。因此记录默认值没有提供额外语义，只把密集的默认状态复制到两张临时表。

## 解决方案

有序字体 ID 继续完整记录；两个字体上下文 map 只记录非默认值。`ZW_SPARSE_FONT_OVERRIDES=0` 可恢复旧全量记录用于 A/B。优化前必须审计所有消费点，只有缺键语义与被省略值完全相同时才能使用稀疏表示。

## 验证

两组反序 medium A/B 的 layout p95 从 `387→365ms`、`419→392ms`，total p95 从 `648→608ms`、`716→664ms`，RSS 两组均下降约 2MiB。默认缺省与非默认保留测试通过；reftest `687/687`，welcome `16.61%`，完整性能绝对门通过。

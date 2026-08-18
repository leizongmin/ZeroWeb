---
date: 2026-08-17
modules: zero-layout-engine DOM-to-taffy 构树
---

# 布局构树应借用只读 ComputedStyle

## 问题描述

`build_subtree` 为每个 DOM 节点 clone 完整 `ComputedStyle`，随后只读该副本生成 taffy style 和布局元数据。perf 中 clone/drop 本体约占 2.6%，并放大 `memmove`、分配器和释放路径。

## 根因分析

`ComputedStyle` 包含大量字符串、向量和复合属性。即使单个 clone 的符号 self 占比不高，8,000 节点页面会复制和释放整批堆数据，造成 allocator contention、内存搬运和 cache pollution。布局构树期间 `styles` 不变，不需要拥有副本。

## 解决方案

使用 `Cow<ComputedStyle>`：样式存在时直接 `Borrowed`，缺失时才 `Owned(ComputedStyle::default())`。`ZW_LAYOUT_STYLE_BORROW=0` 恢复旧的逐节点 clone 路径。helper 与测试放在 `tree/style_borrow.rs`，避免让 1990 行的 `tree.rs` 超过仓库上限。

通用规则：热路径读取大结构时先确认生命周期和可变性；只读且源容器稳定时，借用优先于 clone。

## 验证

Borrowed/Owned 分支测试覆盖样式存在、回滚和缺失 fallback。两组反序 medium A/B 均显著改善：layout p95 `510→389ms` 与 `462→412ms`，total p95 `842→629ms` 与 `852→676ms`。reftest 687/687，welcome product smoke 16.61%，所有结构门通过。

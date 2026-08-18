---
date: 2026-08-17
modules: zero-layout-engine DOM-to-taffy 构树
---

# 热路径运行开关应按布局快照

## 问题描述

`build_subtree` 在每个 DOM 节点重复读取 margin-trim、content-visibility、content replacement、BR 和 inline coherence 等环境开关。margin-trim 每个节点还读取两次。perf 中 `getenv` 占 medium 页面约 8.75% CPU。

## 根因分析

这些开关在一次 layout transaction 内不应变化，但旧实现把进程环境当作逐节点数据源。每次 `std::env::var` 都进入 libc、复制字符串并分配/释放，8,000 节点页面会放大成数万次系统环境查询。

## 解决方案

`BuildContext` 创建时构造 `TreeRuntimeFlags`，一次读取七个构树开关；递归节点只读取 bool 字段。每次新布局都会重新快照，因此保留跨 layout 动态切换。`ZW_TREE_ENV_SNAPSHOT=0` 恢复旧的逐节点查询路径，原七个 kill-switch 的名称、默认值和判定语义不变。

通用规则：事务内不变的运行配置应在事务边界读取一次，不应在递归热循环中访问进程环境。

## 验证

闭包计数测试证明 snapshot 路径不执行 live lookup，回滚路径执行一次。两组反序 medium A/B 均改善：layout p95 `418→374ms` 与 `384→376ms`，total p95 `669→614ms` 与 `632→608ms`；RSS 分别下降约 1.9MiB 和 4.9MiB。reftest 687/687，welcome 16.61%，所有结构门通过。

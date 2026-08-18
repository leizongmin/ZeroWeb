---
date: 2026-08-17
modules: zero-layout-engine
---

# 热路径中的进程策略应统一快照

## 问题描述

IFC 的 child 收集、换行、CJK 拆词和 BiDi source 构造会按 child、run 或 fragment 重复调用 `std::env::var`。这些环境变量实际是进程启动策略，但 live lookup 将字符串扫描、锁和分配成本带入文本热循环。frame-pointer profile 中 `getenv` self 一度占 medium 全帧7.14%。

## 根因分析

开关最初分散在各特性实现附近，单次读取成本很小；当 IFC 在中型页面上处理数千文本节点时，调用次数按节点和 fragment 放大。逐个修成局部静态还会留下不一致的回滚语义。

并非所有开关都适合快照。已有测试若在同一进程内切换某变量，该变量必须继续 live lookup，或先重构测试和运行时契约。本轮发现 `ZW_IFC_IMG_INTRINSIC` 属于该类，因此明确排除。

## 解决方案

将 11 个进程级 IFC 开关集中到 `inline/runtime_flags.rs`，用 `LazyLock` 保存原有 default-on、opt-in 或 presence 语义。所有调用方只读 bool。`ZW_IFC_ENV_SNAPSHOT=0` 统一恢复旧 live lookup，便于同一 release 二进制跨进程 A/B 和紧急回滚。

通用规则：快照前先全仓搜索测试和调用方是否依赖进程内动态切换；有动态语义的变量保持 live，其余进程策略使用统一 helper 和统一总开关。

## 验证

两组反序 medium A/B 的 layout p50 分别改善12.4%和7.1%，total p50 改善6.8%和6.1%。frame-pointer profile 中 `getenv` self 从6.50%降至3.63%。layout `1392/1392`、reftest `687/687`、产品 smoke 和性能绝对门均通过。

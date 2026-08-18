---
date: 2026-08-19
modules: apps/browser/src/process_backend/indexed_db_owner_tests.rs
---

# IndexedDB terminal state wait

## 问题描述

跨 renderer versionchange 测试在全量并行运行时偶发读到 `upgrade`，随后立即断言期望的
`success` 并失败；串行运行通常在第一次查询前已完成 success，因此长期掩盖问题。

## 根因分析

测试等待器按“值不等于 pending”判断完成，调用方只把 `blocked` 映射回 pending，却遗漏
合法中间态 `upgrade`。并行负载改变事件推进速度后，等待器把中间态误判为终态。

## 解决方案

等待异步状态机时明确列出全部非终态，而不是依赖固定轮询次数或任意 sleep。该用例把
`blocked` 和 `upgrade` 都映射为 pending，继续复用既有 20 秒墙钟上限，只在
`success:<version>` 或错误状态出现后结束等待。

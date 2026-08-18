---
date: 2026-08-19
modules: engine, wpt-runner
---
# IndexedDB listener 异常必须结合 transaction 状态处理

## 问题描述

IndexedDB request 的 success/error/upgradeneeded listener 抛异常时，旧实现直接吞掉异常，
导致 transaction 错误提交；改为无条件 abort 后，explicit commit 用例又因 committing
transaction 上调用 abort() 抛 InvalidStateError 而超时。

## 根因分析

IndexedDB 的 fire-event 算法要求报告异常、继续同次派发中的后续 listener，并在 transaction
仍可回退时以 AbortError abort。显式 commit 已把 transaction 推进到不可回退状态，之后的
callback 异常只能报告，不能撤销 commit。事件异常处理若脱离 transaction 状态机实现，必然在
“未提交应 abort”和“已 committing 不得 abort”之间顾此失彼。

## 解决方案

共享 listener 调用 helper 返回“本次派发是否抛异常”，同时负责向全局 error 路径报告且继续
后续 listener。request 派发收尾仅在 transaction 尚未 aborted/committing/finished 时写入
AbortError 并 abort；upgradeneeded 使用同一异常信号触发 versionchange rollback。回归必须同时
覆盖普通 transaction abort、第二 listener 继续执行、explicit commit 保持完成三个状态。

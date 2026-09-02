---
date: 2026-09-02
modules: service-worker, js-dom-shim, page-runtime, wpt-runner
---

# Service Worker Unregister Poll Leak

## 问题描述

`registration-updateviacache.https.html` 在单独运行前 24 个 subtest 时可以通过，但完整
25 个 subtest 会在 testharness 完成前挂起。把最后一个 subtest 替换为立即通过的 marker
后仍然挂起，说明问题不在失败 update 回滚逻辑本身，而在前序注销用例后的异步队列状态。

## 根因分析

页面侧 `ServiceWorkerRegistration.unregister()` 会把 registration 从 `_registrations` 删除，
但已排队的 `pollRegistration(reg)` 定时器仍持有旧 JS registration 对象。该对象已不再能从
宿主 manager 读到有效 registration，却会继续在每次 poll 末尾重新 `setTimeout()` 自身。
testharness runner 的完成探针会等待近未来定时器排空，因此这个轮询链让完整 case 永远无法
进入 terminal 状态。

同一轮排查还发现 page-runtime 的 runtime capacity 统计把 stopped-but-not-reaped runtime
也算入 `DEFAULT_RUNTIME_LIMIT`，导致连续 unregister/client cleanup 后后续注册可能被容量门拒绝。

## 解决方案

- `LocalServiceWorkerHost::runtime_count()` 只统计 `runtime.is_running()` 的 runtime，避免已停止
  worker 继续占用容量预算。
- 页面 shim 在 `unregister()` 成功后标记 JS registration 为 `_unregistered`，并让
  `pollRegistration(reg)` 对该标记早返。旧 registration 的 `updateViaCache` 等属性仍可读，
  但不再继续向宿主排状态 poll。
- 回归测试覆盖 manager 侧受控 client 移除后的 runtime 回收，以及 WebView 页面侧注销后
  pending timer 不再被旧 registration 轮询重新填充。

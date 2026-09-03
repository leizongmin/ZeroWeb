---
date: 2026-09-03
modules: script-sandbox,page-runtime,wpt-runner
---

# Service Worker waitUntil 必须等待全部 lifetime promises settle

## 问题描述

`service-workers/service-worker/extendable-event-waituntil.https.html` 暴露出
Service Worker lifecycle `waitUntil()` 的两个语义偏差：

- install 事件里多个 `waitUntil()` promise 中有一个先 reject 时，worker 过早进入
  `redundant`，没有等待后续 lifetime promise settle。
- activate 事件的 `waitUntil()` rejection 会让 manager 把新 worker 标记为
  `redundant`，但 WPT 期望 activation 仍进入 `activated`。

## 根因分析

`crates/script-sandbox/src/service_worker.rs` 的 JS shim 使用
`Promise.all(pending)` 汇总 lifecycle promises。`Promise.all` 在首个 rejection 时立即
reject，这不等价于 Service Worker 规范的“等待所有 asynchronous extensions 完成后再判定”
语义。

同时 `crates/page-runtime/src/service_worker_manager.rs` 将 activate 阶段的
`LifecycleSettled.succeeded=false` 直接传给 activation 状态推进逻辑，导致 activation
lifetime promise rejection 阻断新 worker 成为 active。

## 解决方案

- lifecycle dispatch 汇总时把每个 promise 转成 always-fulfilled wrapper，记录首个
  rejection，等 `Promise.all(wrappers)` 完成后再设置 lifecycle result。
- install 阶段继续使用 `succeeded=false` 失败安装。
- activate 阶段保留 `LifecycleSettled.succeeded=false` 和错误 message 作为诊断事件，但
  manager 应按 activation algorithm 继续完成 activation，避免把 worker 置为 redundant。


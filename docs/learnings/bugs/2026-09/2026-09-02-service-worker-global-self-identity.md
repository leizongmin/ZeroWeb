---
date: 2026-09-02
modules: script-sandbox, wpt-runner, service-workers
---

# Service Worker Global Self Identity

## 问题描述

`service-workers/service-worker/global-serviceworker.https.any.js` 在 worker global 中直接断言
`self.serviceWorker`、`registration.installing` / `registration.active` 与启动期
`serviceWorker.postMessage()`。最初 runner wrapper 可以注册并收集 worker 结果，但 worker
内部 4 个 subtest 全部失败：`registration.installing` 为 `undefined`，`self.serviceWorker`
不存在，自消息也无法派发。

## 根因分析

Service Worker runtime 的 bootstrap 只暴露了面向页面投影的 `registration.active` 固定对象，
没有给 worker global 建立“当前 ServiceWorker 对象”这一身份锚点。install/activate 事件派发时，
registration slot 也没有按事件窗口切到当前 worker，导致 worker 侧无法观察自己的 lifecycle
identity。

第二个缺口是 sandbox 没有内建 `queueMicrotask`。自消息需要异步派发，直接调用会破坏 WPT
启动期监听注册顺序；但没有 microtask fallback 时，`serviceWorker.postMessage()` 会抛
`queueMicrotask is not defined`。

## 解决方案

在 Service Worker bootstrap 中创建单一 `currentServiceWorker`：

- 通过只读 `globalThis.serviceWorker` 暴露给 worker global。
- `scriptURL` / `state` 使用 accessor 暴露，只读断言通过，内部仍可随 lifecycle 更新状态。
- install 事件窗口内：`registration.installing === serviceWorker`，state 为 `installing`。
- install settled 后：`registration.waiting === serviceWorker`，state 为 `installed`。
- activate 事件窗口内：`registration.active === serviceWorker`，state 为 `activating`。
- activate settled 后：`registration.active === serviceWorker`，state 为 `activated`。
- 给 worker global 补 `queueMicrotask` 的 `Promise.resolve().then(callback)` fallback，并让
  `serviceWorker.postMessage()` 对当前 worker 自消息异步派发 `MessageEvent`，其中
  `event.source === serviceWorker`。

避免方式：新增 worker-global API 时不要只对齐页面侧 projection；凡 WPT 直接在 SW global
读取的对象，必须在 runtime bootstrap 内建立独立身份与事件期状态投影，并用 worker-side
testharness 结果验证真实观察值。

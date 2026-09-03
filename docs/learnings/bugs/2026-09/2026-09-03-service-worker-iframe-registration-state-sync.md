---
date: 2026-09-03
modules: engine, webview, wpt-runner
---

# Service Worker Iframe Registration State Sync

## 问题描述

推进 `service-workers/service-worker/controller-on-load.https.html` 时，为 iframe
`ServiceWorkerRegistration.active` 增加了 iframe realm wrapper。目标 case 通过后，完整
Service Worker core baseline 暴露 `registration-updateviacache.https.html` 超时：前 4 个
顶层 registration 子测通过，随后 21 个涉及 iframe registration 的子测停在 pending。

## 根因分析

父 window 的 registration 状态推进只更新父 realm 的 `ServiceWorker` 对象并派发
`statechange`。iframe `getRegistration()` 返回的是跨 realm wrapper；wrapper 读到的是父
registration 的 slot，但一旦测试把 `registration.installing` 保存到局部变量并等待
`statechange`，后续父状态变化没有派发到这个 iframe wrapper。结果 iframe 内
`wait_for_state(t, registration.installing, 'activated')` 永远等不到事件。

另一个相关边界是 iframe 内 `navigator.serviceWorker.register()` 不能直接复用父容器的
document URL 基准。scriptURL/scope 必须按 iframe document URL 解析，同时 host callback 要
验证该 document URL 与顶层同源，避免页面传入跨源伪 document URL。

## 解决方案

- iframe Service Worker container 的 `register()` 显式把 iframe `doc._zwURL` 作为 document
  URL 传入 `__zw_sw_register`，host callback 使用该 URL 做标准注册校验和 client observe。
- host callback 接受 JS 传入的 document URL 前先校验与顶层 document URL 同源。
- 父 registration `applyState()` 发生状态变化时调用 `notifyIframeRegistrationChange(reg)`。
- iframe 侧 `__zwRefreshServiceWorkerRegistration()` 刷新本 realm 已创建的 worker wrapper，
  在 state 改变时派发 `statechange`。

以后改 iframe/window 跨 realm Service Worker 投影时，必须同时验证目标新 WPT 和既有
`registration-updateviacache.https.html`，因为后者覆盖了 iframe registration wrapper 的
长期 state/event 同步。

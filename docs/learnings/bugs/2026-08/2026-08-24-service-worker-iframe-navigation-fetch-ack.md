---
date: 2026-08-24
modules: service-worker,webview,js-dom
---

# Service Worker iframe navigation fetch must not block page-side ACK

## 问题描述

导入 `fetch-event-throws-after-respond-with.https.html` 时，Service Worker runtime
已经能正确保留先前 `respondWith()` 提交的 response promise，不会被 handler 后续同步
throw 覆盖，但完整 WPT 仍失败在受控 iframe load 后读取
`frame.contentDocument.body.innerHTML`。

## 根因分析

iframe 初始 navigation fetch 走 WebView 同步 `__zw_fetch` callback。该 WPT 的 worker
在 `respondWith(sync().then(...))` 中通过 `MessagePort` 发 `SYNC`，等待页面回 `ACK` 后才
resolve response。同步 host callback 在等待 Service Worker fetch settle 时占住页面 JS
调用栈，页面无法继续轮询 worker-to-client message 并发送 ACK，于是 iframe 文档不能在
`onload` 前物化。

同类问题容易被误判为 iframe document 构造问题；真正的阻塞点是同步 fetch callback 与
page-side message pump 之间互等。

## 解决方案

iframe navigation fetch 需要使用异步 completion：shim 为每个 iframe 生成稳定 pending id，
host 线程后台派发 Service Worker fetch，完成后通过 `__zwResolveCallback` 回填 iframe
document/window，再由 iframe load 调度等待该 entry settle。等待 fetch 期间，
`ServiceWorkerManager` 可在只有一个 pending fetch client 时把未显式寻址的
worker-to-client message 路由给该 client，但不能释放普通 page-message reservation。

验证时要覆盖 MessagePort-backed response：worker 发 `SYNC`、页面回 `ACK`、最终 iframe body
来自 SW response，而不是网络 fallback。

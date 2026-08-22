---
date: 2026-08-22
modules: service-worker,webview,js-dom
---

# Service Worker iframe client observation must cross the WebView bridge

## 问题描述

`fetch-event-async-respond-with.https.html` 的 page setup 能完成，受控 iframe 的
`contentWindow.fetch()` 也会结束，但测试页收不到 worker 通过 `event.source.postMessage()`
回传的 `InvalidStateError` 结果，最终表现为 result message timeout。

## 根因分析

JS DOM shim 在创建 iframe 时已经调用 `__zw_sw_observe_window_client(clientId, url, "nested")`
登记 nested window client，但 WebView 宿主没有注册这个 callback。in-process WebView 路径下
iframe client 因此没有进入 `ServiceWorkerManager` 的 client registry；后续 iframe 发起
`fetch()` 时只能按请求 URL scope 匹配，无法优先使用 iframe 自身受控 registration，也无法把
`event.source.postMessage()` 路由回对应 client。

同类问题容易被误判为 worker event-loop 或 `respondWith()` timing bug，因为页面侧只看到消息
轮询超时。

## 解决方案

WebView 必须和 renderer/browser 路径一样暴露 window client 生命周期桥：
`__zw_sw_observe_window_client(clientId, url, frameType)` 调用
`ServiceWorkerManager::observe_window_client_with_frame_type()`，
`__zw_sw_remove_window_client(clientId)` 调用 `remove_client()`。iframe
`contentWindow.fetch()` 还需要把 iframe 的 client id 与 referrer 透传给 host fetch bridge，
manager dispatch 再优先按 `request.client_id` 找 active controlled registration，最后才按请求
URL 的 longest-scope fallback。

验证时不要只看 fetch promise 是否 resolve；必须断言 worker 发回的 client message、task 分支的
网络 fallback，以及 iframe 初始 navigation fetch 均符合预期。

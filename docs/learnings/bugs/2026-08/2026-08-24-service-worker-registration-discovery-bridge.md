---
date: 2026-08-24
modules: service-worker,webview,js-dom
---

# Service Worker registration discovery must be available in WebView

## 问题描述

`navigator.serviceWorker.register()` 更新同一 scope 后，旧 active worker 调用
`skipWaiting()` 会把旧 registration snapshot 推到页面侧的 `redundant` 状态。页面随后轮询
`registration.active.scriptURL` 等待替换 worker，但 WebView 测试稳定超时：
`registration.active` 已被清空，页面侧对象没有切到新的 active registration。

## 根因分析

JS DOM shim 已经有 `navigator.serviceWorker.getRegistration()` 的 host bridge 分支，但
WebView 宿主没有注册对应的 `__zw_sw_get_registration` callback。旧 worker 变成
`redundant` 时，shim 只能按旧 snapshot 更新当前 `ServiceWorkerRegistration` 对象，无法重新向
`ServiceWorkerManager` 查询同一 client/scope 下的代表 registration。结果是 manager 里新版本已
经 activated，页面侧 wrapper 仍停留在旧 id，表现为 `active === null` 的 stale registration。

第二个盲区在状态变更日志路径：`applySnapshot(... redundant ...)` 发现了替换 registration，但
`pollRegistration()` 处理 queued state change 时只更新旧 worker state，未执行替换发现，因此完
整 WebView lib 测试压力下仍可能暴露 stale wrapper。第三个盲区在
`navigator.serviceWorker.controller` getter：如果较早排队的 `controllerchange` 任务晚于新版本
激活执行，事件处理器读到的 cached `_controller` 可能已经过期，需要在 getter 中重新向 host 读取
当前 controller snapshot，再返回页面可见对象。

这类问题跨过了 JS shim、WebView host callback 和 page-runtime manager 三层，不能只在测试里延
长等待时间；等待只会掩盖缺失的发现通道。

## 解决方案

WebView 路径必须暴露与页面 runtime 契约一致的 registration 查询桥：
`__zw_sw_get_registration(clientURL)` 解析 client URL，轮询 `ServiceWorkerManager`，再用
`registration_for_url(origin, clientURL)` 返回当前代表 registration snapshot。

页面侧在收到 `redundant` 状态时应主动走一次该发现桥；若返回的 registration id 不同，则通过
`upsertSnapshot()` 合并新 snapshot，让既有 `ServiceWorkerRegistration` wrapper 跟上替换版本。
这一步必须覆盖两条入口：直接应用 registration snapshot 的路径，以及从状态变更日志恢复 worker
状态的路径。

`navigator.serviceWorker.controller` getter 也应按 host 最新 snapshot 刷新 activated controller，
避免异步事件处理器观察到旧缓存；排队的 `controllerchange` 任务应带 generation/id guard，防止旧
任务在后续 controller 切换之后再次派发。

回归验证要覆盖 `skipWaiting()` 替换版本、controller 跟踪，以及完整 WebView lib 测试，确保修
复不是只满足单个轮询条件。

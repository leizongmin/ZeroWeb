---
date: 2026-08-24
modules: service-worker,webview,js-dom
---

# Service Worker registration polling must rearm across execute_script calls

## 问题描述

`make test` 在全量并行负载下间歇性暴露 Service Worker replacement 用例失败：
`navigator_skip_waiting_activates_replacement_version` 等待超时，或
`navigator_controller_tracks_document_and_skip_waiting_replacement` 收到 `controllerchange` 后仍看到旧
controller script URL。两个用例单独运行可通过。

## 根因分析

WebView 进程内测试路径没有注册真实 host timer，JS shim 的 `setTimeout` fallback 会走同一次
`execute_script` 的 microtask 队列。Service Worker registration 投影依赖 `pollRegistration()`
通过 `setTimeout(..., 0)` 继续轮询状态；当 worker 生命周期事件在全量负载下晚于单次脚本的
microtask budget 时，轮询链会停止在旧的 `ServiceWorkerRegistration` / `ServiceWorker` 对象上。

后续测试循环虽然反复调用 `execute_script` 读取全局变量，但独立 `execute_script` 路径没有像
页面脚本路径那样调用 `__zw_begin_script()`，因此不会重置 microtask budget，也不会重新启动
Service Worker registration 投影轮询。底层 `ServiceWorkerManager` 状态已经推进，页面侧对象仍可能
停留在旧 active worker。

## 解决方案

把 Service Worker registration 投影刷新挂到 `__zw_begin_script()`，并让 WebView 的独立
`execute_script` 路径在执行用户脚本前调用该 hook，和 `run_page_scripts` 的 task-start 语义保持一致。
这样每次测试/嵌入方轮询脚本都会重新给 shim 的异步投影一次预算，而不是依赖上一批脚本内已经耗尽的
fallback timer 链。

排查同类问题时，不要只复跑单个 service worker 测试。单测通过而全量失败时，应检查：

- 进程内路径是否缺少真实 host timer；
- JS shim 的异步投影是否只依赖一次性 microtask budget；
- 后续 `execute_script` 批次是否调用了和页面脚本相同的 begin-script hook；
- 断言读取的是底层 manager 状态，还是页面侧 cached registration/controller 投影。

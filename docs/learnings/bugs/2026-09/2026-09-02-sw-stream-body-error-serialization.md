---
date: 2026-09-02
modules: script-sandbox, engine, service-worker
---

# Service Worker stream body errors must survive response serialization

## 问题描述

`service-workers/service-worker/fetch-error.https.html` 构造 `respondWith(new Response(stream))`，
其中 `ReadableStream` 先 `enqueue()` 一个 chunk，随后异步 `controller.error()`。规范期望
受控页面的 `fetch()` 先 resolve，随后 `response.text()` 在 body 消费阶段 reject。

ZeroWeb 原实现把 Service Worker `Response` body 在 `respondWith()` 边界同步快照为字符串。
当 body 是 stream 时，`normalizeBody()` 走 `String(stream)`，后续 stream error 被丢失，页面
`response.text()` 错误地 resolve，WPT 失败。

## 根因分析

Service Worker runtime 与页面 runtime 之间的 `ServiceWorkerFetchResponse` 是纯值结构。普通
string/blob/typed-array body 可以在 `respondWith()` 时快照，但 stream body 的错误是异步产生的，
必须先 drain stream，再把“body 已产生进展但最终 error”的状态跨边界传出去。

如果直接把 `respondWith()` 标为失败，会把 fetch promise 变成 reject，违背该 WPT 的核心语义：
网络响应已经开始，失败发生在 body 消费阶段。

## 解决方案

- 在 Service Worker sandbox 中提供最小 `ReadableStream` 支撑，覆盖 constructor、
  `controller.enqueue/close/error`、`getReader().read()` 与 timer 驱动的异步 error。
- `Response._serialize()` 对 stream body 异步 drain；成功时输出正常 body，失败时保持 response
  成功，但写入内部 header `x-zero-body-error`。
- 页面 `Response` shim 从 wire 中取出 `x-zero-body-error` 并从公开 headers 中移除；`text()`、
  `json()`、`blob()`、`arrayBuffer()`、`formData()` 和 `body.getReader()` 在 body 消费时 reject。

后续扩展 streamed response 时，应区分“fetch 结算失败”和“response body 消费失败”，不要把二者
折叠为同一个 `respondWith()` network error。

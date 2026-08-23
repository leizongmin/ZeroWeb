---
date: 2026-08-23
modules: service-worker,cache-api,js-dom
---

# Service Worker iframe Cache API must use the iframe fetch context

## 问题描述

导入 `fetch-event-within-sw.https.html` 后，受控页面里的 iframe 能加载
`simple.html`，但测试在 iframe 中执行 `contentWindow.caches.open()` 时直接抛
`TypeError`。补上 `caches` 后，`cache.add("sample.txt")` 仍容易误用顶层页面 URL、client id
或 referrer，导致 worker 拦截路径与 Chrome/WPT 不一致。

## 根因分析

`_zwMakeIframeWin` 为 iframe `contentWindow` 暴露了 `fetch`、`Request`、`Response`、
`XMLHttpRequest` 和 `navigator.serviceWorker`，但没有暴露 Cache API。WPT 通过 iframe
global 调 `caches.open().then(cache => cache.add("sample.txt"))`，这要求相对 URL 按 iframe
document URL 解析，并且后续 fetch 的 client/referrer 都来自 iframe，而不是顶层 window。

同类问题不能只靠底层 `Cache`/`CacheStorage` 存储语义修复；Cache API 的 `add()`/`addAll()`
本质上会发起 fetch，必须进入当前 global object 对应的 fetch client 上下文。

## 解决方案

iframe `contentWindow.caches` 应是 iframe-local wrapper：存储操作仍委托给主 Cache API
实现，避免复制 `match()`/`put()` 等校验语义；但 `open()` 返回的 cache 在 `add()` 和
`addAll()` 期间需要临时设置 iframe 的 fetch client id 与 referrer，并把字符串 request
相对 iframe document URL 解析。

验证时同时覆盖三点：`contentWindow.caches.open()` 可用；`cache.add("sample.txt")` 实际请求
iframe 目录下的资源且携带 iframe client/referrer；`contentWindow.caches.match()` 缺少 request
参数时仍抛主实现的 TypeError，避免 wrapper 静默绕过原有 Web API 校验。

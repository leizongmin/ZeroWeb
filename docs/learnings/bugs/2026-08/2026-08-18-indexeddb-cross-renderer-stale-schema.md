---
date: 2026-08-18
modules: engine/js_dom_shim, browser/process_backend
---

# IndexedDB cross-renderer stale schema after upgrade

## 问题描述

Renderer A 关闭 version 1 connection 后，renderer B 将同一数据库升级到 version 2。A 再次 `open()` 时仍使用 realm-local version 1 schema，并以错误版本向 browser connection owner 注册，导致请求不完成。

## 根因分析

数据和 schema 的权威状态已在 browser storage owner，但每个 renderer realm 仍缓存 `_idb_databases`。跨 renderer `versionchange` 只通知旧 connection，不会直接修改其 `IDBDatabase.version`。旧 connection 全部关闭后，realm-local cache 已无必须保留旧 snapshot 的对象，却仍被后续 reopen 复用。

## 解决方案

仅在 browser host 明确声明支持 cross-renderer connection ownership，且该 realm 对目标数据库没有 live connection 时，`open()` 先通过 host `inspect` 重建 schema cache。Embedded WebView 不启用该 capability，继续使用 realm-local queue 和 schema snapshot。

跨 realm schema 刷新必须同时满足：

+ host 是 schema authority
+ 当前 realm 没有 live connection
+ capability 明确启用，不能通过异常或 callback 存在性猜测

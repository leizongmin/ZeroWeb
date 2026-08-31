---
date: 2026-08-31
modules: engine, webview, service-worker
---

# Iframe navigation pending ids and load ordering

## 问题描述

Service Worker CacheStorage 的 `cache-keys-attributes-for-service-worker.https.html` 在连续 iframe 导航中会丢失 `Request.isReloadNavigation` 或 `Request.isHistoryNavigation` 的持久化结果。失败形态包括 reload 后仍读到第一次导航的 response，或 history traversal 回退到 fallback 文档。

## 根因分析

动态 iframe 的 `src`、append、reload、history traversal 共用同一个 browsing context 状态，但 shim 里存在三处隐式分叉：

- detached iframe 设置 `src` 时，不能用通用 `Node.isConnected` 的布局 rect 兜底判断来启动导航。该兜底适合 DOM 断言，但对导航触发过宽。
- iframe 元素 `load` 必须绑定到当前加载 entry 完成后派发；同一个 entry 从 `src` 路径和 append 路径各排一次 `load` 会让 Promise 链提前进入下一段导航。
- WebView 的异步 iframe navigation fetch 不能复用 `r115iframe:<frameKey>` 作为唯一 pending id。连续 reload/src/history 导航会覆盖结果槽，导致后续 `contentDocument` 读到旧 response。

## 解决方案

为 iframe 导航增加专用 connected 判定：selector-backed 节点按 host contains 判断，handle-only 节点只接受 `_zwNodeParent`/shadow/iframe body 父链证明，不使用 layout rect 兜底。

iframe element `load` 改为等待对应 entry settle 后派发，并给 entry 打 `_zwFrameElementLoadQueued` 标记，避免 append 和 `src` 路径重复触发同一 entry 的 load。

iframe async fetch id 增加单调序号，保留 `r115iframe:` 前缀给宿主识别 navigation，同时保证每次导航有独立 pending/result 槽。

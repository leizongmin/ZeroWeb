---
date: 2026-09-03
modules: engine/js-dom-shim, service-worker, wpt-runner
---

# Service Worker iframe controllerchange event state must be event-scoped

## 问题描述

`ServiceWorkerGlobalScope/error-message-event.https.html` promotion 后，完整
Service Worker core baseline 固定顺序执行时，
`skip-waiting-using-registration.https.html` 稳定失败：
iframe 的 `navigator.serviceWorker.oncontrollerchange` 事件里读到
`event.target.controller.state === "activated"`，而 WPT 期望事件期仍为
`"activating"`。单独运行该 WPT 可以通过，说明问题依赖前序 case 组合后的消息泵时序。

## 根因分析

主 window 的 `setController()` 已经在派发 `controllerchange` 前把新 controller 的状态临时定格为
`eventState`，但 iframe bridge 的 `__zwRefreshServiceWorkerController(hint, previous, eventState)`
在带 `hint` 路径下仍用 `hint.state` 构造 iframe 侧 controller。完整 baseline 中前序 case
让 host snapshot 更快推进到 `activated`，iframe 事件处理器因此读到了最新状态，而不是
`controllerchange` 事件发生时的状态。

## 解决方案

iframe bridge 构造 hint controller 时优先使用调用方传入的 `eventState`，再回退到
`hint.state`。这样 iframe 与主 window 共享同一个事件期状态契约：事件处理器内观察到
`activating`，事件后再回到 worker 的最新状态。

## 如何避免

Service Worker controller 投影有两类状态：host 最新 snapshot 和事件期 observable state。
新增 iframe / nested client / popup 桥接路径时，不要只从最新 snapshot 或 worker object 读
`state`；凡是用于派发 `controllerchange` 的 controller wrapper，都必须显式携带并优先使用
事件期状态。

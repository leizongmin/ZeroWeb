# Renderer deferred IPC 消息自旋

**日期**：2026-08-08

**相关模块**：`zero-renderer`、`zero-protocol`

## 问题描述

页面已经显示完成后，单个 renderer 仍长期占用接近 100% 的单核 CPU。browser 主进程和其他 renderer 可以保持空闲，因此问题不在窗口合成或全局事件循环。

macOS `sample` 显示高占用 renderer 的主线程持续停在：

`RendererRuntime::run → rerender_publish_webview → publish_webview → fetch_image_payloads_with_cache → ipc_fetch_get`

## 根因分析

`ipc_fetch_get` 同步等待指定 `FetchResponse` 时，会优先读取 `deferred_inbound`。旧实现遇到非目标消息后立即将其放回同一个队列尾部，下一轮又从该队列头部读取。

当队列中只有一条 `SetViewport`、其他请求的 FetchResponse 或任意非目标消息时，执行路径变成：

`pop_front → push_back → pop_front → push_back`

循环不再阻塞等待新的 IPC 消息，导致 renderer 主线程单核自旋，目标 FetchResponse 即使已到达 `inbound_rx` 也无法被读取。

## 解决方案

同步等待期间将非目标消息移入局部 `skipped` 队列。仅在目标响应到达或等待失败后，才把这些消息按原顺序恢复到 `deferred_inbound`。Heartbeat 仍即时响应，不进入暂存队列。

回归测试必须覆盖“deferred 中预存非目标消息，inbound 中已有目标 FetchResponse”的场景，并断言：

+ fetch 正常返回目标 body；
+ 非目标消息没有丢失；
+ 非目标消息顺序保持不变。

## 如何避免

+ 消息暂存队列不能在同一个消费循环中同时作为优先输入和回退输出。
+ 等待特定响应时，无关消息应使用独立局部队列隔离，退出等待后再恢复。
+ 遇到“页面已完成但单进程单核满载”，优先做线程栈采样，区分计算热点、阻塞等待和用户态自旋。

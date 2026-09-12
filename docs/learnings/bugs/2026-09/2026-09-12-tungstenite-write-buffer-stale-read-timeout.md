---
date: 2026-09-12
modules: apps/browser
---

# tungstenite write() 缓冲不落盘 + peek 阶段 read timeout 未恢复 → CDP 客户端首连必败

## 问题描述

Playwright `connectOverCDP` 连接 ZeroWeb headless 服务器：WS 握手成功后客户端发送
`Browser.getVersion`，服务器**处理了消息但客户端永远收不到响应**；约 5 秒后服务器日志报
`WebSocket read error: IO error: Resource temporarily unavailable (os error 11)`（EAGAIN）
并断连，客户端以 1006 异常关闭收场。dispatch 层逻辑完全正确（单测全绿），纯传输层问题，
且无任何真实 WS 往返测试覆盖（既有测试全部 dispatch 直连），故长期未暴露。

## 根因分析

两个 bug 叠加，都在 `apps/browser/src/headless/mod.rs` 的连接循环：

1. **tungstenite 0.29 `write()` 不保证落盘**：`WebSocket::write()` 内部把帧写入输出缓冲，
   只有 `_write` 返回 `should_flush=true`（缓冲不足以容纳帧等场景）才自动 `flush()`。
   小体积 JSON 响应恰好走「入缓冲、不 flush」路径，滞留在进程内缓冲里。
2. **5s read timeout 从未恢复**：accept 循环用 `stream.set_read_timeout(5s)` 给 HTTP/WS
   分流的 `peek()` 兜底，但 WS 分支接受连接后没有重设。客户端等待永不到来的响应（bug 1）
   → 服务器 5s 无数据可读 → `read()` 返 EAGAIN → `break` 断连。

## 解决方案

1. WS 循环内所有 `ws.write(...)` 之后显式 `ws.flush()`（pong / 事件 / 认证错误 / 命令响应
   四处）。
2. WS 分支 `accept(stream)` 之前 `stream.set_read_timeout(600s)`——CDP 客户端连接后可能
   长时间静默等事件，沿用 5s 会误杀健康连接；取大值兜底而非 `None`，避免半开连接永久
   阻塞单线程 accept 循环。

## 如何避免

- 传输层协议实现必须有**真实 WS 往返**测试（真实 TCP/WS 客户端对真实服务器），dispatch
  直连测试验证不了缓冲/flush/超时这类 socket 语义。cdp-protocol goal 的 Playwright E2E
  （`tests/playwright-matrix/`）即是这层防线。
- 对「peek/探测阶段设短超时」的模式，分流完成后必须恢复业务态超时；探测参数不应泄漏到
  长生命周期会话。

# Image Stream File Descriptor Exhaustion

- 日期：2026-08-17
- 相关模块：`apps/browser/src/fetch_proxy.rs`、`crates/net/src/client.rs`

## 问题描述

复杂页面加载大量图片后，Browser 主线程在创建 `reqwest::blocking::Client` 时因 `Too many open files` panic。安全策略阻止 mixed-content 图片的日志与崩溃时间接近，但不是崩溃原因。

## 根因分析

普通资源请求经过全局和 per-origin 并发调度，流式图片路径却为每个请求直接启动异步任务，没有连接并发上限和排队上限。每个流任务还调用 `HttpClient::new()`，额外创建一个该异步路径不会使用的 blocking client。大量图片请求因此同时占用 socket 和重复连接池资源，最终触发 `EMFILE`；blocking client 构造使用 `expect`，把可恢复的资源错误升级为主线程 panic。

## 解决方案

流式图片使用共享异步连接池，并通过全局 Semaphore 限制并发数；排队请求数量超过上限时返回确定失败。blocking client 按 timeout、代理和 HTTP2 配置进程级复用，构造失败保存在 `HttpClient` 中并由同步 `send` 返回 `NetError`，不再 panic。DNS prefetch 同样改用不构造 blocking client 的静态异步入口。

## 如何避免

所有能打开 socket 的旁路都必须复用统一资源预算。异步 API 不应为满足结构体构造而初始化未使用的同步连接池；操作系统资源不足应沿错误边界返回，不能使用 `unwrap` 或 `expect`。

---
date: 2026-08-14
modules: zero-net 的 HttpClient, 资源调度器 async 迁移
---

# blocking reqwest 不可在 Tokio worker 内析构

## 问题描述

将 `HttpClient::new().send_async(...)` 直接放进 Tokio worker 后，POST fixture 在 worker 中 panic：Tokio 不允许在 async context 内析构一个内部含 blocking runtime 的对象。

## 根因分析

`HttpClient` 持有 `reqwest::blocking::Client`。临时 `HttpClient` 在 async task 结束时析构，blocking client 内部 runtime 的 shutdown 会执行阻塞操作，Tokio 因此拒绝该析构。

## 解决方案

async 调度路径只使用不持有 blocking client 的 async transport；在两套 client 内部状态彻底拆分前，unsafe write-through 保持在 blocking worker。不要在 Tokio task 中构造或析构 `HttpClient`。

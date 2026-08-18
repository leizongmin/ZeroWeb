---
date: 2026-08-14
modules: zero-webdriver
---

# 并行测试中的空闲端口 TOCTOU

## 问题描述

WebDriver HTTP 集成测试单独运行稳定通过，但在 workspace 全量测试中，首个 New Session 请求偶发收到 `Connection reset by peer`。

## 根因分析

测试先用 `TcpListener::bind("127.0.0.1:0")` 获取空闲端口，释放 listener 后再启动服务子进程。并行测试可在子进程完成 bind 前再次取得同一端口。后启动的服务绑定失败退出，但 readiness probe 可能误连到另一条测试的服务，因此问题直到首个真实请求或另一条测试清理进程时才暴露。

## 解决方案

对同一 test binary 内的“选择端口、启动子进程、确认服务完成 bind”窗口加互斥。readiness 成功后立即释放锁，测试主体仍可并行。

如果服务支持继承已绑定 socket，应优先直接传递 listener，从根本上消除探测与 bind 之间的空窗。仅接受端口号的 CLI 测试服务，可用上述短临界区限制并发竞争。

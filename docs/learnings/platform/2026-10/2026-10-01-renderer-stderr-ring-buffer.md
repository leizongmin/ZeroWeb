---
date: 2026-10-01
modules: protocol, renderer
---

# renderer 进程 tracing 日志不落浏览器 stderr（内存环形缓冲）

## 问题描述

实站排障时给 `zero-browser` 设 `RUST_LOG=info` 并捕获浏览器 stderr，但 renderer 进程内
的 `tracing::warn!`（如 page_scripts 的 `apply DOM mutations: {e}`）从未出现在捕获文件中
——文件里只有主进程自己的日志（如网络栈 mux-debug）。

## 根因分析

多进程 spawn 点 `zero-protocol` 的 `RendererHandle::spawn`（crates/protocol/src/process.rs）
对 renderer 子进程显式 `stderr(Stdio::piped())`，随后把管道交给
`spawn_stderr_reader(id, stderr, stderr_tail)`：renderer 的 stderr 被**读进内存环形缓冲**
（`STDERR_TAIL_LIMIT` 条），仅当 renderer 崩溃/退出时随诊断报告倾倒。这是 S78 诊断设施
（reader 静默退出不可见）的设计取舍：renderer 平时日志不污染浏览器 stderr。

注意对照：compositor 的 spawn（apps/browser/src/compositor_client.rs）用
`stderr(Stdio::inherit())`，所以 compositor 日志能直接出现在浏览器 stderr——两个进程
家族行为不同，容易误判「日志丢了」。

## 解决方案

需要 renderer 侧运行时证据时，不要指望浏览器 stderr：

- 短期：在目标位点把证据经 IPC/CDP 面暴露（如 ConsoleLog → CDP
  `Runtime.consoleAPICalled`，页面 JS console 可捕获），或临时改成 eprintln 到
  inherit 的通道（改代码须评估是否值得）。
- 评估中：给 renderer stderr_tail 增加「按需倾倒」的诊断命令（崩溃之外也可读取），
  属产品改动，需单独立项。

## 如何避免

任何「给 renderer 加 warn 再从浏览器 stderr grep」的排障计划都会空手而归。先确认目标
进程的 stderr 路由（renderer=piped 环形缓冲，compositor=inherit），再设计取证手段。

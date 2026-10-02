---
date: 2026-10-02
modules: zero-browser
---

# 日志写 stdout 管道无读取者：64KB 缓冲写满后全进程停摆

## 问题描述

bilibili 验收跑中 ZeroBrowser 被 playwright/CDP 自动化以管道 spawn 后，`goto` 必超时、导航/CDP 派发全部停摆；Example 等低日志量页面幸存。同一二进制终端直跑正常。

## 根因分析

`tracing_subscriber::fmt()` 默认 writer 是 **stdout**。宿主（playwright connectOverCDP）只读 stderr，stdout 管道无读取者：Linux 管道缓冲 64KB 写满后，`write` 阻塞——所有打日志的 browser 线程相继卡死在日志调用上，主循环随之停摆。日志量小的页面永远写不满缓冲，掩盖缺陷（与站点复杂度无关，纯属日志量阈值）。

修复（fix#17）：`init_logging` 全部四个初始化点改 `with_writer(io::stderr)`。Chromium 日志同样走 stderr；stdout 留给程序输出（如 `--version`）。

## 解决方案

- 长驻进程的日志 writer 永远选 stderr 或文件，不选 stdout——stdout 语义是程序输出，且宿主常不读。
- 「低日志量场景正常」不是无缺陷证据，只是未达触发阈值；管道类资源（缓冲有限、写阻塞）的故障形态是渐进全停而非报错。
- 诊断同类停摆时先查线程栈卡点（/proc/PID/task/*/stack 或 wchan）：全部卡在 write = 管道无读取者，不是业务死锁。

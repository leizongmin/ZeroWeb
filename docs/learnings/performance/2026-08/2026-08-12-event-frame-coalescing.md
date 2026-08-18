---
date: 2026-08-12
modules: page-runtime, renderer, browser process_backend
---

# 页面输入事件的帧合并与 IPC latest-wins

## 问题描述

页面已经具备 retained 表单值和 value-only 局部绘制，但一次鼠标、键盘或 IME 消息仍可能经过 DOM 监听器、焦点切换和默认动作，沿多个辅助函数立即发布页面帧。可选的异步发布线程使用 8 槽 FIFO，积压时会继续发送已过期帧，放大输入延迟。

## 根因分析

增量渲染解决的是单次 mutation 的计算范围，并不会自动建立事件事务边界。发布调用散落在事件派发和默认动作中，使同一外部事件产生多次 IPC 帧。FIFO 只把阻塞移到后台，不能减少过期工作。

异步线程与主线程还各持一个 `PipeTransport`。如果共享 writer 只在每次 `write()` 时加锁，4 字节长度头和消息体可能被另一个发送端插入，破坏 IPC 帧边界。

## 解决方案

1. 用只升级的 `FrameInvalidation` 表达 style/layout/paint/composite/publish/hit-test 依赖。
2. 把一个外部输入消息及其同步 JS 回调包进 `FrameTransaction`，最外层退出时最多发布一次。
3. 导航时丢弃旧文档尚未提交的事务，禁止旧快照冒用新 epoch。
4. 异步页面帧使用单槽 latest-wins 邮箱；消费者忙时直接替换未发送的旧帧。
5. 每个发送端先在本地收集完整 IPC 帧，在 `flush()` 时持共享锁一次性写入，保证多发送端帧原子性。

## 如何避免复发

- 新输入路径不得直接绕过事务边界发布。
- 队列用于实时 GUI 帧时优先 latest-wins；只有不可丢控制消息使用 FIFO。
- 多个 framed-protocol writer 共享字节流时，锁粒度必须覆盖完整帧而不是单次 write。
- 自动测试同时断言事务发布次数、latest-wins 和完整帧写入边界。

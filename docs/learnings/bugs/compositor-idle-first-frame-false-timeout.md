# Compositor 空闲后首帧被误判超时

日期：2026-08-14 ｜ 模块：browser/compositor_client、browser/process_backend

## 问题描述

页面正常完成若干帧后，空闲十几秒再产生新帧，browser 会立即报告
“帧响应超时（10s 无响应）”并切换到 legacy。compositor 通常在告警后几毫秒内仍能
完成该帧，进程和 IPC 实际没有断开。

## 根因分析

旧看门狗在 UI 入队时更新 `last_frame_sent`，却用
`now - last_response > 10s` 判断超时。空闲超过 10 秒后，新帧刚入队就满足
`last_frame_sent > last_response`，因此本次请求尚未真正发送便会立即超时。

持续提交新帧时，如果简单改成 `now - last_frame_sent`，时间戳又会被 UI 不断刷新，
反而可能永久掩盖真正卡死。因此 UI 入队时间不适合作为 IPC 请求看门狗基准。

## 解决方案

- 看门狗状态由独占管道的 compositor worker 维护。
- worker 开始发送 `CompositorFrame` 时启动计时。
- 收到 `CompositorFrameResult` 等阶段性响应时刷新进度时间。
- 收到最终 `CompositorFrameData` 时清空 in-flight 状态。
- UI 线程只检查真实 in-flight 请求是否连续 10 秒没有进展，不再用历史响应时间推断。

## 如何避免

异步请求的超时基准必须绑定到“实际开始执行的请求”，不能绑定到调用方入队时间，
也不能用系统最近一次成功响应时间代替。多阶段协议还应在每次有效进展时刷新计时，
并为“长时间空闲后的首个请求”建立常驻回归测试。

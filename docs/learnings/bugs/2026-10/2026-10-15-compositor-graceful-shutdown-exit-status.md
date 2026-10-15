---
date: 2026-10-15
modules: apps/browser compositor_client, apps/compositor process shutdown
---
# compositor 关闭不能先杀进程再读取退出状态

## 问题

Workspace 测试中 browser 目标曾报告全部测试通过，但其子 compositor 在关闭管道后输出 `corrupted size vs. prev_size`。browser 的 `reap_child` 立即 kill 仍运行的 compositor，并丢弃 `Child::wait()` 返回值；`Client::shutdown` 也在关闭命令队列后立刻 kill。这样既可能在 compositor 正常处理 EOF 前打断其清理，也让父测试无法检查最终退出状态。

## 根因与修复

关闭命令队列后，worker 需要退出当前 IPC 并关闭管道，compositor 才会正常看到 EOF、完成清理。先 kill 会与该过程竞态。现在 shutdown 最多等待 500ms 让 child 自行退出；超时仍会 kill，避免阻塞。wait 状态向调用者返回；正常连接关闭时的非零状态会记 error。新增真实子进程回归，要求 idle compositor 经关闭后在时限内以成功状态退出。

## 验证与避免方式

- 受守卫的单项回归：`shutdown_gracefully_reaps_compositor_and_reports_exit_status` 通过；子进程记录正常 `通道关闭，退出`，未再打印堆损坏。
- 完整 V8/QuickJS `make guarded-test` 通过；browser 二进制矩阵 564 通过、1 忽略；完整输出无 `corrupted size vs. prev_size`。
- 进程外组件测试不可只看父 Rust 用例汇总。关闭子进程时应给协议级 EOF 一个短的有界宽限，最后读取并验证退出状态；卡死仍需有强制终止上限。

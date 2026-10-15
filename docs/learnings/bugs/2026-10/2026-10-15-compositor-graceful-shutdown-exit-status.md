---
date: 2026-10-15
modules: apps/browser compositor_client, apps/browser process_backend, apps/compositor shutdown
---
# compositor 关闭不能先杀进程再读取退出状态

## 问题

Workspace 测试中 browser 目标曾报告全部测试通过，但其子 compositor 在关闭管道后输出 `corrupted size vs. prev_size`。browser 的 `reap_child` 立即 kill 仍运行的 compositor，并丢弃 `Child::wait()` 返回值；`Client::shutdown` 也在关闭命令队列后立刻 kill。这样既可能在 compositor 正常处理 EOF 前打断其清理，也让父测试无法检查最终退出状态。

## 根因与修复

关闭命令队列后，worker 需要退出当前 IPC 并关闭管道，compositor 才会正常看到 EOF、完成清理。先 kill 会与该过程竞态。现在 shutdown 最多等待 500ms 让 child 自行退出；超时仍会 kill，避免阻塞。Compositor client 与 `ProcessTabBackend::shutdown_all` 返回 child 退出状态；非零状态记录 error。测试构建的 backend Drop 断言状态为成功，service-worker restart 回归在两次 backend 回收后分别检查真实 compositor 退出码。

## 验证与避免方式

- `shutdown_gracefully_reaps_compositor_and_reports_exit_status` 在空闲真实 compositor 上通过，子进程记录正常 `通道关闭，退出`。
- `multiprocess_restart_restores_persisted_controller_without_refetch` 现在检查两次真实 compositor 回收的退出状态，定向运行通过；browser 全量目标 564 通过、0 失败、1 忽略。
- 完整 V8/QuickJS `make guarded-test` 在本次进展前通过，含重复-Vary/缓存和子进程回收回归；该次完整日志无 `corrupted size vs. prev_size`。后续完整矩阵应继续确认新加入的两个退出码断言。
- 进程外组件测试不可只看父 Rust 用例汇总。关闭子进程时应给协议级 EOF 一个短的有界宽限，并把退出状态传回可能检查结果的宿主；卡死仍需有强制终止上限。

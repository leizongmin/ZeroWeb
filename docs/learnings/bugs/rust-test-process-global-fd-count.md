# Rust 并行测试中的进程级 FD 计数误报

- 日期：2026-08-14
- 相关模块：`zero-protocol`

## 问题描述

FD 泄漏回归测试通过 `/proc/self/fd` 比较测试前后的进程打开 FD 总数。用例单独运行稳定通过，但在 `cargo test --workspace` 中会报告净增 5 至 6 个 FD。

## 根因分析

Rust test harness 默认在同一进程内并行运行用例。`serial_test::serial` 只会串行化使用相同 serial 锁的测试，无法阻止未标记的其他测试同时创建 socket、pipe 或子进程。因此 `/proc/self/fd` 统计包含无关用例的 FD，不能作为当前用例的隔离观测。

## 解决方案

保留 FD 总数的严格泄漏判据，但由父用例通过当前 test binary 的 `--exact` 参数启动单用例子进程。子进程设置内部环境变量防止递归，并以 `--test-threads=1` 执行实际断言。这样无需提高容差，也能继续检测每轮泄漏一个 FD 的线性增长。

凡是测试依赖进程级资源总量、环境变量或其他全局状态，都应先确认 test harness 并行模型。局部 serial 锁不能隔离未参与同一锁的测试。

---
date: 2026-10-02
modules: zero-renderer
---

# renderer 主函数返回 ≠ 进程退出：join 长臂把退出拖成泄漏进程

## 问题描述

Browser 断连后 renderer 日志已打「renderer exiting」，进程却继续存活且 100% CPU 挂 10 分钟以上。泄漏的 renderer 持续抢占 CPU 核，干扰后续测试/基准（噪声毒化），多次出现后才发现是同一进程族。

## 根因分析

`run_desktop_role` 返回只结束主函数的控制流；main 返回后的进程退出要等所有非 daemon 线程终结。renderer 的 JS worker 线程可能仍在 30s 级长臂中（页面脚本 timer 回调），`Drop→shutdown` 的 join 语义是优雅排空——主函数「返回」后实际阻塞在 join 上等长臂跑完。修复（fix#16）：`run_desktop_role` 返回即 `std::process::exit(0)`——Browser 断连后 renderer 无需排空（无落盘状态，状态真值在 browser 进程），与 Chrome renderer 的退出语义一致。

## 解决方案

- 「主函数返回 = 退出」只在所有线程都可随时终结时成立；有长臂 worker 的进程要么 shutdown 语义支持中断，要么在无状态前提下进程级退出。
- 测试/基准出现无主 100% CPU 进程时，先 `ps -ef | grep` 查泄漏的旧进程——「噪声」可能是上一轮的僵尸 worker，不是当前被测代码慢。
- 排空（join）与放弃（exit）是两种合法退出语义；选择依据是**状态真值在谁手里**，不是哪个更优雅。

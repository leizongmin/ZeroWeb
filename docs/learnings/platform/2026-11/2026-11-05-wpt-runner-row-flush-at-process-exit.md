---
date: 2026-11-05
modules: wpt-runner
---

# wpt-runner testharness 行输出进程末统一落盘——stdout 静默 ≠ 挂死

## 问题描述

multitest.html 全页（10 variant，628 条目 ~9,811 行）跑批期间 stdout
20+ 分钟零字节，连续多轮被误判为「挂死/慢化」：催生 variant 裁剪
bisect、30 分钟 0 行误账、乃至并行实例 gdb ptrace + SIGTERM 误杀健康
runner（dmesg 铁证 2026-11-05 03:07:52）。

## 根因分析

`zero-wpt-runner testharness-editing-run` 通道（main.rs
`run_observers_cmd`）收集**全部** cases 后在**进程末统一 println**
（testharness.rs:1626-1636）——运行中 stdout 静默是设计行为，不是
进度信号。每 variant 行在 variant 结束时入 collector，但 collector
不落盘，直到整页跑完。

叠加误诊因素：

- 页内 `subsetTest` 按子测试计数门控（~16.5 行/条目），`?1-1000`
  窗 = 前 ~60 条目全断言 + 全 628 条目非门控内联段（setup/双
  documentElement clone/前后查询），行数与条目数不是一比一。
- 并发 cargo/make 争抢把健康 ~85ms/条目放大到 ~10s/条目，制造
  「3× 慢化」假象（2026-11-05 实测：独占忠实复刻 probe 75-85ms/
  条目 vs 并发期 cropped 真实页 10.8s/条目，127×）。

## 解决方案

- 判活不看出 stdout：看进程树存活（`pgrep`/`systemctl is-active`）
  与 CPU 时间增长（`ps -o time`），或页尾插桩探针（diag2 形态）。
- 全页跑批用 `systemd-run --user --unit=<name>` 脱离 session 撕裂
  （WorkingDirectory/PATH 必设），跑批纪律记入
  [editing-compat evidence](../../goal/editing-compat/evidence/2026-11-05-qv-peel-asym-recursion-break.md)。
- 单页短窗用 variant 裁剪跑法（每 variant ~2.5 分钟独立出账）替代
  全页长跑做判读。
- 长 wall-clock 测量必须独占：并发 cargo build 会引入量级级失真。

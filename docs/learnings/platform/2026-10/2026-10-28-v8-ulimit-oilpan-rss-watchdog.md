---
date: 2026-10-28
modules: tests/wpt-runner,engine
---

# V8 宿主进程不能用 ulimit -v 封内存——外部 RSS 看门狗替代

## 问题

无人值守跑 zero-wpt-runner（内嵌 V8）大语料，按「ulimit -v 8G 直跑二进制」
护栏执行，进程立即死：`Fatal process out of memory: Oilpan: CagedHeap
reservation`（8G 与 16G 同样失败，core dump EXIT=133）。

## 根因

V8 启动时一次性 reserve 大块虚拟地址空间（Oilpan caged heap 等），`ulimit -v`
按**虚拟内存**上限拦截 reserve 动作——与实际 RSS 远小于上限无关。V8 宿主
（runner、browser renderer）在 ulimit -v 下无法启动。

## 解决方案

- V8 宿主进程的无人值守内存封顶改用**外部 RSS 看门狗**：旁路脚本轮询
  `/proc/<pid>/status` VmRSS，超阈值杀进程树（如 8GB 阈值）。
- test-guard 的 `--per-proc-mem/--total-mem` 即此机制（外部监控），make 目标
  内跑 runner 无需额外处理；绕过 make 直接跑二进制时不得用 ulimit -v。
- fuzz 复现等非 V8 二进制不受此限（ulimit -v 仍可用，见 2026-10-07 记录）。

---
date: 2026-10-11
modules: apps/renderer
---

# QuickJS 全局词法作用域重复 `const` 声明使脚本静默死亡

## 问题描述

r2946 快照屏障 F1 返修的双扫描交错回归测试
（`snapshot_barrier_preserves_cross_round_fifo_order_r2946`）首轮运行
0.79s 即失败，`__order_done` 为 `r,t2` 而非预期的任一派发序——被观测的
优先脚本 T1（含 800ms 忙臂）的副作用完全缺失，忙窗塌缩，测试没有覆盖到
目标代码路径。

## 根因分析

同一 worker 的多次 `Execute` 命令共享同一个全局词法环境（QuickJS 全局
scope）。测试里忙臂脚本声明了 `const __t = Date.now()`，T1 脚本再次声明
`const __t`——第二个脚本在解析期抛 `SyntaxError: redeclaration`，整段
脚本不执行（无副作用、无异常上报到测试断言的任何全局）。V8 下两个独立
`<script>` 各自的顶层 `const` 同样共享全局词法环境，行为一致，非 QuickJS
特有。

与页内 `<script>` 标签不同：多个 `<script>` 标签间顶层 `let/const` 重名
同样互斥——这是规范行为（global lexical environment 单例），不是执行器
缺陷。

## 解决方案

- 测试脚本内的计时/临时变量一律用唯一专名（`__t_arm`、`__t_t1`），或包
  IIFE `(() => { const __t = ...; ... })()` 隔离词法作用域。
- 排查手段：给各阶段脚本埋 `globalThis.__log` 时序日志后 `--nocapture`
  跑单测，缺段即定位到「脚本未执行」而非「执行序错误」——两类失败的
  修复方向完全不同。

---
date: 2026-11-11
modules: rally,parallel-streams
---

# 多 clone 并行时 ps 全局 grep 等场环会卡在异 clone 进程上

## 问题描述

共享树多流协作中，链脚本用 `ps ax -o command= | grep 'zero-wpt-runner'`（或
`make test`、`test-guard` 等模式）实现"他流在跑则等待"的等场环。实测该环被
**另一个 clone（ZeroWeb-3）的 `make testharness-editing-run-one`** 卡住 ~30
分钟：异 clone 的 runner 命令行同为相对路径 `./target/release/zero-wpt-runner`，
与本树进程在 ps 输出里不可区分，而它对本树的 release 重建零毒化（独立
target 目录、独立二进制）。该链 TIME_LIMIT=7200，最坏空等 ~1.5h。

## 根因分析

`ps` 全局快照只含命令行，不含 cwd/exe 树归属；相对路径进程名在多 clone
共存的机器上是歧义的。等场环的真实关切是「会不会毒化本树产物」（换二进制、
抢 target 锁），异 clone 进程与此无关，却被无差别计入。

## 解决方案

等待判据加树归属过滤：对候选 PID 读 `/proc/<pid>/cwd` 与
`/proc/<pid>/exe`，只认目标 clone 根的路径前缀（判据串即本仓根目录名
`ZeroWeb-2`，详见下方示例）。
落地示例（run-fd1.sh，fd 流采纳）：

```bash
if pgrep -f 'zero-wpt-runner' | while read p; do
     readlink "/proc/$p/cwd" "/proc/$p/exe" 2>/dev/null
   done | grep -q 'ZeroWeb-2'; then
  sleep 30   # 仅本树 runner 在跑才等待
fi
```

另注意 pgrep 用括号形式防自匹配，`grep` 形态需 `grep -qv grep`（相对路径
段同样会命中 grep 自身命令行）。判毒化前先看 `Finished` 秒数判腿（同树
情况见 battery 窗口先例），树归属是第一层过滤。

---
date: 2026-11-11
modules: Makefile,test-guard
---

# make test 外层兜底三连杀——RSS 求和守卫不配双腿并行编译，顶层守卫归 cgroup

## 问题描述

无人值守 `make test` 外层再包 test-guard 兜底内存时，`--total-mem` 12G/16G/24G
三档全部在**编译期**被杀（`总内存超限 … 已杀死进程树`，EXIT=124），实测树内
RSS 峰值 13.4G → 17.7G → 26.5G 随编译推进递增，看起来像预算永远不够；每次
kill 还丢掉在途 rustc 进度，重跑从半冷状态再编译，形成 doom loop。

## 根因

两点叠加：

1. Linux `make test` 主 recipe **并行双腿**：`cargo test --workspace … &` 与
   quickjs feature `cargo clippy … &` 同时 `wait`。两棵 rustc 编译树并行推进，
   深水区（browser/renderer 等大箱体 + clippy 树）各自数 GB。
2. test-guard 的总内存按进程树 **RSS 求和**——rustc 间共享的 .rlib/.rmeta mmap
   映射被逐进程重复计数，RSS 和显著高于真实物理占用；文档 16G 宽松形态
   （`--per-proc-mem 6 --total-mem 16 -- cargo test --workspace`）针对**单条**
   cargo 命令，从未覆盖 make 腿并行叠加。

## 解决

Linux 顶层守卫用 systemd scope cgroup（与 oom-guard L2 同源）：

```
systemd-run --user --scope -p MemoryMax=24G -p MemorySwapMax=0 -u <unit> \
  bash -c 'timeout 5400 make test'
```

cgroup 按 unique page 计费，共享映射不重复算，24G 上限实测容得双腿编译全峰；
精细限交给 Makefile 各腿**自带**的 per-leg test-guard
（`--compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900`），墙钟由
`timeout` 兜底。外层 test-guard 仍适合单进程树命令（corpus runner、单页
testharness 等）；四跑实证：cgroup 形态完整走完编译 + 147 个测试套件。

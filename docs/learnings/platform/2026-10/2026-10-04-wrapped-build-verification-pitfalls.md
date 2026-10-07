---
date: 2026-10-04
modules: docs/rally
---

# 包裹构建的三个假阳性/假破损陷阱（看门狗空转、pgrep 自匹配、cd 后台化）

## 问题描述

无人值守集成验收里，同一构建命令连出三轮假象：

1. 构建日志 0 字节、exit 124——看似超时被杀，实际**根本没开始编译**；
2. `pgrep -f rustc` 报告 BUILDING——实际在跑的只是看门狗自己的 bash（命令行参数里含 "rustc" 字样）；
3. 构建报源码级编译错误（E0063 missing field）——**被编译的树根本不是目标树**（落在了主仓某个 detached HEAD 旧本地线上），目标工作树完好。

## 根因分析

- 本机内存看门狗（/tmp/zw*-memguard.sh 型）是**旁视进程**：循环 `pgrep` 监控进程树 RSS 超限即杀，**不执行传入的命令**。把它当执行器包在 cargo 外面（`timeout N memguard.sh LIMIT cargo build …`），看门狗只会空转到 deadline。
- `pgrep -f <pattern>` 匹配整条命令行——看门狗进程自身的 argv 里就含 pattern，永远自命中。
- shell 里 `cd X && setsid Y … &` 的 `&` 作用于整个 `A && B` 列表：cd 连同看门狗一起进了后台子壳，后续 cargo 留在原 cwd 跑——构建了错误的树。GitHub MERGEABLE 只保证文本无冲突；拿错误树的编译失败反推"合并破损"会得出完全错误的结论。

## 解决方案

正确语句结构（独立语句，不用 `&&` 链接后台段）：

```bash
cd /path/to/target-worktree
setsid /tmp/zw28-memguard.sh 10240 >/dev/null 2>&1 </dev/null &   # 旁路看门狗
MG=$!
env -u http_proxy … timeout 3600 cargo build … > build.log 2>&1   # 前台真构建
EC=$?; kill -9 $MG 2>/dev/null
```

三轮验证口令：

1. **真在编**：`grep -c "^   Compiling" build.log`（且源路径 == 预期工作树）> 0；
2. **进程实态**：`pgrep -x rustc`（精确进程名）而非 `pgrep -f`；
3. **树身份**：构建日志里的源路径前缀与目标工作树一致（cargo 输出 `Compiling X (/abs/path)` 直接可核）。

## 如何避免

- 包裹器上线先自测一次小命令，确认它会真的执行目标命令。
- 编译失败的定罪链必须包含"编译的是哪棵树"这一环；路径不对，一切源码级结论作废。
- 短超时包装长探针会制造"挂起"假象：给探针的预算要先问其内部上限（如 s26o-final 自带 500s），外部 timeout 必须 ≥ 内部上限。

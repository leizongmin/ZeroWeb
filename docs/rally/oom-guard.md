# OOM 防护（rally / 无人值守开发）

## 背景

某次 css-parser 改动引入「未闭合括号 + EOF」的死循环：声明值收集 loop 里
`group_depth > 0` 时 `Eof` 不 break，`advance()` 越界后 `peek()` 永远返回 Eof，
loop 无限 `push_str`，单进程吃 **47GB 物理内存 / 135GB 虚拟内存**，触发系统级
OOM，内核回收整个 `app-tmux.slice`（含 tmux session），rally 无人值守流程被
反复整垮。本文件记录防护分层。

## 防护分层

1. **L1 测试进程隔离（跨平台，主防线）**：`make test` / `make reftest` 先完成不受
   内存阈值限制的编译，再通过 `scripts/test-guard.rs` 包裹运行阶段；单进程 RSS > 6GB 或全树 > 16GB 或总时长
   > 1800s 即杀掉整棵进程树（退出 124）。macOS / Linux 通用，不依赖
   `ulimit -v`（macOS 无效）或 `timeout`（macOS 默认无）。
2. **L2 Linux cgroup 兜底（本文件）**：把 rally 跑在限内存的 systemd scope
   内。L1 万一失效（agent 裸跑 `cargo test`、或非测试命令爆内存），OOM 只
   杀 scope，不动 tmux server。
3. macOS 无 cgroup，仅依赖 L1。

## L2：在限内存 scope 内跑 rally（仅 Linux）

```bash
systemd-run --user --scope --unit=rally-oom-guard \
  -p MemoryMax=32G -p MemorySwapMax=8G \
  rally run "你的任务" -w ~/work/ZeroWeb
```

- `MemoryMax=32G`：rally + agent + cargo test 全树硬上限（47GB 机器留约
  15GB 给系统 / tmux / 其他进程）。
- `--scope`：当前进程树归入新 scope，OOM 时 systemd 只杀该 scope 内进程。
- tmux server 不在该 scope，不受影响——这正是「整垮 session」问题的根治点。

## 验证 scope 生效

```bash
systemctl --user status rally-oom-guard.scope
cat /proc/$(pgrep -f 'rally run')/cgroup   # 应含 rally-oom-guard.scope
```

## 临时调整 L1 阈值

`test-guard` 阈值可按需覆盖（不改源码）：

```bash
# 放宽（大测试）
./target/test-guard --compile-first --per-proc-mem 12 --total-mem 32 -- cargo test --workspace
# 收紧（更快拦截测试运行时内存异常；不会限制编译）
./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 -- cargo test --workspace
```

## L3：target/ 磁盘占用守卫（2026-08-18）

内存之外的姊妹问题：长时间 rally 循环中 `target/` 只增不减（多 feature 组合产物 +
incremental 缓存），曾把整块磁盘跑满。`scripts/target-disk-guard.sh` 作为重型
make 入口（build / test / browser / reftest / product-smoke / bench / Android 构建家族）的前置 prerequisite：

- **每次执行**都清仓库根 `core.*` OOM 转储（git 不追踪，堆积无上限；2026-08-18
  实测积了 23 个 / 973MB）。
- `target/` 占用超过 **50GB**（`ZW_TARGET_DISK_LIMIT_GB` 可调）时先删除 incremental 缓存；
  仍超阈值再全量清空后继续。选全量清而非按 mtime 部分清：partial 清理破坏 cargo
  增量一致性，下次构建反而产出更多中间产物。
- 阈值内零开销放行（一次 `du`）；守卫自身故障（权限等）放行不阻塞。
- 跳过：`ZW_TARGET_DISK_GUARD=0`；调阈值：`make test ZW_TARGET_DISK_LIMIT_GB=80`。
- 跨平台：Linux/macOS 原生 bash；Windows 经 Git Bash（Makefile `$(WPT_BASH)`，
  与 fetch-wpt-data 同款入口，见 Makefile 顶部注释）。

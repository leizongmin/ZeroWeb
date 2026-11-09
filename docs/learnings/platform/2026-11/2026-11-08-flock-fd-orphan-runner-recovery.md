---
date: 2026-11-08
modules: .acceptance/dps, scripts
---

# 批脚本被外部 SIGTERM 后 runner 孤儿持锁继续跑——先判孤儿活性再决定等待完账，补跑分支禁止原地截断

## 问题描述

F2 门禁批（2026-11-08）mt 腿遭外部 SIGTERM（`make: Terminated` rc=143，
跨 clone cmdline 同形误杀环境），但 `zero-wpt-runner` 子进程存活为孤儿，
继续向目标日志写（语料尾缓冲，全程 0 行直到跑完才落盘）。连锁两个坑：

1. 孤儿通过 `exec 9>batch.lock` 继承的 fd 继续持有批锁 → 后继 launcher
   长时间等锁，直接重发批会 ABORT LOCK-BUSY。
2. 后继批的 mt 补跑分支 `> f2ga-mt-post.log` **原地截断**了孤儿正在写
   （且即将完账）的日志——孤儿输出被销毁，只好再补跑一整轮。

## 根因分析

- flock 锁的生命周期绑 fd 不绑进程：父批脚本死于信号后，子进程树里持有
  fd 9 的 runner 仍在，锁不释放。`agent_exit ≠ 进程死亡`。
- wpt-runner editing 语料腿**尾缓冲输出**：跑完整腿才 flush，中途日志空
  不等于挂死；判活看 `/proc/<pid>/stat` 的 R 态 + CPU 时间增长 +
  历史墙钟（mt ~31 min）。
- 信号源在跨 clone 并行环境不可考（双 clone cmdline 同形），逐进程
  `/proc/<pid>/cwd` 归属是唯一可靠判别。

## 解决方案

- 批 ABORT/被杀后先 `ps` 找同 cmdline 孤儿（核 cwd 归属 clone），
  R 态 + CPU 增长 ⇒ **等待完账**（其输出仍写向原日志，是最干净的产物），
  不立即重发。
- 后继批补跑分支先旁存：`mv f2ga-mt-post.log f2ga-mt-post.orphan.bak`
  再 `>` 重跑——怀疑中的孤儿日志可能是唯一实物证据（本次双写嫌疑
  因截断而永久不可考）。
- launcher 轮询窗要覆盖「孤儿完账 + 锁释放」的真实时长（本次 mt 孤儿
  又跑了 35 min），窗内 flock -n 探测锁空即发，双 launcher 靠
  flock-before-truncation 天然去重（败者 ABORT LOCK-BUSY 不碰共享日志）。

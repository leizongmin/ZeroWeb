---
date: 2026-10-05
modules: rally-infra, docs/rally
---

# pgrep/pkill -f 自匹配静默卡死守候器并击杀自身 shell

## 问题

无人值守巡检用 `nohup bash -c` 部署的静窗守候器（quiet-window watcher）始终不触发 bench 复跑；排查时又发现 `pkill -f watch-run3.sh` 直接把自己所在的复合命令 shell 一起 SIGTERM（exit 144），python 后半段静默未执行。

## 根因

1. **pgrep/pkill -f 的模式会匹配调用者自己的命令行**：守候器脚本文本里含有 `pgrep -f "css_bench|dom_bench"` 这类字面量，于是 `S2=$(pgrep -f ... | wc -l)` 恒 ≥1（至少计到守候器自身 bash），`sib==0` 永假——守候器不是在等静窗，是被自己的匹配串卡死。轮询命令同样会把自己算进去，造成"存在共租进程"的假读数。
2. **复合命令与 pkill 同命令行**：复合命令文本含 `watch-run3.sh` 字面量，`pkill -f` 把它一并命中，杀掉自己的进程组，`&&` 后半段（结果落盘）静默丢失，exit code 144。

## 解决方案

- 模式用**字符类断开自匹配**：`pgrep -f 'watch-run[3]'` 自身的 cmdline 含 `watch-run[3]` 字面量，不匹配正则 `watch-run[3]` 对应的目标串 `watch-run3`。
- 共租检测**不用进程名**，改用 `/proc/*/cwd` 判兄弟 clone 归属；注意 `readlink` 结果**无尾斜杠**，`case` 分支要同时覆盖 `/path/ZeroWeb` 与 `/path/ZeroWeb/*`，且宽松前缀会误吞本仓 `-cronjob` 后缀，必须显式双锚。
- 守候器每 60s 把触发原因写日志（`quiet load=… -> firing`），使"未触发"可归因（在等负载还是在等 sibcount）。

## 如何避免

无人值守 harness 中凡是 `-f` 全命令行匹配（pgrep/pkill/grep ps），写模式前先问：这个字符串是否也出现在**本次命令/脚本自己的 cmdline** 里？是，就用字符类拆开，或改按 `/proc/<pid>/cwd`、可执行文件真实路径（`/proc/<pid>/exe` readlink）判定。

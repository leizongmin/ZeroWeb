---
date: 2026-11-15
modules: .acceptance/dps,tests/wpt-runner
---

# 长批唯一可靠载体是 systemd --user service；setsid 与后台 Bash 都会被拆树或到点杀

## 问题描述

多席并行的 wpt-runner 五腿门禁批（全程 1-2 小时）以 `setsid nohup ... & disown` 发射后仍反复静默死亡（v1 批 SILENT-DEATH，账本可查）；Claude Code 后台 Bash（run_in_background）默认 600s 到点后整链被杀（含 detached 孙进程），表现为批中途消失、腿日志戛然而止、无 ABORT 标记。

## 根因分析

- 会话退出时按进程树 sweep 发 SIGTERM；`setsid` 只换 session 不脱离树管理，照样被拆（`kill -0` 验活时批已死）。
- CC 后台任务超时是整链 kill 而非孤儿化，包装壳与孙进程一起没；`setsid $!` 取到的是立即退出的包装壳 pid，绑它判活必误报。
- `systemd --user` service 不随登录会话/终端拆树，是唯一真脱离载体；但 CC 内 `systemd-run` 可能被 polkit 拒（logind 活跃态敏感），需 fallback 检查。

## 解决方案

- 长批一律 `systemd-run --user --unit=<中性名> --service-type=exec` 发射：`WorkingDirectory` 必设，代理/rustup/PATH 环境变量显式传入；中性名抗误扫与 pkill 误伤。
- 判活绑批日志首行记录的脚本 pid（`pid=$$`），加腿 runner 子进程 CPU 活性；`$!` 是包装壳不可绑。
- 死亡判定看日志终态标记（ALL-DONE/ABORT）而非仅 ps；session 内同步等待用 `timeout + until` 复合循环，监控类后台任务显式给足 timeout。
- 例证：v6 批（unit rt7q）08:34 systemd 发射，历经同 clone 内多次会话结束与他席风暴，09:52 自然 ALL-DONE 退出；对照 v1 setsid 批在会话收尾窗被杀。

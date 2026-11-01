---
date: 2026-11-01
modules: rally-infra, dev-machine
---

# 无差别 `pkill -x Xvfb` 误杀开发机远程桌面：批量清理必须按 PID/显示号收窄

## 问题

2026-11-01 rally 自主任务做 GUI 复测时，两次在探针启动前执行 `pkill -x Xvfb` 清理"残留实例"，把开发机（LightOS 容器 lei-dev）远程桌面的 Xvfb :20 一并杀掉：14:32 CST 第一次杀死常驻桌面（用户 14:37 连接即黑屏）；14:38 systemd `Restart=on-failure` 拉起新桌面会话后，14:44 第二次 `pkill -x Xvfb` 又将其杀死。用户侧表现为"桌面死掉"，且服务不会自愈。

排查闭环证据链：agent 会话转录中两条命令的 UTC 时间 +8 与 `/run/lightos-remote-desktop/session.log` 的 ICE 断连时刻精确吻合（第二次误差 40ms）；`/tmp/.X11-unix/X20` 与 `.X20-lock` 被干净移除（可处理信号下正常退出，排除段错误）；服务 cgroup `memory.events` `oom_kill=0` 排除 OOM；无内核 GPU/DRM 错误。

## 根因

1. **`pkill -x Xvfb` 按可执行名匹配全机所有 Xvfb**，不区分"本任务自起的实例"与"环境常驻服务"。GUI smoke 曾正确地对自起 `Xvfb :52` 按 PID 回收，但重跑前的"保险性清理"退化成按名全杀。与 [pgrep/pkill -f 自匹配](../2026-10/2026-10-05-pgrep-pkill-self-match-watchers.md) 同族：匹配面大于任务实际拥有的资源面，且静默命中无辜进程。
2. **被杀目标不是测试资源而是用户实时桌面**：本机 `:20` = `lightos-remote-desktop.service` 管理的 Xvfb + XFCE 会话（selkies 8766 端口远程呈现）。既有记忆笔记只约束了"勿在 :20 开 GUI 窗口"（用途面），没有约束"清理动作不得波及"（动作面）。
3. **服务脚本缺陷放大后果**：`lightos-remote-desktop-run.sh` 末尾只 `wait $selkies_pid`，不监督 Xvfb/会话存活；Xvfb 死后 selkies 仍占端口，服务既不退出也不拉起会话，systemd 看到的始终是 active，表现为静默黑屏而非可见故障。

## 解决方案

- 用户侧恢复：`sudo systemctl restart lightos-remote-desktop.service`（Xvfb 死后 selkies 仍占着服务，必须整服务重启，仅杀 selkies 无效）。
- 事故本身已如实记档于 renderer-trust-boundary M4 DC-5 交付说明（285d16759），教训按本文与 agent 记忆双线固化。

## 如何避免

- **批量清理一律收窄匹配面**：自起实例记 PID、结束 `kill $XPID`；确需按模式兜底时带任务独有标识，如 `pkill -f "Xvfb :52"`（按显示号过滤），或按 `/proc/<pid>/environ` 的 `DISPLAY=` 判定属主。**永远不用 `pkill -x Xvfb`/`pkill -f Xvfb` 一类按名全杀**。
- 动手前先盘点机器常驻 X 资源：`ls /tmp/.X11-unix/`、`pgrep -af Xvfb`；凡非本任务自起 PID 一律不碰。
- 环境探针结论要同时约束"用途"和"动作"：":20 是用户实时桌面"对应的规则不仅是"勿开窗口"，还包括"勿纳入任何清理/重启/回收面"。

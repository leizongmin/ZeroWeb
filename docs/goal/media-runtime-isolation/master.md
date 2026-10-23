# 媒体运行时隔离控制面

**日期**：2026-10-20
**状态**：Active（**M1 已解锁**——2026-10-23 用户决策，无人值守可启动；M2 RFC 架构门与平台/许可待决不随 M1 解锁）；规划已创建，开发/长视频/故障验收 NOT_RUN。
**当前卡点**：缺少生产复制、同步和权限基线；隔离方案尚待 RFC。
**下一步**：执行 M1——固定长视频、版本与真实帧消费链，测量现有 CPU/帧复制/A-V 同步与权限路径；归档 background-resource 的媒体时钟三缺口（其唯一 INCONCLUSIVE）记 M1 盘点输入，产品修复与 bg02-media 基线 v2 回补按证据另行立项确认。
**入口**：[目标契约](../media-runtime-isolation.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| CPU/帧复制/A-V 同步与权限 | 固定长视频、版本与真实帧消费链 | NOT_STARTED |
| 同步状态代理/时钟/seek 的进程契约 | 比较现状、平台解码、独立运行时 | NOT_STARTED |
| 故障、权限不足与软件回退 | 真实隔离实例注入并验证浏览器恢复 | NOT_STARTED |
| 平台 SDK/codec 许可/分发与架构投入 | M1/M2 证据与 RFC 后决定，不自动换许可证 | 待决 |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 生产资源、能力与权限基线 | NOT_STARTED | 无 |
| M2 | 方案/许可证/回滚 RFC | NOT_STARTED | 无 |
| M3 | 已确认平台与隔离切片 | NOT_STARTED | 无 |
| M4 | 配对媒体、故障和资源审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | MEDIA-01-cpu-copy-and-capability | NOT_RUN | 无 |
| DC-2 | MEDIA-02-clock-seek-and-state | NOT_RUN | 无 |
| DC-3 | MEDIA-03-decoder-crash-recovery | NOT_RUN | 无 |
| DC-4 | MEDIA-04-platform-fallback-and-permissions | NOT_RUN | 无 |
| DC-5 | RFC 与最终支持范围审计 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：media/engine/webview/protocol/renderer/compositor 与 Android。实际占用 **NOT_CHECKED**；启动前按 run-rules §9–11 核对，不扩大支持承诺。

本轮未安装 SDK，不授权提交、推送或通知。后续守卫保护长视频与故障实验，保留软件回退、生产测量与清理证据；`evidence/`、`archive/` 只追加。若方案不值得投入，用户认可调整 DC 后才能收口；当前没有运行记录。

# 多进程运行、共享契约与跨域配置

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 browser、renderer、compositor、image-decoder 和 Android 角色之间的消息、页面
执行、帧资源与生命周期。共享 API 的真实消费关系在组件索引维护，具体存储与安全
算法在 web-data，字体与光栅算法在 graphics。

### User Task Navigation

| 用户问题 | 源码入口 | 相关知识 |
|---|---|---|
| peer 断连、响应卡住或载荷新增 | `crates/protocol/src/transport.rs`、`apps/browser/src/process_backend.rs` | 共享 IPC |
| 加载阶段、默认动作或帧失效 | `crates/page-runtime/src/lib.rs`、`html_actions.rs` | 共享页面契约 |
| 导航抢占、旧资源或帧乱序 | `apps/renderer/src/runtime.rs`、`compositor_publish_thread.rs` | renderer 协调 |
| surface、滚动、GPU 回退或窗口呈现 | `apps/compositor/src/lib.rs`、`apps/browser/src/compositor_client.rs` | compositor |
| 同一页面在产品/测试/平台上不同 | `apps/browser/src/main.rs`、各消费者 Cargo 文件 | 跨域配置 |

运行判据和真实入口从 [验证地图](../../../zeroweb-verify/features/index.md) 按任务选，
本领域没有产品实跑；初始化验证/知识资产已在 d0f281d509de3bf4f9dcd7e8dd01dac3464a663b 提交，不归为上方旧源码基线。

## Gotchas

- 请求回执、页面帧发布与窗口呈现属于不同阶段；协议成功或 surface 已有像素不能单独证明用户窗口已显示当前导航。 (`crates/protocol/src/message.rs`, `apps/renderer/src/runtime.rs`, `apps/compositor/src/lib.rs`)
- 共享角色入口只统一消息循环接口，平台 bootstrap、句柄能力和测试替代仍不同；改通用契约时不能只看一个桌面 peer。 (`crates/protocol/src/channel.rs`, `apps/android-browser/rust/src/lib.rs`, `apps/compositor/src/lib.rs`)

## Architecture

- 页面运行契约不持有窗口或网络权限，renderer 执行页面并产出资源，browser 持有宿主状态和网络/存储代理，compositor 消费图元及输出帧；跨边界能力需沿真实请求与完成路径追踪。 (`crates/page-runtime/src/lib.rs`, `apps/renderer/src/runtime.rs`, `apps/browser/src/process_backend.rs`, `apps/compositor/src/lib.rs`)

## Child Knowledge Nodes

- [共享 IPC 与进程协议](protocol/index.md) — 消息、通道、进程、共享帧、mailbox 与 FD。
- [共享页面运行时契约](page-runtime/index.md) — host trait、加载阶段、帧事务与用户动作。
- [renderer 与浏览器进程协调](renderer/index.md) — epoch、抢占、预算、资源发布和隔离。
- [compositor、呈现与恢复](compositor/index.md) — surface、滚动、输出、GPU 回退与断连。
- [跨域环境与构建分支](conditions/index.md) — 环境、feature、平台和测试/发布差异。

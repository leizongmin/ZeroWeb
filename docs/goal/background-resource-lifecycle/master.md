# 后台资源生命周期控制面

**日期**：2026-10-20
**状态**：Active（启动门控）；规划已创建，开发与生产资源采样 NOT_RUN。
**当前卡点**：没有前后台、重复导航与停滞资源的配对基线。
**下一步**：明确启动后建立隔离 profile、本地 fixture 与各进程观测。
**入口**：[目标契约](../background-resource-lifecycle.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 嵌套 timer、后台与视口外动画策略 | 核对规范时序、呈现计数与前台恢复 | NOT_STARTED |
| 重复导航旧文档/回调/资源释放 | 固定不结束子资源 fixture，追踪真实 owner | NOT_STARTED |
| 关闭后的连接/队列/进程生命周期 | 真实宿主操作并记录所有本次 PID | NOT_STARTED |
| CPU/RSS/时长/预热门禁 | 配对基线后固定，不预设达标数字 | 待核查 |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 四类场景与生产资源基线 | NOT_STARTED | 无 |
| M2 | 首个增长来源、共享 owner 修复和回归 | NOT_STARTED | 无 |
| M3 | 空闲/视口外调度及恢复 | NOT_STARTED | 无 |
| M4 | 资源门禁、持续配对复测与清理 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | BG-01-nested-and-hidden-timers | NOT_RUN | 无 |
| DC-2 | BG-02-offscreen-and-idle-present | NOT_RUN | 无 |
| DC-3 | BG-03-repeat-navigation-stalled-resource | NOT_RUN | 无 |
| DC-4 | BG-04-close-and-resume | NOT_RUN | 无 |
| DC-5 | 固定协议长期复测 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：browser/renderer/page-runtime/engine/compositor 与 navigation/page-state 流。实际占用 **NOT_CHECKED**；启动前按 run-rules §9–11 核对。

本轮未操作用户浏览器；后续仅清理自己创建的实例和资源。本轮不授权提交、推送或通知。运行版本、profile/PID、采样条件与清理按统一协议落 `evidence/`；里程碑追加 `archive/`。当前没有运行记录。

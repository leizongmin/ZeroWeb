# 合成动画连续性控制面

**日期**：2026-10-20
**状态**：Active（启动门控）；规划已创建，开发与生产 GPU 验收 NOT_RUN。
**当前卡点**：尚未实测主线程阻塞时的动画支持范围。
**下一步**：明确启动后固定有界 JS 负载，采集 opacity/transform 生产帧。
**入口**：[目标契约](../compositor-animation-continuity.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 动画是否独立推进及实际支持子集 | 同时记录帧、查询状态与事件 | NOT_STARTED |
| 时间/状态/命中几何共享契约 | 暂停、取消、结束和后台恢复联合验证 | NOT_STARTED |
| Worker OffscreenCanvas 2D 呈现与释放 | 转移、worker 退出和 context lost fixture | NOT_STARTED |
| WebGL 依赖与扩展属性 | 遵守 webgl-compat 门控，独立记录未支持项 | 待核查 |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 真实窗口阻塞基线与支持矩阵 | NOT_STARTED | 无 |
| M2 | 合成动画、状态与输入一致切片 | NOT_STARTED | 无 |
| M3 | Worker 2D 呈现与生命周期 | NOT_STARTED | 无 |
| M4 | GPU/语义/资源与 DC 审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | COMP-01-transform-opacity-under-js-load | NOT_RUN | 无 |
| DC-2 | COMP-02-script-state-and-end-event | NOT_RUN | 无 |
| DC-3 | COMP-03-hit-test-pause-cancel | NOT_RUN | 无 |
| DC-4 | COMP-04-worker-canvas-transfer | NOT_RUN | 无 |
| DC-5 | 关闭/导航释放与支持审计 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：engine/protocol/compositor/canvas/workers、Android 协议消费者。实际占用 **NOT_CHECKED**；启动前按 run-rules §9–11 核对，协议更改同步双方。

本轮不授权提交、推送或通知。后续生产 GPU 帧、脚本事件、真实输入和清理按统一协议保存，CPU 截图不能替代 GPU 证据。`evidence/` 和 `archive/` 追加；当前没有运行记录。

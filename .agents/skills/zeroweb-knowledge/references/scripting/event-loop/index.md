# 页面事件循环与用户动作

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

查任务边界、事件可取消性与宿主默认动作。JS task/checkpoint、DOM apply 与 frame 发布分别由不同层承担；帧传输见 [运行时帧](../../runtime/protocol/frame-delivery.md)。

## Architecture
- 默认动作需把事件结果带回宿主 planner，再决定 commit/rollback 与效果；只看 dispatch 是否执行或脚本返回成功，不能判断点击导航、checkedness 与输入是否应发生。 (`crates/page-runtime/src/html_actions.rs`, `crates/webview/src/webview/user_actions.rs`)
- observer/rAF 的脚本派发和渲染几何刷新不能合成一个“tick 已完成”信号；renderer 的结构测试不覆盖 host 每回调之后的真实几何更新。 (`apps/renderer/src/page_scripts.rs`, `apps/renderer/src/tick_per_task_tests.rs`, `crates/engine/src/timer_bridge.rs`)

## Child Knowledge Nodes
- [脚本顺序、microtask、定时器与观察者](tasks-and-checkpoints.md) — per-task 分支、64 轮上限、异步排水与加载状态。
- [事件分发、默认动作、焦点与表单激活](events-and-default-actions.md) — prepare/rollback、原子文本、焦点镜像与宿主事务。

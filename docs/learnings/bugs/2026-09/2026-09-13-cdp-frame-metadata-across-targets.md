---
date: 2026-09-13
modules: apps/browser
---

# CDP 子帧元数据面跨 target 串扰与 frameAttached 主帧改写契约

## 问题描述

headless CDP 服务器实现 `Page.frameAttached`/`frameDetached` 子帧元数据探测后，CDP E2E 门出现两步回归（`browserContext.newPage: Frame has been detached.`），且均发生在**第二个 page**（`context.newPage()` / `newCDPSession`）场景；首步 `frames.access` 反而正常翻绿。

## 根因分析

两个独立但叠加的机制：

1. **会话级扁平记录跨 target 串扰**：子帧记录（已 attach 未 detach 的 child frame id）放在单例 `HeadlessSession` 的扁平 `Vec<String>` 上，不分 target。headless 是单 session 多 target 模型——p1 导航留下的记录会被 p2 的导航事件族误 detach（`frameDetached` 事件盖章在 p2 会话上发出），p2 的 `getFrameTree` 也会读到 p1 的幽灵 child。
2. **Playwright 对畸形 frameAttached 的破坏性分支**：PW FrameManager 的 `frameAttached(frameId, parentFrameId)` 在 `parentFrameId` 在本页帧表查不到时，走**主帧 id 改写**分支（`this._mainFrame._id = frameId`，为 OOPIF session 切换设计）；后续 per-frame session 查找（`_sessionForFrame` 沿 parent 链找）失败即抛 `Frame has been detached.`。

## 解决方案

- 子帧记录改按主帧 id（=targetId）`HashMap<String, Vec<String>>` 分组：detach/attach/getFrameTree 均只操作本页分组，串扰根除。
- `frameAttached` 必须带 `parentFrameId`（且为主帧 targetId）——这是 PW 创建子 Frame（而非改写主帧）的分叉点。

## 如何避免

- 单 session 多 target 的 headless 服务器上，任何 per-page 状态（帧表、context 表、子帧记录）都必须按 targetId 分组，禁止会话级扁平容器。
- 对接 PW 的 CDP 事件形状，先读 pin 版本 `playwright-core/lib/coreBundle.js` 的消费代码（`frameAttached` 的主帧改写分支、`_sessionForFrame` 的沿链查找）——事件形状差一个字段就是页面级崩溃。
- raw CDP 探针（裸 WS + setAutoAttach flatten + per-session 事件记录）是定位「事件串扰/盖章路由」类问题的最快手段，比整门重跑 + 步骤二分快一个量级。

---
date: 2026-08-17
modules: apps/browser/src/process_backend.rs, apps/browser/src/compositor_client.rs
---

# Active Tab IPC Poll Starvation

## 问题描述

创建多个标签页后，新活动标签页可能长期空白。对应 renderer 和 compositor surface 实际仍存活并持续产帧。

## 根因分析

Browser 每帧只给所有 renderer 共用 4ms IPC 轮询预算，但此前按 `HashMap` 顺序遍历 renderer，且忽略传入的活动标签页和后台轮询开关。旧标签页积压的 resize 和 paint 消息会耗尽预算，使新活动标签页的首帧长期得不到消费。

此外，Browser 与 compositor 使用单连接串行请求。当 `qq.com` 等复杂页面的一帧长期卡在 compositor 光栅化时，后续 surface 即使已在 renderer 产帧，也无法得到 compositor 完成帧。现有 watchdog 只把状态改为 `Disconnected`，但不终止卡死进程；隔离进程重构还移除了 renderer 的 `ViewPainted` 故障回退，导致断线后新标签页永久空白。

## 解决方案

每帧始终优先轮询活动标签页；后台标签页仅在既有的低频轮询周期内参与。使用纯单测锁定轮询顺序，并用真实多进程测试验证连续创建 8 个标签页后每个标签页都有 renderer 映射和页面快照。

watchdog 超时后终止无响应的 compositor 子进程，并在状态进入 `Disconnected` 时让所有独立 renderer 切换到 legacy frame 发布。这里的 legacy 仅指 `ViewPainted` IPC，不退回单进程页面运行时；新建 renderer 在 compositor 断线期间也直接使用该发布模式。

## 如何避免

共享时间预算下不要直接依赖无业务优先级的容器遍历顺序。前台交互对象必须显式排在第一位，后台工作应单独限频，并为“活动对象在预算耗尽前得到服务”添加确定性测试。

跨进程渲染链路的 watchdog 必须同时具备资源终止和可用降级路径。只更新状态、不解除阻塞或不切换帧来源，会把一次慢帧放大为全浏览器永久空白。

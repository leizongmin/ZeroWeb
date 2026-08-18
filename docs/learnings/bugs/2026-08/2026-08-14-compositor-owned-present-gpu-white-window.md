---
date: 2026-08-14
modules: apps/browser, apps/compositor
---

# Compositor owned-present 导致 GPU 启动白窗

## 问题描述

GPU 模式启动后窗口长时间保持白色。与此同时，compositor 反复收到
`3024x1802` 的 Chrome UI 位图；每张 RGBA 位图约 20.8 MiB。

## 根因分析

`owned present` 在 compositor 连接健康后立即禁止 browser 本地 GPU present，
但 compositor 当前只通过 IPC 返回完整 RGBA 位图，并不能直接提交 browser 所拥有的
GPU swapchain。GPU 路径在上传 UI 和请求 present 后直接返回，也没有把返回位图 blit
到窗口，因此只能等待 compositor 超时并回退 legacy 后才恢复本地呈现。

加载动画持续触发重绘时，这条错误路径还会把整张物理窗口 UI 位图反复发送给
compositor，造成大量光栅化、内存复制和 IPC 流量。

## 解决方案

只在没有本地 GPU present 能力的路径允许 `owned present` 抑制本地呈现。GPU 模式继续
由 browser 提交最终窗口帧，页面位图仍由 compositor 生成并交给 browser 合成。这样既
保留 compositor 页面渲染链路，也避免启动白窗和全窗口 UI 位图重复传输。

回归测试固定以下约束：即使 owned-present、present 和 compositor 健康状态同时开启，
只要本地 GPU present 可用，就不能跳过本地提交。

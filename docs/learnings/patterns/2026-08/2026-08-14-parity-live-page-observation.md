---
date: 2026-08-14
modules: apps/browser/src/parity_smoke.rs, zeroweb-browser-chrome-parity skill
---

# 一致性验收应直接观测 live page

## 问题描述

页面一致性采集若依赖固定 ID、特殊 title 或测试页状态节点，只能覆盖专门适配过的 fixture，无法可靠用于普通页面。

## 根因分析

状态、几何和点击目标曾分别走页面埋点、ID 扫描和预先观测的几何缓存。这些通道没有共享浏览器实际执行页面的 DOM 语义，也会让通过静态校验的非 ID 场景在 ZeroWeb 端失败。

## 解决方案

通过现有 renderer automation IPC 在当前 live document 中执行场景的 `stateExpression`，使用 `querySelector()` 与 `getBoundingClientRect()` 读取观测和每次点击的目标；输入仍走浏览器真实 mouse move/down/up，截图仍取生产 compositor GPU 帧。事件目标对无 ID 元素记录 `nth-of-type()` DOM 路径。无法由 ZeroWeb 真实应用的环境值应在采集前拒绝，不能只在 Chrome 端模拟。

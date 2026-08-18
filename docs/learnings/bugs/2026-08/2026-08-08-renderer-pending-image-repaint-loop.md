---
date: 2026-08-08
modules: zero-webview, zero-renderer
---

# Renderer 等待图片期间重复重绘

## 问题描述

页面首屏已经可见，但子资源仍在下载时，renderer 持续占用接近一个 CPU 核心。日志以约 20 次/秒重复输出相同的 `image batch ready`、`budget render start` 和 `budget render complete`，同时剩余图片数不变。

## 根因分析

`AsyncPageLoad::tick` 用 `changed` 汇总整个 tick 的状态变化。`poll_images` 错误地用这个共享值判断本轮是否有图片完成。

一次图片完成会设置 `budget_pending`。下一 tick 开始时，`advance_render` 完成重绘并令 `changed = true`；随后 `poll_images` 即使没有收到任何图片结果，也会把这个值误判为图片变化，再次设置 `budget_pending`。状态机因此在“完成重绘”和“请求重绘”之间永久循环。

## 解决方案

`poll_images` 使用局部 `image_changed` 记录本轮实际收到的图片结果。只有该值为真时才请求增量重绘，最后再将其合并到 tick 级 `changed`。

回归测试使用保持连接但不返回结果的图片请求，并断言进入 `FetchingImages` 后：

+ 连续 tick 不报告状态变化；
+ 不创建新的渲染会话；
+ 不重新设置 `budget_pending`。

## 如何避免

+ 状态机的局部触发条件不能复用上层汇总标志。
+ 等待异步资源时，只有资源完成、失败或明确的外部状态变化才能触发重绘。
+ 性能问题复测应同时检查 CPU、资源剩余数和阶段日志；页面可见不等于加载状态机已空闲。

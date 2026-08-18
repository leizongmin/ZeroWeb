---
date: 2026-08-17
modules: apps/compositor, zero-protocol
---

# DMA-BUF 测试必须关闭 compositor scroll transform

## 问题

`compositor_gpu_dmabuf_browser_import_round_trips` 在 headless GPU 可用时稳定收到 RGBA，
而不是预期的 `CompositorResolvedFrame::Dmabuf`。

## 根因

`apps/compositor/src/main.rs` 仅在 `ZW_COMPOSITOR_SCROLL_TRANSFORM=0` 时导出 DMA-BUF。
测试 helper 开启了 GPU、GPU image、texture export 和 browser import，却漏掉该互斥开关。
由于 scroll transform 默认开启，生产分支按设计回退到 RGBA。

## 解决

DMA-BUF 专用测试环境显式设置 `ZW_COMPOSITOR_SCROLL_TRANSFORM=0`。修改后目标测试连续
3 轮通过。以后新增 DMA-BUF 端到端测试时，应同时声明这一前提，不能只用 headless
adapter 创建成功作为导出能力判断。

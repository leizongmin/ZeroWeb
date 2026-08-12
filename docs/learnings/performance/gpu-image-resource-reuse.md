# GPU 图片资源复用与 compositor 图片生命周期

- 日期：2026-08-12
- 相关模块：`render-foundation/gpu`、`compositor`

## 问题描述

GPU 全场景路径每帧为每个图片创建 texture、sampler、bind group 和 vertex buffer，并重复创建 viewport uniform。多进程路径中 renderer 只在首次见到图片 key 时传输像素，但 compositor 没有持久消费 `image_payloads`，后续帧可能只有图片图元而没有像素来源。

## 根因分析

CPU 侧 `ImageCache` 的生命周期没有延伸到 compositor surface，GPU 资源也没有与解码图片的稳定身份绑定。只用 ImageKey 作为 GPU 缓存键又不安全：同 key 的像素或尺寸发生变化时会错误显示旧纹理。

## 解决方案

1. GPU 缓存键同时包含 ImageKey、宽高和像素内容摘要。
2. renderer 生命周期内复用图片 sampler、texture/bind group 与 viewport uniform/bind group；缓存设置硬上限。
3. 渐变和图片顶点分别合并成单个 vertex buffer，仍按原图元顺序绑定资源和切片绘制。
4. compositor 每个 surface 持有独立、有界的解码图片缓存；navigation epoch 变化时清空，surface 释放时随状态释放。
5. CPU 与 GPU 光栅路径读取同一 surface 图片缓存。局部 GPU 路径遇到图片时回退到具备图片缓存的 CPU 局部绘制，避免静默漏图。

## 如何避免复发

- 实时渲染缓存不能只按逻辑 key 命中，必须覆盖影响像素的尺寸和内容版本。
- 发送方做像素去重时，接收方必须拥有同等或更长的缓存生命周期。
- 顶点合批必须保持 painter order；只合并 buffer 分配，不跨资源重排图元。
- 测试必须覆盖首次带 payload、后续无 payload、同 key 内容变化，以及 CPU/GPU 回读一致性。

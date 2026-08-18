---
date: 2026-08-16
modules: zero-canvas（gpu_path 测试）, zero-render-foundation（gpu/renderer）
---

# GPU clip「擦白」语义 vs canvas clip()「持续裁剪」语义差异

## 问题描述

canvas 场景（`clip()` 后全画布 `fillRect`）经 `render_full_scene_gpu` 渲染：
clip 外的像素被后续 fill 覆盖（clip 未生效），与 CPU 光栅（clip 内红、外蓝）
不一致。测试断言「clip 外不得被红覆盖」失败。

## 根因分析

GPU 渲染器的 `ClipPrimitive` 是 **CSS clip-path 模型**：draw_order 插入位置画
「clip 外白 rect」**一次性擦除**——假定后续图元本就在 clip 区域内。canvas 的
`clip()` 是 **状态性持续裁剪**：clip 之后的所有绘制都被裁剪，clip 图元之后的
全屏 fill 会覆盖擦白区域。

`scene_supported` 对 clips 返回 true（CSS 场景语义下正确）——canvas 场景走
GPU 后语义不成立。**生产路径不受影响**：canvas 显示链路（engine painter R3268）
把 CPU 光栅像素快照（clip 已生效）上传纹理，primitives 的 clip 图元不参与
canvas 生产渲染——该差异只在测试面暴露。

## 解决方案

gpu_path.rs 的 clip 测试改为诚实断言：CPU 像素断言（clip 持续裁剪语义）+ 图元
产生 + GPU 渲染不 panic（注释说明语义差异，不做像素级断言——防假绿）。

## 如何避免

- **图元语义≠像素语义**：`RenderPrimitives` 的图元是页面 CSS 绘制语义的载体，
  canvas 的持续裁剪状态无法用一次性图元表达。跨语义桥接的测试要明确「哪层
  负责哪段语义」——canvas 的裁剪语义在 CPU 光栅（像素缓冲）层，GPU 图元层
  只有 CSS 语义。
- 测试断言前先验证渲染器底色/裁剪约定（GPU clear 白、擦白一次性）——与
  parity_tests「GPU clear 为白、CPU framebuffer 初始为黑」同类的环境语义。

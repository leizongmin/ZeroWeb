# ZeroWeb Canvas (`zero-canvas`)

> Canvas 2D 绘图 API 实现 — 渲染上下文、路径构建、仿射变换与图像数据

## 概述

`ZeroWeb Canvas` (`zero-canvas`) 实现了 ZeroWeb 的 Canvas 2D 绘图能力，提供与 HTML Canvas `CanvasRenderingContext2D` API 对应的 Rust 接口。它在渲染管线中位于 `zero-render-foundation` 之上，将绘图命令转化为渲染图元（`RenderPrimitives`），供下游渲染器消费。核心组件包括 `CanvasContext`（2D 渲染上下文）和 `Path2D`（路径命令构建器）。

## 主要功能

- **矩形绘制** — `fill_rect`、`stroke_rect`、`clear_rect`
- **路径构建** — `begin_path`、`move_to`、`line_to`、`arc`、`quadratic_curve_to`、`bezier_curve_to`、`close_path`，以及独立的 `Path2D` 类型
- **文本渲染** — `fill_text`、`stroke_text`、`measure_text`，支持字体族、大小、粗细和样式配置
- **仿射变换** — 2D 矩阵（`Transform2D`）支持平移、缩放、旋转，以及 `set_transform` / `reset_transform`
- **状态管理** — `save` / `restore` 栈，完整保存和恢复填充色、描边色、线宽、字体、透明度与变换矩阵
- **像素数据** — `ImageData` 及 `get_image_data` / `put_image_data` 接口
- **渲染输出** — 通过 `into_primitives()` 消费上下文，输出 `RenderPrimitives` 供渲染管线使用

## 使用示例

```rust
use zero_canvas::{CanvasContext, FontDescriptor, FontWeight};
use zero_render_foundation::color::Color;

// 创建 800×600 的画布上下文
let mut ctx = CanvasContext::new(800, 600);

// 设置填充颜色并绘制矩形
ctx.set_fill_color(Color::RED);
ctx.fill_rect(100.0, 100.0, 200.0, 150.0);

// 变换：平移 + 旋转
ctx.save();
ctx.translate(400.0, 300.0);
ctx.rotate(std::f32::consts::FRAC_PI_4);
ctx.set_fill_color(Color::BLUE);
ctx.fill_rect(-50.0, -50.0, 100.0, 100.0);
ctx.restore();

// 路径绘制
ctx.begin_path();
ctx.move_to(10.0, 10.0);
ctx.line_to(200.0, 10.0);
ctx.line_to(200.0, 200.0);
ctx.close_path();
ctx.set_fill_color(Color::GREEN);
ctx.fill();

// 文本绘制与测量
ctx.set_font(FontDescriptor {
    family: "serif".to_string(),
    size: 24.0,
    weight: FontWeight::Bold,
    ..Default::default()
});
ctx.fill_text("Hello, ZeroWeb!", 50.0, 50.0, None);
let metrics = ctx.measure_text("Hello, ZeroWeb!");

// 获取渲染图元，交由渲染器处理
let primitives = ctx.into_primitives();
```

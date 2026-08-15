# ZeroEngine (`zero-engine`)

> 页面渲染管线核心 — 编排 HTML 解析、CSS 样式计算、布局、绘制与合成的全流程。

## 概述

`ZeroEngine` (`zero-engine`) 是 ZeroWeb 的页面内核，负责协调 DOM、CSSOM、样式系统、布局引擎和渲染基础层等子系统，实现完整的 HTML 到渲染图元的端到端管线。它整合了绘制命令生成、脏区域追踪、合成层提升等核心能力，是浏览器渲染架构的关键枢纽。

## 主要功能

- **渲染管线**（`RenderPipeline`）— 端到端编排 HTML 解析、CSS 解析、样式计算、布局计算和绘制命令生成，支持全量渲染和增量更新
- **绘制命令生成**（`Painter`）— 将布局盒树遍历转换为背景色填充、边框填充等渲染图元，处理绝对偏移和子节点递归
- **脏区域追踪**（`DirtyTracker`）— 追踪因 DOM 或样式变化导致的屏幕失效区域，支持矩形合并以减少重绘面积
- **合成层提升**（`promote_compositing_layers`）— 根据透明度、固定定位等条件将元素提升为独立合成层，优化渲染性能
- **性能计时**（`PipelineTimings`）— 记录管线各阶段耗时，便于性能分析和优化

## 使用示例

```rust
use zero_engine::RenderPipeline;

// 创建渲染管线，指定视口尺寸
let mut pipeline = RenderPipeline::new(800.0, 600.0);

// 提供 HTML 和 CSS，执行完整渲染
let html = r#"<html><body><div id="main">Hello</div></body></html>"#;
let css = r#"div { background-color: red; width: 200px; height: 100px; }"#;
let result = pipeline.render_html(html, css);

// 获取渲染图元和各阶段耗时
println!("填充图元数量: {}", result.primitives().fills.len());
println!("布局耗时: {:.2} ms", result.timings.layout_ms);
println!("总耗时: {:.2} ms", result.timings.total_ms);
```

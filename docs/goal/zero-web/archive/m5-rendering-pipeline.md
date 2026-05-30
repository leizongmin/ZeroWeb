# M5 归档：渲染管线集成

**状态**: ✅ 已完成
**完成日期**: 2026-05-30
**提交**: 45d5477

---

## 交付物

| # | 交付物 | 状态 |
|---|--------|------|
| 1 | 布局盒 → 渲染命令的转换（paint） | ✅ Painter 递归转换 LayoutBox → RenderPrimitives |
| 2 | 矩形/背景/边框渲染 | ✅ background_color + 4-side border fills |
| 3 | 增量渲染：脏矩形追踪 | ✅ DirtyTracker with merge_overlapping |
| 4 | GPU 加速合成层 | ✅ promote_compositing_layers for opacity/fixed |
| 5 | 端到端管线 | ✅ RenderPipeline: HTML→CSS→Style→Layout→Paint |
| 6 | WPT 测试基础设施 | ✅ tests/wpt-runner/ 结构就位 |
| 7 | 单元测试 ≥30 个 | ✅ 39 个测试，各模块 ≥98% |
| 8 | 基准测试 ≥4 个 | ✅ 5 个 criterion 基准 |

## 覆盖率

| 模块 | Line Coverage |
|------|---------------|
| composite.rs | 100.00% |
| dirty.rs | 98.31% |
| paint.rs | 99.39% |
| pipeline.rs | 98.43% |

## 关键技术决策

- Painter 采用递归遍历 LayoutBox 树，为每个有样式的节点生成填充图元
- 背景色跳过 Transparent（不生成填充）
- 边框用 4 个矩形表示（top/right/bottom/left）
- DirtyTracker 支持合并重叠矩形以减少重绘面积
- 合成层提升策略：opacity < 1.0、position: fixed 提升为独立层
- RenderPipeline 记录各阶段耗时（PipelineTimings）

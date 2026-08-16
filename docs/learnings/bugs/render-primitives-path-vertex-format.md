# RenderPrimitives 路径图元顶点格式契约：段序列 vs 点序列

**日期**: 2026-08-16
**相关模块**: zero-canvas、zero-render-foundation（primitive/gpu mesh）

## 问题描述

canvas `fill()`/`stroke()` 产生的 `PathFillPrimitive`/`PathStrokePrimitive` 经 GPU
渲染器（`render_full_scene_gpu`）后全白——图元存在（`path_fills` 非空）但像素未
绘制。`gpu_path.rs` 端到端测试暴露（手搓点序列图元渲染正确 → 定位到 canvas 侧
图元内容）。

## 根因分析

两个层的顶点格式契约不一致：

- **canvas 侧**：`flatten_path()` 输出**段序列**（每 4 个 f32 = 一段 (x1,y1,x2,y2)，
  段间首尾重复，如环 A→B→C→A = 12 个 f32）。CPU 扫描线光栅化（`fill_rule_spans`）
  消费段格式。
- **render-foundation 侧**：`PathFillPrimitive.vertices` 文档契约为**点序列**（每
  2 个 f32 = 顶点，闭合多边形）——`push_path_fill_mesh`（ear-clip 三角化）与
  `push_path_stroke_mesh`（相邻顶点成线）都按每 2 个 f32 解析。

canvas 把段序列直接塞进图元 → GPU 把每 2 个 f32 当一点解析 → 重复点生成退化
三角形/零长线 → 面积 0 → 全白。CPU 渲染不受影响（canvas 生产路径是像素快照
`snapshot_rgba` 上传，不消费 primitives——该 bug 只影响测试面/未来图元消费方，
但契约违规是真实的）。

## 解决方案

canvas 侧加 `segs_to_point_verts()`（raster.rs）：段序列取每段起点、去重相邻重复
点 → 点序列。8 处 `add_path_fill`/`add_path_stroke` 调用统一转换。契约以
render-foundation（primitive/mod.rs 文档 + mesh.rs 解析）为准——跨 crate 边界
的共享数据格式必须查对方解析代码，不能假设。

## 如何避免

- **跨 crate 数据格式必须双向核验**：传给下游的缓冲区格式以下游解析代码为准
  （本例文档注释就是契约，读注释即可）。「我这么填、那边那么读」的假设是
  R56h「shim 已发 + API 已有 ≠ 链路通」教训的同构变体——桥接/边界层验证必须
  端到端（本例 GPU 像素回读），Rust 单测（只断言图元非空）会假绿。
- 图元级断言至少验证「消费方可解析的内容」：顶点数为点格式特征值（偶数、
  首尾不重复），而非仅非空。

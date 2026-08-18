# R57 batch-5 证据：路径填充 AA + RenderPrimitives 顶点契约修复 + GPU 测试扩展（2026-08-16）

## 修复 1：路径填充边界 AA（blit_path_to_pixels_rule / blit_path_gradient_rule）

- 非轴对齐 CTM 下 span 边界像素 4×4 超采样覆盖率（`path_pixel_coverage`——
  `spans_hit` 点内测试，与填充光栅化同一非零/偶奇判定；与 fillRect 的
  `rect_coverage` 同模式）；轴对齐恒硬边（零回归）。
- 边界像素：源 alpha × coverage 半色调（`blit_path_edge_pixel`——clip + composite
  + f32 并行写与内部像素一致）；coverage ∈ (0,1) 才处理（整数边界硬边、双写防护）。
- 单测 ×4（tests/raster.rs）：旋转路径 fill 半色调/内部满色/外部透明、evenodd
  挖洞旋转、渐变旋转、轴对齐零回归。

## 修复 2：RenderPrimitives PathFill/PathStroke 顶点格式契约（段序列 → 点序列）

- 根因：`flatten_path()` 段序列（每 4 f32 = 一段，首尾重复）被直接塞进图元——
  GPU mesh 按每 2 f32 = 顶点解析（点序列契约）→ 重复点退化三角形 → 旋转三角形
  经 GPU 渲染全白（gpu_path 端到端实测）。
- 修复：`segs_to_point_verts()`（raster.rs）段序列取起点去重 → 点序列；8 处
  add_path_fill/add_path_stroke 调用统一转换。
- 测试更新 ×11：图元顶点数断言从段格式（N×4）改点格式（N×2）——契约以
  render-foundation primitive/mod.rs 文档为准。详见
  docs/learnings/bugs/2026-08/2026-08-16-render-primitives-path-vertex-format.md。

## 修复 3：GPU 测试扩展（gpu_path.rs 5 → 8 个 GPU 测试）

- 进程内串行锁 `TEST_GPU_MUTEX`（软件后端非线程安全——与 render-foundation
  parity_tests serial 同语义；先前 5 个 GPU 测试无保护，并发时断言失败毒化）。
- `gpu_render_pixels` helper：render_full_scene_gpu 全 13 图元 + 像素回读
  （texture_export / read_pixels fallback）。
- 新增：旋转路径填充 GPU 像素（内部红满色、外部非红）、clip（CPU 持续裁剪
  像素断言 + GPU 不 panic——GPU 擦白为 CSS 一次性语义，canvas 持续裁剪语义在
  像素缓冲层，像素级 GPU 断言无意义防假绿）、半透明（GPU 白底混合 (255,127,127,255)）。

## 语义注记（learnings）

- GPU clip 擦白 vs canvas clip() 持续裁剪语义差异：
  docs/learnings/bugs/2026-08/2026-08-16-gpu-clip-erase-vs-canvas-clip-state.md
- 跨 crate 数据格式双向核验（R56h 教训同构变体）：
  docs/learnings/bugs/2026-08/2026-08-16-render-primitives-path-vertex-format.md

## 验证

- canvas 809 全绿（+9：路径 AA ×4、GPU ×4 修正后净增、格式契约断言更新 ×11）
- layout 1381 / engine 2158 / webview 599 / wpt-runner 171 全绿（零回归）
- browser 321 + 4 既有失败（form/input 快照，非 canvas 面）
- clippy 全 workspace 零警告；fmt 无 diff

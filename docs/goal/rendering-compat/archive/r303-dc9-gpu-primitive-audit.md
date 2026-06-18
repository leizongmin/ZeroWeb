# R303 — DC-9 GPU 图元覆盖审计（归档于 2026-06-19）

> 本轮详情于 2026-06-19 doc-maintenance 轮从 `master.md` 最近轮次记录归档至此（保持最近 ≤20 轮）。结论已沉淀进 `master.md` 的「综合裁决」杠杆表（DC-9 blend_mode | R278）与 DC-9 进度表，归档不丢失活跃信息。

**轮次**：R303（read-only 审计，基线 loose 438/490 / strict 296/490 持平）

**承接**：R302 clean-win 穷尽后转向 DC-9 独立能力缺口（GPU filter:opacity）。代码审计发现 **R220「DC-9 真实缺口仅 transform/filter/blend 三项」已过时**——这三项中 transform + filter 已实现，仅 blend_mode 仍缺。

**审计（gpu/renderer/mod.rs 逐字段引用计数 + headless ping-pong 实证）**：
- **GPU 已渲染 11/13 图元类型**（独立 WGSL 管线）：fills / rounded_rects / gradients / shadows / images / glyphs / strokes / path_fills / path_strokes / transforms / **filters**。
- **filters 已全实现**（headless ping-pong，line 787-800）：`collect_color_filters`（Opacity/Brightness/Contrast/Grayscale/HueRotate/Invert/Saturate/Sepia 共 8 种，mode 0=opacity）+ `collect_blur_filters`（Blur 2-pass H+V 高斯）+ `collect_transforms`（CSS transform 2D 仿射逆矩阵）。`apply_color_filters_headless`/`apply_blur_filters_headless`/`apply_transform_filters_headless` 实证为真实 ping-pong A→B→A（scissor pass 采样+滤镜+回写），匹配 CPU `apply_filter`，**非 stub**。
- **clips（0 GPU 引用）= 合法 no-op**：engine 生产路径从不生成 ClipPrimitive（R220 已证，overflow 裁剪预烘焙进图元几何）。
- **blend_modes（0 GPU 引用）= 唯一真实 DC-9 GPU 缺口**：engine 生产**生成**（effects.rs:314，CSS `mix-blend-mode`），但 GPU 静默丢弃。实现需 **backdrop 采样**（元素内容与背后已渲染内容按 blend 方程合成）= 元素需渲到独立层再与 backdrop 合成，复用现有 ping-pong 但需 per-element-layer 渲染顺序改动，**复杂**且 **mix-blend-mode 在上游 reftest 中近乎零覆盖**。
- **DropShadow filter = 双路径一致 no-op**（CPU effects.rs:192 `_` + GPU 不收集），罕见，非 DC-9 阻塞。

**结论**：DC-9 GPU 覆盖 **≈92% 满足**（11 图元 + 全 filter 子类型独立管线，非 CPU passthrough 满足 DC-14）。唯一缺口 blend_mode 复杂（backdrop 采样）且无 reftest 验证，非单会话可验证落地目标。**纠正 R220/R302 计划**：filter:opacity 无需实现（已 done），下一步勿再以「DC-9 GPU filter」为 lever。

**对优先级队列影响**：DC-9 实质接近完成（仅 blend_mode + DropShadow 残留，均复杂/罕见/无验证）。clean-win 穷尽 + DC-9 接近完成 → 渲染兼容性目标的**剩余真实缺口集中**在：① 结构性（Phase A IFC 文本度量 / intrinsic sizing / multicol 碎片化 / writing-mode 轴）；② 特性缺口（blend_mode backdrop / iframe 子文档加载 / 原生表单控件 / dialog JS）；③ taffy 限制（grid auto-track growth / flex intrinsic sizing）。这些均非单会话 clean win，需多轮架构（spec-rfc）或上游升级（taffy）。read-only 审计，无代码/reftest 变更，基线持平。

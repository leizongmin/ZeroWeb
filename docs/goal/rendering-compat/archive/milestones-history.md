# 历史里程碑归档

> **说明**：本文档归档 `rendering-compat.md` 主文档中已完成的历史里程碑详情。活跃的 M7-M11 在主文档中保留精简版本。

## M2 — CSS 2.1 核心渲染修复 + Quirks Mode

**目标**：修复 CSS 2.1 reftest 发现的渲染错误，实现完整 quirks mode，达到 CSS 2.1 核心通过率 ≥ 95%（基于上游真实 WPT reftest）。

**范围**：
- 盒模型计算精度
- Margin 折叠
- BFC 触发与隔离
- Inline formatting 正确性
- 颜色和背景绘制
- 边框绘制（border-radius、border-style）
- 基础定位（static/relative/absolute）
- Float 基础布局（含 clear）
- **Quirks mode 完整实现**：
  - CSS parser：quirky color values、quirky unitless lengths、quirky hash-less color
  - Style system：quirks mode 特定样式规则（表格高度 quirks、百分比高度 quirks、inline 元素宽高 quirks）
  - Layout engine：quirks mode 特定布局行为
  - DOM parser quirks mode 状态传递到下游链路

**依赖**：M1 完成（需要 reftest 基础设施来验证修复）

## M3 — Flexbox + Grid 渲染修复

**目标**：修复 Flexbox 和 Grid reftest 发现的渲染错误，达到各自通过率 ≥ 95%（基于上游真实 WPT reftest）。

**范围**：
- Flexbox 所有子属性的正确布局
- Grid 所有子属性的正确布局
- 响应式布局 edge case
- 嵌套 flex/grid 场景

**依赖**：M1 完成

## M4 — Float + Table + Multicol 布局兼容性收敛

**目标**：在已落地 Float、Table、Multi-column 基础算法的前提下，继续修复真实 WPT reftest 暴露的结构性残余，使各自通过率达到 ≥ 95%（基于上游真实 WPT reftest）。

**范围**：
- Float 残余边缘 case（table+float、margin-collapse、BFC containment 等）
- Table 残余边缘 case（border-collapse、vertical writing-mode table、spanning 深水区等）
- Multi-column 残余边缘 case（nested multicol、column-span、fragmentation / balancing、LayoutNG 对齐）
- position: fixed/sticky 的精确实现

**依赖**：M1 完成（M2/M3 可并行）

## M5 — 文字排版能力实现

**目标**：实现完整的文字排版能力，达到文字排版 reftest 通过率 ≥ 95%（基于上游真实 WPT reftest）。

**范围**：
- OpenType shaping（ligatures、kerning、features）— `rustybuzz` 已接入，后续修具体 reftest 缺口
- BiDi 算法实现 — `unicode-bidi` 已接入，后续修具体 reftest 缺口
- CJK 排版优化
- writing-mode: vertical-* 实现
- text-align: justify 的精确实现
- word-break / overflow-wrap / hyphens 的完整实现
- text-decoration 的精确绘制

**依赖**：M1 完成（M2/M3/M4 可并行）

## M6 — 全量扩展 + 通过率冲刺（已声称完成）

**目标**：扩大各领域 reftest 覆盖范围，达到总体 95%+ 通过率。

**范围**：
- 扩大每个目录的 reftest 导入数量（目标每个目录 ≥ 100 个 case）
- 修复所有剩余渲染缺口
- CPU + GPU 双模式验证
- 回归测试确保已通过的 case 不退化

**依赖**：M2-M5 完成

**状态**：⚠️ 已声称完成（685/685 inline reftest 100% 通过），但审计发现这些 reftest 均为手写简单场景，**不是上游 WPT 真实 reftest**，未覆盖渲染器实际输出能力缺口。真实渲染效果仍然与主流浏览器差距巨大。后续 M7-M11 里程碑旨在解决这些根本问题。**本目标的通过率标准必须基于上游真实 WPT reftest**，685 个 inline reftest 不计入通过率统计。

## M7 背景事实（⚠️ pre-M7 历史快照）

> **注意**：M7 已完成，「断桥」已修复，见 DC-8/9/10 ✅。本节仅作为 pre-M7 历史快照保留。

**当时（pre-M7）**渲染管线存在一个严重的「断桥」（下列 3 项现已全部修复——CPU/GPU/浏览器消费全 13 图元）：

1. **Paint 系统**（`crates/engine/src/paint/`）已能生成 13 种图元类型 ✅
2. **CPU 渲染器**仅渲染其中 3 种（fills、rounded_rects、glyphs）❌
3. **GPU 渲染器**仅渲染其中 2 种（fills、glyphs）❌
4. **浏览器 `append_webview_primitives()``** 仅传递 2 种到渲染器 ❌

这意味着渐变、阴影、图片、线段（边框虚线/点线）、路径、变换、裁剪、滤镜、混合模式全部在渲染阶段被静默丢弃。

---

**完成状态**：以上 M2-M6 历史里程碑已完成或已过时。当前活跃里程碑为 M7-M11（在主文档 `rendering-compat.md` 中保留精简版本）。

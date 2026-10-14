# 行内格式化与布局结果复用

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

本层按文本收集、字体/书写模式与最终结果复用分开阅读，形成第 5 层 IFC 叶子。普通行内算法不归 Taffy；真实字体来自图形底座，DOM 几何和 paint 则消费终化结果。

## Architecture
- final IFC 与 paint replay 共用属性 resolver 和 override 重键，但 stored 行盒有独立资格门；排查“布局正确、绘制换行错误”先确认实际走 Path A 还是 Path B。 (`crates/layout-engine/src/inline_finalization.rs`, `crates/engine/src/paint/painter/text.rs`)
- 中间 float/inline-only 重测与最终 IFC 均会改变几何与度量，finalized 集只能在限定条件下免掉重复执行，不是全页结果缓存。 (`crates/layout-engine/src/engine.rs`, `crates/layout-engine/src/final_ifc_reuse.rs`)

## Child Knowledge Nodes
- [文本收集、空白、断行与双向文字](collection-and-line-breaking.md) — 消失的 nbsp、换行、OOF 文本泄漏与 bidi 顺序。
- [度量、基线、竖排、ruby 与替换元素](metrics-and-writing-modes.md) — 字体源、line-height、比例与物理轴映射。
- [最终 IFC、几何缓存与绘制一致性](finalization-and-reuse.md) — stored/replay 分支、回填、line-clamp 与重复绘制。

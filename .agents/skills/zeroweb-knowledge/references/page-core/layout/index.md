# 格式化上下文与布局算法

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 `layout-engine` 与生产 `taffy-local` patch：盒树/Taffy 初算与重算→浮动和 IFC 重测→表格→多列→定位修正→最终 IFC/几何。源码以协调器的现行调用次序为准，域内纯算法和历史 dormant 注释不代表当前接线状态。

## Gotchas
- 全量布局和 compute_incremental 的后处理覆盖不同，mutation 管线已撤用 SetText 增量臂；定位错误不能凭“缓存复用成功”排除布局分支差异。 (`crates/layout-engine/src/engine.rs`, `crates/engine/src/pipeline/mod.rs`)

## Architecture
- float 表格预定尺寸、表格遇浮动的 scoped 修正、多列收缩 restack 与最终 IFC 都会再改几何；仅观察最终盒宽高不能归因某一 CSS 机制。 (`crates/layout-engine/src/engine.rs`)
- 行度量和字宽由两个 provider 注入，final IFC 又把 replay 所需样式存到盒；字体接口变化同时影响 intrinsic、断行、baseline 和 paint 消费。 (`crates/layout-engine/src/inline/font_metrics.rs`, `crates/layout-engine/src/inline/text_metrics.rs`, `crates/layout-engine/src/inline_metric_storage.rs`)

## Child Knowledge Nodes
- [盒树、尺寸、定位、浮动与 margin](boxes-and-positioning.md) — 包含块、根节点、浮动排斥、intrinsic 与后处理次序。
- [Flex、Grid 与本地 Taffy 契约](flex-grid-and-taffy.md) — base size、ratio 传递、轨道与补丁 API。
- [行内格式化与布局结果复用](inline/index.md) — 文本、字体、竖排、权威 IFC 和 replay。
- [表格网格、边框与可见性](tables.md) — 匿名表结构、行组、列压缩、rowspan 和 collapse。
- [多列、碎片化与分页](fragmentation.md) — 视觉片段、nested spanner、溢出列与 Print 边界。
- [增量布局、脏标记与受控管线](invalidation-and-budget.md) — mutation 回退、预算粒度与脏区域口径。

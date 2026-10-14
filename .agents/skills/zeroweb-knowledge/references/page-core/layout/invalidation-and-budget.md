# 增量布局、脏标记与受控管线

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 LayoutDirtyTracker、屏幕 DirtyTracker、mutation 分支和 BudgetedRenderSession。布局脏节点、像素脏矩形、预算阶段状态是不同契约；预算会话不替代进程树内存与墙钟守卫。

## Gotchas
- `LayoutDirtyTracker::mark_subtree_dirty` 接收调用方给出的节点数组，本身不遍历 DOM；drain_dirty 同时清掉 full_recalc，不能先 drain 再判断是否必须全量。 (`crates/layout-engine/src/dirty.rs`)
- `compute_incremental` 只 mark_dirty 已缓存的 Taffy 节点，没有更新其 style 快照；样式从 height:50 改120的常驻测试明确断言增量仍保持50，这是边界测试而不是正确性 PASS。 (`crates/layout-engine/src/engine.rs`, `crates/layout-engine/src/engine/tests/incremental_parity_experiment.rs`)
- SetText 曾使用增量布局，但绝对定位/inline 锚页面暴露缺失 final IFC、margin 与 abs 后处理；当前 mutation 管线与结构变化一起快照全量重建，双块实验不能证明可重新启用。 (`crates/engine/src/pipeline/mod.rs`, `crates/layout-engine/src/engine/tests/incremental_parity_experiment.rs`)
- `advance_budgeted_render` 只在完整 Parse/Style/Layout/Paint 之间检查 deadline，单个阶段不会中途抢占；budget_ms 是推进预算，不是保证本次调用耗时上限。 (`crates/engine/src/pipeline_budget.rs`)
- DirtyTracker 的 merge 先按 left 排序再只与最后一个 merged rect 比较，合并条件是 union 面积≤两面积和的1.5倍，并非严格相交；不保证全体二维重叠都合并。 (`crates/engine/src/dirty.rs`)
- `incremental_render` 根据脏面积决定 full_redraw 后仍调用完整 render_html；函数名和脏矩形统计不证明已省掉 parse/style/layout。 (`crates/engine/src/pipeline/mod.rs`)

## Architecture
- LayoutDirtyTracker 管 DOM NodeId 与 full_recalc，DirtyTracker 管屏幕 Rect 与 full_redraw；mark_node_dirty 的 offset 由调用方给出，不自动上行累积祖先。 (`crates/layout-engine/src/dirty.rs`, `crates/engine/src/dirty.rs`)
- mutation 仅全部匹配 paint-only 白名单才复用布局重 style+paint，表单 value-only 有单独分支，混合或结构 mutation 回退全量；defer 模式只 apply，批量边界再渲染。 (`crates/engine/src/pipeline/mod.rs`)
- 预算会话跨帧持有 Document、styles、layout 与 result，Done 后 take_result 消耗结果；渲染结束替换 cached_doc 并清 CSS 缓存和持久 handle 映射，防旧 NodeId 跨文档代际复用。 (`crates/engine/src/pipeline_budget.rs`)

## Patterns
- paint-only 成功后刷新 cached_stylesheets 供动画时钟泵读取 inline transition，否则仅 opacity 等样式变化已上屏而 transition 注册表仍旧。 (`crates/engine/src/pipeline/mod.rs`)
- Taffy mark_dirty 递归清祖先缓存，遇 AlreadyEmpty 停止，依赖祖先已脏的不变式；DOM tracker 只是集合，不承担这条传播。 (`crates/taffy-local/src/tree/taffy_tree.rs`, `crates/layout-engine/src/dirty.rs`)

# 最终 IFC、几何缓存与绘制一致性

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位最终行盒、override 存储、几何回填与重复绘制。`inline_finalization.rs` 产出布局度量，`inline_metric_storage.rs` 保存空 styles replay 的上下文，`paint/painter/text.rs` 在 stored Path A 与重跑 Path B 间选择。

## Gotchas
- “已 finalization”不是任意复用许可：`can_skip_final_ifc` 同时要求无 float、多列、line-clamp、size-adjust、orphan 回填需求，单 generic family、无 stored layout 且已有字号/行高表，任一条件失败都重算。 (`crates/layout-engine/src/final_ifc_reuse.rs`)
- 布局执行 IFC 不意味着保存权威行盒：普通最终存储限纯 Ahem，直接 float exclusion 可放开非 Ahem，浮动容器多行仍拒绝；override 度量在 store gate 之前保存。 (`crates/layout-engine/src/inline_finalization.rs`)
- paint 只有无 multicol、存在 stored 行盒且记录宽度与当前 IFC 宽差小于 1px 时使用 Path A，否则重跑；后处理改宽度可能让已有 stored 布局自动失效。 (`crates/engine/src/paint/painter/text.rs`)
- 普通 inline 自身不再跑独立 IFC，文本归父；例外包含 flex/grid item、带裸文本 table-internal 和竖排 table-cell，不可只按 display:inline 判定全部早退。 (`crates/layout-engine/src/inline_finalization.rs`)
- 纯 inline 嵌套允许 stored，但后代块、嵌套 br、ruby 或与外层不同字体度量仍排除；paint 的正文入口需同步守卫，ruby 在 paint 放行 Path B，不等于 layout 保存 ruby 行盒。 (`crates/layout-engine/src/inline_finalization.rs`, `crates/engine/src/paint/painter/text.rs`)
- line-clamp cap/clamped 必须在 store gate 前存到盒；Path B 空 styles 无法解析 auto cap，Path A 已截断后也不能再用“现有行数超过 cap”判断省略号。 (`crates/layout-engine/src/inline_finalization.rs`, `crates/engine/src/paint/painter/text.rs`)
- `inline_reported_rect` 不覆盖树内行盒高，但 gBCR 与默认命中均消费上报 border-box；命中对子盒累积仍用树几何，ZW_INLINE_HIT_SURFACE=0 才回退原行盒命中，不能把上报值直接覆盖布局 y/height。 (`crates/layout-engine/src/types/mod.rs`, `crates/engine/src/hit_test.rs`, `crates/engine/src/rect_bridge.rs`)

## Architecture
- white-space 和 fallback glyph 垂直度量按 fragment NodeId 与 owner 元素双键存储，兼容 text 分支及 flatten element 分支；仅保留文本键会让 Path B 展平后丢上下文。 (`crates/layout-engine/src/inline_metric_storage.rs`)
- font family、size-adjust、bidi override、text-transform 与间距随 fragment 保存；新增影响文本输出的字段必须审计 TextFragment→InlineLayoutFragment→PaintFragment 和 replay override 各转换。 (`crates/layout-engine/src/inline_metric_storage.rs`, `crates/layout-engine/src/inline_finalization.rs`, `crates/engine/src/paint/painter/text.rs`)
- orphan inline 与 run-in 回填代理 LayoutBox 同时登记 paint_skip，分别满足几何可见与抑制自盒绘制；只回填几何而漏登记会重复绘制父 IFC 已包含的文字。 (`crates/layout-engine/src/inline_finalization.rs`)

## Patterns
- 复用策略的测试逐一关闭十个布尔条件并断言拒绝，覆盖 fail-closed 组合边界；它验证策略函数，不等于生产渲染已经验收。 (`crates/layout-engine/src/final_ifc_reuse.rs`)
- 同一 override 重键 helper 在终化与 paint 复用，过滤混入的 inline 元素片段以避免 HashMap last-write-wins 不确定性，不能独立各写一份转换。 (`crates/layout-engine/src/inline_finalization.rs`, `crates/engine/src/paint/painter/text.rs`)

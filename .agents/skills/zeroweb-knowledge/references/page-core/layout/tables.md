# 表格网格、边框与可见性

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

查 `adjust_table_layout_with_fonts`、`build_grid`、列压缩、collapse 边框与匿名表盒。Taffy 将 table 系列映射成 block，表格后处理重新分配几何；表格遇到浮动及多列时另沿 [盒定位](boxes-and-positioning.md) 与 [碎片化](fragmentation.md) 查。

## Gotchas
- inside_table 标记在普通 block 或经 float 块化的内部表盒处中断；如果仅继承祖先曾是 table，会漏掉块化子树内部所需的匿名 table 修复。 (`crates/layout-engine/src/table.rs`)
- 连续孤立 table-internal 兄弟合并进一个匿名 table；单孤立 cell 的专门包装限竖排，水平保留旧 retrofit 路径，不能据 CSS 理想模型宣称所有匿名表盒都统一实现。 (`crates/layout-engine/src/table.rs`)
- 只有首个 thead 和首个 tfoot 被移至头尾，后续同类型行组保持 DOM 序；优先级在排序前一次标记，不能在 sort key 回调里改变 seen 状态。 (`crates/layout-engine/src/table_grid.rs`)
- 真 table 容器和 node_id=None 的合成匿名 table 才走一般匿名 cell 包裹；带 NodeId 的孤立内部盒即使 `is_anon_table_root=true` 也不等同该上下文。 (`crates/layout-engine/src/table_grid.rs`)
- 当前 rowspan 占列簿记仅竖排启用，水平分支返回空 blocked-cols；竖排定位还把 rowspan cell 的物理 x/width 扩到跨行区间，旧“rowspan 不支持”的注释不能覆盖现行分支。 (`crates/layout-engine/src/table_grid.rs`, `crates/layout-engine/src/table/table_vertical.rs`)
- 压缩列后不可只缩 auto block 宽度而不重测行高；老 clamp helper 仅处理 max-content 装得下的内容，forced clamp+remeasure 路径才覆盖 wrapping 内容。 (`crates/layout-engine/src/table_cell_content.rs`)
- 被 visibility:collapse 折叠的真实 row 不贡献行高和相邻 spacing；匿名 row 没有可设置 visibility 的对应元素，检测始终不折叠。 (`crates/layout-engine/src/table_visibility.rs`)
- collapsed 列边框半宽 helper 仅计算 col/colgroup 自身边，不完整重做 cell/table 冲突；它与 cell explicit-width 既有贡献配合，不能当作通用 border winner 求解。 (`crates/layout-engine/src/table_borders.rs`)

## Architecture
- 表格高改变后立即重排后续兄弟；abspos table 又按内容求解后的高度重新做垂直 auto-margin 居中，不能沿用 Taffy stretch 高。 (`crates/layout-engine/src/table.rs`)
- `remeasure_cells_at_compressed_widths` 在 position_cells 消费行高前做重测，传入字体上下文但 image intrinsic 表为空；解码图片比例缺口不能归为字体换行问题。 (`crates/layout-engine/src/table_cell_content.rs`)
- 竖排行本身代持匿名 cell 时只同步 content 尺寸，不以 cell.x=0 覆盖已定位行盒；列槽映射到物理 y，colspan 汇总列宽，rowspan 跨物理 x。 (`crates/layout-engine/src/table/table_vertical.rs`)

## Patterns
- R5097 用纯文本 auto-height table 常驻断言内容高、收缩宽与后继不重叠，并用元素子和 definite-height 负例收窄修复；其测试通过 inline style 输入，空外部样式表不读取 HTML style 块。 (`crates/layout-engine/src/engine/tests/r5097_table_raw_text_anon_cell_tests.rs`)
- collapsed-border 冲突入口与列边框贡献分开，后者用列覆盖游标处理 col/colgroup span；改列结构需同时核对网格、列宽与边框覆盖，不只改 build_grid。 (`crates/layout-engine/src/table_grid.rs`, `crates/layout-engine/src/table_borders.rs`)

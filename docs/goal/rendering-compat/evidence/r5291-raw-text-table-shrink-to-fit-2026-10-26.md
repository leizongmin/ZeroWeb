# R5291 — 裸文本表 shrink-to-fit（折行重测）——inline-table-width-001b ✓0.00%

日期：2026-10-26。改动：`crates/layout-engine/src/table.rs` `size_table_with_raw_text_content`（R5097 臂内 +29/−2）、`crates/layout-engine/src/engine/tests/r5097_table_raw_text_anon_cell_tests.rs`（新常驻测试 2）。无新开关（行为在 R5097 既有 `ZW_TABLE_RAW_TEXT_ANON_CELL` 门内）。

## 案面

`inline-table-width-001b`（账本 14.97 前二 `before-after-table-parts-001` 之外的暖域候选 3.02%）test/ref 双页不对称：

- **test 页 ✓**：`div` 显式 `width:10em` 的 inline-table → 正确 160 宽 4 行折行。
- **ref 页 ✗**：`div` 块级 `display:table`（auto 宽）含裸文本 → ZW 渲染 491px 单行溢出（应 160 宽折行）。reftest 两页均 ZW 渲染 → 差异全在 ref 页。

## 取证链

1. `product-smoke` 单页 + PIL：test 页绿块 160×72 四行 ✓；ref 页绿块 ~500×18 单行、文本截断于 "than"（应折行）。
2. `ZW_LAYOUT_DUMP_REF=1 layout-dump`：REF 页 table 盒 **w=491 h=18**（父容器 w=160）——max-content 宽被固化，available 约束缺失。
3. 探针（临时 eprintln，已删）：`size_table_with_raw_text_content` 入口 `table_box.content_width = 160`（taffy 对 block 表 leaf stretch 到包含块）、MaxContent 测量 `491.48×18`——**taffy 分配宽就是包含块约束**，R5097 直接固化 MaxContent 单行。

## 实现

R5097 臂内加 shrink-to-fit（CSS2 §17.5.2）：max-content 测量宽超出 taffy 分配宽（available）时按 `Definite(available)` 重测——宽取 available、高取折行后 IFC 高（`.floor()` 沿用 R4333 整数相位口径）。`available ≤ 1` 为 taffy sizing 退化（contain:inline-size，同 R4298 `col_available >= 1.0` 口径）回落旧行为。

`// FIXME:` min-content 下限未建模（最长词宽于 available 时表应取词宽、词溢出由 overflow 呈现）——案面明确「no words wider than 10em」，超长词角案驱动另切片。

## 交付

- `inline-table-width-001b` **✗3.02 → ✓0.00**；inline-table-width 族 4/4 全 ✓。
- 全量 corpus **16590/15728P/862F**：失败集 vs R5290 交付态（863F）**恰移出 001b 一案、零新增翻转**（sorted A/B：`/tmp/r5291-fail.txt` vs `r5290-fail-base.txt`−085/086）；真通过-可信 10235（61.7%，基线 61.6%）。
- R5097 同路径锚点 run-in 族 133 案 130✓/3✗，3✗（run-in-inherit-001 / run-in-replaced-001 / run-in-run-in-between-005）全为基线既有失败。

## 教训

- reftest 双页不对称案先分清哪页错：`--out` 单页渲染 test 与 ref 各自对照源码期望，别默认 test 页是错的那页（本案 test 页全对）。
- taffy 对 block leaf 的 stretch 分配宽 = 包含块约束，可直接作 available 消费——R5097 首版把「taffy 已定宽」误解为「无 available 信息」，实际约束一直在盒上。

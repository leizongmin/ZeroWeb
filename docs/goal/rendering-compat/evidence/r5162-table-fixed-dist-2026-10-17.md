# R5162 域核记录 — table-layout:fixed 列宽分布（2026-10-17）

驱动案：`css/css-contain/contain-paint-cell-001.html`（1.79%）+
`contain-paint-cell-002.html`（1.88%）（R5161 下一步 ② css-contain 余红续扫，
R5161 同根因改善过的近亲）。

## 1. 域核

layout-dump + 单测探针（Ahem 100px，ch=0.5em 兜底）：

- cell-001：`display:table; table-layout:fixed; width:200px` + 中列 col 4ch=200
  三列表，ZW 列宽 [21.43, 157.14, 21.43]，chromium [0, 200, 0]——specified 列被
  摊薄、auto 列分到残余。
- cell-002：真 `<colgroup><col><col>{width:4ch}</colgroup>` 同形。

**根因**（CSS 2.2 §17.5.2.1）：fixed 布局列宽只来自 col 元素/首行显式宽单元格的
specified width（步骤 1-2），剩余表宽由 auto 列**均分**（步骤 3「equally
divide」），单元格内容不参与列宽。旧路径把内容固有宽 `.max()` 进列宽后按
fixed_capped 比例收缩：空 cell 的 strut 残留 min-content（0.6em）与指定内容宽
一起把 specified 列摊薄——cell-001 [60,440,60] 按比例压到 200。

**上轮误设纠正**：R5162 首版认为真 `<col>` 元素不生成布局盒、盒循环不可见，在
`collect_col_widths` 加了 DOM 层回退遍历（`ZW_TABLE_DOM_COL`）。本轮 kill-switch
反证：`ZW_TABLE_DOM_COL=0` 时 cell-002 与真 col 单测仍全绿——盒循环（匿名盒修复
为 col 元素生成盒）+ R2045（HTML width 属性 → CSS computed width）早已在位，
回退遍历是无贡献死代码，**移除**（教训：防御层先做 OFF 反证再留）。

## 2. 实现

`table.rs compute_column_widths_inner`（kill-switch `ZW_TABLE_FIXED_DIST=0`）：

- `cell_used_width` 返回三元组，新增原始 specified px（无 R364b 内容 floor、
  无 collapse 半宽加成——fixed 布局内容不参与列宽，floor 的 min-content 属内容域）；
- `is_fixed_layout` 判定上移到 Pass 1 前，Pass 0 后快照 `fixed_spec_cols`
  （col 元素未污染宽，含 R3810 col 边框半宽）+ `col_cell_spec`（单元格原始 specified）；
- fixed + 显式表宽 + `col_available > 0` 新分支：
  ① specified 列取快照宽、auto 列内容贡献清零；
  ② `remaining = col_available − Σ specified` > 0 时由 auto 列**均分**；
  ③ 无 auto 列且 remaining > 0 时均分给 specified 列（css-tables-3 §3.4.3 CAP，
  separated-border-model-003b：单 td 100 的 500 表 spacing 50 → td 300，chromium
  实证——旧路径经 expansion 比例放大偶然达标）；
  ④ 折叠列宽度 0（与 auto 路径同口径）；early return，`cols_auto_shrunk=false`。
- 仅覆盖显式表宽；auto 宽 fixed 表维持旧路径（CAP/MIN 交互未案驱动）。

## 3. 验证

- 单测 +5（r5162_table_fixed_dist_tests）：specified 列不摊薄 / auto 均分 /
  内容不 dilate / 真 col CSS 宽 / auto 宽表旧路径保持。
- **OFF 反证**：`ZW_TABLE_FIXED_DIST=0` 目标双案精确复红（1.79% / 1.88% 基线值），
  auto 宽用例不受影响。
- 全 corpus 16,590：fail 1,124 → **1,121**（**+3 翻绿 0 新红 0 churn**）：
  cell-001 / cell-002 / fixed-table-layout-025 全部 **0.00%**。
- 严格带迁移：+5 进（fixed-table-layout-026 0.35→0 / 028 0.77→0 / 030、031
  0.85→0 / separated-border-model-004c 0.62→0）；−8 出（003e07-010 0.08→0.63、
  003f03-06 0.08→0.45-0.49，均仍 PASS）。真通过-可信 9,793 → **9,802**。

## 4. 中途回归定谳与门（padding-applies-to-013a）

首版全 corpus 引入 1 新红：`padding-applies-to-013a` 0.625%（PASS）→ 1.04%。
layout-dump 对比：新分支把两 auto 列分到 0 宽（旧路径 strut 9.6px）。

根因不在新算法，在 **converter 表宽语义分叉**：R4243 口径把 specified width 按
border-edge 语义扣 pb——对 **HTML `<table>`** 正确（separated-border-model-003b：
width 500 = border-edge，assert 明言）；但 **CSS `display:table` div** 的 width
是 content-edge（013a：width:50 + padding:25，chromium content=50）→ ZW 侧
col_available 退化为 0，均分无意义且 0 宽列比 strut 路径更差。

处置：分支加 **`col_available > 0` 门**——退化角回落 legacy（013a 复测 0.62%
PASS），converter 双语义分叉记 FIXME 挂账另案（涉及 R4243 已定口径，需独立域核）。

## 5. 门禁记录

fmt / clippy（workspace --all-targets -D warnings）0 告警；layout-engine 全测
1955+5+1 绿；make reftest 711/711；make test 71 套全绿（21,045P，含 browser xvfb
腿 + GPU adapter 腿 + quickjs 腿）；product-smoke 6 页 struct PASS（welcome
17.07% 同值）；product-smoke-legacy 42 fixture struct 0 FAIL；bench-gate 定向
zero-layout-engine：ON 10 FAIL → ON 复跑 8 FAIL → OFF 6 FAIL、三跑 NEW=0、
集合逐跑漂移（wide_tree↔block/flex↔medium/paint 互换、OFF 反更差）→ 判属
R5149-R5161 环境噪声族延续（本轮跑窗与并行流 rustc 重载重叠，load 3.4-8.3），
非本轮引入，不 relax。

## 6. 限制/挂账

- collapse 模式 specified 列取原始 specified，未加 collapse 半宽（盒循环同口径
  R364b 在 auto 路径有）——003e/003f 簇 0.08→0.45-0.63% 带迁移疑似来源，均仍
  PASS，案驱动另案。
- converter 表宽双语义分叉（`<table>` border-edge vs display:table content-edge）
  挂账——013a 当前以 strut 补偿 PASS（0.62%），真修需独立域核。
- 无 auto 列且 remaining < 0（specified 和超表宽）保持 specified（CAP），表盒
  加宽由既有路径消费；未案驱动。

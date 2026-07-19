# RFC：匿名 table-cell 盒生成（CSS2 §17.2.1）

状态：⬇ DOWNGRADED（R1754 de-risk 证 display-normalization 不足 + net-negative；真挑战 = table 跨行 margin-collapse/row-height-distribution，非 cell generation；降级为 documented edge case，不建议 dedicated 推进）
日期：2026-07-19（R1752 根因确认 / R1753 本 RFC / R1754 de-risk 负结果）
承接：R1752（characterize anon-table-cell-margin-collapsing bug）forward ①
谱系：css-tables/anonymous-table-cell-margin-collapsing + CSS2/tables/table-anonymous-objects-197..204（9 案）

---

## 0. R1754 de-risk 负结果（display-normalization 不足 + net-negative）

R1754 实验简化版 Slice A：style-system compute_style 把 table-row/row-group 的 block-level
非 table-internal 子 display 归一化为 TableCell（kill-switch `ZW_ANON_TABLE_CELL`，不生成
独立包装盒）。结果：

- **归一化生效**（dump 实证 div 现 `disp=TableCell`），但 **css-tables 目标案 table 仍 h=150**（未修）。
- **CSS2 oracle 4464/6219 vs baseline 4465/6219 = NET −1**（1 回归）。
- **revert**（实验代码还原，working tree clean）。

**真挑战（精化认知）**：css-tables 案 table h=150（应 100）根因**不是** cell 缺失（build_row 已
把 div 作 cell，归一化让它成真 TableCell），是 **table 跨行 margin-collapse + definite-height
row-height-distribution**——`height:100px`（minimum）下 row1 h=100 + row2 h=50 = 150 应被分布到
100（margins 跨行折叠 + 行高重分配），ZW 行高纯内容驱动不分布。此为 **table-level margin-
collapse/distribution**，属 R1630（broad-table 5 轮 abandon）+ R1047（margin-collapse postprocess
net-negative）谱系**深水区**，非本 RFC 的 cell generation 范围。

**裁决**：~9 yield 涉及深水区，ROI 不足 → 降级为 documented edge case，**不建议 dedicated 推进**。
下方 Slice A/B/C 设计保留作历史参考，但真入口须先解 table 跨行 margin-collapse/distribution
（独立更深 RFC，非 cell generation）。


---

## 1. 背景 + 确认的 bug

CSS2 §17.2.1：table-row / table-row-group / table 的非 table-cell 子元素（block `<div>` /
inline / 裸文本）须被**匿名 table-cell 盒包裹**（anon box，display:table-cell，无 node_id）。

**ZW 现状（R1752 LAYOUT_DUMP + PIL 像素采样实证）**：`build_row`（table_types.rs:186）
把 table-row 的**每个 in-flow 子无条件下标为 cell**（line 203 `cells.push(TableCell{child_index:
cell_idx, ..})`），但 cell box 直接指向该子 LayoutBox——子保持其原 display（block div =
`Element[Block]`），**不生成 anon table-cell 包装盒**。

**症状（anonymous-table-cell-margin-collapsing.html，css-tables 3.40%）**：
- table `height:100px` bg:red，3 rows bg:green，row 内 block div `margin:50px 0`。
- ZW 渲染 green **110×151px**（chromium ref = 100×100 filled green，red=0）。
- 根因：block div 作 cell 但 display=Block → 不建立 table-cell BFC → margin 不折叠 / 行高
  累加 100+50=150 → 溢出 height:100px；宽多 10（第二 anon cell div 仅 10px 宽，列宽算法
  对 block-as-cell 失真）。

**table-anonymous-objects-197**（CSS2/tables，self-source）：`<span display:table-row>` 内
`<span display:table-cell>a</span> bc <span display:table-cell>d</span>`——裸文本 " bc "
须成 anon cell。ZW 渲染 test red=113（ref red=0，green 同 348）→ "bc" anon cell 缺失致红露出。

**yield basis（R1753 grep + 渲染确认）**：9 案 = css-tables×1（3.40%，oracle set）+ CSS2/tables
table-anonymous-objects-197..204×8（self-source，CSS2 oracle 经 filesystem scan 含）。均为真
geometry bug（非 font-wall）。

## 2. 设计

### Slice A：tree-building 期生成 anon table-cell 包装盒（核心）

`tree.rs build_subtree`（或 table 专用 build 路径）：识别 table-row / table-row-group 的
**非 table-cell in-flow 子**（display 不是 TableCell/TableRow/TableRowGroup/TableCaption/
TableColumn(Group)），把**连续**的非-cell 子包成一个 anon `LayoutBox`（`display=TableCell`，
`node_id=None`，新 flag `is_anon_table_cell=true`），原子成为该包装盒的 child。

- 连续非-cell 子合并到一个 anon cell（CSS2 §17.2.1：consecutive non-cell children share one
  anon cell）——同 `direct_cells`（table_grid.rs:61）已有 anon-row 合并逻辑的对称。
- inline / 裸文本子也包（table-anonymous-objects-197 的 "bc" 文本）——须在 inline 收集
  前介入（build_subtree 对 table-row 子的 display 归一化）。
- anon cell 继承 table-row 的可继承样式（font/color 等），box-model 默认 0 padding/border。

### Slice B：build_row / get_cell_box 适配 anon 包装盒

`build_row`（table_types.rs:186）：cell `child_index` 指向 **anon 包装盒**（在 row_box.children
中，包装盒 display=TableCell），而非原 block 子。`get_cell_box` 返回包装盒；包装盒的 child
（原 block 子）正常 block 布局。constrain_table_cell_content_widths / position_cells 对包装盒
按 cell 语义处理（BFC、列宽分配）。

### Slice C：anon cell box-model + BFC

converter/postprocess：anon 包装盒设 `display=TableCell`（建立 BFC，§9.4.1），box-model
默认 0（无显式 padding/border）。使其内部 block 子的 margin 被包含（不穿出），与真 `<td>`
一致。margin-collapse-through-anon-cell（css-tables 案）由 table height distribution + cell
BFC 含 margin 共同决定——须 A/B 验证 197..204（span/text 案）与 css-tables（block+margin 案）
均向 chromium 收敛。

## 3. 实施顺序 + 验收

kill-switch `ZW_ANON_TABLE_CELL`（env，default-off，gate = table-row/table-row-group 有非-cell
in-flow 子）。每 slice 独立 A/B，net≥0 才 land：

1. **Slice A**（tree-building 包装）+ load-bearing 单测（table-row 含 block div → 生成 anon
   TableCell 包装盒，node_id=None，div 作其 child；纯 `<td>` 行不生成包装盒守卫）。A/B。
2. **Slice B**（build_row/get_cell_box 适配）+ 单测（cell box = 包装盒，包装盒 child = 原 div）。
3. **Slice C**（box-model/BFC）+ 单测（anon cell 内 block 子 margin 被包含，不穿出）。
4. 全量 A/B（见 §4）+ default-on + 删 kill-switch。

**flip 目标**：anonymous-table-cell-margin-collapsing 3.40%→<1%（flip）；table-anonymous-
objects-197..204 red→0（self-source test==ref 或 oracle flip）。

## 4. A/B 验收（须全过）

- **CSS2 oracle**（4465/6219 baseline）：tables 子簇 net≥0，目标 +8（table-anonymous-objects）。
- **css-tables oracle**（113/364 baseline 含该簇）：anonymous-table-cell-margin-collapsing flip。
- **CSS2 self-source reftest**（772 baseline）：tables 不回归（margin-collapse-101 / table-grid-*
  等已 PASS 案不受 anon 包装影响——gate 仅触发非-cell 子）。
- **writing-modes**：vertical 表（border-conflict-element-vlr/vrl 现 PASS）不回归——anon 包装
  对 vertical table-row 须对称（position_cells_vertical 路径）。
- **product-smoke** welcome/wintertc/morning struct 全 PASS（产品页用真 `<td>`，gate 不触发，
  字节一致零回归——关键安全属性）。

## 5. 风险与回退

- **broad-table-regresses 谱系（R1518/R1610/R1622/R1630）**：本 slice **blast radius 更窄**——
  gate 仅触发「table-row 有非-cell in-flow 子」，真 `<td>` 表（绝大多数 reftest + 全产品页）
  不受影响（字节一致）。与 R1630（改所有 collapsed 表的 cell 定位/border paint）本质不同。
- **margin-collapse 交互**：css-tables 案的 margin-through-anon-cell 收敛须实测；若 net-negative，
  scope-gate 排除 block+margin 子（仅 span/text anon cell 先 land，block anon cell 后续）。
- **vertical 对称**：vertical table-row 的 anon 包装须验 writing-modes 不回归。
- **回退**：kill-switch default-off 等效旧行为；net<0 整体 revert，本 RFC 留作重设计依据
  （同 R1628-R1630 先例）。

## 6. 不在本 RFC 范围

- collapsed-border overlap geometry（R1630 ⛔ABANDONED，独立）。
- anon table **root** 包装（is_anon_table_root，table.rs:170 已实现，本 RFC 是其 cell 层对偶）。
- font-wall 主指标 deadlock（独立多周工程，user-gated）。

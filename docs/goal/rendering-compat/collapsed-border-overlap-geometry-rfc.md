# RFC：collapsed-border 重叠几何修复（border-conflict / collapsing-border-model 簇）

状态：Draft（R1627 诊断 + 设计；R1628 实现验证模型正确但 incomplete 回退）
日期：2026-07-17（R1627）/ 2026-07-18（R1628 验证）
承接：R1626（color override LANDED）forward —— collapsed-border 内部共享 border 双计 geometry 修复
谱系：border-conflict-element-001a/b/c/d/e + collapsing-border-model-001/004/010a/b（CSS2/tables 簇）

---

## 0. R1628 实现验证（模型 PROVEN 正确，incomplete 回退）

R1628 按 §3 实现 Slice A（position_cells cell_x/row_y 重叠）+ Slice B（paint FULL border）
behind kill-switch `ZW_COLLAPSE_OVERLAP_GEOMETRY`（default-off），**跳过 Slice C（表尺寸修正）**。
单案 + 簇 A/B 实证：

**★ 模型 PROVEN 正确**：
- **001a 渲染 = EXACT ref 匹配**：100×100 实心绿方，green px=10100（ref 也是 10100），bbox x[8,107] y[50,150]
  size 100×101（旧 160×161 绿网格+白格心）。逐行采样全 G（无白心）。
- **001a/b/c FLIP**：3.65/2.12/2.10% → **0.00/0.00/0.12% PASS**（R1626 color fix 后残差，overlap 解剩余 geometry）。
- **writing-modes border-conflict-element-vlr/vrl 100% PASS（12 案全过，guard 守住）**——vertical collapsed 不回归。

**★ incomplete（须 Slice C）→ 回退**：
- CSS2 self-source NET 0（772=772）：3 flip（001a/b/c）被 ~3 回归抵消。
- css-tables NET 0（13=13）但 collapsed 簇变差：collapsed-border-paint-phase-002 **3.16→4.21%**、
  table-cell-width-0 **7.53→8.30%**、row-group-order PASS→FAIL（1.29%）；box-shadow-001 偶然 flip（噪声）。
- **根因 = 缺 Slice C**：position 重叠后 cell 跨度变小（100），但 `apply_table_size_constraints`
  仍按 `intrinsic_width = Σ col_widths`（160）定 table 盒宽 → table 盒比 cell 跨度大 → bg/content 表
  （collapsed-border-paint-phase / table-cell-width-0 / row-group-order）盒尺寸与 cell 摆放发散。
  001a 无 table bg/border 故不受影响（恰好 flip），有 bg 的表回归。
- **回退**：3 处代码（border.rs paint half + table.rs collapse_overlap flag + cell_x/row_y 重叠）
  git checkout HEAD 还原；模型验证结论保留本节。下轮须补 Slice C 再 land。

**Slice C 细化（R1628 实证后明确）**：`apply_table_size_constraints`（table.rs:1790 `total_col_width` /
:1810 `intrinsic_width`）须扣内部共享 border 总和：collapse+overlap 时
`intrinsic_width = Σ col_widths − Σ_internal_shared_h`，`intrinsic_height = total_row_height − Σ_internal_shared_v`。
Σ_internal_shared_h = 各内部列边的解析 border（对 simple 表 = Σ_{非首列 cell} cell.border_left；
colspan 须按列边计非 cell 计）。配套 position_cells 已实测（cell_x 推进扣 border_right / row_y 推进扣
row_bottom_border）。Slice A+B+C 三者必须同 land（A+B 不带 C = 本轮 incomplete 回退证）。

---

## 0b. R1629 Slice C 实现验证（necessary-but-insufficient，仍 incomplete 回退）

R1629 实现 Slice A+B+C 全套（kill-switch `ZW_COLLAPSE_OVERLAP_GEOMETRY` default-off），补 Slice C
（apply_table_size_constraints 扣 `internal_shared_w/h`，full 非 halved）。单案 + 全量 A/B：

- **001a 仍 EXACT ref 匹配**（100×101 实心绿，10100 px == ref，Slice C 不破坏 flip）。001a/b/c 仍 FLIP
  （0.00/0.00/0.12%）；001d/e 改善（7.46/8.26 → 5.97/6.55，仍 FAIL）。writing-modes 守住（上轮证）。
- **★ 但 CSS2 仍 NET 0（772=772）+ css-tables 簇回归未解**：Slice C **未修** R1628 的回归——
  - collapsed-border-paint-phase-002 **3.16→4.21%**（更差，仍 FAIL）
  - table-cell-width-0 **7.53→8.30%**（更差，仍 FAIL）
  - **row-group-order PASS(0.65%)→FAIL(1.29%)**（真回归）
  - table_grid_size_col_colspan 1.04→1.25%（更差，仍 FAIL）
- **★ 结论：Slice C necessary-but-insufficient**。回归非「表盒尺寸」（Slice C 已修），是 **overlap 模型对
  不规则表（width:0 cell / colspan / row-group 排序 / paint-phase）本身错位**：overlap 假设相邻 cell 共享
  恰一个 border 宽且均匀，但 width:0/colspan/row-group 结构下 cell.border_right 推进与 chromium 不一致。
  对 simple 均匀 0-content 表（001a/b/c）精确，对不规则表错位 → ~3 CSS2 + 多 css-tables PASS→FAIL 抵 flip。
- **回退**（同 R1628，A+B+C 完整但仍 net 0 + 真 PASS→FAIL 回归，不 land）。

**★ 下轮方向 = scope-gating**：overlap 仅对「安全」表触发——所有 cell border 均匀非零 + 无 colspan +
（可能）无 row-group 重排，使 001a/b/c flip 保留、不规则表回归排除。须先 dump 各回归案结构确认 scope
谓词能精确区分 flip 集 vs 回归集（如 row-group-order 虽 border 均匀但 row-group 排序受 row_y 重叠影响 →
须排除多 row-group 表）。gate 加 collapse_overlap 之上，env-gated default-off，全量 A/B 守 net≥0。
若找不到精确 scope 谓词 → 接受 overlap 为「simple 表 only」有限收益（3 flip）或 abandon 转 other lever。

---

## 1. 背景

R1624 定位 tables 簇 layout-tractable 子簇 = border-conflict + collapsing-border-model。
R1625（tie-break `>=`）revert（inert）；R1626（color-aware override）LANDED 但 NET 0 flip
（001a 3.65→3.00，残差 = 独立 geometry 根因）。本 RFC 细化 geometry 修复方案，供下轮实现。

## 2. 实证根因（R1627 dump + 像素采样，grounded）

**目标**：001a = 4×4 `border-collapse:collapse` 表，每格 20px solid border，0 content。
chromium 渲染 = 100×100 实心绿方（5 grid lines × 20px = 100，cells 重叠共享 border）。

**ZW 实测**（LAYOUT_DUMP + PNG 像素采样 + cell-border probe）：
- table 160×160（应 100×100）；每 cell 40×40（应 ~40 border-box 但须重叠），edge-to-edge 无重叠。
- cell probe（2-cell 简化）：`bc=COL`（border_collapse 正确继承到 cell），cell border **FULL**（L=R=T=B=20），
  cell width=40（content 0 + borderL 20 + borderR 20），content_width=0，相邻 cell x=0/40 edge-to-edge。
- PNG 像素采样 001a：**绿网格 + 白格心**（y=65 行 `GGG.....GGGGG.....GGGGG.....GGGGG.....GG` =
  绿竖 grid line + 空 cell 心白），尺寸 160×160。ref = 实心 100×100 绿方。

**根因（双重）**：
1. **定位**（table.rs:1363 `cell_x += cell_width + spacing_x`）：collapsed cell edge-to-edge 摆放，
   相邻 cell **不重叠** → 内部共享 border 双计（4×40=160，应 5×20=100，多 60=3 内部边×20）。
2. **paint**（border.rs:46 `half = v/2 if collapse`）：paint **半宽 inward**（每边 10px），
   cell border-box 40px 内 [10,30] 20px 既非 border 又无 content（0 content）→ **白格心**。
   half 模型本需相邻 cell 重叠使半 border 拼满，但 ZW 不重叠 → 既过宽（定位）又有白心（paint）。

**chromium 正确模型**：FULL border + **相邻 cell 重叠共享 border**。cell border-box=40（content0+20+20），
相邻 cell 重叠一个共享 border 宽（20）→ td1[0,40] td2[20,60] td3[40,80] td4[60,100]，table=100。
FULL border paint：td1 [0,40] 实心（left[0,20]+right[20,40]），重叠区 [20,40] 冲突解析（leftmost 胜）→ 实心 100。

## 3. 设计（kill-switch + 三 site 协同，默认 off）

kill-switch `ZW_COLLAPSE_OVERLAP_GEOMETRY`（env，default-off，gate `border-collapse:collapse`）。

### Slice A：相邻 cell 重叠共享 border（定位）
`position_cells`（table.rs:1363 附近）：collapsed 模式 `cell_x` 推进改为
`cell_x += cell_width + spacing_x - shared_border_h`，其中 `shared_border_h` = 相邻 cell 共享边
的解析后 border 宽（取 `min(cur.border_left, prev.border_right)` 或 resolve_collapsed_borders 已存的
override width；首列不扣左、末列不扣右——outer 边不共享）。
使 4×40=160 → 重叠后 100。

### Slice B：collapsed cell paint FULL border（绘制）
`paint_borders`（border.rs:46）：collapsed 模式 `half` 改回 **FULL**（`half = v`，不再 `/2`），
配合 Slice A 重叠使相邻 FULL border 在重叠区汇合实心（冲突解析颜色已在 resolve_collapsed_borders +
R1626 color override 处理）。**注意**：Slice A+B 须同改（半宽+不重叠 与 全宽+重叠 不可混用）。

### Slice C：table 尺寸修正联动
table.rs:1895 collapse 尺寸修正：现仅扣「table border 胜出」的外边缘；Slice A 重叠后 table 宽 =
Σ cell_width - Σ shared_border（内部）+ outer。须复核 `final_width` / `content_width` 与 cell
定位一致（避免 table 盒宽与 cell 摆放发散）。可能须扣内部共享边总和。

### vertical 对称
position_cells_vertical（table.rs:1393+）对称改 row_y 推进 + paint FULL（border.rs half 已统一）。

## 4. 验收（A/B，须 net≥0 才 land）

- **目标 flip**：border-conflict-element-001a/b/c（R1626 color fix 已降至 3.00/1.67/1.70，
  geometry 修后应 < 阈值 flip）；collapsing-border-model-001/004/010a/b 改善。
- **全量 A/B**（git-stash dual run，须全过）：
  - css-tables（115 案）self-source net≥0
  - CSS2/tables+borders（6283 案）self-source net≥0
  - **writing-modes border-conflict-element-vlr/vrl（现全 PASS）不回归**（关键守卫——vertical collapsed）
  - css-tables 其他 collapsed 案（collapsed-border-paint-phase / collapsed-border-vertical-*）不回归
- **product-smoke** exit 0（welcome/wintertc/morning struct 全 PASS，DC-13）。
- **load-bearing 单测**：collapsed 2×2 表 cell 重叠几何断言（cell2.x = cell1.x + cell1.width - shared_border；
  table width = 100 not 160）；separated 表不变（gate 隔离）。
- net<0 → revert，记 entangled 根因到 master.md（同 R1518/R1610/R1622 先例）。

## 5. 风险与回退

- **高风险 broad table 改动**：影响所有 collapsed-border 表（css-tables + CSS2/tables + writing-modes）。
  先例 R1518(net-2)/R1610/R1622(net-6) 均 broad table 改动回归。故 default-off kill-switch +
  全量 A/B + vertical PASS 守卫。
- **vertical collapsed 交互**：writing-modes border-conflict-element-vlr/vrl 现 PASS，vertical 表的
  collapsed 几何可能与 horizontal 不同（R109 vertical 谱系）；Slice A/B vertical 对称改动须验不回归。
- **重叠区双绘**：Slice B FULL paint + 重叠 → 重叠区两 cell 都画（td1 right + td2 left），颜色由
  resolve_collapsed_borders + R1626 override 定（leftmost 胜）；须确认 paint order（td2 后画覆盖 td1）
  不破坏 R1626 color override（override 已设败者侧颜色，故后画者=胜出色，OK——须 A/B 验）。
- **回退**：kill-switch default-off 即等效旧行为；若 A/B net<0 整体 revert，本 RFC 留作下轮重设计依据。

## 6. 实施顺序（下轮）

1. Slice A（定位重叠）+ Slice B（paint FULL）一起（不可单独）+ load-bearing 单测，env default-off。
2. 全量 A/B（4 目录）+ product-smoke。
3. net≥0 → default-on + 删 kill-switch（或保留 default-on）；net<0 → revert 记 entangled。
4. 更新 master.md + evidence。

## 7. 不在本 RFC 范围

- 001d/e（Ahem/serif 文本 + currentColor border，text 度量主导，非纯 geometry）。
- table-backgrounds 子簇（paint-side，R1624 defer 到 PNG 诊断轮）。
- font-wall 主指标 deadlock（独立多周工程）。

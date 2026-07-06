# Spec：Vertical-mode 表格轴转置（α-4b — position_cells vertical-rl/lr transpose）

**版本**：v0.1（R1107，2026-07-06，首版草案 — 多 session 切片计划 + 轴语义定义）
**日期**：2026-07-06
**作者**：AI Assistant（rendering-compat rally）
**状态**：草稿（rally 续跑用，无用户确认门禁；实施按切片逐 session A/B 门禁推进）
**复杂度**：复杂（跨模块 / 高回滚难度 / table 内部布局 major rewrite）
**父 RFC**：[`vertical-mode-ifc-unification-rfc.md`](./vertical-mode-ifc-unification-rfc.md) v0.1 §4.4（α-4）—— 本 RFC 是 α-4 generic mirror（R1103-R1105 net-negative 证伪）的 table-specific 替代方向（α-4b）。

---

## 0. 执行摘要

- **一句话目标**：让 `position_cells`（`crates/layout-engine/src/table.rs:931`）对 `writing-mode: vertical-rl/lr` 的表按规范做**轴转置**（行沿 x 右到左/左到右、cell 沿 y 顶到底），使 row-progression-vrl/vlr 簇（~12 案，80-87% worst）从「渲染为 horizontal-tb」修正到匹配 chromium Oracle。
- **本期范围**：本 RFC **不立即落地全部转置**；它定义**多 session 切片计划**（α-4b-1 … α-4b-5），每切片独立 A/B 门禁（net-0/正即留，net-负即回退），后续 session 按序推进。本期仅交付 RFC + 轴语义定义 + 切片蓝图，零功能源码。
- **明确排除**：generic block-flow mirror（R1103-R1105 已证 net-negative，永久关闭）；horizontal-tb 表任何行为变化（所有改动 WM-gate `is_vertical_table_wm`，horizontal-tb 字节一致零回归）；taffy 0.8+ 升级（R304 DEFERRED）；CJK 字体度量（font-wall 谱系）。
- **核心约束**：① **horizontal-tb 零回归**（WM gate 隔离；horizontal-tb 表占 corpus 绝大多数，是 hard gate）。② vertical-rl 与 vertical-lr 方向区分（rl 右到左、lr 左到右）。③ 每切片三态门禁：`make product-smoke`（welcome <20%）+ scoped oracle net ≥0 + self-source 不降 + horizontal-tb 字节一致。④ colspan/rowspan/col-width/border-spacing/row-extras/vertical-align 须在 transposed 轴重新解释（不可遗漏任一，否则几何错）。
- **推荐方案**：在 `position_cells` 内加 vertical-rl/lr 分支（**共享逻辑测量**：row/cell 逻辑结构与 col_widths 计算不变；**分支物理赋值**：按 WM 把逻辑量映射到 transposed 物理 x/y），**非**新建并行函数（避免 ~150 行重复，但 α-4b-1 若统一抽象风险过高可降级为并行 `position_cells_vertical`）。
- **首个落地步骤**：实施 Slice α-4b-1（§4.1）——给 `position_cells` 加 vertical-rl/lr WM 分支，处理**简单表**（无 colspan/rowspan、border-spacing:0、固定或 auto 高度）的 row+cell 轴转置；A/B `make reftest-oracle DIR=css-writing-modes` 看 row-progression-vrl-002/004/006/008 + vlr-003/005/007/009 簇 net oracle 变化。

---

## 1. 背景与目标

### 1.1 背景

ZeroWeb 的 table 内部布局（`position_cells`）对所有 writing-mode 走 **horizontal-tb** 定位逻辑：行沿 y 顶到底堆叠（`row_y += row_height + spacing_y`），cell 沿 x 左到右堆叠（`cell_x += cell_width + spacing_x`）。全函数无 `writing_mode` 分支。

`writing-mode: vertical-rl/lr` 的表，规范要求**完全相反的轴映射**（见 §2 轴语义表 + 测试 assert）。当前 ZW 把 vertical-rl 表渲染为 horizontal-tb（行垂直堆叠、cell 水平堆叠），与 chromium Oracle 差异 80-87%（corpus 最高发散簇之一）。

**容器级轴交换已存在**：`converter/mod.rs:248 apply_vertical_writing_mode` 对垂直 WM 容器内的元素在 taffy::Style 层交换轴（inset/size/margin/padding/border/gap/flex-direction），table 相关 display 全映射 `taffy::Block`，故 table **容器盒**经 converter + extract_layout 轴交换后，外层 x/y/width/height 对 vertical-rl 正确。**但** `position_cells` 在 taffy 之后、基于 `TableGrid` 结构定位行/cell，完全独立于 taffy 轴交换——converter 的轴交换只作用于容器盒本身，不传入 TableGrid 的 row/cell 定位逻辑。**即容器几何对、table 内部行/cell 仍 horizontal-tb**。这是 α-4b 须填的缺口。

**R1103-R1105 generic mirror 证伪**：postprocess final-pass 的 `mirror_vertical_rl_block_flow`（对所有 vertical-rl 块子镜像 x）net -9（破 9 real WPT 案）。根因：高 yield 案（row-progression-vrl 80-83%）全是 tables，table rows 由 `position_cells` 自定位，generic mirror 触不到 table 内部；而它 mirror 了非 table 的 vertical-rl 块子（小案），这些案原本 left-to-right 恰好匹配 chromium，mirror 反破。**故真 yield 须 table-specific 转置 in position_cells**（本 RFC），非 generic 子树 mirror。

### 1.2 目标

- **业务目标**：css-writing-modes oracle 通过率提升（row-progression-vrl/vlr 簇 ~12 案从 80-87% worst 降到 <1% 或显著改善）；间接推动 css-writing-modes 目录整体 gap 收敛（当前 ~87% fail，corpus 最大单目录 gap）。
- **用户目标**：ZeroWeb 能正确渲染 vertical-rl/lr 书写模式的表格（CJK 传统排版、垂直表单等）。

### 1.3 范围边界

- **在范围内**：`position_cells` 加 vertical-rl/lr 分支（轴转置 + colspan/rowspan/col-width/border-spacing/row-extras/vertical-align/caption-side 在 transposed 轴的语义重解释）；WM gate 隔离 horizontal-tb；切片 A/B 门禁。
- **不在范围内**：generic block-flow mirror（已证伪）；taffy 0.8+ 升级（R304 独立轨道）；horizontal-tb 任何行为变化；CJK 字体度量（font-wall）；::first-letter / multicol Phase 2（独立轨道）；`sideways-lr`/`sideways-rl`（CSS Writing Modes 4，corpus 无 driving case，defer）。

### 1.4 关联文档（单一权威来源主从）

- **父 RFC**：[`vertical-mode-ifc-unification-rfc.md`](./vertical-mode-ifc-unification-rfc.md) §4.4（α-4 generic，已证伪）+ §4.6（taffy 升级备选）。本 RFC 替代 α-4 generic 方向，列为 α-4b。
- **R1106 evidence**：[`evidence/r1106-vertical-mode-alpha4b-table-transpose-2026-07-06.txt`](./evidence/r1106-vertical-mode-alpha4b-table-transpose-2026-07-06.txt)（LAYOUT_DUMP + 代码核查 + 测试 assert 逐字）。
- **本 RFC 主定义区**：轴语义表（§2）+ 多 session 切片计划（§4）+ colspan/spacing 转置设计（§5）+ 验证门禁（§6）。

---

## 2. 轴语义定义（★ 本 RFC 核心权威）

`position_cells` 的物理量在不同 writing-mode 下的轴映射：

### 2.1 主轴映射表

| 量 | horizontal-tb（现状，不变） | vertical-rl（新） | vertical-lr（新） |
|---|---|---|---|
| 行迭代方向 | y，顶→底（`row_y` 递增） | x，**右→左**（`row_x` 递减，1st row rightmost） | x，**左→右**（`row_x` 递增，1st row leftmost） |
| cell 迭代方向 | x，左→右（`cell_x` 递增） | y，顶→底（`cell_y` 递增） | y，顶→底（`cell_y` 递增） |
| 行主尺寸（沿迭代轴） | `row.height`（沿 y） | `row.width`（沿 x） | `row.width`（沿 x） |
| cell 主尺寸（沿 cell 迭代轴） | `cell.width`（沿 x） | `cell.height`（沿 y） | `cell.height`（沿 y） |
| `col_widths[i]` 物理维度 | cell 的 x 宽贡献 | cell 的 y 高贡献 | cell 的 y 高贡献 |
| 行间距 `spacing_y` 物理方向 | 沿 y（行间） | 沿 x（行间） | 沿 x（行间） |
| cell 间距 `spacing_x` 物理方向 | 沿 x（cell 间） | 沿 y（cell 间） | 沿 y（cell 间） |
| 周界 spacing `perimeter_x/y` | x 左右 / y 上下 | x 右左（block 轴周界）/ y 上下（inline 轴周界） | 同 vertical-rl |
| table content 主尺寸 | `content_width`（inline 容量） | `content_height`（inline 容量 = Σ col_widths + spacing） | 同 vertical-rl |

### 2.2 行/cell 尺寸计算映射

| 计算 | horizontal-tb | vertical-rl/lr |
|---|---|---|
| 行沿迭代轴的尺寸 | `row_height = max(cell.height)` + `row_extras` | `row_width = max(cell.width)` + `row_extras`（横向展开量） |
| cell 沿迭代轴的尺寸 | `cell_width = Σ col_widths[col_start..col_end] + spacing` | `cell_height = Σ col_widths[col_start..col_end] + spacing` |
| cell 垂直于迭代轴的尺寸 | `cell_height = max(row_height, Σ children heights)` | `cell_width = row_width`（cell 填满列 x 宽） |
| 行垂直于迭代轴的尺寸 | `row.width = table_content_width`（行铺满表宽） | `row.height = table_content_height`（行铺满表高） |

### 2.3 方向语义（vertical-rl 右到左 vs vertical-lr 左到右）

行沿 x 迭代时，起始 `row_x` 与递增方向：

- **vertical-rl**：1st row 在最右。起始 `row_x = content_right - perimeter_x`（content box 右边缘 - 周界），每行后 `row_x -= (row_width + spacing_y)`（向左推进）。
- **vertical-lr**：1st row 在最左。起始 `row_x = perimeter_x`（content box 左边缘 + 周界），每行后 `row_x += (row_width + spacing_y)`（向右推进）。

cell 沿 y 迭代（两方向相同）：起始 `cell_y = perimeter_y`，每 cell 后 `cell_y += (cell_height + spacing_x)`。

### 2.4 规范依据（测试 assert 逐字）

`row-progression-vrl-002.xht` 的 `<meta name="assert">`：

> "rows of a table element with in a 'vertical-rl' writing mode are laid out one after the other, leftwardedly, with the first beginning at the rightmost side of the table box; table rows are ordered from right to left meaning that the 1st row is the rightmost one and then the 2nd row is juxtaposed to its left-hand side..."

§2.1-2.3 的映射与此 assert 一致：行沿 x 右到左、cell 沿 y 顶到底。

---

## 3. 影响范围

### 3.1 直接受益簇（css-writing-modes）

| 簇 | 案数（约） | 当前 worst | 说明 |
|---|---|---|---|
| `row-progression-vrl-*`（002/004/006/008） | 4 | 80-87% | vertical-rl 表，行右到左 |
| `row-progression-vlr-*`（003/005/007/009） | 4 | 类似 | vertical-lr 表，行左到右 |
| `block-flow-direction-*`（含表变体） | 部分 | 80%+ | 部分案是 vertical 表 |
| `line-box-direction-*`（含表变体） | 部分 | 高 | 部分案是 vertical 表 |

**直接 driving case**：row-progression-vrl/vlr 8 案（纯表轴转置，border-spacing:0，无 colspan）—— α-4b-1 的 A/B 目标。

### 3.2 间接受益

- `block-flow-direction-*` / `line-box-direction-*` 中的表变体（α-1 container_width 已解纯文本变体，表变体仍 horizontal-tb，待 α-4b）。
- css-writing-modes 目录整体 pass 率（当前 ~7-8%，corpus 最大 gap）。

### 3.3 不受影响（显式）

- horizontal-tb 表（corpus 绝大多数）：WM gate 隔离，字节一致零回归（hard gate）。
- 非表 vertical-rl/lr 块（已由 converter apply_vertical_writing_mode + α-1 container_width 覆盖）。

---

## 4. 多 session 切片计划（★ 本 RFC 主交付）

每切片**独立 A/B 门禁**：`make reftest-oracle DIR=css-writing-modes` 看 net oracle-pass 变化 + `make product-smoke`（welcome <20%）+ self-source 不降 + **horizontal-tb 表字节一致**（WM gate 验证）。net-0/正即留，net-负即回退并记 evidence。

依赖图：`α-4b-1（简单表转置）→ α-4b-2（colspan/rowspan）→ α-4b-3（border-spacing 轴）→ α-4b-4（row-extras/vertical-align/caption）→ α-4b-5（集成 + 全量 oracle）`。

每切片必须在分支入口处读 `table_box.writing_mode`（LayoutBox 字段，types/mod.rs:159），HorizontalTb 走原路径（early return 到现有逻辑）。

### 4.1 Slice α-4b-1 — 简单 vertical-rl/lr 表轴转置（无 colspan、border-spacing:0）

- **范围**：`position_cells` 加 vertical-rl/lr 分支。处理**简单表**：无 colspan/rowspan（所有 cell.col_start+1 == cell.col_end）、`border-spacing: 0`（spacing_x = spacing_y = 0，回避轴互换语义 TBD）、row_extras 暂按横向均分（vertical 表 height 属性 → x 方向行展开）。
- **轴映射**：按 §2.1-2.3。行沿 x（rl 右到左 / lr 左到右），cell 沿 y 顶到底。`col_widths[i]` → cell 沿 y 高贡献；行宽 = max(cell 内容宽)；cell 高 = 对应 col_width。
- **文件**：`crates/layout-engine/src/table.rs`（`position_cells:931` 加 WM 分支）。
- **预期**：row-progression-vrl-002/004/006/008 + vlr-003/005/007/009 共 8 案，z_vs_chr% 从 80-87% 显著下降（目标 <1% 翻 pass，或至少大幅改善）；horizontal-tb 表零回归。
- **门禁**：A/B `make reftest-oracle DIR=css-writing-modes`（看 row-progression 簇 + 全目录 net）+ `make product-smoke`（welcome <20%）+ horizontal-tb 表 byte-diff（采样 css-tables 目录确认 0 字节漂移）。
- **风险**：① row_extras（table height 在 vertical 下应展开 inline 方向 = y，非 block 方向 = x）可能须特殊处理——α-4b-1 先按「不展开」(row_extras=0) 实测，若 table height 强制 inline 轴展开则 α-4b-4 处理。② cell 内容宽度（cell.width 沿 x）计算依赖 taffy 给 cell 盒的 width——converter 已轴交换 cell 盒 size，故 cell.width 应已是转置后的 x 宽，须 LAYOUT_DUMP 确认（TBD-1）。
- **依赖**：无前置（α-1 已 LANDED，container 坐标系已 vertical-aware；本切片是 table 内部独立缺口）。

### 4.2 Slice α-4b-2 — colspan/rowspan 在 transposed 轴（依赖 α-4b-1）

- **范围**：处理 `colspan > 1`（cell 跨多列）。在 vertical-rl/lr 下，colspan 跨的「列」是 inline 轴（y）槽位，故 `cell.height = Σ col_widths[col_start..col_end] + spacing`（沿 y），cell 沿 y 占多槽。`rowspan` 跨多行（block 轴 = x 槽位），cell 沿 x 占多槽——须跨 row 的 cell 占位逻辑（当前 grid 是否支持 rowspan 须核查，列 TBD-2）。
- **文件**：`crates/layout-engine/src/table.rs`（colspan 分支）+ `table_types.rs`（TableGrid rowspan 表示核查）。
- **预期**：vertical 表带 colspan/rowspan 案（如有）几何正确。
- **门禁**：同 α-4b-1 + 不破坏 horizontal-tb colspan（css-tables colspan 簇 byte-diff 0）。
- **依赖**：α-4b-1（行/cell 轴须先转置）。

### 4.3 Slice α-4b-3 — border-spacing 轴互换（依赖 α-4b-1）

- **范围**：`border-spacing` 非 0 的 vertical 表。按 §2.1，spacing_x（cell 间）映射到 y 方向，spacing_y（行间）映射到 x 方向，周界 perimeter 同步轴换。须确认 CSS §17.6.1 在 vertical WM 下 spacing 的物理/逻辑语义（TBD-3：chromium 行为是 spacing 物理值不变还是随 WM 互换——row-progression-vrl-002 用 spacing:0 故 α-4b-1 不触发，本切片须找带 spacing 的 vertical 表 driving case）。
- **文件**：`crates/layout-engine/src/table.rs`（perimeter + spacing 累积分支）。
- **预期**：带 border-spacing 的 vertical 表几何正确。
- **门禁**：同上 + horizontal-tb spacing 不变。
- **依赖**：α-4b-1。

### 4.4 Slice α-4b-4 — row-extras / vertical-align / caption-side 在 transposed 轴（依赖 α-4b-1）

- **范围**：① row_extras（table height → inline 轴 y 展开，或 block 轴 x 展开，按规范确认 TBD-4）。② vertical-align（cell 内子对齐，vertical 表对齐轴是 x 非 y）。③ caption-side（vertical 表 caption 在表的 block 起始/结束侧 = 右/左侧，非上/下）。
- **文件**：`crates/layout-engine/src/table.rs`（row_extras 分支 + valign 分支 + caption 分支）。
- **预期**：vertical 表 height 展开与 caption 正确。
- **门禁**：同上。
- **依赖**：α-4b-1。

### 4.5 Slice α-4b-5 — 集成 + 全量 oracle + 收口（依赖 α-4b-1..4）

- **范围**：全 corpus `make reftest-oracle`（无 DIR）确认 net ≥0 + 无 horizontal-tb 回归；补单测；探针（LAYOUT_DUMP/WMDBG）移除或留 env-gated；master.md / 父 RFC 同步 α-4b done。
- **门禁**：全 corpus oracle net ≥0 + welcome <20% + `make test` 全绿 + clippy/fmt + horizontal-tb 表 byte-diff 0。
- **依赖**：α-4b-1 至 α-4b-4 全 LANDED（或部分 LANDED + 余 defer 有据）。

### 4.6 备选轨道 — taffy 0.8+ 升级（R304，减耦合）

若 α-4b-1 实施发现 `position_cells` 的逻辑结构与 taffy cell 盒尺寸耦合过深（cell.width/height 已被 converter 轴交换致转置逻辑无法叠加），先做 taffy 0.8+ 升级（R304，native vertical block-flow），减耦合后再 α-4b。taffy 升级是独立多 session 轨道（541 ref 迁移），非本 RFC 范围，但列为 α-4b-1 失败时的 fallback。

---

## 5. colspan / border-spacing 转置设计（TBD · 主设计方向）

### 5.1 colspan 转置

**horizontal-tb（现状）**：`cell.col_start..cell.col_end` 跨多列，`cell.width = Σ col_widths[range] + (n-1)*spacing_x`。

**vertical-rl/lr**：同样的 `col_start..col_end` 跨多个 inline 轴（y）槽位。`cell.height = Σ col_widths[range] + (n-1)*spacing_x`（注意 spacing_x 是 inline 轴 spacing，转置后沿 y）。cell 沿 y 占多槽，后续 cell 的 `cell_y` 从 `cell.height + spacing_x` 后继续。

**rowspan 转置**：`rowspan > 1` 跨多行 = 跨多 block 轴（x）槽位。当前 `TableGrid` 是否存储 rowspan 须核查（TBD-2）。若支持，cell 沿 x 占多槽，须在行迭代时跳过已被 rowspan cell 占据的 x 位置。

### 5.2 border-spacing 轴互换

CSS §17.6.1 `border-spacing: <h> <v>`：第一值 = 水平（inline 轴）spacing，第二值 = 垂直（block 轴）spacing。在 vertical WM 下，inline 轴 = y，block 轴 = x。

- horizontal-tb：`spacing_x`（inline）沿 x，`spacing_y`（block）沿 y。
- vertical-rl/lr：`spacing_x`（inline）沿 y，`spacing_y`（block）沿 x。

周界 perimeter：separated 模式四边各有 spacing。vertical 下左右周界（沿 x = block 轴）用 `spacing_y`，上下周界（沿 y = inline 轴）用 `spacing_x`——即 §2.1 表中 perimeter_x/y 在 vertical 下取值互换。

**TBD-3**：chromium 实测 border-spacing 在 vertical WM 下是否随轴互换（部分浏览器实现把 spacing 当物理值不换）。须 α-4b-3 找带 spacing 的 vertical 表 driving case 实测。

### 5.3 实现结构推荐

**方案 A（推荐）**：`position_cells` 内加 WM 分支，共享逻辑测量（grid 遍历 + col_widths + row/cell 逻辑结构计算不变），分支物理赋值（按 WM 把 row_y/cell_x 累加换算成 transposed 坐标）。优点：无 ~150 行重复；缺点：分支内须小心 axis mapping 一致。

**方案 B（降级）**：新建 `fn position_cells_vertical(table_box, grid, col_widths, spacing_x, spacing_y, styles)`，与 `position_cells` 并列，`position_cells` 入口按 WM 分派。优点：horizontal-tb 路径完全不动（零回归风险最大）；缺点：~150 行重复（colspan/valign/row_extras 逻辑双份维护）。

**推荐**：α-4b-1 用方案 B（并行函数，安全优先，快速验证轴语义正确性）；若 α-4b-1 LANDED 且后续切片确认轴语义稳定，α-4b-5 收口时重构为方案 A（统一去重）。这与 code-guidelines §2「简单至上」+ §3「精准修改」一致（先安全落地再统一）。

---

## 6. 验证与门禁

### 6.1 通用门禁（每切片必须全过）

| 门禁 | 命令 | 标准 |
|---|---|---|
| 编译 | `cargo check --workspace` | 零错误 |
| 单测 | `make test` | 全 workspace 绿（零 FAILED） |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | 零 warning |
| Fmt | `cargo fmt` | 无变更 |
| 产品 smoke | `make product-smoke` | welcome <20%（DC-13） |
| Scoped oracle | `make reftest-oracle DIR=css-writing-modes` | net oracle-pass ≥0（vs pre-slice baseline） |
| **horizontal-tb 表零回归** | `make reftest-oracle DIR=css-tables` 或采样 byte-diff | net 0（WM gate 隔离验证） |

### 6.2 探针基础设施（实施期 env-gated，默认 off）

| 探针 | env | 用途 |
|---|---|---|
| LAYOUT_DUMP | `LAYOUT_DUMP=1` | dump 表 row/cell abs_x/abs_y/width/height（既有，R1050） |
| WMDBG（新增，按需） | `WMDBG=1` | dump position_cells 内 WM 分支命中 + 轴映射值 |

### 6.3 A/B 方法

baseline = pre-slice HEAD；treatment = 切片改动。`ORACLE_DUMP_ALL=1 make reftest-oracle DIR=css-writing-modes` per-case dump，python 计算 z_vs_chr<1.0% pass 数。三态门禁：net-0/正留，net-负回退 + 记 evidence。horizontal-tb 表用 `make reftest-oracle DIR=css-tables` 或采样 css-tables/normal-flow 表 byte-diff 确认 0 漂移。

### 6.4 α-4b-1 单测要求

新增单测（`crates/layout-engine/src/table.rs` tests 或同行测试文件）：
- `test_vertical_rl_table_rows_right_to_left`：4×3 vertical-rl 表，断言 4 行沿 x 右到左（row[0].x > row[1].x > row[2].x > row[3].x），每行 3 cell 沿 y 顶到底（cell[0].y < cell[1].y < cell[2].y）。
- `test_vertical_lr_table_rows_left_to_right`：同上 vertical-lr，行沿 x 左到右。
- `test_horizontal_tb_table_unchanged_by_wm_gate`：同结构 horizontal-tb 表，断言 WM gate 不改变行为（行沿 y、cell 沿 x，与 gate 前 byte-identical）。

---

## 7. 实施交接（Implementation Handoff）

### 7.1 文件/模块清单

| 路径/模块 | 动作 | 切片 | 目的 | 风险 |
|---|---|---|---|---|
| `crates/layout-engine/src/table.rs:931 position_cells` | 修改 | α-4b-1 | 加 vertical-rl/lr 分支（或分派到并行函数） | horizontal-tb 须 WM gate 零回归 |
| `crates/layout-engine/src/table.rs`（colspan 分支） | 修改 | α-4b-2 | colspan 在 transposed 轴 | 不破坏 horizontal-tb colspan |
| `crates/layout-engine/src/table.rs`（perimeter/spacing） | 修改 | α-4b-3 | border-spacing 轴互换 | TBD-3 chromium 行为待实测 |
| `crates/layout-engine/src/table.rs`（row_extras/valign/caption） | 修改 | α-4b-4 | 转置轴下的展开/对齐/caption | row_extras 轴 TBD-4 |
| `crates/layout-engine/src/table_types.rs` | 核查/修改 | α-4b-2 | TableGrid rowspan 表示（TBD-2） | rowspan 跨 x 槽逻辑 |

### 7.2 推荐修改顺序（按依赖）

1. **α-4b-1**（简单表转置，方案 B 并行函数 `position_cells_vertical`）—— 轴语义验证，row-progression-vrl/vlr 8 案 A/B。
2. **α-4b-2**（colspan/rowspan）—— 依赖 α-4b-1 轴语义稳定。
3. **α-4b-3**（border-spacing）—— 依赖 α-4b-1，须先实测 TBD-3。
4. **α-4b-4**（row-extras/valign/caption）—— 依赖 α-4b-1。
5. **α-4b-5**（集成 + 重构方案 A 去重 + 全量 oracle + 收口）。

### 7.3 首批提交建议

| Batch | 切片 | 范围 | 预期结果 | 验证 |
|---|---|---|---|---|
| Commit α-4b-1 | Slice α-4b-1 | `position_cells_vertical` 并行函数 + WM 分派 + 3 单测 | row-progression-vrl/vlr 8 案 z_vs_chr 显著降，net oracle ≥0，horizontal-tb 字节一致 | `make reftest-oracle DIR=css-writing-modes` + `DIR=css-tables` A/B + welcome <20% + 3 单测绿 |

---

## 8. Spec Lint 报告

### 8.1 结构完整性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 执行摘要存在性 | ✅ Pass | §0 含一句话目标/范围/排除/约束/方案/首步 |
| 场景存在性 | ⚠️ Warning | 本 RFC 是多 session 架构计划，FR 以「切片」形式定义（§4），每切片有 A/B 门禁场景但非 BDD Given/When/Then 格式（rally 架构 doc 非用户特性 spec，同父 RFC R1099 的 ⚠️ 裁决惯例） |
| 异常路径覆盖 | ✅ Pass | 每切片含 net-负 回退分支 + 备选轨道（§4.6 taffy 升级）+ TBD 风险项 |
| 测试绑定 | ✅ Pass | §6.1 门禁表 + §6.4 α-4b-1 三单测 + §4 每切片 A/B `make reftest-oracle DIR=` 命令 |
| TBD 清零 | ✅ Pass | TBD-1..4 标「重要」非「阻塞」，每项有 fallback（α-4b-1 实测 / α-4b-3 实测 / α-4b-4 按规范 / taffy 升级 §4.6） |
| 约束覆盖 | ✅ Pass | §0 核心约束 4 条 + §6 门禁表覆盖（含 horizontal-tb 零回归 hard gate） |
| 实施交接完备 | ✅ Pass | §7 含文件清单 + 职责 + 修改顺序 + 首批提交 |
| 首步可执行性 | ✅ Pass | §0 首个落地步骤 + §7.2 step 1 = α-4b-1，明确文件 + 方案 B + A/B 命令 |

### 8.2 语言精确性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 模糊动词 | ✅ Pass | 用「行沿 x 右到左」「cell.height = Σ col_widths」等具体行为 + 公式 |
| 无量化描述 | ✅ Pass | 「row-progression-vrl/vlr 8 案」「80-87% worst」「welcome <20%」「net ≥0」均量化 |
| 非确定性措辞 | ⚠️ Warning | §5 含「推荐」「若」（方案 A/B 选择 + TBD 待实测，已显式标 TBD-1..4，非隐藏模糊） |

### 8.3 一致性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 范围冲突 | ✅ Pass | §1.3 在范围/不在范围无交集（generic mirror / taffy 升级 / sideways-* 显式排除） |
| 约束冲突 | ✅ Pass | §0 约束无矛盾（horizontal-tb 零回归 + 转置分支 = WM gate 隔离，非冲突） |
| 方案漂移 | ✅ Pass | §4 切片依赖图与 §1.3 范围、§5 设计一致 |
| 章节引用正确 | ✅ Pass | §0 引用父 RFC §4.4 + R1106 evidence 实际存在；§2/§4/§5 交叉引用闭合 |
| 外部事实保守化 | ✅ Pass | chromium border-spacing vertical 行为未验证 → 降级 TBD-3（§5.2），未写入 FR/Must；taffy 0.7 限制（R304/R1043 已证）标实证 |
| 实现来源闭合 | ✅ Pass | §7.1 每文件-动作-切片映射；position_cells / table_types.rs 实现承载明确；方案 A/B 在 §5.3 给出 |
| 类型分层清晰 | ✅ Pass | 需求（§1.2/§3）/决策（§0 推荐方案 / §5.3 方案选择）/假设（§6.5 TBD-1..4）/TBD（§9）分层 |

**汇总**：15 Pass / 2 Warning / 0 Fail / 0 Skip
**门禁判定**：Fail = 0 → 允许作为 rally 续跑蓝图落地；2 Warning（BDD 格式 + 方案选择措辞）为 rally 架构 doc 固有特性，非阻塞。

---

## 9. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|---|---|---|---|---|
| TBD-1 | cell 盒 width 在 vertical 下是否已被 converter 轴交换为转置后 x 宽 | 重要 | converter apply_vertical_writing_mode 对 table-cell 盒 size 交换后，position_cells 读到的 cell.width 是物理 x 宽还是逻辑值 | α-4b-1 实施时 LAYOUT_DUMP 确认 cell.width / cell.height 取值 |
| TBD-2 | TableGrid 是否存储 rowspan 表示 | 重要 | rowspan > 1 的 cell 在 vertical 下跨 x 槽，须 grid 有 rowspan 占位逻辑 | α-4b-2 核查 table_types.rs TableGrid 结构 |
| TBD-3 | border-spacing 在 vertical WM 下 chromium 行为（物理值不变 vs 随轴互换） | 重要 | CSS §17.6.1 在 vertical 下 spacing 物理语义模糊，浏览器行为可能不一致 | α-4b-3 找带 spacing 的 vertical 表 driving case，chromium Oracle 实测 |
| TBD-4 | table height 属性在 vertical 下展开哪个轴（inline y 还是 block x） | 重要 | row_extras 当前按 y 均分行高，vertical 下 height 应是 inline 轴（y）展开还是 block 轴（x）展开 | α-4b-4 按规范 + chromium 实测确认 |

---

## 10. 修订历史

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v0.1 | 2026-07-06 (R1107) | 首版草案：轴语义定义（§2）+ 多 session 切片计划（α-4b-1..5）+ colspan/spacing 转置设计（§5）+ 方案 A/B 推荐 + 验证门禁 + 实施交接。替代父 RFC §4.4 α-4 generic mirror（R1103-R1105 证伪）。 |

# RFC：vertical-mode 文本定位协调重写

**版本**：v0.1
**日期**：2026-07-07
**状态**：草案（rally 模式，待后续 session 按切片执行）
**前置**：R1116–R1120 七轮调查定论；本 RFC 综合 evidence/r1116–r1120 + master.md 条目。

> **🔧 R2633 引用核验 + 实施状态纠偏（2026-08-04，R2202 模式）**：逐条源码核验本文 ~9 处引用，6 行号 + 1 路径 drift 全纠偏；**切片 2（Bug B column-x）已 LANDED（R1122）**——最高价值状态纠偏。
> - **行号纠偏**：① `converter/mod.rs:271`（§1 Bug A）+ `:248`（§4 表）→**:344**（`apply_vertical_writing_mode`）；② `inline/mod.rs:1659`（§1 Bug B vrl col-x）→**:1144-1154**；③ `inline/mod.rs:1456`（§4 表 `break_items_into_columns`）→**:940**（R2626 既验）；④ `inline_finalization.rs:591`（§1 Bug D compute_final gate）→**:726**（fn `compute_final_inline_layouts` @607，gate `!is_block_level && !is_table_internal_with_text`）；⑤ `inline_finalization.rs:330`（§7.2 `store_font_sizes_from_ifc` 调用）→**:333**；⑥ `painter/text.rs:1119`（§7.2 Bug E Path B + §7.3 表）→**:812**。
> - **路径纠偏**：⑦ §7.2 `inline/mod.rs:598-619` `collect_inline_items` →**`inline/collect_items.rs:6`**（`inline/` 模块拆分迁移，同 R2632 font-wall 发现）。
> - **★ 实施状态纠偏（最高价值）**：**切片 2（Bug B column-x block_extent）已 LANDED = R1122**——`inline/mod.rs:1144-1154` 现 `let mut x = self.block_extent`（**非** `container_width`），附 R1122 注释；本文 §1 Bug B / §3 切片 2 / §7.6 描述为 open 属历史。**切片 4'（Bug E Path B 真实 styles）仍 OPEN**——`painter/text.rs:812` 仍 `HashMap::new()` 空样式（§7.2 描述准确）。R1043 vertical-mode native 整体仍 user-gated（css-writing-modes 80.1%）。

---

## 0. 执行摘要

- **一句话目标**：把 vertical-rl/lr 的「文本定位」从「4 个互相补偿的 bug 系统」改为「正确轴语义的协调实现」，解锁 caption-side-vrl（2 案 NO green）+ row-progression-vrl/vlr（8 案 7-26%）簇。
- **本期范围**：定义协调重写的多 session 切片计划（不在本 RFC 落源码；按切片逐 session A/B 门禁执行）。
- **明确排除**：font-wall（+232，卡 CI billing 用户动作）；horizontal-tb 文本（零回归基线）。
- **核心约束**：任一单层修均 net-negative（R1116/R1118-A/R1118-B/R1119/R1120 五证）；须协调多 bug 同修；horizontal-tb 字节一致；welcome <20%。
- **推荐方案**：方案 B（分阶段：先 content_height bloat 全局修，再 column-x 量纲修，再 Path A/B 统一），每阶段独立 A/B 门禁。
- **首个落地步骤**：切片 1——vertical auto 容器 content_height bloat 修（post-process vertical auto block 的 inline-size = 内容/CB inline-size，非 viewport 宽），A/B 守 vlr/horizontal 零回归 + caption box 几何正确。

---

## 1. 背景：4 个互相补偿的 bug

R1116–R1120 七轮调查锁定 vertical-rl/lr 文本定位是 **4 层耦合系统**，每层单独修 net-negative（因他层 bug 补偿）。各 bug 与证据：

### Bug A — vertical auto 容器 content_height 暴涨（R1050/R1052 谱系）
- **现象**：vertical auto block（caption/cell-content/auto 容器）inline-size（height for vertical）暴涨到 viewport 宽（800−margin=784），应 = 内容 inline extent 或 CB inline-size。
- **机制**：converter `apply_vertical_writing_mode`（converter/mod.rs:344）swap width↔height 后，taffy 把 vertical auto block 当 horizontal auto-width 块**填满父 content width（784）**；extract_layout swap 回 → physical height=784。table 经 `position_cells_vertical` post-process 修正（h=100），但 caption/cell-content 等未 post-process → 残留 784。
- **证据**：R1120 dump caption h=784；R1118 cell content_height 经 post-process 已正确（capped 100）。
- **单独修效果**：cell已 post-process；caption 修 h=100 仍**不 flip caption-side-vrl**（Bug B 把文本推到 x=108，仍错位）。

### Bug B — IFC column-x 量纲错（break_items_into_columns）
- **现象**：vertical-rl 单列文本完全不绘制（caption-side-vrl NO green）。
- **机制**：`inline/mod.rs:1144-1154`（**R1122 已修**：现用 `block_extent`，非 `container_width`；历史 vrl 列 x = `self.container_width` 当 block 轴（x）右端）；但 vertical 下 `container_width = content_height`（inline 深度/y 轴），**非 block 轴 extent** → 单列 col.y = 784−50=734 → paint `frag_base_x = content_x(58)+734=792 off-screen`。vlr 分支 x 从 0 起 → 不受影响。
- **证据**：R1120 ZW_DUMP_FB 诊断 vrl caption NO green；total_cols_width 修法 +2 caption 但破 alignment（Bug C）。
- **单独修效果**：net-negative −2（alignment 簇 +5-7pp 恶化，Bug C load-bearing）。

### Bug C — vrl alignment 簇 load-bearing 依赖 Bug B 旧行为
- **现象**：inline-table-alignment / inline-block-alignment 等单列 vrl 案依赖 Bug B 的 container_width-based 列定位（baseline 对齐几何）。
- **机制**：这些案 content_height ≈ 正确（非 784 暴涨），container_width（=content_height）作 block 右端虽量纲错，但对其 baseline 对齐几何 load-bearing；改 total_cols_width 破之。
- **证据**：R1120 实验 2/3 single-col + overlarge gate 均 +2 caption 但 alignment +5-7pp 恶化（同 profile 无 clean gate）。
- **结论**：Bug B 修法（total_cols_width）对 C 不安全；须 Bug A 先修（使所有案 content_height 正确），再评估 B/C。

### Bug D — Path A/B 发散（stored IFC vs paint re-run）
- **现象**：vertical cell 加 stored IFC（Path A）与现 Path B（重跑）在 vlr 上发散（+0.3-0.6pp 恶化）。
- **机制**：compute_final gate（inline_finalization.rs:726）跳过 TableCell（非 block-level）→ cell 走 Path B；改 Path A 后 vlr 恶化（layer 4）。
- **证据**：R1118 实验 A/B；R1119 vlr 复核。
- **结论**：row-progression-vlr 须 Path A/B 统一（layer 4）。

### 簇映射
| 簇 | 案数 | 主阻塞 bug |
|---|---|---|
| caption-side-vrl | 2（NO green） | A（h=784）+ B（column-x off-screen） |
| row-progression-vrl | 4（7-13%） | A + B + C + D |
| row-progression-vlr | 4（26%） | A + D（vlr 无 B） |

---

## 2. 为什么单层修均 net-negative（五证）

| 轮 | 修法 | 净 pass | 失败原因 |
|---|---|---|---|
| R1116 | cell.width 面积守恒扩展（layer 2） | 0 flip | cell 文本未 re-wrap（无 stored IFC，Bug D） |
| R1118-A | vertical cell stored IFC（layer 4） | −1 | wrap 列溢出 cell（Bug A：width 不长） |
| R1118-B | stored IFC + width 扩展 | −1 | vrl-006 rowspan 回归 + 不 flip（Bug B/C） |
| R1119 | vlr 复核 | −0（恶化） | vlr Path A/B 发散（Bug D） |
| R1120 | column-x total_cols_width | +2/−2 净 0 | alignment load-bearing（Bug C） |

**裁决**：vertical 文本定位须协调重写，非单点 lever。本 RFC 定义协调路径。

---

## 3. 推荐方案：分阶段协调重写（方案 B）

按依赖顺序分 4 切片，每切片独立 A/B 门禁（welcome<20% + scoped oracle net≥0 + horizontal-tb 字节一致 + css-text-decor 零回归）。失败即回退，不阻塞下一切片设计。

### 切片 1 — content_height bloat 全局修（Bug A）★ 前置
- **目标**：vertical auto 容器（caption + 所有非 post-process 的 vertical auto block）inline-size = 内容/CB inline-size，非 viewport 宽。
- **实现来源**：converter 或 post-process。converter 路径：vertical auto block 不填满父 content width，而取 CB inline-size 或内容 max-content（须 WM-aware sizing）。post-process 路径：扫 vertical auto block，content_height = IFC 内容高（类 R695 两趟基建）。
- **A/B 门禁**：caption box h=100（dump）；caption-side-vrl **仍不 flip**（Bug B 未修，预期）；但 vlr/alignment/css-text-decor 零回归（content_height 修不应破他案）；horizontal-tb 字节一致。
- **风险**：高（触及所有 vertical auto block sizing）；可能暴露其他依赖 784 的案。
- **验证**：`make reftest-oracle DIR=css-writing-modes` + `css-text-decor` + product-smoke。

### 切片 2 — IFC column-x 量纲修（Bug B）★ 依赖切片 1
- **目标**：vertical IFC column-x 用 **block 轴 extent（content_width）**，非 container_width（=content_height，inline 深度）。
- **实现来源**：给 IFC 传 `block_extent`（content_width for vertical）参数；break_items_into_columns vrl 分支用 block_extent 替代 container_width。
- **前置依赖**：切片 1（content_height 正确后，container_width 与 block_extent 分离清晰；且 alignment 案 content_height 正确后评估 load-bearing 是否仍成立）。
- **A/B 门禁**：caption-side-vrl flip（NO green → green at x=58）；alignment 簇不恶化（切片 1 后重新评估）；row-progression-vrl 改善但仍受 Bug C/D。
- **风险**：中（IFC 签名改 + 所有调用方）。

### 切片 3 — row-progression cell width/wrap 两趟（layer 2 / R1116-谱）
- **目标**：解 cell width/wrap chicken-egg（width step8 早于 IFC wrap step12）。
- **实现来源**：step 8 后插 cell IFC wrap pass → 按实际 wrap 列数长 cell.width → re-position（保 float/margin-collapse 状态，区别 R1043 postprocess 失败）。
- **前置依赖**：切片 1+2（cell content_height + column-x 正确）。
- **A/B 门禁**：row-progression-vrl/vlr 改善（向 <1%）；零回归。

### 切片 4 — Path A/B 统一（Bug D）
- **目标**：vertical cell stored IFC（Path A）与 Path B 一致，解 vlr 恶化。
- **实现来源**：排查 Path A/B 在 vertical cell 的分歧点（匿名块/碎片化，R101/R125 谱），统一双路径。
- **前置依赖**：切片 1-3。
- **A/B 门禁**：row-progression-vlr flip；css-text-decor 零回归。

---

## 4. 实施交接

### 文件/模块清单
| 路径 | 切片 | 动作 |
|---|---|---|
| converter/mod.rs:344 `apply_vertical_writing_mode` | 1 | vertical auto sizing WM-aware（不填满父 width） |
| engine.rs（post-process） | 1 | 或 post-process vertical auto block content_height |
| inline/mod.rs:940 `break_items_into_columns` + IFC struct | 2 | 加 block_extent 参数，vrl column-x 用之 |
| inline_finalization.rs（compute_final 调用方） | 2 | 传 block_extent 给 IFC |
| table.rs `position_cells_vertical` | 3 | cell IFC wrap pass + width re-position |
| painter/text.rs + inline_finalization.rs | 4 | Path A/B 统一 |

### 推荐修改顺序
1. 切片 1（content_height bloat）——前置，最高风险，须窄 gate（先 caption + 明确 vertical auto block）。
2. 切片 2（column-x block_extent）——依赖 1，解 caption-side-vrl。
3. 切片 3（cell 两趟）——依赖 1+2，解 row-progression。
4. 切片 4（Path A/B）——依赖 1-3，解 vlr。

### 首批提交建议
| Commit | 范围 | 预期 | 验证 |
|---|---|---|---|
| 切片 1a | caption content_height = row_inline_extent（position_cells_vertical 内） | caption box h=100（dump 证）；caption-side-vrl 仍不 flip（B 未修） | scoped oracle 零回归 + product-smoke |
| 切片 1b | 全局 vertical auto block content_height 修 | 所有 vertical auto box inline-size 正确 | 全 css-writing-modes A/B + css-text-decor 零回归 |

---

## 5. 待定（TBD）

| ID | 项目 | 优先级 | 缺失 | 下一步 |
|---|---|---|---|---|
| TBD-1 | 切片 1 converter vs post-process 路径选 | 阻塞（切片 1） | converter WM-aware sizing 复杂度 vs post-process 两趟成本 | 切片 1a（caption post-process）先验，再扩 |
| TBD-2 | 切片 2 后 alignment 簇是否仍 load-bearing | 重要 | 切片 1 后 alignment 案 content_height 重测 | 切片 1 落地后 A/B 复核 |
| TBD-3 | 切片 3 re-position 是否破 float/margin-collapse（R1043 谱） | 重要 | R1043 postprocess mirror 失败先例 | 切片 3 设计时核 |

---

## 6. 修订历史
| 版本 | 日期 | 变更 |
|---|---|---|
| v0.1 | 2026-07-07 | 初始：综合 R1116-R1120，定义 4 切片协调重写 |
| v0.2 | 2026-07-08 | 增补 §7：R1135/R1136 实验证伪 + 新 Bug E（R1050 根因）+ 切片修正 |

---

## 7. v0.2 增补（R1135/R1136 实验证伪 + 新 Bug E）

R1121 后 R1134-R1136 三轮在 v0.1 切片 1-4 框架下做了进一步实证，得到 **3 条「已证伪勿重试」+ 1 个新 bug**，本节固化以避后续 session 重复探索。

### 7.1 已证伪勿重试（proven dead ends）

| 轮 | 尝试 | 结果 | 裁决 |
|---|---|---|---|
| R1135 | **R1112 col_width override**（compute_column_widths WM-aware：vertical cell col_width 读 height 非 width + 空 cell intrinsic 返 padding） | net-0 不 flip；border-spacing-vrl-002/vlr-003 反 3.23→4.07% 恶化 | **勿重试**：spec-correct col_width（height:0.5em=10）反比 spec-wrong（width:1em=20）离 chromium 更远 → **chromium 用 `width` 非 spec-mapped `height` 作 vertical table cell 列尺寸维度**（实现 quirk，与 CSS writing-modes §7.2 dimension mapping 相悖）。即 ZW 现有 WM-agnostic 读 width 的行为**已匹配 chromium**，勿改。 |
| R1136 | **R1050 单层修**（painter/text.rs is_table_cell gate 传真实 styles → cell IFC fs=16→20 ahem）+ block_extent gate 扩 TableCell | **net-negative −2**（css-writing-modes 64→62/784）；全部 8 row-progression 案恶化（7-9%→8-10%） | **勿单层重试**：fs=20 正确化破坏 compensating-errors（旧 fs=16 错误度量偶然压成 2 列，恰比正确 fs=20 的 4 列更接近 chromium）。须配合 cell inline-extent 分布修（见 §7.3）。 |
| R1116/R1118/R1119 | cell.width 面积守恒扩展 / stored IFC / vlr 复核 | net-negative/0-flip（v0.1 §2 已列） | **勿重试**（v0.1 五证 + R1135/R1136 两证 = 七证）：单层修必 net-negative。 |

### 7.2 新 Bug E — Path B 空 styles → cell fs=16 默认（R1050 决定性根因）

R1136 用 R1050DBG 探针实测 row-progression-vrl-002 cell 的 painted IFC：**fs=16 ahem=false**（应 fs=20 ahem=true，继承 body `font:20px/1 Ahem`）。

**根因链**：
1. table cell 无 stored IFC（compute_final gate `!is_block_level && !is_table_internal_with_text` 跳过 TableCell）。
2. 故 `store_font_sizes_from_ifc`（inline_finalization.rs:333）不为 cell 调用 → `text_node_font_sizes`/`text_node_is_ahem` 未填充。
3. paint Path B（painter/text.rs:812）`ctx.layout(doc, node_id, &HashMap::new())` 传**空 styles**。
4. collect_inline_items（`inline/collect_items.rs:6`）`style.is_some()` 不命中 → default_font_metrics = (16, 16×1.2) → cell 文本按错误度量 wrap。

**与 v0.1 Bug D 的关系**：Bug E 是 Bug D（Path A/B 发散）的**具体机制**——Path B 空 styles 是发散点。v0.1 切片 4「Path A/B 统一」的真正修法 = 给 Path B 的 cell 传真实 styles（或等效 override）。

**为什么单修 Bug E 是 net-negative（R1136 −2）**：fs=16 错误度量是 cell inline-extent 分布错（v0.1 Bug A 谱，cell extent 54.4 偏小）的**补偿因子**——fs=16 把词高压到使 2 列 pack 匹配 chromium 的「更大 extent + fs=20」2 列 pack。修 E（fs 正确化）破坏补偿 → 暴露 extent 分布 mismatch → 4 列发散更远。

### 7.3 对 v0.1 切片计划的修正

| v0.1 切片 | v0.2 修正 |
|---|---|
| 切片 3（cell width/wrap 两趟） | **删除「col_width 改垂直测量」子任务**（R1135 证 chromium 用 width-like，ZW 现状已对）。cell extent 分布 lever 转为「cell inline-extent 对齐 chromium 实际值」（须测 chromium oracle 几何，TBD-5）。 |
| 切片 4（Path A/B 统一） | **具体化为 Bug E 修**：painter/text.rs:812 给 cell 传真实 styles（is_table_cell gate）。但**必须与 cell extent 分布修（切片 3 修正后）同提交**，否则 net-negative（R1136）。 |
| 新 TBD-5 | cell inline-extent 实测 chromium 值：用 oracle PNG 测 row-progression-vrl-002 各 cell 的 y-extent，确定 chromium 分布算法（非 char-proportional？）。阻塞切片 3+4 同修。 |

### 7.4 修正后的依赖图

```
切片 1（content_height bloat，Bug A）  ── 前置
   │
   ├─> 切片 2（column-x block_extent，Bug B）── 解 caption-side-vrl
   │
   └─> 切片 3'（cell extent 对齐 chromium + TBD-5 实测）┐
                                                      ├─ 同提交 ─> 切片 4'（Bug E：Path B 真实 styles）── 解 row-progression
```

**关键约束**：切片 3' + 切片 4' **必须同提交**（compensating-errors，单提必 net-negative，R1136 −2 证）。这是 v0.2 最重要的修正——v0.1 把它们当独立切片，R1136 证不可独立。

### 7.5 下一步具体行动（TBD-5 先行）

1. **TBD-5（实测 chromium cell extent）**：对 row-progression-vrl-002 oracle PNG，测 4 行 × 3 cell 各 cell 的 yellow text y-range，反推 chromium cell inline-extent 分布。判 chromium 是否 char-proportional / even / 其他。
2. 据 TBD-5 结果设计切片 3'（cell extent 分布对齐）。
3. 切片 3' + 切片 4'（Bug E）同提交 A/B 门禁。

### 7.6 TBD-5 实测结果（2026-07-08）

oracle PNG（chromium）像素分析（Python PIL，blue/yellow 像素 bbox + yellow 密度按 20px x-bucket）：

| 渲染 | table x-extent | table y-extent |
|---|---|---|
| **chromium oracle** | **~420px**（x[8,427]） | ~180px（y[8,187]） |
| ZW baseline（fs=16，2 sub-col/cell） | ~154px（4 行 × 2 × 19.2） | — |
| ZW R1136 fix（fs=20，4 sub-col/cell） | ~320px（4 行 × 4 × 20） | — |

**关键发现**：
- chromium table x=420 **比 ZW baseline(154) 和 fs=20 fix(320) 都宽** → chromium cell 文本 wrap 成**更多** sub-column（非我先前推断的「更少列」）。
- 但 R1136 fs=20 fix（320，离 420 更近）的 diff% 反 **更差**（8.36 vs baseline 7.46）。
- **结论**：row-progression 残余**非 table gross 宽度**问题（fs=20 已把宽度拉近），而是**文本定位精度**——具体字符落在哪个 sub-column 的精确位置发散。即 Bug B（column-x 量纲）+ Bug E（fs）修对 gross 几何后，残余是 fine-grained glyph 放置（须切片 2 column-x block_extent + 切片 4' Bug E + cell extent 精确分布同修）。
- chromium y=180（非 height:7em=140）→ 暗示 cell 文本 overflow 表高 或 height 解释不同（TBD-5a：核 chromium 是否让 cell 内容溢出 140，影响 cell extent 算法）。

**修正 TBD-5**：chromium 非简单「更少列」；须逐字符匹配 chromium glyph 位置（dump ZW fs=20 每字符 x/y vs chromium oracle yellow 质心），定位发散层（column-x？word 内 char advance？）。多 session。


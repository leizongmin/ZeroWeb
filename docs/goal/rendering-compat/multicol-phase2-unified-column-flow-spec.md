# Spec：multicol Phase 2 — 统一 column-flow 碎片化（layout 侧 block+inline 按文档序逐列流动）

**版本**：v1.0（rally 自主模式，假设见 §6.5）
**日期**：2026-06-20
**作者**：AI Assistant（rendering-compat rally）
**状态**：草稿（待多会话实施接力）
**关联**：rendering-compat master.md；[`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md)（v0.4 分析）；[`column-aware-IFC-spec.md`](./column-aware-IFC-spec.md)（v1.0 Phase 1，**A1 gate 已证伪关闭**）；css-multicol 当前 41/57 (71.9%)，16 失败

---

## 0. 执行摘要

> **⚠️ v1.0-R383 重大纠正：混合内容目标案前置依赖 Phase A，本 spec 非独立可实施。** R383 LAYOUT_DUMP 深度诊断 multicol-block-no-clip-002 发现：5 个 `<span>`（inline）经 **R109（inline→block converter）被转成 block-level LayoutBox**，multicol 按原子 block 分配到列（非 IFC 流动）；ref 期望 inline 作单一 IFC 跨列流动（span 跨列分裂）。**根因 = R109 entanglement**，非「inline 未分配」。故本 spec 的统一 column-flow **即使实现也修不了混合内容案**（spans 已是 block 盒）——真修复须 **先 Phase A（inline 内容作流动 IFC / R109 解转换）再 multicol 列碎片化**。**两多会话 lever 依赖：Phase A → multicol**。下方 Phase 2 设计保留作 Phase A 完成后的实施基础；**勿在 Phase A 前以混合内容为目标实现统一流**（会重复 R382 spec 的浪费）。R109-independent 的 multicol 失败（嵌套 breaking，Phase 3，真 block 子）可独立推进。

- **一句话目标**：把 multicol 容器的 **block + inline 子元素按文档序统一逐列流动**（CSS Multicol §6 fragmentation + §8 balance），取代当前「taffy 块堆叠 → `assign_children_to_columns_*` 重分配 block 子 → IFC 单独跑 inline（paint 侧仅 pure-inline 分配）」的三段分离模型——该模型**结构性无法表达 block/inline 交错的列流动**，是 css-multicol 16 失败（混合内容 balance / 嵌套 breaking）的根因。
- **本期范围（Phase 2）**：实现 layout 侧「统一 column-flow」——单层、非嵌套 multicol 容器，`column-fill: balance`（默认）+ `height: auto`，含**混合 block + inline 子元素**（即 paint 侧 `text.rs:569` 门控 `!has_in_flow_children` 为 false 被跳过、当前整块堆叠错渲染的案）。目标案：multicol-block-no-clip-002（1.81%）、multicol-containing-002（3.92%）、multicol-count-computed-003（1.78%）、multicol-collapsing-001（1.68%）等混合内容 balance 案（~6-8 案）。
- **明确排除（本期）**：① **嵌套 multicol / column-fill:auto + 明确高度的 breaking**（multicol-breaking-004/005/006/nobackground-004，outer 把 inner 碎片化）= Phase 3（真嵌套碎片化，循环依赖硬核）；② **inline 内容跨列断裂**（单段长文本跨多列）= Phase 2c（行级 fragmentation，依赖 Phase 2b 的统一流基础设施）；③ Phase 1（pure-inline balance 明确高度）**已 A1 gate 证伪关闭**（0/16 失败案匹配，见 column-aware-IFC-spec.md §10），勿再做。
- **核心约束**：① **零 self-source 回归**（loose 443/490 不退；41 通过 multicol 案 byte-identical 或逐案验证不翻）；② **守 multicol-fill-auto-001 sentinel**（R198/R209 font_size 耦合，0.63% 余量小）；③ **chromium-Oracle z_vs_chr 门禁**（非 self-source——R381 实测 R355/R362 self-source 翻转但 chromium 退步，self-source 非可靠代理）；④ 单 `.rs` ≤2000 行；⑤ 测试用 `make test`/`make reftest`（test-guard 包裹）。
- **推荐方案**：新增 layout 侧 `flow_children_into_columns` 统一列流函数——按文档序遍历 multicol 容器子节点，维护 `(col_idx, col_y)` 游标；block 子放入当前列（超高则 breaking），inline 子累积成 inline-run，遇 block 边界/列满时对该 run 跑 IFC（用当前列剩余高度作预算）并把行盒定到列内；结果（每列 block 片段 + 每列行盒）存 `LayoutBox` 新字段，paint 直接消费。
- **首个落地步骤**：① 先 probe 验证假设 A1（§6.5）：混合内容 balance 案的 diff 是否**全部**来自 inline 未分配（而非 block 子位置错）；② 若是，实现 `flow_children_into_columns` 的 block-only 等价路径（必须 byte-identical 复现现有 `assign_children_to_columns_balanced`+`position_multicol_children`），作为统一流的回归安全网；③ 再加 inline-run 处理，逐案 chromium-Oracle 验证。

---

## 1. 背景与目标

### 1.1 背景

当前 multicol 渲染经三段分离模型（master.md「IFC 之外的其他卡点 #2」+ multicol-fragmentation-design.md §1.1）：

| 阶段 | 位置 | 行为 | 缺陷 |
|------|------|------|------|
| taffy 块布局 | taffy-local | block 子元素正常堆叠 | 不感知列 |
| block 子重分配 | `multicol.rs::assign_children_to_columns_*` + `position_multicol_children` | 把 block 子按高度分配到各列（balance=均高顺序填充；auto+breaking=列高限制+碎片化） | **仅处理 block 子**，不碰 inline |
| inline 内容 | `painter/text.rs:569` 门控 | 仅 `!has_in_flow_children && is_balance_mode && height_auto`（pure-inline balance auto）触发 paint 侧 `target_h=total/col_count` 分配；**混合内容（has_in_flow_children）→ multicol_info=None → inline 整块堆叠错渲染** | paint 侧 5 轮证伪（R157/R198/R203/R317/R122）不可解；layout 侧缺失 |

**根本问题**：CSS multicol 要求 block + inline 子元素**按文档序统一逐列流动**（一段 inline 文本流到列底，下一段从下列顶续；一个 block 子可能落在某列中间，把 inline 文本断在前列底/后列顶）。三段分离模型把 block（taffy+multicol.rs）与 inline（IFC）解耦处理，**结构性无法表达这种交错流动**。

**Phase 1（pure-inline balance 明确高度）已关闭**：column-aware-IFC-spec.md §10 假设 A1 经 R381 gate 验证为 FALSE——css-multicol 全 16 失败案 0/16 匹配「单层+balance+明确高度+纯 inline」结构（每案或有 block 子、或 height:auto、或 column-fill:auto、或 breaking/嵌套）。故真实 lever 在混合内容（Phase 2b）与嵌套 breaking（Phase 3），非 pure-inline。

### 1.2 目标

- **业务目标**：css-multicol 通过率 41/57 (71.9%) → ≥95%（DC-4），需 +13 案；Phase 2 目标混合内容 balance 簇 ~6-8 案。
- **用户目标**：ZeroWeb 渲染混合内容 multicol 与 chromium 一致（DC-14 chromium-Oracle z_vs_chr < 1%）。

### 1.3 范围边界

- **在范围内**：单层 multicol + `column-fill: balance` + `height: auto` + **混合 block+inline 子** 的 layout 侧统一 column-flow；新增 `LayoutBox` 列流存储字段 + layout 填充 + paint 消费。
- **不在范围内**：嵌套 multicol / column-fill:auto+明确高度 breaking（Phase 3）；inline 跨列断裂（Phase 2c，依赖 2b）；pure-inline balance 明确高度（Phase 1 已关闭）；baseline-export（独立卡点 #4）；taffy 升级（R304 DEFER）。

---

## 2. 需求类型概览

| 类型 | 是否适用 | 来源 |
|------|---------|------|
| 业务需求 | 是 | DC-4 css-multicol ≥95% |
| 功能需求 | 是 | §3 |
| 非功能需求 | 是 | §4（零回归 + chromium-Oracle） |
| 接口需求 | 是 | §5（新字段 + paint 契约） |
| 过渡需求 | 是 | env 门控逐步启用（§6.3） |

---

## 3. 功能需求

### FR-001：layout 侧「统一 column-flow」按文档序把 block+inline 子流动到列

- **描述**：当 multicol 容器满足（单层、`column-fill: balance`、height: auto、含混合 block+inline 子）时，layout 阶段必须按 DOM 文档序遍历子节点，维护 `(col_idx ∈ [0, col_count), col_y)` 游标：block 子放入当前列（记录列内 y，超高则触发列前进/breaking）；连续 inline 子累积为 inline-run，遇 block 边界或列满时用「当前列剩余高度 = balance_height − col_y」作预算对该 run 跑 IFC，行盒定位到列内 y，行盒溢出列底则续到下列。产出每列的 `[block 片段位置] + [inline 行盒]`。
- **优先级**：必须（Phase 2b 核心）
- **来源**：CSS Multicol §6 fragmentation + §8 balance；multicol-fragmentation-design.md R201/R317

**验收场景**：

```
场景: 混合内容 balance 正确分布（multicol-block-no-clip-002）
  假设 column-count:3 balance height:auto，含 span(blue)+h4+span(orange)+span(pink)+span(yellow)
  当 ZeroWeb layout+paint 渲染 test
  那么 block(h4) 与各 inline span 按文档序流入 3 列，蓝色 span 在列1顶、h4 黑条在列1、橙色 span 续列1底/溢列间、粉色列2、黄色列3，与 ref 的绝对定位 img swatches 一致
  验证: REFTEST_DUMP + chromium-Oracle z_vs_chr < 1%（DC-14）；self-source loose 通过

场景: inline 内容跨列（multicol-containing-002）
  假设 column-count:4 balance，含长 inline 文本 + abspos img 子
  当 渲染 test
  那么 inline 文本按列宽换行后按 balance 高度顺序填入 4 列，abspos img 相对 multicol 容器定位
  验证: z_vs_chr < 1%；self-source 通过

场景: 现有 pure-inline balance 不回归（multicol-fill-auto-001 sentinel + height:auto+balance pure-inline 案）
  假设 pure-inline balance height:auto multicol（当前 paint 侧 text.rs:854 正确处理）
  当 启用统一列流（MULTICOL_UNIFIED_FLOW=1）
  那么 渲染 byte-identical（统一流对 pure-inline 须复现 paint 侧 total/col_count 分配）
  验证: self-source 443/490 不退；multicol-fill-auto-001 z_vs_chr 不变

场景: 现有 block-only multicol 不回归
  假设 multicol 仅含 block 子（当前 assign_children_to_columns_balanced 正确）
  当 启用统一列流
  那么 block 子列分配 byte-identical（统一流的 block 路径须复现 assign_children_to_columns_balanced）
  验证: 41 通过 multicol 案 byte-identical 或逐案 z_vs_chr 不升
```

### FR-002：统一流结果存 `LayoutBox` 新字段，paint 优先消费

- **描述**：统一流产出的每列行盒存入 `LayoutBox.inline_multicol_columns: Option<Vec<Vec<InlineLayoutLine>>>`（IF-001）；paint 侧 `painter/text.rs` 当该字段 Some 时直接按列渲染（列 x 偏移 + 列内 y + 列裁剪），None 时回退现有 paint 侧分配（pure-inline）或整块堆叠（混合内容，旧行为）。
- **优先级**：必须
- **来源**：column-aware-IFC-spec.md IF-001/IF-002（Phase 1 设计复用，Phase 1 已关闭但字段设计有效）

**验收场景**：

```
场景: paint 消费存储结果
  假设 统一流已填充 inline_multicol_columns
  当 paint 渲染该 multicol 容器
  那么 按列 x 偏移 + 列内行盒 y 渲染，超出列边界的内容按 overflow 裁剪
  验证: 单元测试 test_unified_flow_paint_consumes_stored_columns

场景: None 回退（非目标结构 multicol）
  假设 multicol 不满足统一流条件（嵌套/breaking/明确高度）
  当 paint 渲染
  那么 inline_multicol_columns=None，paint 走旧路径，行为不变
  验证: 现有 multicol 案 byte-identical
```

### FR-003：env 门控渐进启用

- **描述**：统一流默认关闭（`MULTICOL_UNIFIED_FLOW` 未设），全部走旧路径；设为 `1` 时对目标结构（单层 balance auto 混合内容）启用统一流。便于 A/B 测量与回滚。
- **优先级**：必须（多会话安全网）

**验收场景**：

```
场景: 默认关闭零行为变化
  假设 MULTICOL_UNIFIED_FLOW 未设
  当 全量 reftest
  那么 443/490 byte-identical（字段始终 None，paint 全走旧路径）
  验证: make reftest 全量 loose 443/490

场景: 开启后仅目标案变化
  假设 MULTICOL_UNIFIED_FLOW=1
  当 全量 reftest
  那么 仅混合内容 balance 案渲染变化（目标案 z_vs_chr 下降），其余 byte-identical
  验证: 逐案 set-diff + chromium-Oracle
```

---

## 4. 非功能需求

### NFR-001：零 self-source 回归
- **描述**：默认关闭时 443/490 byte-identical；开启时 41 通过 multicol 案不翻 FAIL，全量 loose ≥443/490。
- **测量标准**：`make reftest`（reftest-upstream）loose 通过率 + multicol 子集 41/57 不退。
- **优先级**：必须

### NFR-002：chromium-Oracle 真一致率提升（DC-14）
- **描述**：目标案 `z_vs_chr`（ZeroWeb-test vs chromium-test，cross-validate.py）必须下降至 < 1%（真通过口径）；**仅 self-source 翻转不算达成**（R381 教训：R355/R362 self-source 翻转但 chromium 退步）。
- **测量标准**：`cross-validate.py --dump target/reftest-dump --oracle oracle-shots` 目标案 z_vs_chr。
- **优先级**：必须

### NFR-003：文件行数
- **描述**：新增统一流逻辑优先放 `multicol.rs`（当前 628 行）或新模块 `multicol_unified_flow.rs`；单文件 ≤2000 行。
- **优先级**：必须

### NFR-004：单元测试
- **描述**：统一流的 block 等价路径（复现 assign_children_to_columns_balanced）+ inline-run 列内定位 + 列满续列 各有单测。
- **优先级**：必须

---

## 5. 接口需求

### IF-001：`LayoutBox.inline_multicol_columns` 新字段

- **类型**：数据结构（layout → paint 契约）
- **规格**：`pub inline_multicol_columns: Option<Vec<Vec<InlineLayoutLine>>>`（外层 Vec = 列，内层 = 该列行盒）。复用 `InlineLayoutLine`（types/mod.rs:502）。与 `inline_layout`（单列，Phase A stored-path 消费）**分开字段**避免耦合（同 column-aware-IFC-spec.md §6.3 决策）。
- **错误处理**：None = 未走统一流（paint 回退）。
- **交叉引用**：types/mod.rs:267 `inline_layout` 并行；column-aware-IFC-spec.md IF-001。

### IF-002：`flow_children_into_columns` 函数（layout 侧）

- **类型**：内部 API（`multicol.rs` 或新模块）
- **规格**：`fn flow_children_into_columns(container: &mut LayoutBox, info: &ColumnInfo, styles: &HashMap<NodeId, ComputedStyle>, doc: &Document) -> Option<Vec<Vec<InlineLayoutLine>>>`。按文档序遍历 `container.children`，维护游标，对 inline-run 调 IFC（复用 `InlineFormattingContext` + `compute_final_inline_layouts` 的 IFC 调用模式），返回每列行盒（同时通过 `position_multicol_children` 等价逻辑定位 block 子）。
- **默认动作**：不满足目标结构（嵌套/breaking/明确高度/pure-inline balance auto）→ 返回 None（paint 走旧路径）。
- **交叉引用**：复用 `multicol.rs::assign_children_to_columns_balanced`（block 等价）、`inline/mod.rs::InlineFormattingContext`（IFC）、`inline_finalization.rs`（存储模式）。

### IF-003：env 门控 `MULTICOL_UNIFIED_FLOW`

- **类型**：环境变量
- **规格**：未设/`0` = 关闭（默认）；`1` = 对目标结构启用统一流。
- **默认动作**：关闭。

---

## 6. 约束与假设

### 6.1 必须约束（Must）
- 统一流仅作用于「单层 + balance + height:auto + 混合 block+inline 子」multicol；其余全走旧路径（字段 None）。
- 默认 `MULTICOL_UNIFIED_FLOW` 关闭；开启须守 41 通过 multicol 案不翻 + multicol-fill-auto-001 sentinel。
- 目标案须用 chromium-Oracle z_vs_chr 验证（非仅 self-source）。

### 6.2 禁止约束（Must Not）
- 不放宽 `painter/text.rs:569` 的 `height_auto` 门控（R317 实测 balance 侧 -5 回归）。
- 不改 taffy-local 内部（R304 DEFER 升级）。
- 不引入新 crate 依赖（IFC/列分布全仓内复用）。
- 不为不可能场景写错误处理（统一流仅目标结构）。

### 6.3 已定决策
- 存储用新字段 `inline_multicol_columns`（非复用 `inline_layout`，避免与 Phase A stored-path 耦合）。
- env 门控 `MULTICOL_UNIFIED_FLOW` 渐进启用（多会话安全网）。
- block 等价路径必须 byte-identical 复现 `assign_children_to_columns_balanced`（回归安全网，非可选）。

### 6.4 技术约束
- taffy 0.7.7（vendored，`crates/taffy-local`）；IFC = `inline/mod.rs::InlineFormattingContext`；存储模式见 `inline_finalization.rs`。
- 单 `.rs` ≤2000 行（multicol.rs 当前 628，统一流逻辑若使超限则抽 `multicol_unified_flow.rs`）。

### 6.5 假设（rally 自主模式，待实施第一步 probe 验证）
- **A1（目标案 diff 根因）**——**R383 LAYOUT_DUMP 深度诊断重大纠正**：原假设「inline 未分配」**不成立**。multicol-block-no-clip-002 的 5 个 `<span>`（display:inline）经 R109（inline→block converter）**被转成 block-level LayoutBox**，与 h4 共 6 个 block 子，multicol 把它们按原子 block 分配到 3 列（blue+h4→col1、orange+pink→col2、yellow→col3，各列顶 y=28/48）。但 ref 期望 **inline 内容作单一 IFC 跨列流动**（blue 4 行+h4+orange 1 行填 col1 至 balance 高 → orange 余+pink 溢 col2 → pink 余+yellow 填 col3，**span 跨列分裂**）。**根因 = R109 entanglement**：spans 已是 block 盒，统一 column-flow 即使实现仍按 block 分配，**修不了这些案**。**真修复须先 Phase A（inline 内容作流动 IFC，R109 解转换）再 multicol 列碎片化——两多会话 lever 依赖（Phase A → multicol）**。状态：根因已定位，**本 spec 的混合内容目标案前置依赖 Phase A，非独立可实施**。
- **A2（balance 高度可复用）**：统一流的 balance 高度 = 现有 `total_content_height / col_count`（pure-inline paint 侧同公式），对混合内容（block+inline 总高）同样适用。**待 probe**：目标案 chromium 的 balance 高度是否 = 总高/列数。若 chromium 用迭代二分搜索（R199/R321/R322 证 pure-inline 非此），混合内容可能不同。状态：待验证。
- **A3（IFC 可按列预算跑）**——**R382 probe 已解决（无需扩展 IFC）**：原假设「IFC 须加 height-budget 入参」**不成立**。重读 §8.4 `flush_inline_run`：IFC 产出**宽度换行的全部行盒**（现有 `break_items_into_lines` 能力），统一流再按 balance 高度切片到列——**列切片逻辑在统一流，非 IFC**，故 IFC 接口不变。注：`inline/mod.rs:940 break_items_into_columns` 是**垂直书写模式**换行（line 941「垂直模式 container_width=向下推进最大高度」），**非 multicol 列能力**（命名碰撞），与本 spec 无关。状态：已解决，简化实施。

### 6.5A 实现来源说明

| 能力/行为 | 来源类型 | 具体来源 | 备注 |
|----------|----------|----------|------|
| block 子列分配（等价） | 复用现有 | `multicol.rs::assign_children_to_columns_balanced` + `position_multicol_children` | 统一流 block 路径须 byte-identical 复现 |
| inline 行盒（按列预算） | 复用现有（无需扩展） | `inline/mod.rs::InlineFormattingContext::break_items_into_lines` 产出宽度换行行盒；统一流 `flush_inline_run` 按 balance 高度切片到列 | A3 已解决：IFC 接口不变，列切片在统一流 |
| 列几何（col_count/col_width/gap） | 复用现有 | `multicol.rs::compute_column_info` / `compute_single_column_width`（R185 验证公式正确） | |
| 存储 + paint 消费 | 复用现有模式 | `inline_finalization.rs` 存储模式 + `painter/text.rs:854` 列渲染（列 x 偏移 + 列内 y + 裁剪） | 新字段 `inline_multicol_columns` |
| chromium-Oracle 验证 | 复用现有 | `tests/wpt-runner/scripts/cross-validate.py` + `oracle-shots/` | R381 方法论 |

### 6.6 代码变更边界
- **允许修改**：`crates/layout-engine/src/multicol.rs`（或新增 `multicol_unified_flow.rs`）、`crates/layout-engine/src/types/mod.rs`（加字段）、`crates/layout-engine/src/engine.rs`（调用统一流）、`crates/engine/src/paint/painter/text.rs`（消费新字段）、`crates/layout-engine/src/inline/mod.rs`（A3 若需扩展 IFC 预算入参）。
- **禁止修改**：`crates/taffy-local/**`（R304 DEFER）、`painter/text.rs:569` 门控的 `height_auto` 条件（R317）。

---

## 7. 实施交接（Implementation Handoff）

### 文件/模块清单

| 路径/模块 | 动作 | 目的 | 风险/注意 |
|----------|------|------|----------|
| `crates/layout-engine/src/types/mod.rs` | 修改（加字段） | `LayoutBox.inline_multicol_columns` | 默认 None，不影响现有 |
| `crates/layout-engine/src/multicol.rs` 或新增 `multicol_unified_flow.rs` | 新增 | `flow_children_into_columns` 统一流 | 行数控制（≤2000）|
| `crates/layout-engine/src/engine.rs` | 修改 | 调用统一流（env 门控） | 仅目标结构触发 |
| `crates/engine/src/paint/painter/text.rs` | 修改 | 消费 `inline_multicol_columns`（Some 时按列渲染，None 回退）| 不动 :713 门控 |
| `crates/layout-engine/src/inline/mod.rs` | 可能修改（A3） | IFC 加 height-budget 入参（若核查发现需要）| 接口扩展需守 Phase A stored-path 不回归 |

### 推荐修改顺序

1. **Probe A1/A2/A3**（read-only，先做）：REFTEST_DUMP multicol-block-no-clip-002 + multicol-containing-002，逐像素确认 block 子位置对、inline 未分配（A1）；核查 chromium balance 高度（A2）；grep IFC 是否有 height-budget 入参（A3）。**若 A1=block 也错，停止重估范围**。
2. **block 等价路径**（env 门控，byte-identical）：实现 `flow_children_into_columns` 的 block-only 分支，验证全量 reftest byte-identical（41 multicol 通过案不变）。这是回归安全网。
3. **inline-run 处理**（env 门控）：加 inline-run 累积 + 按列预算跑 IFC + 行盒定到列内。逐案 chromium-Oracle 验证目标案 z_vs_chr 下降。
4. **paint 消费**：text.rs 当 `inline_multicol_columns` Some 时按列渲染。
5. **开启 + 全量验证**：`MULTICOL_UNIFIED_FLOW=1` 全量 reftest + cross-validate，确认净正向（目标案 z_vs_chr 降 + 零回归）后默认开启。

### 首批提交建议

| 提交/批次 | 范围 | 预期结果 | 验证 |
|----------|------|----------|------|
| Probe | A1/A2/A3 诊断 evidence | 确认/证伪假设 | evidence/r382-multicol-phase2-probe-*.txt |
| Commit 1 | block 等价路径（env 门控） | byte-identical，443/490 不退 | make reftest 全量 |
| Commit 2 | inline-run + paint 消费 | 目标案 z_vs_chr 降 | cross-validate 目标案 + 全量零回归 |

---

## 8. 技术设计（RFC）

### 8.1 现状分析
- **当前架构**：三段分离（taffy 块堆叠 → `assign_children_to_columns_*` 重分配 block 子 → IFC 单独跑 inline，paint 侧仅 pure-inline 分配）。见 §1.1 表。
- **痛点**：block/inline 交错流动结构性不可表达；混合内容 multicol 整块堆叠错渲染（16 失败的主因）。
- **相关代码**：`multicol.rs`（628 行）、`painter/text.rs:569-964`（multicol 门控+分配）、`inline/mod.rs`（IFC）、`inline_finalization.rs`（存储）。

### 8.2 目标状态
- **提议架构**：layout 侧统一 `flow_children_into_columns`，按文档序遍历子节点，block 与 inline 在同一游标下逐列流动；结果存 `inline_multicol_columns`，paint 直接消费。
- **关键变更**：① 新字段；② 新统一流函数（block 等价 + inline-run）；③ paint 消费路径；④（A3 若需）IFC height-budget 入参。

### 8.3 影响范围分析

| 影响项 | 影响程度 | 说明 |
|--------|----------|------|
| multicol 渲染（目标结构） | 高 | 混合内容 balance 案从整块堆叠 → 正确列分布 |
| multicol 渲染（非目标结构） | 低 | env 关闭=byte-identical；开启=字段 None 走旧路径 |
| Phase A stored IFC 路径 | 中 | 新字段独立，不耦合 `inline_layout`；但 IFC 若加预算入参须守 stored-path 不回归 |
| paint text.rs | 中 | 加 Some 分支消费新字段，不动 :713 门控 |

### 8.4 详细设计

**统一流伪代码**（`flow_children_into_columns`）：

```
输入: container(含 children 按 DOM 序), col_info{count,width,gap}, styles, doc
输出: Option<Vec<Vec<InlineLayoutLine>>>  // 每列行盒；None=非目标结构走旧路径

# 门控：仅 单层 + balance + height:auto + 混合 block+inline 子
unless is_target_structure(container, style): return None

balance_h = estimate_balance_height(container)   # A2：总内容高 / col_count（待 probe）
col_x[i] = i * (col_width + gap)
cols: Vec<Vec<InlineLayoutLine>> = vec![vec![]; col_count]
block_frags: Vec<(col_idx, child_idx, y_in_col)>  # block 子列定位（替代 position_multicol_children）

col_idx = 0; col_y = 0.0
inline_run: Vec<NodeId> = []   # 累积的连续 inline 子

for (child_idx, child) in container.children (DOM 序):
    if child is block-level:
        flush_inline_run(inline_run, col_idx, col_y, balance_h, col_width) → 追加行盒到 cols[col_idx]，更新 col_y
        # 放 block 子
        if col_y + child.height > balance_h and col_idx < col_count-1:
            col_idx += 1; col_y = 0
        block_frags.push((col_idx, child_idx, col_y)); col_y += child.height
        # 注：block 子的 taffy 几何保留，统一流只决定它在哪列
    else:  # inline
        inline_run.push(child)
flush_inline_run(inline_run, col_idx, col_y, ...)   # 收尾

# block 子定位写回（复用 position_multicol_children 逻辑，按 block_frags）
apply_block_column_positions(container, block_frags, col_info)
return Some(cols)

fn flush_inline_run(run, col_idx, col_y, balance_h, col_width):
    if run empty: return
    available_h = balance_h - col_y
    ifc = IFC::new(run, col_width, height_budget=available_h)   # A3：IFC 须支持预算
    for line in ifc.lines:
        if col_y + line.height > balance_h and col_idx < col_count-1:
            col_idx += 1; col_y = 0; available_h = balance_h
        line.y_in_col = col_y; cols[col_idx].push(line); col_y += line.height
```

**关键子问题**：
- **block 等价**：`block_frags` 的列分配须与 `assign_children_to_columns_balanced` 一致（顺序填充到 balance_h）——byte-identical 验证。
- **IFC height-budget（A3）**：当前 IFC（`inline/mod.rs`）按可用**宽度**换行，不按高度截断。统一流需要 IFC 在给定高度预算内产出行盒、剩余行交给下列。**若 IFC 无此能力，Phase 2 须先扩展 IFC 接口**（加 `height_budget: Option<f32>`，超过预算的行标记溢出）。这是 A3 核查的关键。
- **balance 高度（A2）**：`estimate_balance_height` = 总内容高 / col_count。混合内容的「总内容高」= block 子高之和 + inline 内容高（须先跑一次 IFC 量 inline 高，可能两趟）。

### 8.5 安全考虑
- **回归风险**：41 通过 multicol 案 + multicol-fill-auto-001 sentinel。缓解：env 门控 + block 等价 byte-identical + 逐案 chromium-Oracle。
- **多会话风险**：统一流是大改，半完成状态须 env 关闭=零影响。缓解：每个 Commit 独立可合并（block 等价先行）。

### 8.6 替代方案

| 方案 | 描述 | 优点 | 缺点 | 决定 |
|------|------|------|------|------|
| A. layout 侧统一 column-flow | 本 spec | 结构性正确，覆盖混合内容+breaking | 大改，多会话 | ✅ 选定 |
| B. paint 侧协调（放宽门控+target_h 扣 block 高） | R157/R317 谱系 | 改动小 | **5 轮证伪不可解**（paint 无 block 子列位置） | ❌ 拒绝 |
| C. taffy 升级 0.11 用 native multicol | 上游原生 | 理论最干净 | R304 DEFER（541 ref+冲突，具名缺口零收益） | ❌ 拒绝 |

**最终选择**：方案 A。理由：① B 已 5 轮证伪；② C 已 DEFER；③ A 是唯一结构性正确路径，且可 env 门控渐进、block 等价 byte-identical 安全网。

### 8.7 实施计划
见 §7 推荐修改顺序（Probe → block 等价 → inline-run + paint → 开启验证）。多会话：本会话产 spec + Probe（若时间允许），后续会话 Commit 1/2。

### 8.8 测试策略
- **单元测试**：block 等价（复现 assign_children_to_columns_balanced）、inline-run 列内定位、列满续列、IFC height-budget 溢出。
- **reftest**：全量 loose 443/490 不退 + multicol 子集 41/57 不退。
- **chromium-Oracle**：目标案 z_vs_chr < 1%（cross-validate.py）。

### 8.9 回滚计划
env `MULTICOL_UNIFIED_FLOW=0`（默认）即完全回退；任一 Commit 净负向即 `git revert`。block 等价 Commit 独立可保留（byte-identical 无害）。

---

## 9. Spec Lint 报告

### 结构完整性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要存在性 | ✅ Pass | §0 含目标/范围/排除/约束/方案/首步 |
| 场景存在性 | ✅ Pass | FR-001~003 各 ≥2 验收场景 |
| 异常路径覆盖 | ✅ Pass | FR-001 含「现有案不回归」异常场景；FR-002 含 None 回退；FR-003 含开启后非目标不变 |
| 测试绑定 | ✅ Pass | 每场景标 REFTEST_DUMP/cross-validate/单测名/make reftest |
| TBD 清零 | ⚠️ Warning | §10 A1/A2/A3 标「待 probe/核查」——非阻塞性（首步即 probe 验证），但未清零 |
| 约束覆盖 | ✅ Pass | NFR-001/002 覆盖 §6.1 零回归 + chromium-Oracle 约束 |
| 实施交接完备 | ✅ Pass | §7 含文件清单/职责/修改顺序/首批提交 |
| 首步可执行性 | ✅ Pass | §7 步骤 1 = Probe A1/A2/A3（read-only，验证方式明确） |

### 语言精确性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 模糊动词 | ✅ Pass | 用「流动/定位/累积/flush」等具体动词 |
| 无量化描述 | ✅ Pass | NFR 量化为「443/490」「z_vs_chr<1%」「41/57」 |
| 非确定性措辞 | ✅ Pass | 用「必须」；A1/A2/A3 显式标「待验证」未升为 FR |

### 一致性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 范围冲突 | ✅ Pass | §1.3 在范围（混合内容 balance auto）与不在范围（嵌套/breaking/pure-inline）无交集 |
| 约束冲突 | ✅ Pass | §6.1 必须「仅目标结构」与 §6.2 禁止「放宽 :713 门控」互补 |
| 方案漂移 | ✅ Pass | 方案 A 依赖新字段+新函数+paint 消费，均在 §6.6 允许范围；不碰禁止的 taffy/:713 |
| 章节引用正确 | ✅ Pass | IF-001 引用 types/mod.rs:267/371（已验存在）；§8.4 引用 inline/mod.rs（已验存在） |
| 外部事实保守化 | ✅ Pass | taffy 0.7.7、IFC 接口已验；A3（IFC height-budget 能力）标「待核查」未升为 FR |
| 未验证细节泄漏 | ✅ Pass | A1/A2/A3 未写进 FR 期望值，仅在 §7 首步 probe |
| 场景预期泄漏 | ✅ Pass | 验收场景期望「z_vs_chr<1%」「byte-identical」是行为非硬编码；未把 A3 未验证的 IFC 接口写进断言 |
| 实现来源闭合 | ✅ Pass | §6.5A 表覆盖 block 等价/IFC/列几何/存储/Oracle 验证，均指明来源 |
| 类型分层清晰 | ✅ Pass | FR/§6.3 决策/§6.5 假设/§10 TBD 分层明确 |
| 优先级完备 | ✅ Pass | 每个 FR/NFR 标优先级 |
| 代码边界完备 | ✅ Pass | §6.6 允许/禁止路径均列 |
| 重复失控 | ✅ Pass | 统一流伪代码主定义在 §8.4，§3/§5 仅引用 |

**汇总**：23 Pass / 1 Warning / 0 Fail / 0 Skip
**门禁判定**：Fail = 0 → **允许确认（rally 协议下直接进入实施接力，首步 Probe A1/A2/A3）**

---

## 10. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|----|------|--------|----------|--------|
| A1 | 混合内容 balance 案 diff 是否**全部**来自 inline 未分配（block 子位置对） | 重要（决定范围） | **R382 部分probe：内容挤左上角未跨列，范围成立；block-vs-inline 精确归因待深 probe** | 实施 Commit 1 前：逐像素确认 h4 是否也应跨列 |
| A2 | chromium 对混合内容 balance 高度是否 = 总高/列数（vs 迭代二分） | 重要 | 未单用例验证 | Commit 2 前：量目标案 chromium 列高 |
| A3 | `InlineFormattingContext` 是否支持 height-budget 入参 | ~~重要~~ **R382 已解决：无需扩展 IFC** | 已核查：IFC 产宽度换行行盒，列切片在统一流；`break_items_into_columns` 是垂直模式无关 | ✅ 关闭 |

---

## 11. 修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v1.0 | 2026-06-20 | 初始版本（R382，spec-rfc 标准模式，承接 R381 Phase 1 gate 关闭后的 Phase 2 路由） |
| v1.0-probe | 2026-06-20 | R382 probe：A3 已解决（IFC 无需扩展，列切片在统一流；`break_items_into_columns` 是垂直模式无关）；A1 部分 probe（multicol-block-no-clip-002 内容挤左上角未跨列，范围成立，block-vs-inline 精确归因待深 probe）；A2 待 Commit 2 前验证 |
| v1.0-R383 | 2026-06-20 | **R383 LAYOUT_DUMP 深度诊断重大纠正**：A1 根因 = **R109 entanglement**（inline spans 被转 block 盒按原子分配到列，非 IFC 流动）。混合内容目标案前置依赖 Phase A（inline→IFC），本 spec 统一流非独立可实施。两 lever 依赖：Phase A → multicol。R109-independent 的嵌套 breaking（Phase 3）可独立推进 |

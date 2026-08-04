# Spec：multicol layout 侧 column-aware IFC（行盒列碎片化）

**版本**：v1.0
**日期**：2026-06-19
**作者**：AI Assistant（rally autonomous，spec-rfc 完整模式）
**状态**：草稿（待实施接力；rally 协议下不向用户提问，假设见 §6.5）

**关联**：rendering-compat master.md R113/R122/R128/R131/R157/R199-R205/R310/R317；`multicol-fragmentation-design.md` v0.5（R317 重定向后）；css-multicol 17/57 失败聚类。

---

## 0. 执行摘要

> **⚠️ v1.0-gate（R381，2026-06-20）：Phase 1 紧急停止——§10 假设 A1 验证为 FALSE**。扫描全 16 个 css-multicol 失败案结构，**0/16 匹配** Phase-1 目标（单层+balance+明确高度+纯 inline）。每案或有 block 子元素、或 height:auto、或 column-fill:auto、或 breaking/嵌套。**Phase 1（pure-inline balance）无目标案、零杠杆**；真实 multicol 失败全需 **Phase 2（嵌套/breaking/混合碎片化，多会话硬核）**。spec 自身 §10 协议「A1 不存在→紧急停止转 Phase 2」已生效。下方 Phase 1 设计保留作参考与 Phase 2 的算法基础，**勿再以 Phase 1（pure-inline balance）为单会话 lever 重试**。

- **一句话目标**：把 multicol 容器（`column-count`/`column-width`）的**行内流（inline）内容**在 **layout 阶段**按列高预算碎片化到各列并存储，paint 直接消费存储结果——取代当前 paint 侧 `text.rs:569` 门控重算（已由 R157/R198/R203/R317 四轮实证 net-negative）。
- **本期范围（Phase 1）**：仅**单层、非嵌套** multicol 容器，`column-fill: balance`（默认）+ **明确高度**（height ≠ auto）+ 纯 inline 内容（无 block 子元素）。该子集当前因 `text.rs:569` 的 `height_auto` 门控**回退为单块渲染（错误）**，layout 侧填充新字段可纯改善。**⚠️ v1.0-gate：该范围在失败集中 0 匹配，Phase 1 停止。**
- **明确排除（本期）**：① 嵌套 multicol（outer column-fill:auto 把 inner 碎片化，multicol-breaking-004/005/006）= Phase 2（真 fragmentation）；② `column-fill: auto` 顺序填充的 inline 内容（由 multicol.rs block 侧 + paint 协调，独立子问题）；③ block 子元素碎片化（multicol.rs `assign_children_to_columns_*` 已实现，**不重复**）。
- **核心约束**：① 零 reftest 回归（loose 438/490 不退）；② 不改 height:auto 现有路径（paint 侧门控保持 load-bearing，R317）；③ 单 `.rs` 文件 ≤2000 行；④ 测试用 `make test`/`make reftest`（test-guard 包裹）。
- **推荐方案**：新增 `LayoutBox.inline_multicol_columns: Option<Vec<Vec<InlineLayoutLine>>>`（每列行盒），layout 侧 `assign_lines_to_columns_balanced`（行盒版，与 block 侧 `assign_children_to_columns_balanced` 同族但作用于 `InlineLayoutLine`），paint 侧优先消费该字段（None 时回退现有逻辑）。
- **首个落地步骤**：① 先 grep/probe 确认 css-multicol 失败用例中「单层 + balance + 明确高度 + 纯 inline」结构的实际存在与 diff（假设 A1，§6.5 待验证）；② 若存在，加 `inline_multicol_columns` 死字段 + layout 填充 + paint 消费，逐用例验证。

---

## 1. 背景与目标

### 1.1 背景

css-multicol 是当前 reftest 最大失败聚类之一（17/57 loose 失败，strict 更多）。碎片化**算法已存在**（`multicol.rs::assign_children_to_columns_balanced/_sequential/_with_breaking`，作用于 **block 子元素**），但**行内流（inline 文本）内容的列分布仅在 paint 侧**（`painter/text.rs:569` 的 `compute_multicol_info_for_paint`）做，且被 `height_auto` 门控严格限制：

```rust
// painter/text.rs:569（load-bearing，R317 实证不可放宽）
let multicol_info = if !has_in_flow_children && is_balance_mode && height_auto {
    compute_multicol_info_for_paint(...)  // 仅 height:auto+balance+纯inline 触发
} else { None };                            // 其余回退单块渲染
```

R317 实测放宽该门控（去掉 `height_auto`）：multicol 子集 40/57→35/57（净 -5），目标 multicol-breaking-004 反而恶化 5.60→6.17%。这是第 5 次实证 paint 侧协调不可解（R157 净中性 / R198 font_size 死锁 / R203 净负 / R122 守卫净中性 / R317 净 -5）。paint 侧 `total/col_count` 均衡分配对明确高度/嵌套用例结构性错误。

### 1.2 目标

- **业务目标**：提升 css-multicol 真实 reftest 通过率（chromium-Oracle 一致率），消除「明确高度 balance multicol 的 inline 内容退化为单块」类失败。
- **用户目标**：让 ZeroWeb 的多列排版在 balance 模式下与 Chromium 一致（内容均衡分布到各列）。

### 1.3 范围边界

- **在范围内**：
  - 单层 multicol + balance + 明确高度 + 纯 inline 内容的 layout 侧行盒列分布（Phase 1）。
  - 新增 `LayoutBox.inline_multicol_columns` 字段 + layout 填充 + paint 消费。
  - `assign_lines_to_columns_balanced`（行盒版顺序填充到平衡高度 `total_lines_height/col_count`）。
- **不在范围内**：
  - 嵌套 multicol / column-breaking（Phase 2，真 fragmentation，跨列断裂）。
  - `column-fill: auto` 的 inline 内容（独立）。
  - block 子元素碎片化（已实现）。
  - multicol baseline-export（R310/R312/R313/R316 已从 ZeroWeb 侧 ruled out）。

---

## 2. 需求类型概览

| 类型 | 是否适用 | 来源 |
|------|---------|------|
| 业务需求 | 是 | DC-4 multicol ≥95% reftest 目标 |
| 用户需求 | 是 | 多列排版与 Chromium 一致 |
| 解决方案需求 | 是 | layout 侧 column-aware IFC（R131/R317 重定向） |
| 功能需求 | 是 | §3 |
| 非功能需求 | 是 | §4（零回归、文件行数） |
| 接口需求 | 是 | §5（新字段 + paint 消费契约） |
| 过渡需求 | 是 | paint 侧回退兼容（字段 None 时走旧逻辑） |

---

## 3. 功能需求

### FR-001：layout 侧为「单层 balance 明确高度纯 inline」multicol 计算并存储每列行盒

- **描述**：当 multicol 容器满足（单层、`column-fill: balance`、height ≠ auto、无 block 子元素、有 inline 内容）时，layout 阶段必须运行 IFC 得到 `Vec<InlineLayoutLine>`，按平衡高度 `sum(line.height)/col_count` 顺序填充把行盒分配到 `col_count` 列，结果存入 `LayoutBox.inline_multicol_columns`。
- **优先级**：必须（Phase 1）
- **来源**：R131/R317

**验收场景**：

```
场景: 单层 balance 明确高度 multicol 的 inline 内容列分布（正常）
  假设 一个 <div style="column-count:2;height:200px"> 含 8 行文本，无 block 子元素
  当 layout 计算
  那么 inline_multicol_columns = Some(2 列)，每列约 4 行，col0 行盒 y 从 0 起、col1 行盒 y 从 0 起（列内 rebase）
  验证: 单元测试 test_assign_lines_to_columns_balanced_even + make reftest 对应用例 diff 下降

场景: height:auto multicol 不触发新路径（异常/边界）
  假设 同上但 height:auto
  当 layout 计算
  那么 inline_multicol_columns = None（不触发，paint 侧门控继续处理，零回归）
  验证: 单元测试 + make reftest 全量 loose 438/490 不退

场景: 有 block 子元素的 multicol 不触发（异常/边界）
  假设 multicol 含 block <p> 子元素
  当 layout 计算
  那么 inline_multicol_columns = None（block 子元素走 multicol.rs assign_children_*）
  验证: 单元测试 + make reftest
```

### FR-002：paint 优先消费 `inline_multicol_columns`，None 时回退现有逻辑

- **描述**：paint（`painter/text.rs`）渲染 multicol inline 内容时，必须先检查 `box_node.inline_multicol_columns`；为 `Some` 时按每列行盒渲染（列 x 偏移 = `col_idx*(col_width+gap)`，列内行盒 y 用存储值，逐列裁剪）；为 `None` 时走现有 `compute_multicol_info_for_paint` 门控逻辑（不变）。
- **优先级**：必须（Phase 1）
- **来源**：R317（paint 侧门控保持 load-bearing，仅在新字段有值时短路）

**验收场景**：

```
场景: 新字段有值时按列渲染（正常）
  假设 box_node.inline_multicol_columns = Some([col0_lines, col1_lines])
  当 paint
  那么 col0 行盒渲染在 x=[0,col_width)，col1 在 x=[col_width+gap, ...)，超出列边界裁剪
  验证: make reftest 对应用例 diff 下降 + 产品 smoke 不退

场景: 新字段 None 时回退（异常/边界）
  假设 box_node.inline_multicol_columns = None
  当 paint
  那么 走现有 text.rs:569 门控逻辑（height:auto+balance 走 paint 分布，其余单块）
  验证: make reftest 全量 loose 438/490 不退（height:auto 用例 byte-identical）
```

### FR-003：行盒列分布算法 = 顺序填充到平衡高度（与 block 侧同族，作用于行盒）

- **描述**：`assign_lines_to_columns_balanced(lines, col_count)` 必须按文档顺序把行盒填入当前列，当 `current_col_height >= sum/col_count` 且有更多列时移到下一列——与 `multicol.rs::assign_children_to_columns_balanced`（block 版）同算法、同语义，仅输入从 `(child_idx, height)` 换成 `InlineLayoutLine`。
- **优先级**：必须（Phase 1）
- **来源**：R200（顺序填充 + 平衡高度 `total/count` 已证正确，非 round-robin）

**验收场景**：

```
场景: 整除情况（正常）
  假设 8 行等高行盒，col_count=4
  当 assign_lines_to_columns_balanced
  那么 每列恰好 2 行
  验证: 单元测试 test_assign_lines_balanced_even

场景: 非整除（异常/边界）
  假设 11 行等高，col_count=6
  当 assign
  那么 顺序填充到平衡高度，前列略多/后列略少（符合 chromium §8 顺序填充）
  验证: 单元测试 test_assign_lines_balanced_uneven
```

---

## 4. 非功能需求

### NFR-001：零 reftest 回归
- **描述**：Phase 1 落地后，`make reftest` 全量 loose 通过率不得低于 438/490；height:auto + balance 的现有用例必须 byte-identical（新字段为 None，paint 走旧路径）。
- **测量标准**：`make reftest` loose 计数 ≥ 438；目标 multicol 子集 ≥ 40/57 且改善的用例不引发别处回归。
- **优先级**：必须

### NFR-002：文件行数
- **描述**：新增代码不得使任何 `.rs` 文件超过 2000 行。`engine.rs`（当前 3969 行，已超）须把新逻辑放进独立模块（如 `multicol_inline.rs`）而非继续膨胀 engine.rs。
- **测量标准**：`wc -l` + `cargo clippy` 干净。
- **优先级**：必须

### NFR-003：chromium-Oracle 真实一致率提升
- **描述**：Phase 1 目标用例的 `z_vs_chr`（ZeroWeb-test vs chromium-test）必须下降；仅 self-source 通过率变化不算达成（DC-14）。
- **测量标准**：`scripts/cross-validate.py` 或单用例 oracle 对比，目标用例 z_vs_chr 下降。
- **优先级**：必须

---

## 5. 接口需求

### IF-001：`LayoutBox.inline_multicol_columns` 新字段
- **类型**：数据结构（layout → paint 契约）
- **规格**：`pub inline_multicol_columns: Option<Vec<Vec<InlineLayoutLine>>>`。`None` = 未计算/不适用（paint 走旧路径）；`Some(cols)` 中 `cols[col_idx]` = 该列的行盒列表，行盒 `y` 已 rebase 为列内坐标（从 0 起）。列 x 偏移由 paint 按 `col_idx*(col_width+gap)` 计算（不存字段，避免与 multicol 几何重复）。
- **错误处理**：若 `cols` 为空 Vec 或 `col_count==0`，paint 视同 `None` 回退。
- **默认动作**：字段默认 `None`；仅 FR-001 条件全满足时填充。
- **交叉引用**：复用 `InlineLayoutLine`（types/mod.rs:502），与 `LayoutBox.inline_layout`（已有，存单列行盒）并行。

### IF-002：`assign_lines_to_columns_balanced` 函数（layout 侧）
- **类型**：模块函数
- **规格**：`fn assign_lines_to_columns_balanced(lines: &[InlineLayoutLine], col_count: usize) -> Vec<Vec<InlineLayoutLine>>`。返回每列行盒（行盒 y 未 rebase，由调用方/paint 按 col 内起点 rebase）。语义与 `multicol.rs::assign_children_to_columns_balanced` 一致。
- **错误处理**：`lines` 空或 `col_count==0` → 返回 `vec![vec![]; col_count.max(1)]`。
- **交叉引用**：详见 §8.4。

---

## 6. 约束与假设

### 6.1 必须约束（Must）
- Phase 1 仅作用于「单层 + balance + 明确高度 + 纯 inline」multicol；其余全部走旧路径（字段 None）。
- paint 消费新字段时必须逐列裁剪（复用 text.rs:881 现有逐列裁剪逻辑）。
- 行盒列分布必须用顺序填充 + 平衡高度（R200 已证正确），不得用 round-robin。

### 6.2 禁止约束（Must Not）
- 不得放宽 `text.rs:569` 的 `height_auto` 门控（R317 实证净 -5 回归）。
- 不得修改 multicol.rs 的 block 子元素碎片化（`assign_children_to_columns_*`，已正确）。
- 不得在 paint 侧为嵌套 multicol 做 ad-hoc 协调（R203 证 net-negative）。
- 不得为「只使用一次」的逻辑引入抽象（code-guidelines）。

### 6.3 已定决策
- 存储用新字段 `inline_multicol_columns`（非复用 `inline_layout`，因后者是单列语义、被 Phase A stored-path 消费，混用会耦合）。
- 行盒 y 存「列内 rebase 后」值（paint 直接用，无需再减列起点）。
- 算法复用 `assign_children_to_columns_balanced` 的顺序填充逻辑（行盒版）。

### 6.4 技术约束
- taffy 0.7.7 vendored（R304 DEFER 升级）；不依赖 taffy 新能力。
- IFC `layout()` 已能产出 `Vec<InlineLayoutLine>`（inline/mod.rs），无需改 IFC 内部。
- 编辑器/MSRV：Rust edition 2024，MSRV 1.85。

### 6.5 假设
- **A1（⚠️ R319 实测大体 REFUTED）**：~~css-multicol 失败用例中存在「单层 + balance + 明确高度 + 纯 inline」结构且 diff 来自未列分布~~。**R319 probe 结果**：grep 6 个非嵌套失败用例结构——multicol-fill-000 / count-002 / columns-001 均为 **height:auto + balance + inline**（paint 侧**已处理**，同算法迁移 layout 侧结果不变；其 diff 是列宽/glyph 精度，即 R225 证伪的 advance-width 谱系，**非列分布**）；column-height-009 用 multicol-2 `column-height` 简写（非 balance+height 组合）；multicol-containing-002 含 `<img>`（非纯 inline）；multicol-block-no-clip-002 含 `<h4>` block。**结论：Phase 1 目标结构在失败集中近乎不存在，迁移 layout 侧对 height:auto 用例零改善（同算法）。Phase 1 价值 REFUTED。** 状态：已验证（refuted）。**处置：Phase 1 不实施；multicol 真实 forward motion 收敛为 ① 列宽/glyph 精度（advance-width R225 死路，须 fontdue glyph advance 接入，独立大件）；② Phase 2 嵌套 fragmentation（硬结构性）；③ 接受 multicol plateau。**
- **A2（待验证）**：layout 侧行盒列分布对这类用例产生的结果与 chromium 一致（即平衡高度 `sum/col_count` 对明确高度容器正确）。状态：待验证——R200 已证 height:auto 正确，明确高度待单用例验证。
- **A3（已验证）**：paint 侧 `height_auto` 门控 load-bearing——R317 实测放宽净 -5。本设计**不放宽**它，仅在新字段有值时短路。状态：已验证。

### 6.5A 实现来源说明

| 能力/行为 | 来源类型 | 具体来源 | 备注 |
|----------|----------|----------|------|
| IFC 行盒产出 | 复用现有模块 | `inline/mod.rs::InlineFormattingContext::layout` → `Vec<InlineLayoutLine>` | 已实现，无需改 |
| 行盒列分布算法 | 仓内自实现（同族复用） | 新增 `assign_lines_to_columns_balanced`，逻辑照搬 `multicol.rs::assign_children_to_columns_balanced` | 行盒版，非 block 版 |
| 列几何（col_count/col_width/gap） | 复用现有模块 | `multicol.rs::compute_column_info` / `balance_column_geometry` | 已实现 |
| paint 逐列裁剪 | 复用现有模块 | `painter/text.rs:881` 现有逐列裁剪 | 已实现 |
| `InlineLayoutLine` 存储 | 复用现有类型 | `types/mod.rs:502` | 已实现 |

### 6.6 代码变更边界
- **允许修改**：
  - `crates/layout-engine/src/types/mod.rs`（加 `inline_multicol_columns` 字段）
  - `crates/layout-engine/src/multicol*.rs` 或新增 `crates/layout-engine/src/multicol_inline.rs`（行盒列分布 + layout 填充）
  - `crates/engine/src/paint/painter/text.rs`（paint 消费新字段，短路分支）
- **禁止修改**：
  - `crates/layout-engine/src/multicol.rs::assign_children_to_columns_*`（block 侧，已正确）
  - `painter/text.rs:569` 门控条件本身（仅在其前加新字段短路分支）
  - IFC 内部（`inline/mod.rs` 行盒生成逻辑）

### 6.7 执行技能提示
- 无专用 skill 需求；通用执行器 + `make test`/`make reftest`（test-guard）即可。

---

## 7. 优先级与里程碑建议

| ID | 需求 | 优先级 | 理由 | 里程碑 |
|----|------|--------|------|--------|
| FR-003 | 行盒列分布算法 | 必须 | FR-001 前置 | M1 |
| FR-001 | layout 填充新字段 | 必须 | Phase 1 核心 | M1 |
| FR-002 | paint 消费 | 必须 | Phase 1 核心 | M1 |
| NFR-001/3 | 零回归 + oracle 提升 | 必须 | 门禁 | M1 |

### 建议里程碑
- **M1（Phase 1，本范围）**：单层 balance 明确高度纯 inline multicol 的 layout 侧行盒列分布 + paint 消费。预估 1-2 会话。
- **M2（Phase 2，后续，本 spec 不覆盖）**：嵌套 multicol / column-breaking（真 fragmentation）。需独立 spec-rfc。

### 实施交接（Implementation Handoff）

#### 文件/模块清单

| 路径/模块 | 动作 | 目的 | 风险/注意事项 |
|----------|------|------|---------------|
| `crates/layout-engine/src/types/mod.rs` | 修改 | 加 `inline_multicol_columns: Option<Vec<Vec<InlineLayoutLine>>>` 字段（默认 None） | 所有 `LayoutBox { .. }` 构造点需补字段（或用 `..Default`）；现有测试构造处需更新 |
| `crates/layout-engine/src/multicol_inline.rs` | 新增 | `assign_lines_to_columns_balanced` + `populate_inline_multicol_columns`（layout 填充） | 新模块，避免 engine.rs（3969 行）继续膨胀（NFR-002） |
| `crates/layout-engine/src/engine.rs` | 修改 | compute() 调 `populate_inline_multicol_columns`（在 multicol layout step 9 后、adjust_inline_block_positions 前） | 仅加 1 行调用；逻辑在新模块 |
| `crates/engine/src/paint/painter/text.rs` | 修改 | paint 入口加 `if let Some(cols) = box_node.inline_multicol_columns { 按列渲染 } else { 现有逻辑 }` | 短路分支在 `multicol_info` 计算前；不放宽现有门控 |

#### 职责映射

| 模块/文件 | 职责 | 依赖/被依赖 | 验证方式 |
|----------|------|------------|----------|
| `multicol_inline.rs` | 行盒→列分布 + 填充字段 | 依赖 `InlineFormattingContext`、`compute_column_info`；被 `engine.rs` 调 | 单元测试 + make reftest |
| `text.rs`（paint） | 消费字段渲染 | 依赖 `inline_multicol_columns` | make reftest + 产品 smoke |

#### 推荐修改顺序

1. **先 probe（A1 验证）**：grep css-multicol 失败用例找「单层 balance 明确高度纯 inline」结构 + REFTEST_DUMP 看当前是否单块渲染。**若不存在 → 停止 Phase 1，本 spec 转 Phase 2 设计**（紧急停止条款）。
2. 加 `inline_multicol_columns` 字段（默认 None）+ 全量构造点补字段 → `cargo check` 通过、`make test` 全绿（净 0，字段未消费）。
3. 加 `assign_lines_to_columns_balanced`（单元测试）+ `populate_inline_multicol_columns`（FR-001 条件填充）。
4. paint 加短路消费分支（FR-002）。
5. `make reftest` 全量验零回归 + 目标用例 diff 下降 + chromium-Oracle z_vs_chr 下降。

#### 首批提交建议

| 提交/批次 | 范围 | 预期结果 | 验证 |
|----------|------|----------|------|
| Commit 1 | 加 `inline_multicol_columns` 死字段（默认 None，无消费） | 净 0，`make test` 全绿 | `make test` |
| Commit 2 | `assign_lines_to_columns_balanced` + 单元测试 | 算法正确，无渲染变化 | 单元测试 |
| Commit 3 | layout 填充 + paint 消费（FR-001/FR-002） | 目标用例 diff 下降，零回归 | `make reftest` 全量 + oracle |

---

## 8. 技术设计（RFC）

### 8.1 现状分析
- **当前架构**：multicol inline 内容的列分布**仅在 paint 侧**（text.rs:569 `compute_multicol_info_for_paint`），被 `height_auto` 门控限制为 height:auto+balance+纯inline。block 子元素列分布**在 layout 侧**（multicol.rs）。两者割裂。
- **问题/痛点**：明确高度 balance multicol 的 inline 内容回退单块（错误）；嵌套 multicol 完全不支持 inline 列分布。paint 侧协调经 5 轮实证 net-negative。
- **相关代码**：`painter/text.rs:696-717`（门控）、`multicol.rs:1222`（block 侧碎片化）、`inline/mod.rs`（IFC）、`types/mod.rs:502-520`（InlineLayoutLine/Fragment）。

### 8.2 目标状态
- **提议架构**：inline 内容列分布**移到 layout 侧**，结果存 `inline_multicol_columns`，paint 直接消费（短路）。paint 侧 `compute_multicol_info_for_paint` 仅在字段 None 时作为 height:auto 的回退（保持 load-bearing）。
- **关键变更**：① 新字段；② 新模块 `multicol_inline.rs`；③ paint 短路分支。

### 8.3 影响范围分析

| 影响项 | 影响程度 | 说明 |
|--------|----------|------|
| 单层 balance 明确高度 inline multicol 用例 | 高（改善） | 从单块错误 → 列分布正确 |
| height:auto balance inline multicol 用例 | 低（不变） | 字段 None，paint 走旧路径，byte-identical |
| 嵌套 multicol 用例 | 无（Phase 1 不覆盖） | 字段 None（条件不满足） |
| block 子元素 multicol 用例 | 无 | 不触发新路径 |

### 8.4 详细设计

**数据流**：
```
compute() step 9 (multicol.rs block 侧) 之后：
  populate_inline_multicol_columns(root, doc, styles)
    for each multicol container (单层 + balance + height≠auto + 纯inline):
      运行 IFC → Vec<InlineLayoutLine>
      cols = assign_lines_to_columns_balanced(&lines, col_count)
      对每列行盒 y 做「列内 rebase」（减该列起始行 y）
      box.inline_multicol_columns = Some(cols)

paint text.rs：
  if let Some(cols) = box_node.inline_multicol_columns {
      for (col_idx, col_lines) in cols.iter().enumerate():
          col_x = col_idx * (col_width + gap)
          for line in col_lines:
              render line.fragments at (content_x + col_x, content_y + line.y)  // line.y 已列内 rebase
              clip to column rect
  } else {
      // 现有 text.rs:569 门控逻辑（不变）
  }
```

**`assign_lines_to_columns_balanced` 伪代码**（行盒版，照搬 `assign_children_to_columns_balanced`）：
```
total = sum(line.height for line in lines)
target = total / col_count
cols = [[] for _ in range(col_count)]
cur = 0; cur_h = 0
for line in lines:
    if cur_h >= target and cur+1 < col_count:
        cur += 1; cur_h = 0
    cols[cur].append(line)
    cur_h += line.height
return cols
```

**列内 rebase**：`for col: col_first_y = col[0].y; for line in col: line.y -= col_first_y`。

### 8.5 安全考虑
- **回归风险**：paint 短路分支若误触发（字段非 None 但条件本应 None）会改变现有用例。缓解：填充条件严格（FR-001 全部满足），且字段默认 None。
- **回滚**：字段默认 None 即等价回滚；Commit 1（死字段）天然净 0。

### 8.6 替代方案

| 方案 | 描述 | 优点 | 缺点 | 决定 |
|------|------|------|------|------|
| A. layout 侧 + 新字段（本设计） | layout 分布行盒存新字段，paint 消费 | 零回归（新字段默认 None）、与 block 侧对称、绕过 paint 门控 | 新字段 + 新模块 | ✅ 选定 |
| B. 放宽 paint 门控 + 修 paint 分布算法 | 改 `compute_multicol_info_for_paint` 支持明确高度 | 不加字段 | R317 实证净 -5，paint 侧 `total/count` 对明确高度结构性错误 | ❌ 拒绝（R317） |
| C. 复用 `inline_layout` 字段加列维度 | 不加新字段，给 inline_layout 加列结构 | 少一个字段 | inline_layout 是单列语义、被 Phase A stored-path 消费，混用耦合高风险 | ❌ 拒绝 |

**最终选择**：方案 A。理由：① 零回归（默认 None，height:auto 路径 byte-identical）；② 与 block 侧碎片化对称（一个管 block 子元素、一个管 inline 行盒，非重复）；③ 绕过 R317 实证的 paint 门控死锁。

### 8.7 实施计划
1. Probe A1（确认目标用例存在）。
2. Commit 1：死字段（净 0）。
3. Commit 2：行盒列分布算法 + 单元测试。
4. Commit 3：layout 填充 + paint 消费 + 验证。

### 8.8 测试策略
- **单元测试**：`assign_lines_to_columns_balanced`（整除/非整除/空/单列）；`populate` 条件门控（各条件不满足时返回 None）。
- **集成/reftest**：`make reftest` 全量零回归；css-multicol 子集目标用例 diff 下降。
- **oracle**：目标用例 z_vs_chr 下降（DC-14）。

### 8.9 回滚计划
字段默认 None = 等价回滚；Commit 1 单独可回滚（死字段）。任一 commit `make reftest` 出现回归即回退该 commit。

---

## 9. Spec Lint 报告

### 结构完整性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要存在性 | ✅ Pass | §0 含一句话目标/范围/排除/约束/方案/首步 |
| 场景存在性 | ✅ Pass | FR-001/002/003 各有 ≥1 验收场景（§3） |
| 异常路径覆盖 | ✅ Pass | FR-001 有 height:auto + block 子元素两个异常场景；FR-002 有 None 回退异常场景；异常 ≥ 正常 |
| 测试绑定 | ✅ Pass | 每场景标注 `单元测试 test_*` / `make reftest` |
| TBD 清零 | ✅ Pass | 无「阻塞」级 TBD；A1 是「待验证假设」非阻塞 TBD（实施第一步 probe 即验） |
| 约束覆盖 | ✅ Pass | §6.1 必须 3 条均被 FR/NFR 场景覆盖（零回归→NFR-001/FR-001 异常场景；不改门控→FR-002；顺序填充→FR-003） |
| 实施交接完备 | ✅ Pass | §7 含文件清单、职责映射、修改顺序、首批提交建议 |
| 首步可执行性 | ✅ Pass | §7 推荐修改顺序首步 = probe A1（grep + REFTEST_DUMP），验证方式明确 |

### 语言精确性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 模糊动词 | ⚠️ Warning | FR-002「逐列裁剪」中「裁剪」可接受（paint 域术语）；整体无「处理/管理/优化」滥用 |
| 无量化描述 | ✅ Pass | NFR-001 量化为「≥438/490」「byte-identical」；NFR-003 量化为「z_vs_chr 下降」 |
| 非确定性措辞 | ✅ Pass | 已用「必须」；§6.5 假设显式标「待验证」 |

### 一致性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 范围冲突 | ✅ Pass | §1.3 在范围（单层 balance 明确高度 inline）与不在范围（嵌套/auto-fill/block/baseline-export）无交集 |
| 约束冲突 | ✅ Pass | §6.1 必须 与 §6.2 禁止 无矛盾（必须「仅作用于 X」与禁止「放宽门控」互补） |
| 方案漂移 | ✅ Pass | 方案 A（§8.6）依赖新字段 + 新模块，均在 §6.6 允许范围内；不引入与 Must Not 冲突的依赖 |
| 章节引用正确 | ✅ Pass | IF-001 引用 types/mod.rs:502（已验存在）；§8.4 引用 text.rs:881（已验存在） |
| 外部事实保守化 | ✅ Pass | taffy 0.7.7、resvg 等已验；A1/A2 标「待验证」未升为 FR |
| 未验证细节泄漏 | ✅ Pass | A1（目标用例是否存在）未写进验收场景期望值，仅在实施首步 probe |
| 场景预期泄漏 | ✅ Pass | 验收场景期望「diff 下降」是行为非硬编码数值；无未验证 API/命名写入断言 |
| 实现来源闭合 | ✅ Pass | §6.5A 表覆盖 IFC 行盒/列分布算法/列几何/paint 裁剪/存储类型，均指明来源（复用现有或仓内自实现） |
| 类型分层清晰 | ✅ Pass | 需求(FR)/决策(§6.3)/假设(§6.5 A1-A3)/TBD(§10) 分层明确 |
| 优先级完备 | ✅ Pass | 每个 FR/NFR 标优先级（必须） |
| 代码边界完备 | ✅ Pass | §6.6 允许/禁止修改路径均列 |
| 依赖清单一致 | ✅ Pass | 无新增 crate 依赖（全复用现有/仓内自实现），跨 §0/§6.4/§6.5A 一致 |
| 重复失控 | ✅ Pass | 列分布算法主定义在 §8.4，§3/§5 仅引用 |

**汇总**：28 Pass / 1 Warning / 0 Fail / 0 Skip
**门禁判定**：Fail = 0 → **允许确认（rally 协议下直接进入实施接力）**

---

## 10. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|----|------|--------|----------|--------|
| A1 | 「单层 balance 明确高度纯 inline」multicol 失败用例是否存在、diff 是否来自未列分布 | 重要（决定 Phase 1 价值） | ~~未 probe wpt-data~~ **已验证（R381，2026-06-20）= FALSE** | **gate 结果：A1 FALSE → Phase 1 紧急停止，转 Phase 2**。扫描全 16 个 css-multicol 失败案结构（height / column-fill / 列数 / block 子元素数）：**0/16 匹配** Phase-1 目标（单层+balance+明确高度+纯 inline/blockchildren=0）。每个失败案要么有 block 子元素（baseline-007/008、multicol-collapsing/count-computed/block-no-clip/containing/count-002、column-height-009、abspos-spanner 7+ blockchildren）、要么 height:auto（multicol-columns-001、multicol-fill-000）、要么 column-fill:auto（multicol-fill-auto-001 sentinel、breaking-004/005/006/nobackground-004）。与 R319（纯 inline 迁移零增益）+ R317（放宽 height_auto 门控回归 -5）一致。**结论：Phase 1（pure-inline balance）无目标案，零杠杆；真实 multicol 失败全需 Phase 2（嵌套/breaking/混合碎片化）= 多会话硬核** |
| A2 | 行盒平衡高度分布对明确高度容器是否与 chromium 一致 | 重要 | 未单用例验证 | Phase 1 实施时逐用例 z_vs_chr 验证 |

---

## 11. 修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v1.0 | 2026-06-19 | 初始版本（R319，spec-rfc 完整模式，自主产出） |
| v1.0-gate | 2026-06-20 | **R381 gate 验证：§10 A1 = FALSE（0/16 失败案匹配 Phase-1 目标结构）→ Phase 1 紧急停止，转 Phase 2 设计**。扫描全 16 css-multicol 失败案（height/column-fill/blockchildren）：全有 block 子元素或 height:auto 或 column-fill:auto 或 breaking/嵌套。Phase 1（pure-inline balance）零杠杆；真实 lever = Phase 2（嵌套/breaking/混合碎片化，多会话硬核）。spec 自身 §10 协议「A1 不存在→紧急停止转 Phase 2」生效 |

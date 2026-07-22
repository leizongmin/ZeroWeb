# RFC: multicol block-level fragmentation（子块跨列分片）

**日期**：2026-07-23
**状态**：草案（rally autonomous，待实施会话 review）
**作者**：rally agent
**相关**：R1862（characterization + deferred）、R383（mixed-content deadlock 史）、master.md

---

## 0. 执行摘要

- **一句话目标**：让 multicol 容器能下降进**单个子块**，把其块级后代（含 forced break）跨列分布，并按列片段画子块背景——修 multicol-fill-auto-004 等 ~2-3 案。
- **本期范围**：仅「multicol 唯一直接子是**纯块**（无 inline 混合、无 column-span、非 monolithic）且其后代含 forced column break」的子情形。
- **明确排除**：mixed inline+block 内容（R383 Phase-A-blocked）、column-span:all spanner（已独立路径）、wide-child 横向跨列（columnfill-auto-max-height-003，另一独立子 bug）。
- **核心约束**：可回退切片 + kill-switch + 全量 css-multicol A/B 守 net≥0；不破坏现有 direct-children 模型（gated 增量路径）。
- **推荐方案**：方案 B（layout-time 单子块「透明展开」：检测单子块 + 后代 forced break → 把后代当 multicol fragmentable units 分配列，paint 按 column_span_offsets 每列画子块背景）。
- **首个落地步骤**：在 `layout_multicol`（multicol.rs:633）加「单子块展开」检测 gate（env ZW_MULTICOL_BLOCKFRAG=1 default-off），先用 multicol-fill-auto-004 做 A/B 看是否 flip，再扩 gate。

---

## 1. 背景

### 1.1 问题（R1862 empirical 实证）

`multicol-fill-auto-004`：`<div columns:5 column-fill:auto><div bg:green><div h:10/><div break-before:column h:10/>×3<div break-before:column h:100/></div></div>`。

ZW LAYOUT_DUMP 实测：5 个 forced-break 子全堆在 1 个 12px 列（green div 12×140 strip）；像素采样 100×100 区 ZW RED=8800/GREEN=1200 vs CHR RED=0/GREEN=10000（纯绿）。

**根因**：ZW multicol fragmentation 仅作用于 multicol 容器的**直接子**（`layout_multicol` 收集 `container.children` + `forced_breaks`，multicol.rs:675-690）。本案 multicol 唯一直接子是 green div（无 break-before），forced breaks 在 green div 的子（multicol 孙元素）→ ZW 看不到 → green div 当原子单元入 col1。

chromium LayoutNG 下降进 green div，分片其内容到 5 列，并在每列片段画 green div 背景（故纯绿 100×100）。

### 1.2 目标

- 让 multicol 对「单子块 + 后代 forced break」子情形正确分片。
- 业务目标：css-multicol oracle 通过率 184→187（+2-3）。
- 用户目标：CJK/文档类页面 column-fill:auto 容器内嵌套内容块正确分列（product-visible，非仅 reftest）。

### 1.3 范围边界

- **在范围内**：multicol 唯一直接子是纯块（display:block / flow-root，无 inline 文本混合、无 column-span:all、非 overflow!=visible monolithic），其后代含 forced column break 的子情形。
- **不在范围内**：mixed inline+block 子（R383 Phase-A-blocked）；多直接子（现有 direct-children 模型已工作）；column-span spanner（独立路径）；wide-child 横向跨列（columnfill-auto-max-height-003 独立子 bug）。

---

## 2. 现状分析（相关代码）

| 路径 | 职责 | 本 RFC 关系 |
|------|------|------------|
| `crates/layout-engine/src/multicol.rs:633 layout_multicol` | 收集 `container.children` + `forced_breaks`，分配列 | 主改点：加「单子块展开」gate |
| `multicol.rs:1096 assign_children_to_columns_with_breaking` | forced break 感知的列分配 | 复用（对展开后的后代） |
| `multicol.rs:1328 position_multicol_children` | 定位子到列 + column_span_offsets | 扩展：子块背景按列画 |
| `crates/engine/src/paint/painter/text/text_multicol.rs` | multicol paint（列 + column-rule） | 可能需扩：子块背景每列片段 |

现有模型：每个直接子是原子 fragmentable unit（whole-child 入一列，除非 column-breaking 拆同一子）。**无「下降进子块分片其后代」机制**。

---

## 3. 设计选项

### 方案 A：postprocess「子块重父化」（reparent）

检测单子块情形，把其后代「重父化」为 multicol 直接子（临时），分配列后再还原。

- ✅ 复用现有 assign/position 逻辑（对重父化后的子）。
- ❌ 高风险：重父化破坏 LayoutBox 树结构（green div 失去子），影响后续 paint / sibling / margin-collapse。blast radius 大。
- ❌ green div 背景画在哪？重父化后 green div 空盒，背景丢失。

**决定**：❌ 拒绝（结构破坏 + 背景丢失）。

### 方案 B：layout-time「透明展开」（推荐）

在 `layout_multicol` 检测「单子块 + 后代 forced break」gate 命中时，把该子块的**后代**当 multicol 的 fragmentable units 收集（而非子块本身），分配列；定位时为每个后代设 `column_span_offsets`（同现有 breaking 机制），并**额外**为子块生成「每列背景片段」记录（子块 background 按其每列内容片段的 extent 画）。

- ✅ 不破坏树结构（子块仍是直接子，后代仍在子块下）。
- ✅ 复用 `column_span_offsets` + `assign_children_to_columns_with_breaking`。
- ⚠️ 子块背景每列画：需 paint 知道子块在每列的内容片段 extent（用其后代的列分配推导）。新增 paint 逻辑。
- ⚠️ gate 须严格（仅单子块 + forced break + 纯块），避误触现有 direct-children 路径。

**决定**：✅ 选定（结构安全 + 复用既有机制，新增 paint 逻辑可控）。

### 方案 C：taffy-local fork 加 block fragmentation

在 taffy-local 给 multicol 节点加 native block fragmentation（容器从子块后代推导列）。

- ❌ taffy fork 改动深、维护成本高、blast radius 全布局。
- ❌ over-engineering（本子情形不需动 taffy 内核）。

**决定**：❌ 拒绝。

---

## 4. 关键风险与 tractability 评估

### 4.1 是否 Phase-A-blocked？（区别 R383）

R383 deadlock 是 **mixed inline+block** 内容（inline spans→block 盒按原子分配，须 Phase A 行盒度量统一）。本 RFC 子情形是**纯块**（green div + 块后代），**无 inline 内容**，故**不依赖 Phase A**。✅ 可独立推进。

### 4.2 子块背景每列画（主要技术风险）

chromium 实测 green div 背景填满每列**全高**（100×100 纯绿，非仅 behind 后代）。机制：fragmented box 的 background 在每列片段画**片段 extent**（CSS Fragmentation §碎裂背景）。

- 若 green div 在某列的内容片段仅 10px（child1），chromium 仍画**全列高**背景？empirical 实测是（纯绿）。这超出「片段 extent」=「内容 extent」——是 green div 作为 fragmentable box 的背景 stretch 到列高。
- ⚠️ 此行为 subtle（fragmented bg stretch to column height），ZW paint 需新逻辑：子块背景 fill 每列**列高**（非内容 extent）。若实现错（按内容 extent）→ 仍 red 露出 → 不 flip。
- **缓解**：先实现「按内容 extent 画背景」（简单），A/B 看是否够；若不够再加「stretch to column height」（第二切片）。

### 4.3 gate 误触现有路径

`layout_multicol` 是所有 multicol 的主路径。gate 须严格排除：多直接子 / column-span / monolithic / inline 混合 / 无 forced break 的单子块。否则回归现有 direct-children 行为。

- **缓解**：env kill-switch（ZW_MULTICOL_BLOCKFRAG default-off）+ 结构签名 gate（单子 + 子.is_block_level + 无 spanner + 后代含 BreakValue::Column）+ 全量 css-multicol A/B 守 net≥0。

---

## 5. 实施计划（可回退切片）

| 切片 | 范围 | gate | 验证 | 预期 |
|------|------|------|------|------|
| Slice 1 | `layout_multicol` 加「单子块展开」检测 + 后代当 fragmentable units 分配列 + 后代 column_span_offsets | ZW_MULTICOL_BLOCKFRAG=1 default-off，结构签名 gate | multicol-fill-auto-004 A/B（layout 分列正确？） | green div 后代跨 5 列，但背景可能仍仅 behind 后代（非全列高） |
| Slice 2 | paint 子块背景每列片段（先按内容 extent） | 同上 | A/B css-multicol 全量 net≥0 | 可能部分 flip（若 chromium 按内容 extent） |
| Slice 3 | paint 子块背景 stretch to column height（若 Slice 2 不够） | 同上 | A/B + product-smoke | multicol-fill-auto-004 flip 到 near-pass |

每个切片：kill-switch + 全量 css-multicol A/B 守 net≥0 + make test + product-smoke。

### 回滚

env ZW_MULTICOL_BLOCKFRAG=0 立即回退到现有 direct-children 模型（零行为变化）。Slice 间独立，可逐切片 revert。

---

## 6. 测试策略

- **单元测试**：`inline_multicol_used_columns` 式纯函数（若抽 helper）——如「检测单子块可展开」判定。
- **reftest A/B**：`make reftest-oracle DIR=css-multicol` 全量 452 案守 net≥0；multicol-fill-auto-004 + 相关案 flip。
- **product-smoke**：welcome / morning-work / wintertc struct-check + diff 不 drift（multicol 在产品页罕见，预期零影响）。
- **make test**：全绿。

---

## 7. 待定（TBD）

| ID | 项目 | 缺失 | 下一步 |
|----|------|------|--------|
| TBD-1 | fragmented bg 是否 stretch to column height（CSS Fragmentation 精确语义） | chromium 行为已实证（纯绿=全列高），但 spec 语义待确认 | Slice 2/3 实测定 |
| TBD-2 | green div 有 padding/border 时背景每列片段的 extent 计算 | 本子情形 green div 无 padding/border | 实施时验，先无 padding/border gate |
| TBD-3 | 单子块展开后，green div 自身高度（应=列内容 max，非 Σ 后代） | layout 须设 green div height = max 列高 | Slice 1 验 |

---

## 8. 裁决

**tractability**：本子情形（纯块 + forced break）**非 Phase-A-blocked**（区别 R383 mixed），可独立推进。主要风险是 fragmented bg stretch to column height（TBD-1），可分切片实证。

**ROI**：~2-3 案 flip（multicol-fill-auto-004 + 可能 columnfill 相关）+ product-visible（CJK vertical/column 文档页）。中 ROI，但是 autonomous structural lever 中最明确者。

**推荐**：Slice 1 起（env default-off，结构签名 gate，multicol-fill-auto-004 A/B），逐切片推进，kill-switch 可回退。

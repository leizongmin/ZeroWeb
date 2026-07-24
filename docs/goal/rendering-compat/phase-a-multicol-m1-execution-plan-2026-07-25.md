# Phase A — multicol M1 执行计划（消灭 Path B 的多列墙 ②）

**日期**：2026-07-25
**性质**：可执行实施计划（Spec + RFC + 切片化增量）。承接 [`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md) §6.4（M1/M2 仅素描，「Phase 3 探针后定」）。
**关联**：[master.md](./master.md) R125/R306/R327/R1985、[blockers-resolution-plan-2026-07-25.md](./blockers-resolution-plan-2026-07-25.md) §2。
**前置裁决**：pre-authorized ruling #4（Phase A 多 session）；本计划把 §6.4 的「探针后定」收敛为「M1 已由排除法选定，须多 session 实施」。

---

## 0. 为什么是 M1（排除法）

Phase A 硬阻塞 = **墙 ② multicol + 换行精度**（[`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md) §2.3/§6.3B）。解墙 ② 的两条路径经多轮裁决收敛：

| 路径 | 裁决轮 | 结论 |
|------|--------|------|
| **M2**（multicol 保 Path B 重跑，仅把空 styles 16px 换真实 font_size） | R125 | ❌ **死锁**——`store_font_sizes 覆盖 -1 / 不覆盖 -1 / 真实 styles -4` 三路全 net-negative（large-font deadlock 谱系 R101/R125/R158）。勿再试。 |
| **Gate 2 盲放宽**（让更多非-multicol 容器进 stored） | R209/R213/R327 | ❌ **不可守**——R327 实证 REF 侧 float 容器（纯 Ahem 多行）切 Path A 产生几何变化，layout 无法区分 test/ref 上下文，multicol-fill-auto-001 pass→fail。勿再盲放宽。 |
| **几何 baseline_y（frag.y+height）作 render y** | R306 | ❌ **证伪**——font-051 探针 `v_offset=frag.height` 渲 16.67% FAIL；geometric baseline ≠ fontdue render baseline。勿再引入几何 baseline 字段。 |
| **line-box metric / inline-block identity 单点 lever** | R1985 | ❌ **ruled-out**——R109 mixed-children 匿名块 = Phase A unification 的 manifestation，非独立可切片。 |
| **M1**（multicol 内层容器照常存 inline_layout；paint multicol 路径消费 stored fragments 做列分配） | 本计划 | ✅ **唯一剩余路径**——M2/Gate2-盲放宽/几何 baseline/metric-lever 全 ruled out 后，M1 由排除法选定。 |

**关键观察（M1 ≠ R327 墙）**：R327 墙是 **Gate 2 盲放宽**（非-multicol 容器进 stored）触发 REF-side float 切 Path A。M1 是 **仅 multicol 内层容器**进 stored + paint multicol 消费 stored——**更窄**，不触及 R327 的非-multicol float 路径。故 M1 可能不撞 R327 墙（须 A/B 实证）。

---

## 1. 现状代码定位

**★ 关键发现（R2039 调查，2026-07-25）：M1 的「auto + inline-only + 定高」子集已被 R900 实现**——非从零开始。

| 位置 | 当前行为 | M1 目标 |
|------|----------|---------|
| `crates/layout-engine/src/inline_finalization.rs:234` `store_inline_multicol_columns` | **R900 已实现**：column-fill:**auto** + inline-only（无 block 子）+ 定高（height/max-height Px）+ 列数≥2 的 multicol 容器，列分布后存 `inline_layout`（line.y=y_in_column，fragment.x += col_idx×(col_w+gap)）；paint `use_stored` 消费。R905 max-height budget、R1429 overflow 列、R1423 填 text_node_is_ahem 度量（供 balance 重跑路径）。**+1 oracle 零回归**。 | ✅ 已 LANDED（auto+inline-only+定高 子集） |
| 同上，**balance 模式**分支（`!info.sequential_fill`，line 281 `return false`） | R902/R1422 实证 balance 存列分布 **net-negative** → balance 仅填度量不存分布，paint 用 multicol_info 重跑 | ❓ M1 剩余：balance 存分布（须克服 R902/R1422 net-negative，难） |
| 同上，**block-children** 分支（`has_block_child`，line 264 `return false`） | block 子走 `multicol.rs assign_children` 路径，不经本函数 | ❓ M1 剩余：block-children multicol 存分布（新机制） |
| 同上，**不定高 auto**（`available_height<=0`，line 294） | 回退不存 | ❓ M1 剩余：不定高 auto（须先解 height，单独 lever） |
| `painter/text.rs:609` `use_stored = multicol_info.is_none() && ...` | multicol 走 Path B；但 R900 stored 的容器 `inline_layout.is_some()` 且... 实际 use_stored 仍 `multicol_info.is_none()` → **须核验 paint 是否真消费 R900 stored**（R900 注释称「无 paint 改动」命中即按列渲染，须实证 use_stored 真路径） | 核验 + 必要时打通 paint 消费 |
| `painter/text.rs` 文件体积 | **1876 行**（近 2000 上限） | incr2 新增逻辑若超 2000 须先拆分 |

**M1 实际剩余 = broaden R900 至 balance / block-children / 不定高 auto**——auto+inline-only+定高 已完成。balance 经 R902/R1422 已证 net-negative（勿盲目重试）。故 M1 剩余比基线 M1 更难（easy case done，remaining 皆曾 net-negative 或需新机制）。

---

## 2. 目标（Spec / FR）

- **FR-M1-1**：multicol 容器的内层 inline 内容容器在 compute_final 存储 `inline_layout`（含 R817 `baseline_y_abs`）。
- **FR-M1-2**：paint multicol 路径（`multicol_info.is_some()`）消费 stored fragments 做列分配（column width / count / balance），不再用空 styles 重跑 IFC。
- **FR-M1-3**（zero-regression 硬门禁）：scoped reftest `css-multicol` + `css-text` + `CSS2/normal-flow` self-source 通过率 **net ≥ 0**；`make product-smoke` welcome diff **< 20%**；全量 `make test` 绿。
- **FR-M1-4**：env-gated（`ZW_PHASEA_MULTICOL_M1=1`），默认 off，A/B 通过后才 default-on。

---

## 3. 切片化增量（R2039 调查后重排；每片须过 FR-M1-3 三态门禁，净负即回退）

**前提**：incr1（store）+ incr2（paint 消费）的「auto + inline-only + 定高」子集已被 R900 LANDED。下述增量针对 R900 **未覆盖**的子集。

### incr-A：核验 R900 paint 消费真路径（read-only，首步）✅ R2041 已解决（favorable）
- **无矛盾**：`multicol_info`（`painter/text.rs:481`）仅在 `!has_in_flow_children && is_balance_mode && height_auto` 时 Some。`is_balance_mode = column_fill != Auto`（line 476）、`height_auto = height == Auto`（line 480）。R900 case = column-fill:**auto** + **定高** → `is_balance_mode=false` 且 `height_auto=false` → `multicol_info=None` → `use_stored = multicol_info.is_none() && inline_layout.is_some() && width_matches` = **true**（R900 stored）→ **R900 stored 经正常 stored 路径（line 626-668）消费**。`multicol_info.is_none()` gate 正确，非矛盾。
- **M1 当前状态图**（R2041 确立）：
  - balance + auto-height + 无 block 子：`multicol_info=Some` → Path B 重跑（用 R1423 填度量）。❌ 勿重试存分布（R902/R1422）。
  - auto + 定高（R900）：`multicol_info=None` + stored → use_stored=true → ✅ stored 消费。
  - block-children：`has_in_flow_children=true` → `multicol_info=None`，但 `store_inline_multicol_columns` `has_block_child→false` 不存 → `inline_layout=None` → use_stored=false → Path B 重跑。← **incr-B 目标**

### incr-B：broaden R900 至 block-children multicol（M1 真增量）⚠ R2042 评估 = 结构性难
- 现 `has_block_child → return false`（line 264）。block-children multicol 走 `multicol.rs assign_children_to_columns_{balanced,with_breaking,multirow,sequential}`（`multicol.rs` 1961 行）。
- **R2042 关键评估**：block-children multicol 产出的是**逐列 block 放置**（哪个 block 进哪列），与 `store_inline_multicol_columns` 存的 `inline_layout`（Vec<InlineLayoutLine> 行盒）是**不同数据结构**。故 incr-B **非 R900 的简单 broaden**——须新存储「逐列 block 放置」+ paint 消费，属 **block fragmentation（R109/blockfrag 谱系，R1870 Slice1 仅部分解）**。
- **裁决**：incr-B 是独立的 block-fragmentation 多 session 架构工作，非 M1（multicol-Path-A）的单点 broaden。勿以「broaden R900」框架单 session 重试。
- **验证**：若启动，须先走 lei-spec-rfc 设计 block-children 逐列放置存储（与 R1870 Slice1 协调）。

### incr-C：broaden R900 至不定高 auto（须先解 height）
- 现 `available_height<=0 → return false`。不定高 multicol 无列高预算 → 无法顺序填列。
- 增量：不定高时先算 content_height（全宽 IFC 高度）作 budget，再分布。
- **风险**：与 R905 max-height budget 交互；不定高 auto 行为 spec 复杂。

### ❌ balance 模式（勿盲目重试）
- R902/R1422 已证 balance 存列分布 **net-negative**。除非发现新机制克服其回归，勿重试。balance 仍走 paint multicol_info 重跑（用 R1423 填的正确度量）。

### incr-D：default-on + 全量验证（broaden 增量 A/B/C 任一 LANDED 后）
- 跑全量 `make reftest` + `make product-smoke` + `make product-smoke-legacy`。
- **验证**：全量 net ≥ 0；welcome < 20%；legacy struct-check 无新 FAIL。

---

## 4. 风险与回退

- **列分配算法复杂度**：M1 主要风险——stored fragments 列分配须复刻 IFC 重跑的 balance/fill 几何。若 incr2 发现列分配须大改，可能拆 incr2a/2b。
- **R327 墙复现**：理论 M1 不撞（仅 multicol），但须 incr2 A/B 实证。若复现，记录机制并回退。
- **vertical-mode multicol**：painter/text.rs 已有 WM gate（line 620 `is_vertical`），M1 须保持 WM 一致。
- **回退**：每片 env-gated，净负即回退 git，记录 evidence 到 master.md。

---

## 5. Do-Not-Repeat 清单（勿再以单 session 重试）

1. M2（multicol Path B + 真实 font_size）——R125 死锁。
2. Gate 2 盲放宽（非-multicol 容器进 stored）——R327 不可守墙。
3. 几何 baseline_y（frag.y+height）作 render y——R306 证伪。
4. line-box metric / inline-block identity / per-font metric 单点 lever——R1985/R1206 ruled out。
5. 37-form-controls 的 break_lines.rs:431 改点——R2026/R2027 dead-code-path。
6. **balance 模式 multicol 存列分布**——R902/R1422 net-negative（除非发现新机制）。
7. **R900 的 auto+inline-only+定高 子集**——已 LANDED，勿重复实现。

---

## 6. 成功标准（Definition of Done）

- incr0~incr3 全 LANDED，`ZW_PHASEA_MULTICOL_M1` default-on。
- `css-multicol` self-source 通过率较 M1 前 **net ≥ 0**（目标 +N，解 large-font 死锁子项）。
- 全量 reftest + product-smoke + legacy 三态门禁全过。
- master.md 记录 M1 yield + 任何残余墙。

---

## 7. 下一步（next session）

**M1 tractable 领域已近穷尽**（R2042 评估）：R900（auto+inline-only+定高）已 LANDED；balance（R902/R1422 net-negative）；incr-B（block-children）结构性难（R109/blockfrag，非 R900 broaden）；incr-C（不定高 auto）spec 上 auto+height:auto 退化似 balance（moot，待实证）。故 M1 单点可推进空间小。

**Phase A 真剩余**（M1 之外）：① Gate 2 放宽（让更多非-multicol 容器进 stored）仍被 R327 墙阻塞（REF-side float 切 Path A）——但 R900 已让 auto+定高 multicol 一致走 Path A，**部分缓解** test/ref 分歧（须重评 R327 在 R900 后是否仍硬阻塞）；② 墙 ③ Path A 多行非-Ahem 垂直定位（v_offset 语义）。建议下 session **重评 R327 墙在 R900-后是否仍成立**（R327 是 R900 前 verdict），若缓解则 Gate 2 放宽 + 墙③ 成 Phase A 真前线。

## 8. M1 viable-limit 评估（R2042）

M1（multicol Path A）tractable 子集经 R900 + 本计划四轮调查（R2039-R2042）**近穷尽**：
- ✅ auto + inline-only + 定高（R900 LANDED，+1 oracle）。
- ❌ balance（R902/R1422 net-negative，勿重试）。
- ⚠ incr-B block-children（结构性难 = R109/blockfrag，非 M1 单点）。
- ❓ incr-C 不定高 auto（spec moot，待实证）。
- ✅ incr-A（paint 消费路径，R2041 确认）。

**结论**：M1 作为「消灭 Path B 的多列墙 ②」的 tractable 部分基本完成（R900）。墙 ② 的残余（block-children）实为 block-fragmentation 独立轨。Phase A 前线应转向 Gate 2 放宽（重评 R327 post-R900）+ 墙 ③。

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

| 位置 | 当前行为 | M1 目标 |
|------|----------|---------|
| `crates/engine/src/paint/painter/text.rs:609` | `use_stored = multicol_info.is_none() && inline_layout.is_some() && width_matches`——multicol **永远** Path B | multicol 也走 stored（消费 inline_layout） |
| `painter/text.rs:668+`（Path B `inline_ctx` 构建） | 用 override-maps（`text_node_font_sizes`）重跑 IFC，再按列分配 | multicol 改读 stored fragments 做列分配（不重跑 IFC） |
| `engine.rs` Gate 1/2（compute_final） | multicol 内层容器是否存 inline_layout？**须实证**（incr0 探针） | incr1：确保 multicol 内层容器存 inline_layout |
| `painter/text.rs` 文件体积 | **1876 行**（近 2000 上限） | incr2 新增逻辑若超 2000 须先拆分（按 stored-vs-rerun / multicol 列分配拆子模块） |

**⚠ 文件体积前置约束**：`painter/text.rs` 1876 行，M1 新增「stored 列分配」逻辑（估 +80~150 行）会超 2000。**incr0 前置 = 拆分 painter/text.rs**（run-rules 单文件 ≤2000），把 multicol 列分配 / Path B override-maps 抽到子模块，再开始 M1。

---

## 2. 目标（Spec / FR）

- **FR-M1-1**：multicol 容器的内层 inline 内容容器在 compute_final 存储 `inline_layout`（含 R817 `baseline_y_abs`）。
- **FR-M1-2**：paint multicol 路径（`multicol_info.is_some()`）消费 stored fragments 做列分配（column width / count / balance），不再用空 styles 重跑 IFC。
- **FR-M1-3**（zero-regression 硬门禁）：scoped reftest `css-multicol` + `css-text` + `CSS2/normal-flow` self-source 通过率 **net ≥ 0**；`make product-smoke` welcome diff **< 20%**；全量 `make test` 绿。
- **FR-M1-4**：env-gated（`ZW_PHASEA_MULTICOL_M1=1`），默认 off，A/B 通过后才 default-on。

---

## 3. 切片化增量（每片须过 FR-M1-3 三态门禁，净负即回退）

### incr1：multicol 内层容器存 inline_layout（compute_final 侧）
- 实证当前 multicol 内层是否存（`ZW_DEBUG_IFC` 探针 + LAYOUT_DUMP）；若不存，扩 Gate 让其存。
- **注意**：paint 暂不消费 → 此片单独是「dead data」（违背 code-guidelines「不推测开发」）。故 **incr1 须与 incr2 合并提交**（存 + 消费同片），避免中间态存而不用。incr1 的价值是定位 Gate 改点 + 实证 multicol 内层当前存储状态。
- **验证**：与 incr2 合并后统一 A/B。

### incr2：multicol 内层存 inline_layout + paint multicol 消费 stored 做列分配（M1 核心，与 incr1 合并）
- `painter/text.rs:609` 改 `use_stored` 含 multicol；compute_final 侧确保 multicol 内层容器存 inline_layout（incr1 的 Gate 改点）；paint multicol 路径列分配改读 stored fragments（按 column width 切片，不重跑 IFC）。
- **文件体积**：若本片新增逻辑使 `painter/text.rs` 超 2000 行，**先拆分**（把 multicol 列分配抽到 `painter/text_multicol.rs`），再实施——拆分本身零行为变更，单独 commit + 零回归验证。
- **风险**：列分配算法须与 IFC 重跑产出的列几何一致（balance / fill / column-gap / column-rule）。这是 M1 主要复杂度。
- **验证**：`css-multicol` scoped reftest A/B；multicol-fill-auto-001 须仍 PASS（守 R327 墙——M1 只动 multicol，REF-side float 不受影响，理论不撞墙，须实证）。

### incr3：default-on + 全量验证
- 移除 env gate；跑全量 `make reftest` + `make product-smoke` + `make product-smoke-legacy`。
- **验证**：全量 net ≥ 0；welcome < 20%；legacy struct-check 无新 FAIL（37-form-controls 仍 Phase A 阻塞，不计）。

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

---

## 6. 成功标准（Definition of Done）

- incr0~incr3 全 LANDED，`ZW_PHASEA_MULTICOL_M1` default-on。
- `css-multicol` self-source 通过率较 M1 前 **net ≥ 0**（目标 +N，解 large-font 死锁子项）。
- 全量 reftest + product-smoke + legacy 三态门禁全过。
- master.md 记录 M1 yield + 任何残余墙。

---

## 7. 下一步（next session）

**从 incr1+incr2 合并切片开始**：先用 `ZW_DEBUG_IFC` + LAYOUT_DUMP 实证 multicol 内层容器当前的 inline_layout 存储状态（incr1 探针），再设计与 IFC 重跑几何一致的 stored-fragments 列分配算法（incr2）。若 `painter/text.rs` 因新增超 2000 行，先做零行为变更的拆分（抽 multicol 列分配到子模块）。每片严格过 FR-M1-3 三态门禁。

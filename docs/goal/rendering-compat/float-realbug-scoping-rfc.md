# RFC：float REAL-BUG 簇 scoped 多会话修复（rendering-compat R1608 起）

**版本**：v1.0
**日期**：2026-07-17
**状态**：已确认执行路线（2026-07-17 用户确认：短期按本 RFC 推进；首切片 R1609/FR-001 为 0-code root-cause 诊断）
**起源**：R1607 发现 css/CSS2/floats+floats-clear 是唯一 REAL-BUG（非 font-wall）territory；R1608 诊断 floats-wrap-bfc-006 定位 bug 性质。

---

## 0. 执行摘要

- **一句话目标**：把 css/CSS2/floats + floats-clear 簇的 REAL-BUG（self-source 高 diff，非 font-wall）按子类 scoped 逐 slice 修复，每个 slice 守全量 A/B net≥0 才 land。
- **本期范围**：仅本 RFC（设计 + 首切片 root-cause 任务定义）；不下结论先改代码。
- **明确排除**：font-wall（R1605 deadlock，独立 track）；adjoining-float（R1393 已 LANDED，仅在出现新 case 时 revisit）。
- **核心约束**：① 每 slice 必须 scoped 到结构签名（R1518e V2 方法论），禁全树重跑（R1518 net-2 教训）；② 每 slice 全量 A/B（floats+floats-clear+margin-collapse+tables+css-position 全套），net<0 即 revert；③ root-cause-first，禁止症状修补。
- **推荐方案**：按子类（BFC-wrap / margin-collapse+clear / zero-height-float / abspos+float）分轨，每轨先 root-cause 再 scoped slice。
- **首个落地步骤**：floats-wrap-bfc-006 deep root-cause——LAYOUT_DUMP 追 nested table block 在 float staircase 旁的定位路径，定位「为何不 BFC-avoid / 为何 x≥158 而非 wrap」。
- **当前裁决**：本线作为近期主线推进；font-stack rebuild 仅作为独立战略 RFC/one-pager 准备，不与本 RFC 混线实现；Taffy 本地 fork 清理不阻塞本线。

---

## 1. 背景与目标

### 1.1 背景

R1605 core reftest ≥95% 定性为 conclusive font-wall deadlock（全角度穷尽）。R1607 扫描 css-text-decor + css-float 时发现：**float 簇是 REAL-BUG territory**——`floats-wrap-bfc-006` self-source（ZW-test vs ZW-ref）= **15.32%**（73528px，max channel diff=255 纯黑白），与 font-wall case（如 R1604 flex-flow-001 self-source 1.03% ZW 自洽）根本不同。**ZW 把 test 和它自己的 ref 渲染得不同 = REAL ZW layout bug**，非字体噪声。这是唯一非 font-wall 的 real-yield 方向。

### 1.2 目标

- 业务目标：提升 css/CSS2/floats+floats-clear oracle 一致率（当前 DIR=CSS2/floats oracle-pass 148/327=47.1%，strict 20=6.4%），贡献 core reftest 真通过率（非 font-wall 增量）。
- 工程目标：把 float REAL-BUG 簇拆成可逐 slice land 的 scoped 修复，避免 R1393 式 10 轮 thrash。

### 1.3 范围边界

- **在范围内**：css/CSS2/floats/* + css/CSS2/floats-clear/* 的 REAL-BUG（self-source 高 diff）；float 定位 / clear / BFC-avoid / margin-collapse×float 交互的 ZW 侧修复。
- **不在范围内**：font-wall（独立 track）；adjoining-float（R1393 已解，新 case 才 revisit）；vertical-mode float（R109/vertical blocked）；multicol fragmentation 内的 float（Phase 2）。

---

## 2. 簇分类与代码映射（R1608 诊断成果）

DIR=CSS2/floats（327 案）top-15 worst 全结构性，按子类分：

| 子类 | 代表 case | self-source diff | ZW 代码区 | R1608 诊断 |
|------|-----------|------------------|-----------|-----------|
| **BFC-wrap**（block/table 不避 float） | floats-wrap-bfc-005/006 / floats-wrap-bfc-with-margin-001/001a/002 | 006=15.32% | `float_positioning.rs` + block/table 布局 float-exclusion | ★ 006 实测 table_float_fix ON/OFF **15.32% 完全不变** → bug **不在** R1518e table_float_fix，在更基础路径（nested table block 定位不 consult float exclusion） |
| **margin-collapse + clear/float** | margin-collapse-157/142/121/125 / clear-on-parent-with-margins | 157=20.20%（最高） | taffy CollapsibleMarginSet + `float_positioning` clear 吸收（R1393） | clear + 负 margin + float 嵌套，R1393 adjoining-clearance 之外的新组合 |
| **zero-height / 空 float** | float-006 | — | `float_positioning` zero-margin-box float 处理 | 「零高空 float 不缩短 line box，滑入 line box 顶」（CSS2 §9.5） |
| **abspos + float** | new-fc-beside-adjoining-float / float-006（green-overlapping-abs-pos） | — | abspos §10.3.7 + float 交互 | R1393 territory 边缘 + abspos shrink-to-fit |
| **floats-wrap-top-below-inline** | 002l/002r | 8.56/7.93% | inline float exclusion（inline_finalization） | inline 文本绕 float 顶/底定位 |

**关键诊断结论（R1608）**：
1. floats-wrap-bfc-006 self-source 15.32% **与 table_float_fix（R1518e）ON/OFF 无关** → BFC-wrap bug 在基础 float/block 布局，非 R1518e scoped pass 可解。
2. PIL 定位：floats（blue）test/ref **完全一致**（9300px 同 bbox x[8,157]）→ **float staircase 本身渲染正确**；发散在 **nested table（purple）+ caption（yellow）**：test table x[158,387] y[8,217] vs ref x[78,307] y[54,266]——**nested table block 未正确 BFC-avoid float**（ZW 把 table 放 float 列右侧 x≥158，ref 应 wrap 到 x≥78）。
3. 全簇 top 案无一是 clean 单 session flip（BFC-wrap / margin-collapse+clear / zero-height-float / abspos+float 均结构性）。

---

## 3. 功能需求（按 slice）

### FR-001：floats-wrap-bfc-006 deep root-cause（首切片，diagnosis-only）

- **描述**：必须先用 LAYOUT_DUMP + 代码追踪定位 nested table block 在 float staircase 旁的定位路径，回答「为何 table block x=158 而非 BFC-avoid wrap」，产出根因写入 evidence，**不改代码**。
- **优先级**：必须（首切片）
- **验收**：evidence 文件给出「table block 定位路径 + 与 ref 期望的差异点 + 候选修复 seam（gate 到结构签名）」。

### FR-002：BFC-wrap scoped slice（次切片，code）

- **描述**：基于 FR-001 根因，在 `float_positioning.rs` 或 block 布局加 **scoped** BFC-avoid（gate 到「in-flow block/table 子 + 同容器 float 子 + 无 clear」结构签名），使 block/table border-box 不与 float 重叠（CSS2 §9.5）。
- **优先级**：应该
- **验收**：floats-wrap-bfc-005/006 self-source diff 显著下降；全量 A/B（floats+floats-clear+margin-collapse+tables+css-position+normal-flow）net≥0；kill-switch（`ZW_BFC_WRAP_AVOID=0`）；load-bearing 单测。

### FR-003：margin-collapse+clear scoped slice（后续）

- **描述**：margin-collapse-157/142/121/125 的 clear+负margin+float 组合，scoped 到结构签名修复。
- **优先级**：可以（BFC-wrap 解后再排）

---

## 4. 非功能需求

- **NFR-001 回归安全**：每个 code slice 全量 A/B net≥0 才 land；net<0 立即 revert 并 evidence 记录 ruled-out 角度（类 R1593/R1594 multicol net-negative 收口）。
- **NFR-002 scoped**：禁全树重跑 float 定位（R1518 net-2 + R1393 margin-collapse 回归教训）；每 slice gate 到结构签名。
- **NFR-003 kill-switch**：每 code slice 配 env kill-switch default-on，可 A/B 可回退。

---

## 6. 约束与假设

### 6.1 必须约束
- 每 code slice 全量 A/B（`make reftest` self-source 全量 + `make reftest-oracle DIR=CSS2/floats` + DIR=css-tables/css-position/normal-flow）。
- root-cause-first（FR-001 先于 FR-002）。
- 跑测试用 `make test` / `make reftest`（test-guard 包裹），禁裸跑 cargo。

### 6.2 禁止约束
- 禁全树 `adjust_float_positions` 重跑（R1518 net-2）。
- 禁 font-wall 角度混入（本 track 只解 REAL-BUG 非 font-wall）。

### 6.5 假设
- float staircase 渲染正确（R1608 PIL 实测 blue test/ref 一致）→ bug 在 block/table 对 float 的反应，非 float 自身定位。**状态：已验证（R1608）**。
- floats-wrap-bfc-006 bug 不在 table_float_fix（R1518e）。**状态：已验证（R1608 ON/OFF 实测 15.32% 不变）**。

### 6.6 代码变更边界
- **允许修改**：`crates/layout-engine/src/float_positioning.rs`、`crates/layout-engine/src/engine.rs`（block/table 布局 float-exclusion 接入点）、相关 tests。
- **禁止修改**：`table_float_fix.rs`（R1518e，已证 006 不涉及，勿改避免回归 table-among-floats-001）；font/raster 路径（font-wall 独立 track）。

---

## 7. 实施交接

### 推荐修改顺序
1. **FR-001（首切片，下轮）**：LAYOUT_DUMP floats-wrap-bfc-006，追 nested table（NodeId）从 build → layout → locate 的 y/x 决策路径；对比 ref 期望；定位「block/table 子是否 consult 同容器 float exclusion」；产出 root-cause evidence + 候选 seam。
2. **FR-002**：在定位到的 seam 加 scoped BFC-avoid（结构签名 gate），A/B，net≥0 land / net<0 revert。
3. **FR-003**：margin-collapse+clear 簇同理 scoped。

### 首批提交建议
| Commit | 范围 | 预期 | 验证 |
|--------|------|------|------|
| R1609 | FR-001 root-cause evidence（0 code） | 定位 006 bug seam | evidence + master.md |
| R1610+ | FR-002 scoped BFC-avoid slice | floats-wrap-bfc-005/006 self-source 降，全量 net≥0 | make reftest + reftest-oracle DIR=CSS2/floats + 全量 A/B |

---

## 8. RFC 关键决策

- **为何 scoped 而非 broad**：R1518 全树 adjust_float_positions net-2（margin-collapse 回归）；R1393 adjoining-float 10 轮 thrash；R1518e V2 scoped 才 net+1 land。float 改动 ripple 经 margin-collapse/table/BFC 全套件，必须 gate 到结构签名。
- **为何 root-cause-first**：R1608 已证 006 bug 不在 table_float_fix（直觉 seam 错误）；盲改 high-risk。FR-001 强制先定位真 seam。
- **为何不改 table_float_fix**：R1608 ON/OFF 实测 006 与之无关；改它有 table-among-floats-001 回归风险且不解 006。

---

## 9. Spec Lint（自检）

- 执行摘要 ✅ | 场景存在性 ✅（FR-001/002/003 各有验收）| 异常路径 ✅（net<0 revert）| 测试绑定 ✅（make reftest/reftest-oracle）| TBD 清零 ✅ | 实施交接 ✅（修改顺序+首批提交）| 首步可执行 ✅（FR-001 LAYOUT_DUMP）| 范围冲突 ✅（font-wall/adjoining-float 明确排除）| 实现来源 ✅（代码区映射 §2）。
- **门禁**：Fail=0，允许作为 rally 设计基线推进。

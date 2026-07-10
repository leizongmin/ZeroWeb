# Inline-Box-Model + Float 统一设计（R109 / floats-006 架构 RFC）

**日期**：2026-07-10（R1275，承接 R1270-R1274 五轮 hands-on 实证）
**状态**：设计文档（read-only，本轮不落地代码）
**关联**：与 [`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)（R306，IFC 三路径统一 / large-font bug）**正交**——本文档覆盖 inline→block 映射 + float 定位 + height + IFC exclusion + paint text **五方耦合**，后者覆盖 layout/paint IFC 字体一致性。

---

## 0. 执行摘要

- **一句话目标**：让 BFC 容器内的纯 inline 级内容（`<span>`/文本）正确参与父 IFC 排版并绕 float 流动，而非被 `convert_display` 映射为 taffy Block 占据垂直空间——解锁 floats-006 等整簇「inline 内容与 float/height 交互」缺陷。
- **本期范围**：仅产出设计文档 + 分阶段实施计划（含五方协调的精确触点），**不落地代码**。
- **明确排除**：IFC 三路径字体一致性（由 phase-a-IFC-unification-design.md 覆盖）；multicol 碎片化（独立轨道）；vertical writing-mode（R1090+ 独立）。
- **核心约束**：① 任一阶段**零 count 回归**（项目硬标准）；② 单文件 ≤2000 行；③ 不得以「改变布局语义」方式重排 glyph（DC-13）。
- **首个落地步骤**：Phase 1 = `compute_final_inline_layouts` 已为含直接文本/inline 子的 BFC 容器建 IFC（R1270 实证 floats-006 的 div1 经 whitespace 已有 IFC + float exclusion 已具备），故 Phase 1 聚焦 **paint text 路径**——让 pure-inline 子（`is_block_level=false`）的文本**只**经父 IFC 渲染（float-excluded），paint_text 跳过其直属文本避免双绘/错位。

---

## 1. 背景与五方耦合（R1270-R1274 实证）

floats-006.xht：`#div1`(300×**200**) 含 `<span>X</span>`(inline orange 100px Ahem) + 2× `<div class=class1>`(float:left 100×100 blue)。期望 = 2 blue float 占 top x=[0,200]、X 绕到 x=[200,300] top-aligned。

ZW bug 经 4 轮 hands-on（R1270-R1274）定位为**五方耦合系统**，单点修任一方破其他方补偿：

| # | 子系统 | 当前行为（bug） | 期望行为 | 单点修后果 |
|---|--------|----------------|----------|-----------|
| ① | **convert_display**（converter/mod.rs:329） | `Inline→taffy::Block`：span 成全宽 300×100 block 占 top | span 为 inline 内容参与父 IFC，不占独立 block 空间 | tree-builder fold 风险高（破 R255 morning.work 等大量 inline→block 依赖） |
| ② | **float_positioning flow_bottom**（float_positioning.rs Phase1 L372 + Phase2 L646） | inline 级子推进 flow_bottom → float 被 span block 推到 rel_y=100 | inline 级不推进 flow_bottom（CSS §9.5.1） → float 上提到 rel_y=0 | R1272 实证：floats-006 11.54→4.79 改善但 floats-clear **NET -3**（破其他案 span-block 补偿）+ R1273 塌缩 div1 高度 |
| ③ | **IFC float exclusion**（inline_finalization.rs:770-809 `with_float_exclusions`） | **已具备**，但被 float 位置（②）+ 文本渲染路径（①'）架空 | float 在 rel_y=0 时正确缩减 X 行盒可用宽 → X 落 x=[200,300] | 已就绪，依赖 ②（float 位置）+ ①'（文本经父 IFC） |
| ①' | **paint text 路径**（painter/mod.rs:507/706 `paint_text`） | paint_text 对**所有** box（含 is_block_level=false 的 span）绘直属文本 → X 绘于 span 位 x=8（非父 IFC float-excluded x=208） | pure-inline 子的文本只经父 IFC 渲染，paint_text 跳过 | R1274 新增层；未单独测，但 R1272 残余 4.79% 推断为此 |
| ④ | **height 计算** | div1（height:200 显式）在 ② 应用后塌缩到 100（R1273）；exclude_floats 已跳过显式高度（postprocess.rs:587 已守卫），故塌缩来自**其他** post-process（prevent_collapse / clamp_pct_height / backfill_r109 / compute_final 之一），源未定位 | 显式高度容器不受 float 重定位影响 | R1273 ④=exclude_floats 假设证伪（R1274）；真正源待 trace |

**关键教训（R1274）**：**layer-peeling 非收敛**——逐个 fix 一方 A/B 必破他方补偿。必须按下方 Phase 协调交付。

---

## 2. 分阶段协调实施计划（核心：任一阶段须全门禁绿 + scoped A/B ≥0）

> **铁律**：不允许单方 patch 上 default（R1272/R1273 实证 net-negative）。每 Phase 须 `ZW_*` env-gated 同码 A/B（floats-clear + welcome 守零回归），全绿且 net ≥0 才能 default-on。

### Phase 1（首落地）：paint_text 跳过 pure-inline 子直属文本（①'）—— ★ R1276 证伪（no-op）

- **R1276 A/B 实证**：paint_text 入口守卫（`ZW_PAINT_SKIP_INLINE=1`，is_block_level=false 跳过）
  对 floats-006 **零效果**（11.54%==11.54%），floats-clear **84==84 完全一致**，welcome 81456→81427
  （0.01pp 噪声）。**①'（paint_text 双绘）非真实机制**——paint_text 对 pure-inline box 本就不绘文本
  （X 已由父 IFC 渲染）。设计文档原 Phase 1 作废。
- **★ 模型收窄**：floats-006 真实 lever = **②（float 位置）+ ④（height）**，③ IFC exclusion **已正确**
  （仅被 ④ height 塌缩干扰 IFC 计算架空）。**① convert_display + ①' paint_text 均非直接 lever**
  （① 是根因架构但 ②④ 可绕过；①' 证伪）。R1275 五方 → **三方（②④③）**。
- **新 Phase 1**：trace ④（height 塌缩源）——见下方 Phase 3（升为新首落地）。

### Phase 2：float_positioning inline 级不推进 flow_bottom（②，复用 R1272 代码）

- **触点**：`float_positioning.rs` `adjust_float_positions_with_context` Phase1（L372 条件）+ Phase2（L646 flow_bottom 更新），加 `lift_inline` gate（R1272 已实现代码，git 历史可复用）。
- **前置**：Phase 1 已落地（①'），否则 ② 单独 net -3（R1272 实证）。
- **A/B**：`ZW_FLOAT_LIFT_INLINE=1`（与 Phase1 `ZW_PAINT_SKIP_INLINE=1` 同时 on），floats-clear ≥0 + floats-006 flip。

### Phase 3：height 塌缩源定位 + 修复（④）—— ★ R1276 升为新首落地

- **触点**：trace div1.height 在 ② 应用后由 200→100 的具体 post-process（prevent_collapse_through_min_height / clamp_percentage_max_height / backfill_r109_anon_block_heights / compute_final_inline_layouts 之一），加守卫跳过 definite-height 容器。
- **方法**：② 应用时（`ZW_FLOAT_LIFT_INLINE=1`）在 engine.rs 各 post-process 入口插 div1.height 探针，二分定位塌缩点。exclude_floats 已排除（postprocess.rs:587 已守卫 is_auto_height）。
- **★ 为何升为首落地（R1276）**：①' 证伪后，floats-006 = ②+④。② 单独 4.79%（非 flip）因 ④ 塌缩
  干扰 IFC exclusion；② 单独 floats-clear -3 疑亦含 ④ 塌缩副作用（待证）。先 trace+修 ④，再 ②+④
  联合 A/B，可能 net ≥0（若 -3 主因是 ④）。
- **A/B**：②+④ 同时 on，floats-006 应 flip（<1%）+ floats-clear ≥0 + welcome 守门禁。

### Phase 4（可选/defer）：convert_display span-fold（①，最深）

- **触点**：`converter/mod.rs:329` + `tree.rs build_subtree`——pure-inline-leaf 不生成 taffy block 节点，文本注入父 IFC。
- **风险最高**：破大量 inline→block 依赖（R255 morning.work）。仅在 Phase1-3 不足以解 floats-006 或整簇时才做。多 session。

---

## 3. 实施交接（Implementation Handoff）

### 文件/模块清单

| 路径/模块 | 动作 | Phase | 风险 |
|----------|------|-------|------|
| `crates/engine/src/paint/painter/mod.rs:507,706` | paint_text 守卫跳过 pure-inline 直属文本 | 1 | 低（paint-only，零布局变更） |
| `crates/layout-engine/src/float_positioning.rs:372,646` | lift_inline gate（R1272 代码复用） | 2 | 中（破补偿，须 Phase1 前置） |
| `crates/layout-engine/src/engine/postprocess.rs` 或 engine.rs post-process | height 塌缩源守卫（definite-height skip） | 3 | 中（源待 trace） |
| `crates/layout-engine/src/converter/mod.rs:329` + `tree.rs` | span-fold（pure-inline 不生成 block） | 4（defer） | 高（破 inline→block 依赖） |

### 推荐修改顺序

1. **Phase 1 先行**（paint_text 守卫）——零布局风险，验证「文本经父 IFC」渲染正确性。
2. Phase 1 全绿 + A/B ≥0 → default-on → Phase 2（②，复用 R1272 代码）。
3. Phase 1+2 A/B → 若 floats-006 未 flip，trace ④（Phase 3）。
4. Phase 1-3 仍不解 → Phase 4（span-fold，多 session）。

### 首批提交建议

| Batch | 范围 | 预期 | 验证 |
|-------|------|------|------|
| Phase 1 | paint_text pure-inline 守卫（env-gated） | floats-006 paint 路径验证（X 经父 IFC） | make test + floats-clear/welcome A/B ≥0 |

---

## 4. 假设 / 待验证（TBD）

| ID | 假设 | 状态 |
|----|------|------|
| A1 | div1（floats-006）的父 IFC（经 whitespace 建立）确实 collect 了 span 的 X 且 float exclusion 正确（仅被 float 位置 + paint 路径架空） | 待 Phase 1 验证（paint 守卫后 X 是否落 IFC 位） |
| A2 | Phase 1+2+3 协调能 flip floats-006 而无 floats-clear 回归 | 待实证（R1272 单 ② = -3，协调后未知） |
| A3 | height 塌缩源在 4 个 post-process 之一（prevent_collapse/clamp_pct/backfill_r109/compute_final） | 待 Phase 3 trace |
| A4 | paint_text 守卫「文本已参与父 IFC」的判定（display:Inline + 父有 stored inline_layout）不误杀合法 inline 文本 | 待 Phase 1 scoped A/B |

---

## 5. 与现有设计文档的关系

- **`phase-a-IFC-unification-design.md`（R306）**：覆盖 IFC **三路径**（layout compute_final / paint Path A / paint Path B）字体一致性（large-font bug）。**正交**于本文档。
- **本文档**：覆盖 inline→block **映射**（convert_display）+ float 定位 + height + IFC exclusion + paint text **五方耦合**。
- 两者可独立推进；floats-006 主要由本文档覆盖（inline-box-model + float 交互），large-font 由 R306 覆盖。

---

## 6. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2026-07-10 (R1275) | 初始：R1270-R1274 五轮实证 → 五方耦合定位 + 分阶段协调计划 |
| v1.1 | 2026-07-10 (R1276) | Phase 1（①' paint_text）A/B 证伪（no-op：floats-clear 84==84）→ 模型收窄五方→三方（②④③，①' 非机制，③ 已正确被 ④ 架空）；Phase 3（④ height trace）升为新首落地 |

# RFC（rally-pattern 设计草案）：nested multicol fragmentation（内层 multicol 跨外层列碎片化）

**版本**：v0.1
**日期**：2026-07-16（R1509）
**模式**：**rally-pattern 设计文档**（非 `lei-spec-rfc` skill —— 该 skill 需用户确认，与无人值守 rally CONTINUE/DONE/BLOCK 协议冲突；见 master.md R896 + `multicol-phase2-column-fragmentation-context.md` 同款约定）
**状态**：分析 + 架构提案；待多 session 分阶段实施
**目标案**：`multicol-breaking-001/002/003/004/005/006` + `multicol-breaking-nobackground-{000..005}`（13 案，oracle 3.16-9.58%，css-multicol 唯一剩余 reftest-validatable lever）
**关联**：
- [`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md) §R201（三 blocker A/B/C，2026-06-17 dump 实测）
- [`multicol-phase2-column-fragmentation-context.md`](./multicol-phase2-column-fragmentation-context.md)（Phase 1/2a 已 DONE：inline-only + single-level block breaking）
- master.md R1351（painter-core 实验 net -1 REVERTED）、R1352-R1361（九轮深调）

---

## 0. 执行摘要

- **一句话目标**：让外层 multicol（`column-fill:auto` + 定高）在跨列拆分时，正确碎片化其**自身也是 multicol** 的子元素（内层 multicol 的 2 列布局按外层列高垂直切片，每片在外层一列内渲染内层完整 2 列 + `box-decoration-break:clone` 装饰）。
- **本期范围**：架构 + 分阶段实施计划；首个 enabling slice 须 **net ≥ 0**（紧 env gate + 全量 css-multicol A/B）。
- **核心约束**：① 不破坏已 DONE 的 inline-only（`store_inline_multicol_columns`）/ single-level block breaking（`assign_children_to_columns_with_breaking`）；② 不重蹈 R1351 painter-core net-negative（cso 传播破坏 deep-nesting）；③ A/B 净负即整体 revert。

---

## 1. 问题（R1509 PIL 实测，multicol-breaking-004）

**结构**：`.outer`(h:125, column-count:4, column-fill:auto, rule:4px blue, gap:16, w:800) > `.inner`(column-count:2, rule:2px fuchsia, gap:16, h:300, border-bottom:25px green, box-decoration-break:clone, 17 行文本)。

**chromium（ref）**：inner 的 17 行经**两级碎片化**分布到 outer 4 列——每 outer 列（188px 宽）渲染 inner 的 **2 个子列**并排（左 ~56px + 右 ~59px，fuchsia rule 居中），文本垂直 cap 在 outer 列高 125px，inner 的 green border-bottom 在每个 outer 列底（`box-decoration-break:clone`，每片都画），outer blue column-rule 在每列间隙。

**ZeroWeb 当前（9.58% FAIL，PIL 实测）**：
| 信号 | ZW | CHR | 含义 |
|------|----|----|------|
| 文本 x-spans | `(8,53)(212,257)(416,461)` 每 outer 列**仅 1 个 ~45px 跨度** | `(8,64)(111,170)(214,276)(315,374)` 每 outer 列**2 个子列** | inner 的 column-count:2 **未生效**——被当单列 |
| 文本 y-range | `2-143`（**溢出 outer 125px 列高**） | `11-102`（cap 在列高内） | 内容未跨 outer 列分布，垂直溢出 |
| green border-bottom | 4385 px | 14100 px（**3.2×**） | clone 装饰未按 outer 列片重复 |
| blue column-rule | 281 px | 1491 px（**5.3×**） | outer rule 大量漏画 |
| fuchsia column-rule | 76 px | 556 px（**7.3×**） | inner rule 几乎全漏 |

**一句话根因**：inner 被外层 `assign_children_to_columns_with_breaking` 当作**原子块**分配到 outer 列，其**自身的 2 列布局从未按 outer 列高切片计算**，导致每 outer 列只渲染 inner 的单列首段 + 垂直溢出 + 装饰/rule 不按片重复。

---

## 2. 根因（代码级）

### 2.1 layout 侧：nested multicol 子被当原子块

`multicol.rs:696-704` `has_nested_multicol_seq` 守卫**显式排除** nested multicol 子走 `assign_children_to_columns_multirow`（列溢出路径），改走 `assign_children_to_columns_with_breaking`（line 712）。后者把每个子当**不可分原子**分配到 outer 列——**不碎片化子的内部内容**。故 inner（自身 multicol）的整体被塞进 outer col0，溢出。

> 注：single-level block breaking（`_with_breaking`）对**普通 block 子**是正确的（block 子无内部列布局须切片）；缺口专指**子自身是 multicol** 时须做二级碎片化。

### 2.2 paint 侧：inner 自身 multicol 布局不计算

`painter/text.rs:870-872`：
```rust
let height_auto = matches!(style.height, LengthValue::Auto);
let multicol_info = if !has_in_flow_children && is_balance_mode && height_auto {
    compute_multicol_info_for_paint(...)
```
inner `height:300`（非 auto）→ `height_auto=false` → gate 不触 → inner 的 2 子列几何**从不计算**（R201 blocker A）。即便计算，paint 的 column loop（`if is_multicol`）也只对 multicol 容器触发，nested 场景下 depth-2 子的碎片化片段不被重绘（R201 blocker B）。

### 2.3 column-rule §5.2 主位置误判（R201 blocker C）

`painter/text.rs` column-rule 内容检测只查 `child.x` 主位置；被碎片化的 inner 只在 outer col0 有主 x → 其余 outer 列误判「无内容」漏画 blue rule。fuchsia rule（inner 自身）因 inner 子列未计算而全漏。

### 2.4 ★ R1509 NESTEDMCDBG 探针精确实测（修正 §2.2 "从不计算"）

探针打点 `store_inline_multicol_columns`（`inline_finalization.rs:202`）入口，实测 004：
```
NESTEDMCDBG node=NodeId(32v1) content_width=188 col_count=2 col_width=86 seq=false children=16   # inner
NESTEDMCDBG node=NodeId(30v1) content_width=800 col_count=4 col_width=188 seq=true  children=1    # outer
```
**关键修正**：inner 的 2 子列布局**确被计算**（`compute_column_info` 出 col_count=2/col_width=86，col_ctx IFC 跑了），且在**正确宽度**（content_width=188 = outer 列宽，非全宽 800——说明 `position_multicol_children` 宽度约束已在 inline_finalization 前生效）。但 `store_inline_multicol_columns:249 if !info.sequential_fill { return false; }`——inner 是 balance（seq=false）→ **计算后即丢弃**，不存 2 子列分布；paint `is_balance_mode && height_auto` gate（height:300 非 auto）又不补算 → inner 最终按**单列**渲染。

**两条 net-negative 先例须同时绕开**：
- **store 侧**：R902/R1422 证 balance multicol **存**列布局 net-negative（top-level balance paint 自路径已工作，存了被忽略/双计）。nested 例外须精确锁「被外层碎片化的 inner」而非所有 balance。
- **paint 侧**：R1351 证 painter cso 传播 net-negative（gate 过宽卷入 deep-nesting）。

**Stage 1 精确化**：nested-only 信号 = 「inner `is_multicol` && balance && 有 explicit height && 其 layout 父是 multicol」。检测时机问题：`store_inline_multicol_columns` 在 inline_finalization 递归中按节点调用，**无 parent 上下文**——须预扫一遍树标记 `is_nested_multicol_child` flag（LayoutBox 新字段），或 inline_finalization 改传 parent。这是 Stage 1 第一个 wiring 决策。

---

## 3. 为何 R1351 painter-core net-negative（不重蹈）

R1351 实现「backfill 复制 cso + painter `paint_as_multicol = is_multicol || any_child_has_cso`」：004a block 双列分布**修对**（-6.25pp），但 `any_child_has_cso` gate **过宽**——把 deep-nesting 的 normal-multicol 路径也卷入 → `remove-transform-descendant-becomes-spanner` 0.63→1.84%（+1.21pp pass→fail）→ NET 155→154 (-1) → 全 revert。

**教训**：painter-side cso 传播对 nested spanner wrapper 有效，但 gate 必须精确锁「is_nested_spanner_wrapper」类信号（不能 `any_child_has_cso`），且须配合 layout 侧的内层子列计算——单 painter 侧补丁无法修 vertical 溢出 + 装饰 clone（须 layout 知道每片几何）。

---

## 4. 架构提案：两级碎片化（layout-driven）

**核心思路**：碎片化在 **layout 期**完成（非 paint 期补丁），使每片几何 + 装饰 + rule 都有确定数据。

### 4.1 数据模型

为 nested multicol 子引入 **per-outer-column fragment**：每个 outer 列片记录
-该片在内层的垂直范围 `[frag_top, frag_top + outer_col_height]`；
- 该片对应的 inner 2 子列布局（文本行分配到 2 子列，受限该片范围）；
- clone 装饰（border-bottom 等）该片是否触底绘制。

### 4.2 ★ R1511 探针实测修正：layout 侧已够，缺口在 paint 侧（new paint path）

**实测结论**（engine.rs pass 顺序：step 9 `adjust_multicol_layout` 在 step 12 `compute_final_inline_layouts` 之前）：
- inner 的 column_span_offsets（outer 碎片化结果）**已在 store_inline_multicol_columns 调用时可用** → nested 检测信号 = `root.is_multicol && !root.column_span_offsets.is_empty()`（无须新字段/预扫，修正 §2.4「须预扫树」）。
- inner 的 2 子列 IFC（col_ctx，col_width=86）也已计算（§2.4）。
- **故 layout 侧无须改动**——inner 的 line 流 + outer 碎片几何都已就位。

**★ 关键阻断 = paint 侧现有 shift+clip breaking 无法渲染 2-subcols/fragment**：
`position_multicol_children` 对每个碎片存 `(child_x, child_y, col_x, col_w, col_top, col_h)`，paint（mod.rs:847-910）按 `frag_offset_y = frag_abs_y − child.y`（child_y 对非首片为负，如 col1 = 0−125 = −125）渲染**整个 child** 再 y-clip 到 `[col_top, col_top+col_h]`。此「整体位移 + 垂直裁剪」模型假定 child 是**单一纵列**；若 inner 行落 2 个 x 位置（2 子列），位移后 y-clip 会把整片裁空（行落在 y[0,100] 位移到 content_y−125..−25，clip [0,125] 落空）。**故「只改 store」行不通**——须 paint 侧识别 nested-multicol 碎片并按「该片 2 子列 + clone 装饰」独立渲染（R1351 painter-side 领域，须精确 gate 避 net-negative）。

### 4.3 正确算法（R1511 从 ref 推导，flow-thread 模型）

ref 实测（004）：outer col0 = inner subcol0(AAAAA-EEEEE 5行) + subcol1(FFFFF-JJJJJ 5行) + clone border-bottom@y100；outer col1 = subcol0(KKKKK-NNNNN 4行) + subcol1(OOOOO-QQQQQ 3行) + clone border。→ **不是切片**（slice 把 340px 单列切 3×125 错），**是 flow-thread 分配**：
1. inner 行流（17行，line_h=20，col_width=86）。
2. 每 outer 列容量 = `2 × floor((col_h − clone_border_bottom) / line_h)`（004: 2×floor((125−25)/20)=2×5=10 行/outer 列）。
3. 行流按 10/outer 列分块，每块拆 2 子列（前 ceil(N/2) 进 subcol0，余进 subcol1）。
4. 每 outer 列：subcol0 行 x=frag_x，subcol1 行 x=frag_x + (col_w/2 + ... 实测 subcol 间距)，y=行内序×line_h；底画 clone border-bottom（green）。
5. outer blue column-rule + inner fuchsia rule 按碎片范围绘。

**实现位置**：新 paint 分支（mod.rs paint_as_multicol loop 内，gate `child.is_multicol && child.column_span_offsets.len()>1`），对每个 outer 碎片调上述分配渲染 inner 行 + clone 装饰。行流来源：layout 侧 store 一个 inner 的 line 列表（新增字段或复用 inline_layout 紧凑存）。

---

## 5. 分阶段实施（每 stage net ≥ 0 才进下一）

### Stage 1（enabling，narrowest）：nested-multicol 碎片 new paint path（行流分配 + 2 子列）
- 范围：outer `column-fill:auto` + 定高 + 唯一 in-flow 子且该子是 multicol（`column-count:2`）+ inner inline-only（无 block 子）+ inner 无 spanner/abspos 子。
- 产出：layout 侧 store inner 行流（col_ctx.lines 紧凑转存 LayoutBox 新字段 `nested_mc_lines`）；paint 新分支按 §4.3 flow-thread 分配渲染每 outer 碎片（先文本 + 2 子列，clone 装饰/rule 留 Stage 2）。
- 验收：004 文本不再单列（PIL 核每 outer 列 2 子列 span，x-span 数 3→6），y cap 在列高内；A/B css-multicol net ≥ 0（守 004a/004b spanner 族 + multicol-fill-auto-001 sentinel）。
- env gate：`ZW_NESTED_MULTICOL_FRAG`（default-off，A/B 证 net≥0 后 default-on）。**精确 gate（避 R1351）**：仅 `child.is_multicol && child.column_span_offsets.len()>1`（被外层碎片化的 inner），不触 deep-nesting normal-multicol。

### Stage 2：paint 全片 + clone 装饰 + fuchsia rule
- 每片绘 inner 2 子列文本 + border-bottom（clone）+ fuchsia column-rule。
- 验收：004 green border-bottom px 趋近 CHR（4385→~14000），fuchsia rule 76→~556；oracle 004 < 5%。

### Stage 3：balance 内层 + outer blue rule 修正 + 全簇验证
- inner `column-fill:balance`（默认）的二级 binary-search balance；outer column-rule §5.2 碎片感知。
- 验收：13 案 multicol-breaking 全簇 oracle < 1%（flip），全量 css-multicol net ≥ 0，product-smoke welcome 不变。

---

## 6. 风险 / gate

- **二级碎片化几何复杂**：inner balance + outer balance 双重迭代（R1348c 已证无 closed-form）—— Stage 1 先锁 inner sequential 避之。
- **回归面广**：multicol breaking 路径被多案共享 → 紧 env gate + 全量 A/B（ORACLE_DUMP_ALL per-case）+ product-smoke。
- **box-decoration-break:clone**：每片重复 border/padding/bg，须不双计（参考 R638 single-line 双计先例）。
- **kill-switch**：`ZW_NESTED_MULTICOL_FRAG=0`（default-off 直至 Stage 1 A/B 证 net≥0）。

---

## 7. 验收总门

- **单案**：multicol-breaking-004 oracle < 1%（flip），PIL 核 2 子列 + 无溢出 + clone 装饰。
- **A/B**：`make reftest-oracle DIR=css-multicol` 全量 net ≥ 0（ORACLE_DUMP_ALL）；特别守 004a/004b（no-border spanner 族不退）、multicol-span-all-017/parallel-flow（R1341 回归规避）、multicol-fill-auto-001（sentinel）。
- **product-smoke**：welcome/wintertc/morning @800+375+320 全 PASS。
- **gates**：fmt + clippy `-D warnings` + `make test` 全绿。

---

## 8. Spec Lint（自检）

| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要 | ✅ | §0 |
| 驱动测试 | ✅ | 13 案 multicol-breaking（PIL 实测） |
| 根因代码级 | ✅ | §2（multicol.rs:696/712 + text.rs:870-872） |
| 分阶段 + 首步可执行 | ✅ | §5 Stage 1（narrowest gate） |
| 验收 + 回滚 | ✅ | §7 + §6 kill-switch |
| 不重蹈先例 | ✅ | §3 R1351 教训 |

**门禁判定**：允许 Stage 1 实施（env gate default-off，A/B 净负即 revert）。

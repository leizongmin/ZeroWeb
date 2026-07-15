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

### 4.2 layout 侧（multicol.rs）

`assign_children_to_columns_with_breaking` 检测到子是 multicol（既有 `has_nested_multicol_seq` 信号已计算）时，**不**当原子块，而是：
1. 先对 inner 做其自身 multicol 布局（2 子列，full content）—— 复用 `store_inline_multicol_columns` 的 IFC + `fragment_lines_into_columns_overflow`；
2. 按 outer `height_limit` 把 inner 的全布局垂直切成 N 片（N = outer 列数或 ceil(inner_h/height_limit)）；
3. 每片存为 outer 一列的子内容（含 inner 2 子列几何 + clone 装饰标志）。

### 4.3 paint 侧（painter/text.rs + mod.rs）

消费 layout 存的 per-fragment 几何：
- 每片在外层列位置渲染 inner 2 子列文本（横向偏移到对应 outer 列）；
- `box-decoration-break:clone`：每片底画 inner border-bottom（green）+ 片间 fuchsia rule；
- outer blue column-rule：碎片化子按片范围计入「有内容」判定（修 blocker C）。

---

## 5. 分阶段实施（每 stage net ≥ 0 才进下一）

### Stage 1（enabling，narrowest）：inner 子列几何按 outer 列高切片计算 + 存 layout
- 范围：仅 outer `column-fill:auto` + 定高 + 唯一 in-flow 子且该子是 multicol（`column-count/width`）+ inner `column-fill:auto`（sequential，先避 balance 二级搜索）+ inner 无 spanner/abspos 子。
- 产出：inner 全布局 → 切 N 片 → 存 per-fragment 几何（文本行 + 子列 x 偏移 + 片顶/底 y）。paint 暂只绘首片文本（验证几何方向）。
- 验收：004 文本不再垂直溢出（y cap ≤ 125），每 outer 列出现 2 子列文本跨度（PIL 核 x-span 数 3→6）；A/B css-multicol net ≥ 0。
- env gate：`ZW_NESTED_MULTICOL_FRAG`（default-off，A/B 证 net≥0 后 default-on）。

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

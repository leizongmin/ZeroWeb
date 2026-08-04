# 实施 handoff：multicol layout 侧 column-aware IFC（R581 代码级修订版）

**版本**：v1.0（R581，2026-06-24）
**作者**：AI Assistant（rendering-compat rally，自主模式，不向用户提问）
**状态**：实施 handoff 蓝图（多 session 架构首个可执行切片；零代码风险 spec）
**承接**：R578（设计 doc 评审）→ R579（代码级调查）→ R580（DC-14 聚合完成，CONTINUE = 产本 handoff）
**关联**：
- [`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md) v0.4（§2 高层架构 / §R317 重定向）
- [`column-aware-IFC-spec.md`](./column-aware-IFC-spec.md) v1.0（R381 Phase 1，**A1 gate 已证伪关闭**）
- [`multicol-phase2-unified-column-flow-spec.md`](./multicol-phase2-unified-column-flow-spec.md) v1.0（R381 Phase 2，**混合内容目标案被 Phase A R109 阻塞**）
- master.md R113/R122/R128/R131/R157/R199-R205/R310/R317/R383/R579

---

## ⚠️ R582 更新（2026-06-24）— 假设 H2 经 probe 证伪：Phase 2a 非安全首切片

R581 handoff §6 假设 **H2**（col_ctx 真实-styles IFC 行盒结构 == paint 空-styles+overrides 行盒，首版可保证 byte-identical）经本轮 probe **证伪**：

- `inline_finalization.rs:944-948` 的 `col_ctx` 仅设 `.with_vertical/.with_text_align/.with_inline_block_sizes`，**不设 white-space/word-break/overflow-wrap/float_exclusions/text_indent/tab_size**。
- `inline/mod.rs:collect_inline_items`（line 691-741）从**真实 styles** 读 per-element font_size/line_height/letter_spacing/word_spacing/is_ahem（故 col_ctx 字体度量正确），但 **white-space/word-break/overflow-wrap 是 config 级**（`self.preserve_whitespace`/`self.word_break`/`self.no_wrap`，默认 = normal）。
- 结论：**H2 仅对 white-space:normal + word-break:normal + overflow-wrap:normal + 无 float/无 text-indent 的 multicol 成立**——而该子集 paint 空-styles+overrides **本就正确**（overrides 由 `store_font_sizes_from_ifc` 完整填充，无 R84 缺口），故 Phase 2a 对该子集**净 0**（无 font 一致性 yield，纠正 §1 发现 2 的乐观推断）。
- 对 pre/pre-wrap/nowrap/break-word/break-all multicol，col_ctx 用 normal 配置换行 != paint 正确换行 → **blanket store+consume 回归**（须 gate）。

**进一步：col_ctx 的 minimal config 不可复用于 Phase 2b**（混合内容须 rich config 的 IFC 行盒，非 minimal col_ctx）。故 R581「复用已算 col_ctx」成本节省前提 **不成立**——Phase 2a 退化为净 0 speculative scaffolding，且其 foundation（2b）被 Phase A R109 阻塞。

**裁决（按 R581 §6 协议「违反 H 即回滚，不强行推进」+ code-guidelines §2 不投机）**：**Phase 2a 当前 spec 不实施**。勿以「capture col_ctx + blanket/gated consume」单会话重试。真重启 Phase 2a 须满足二选一前置：① 发现一个具体 multicol 失败案经诊断确属「paint 空-styles 重跑 font 度量错误」（有驱动 yield）；② Phase A（inline 流动 IFC / R109 解 block 化）先行解锁，使 Phase 2b 可实施（届时 2b 须用 rich-config IFC 重算行盒，非复用 col_ctx）。

本轮另查 R549 next-lever 队列 `blocks-017`（margin-collapse-with-border）= **结构性**（详见 master.md R582：兄弟间距 40/27/40 跨 table↔p 不一致 + table/p 高度 15/19px 不一致 + ref 为 table 基布局）。clean 单会话 lever 第 8 次确证穷尽。

---

## 0. 本文档相对两个 R381 spec 的定位

存在两份 R381（2026-06-19/20）spec，本文档**不重复**其 FR/NFR/IF 正式内容，只做两件事：

1. **修订**：基于 R579 代码级事实，**修正两个 R381 spec 对 Phase 2a 的设计假设**——它们假设 Phase 2a 须「新建 column-aware IFC / 统一 column-flow」，R579 发现该路径的 pure-inline 子集**已在 layout 侧计算好且被丢弃**，Phase 2a 实为「捕获复用」，规模/风险远小于两 spec 描述。
2. **裁决**：明确 Phase 1（R381 column-aware-IFC-spec）**已关闭勿重试**；Phase 2（R381 phase2 spec）混合内容目标案**前置依赖 Phase A**，本 handoff 的 Phase 2a 是**当前唯一独立可实施**（虽 net-0）的首切片。

---

## 1. R579 核心发现（两个 R381 spec 均未捕获）

### 发现 1：balance 模式 column-width IFC 行盒**已在 layout 侧算好并被丢弃**

`crates/layout-engine/src/inline_finalization.rs:1687`（`remeasure_inline_only_containers` 内）：

```rust
let content_height = if let Some((cw, cols)) =
    crate::multicol::balance_column_geometry(style, container_width)
{
    let mut col_ctx = InlineFormattingContext::new(cw)         // ← 列宽 cw 的 IFC
        .with_vertical(is_vertical).with_vertical_rtl(is_vertical_rtl)
        .with_text_align(text_align).with_inline_block_sizes(ib_sizes_for_mc);
    col_ctx.layout(doc, dom_id, styles);                        // ← 用【真实 styles】布局
    let total = col_ctx.total_height();
    let n = col_ctx.lines.len();
    if n > 0 && cols > 0 {
        n.div_ceil(cols) as f32 * (total / n as f32)            // ← 仅用总高+行数算 balance 高
    } else { total }
} else { full_height };
// ← col_ctx.lines 在此处超出作用域被丢弃，仅 content_height 被写回 box_node
```

`balance_column_geometry`（`multicol.rs:120-126`）是 **live 函数**（balance + 列数≥2 即返回 `(col_width, count)`），且其 `col_width = compute_single_column_width(...)` 与 paint 侧 `compute_multicol_info_for_paint().col_width` **同一公式同一结果**。即：layout 侧与 paint 侧各自跑了一次**宽度完全相同、参数相同**的 IFC，layout 侧用真实 styles，paint 侧用空 styles（见发现 2），二者产出的行盒本应一致却被双重计算。

**结论**：pure-inline balance 的「列宽行盒」**不是缺口、是已算好的资产被丢弃**。Phase 2a = 把 `col_ctx.lines` 捕获到 `LayoutBox` 新字段，让 paint 消费，**复用而非新建**。这把两个 R381 spec 描述的「新建 column-aware IFC / 统一 flow」首切片**缩小为「存储接线」**。

### 发现 2：multicol paint **总是用空 styles 重跑 IFC，绕过 R84 stored-font_size 修复**

`crates/engine/src/paint/painter/text.rs:849`：

```rust
let use_stored = multicol_info.is_none() && box_node.inline_layout.is_some() && width_matches;
```

`multicol_info.is_none()` 恒真时才 `use_stored`——**multicol 容器永远不走 stored 路径**，必落 `else` 分支 `ctx.layout(doc, node_id, &HashMap::new())`（text.rs:934，**空 styles**）。

而 R84/R207/R355 正是为消除「paint 空 styles IFC 字体度量不一致」才引入 `inline_layout` 存储（`compute_final_inline_layouts`），却因 `inline_finalization.rs:694` 的 `if root.is_multicol { return }` gate **主动跳过** multicol 容器，使其**永远享受不到** stored-font_size 修正。

**含义（关键 A/B 变量）**：若 Phase 2a 把 col_ctx.lines（真实 font_size）存入并用 `use_stored` 机制消费，multicol 容器将**首次获得** R84 字体度量一致性——这可能修复当前被空-styles-重跑掩盖的小 diff，**也可能触发 multicol-fill-auto-001 类回归**（R198/R209 证 font_size 与列分配耦合易回归，余量 0.63%）。这是 Phase 2a 的**首要 A/B 验证变量**，不是「保证 byte-identical」的简单净 0。

> ⚠️ **纠正 R579 的「Phase 2a = 净 0 foundation」表述**：R579 判 net-0 是基于「paint even-split 已正确（R200）」；但 R579 未发现上述「multicol 永远空-styles 重跑」事实。因此 Phase 2a 的真实结果区间是 **{net-0 ~ 小正收益（font 一致性）~ 小回归（fill-auto 耦合）}**，须 A/B 实测裁定，**不能预设净 0**。这反而让 Phase 2a 有了**可测 yield 假设**（区别于 R579 的「无 immediate-yield」）。

---

## 2. Phase 2a —— 首切片（唯一独立可实施）

### 2.1 范围

仅 **pure-inline + `column-fill: balance` + 任意 height** 的单层 multicol 容器（即当前 paint `text.rs:569` 门控 `!has_in_flow_children && is_balance_mode && height_auto` 的子集，**含去掉 `height_auto` 后新增的明确-height balance 案**）。

**不做**：混合内容（Phase 2b）、嵌套/breaking（Phase 2c）、vertical-rl multicol、column-fill:auto（已有独立路径）。

### 2.2 精确函数级改动（按顺序）

**Step 1 — 新增存储字段（`crates/layout-engine/src/types/mod.rs:192` 旁）**

```rust
/// balance 模式 multicol 容器的「按列宽换行的行盒」，来自 layout 侧 col_ctx
/// （inline_finalization.rs remeasure 路径），供 paint 直接消费，避免空-styles 重跑。
/// 仅 pure-inline balance 容器被填充；其余为 None（paint 走旧路径）。
pub inline_multicol_lines: Option<Vec<InlineLayoutLine>>,   // 复用现有 InlineLayoutLine
```

复用 `InlineLayoutLine`/`InlineLayoutFragment`（types/mod.rs:502-520，已是 paint 消费的格式），**不新建类型**。`Default::default()` 处（types/mod.rs:355 旁）补 `inline_multicol_lines: None`。

**Step 2 — 捕获 col_ctx.lines（`inline_finalization.rs:322-324`）**

在 `col_ctx.layout(...)` 之后、丢弃之前，把 `col_ctx.lines` 转 `Vec<InlineLayoutLine>` 写入 `box_node.inline_multicol_lines`。**转换须复用** `compute_final_inline_layouts` 现有的 `LineBox→InlineLayoutLine` 映射逻辑（同文件已存在，grep `InlineLayoutLine {` 定位），保证字段填充口径一致。`y` 保留列宽 IFC 的行盒 y（paint 消费时按 §2.3 重算列内 y）。

**守卫**：仅当容器 `is_multicol && !has_block_children`（纯 inline）填充；混合内容留 None（交 Phase 2b）。

**Step 3 — paint 消费（`crates/engine/src/paint/painter/text.rs:849` 与 `966-990`）**

改 `use_stored` 判定：multicol 容器若 `inline_multicol_lines.is_some()` 则走 stored 消费分支，复用现有 `stored_fragments`（text.rs:821-848）的展开逻辑，**叠加列分配**：
- 列分配算法**首版与 paint 现有 even-split 完全一致**（`target_h = total/cols`，`floor(line.y/target_h)` 分列，text.rs:853-868），保证首版 **byte-identical**（隔离发现 2 的 font 一致性变量）。
- 列内 y 用 `line.y - col_start_y`（同 text.rs:956），col_x_offset / clip_rect 复用 text.rs:878-882。
- 关键：stored 片段的 `font_size` 来自 col_ctx（真实值），**取代**空-styles 重跑的默认值——这是发现 2 的修复落点，也是 A/B 主变量。

**Step 4 — 保留 paint 旧路径作 fallback**

`inline_multicol_lines.is_none()`（混合/auto/vertical/未填充）时维持 text.rs:849 现有 `multicol_info` 分支不变。**禁止改 `text.rs:569` 门控**（R317 证放宽 height_auto 致 -5 回归）。

### 2.3 门控与验证

| 项 | 约定 |
|---|---|
| env 门控 | 新增 `MULTICOL_STORED_LINES=0` 关闭（回退旧路径），默认开。便于 A/B 与紧急回滚。 |
| sentinel | **`tests/wpt-runner/wpt-data/css/css-multicol/multicol-fill-auto-001.xht`**——`make reftest` 须 self-source **0 翻转**（R198/R209 font_size 耦合历史风险，余量 0.63%）。 |
| A/B 主变量 | 切换前后 multicol 子集 self-source 通过率 + `multicol-fill-auto-001` 的 `z_vs_chr`（chromium Oracle）；以及发现 2 的 font 一致性：挑 1 个非 Ahem pure-inline balance 案（如 multicol-columns-001）REFTEST_DUMP 比 fragment.font_size 切换前后是否从 16（空-styles 默认）变为真实值。 |
| 净结果判定 | 允许三种结局并记录：**net-0**（接受，作为 Phase 2b/c foundation 落地）/ **正收益**（font 一致性修复，额外 win）/ **负**（fill-auto 耦合回归 → 关 env 门控回滚，记录为 R317 同族新证伪）。**任何 -1 self 翻转即回滚**。 |
| 全量门禁 | `make test`（loose 不降）+ `make product-smoke`（DC-13 welcome diff 不退）+ scoped `make reftest`（css-multicol + CSS2 multicol 相关）。 |

### 2.4 文件/行数预算

改动集中 4 文件，每文件 < 60 行净增；`multicol.rs` 不改（复用 `balance_column_geometry`）。`text.rs` 当前已大（paint 入口），新增 stored-multicol 消费若超 ~80 行，抽 `paint_stored_multicol_lines` 私有函数（同文件，遵循 2000 行上限）。

---

## 3. Phase 2b（混合内容）— 前置依赖 Phase A，**本 handoff 不实施**

目标案（multicol-block-no-clip-002 等）经 R383 LAYOUT_DUMP **已证 R109 entanglement**：其 `<span>` 被 R09 converter 转成 block-level LayoutBox，统一 column-flow 即便实现仍按原子 block 分配，**修不了**。真修复须先 **Phase A**（inline 内容作流动 IFC、解 R109 block 化转换）再 multicol 列碎片化——**两多 session lever 依赖**（Phase A → multicol），非独立可实施。

Phase 2a 落地后，Phase 2b 的「缺口收窄」为：paint `text.rs:569` 的 `has_in_flow_children` 门控改为「有 stored `inline_multicol_lines` 即放行」，但 stored 填充须先解决 R109 block 化（Phase A）。详见 [`multicol-phase2-unified-column-flow-spec.md`](./multicol-phase2-unified-column-flow-spec.md) §6.5 A1。

---

## 4. Phase 2c（嵌套 / column breaking）— 结构里程碑，**本 handoff 不实施**

multicol-breaking-004/005/006/nobackground-*。碎片化算法（`assign_children_to_columns_with_breaking`，multicol.rs:336）已存在；缺口是 inline 行盒跨列断裂 + 嵌套 multicol 两趟（R113）。须 Phase 2a stored 基础 + 列预算行盒切片。paint 侧 4 轮（R157/R198/R203/R317）已 ruled out，**必须 layout 侧**。详见 [`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md) §R317 Round 2'。

---

## 5. 实施顺序与里程碑

| 里程碑 | 内容 | 预期 | 风险 |
|---|---|---|---|
| **2a** | §2，本 handoff 唯一实施切片 | net-0 / 小正收益 / 回滚三选一，须 A/B 裁定 | fill-auto 耦合（sentinel 守） |
| 2b | 混合内容 stored 放行 | **阻塞 Phase A** | R109 entanglement |
| 2c | 嵌套/breaking | 结构多轮 | paint ruled out，须 layout |

**首会话目标**：实施 §2 Step 1-4，env 门控默认开，`make test` + `make reftest`(css-multicol) + `make product-smoke` 三门禁全过，记录 §2.3 三结局之一。**若 A/B 为负（fill-auto 回归），关门控回滚，本 handoff 转「Phase 2a 亦 net-negative，multicol 彻底 plateau」结论存档。**

---

## 6. 假设（自主模式，待首步 probe 验证）

- **H1**：`balance_column_geometry` 的 `col_width` 与 paint `compute_multicol_info_for_paint().col_width` 数值一致（同 `compute_single_column_width` 公式）→ 首会话 Step 1 前先 `assert!` 或 probe 打印两者比对确认。
- **H2**：col_ctx（真实 styles）与 paint 空-styles IFC 产出的行盒**结构相同**（行数/换行点一致），仅 font_size 度量差 → probe `col_ctx.lines.len()` vs paint `inline_ctx.lines.len()` 比对。若不一致（换行点不同），Phase 2a 首版不能保证 byte-identical，A/B 区间扩大，须先解释差异。
- **H3**：发现 2 的 font 一致性修复对非 Ahem pure-inline 案是**改善**而非回归（multicol-fill-auto-001 是 column-fill:auto，不属 balance 子集，理论上不受 Step 3 影响，但历史耦合要求 sentinel 必跑）。

**违反任一 H 的处置**：记录为 Phase 2a 新证伪事实，回滚，更新本 handoff 与 master.md。**不强行推进**（code-guidelines §1 暴露困惑优先）。

---

## 7. 代码变更边界（遵守）

- **允许改**：`types/mod.rs`（加字段）、`inline_finalization.rs`（捕获 col_ctx）、`text.rs`（stored-multicol 消费 + use_stored 判定）。
- **禁止改**：`text.rs:569` 门控条件（R317）、`taffy-local/**`（R304 DEFER）、`multicol.rs` 列分配算法（R200 证正确）。
- **禁止**：新建 `multicol_fragment.rs` / `ColumnFragmentationContext` 类型（R199/R200/R579 三轮证伪；复用现有 `InlineLayoutLine`）。

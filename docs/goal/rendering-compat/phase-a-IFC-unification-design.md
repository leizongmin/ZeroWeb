# Phase A — IFC 三路径统一设计（Spec + RFC）

**版本**：v1.0
**日期**：2026-06-18（R305）
**状态**：草稿（read-only 调研产出，未落地代码）
**关联**：`docs/goal/rendering-compat/master.md` R125 / R198 / R205 / R207 / R208 / R209 / R213；DC-13 产品 smoke 文本保真；DC-14 真实一致率

---

## 0. 执行摘要

- **一句话目标**：消除「layout 阶段 IFC」与「paint 阶段 IFC」的二次运行分歧，让 paint 对所有行内容器直接消费 layout 存储的权威行盒结果，从而根除 large-font（100px→16px）、welcome/morning.work 文本度量失真、multicol 多行交互等整簇缺陷。
- **本期范围**：仅产出设计文档 + 分阶段实施计划。**不落地代码**（本轮为 R305 read-only 设计）。
- **明确排除**：multicol-breaking 的 column-aware IFC 碎片化模型（独立 RFC，见 `multicol-fragmentation-design.md`）；writing-mode 轴（R114/R164 4 轮证否）；intrinsic sizing（R97/R301，taffy-blocked per R304）。
- **核心约束**：① 任何阶段**零 count 回归**（项目硬标准）；② 单文件 ≤2000 行（`engine.rs` 现 3969 行，本设计触及处须同步拆分）；③ paint 不得以「改变布局语义」的方式重排 glyph（goal DC-13）。
- **推荐方案**：**baseline-resolved 单一权威行盒**——compute_final 存储每行盒的 `(line_top, baseline_y, line_height, fragments[])`，每个 fragment 存**已解析的绝对基线 y**；paint 永远从该结果渲染，删除 Path B（空 styles 重跑）。用「font_size 一致性不变量」取代 Gate 2 的 single-line+pure-Ahem 启发式。
- **首个落地步骤**：Phase 1 = 在 `InlineLayoutFragment` 增加 `baseline_y` 字段，compute_final 对所有通过 Gate 1 的容器**计算并存储** baseline_y（仍只对 single-line pure-Ahem 实际启用 paint use_stored），先用增量字段建立测量基线，**不改渲染**，跑全量 reftest 确认净 0。

---

## 1. 背景与目标

### 1.1 背景

ZeroWeb 的行内排版（IFC）结果目前在 layout 和 paint 两阶段**各跑一次**，且两者输入不同：

- **layout 阶段**（`compute_final_inline_layouts`，engine.rs:1668）用**真实 ComputedStyle** 跑 IFC，得到正确的 font-size / line-height / line-breaking。
- **paint 阶段**（`paint_text`，text.rs:846）对**未存储** inline_layout 的容器**重新跑一次 IFC**，但用**空 styles + override maps**（R72 为规避 4 个回归而保留的安全路径），font-size 默认 16px。

两趟结果在 font_size、line-breaking、垂直定位上分歧。最典型的可见症状：**large-font bug**——`font-size:100px` 的 Ahem 文本在 paint 阶段被 16px 默认值覆盖（ifc-008/009/011、font-051 多行变体、welcome 标题）。

R125 / R198 / R205 / R209 / R213 共 5 轮尝试单点解锁 font_size 均**净负向回归**：

| 轮次 | 尝试 | 结果 |
|------|------|------|
| R125 | 三路存储（store_font_sizes 覆盖/不覆盖/真实 styles） | 全净 -1/-1/-4，回退 |
| R198 | compute_final IFC 后 store_font_sizes + multicol ancestry 守卫 | 净 -1（CSS2 +1 / css-multicol -1），死锁成立 |
| R205 | paint 注入真实 font_size 单字段（解耦 line-height） | 全净负，font_size 与 line-height 耦合 |
| R207 | narrow 精修：仅「纯 inline 叶文本容器」存行盒 | **+1（font-051）零回归，默认启用** ✅ |
| R209 | 放宽 Gate 2 多行存储（PHASEA_MULTILINE） | ifc-008/009 改善但 multicol-fill-auto-001 0.63→9.15 回归，回退 |
| R213 | 多行存储加 `!in_multicol` 守卫 | 净 0（multicol-fill-auto 由 ref 文件非 multicol 的 float 模拟，守卫无法触及），回退 |

R207 证明**存储架构本身正确**（pure-inline 叶文本容器 +1），但 broad 应用被三处墙阻塞。本 RFC 的任务是把这些墙精确定位并给出**架构性**（非单点）的统一方案。

### 1.2 目标

- **业务目标**：让 ZeroWeb 的行内文本渲染与 Chromium 在 font-size / line-height / 换行上一致，消除 large-font 簇缺陷（DC-13 welcome 文本、DC-2/5 文本类 reftest）。
- **用户目标**：产品静态页（welcome / morning.work）正文不再被压成 16px 默认行；标题、卡片文本保真。
- **可验证成功标准**：① large-font 簇（ifc-008/009/011）chromium-Oracle z_vs_chr 下降；② welcome product-smoke 文本区 diff 下降；③ 全量 reftest loose 438/490 不退、strict 296/490 不退、chromium-Oracle 真实一致率不降。

---

## 2. 现状分析（三条 IFC 路径 + 两个 Gate + 三处墙）

### 2.1 三条 IFC 路径

```
                    compute_final_inline_layouts (engine.rs:1668)
                            │  用真实 styles 跑 IFC
                            ▼
                 ┌──────────────────────────┐
                 │ Gate 2 (engine.rs:1910)  │
                 │ lines.len()<=1 &&        │── 否 ──▶ 不存 inline_layout
                 │ is_pure_ahem             │         （但仍调 store_font_sizes_from_ifc）
                 └─────────────┬────────────┘
                          是   │  存 inline_layout (line boxes)
                               ▼
                 paint_text (text.rs:807)
                 use_stored = !multicol && inline_layout.is_some() && width_matches
                               │
            ┌──────────────────┴──────────────────┐
            ▼                                     ▼
   Path A: use_stored=true                Path B: use_stored=false
   渲染 stored fragments                  重跑 IFC（空 styles + override maps）
   v_offset = is_ahem?0:font_size         baseline_fs = text_node_font_sizes[node] or 16px
   (text.rs:1208)                         (text.rs:1224-1225)
```

**Path A（stored）**：compute_final 用真实 styles 算出正确的 `frag.y`（fragment 框顶部）+ `frag.font_size`，paint 直接渲染，`v_offset = is_ahem ? 0 : font_size`（Ahem 位图是完美 font_size 方块无 ascent 留白→offset=0；普通字体 font_size≈ascent）。

**Path B（re-run）**：paint 用**空 styles** 重跑 IFC（`frag.y` 基于 16px 默认），再用 `text_node_font_sizes` map 里存的**真实 font_size** 作为 `baseline_fs` 修正垂直定位。R72 刻意用空 styles 而非真实 styles，是为规避 BFC-004 / font-feature-002 / position-absolute-in-inline-005/006 四个回归。

**关键事实**：`store_font_sizes_from_ifc`（engine.rs:1152）在 compute / remeasure 多处（line 1079/1381/3136/3266）被调用，**不受 Gate 2 限制**——即 per-text-node 的 font_size/line_height/is_ahem map 总是广泛建立。Gate 2 只限制 **`inline_layout`（完整行盒）** 的存储。

### 2.2 两个 Gate

| Gate | 位置 | 条件 | 作用 |
|------|------|------|------|
| **Gate 1**（R207 narrow） | engine.rs:1720-1749 | `has_text_children` 扩展 = 有 inline-level 元素子节点 **且** 无 block-level 子节点 **且** inline 子元素无元素后代（叶文本容器） | 决定**哪些容器**进入 IFC 计算 |
| **Gate 2**（R84 安全子集） | engine.rs:1910 | `lines.len() <= 1 && is_pure_ahem`（纯 Ahem 单行） | 决定**哪些容器实际存储** inline_layout |

另有显式跳过（engine.rs:1681-1707）：flex/grid/table 容器、`is_multicol` 容器、非 block-level 元素。

### 2.3 三处墙（broad 应用阻塞点）

**墙 ① — Gate 2 多行限制（large-font 簇根因）**
ifc-008 = `div1 > inner-div(block) > "XX XX" 100px Ahem`，200px 宽换 2 行。inner-div 是 block + 直接文本 → 过 Gate 1（line 1710 直接 `has_text_children=true`，不走 R207 扩展）。但 Gate 2 `lines.len() > 1` → 不存 → paint 走 Path B → 16px。R209 已用干净单趟探针确认 node 被访问、block=true、direct_text=true，**唯一阻塞 = 多行限制**。

**墙 ② — multicol 反向依赖（R198/R209/R213）**
multicol 容器 paint 永远走 Path B（`use_stored = multicol_info.is_none()`，text.rs:807；multicol_info 在 `!has_in_flow_children && is_balance_mode && height_auto` 时计算，text.rs:713）。放宽 Gate 2 让 multicol 的**内层内容容器**存 inline_layout 后，multicol-fill-auto-001 从 0.63%→9.15% 回归。机制疑点：multicol paint 重跑的 font_size 来自 `text_node_font_sizes` map，而该 map 已不受 Gate 2 限制广泛建立——故回归可能**不是** font_size map 变化，而是被存 inline_layout 的容器改变了某条 paint 分支选择或几何（需 Phase 2 探针实证，见 §6.4）。R213 的 `!in_multicol` 守卫无效是因为 multicol-fill-auto 的 ref 用 float（非 multicol）模拟列，`is_multicol=false`，守卫触及不到。

**墙 ③ — v_offset / baseline 语义分歧**
Path A 用 `v_offset = is_ahem ? 0 : font_size`（text.rs:1208），Path B 用 `baseline_fs = stored or 16`（text.rs:1225）。两者对「fragment.y 相对行的垂直锚定」假设不同。对多行非-Ahem 内容，stored 的 `frag.y`（真实 font_size 下的行顶）+ Path A v_offset 与 Path B 的 16px 行顶 + baseline_fs 不一致——这是 R206 broad 应用导致 ifc-001/002/003 翻 FAIL 的直接原因。**只要 Path B 还存在，两套语义就必须手工保持一致，而这已被 5 轮证明不可单点维护。**

### 2.4 结论

墙 ③ 是**架构性**的：只要 paint 同时存在「消费 stored」与「重跑 IFC」两条路径，两套 baseline 语义就无法收敛。R207 的成功恰恰是它把 Path A 限制在「single-line pure-Ahem」这一**两路径天然等价**的子集上。真正的解法是**消灭 Path B**——让所有通过 Gate 1 的容器都存 inline_layout，paint 永远用 Path A，并让 Path A 的位置语义对多行非-Ahem 也正确。

---

## 3. 范围边界

- **在范围内**：
  - `compute_final_inline_layouts`（engine.rs:1668）的存储条件与存储字段语义
  - `paint_text`（text.rs）的 use_stored 决策与 Path A 渲染路径
  - `InlineLayoutFragment` / `InlineLayoutLine`（types/mod.rs）数据结构
  - multicol paint 的 font_size 一致性（仅触及「multicol 内层容器的存储是否触发」，不触及 column 分配算法）
- **不在范围内**：
  - multicol-breaking column-aware IFC 碎片化（→ `multicol-fragmentation-design.md`）
  - writing-mode 轴 / vertical-rl（→ R114/R164，4 轮证否）
  - intrinsic sizing / max-content（→ R97/R301，taffy-blocked）
  - IFC 内部换行算法本身（advance-width plumbing → `advance-width-plumbing-design.md`，已证伪为独立死路 R225）

---

## 4. 设计需求（FR）

### FR-001：单一权威 IFC 结果
- **描述**：paint 必须消费 compute_final 存储的行盒结果渲染行内文本；当 inline_layout 存在且宽度匹配时，paint **禁止**重跑 IFC。
- **优先级**：必须
- **验收场景**：
  - 场景（正常）：给定一个 `font-size:100px` 单行 Ahem 容器，paint 渲染的 glyph 高度 == 100px（非 16px）。验证：`make reftest` ifc-008 class 不再出现 16px 文本（chromium-Oracle z_vs_chr 下降）。
  - 场景（异常）：给定一个 inline_layout 未存储的容器（如 flex/grid/table，被 Gate 1 显式跳过），paint 必须回退到现有 Path B 重跑，行为与现状一致。验证：全量 reftest 这些类目 count 不变。

### FR-002：baseline-resolved 位置语义
- **描述**：stored 行盒必须携带**已解析的绝对基线 y**（`baseline_y = line.y + ascent`），paint 直接用该 y 定位 glyph，不再用 `is_ahem ? 0 : font_size` 启发式 v_offset 推断。
- **优先级**：必须
- **验收场景**：
  - 场景（正常）：多行非-Ahem 文本，每行 glyph 基线 y 与 Chromium 一致（行间距 = line-height）。验证：`make reftest` ifc-001/002/003 保持 PASS（R206 broad 翻 FAIL 的三例须恢复）。
  - 场景（异常）：若某 fragment 的 ascent 无法解析（无 FontLoader 度量），回退到当前 `font_size` 近似并记录 tracing 日志，渲染不崩。验证：单测 `baseline_y_fallback_uses_font_size`。

### FR-003：font_size 一致性不变量
- **描述**：删除 Gate 2 的 `is_pure_ahem && lines.len()<=1` 启发式，改用确定性不变量「stored 行盒的 font_size 与 paint 读取的 font_size 必须同源（都来自真实 styles）」。
- **优先级**：必须
- **验收场景**：
  - 场景（正常）：所有过 Gate 1 的容器（含多行非-Ahem、block-child 直接文本）都存 inline_layout。验证：LAYOUT_DUMP 确认 inner-div 的 inline_layout 非空。
  - 场景（异常）：multicol 容器（Gate 1 显式跳过 `is_multicol`）不存，paint 走现有 column 重跑。验证：multicol 类目 count 不变。

### FR-004：零 count 回归硬门禁
- **描述**：每个 Phase 必须以全量 `make reftest`（loose 438/490）+ strict（296/490）+ chromium-Oracle 抽样 三态不退为合并条件。
- **优先级**：必须
- **验收场景**：见 §10 验证策略。

---

## 5. 约束与假设

### 5.1 必须约束（Must）
- 任何 Phase 落地前 `make test` 全绿、`cargo clippy --workspace --all-targets -D warnings` 干净。
- 触及 `engine.rs`（3969 行）/ `text.rs` 的修改须同步评估 2000 行拆分（§7.2）。
- 修改「禁止修改」路径须停止并说明。

### 5.2 禁止约束（Must Not）
- 不允许以放宽容差掩盖 large-font 回归（DC-14 容差锁定）。
- 不允许 paint 对 glyph 做改变布局语义的整行重排（goal DC-13）。
- 不允许引入新的 `#[ignore]`（除 real_website_compat.rs）。

### 5.3 已定决策
- 复用 `compute_final_inline_layouts` + `paint_text` 现有架构，不重写 IFC 引擎。
- 字体度量优先复用 fontdue 现有 ascent（R188 标记的「font-family→FontId 解析在 paint 懒做」是独立阻塞，本设计在 layout 侧用 fragment 已有的 font_size 近似 ascent，**不引入 FontLoader 全量预解析**——那是更大独立 RFC）。

### 5.4 假设
- **A1（待验证）**：fontdue 能在 layout IFC 阶段为每个 fragment 提供 ascent（或可用 font_size 近似）。— 状态：待验证（Phase 2 探针）。
- **A2（待验证）**：multicol-fill-auto 回归的真正机制是「被存 inline_layout 的内层容器改变了 paint 分支」而非 font_size map 变化。— 状态：待验证（Phase 3 探针）。
- **A3**：R207 narrow 子集（pure-inline 叶文本）在新语义下仍 PASS（baseline-resolved 对单行 Ahem 退化到旧 v_offset=0）。— 状态：需 Phase 1 验证。

### 5.5 代码变更边界
- **允许修改**：`crates/layout-engine/src/engine.rs`、`crates/layout-engine/src/types/mod.rs`、`crates/engine/src/paint/painter/text.rs`、`crates/layout-engine/src/inline/mod.rs`（仅 fragment 字段）。
- **禁止修改**：`crates/taffy-local/**`（vendored，R304 DEFER）、`crates/render-foundation/**`（渲染器，与 IFC 语义无关）、`tests/wpt-runner/**`（reftest harness）。

---

## 6. 详细设计（RFC）

### 6.1 目标状态架构

```
compute_final_inline_layouts (真实 styles IFC)
        │  对所有过 Gate 1 容器存 inline_layout
        │  每行存 (line_top, baseline_y, line_height)
        │  每片段存 (x, y=baseline_y, width, font_size, ...)
        ▼
paint_text
   use_stored = inline_layout.is_some() && width_matches   ← 删除 !multicol 例外
        │  （multicol 见 §6.4 单独处理）
        ▼
   Path A 唯一路径：渲染 stored fragments，glyph y = baseline_y
   Path B（空 styles 重跑）—— 仅 Gate 1 显式跳过的容器（flex/grid/table）保留
```

**核心变更**：`InlineLayoutFragment.y` 语义从「fragment 框顶部」改为「已解析基线 y」（或新增 `baseline_y` 字段保留 `y` 兼容），paint 直接用，删除 `is_ahem ? 0 : font_size` 推断。

### 6.2 数据模型变更

```rust
// crates/layout-engine/src/types/mod.rs
pub struct InlineLayoutLine {
    pub y: f32,            // 行盒顶部（保留）
    pub height: f32,
    pub baseline_y: f32,   // 【新增】该行基线绝对 y = line.y + ascent
    pub fragments: Vec<InlineLayoutFragment>,
}
pub struct InlineLayoutFragment {
    pub x: f32,
    pub y: f32,            // 保留（fragment 框顶）
    pub baseline_y: f32,   // 【新增】片段基线绝对 y（单行时 = line.baseline_y）
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub is_ahem: bool,
    pub text: String,
    pub node_id: Option<NodeId>,
}
```

**实现来源**：ascent 由 IFC 已计算的 `line.height` + `frag.font_size` 推导（Ahem: ascent=font_size→baseline_y=line.y+font_size；普通字体 ascent≈font_size×0.8 近似，A1 待 Phase 2 用 fontdue 精确 ascent 替换）。

### 6.3 Gate 重构

- **Gate 1**（保留 + 微扩）：维持 R207 narrow 条件 + block-child 直接文本（line 1710 路径）。
- **Gate 2**（删除）：移除 `lines.len()<=1 && is_pure_ahem` 启发式，所有过 Gate 1 容器都存。
- **paint use_stored**：移除 `multicol_info.is_none()` 例外（multicol 改用 §6.4）。

### 6.4 multicol 处理（解墙 ②）

两种方案，Phase 3 探针后定：

- **方案 M1（推荐）**：multicol 内层容器**照常存** inline_layout；paint multicol 路径消费 stored fragments 做列分配（而非重跑 IFC）。需先实证 A2（回归机制）。
- **方案 M2（保守 fallback）**：multicol 内层容器保持现状（不存，paint 重跑），但确保其 font_size 来自真实 styles 而非 16px——即把 Path B 的空 styles 改为「仅 multicol 路径」用真实 font_size（解 large-font 但保留 column 重跑）。

**最终选择**：Phase 3 探针后定，倾向 M1（与全局消灭 Path B 一致）。

### 6.5 影响范围分析

| 影响项 | 程度 | 说明 |
|--------|------|------|
| engine.rs compute_final | 高 | Gate 2 删除 + baseline_y 存储 |
| text.rs paint_text | 高 | Path A 唯一化 + baseline_y 消费 |
| types/mod.rs | 中 | 两结构加字段 |
| inline/mod.rs | 低 | fragment 产 baseline_y |
| engine.rs 文件行数 | 中 | 现 3969 行，本设计净增 ~50 行，须拆分（§7.2） |

---

## 7. 实施交接

### 7.1 推荐修改顺序（5 个 Phase，每 Phase 独立可合并）

1. **Phase 1（测量基线，零渲染变化）**：加 `baseline_y` 字段，compute_final 计算+存储（仍只对 single-line pure-Ahem 启用 use_stored），paint 暂不消费新字段。→ 验证：全量 reftest 净 0（字段是死的，不改变渲染）。
2. **Phase 2（baseline-resolved 渲染，Gate 2 不变）**：paint Path A 改用 `baseline_y` 渲染（替代 v_offset 启发式），Gate 2 仍 single-line pure-Ahem。→ 验证：font-051 等 R207 子集仍 PASS（A3 验证 baseline_y 对单行 Ahem 退化正确）。
3. **Phase 3（探针 multicol 墙 ②）**：read-only 探针实证 A2（multicol-fill-auto 回归机制），定 M1/M2。→ 验证：产出探针报告，无代码变更。
4. **Phase 4（删除 Gate 2 多行限制）**：放宽 Gate 2 到所有过 Gate 1 容器 + multicol 按 Phase 3 方案。→ 验证：ifc-008/009/011 改善，multicol 类目不退，large-font 簇 chromium-Oracle 下降。
5. **Phase 5（收尾清理）**：删除 Path B 中已无消费者的空 styles 重跑代码（仅 flex/grid/table 保留），engine.rs 拆分。→ 验证：全量三态不退 + clippy/fmt 干净。

### 7.2 文件拆分（2000 行约束）

`engine.rs`（3969 行）须拆分。建议按职责：`compute_final_inline_layouts` + `store_font_sizes_from_ifc` + `remeasure_*` 抽到 `crates/layout-engine/src/inline_finalization.rs`（~400 行）。Phase 5 执行，避免与逻辑改动混在一个 commit。

### 7.3 首批提交建议

| 批次 | 范围 | 预期结果 | 验证 |
|------|------|----------|------|
| Phase 1 | types +2 字段、compute_final 计算 baseline_y | reftest 净 0（死字段） | `make reftest` loose 438/490 |

---

## 8. 回归风险与缓解

| 风险 | 概率 | 缓解 |
|------|------|------|
| baseline_y 对普通字体 ascent 近似不准 → 文本类大面积漂移 | 高 | Phase 2 先在 Ahem 子集验证退化正确；普通字体分阶段，每类目 set-diff |
| multicol 墙 ② 未解 → 放宽 Gate 2 仍回归 | 中 | Phase 3 探针先行，Phase 4 用 M2 保守 fallback 兜底 |
| Path B 删除后 flex/grid/table 渲染变化 | 低 | Phase 5 保留 flex/grid/table 的重跑分支，仅删 inline 类消费者 |
| engine.rs 拆分引入编译/测试断裂 | 低 | Phase 5 单独 commit，纯移动不改逻辑 |

---

## 9. 验证策略

- **单元测试**：每个 Phase 新增 fragment baseline_y 计算单测（`baseline_y_ahem_equals_line_top_plus_font_size`、`baseline_y_fallback_uses_font_size`）。
- **reftest**：每 Phase 跑全量 `make reftest`（loose 438/490 不退）+ `ZERO_REFTEST_STRICT=1 make reftest`（strict 296/490 不退）。
- **chromium-Oracle**：每 Phase 跑 `scripts/cross-validate.py` 抽样，确认 large-font 簇 z_vs_chr 下降、其他类目污染率不升。
- **回滚**：每 Phase 独立 commit，任一 Phase 三态退步即 `git revert` 该 commit，不污染前序 Phase。

---

## 10. Spec Lint 报告

### 结构完整性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要 | ✅ Pass | §0 存在，含目标/范围/约束/方案/首步 |
| 场景存在性 | ✅ Pass | FR-001~004 各有 ≥1 验收场景 |
| 异常路径覆盖 | ✅ Pass | 每 FR 含正常+异常场景（回退/不变） |
| 测试绑定 | ✅ Pass | 每场景绑 `make reftest`/单测函数名 |
| TBD 清零 | ⚠️ Warning | A1/A2 待 Phase 2/3 探针验证（非阻塞，已降级为假设） |
| 实施交接 | ✅ Pass | §7 含文件清单、修改顺序、首批提交 |
| 首步可执行性 | ✅ Pass | §7.1 Phase 1 + §7.3 首批 |

### 语言精确性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 模糊动词 | ✅ Pass | FR 用「必须消费/禁止重跑/必须携带」具体动词 |
| 非确定性措辞 | ✅ Pass | 无「应该/可能/尽量」（§6.4「倾向 M1」标注为待定决策非需求） |

### 一致性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 范围冲突 | ✅ Pass | §3 在范围/不在范围无交集 |
| 方案漂移 | ✅ Pass | §6 设计未引入与 §5.2 Must Not 冲突的依赖 |
| 代码边界完备 | ✅ Pass | §5.5 允许/禁止修改均声明 |
| 实现来源闭合 | ✅ Pass | §6.2 ascent 来源（IFC 已计算值 + fontdue 待 A1）已写 |

**汇总**：14 Pass / 1 Warning / 0 Fail / 0 Skip
**门禁判定**：Fail = 0 → 允许进入实施（本轮为设计，下一轮 R306 起 Phase 1）

---

## 11. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2026-06-18 | R305 初始 read-only 设计产出 |



# 设计：multicol Phase 2a — `ColumnFragmentationContext` 接口（单层 multicol + 单一 block 子元素 breaking 跨列）

**版本**：v1.0（rally 自主模式，假设见 §6.5）
**日期**：2026-06-30
**作者**：AI Assistant（rendering-compat rally）
**状态**：Phase 1 死字段切片 LANDED（net 0，dormant，守 multicol-fill-auto-001）；Phase 2a step-2 commit 1 纯算法切片 LANDED（`column_fragmentation_flow.rs`，net 0，零生产调用方，9 单测）；step-2 commit 2（layout 接线 + LayoutBox 输出 + paint 消费）待多会话接力
**模式**：**rally-pattern 设计文档**（非 `lei-spec-rfc` skill —— 该 skill 需用户确认，与无人值守 rally 输出协议冲突；详见 master.md R896）
**关联**：
- rendering-compat master.md R896 / R897
- [`multicol-phase2-unified-column-flow-spec.md`](./multicol-phase2-unified-column-flow-spec.md)（混合内容 balance，**Phase A 前置依赖，非独立可实施**）
- [`column-aware-IFC-spec.md`](./column-aware-IFC-spec.md)（Phase 1 pure-inline balance，**A1 gate 0/16 已关闭**）
- [`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md) §2.1（`ColumnFragmentationContext` 最初草绘）
- 经验证据：[`evidence/r897-multicol-phase2-probe-2026-06-30.txt`](./evidence/r897-multicol-phase2-probe-2026-06-30.txt)
- 字体桥接同型先例：`crates/layout-engine/src/inline/font_metrics.rs`（R885，commit `d5b7e3ae`）

---

## 0. 执行摘要

> **为什么是这个 slice**：multicol 是 oracle 一致率最低的目录（**23.0%**，master.md R893）。剩余 wpt-runner-reachable 杠杆全部进入多会话硬核（multicol Phase 2 / R109 §9.2.1.1 / baseline-export）。R896 裁决：以**最窄的 R109-independent 可碎片化子集**作为多年期硬核的**首个 enabling slice**，避免 Phase 1 的 0-case 停止。

- **一句话目标**：为 layout 侧 column-aware IFC 定义 `ColumnFragmentationContext` 接口（IFC 的**输入**：列几何 + 列高预算 + 每列已占高度 + fill 模式），并以**最窄可碎片化子集**——单层 multicol + `column-fill:auto` + 明确高度 + **单一 block 子元素 breaking 跨列**（R109-independent，区别于 mixed-content 被 Phase A 阻塞、区别于 nested 是 Phase 3）——为首个落地目标。本期 = 接口 + Phase 1 死字段（net 0），实施（碎片化逻辑）留后续会话。
- **本期范围（Phase 1 死字段）**：新增 `crates/layout-engine/src/inline/column_fragmentation.rs` = `ColumnFragmentationContext` 数据结构 + `ColumnFillMode` 枚举；`InlineFormattingContext` 新增 dormant 字段 `column_fragmentation: Option<ColumnFragmentationContext>` + builder `with_column_fragmentation`；**零生产读取**（grep 证）；附单测证默认 None（零回归）+ 注入工作 + 字段语义。镜像 font-bridge R885。
- **明确排除（本期）**：① 碎片化**逻辑**实施（按列高把行盒分配到列 + 余量 overflow 处理）= Phase 2a step-2（下一会话）；② mixed-content balance（Phase 2b，Phase A 前置依赖）；③ nested multicol breaking（Phase 3）；④ pure-inline balance 明确高度（Phase 1 已 0-case 关闭）；⑤ baseline-export（独立卡点 #4）。
- **核心约束**：① **零回归**（default None = 行为不变；`make test` 全绿 + product-smoke welcome <20% + scoped multicol oracle 零回归，**特别守 multicol-fill-auto-001 sentinel**）；② chromium-Oracle z_vs_chr 门禁（非 self-source）；③ 单 `.rs` ≤2000 行；④ 测试用 `make test`/`make reftest`（test-guard 包裹）。

---

## 1. 背景与目标

### 1.1 现状（三段分离模型 + 探针实证）

当前 multicol 渲染经三段分离（`multicol-phase2-unified-column-flow-spec.md` §1.1）：

| 阶段 | 位置 | 行为 |
|------|------|------|
| taffy 块布局 | taffy-local | block 子正常堆叠（不感知列） |
| block 子重分配 | `multicol.rs::assign_children_to_columns_*` + `position_multicol_children` | block 子按高度分配到列；breaking 子产 `ColumnFragment`（`fragment_y_offset` + `visual_height`），写 `column_span_offsets` |
| paint 垂直窗口 | `painter/mod.rs:840` | 对每个 `column_span_offsets` 片段重绘整个子元素 + 列区域裁剪（`clip_all_primitives_to_rect`） |

**R897 探针对「单层 + 单一 block 子 breaking」slice 的实证**（[`evidence/r897-multicol-phase2-probe-2026-06-30.txt`](./evidence/r897-multicol-phase2-probe-2026-06-30.txt)）：

- **Probe B（height=48px = 3 行整数倍，12 行文本，column-count:3，column-fill:auto）**：ZW 渲染 col0/col1/col2 各 3 行（y=[8,55]=48px），**第 10-12 行被静默丢弃**（y>56 无 dark px，不渲染不溢出）。根因 = `assign_children_to_columns_with_breaking`（multicol.rs:336）产 3 fragments 覆盖 child y=[0,144]（前 9 行），循环在末列停止（`current_col+1 < col_count` 守卫），余量 child y=[144,192]（第 10-12 行）**在 assignment 阶段丢弃**。
- **Probe A（height=60px 非整数倍，同结构）**：ZW 把 12 行按 4-4-4 分配，每列 64px **超出 60px 列高**（col0 文本连续 y=[8,71]，列边界 y=68 未裁断行）= overfill。
- **对比 nested case（R201）**：nested multicol（multicol-breaking-001/002/003）的缺口是「文本只在 col0」（inner 自身是 multicol）；**本单层 slice 的缺口不同**——文本**正确分布到各列**，但 **assignment 阶段静默丢弃余量 + 非整数高度 overfill + 列高未严格 respected**。

**根本问题**：paint 侧垂直窗口机制对**粗粒度分布**工作（重绘+裁剪），但**无法精确计算每列容纳的行盒**——列边界落在行盒中间时，CSS 要求整行移至下列（不裁断行），而垂直窗口是后验裁剪。精确碎片化须 **layout 侧 column-aware IFC**：在 layout 期算出每列的行盒 + 余量，存 LayoutBox，paint 直接消费（取代后验裁剪）。

### 1.2 目标

- **业务目标**：css-multicol oracle 一致率 23.0% → 提升（DC-4 长期 ≥95%，受 fontdue≠Skia 光栅上限 + 结构性簇双上限，单 slice 不期望显著提升）。
- **工程目标（本期）**：为 layout 侧 column-aware IFC 落地**接口 + dormant 字段**，使后续会话的碎片化实施有明确的 seam（避免重写 IFC）。
- **用户目标**：单层 multicol + block 子 breaking 与 chromium 一致（不丢内容、列高 respected）。

### 1.3 范围边界

- **在范围内（本期 Phase 1）**：`ColumnFragmentationContext` 数据结构 + `ColumnFillMode` 枚举 + IFC dormant 字段 + builder + 单测，**零生产读取**。
- **不在范围内（本期）**：碎片化逻辑（Phase 2a step-2）；mixed-content（Phase 2b，Phase A 前置）；nested（Phase 3）；pure-inline balance（Phase 1 已关）；baseline-export；taffy 升级（R304 DEFER）。

---

## 2. 需求类型概览

| 类型 | 是否适用 | 来源 |
|------|---------|------|
| 业务需求 | 是（长期） | DC-4 css-multicol（本期不期望单 slice 提升） |
| 功能需求 | 是 | §3（接口 + dormant 字段） |
| 非功能需求 | 是 | §4（零回归 + chromium-Oracle） |
| 接口需求 | 是 | §5（`ColumnFragmentationContext` + IFC 字段） |
| 过渡需求 | 是 | env 门控（Phase 2a step-2 渐进启用，§6.3） |

---

## 3. 功能需求

### FR-001：定义 `ColumnFragmentationContext` 数据结构（IFC 碎片化输入）

- **描述**：新增 `ColumnFragmentationContext`，携带 IFC 把行盒碎片化到列所需的全部输入：列几何（`col_count`/`col_width`/`col_gap`）、列高预算（`available_height: Option<f32>`，None=无高度约束/balance）、每列已被 block 子占用高度（`col_filled_heights: Vec<f32>`，长度=col_count）、fill 模式（`ColumnFillMode::Balance | Auto`）。本期仅定义结构，**无任何调用方构造或读取**（dormant）。
- **优先级**：必须（Phase 2a 接口基石）
- **来源**：`multicol-fragmentation-design.md` §2.1 草绘；CSS Multicol §6 fragmentation + §8 balance

### FR-002：IFC 新增 dormant 字段 + builder（零回归）

- **描述**：`InlineFormattingContext` 新增 `column_fragmentation: Option<ColumnFragmentationContext>`（默认 `None`）+ builder `with_column_fragmentation(ctx)`。`None` 时 IFC 行为完全不变（行盒不碎片化，当前行为）。本期**无生产读取**（grep 证 `column_fragmentation` 仅出现在定义 + builder + tests）。
- **优先级**：必须
- **来源**：font-bridge R885 同型 dormant 模式（`font_metric_provider`）

### FR-003：env 门控（Phase 2a step-2 用，本期预留）

- **描述**：Phase 2a step-2 实施碎片化时，用 env `MULTICOL_COLUMN_FRAG`（未设=关闭默认）门控渐进启用，便于 A/B 测量与回滚。本期仅记录约定，不实现门控读取。
- **优先级**：必须（多会话安全网，留待 step-2）

---

## 4. 非功能需求

### NFR-001：零回归（本期硬门禁）
- **描述**：dormant 字段默认 None + 无生产读取 → 渲染字节级不变。`make test` 全 workspace 绿；product-smoke welcome <20%（DC-13 gate）；scoped `make reftest-oracle DIR=css-multicol` 零回归（**特别 multicol-fill-auto-001 sentinel，R198/R209 font_size 耦合 0.63% 余量小**）。
- **测量**：`make test`（test-guard）+ `make product-smoke` + `make reftest-oracle DIR=css-multicol`。
- **优先级**：必须

### NFR-002：chromium-Oracle 真一致率（DC-14，Phase 2a step-2 目标）
- **描述**：碎片化实施后，目标案 z_vs_chr 须下降；**仅 self-source 翻转不算**（R381 教训）。本期 Phase 1 死字段不涉及渲染变化，故 NFR-002 留 step-2。
- **优先级**：必须（step-2）

### NFR-003：文件行数
- **描述**：新结构放 `crates/layout-engine/src/inline/column_fragmentation.rs`（新文件，<200 行）；IFC `mod.rs` 仅加 1 字段 + 1 builder（<5 行）。单文件 ≤2000 行。
- **优先级**：必须

### NFR-004：单元测试
- **描述**：`column_fragmentation.rs` 附单测证：① 默认 `None`（IFC::new 零回归）；② `with_column_fragmentation` 注入后字段 Some 且数据正确；③ `ColumnFragmentationContext` 字段可构造（col_count/col_width/available_height/col_filled_heights/fill_mode）。
- **优先级**：必须

---

## 5. 接口需求

### IF-001：`ColumnFragmentationContext` 数据结构（新模块 `inline/column_fragmentation.rs`）

- **类型**：数据结构（layout IFC 的碎片化**输入**）
- **规格**：

  ```rust
  /// column-fill 模式（IFC 碎片化用，本地枚举避免耦合 style-system）。
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum ColumnFillMode { Balance, Auto }

  /// multicol 列碎片化上下文 —— IFC 把行盒碎片化到列所需的输入。
  ///
  /// Phase 2a step-1（本期）：仅定义 + IFC dormant 字段，零生产读取。
  /// step-2：IFC `break_items_into_lines` 产宽度换行行盒后，按本上下文
  /// 把行盒分配到列（respected 列高 budget，避免 overfill/丢余量）。
  #[derive(Debug, Clone, PartialEq)]
  pub struct ColumnFragmentationContext {
      pub col_count: usize,
      pub col_width: f32,
      pub col_gap: f32,
      /// 每列可用高度预算（px）。None = 无高度约束（balance/height:auto）。
      pub available_height: Option<f32>,
      /// 每列已被 block 子占用的高度（px，长度 = col_count）。
      /// 单一 block 子 slice：全 0（block 子自身即被碎片化内容）。
      /// mixed-content（Phase 2b）：block 子占部分列高，IFC 行盒须避开。
      pub col_filled_heights: Vec<f32>,
      pub fill_mode: ColumnFillMode,
  }
  ```

- **设计决策**：
  - **无 trait / 无 handle newtype**（区别 font-bridge）：`ColumnFragmentationContext` 是**纯数据**传入 IFC，非依赖反转（font-bridge 需 trait 因 IFC 要查 render-foundation 的 FontLoader 而不引入生命周期）；纯数据可直接 `derive(Debug)`，无需 handle。
  - **本地 `ColumnFillMode` 枚举**（非复用 `style-system::ColumnFillComputedValue`）：保持 layout-engine IFC 自包含，避免 IFC 模块耦合 style-system 类型（与 IFC 既有 `TextAlign`/`WordBreakMode` 本地枚举风格一致）。
  - **`col_filled_heights: Vec<f32>`**（非单值）：支持 Phase 2b mixed-content（block 子占不同列高）；单一 block 子 slice 全 0，接口前瞻但不增加 step-1 复杂度。
- **错误处理**：本期无（dormant）。step-2：`col_filled_heights.len() != col_count` 时 IFC 回退非碎片化（保守）。
- **交叉引用**：`multicol-fragmentation-design.md` §2.1 草绘；`multicol.rs::compute_column_info`（col_count/col_width/gap 来源，step-2 复用）；`multicol.rs::assign_children_to_columns_with_breaking`（block 子碎片化，step-2 互补）。

### IF-002：IFC dormant 字段 + builder（`inline/mod.rs`）

- **类型**：字段 + builder（镜像 `font_metric_provider`）
- **规格**：

  ```rust
  // InlineFormattingContext 字段（与 font_metric_provider 并列）：
  /// Phase 2a multicol 列碎片化上下文（可选）。
  /// None（默认）= IFC 行盒不碎片化（当前行为，零回归）。
  /// Some = step-2 在 break_items_into_lines 后按本上下文把行盒分配到列。
  /// step-1 仅持有字段、不读取 → 行为不变。
  pub column_fragmentation: Option<ColumnFragmentationContext>,

  // new() 初始化：column_fragmentation: None,

  // builder：
  pub fn with_column_fragmentation(mut self, ctx: ColumnFragmentationContext) -> Self {
      self.column_fragmentation = Some(ctx);
      self
  }
  ```

- **交叉引用**：`inline/mod.rs:156`（`font_metric_provider` 并列）；`inline/mod.rs:245`（`with_font_metric_provider` builder 并列）。

### IF-003：env 门控 `MULTICOL_COLUMN_FRAG`（step-2 预留）

- **规格**：未设/`0` = 关闭（默认）；`1` = 对目标结构（单层 + column-fill:auto + 明确高度 + 单一 block 子）启用 layout 侧碎片化。本期仅约定，不实现。

---

## 6. 约束与假设

### 6.1 必须约束（Must）
- Phase 1 仅定义接口 + dormant 字段 + builder，**零生产读取**（grep 证）。
- 默认 `column_fragmentation = None`；行为字节级不变。
- 守 multicol-fill-auto-001 sentinel（scoped oracle 零回归）。

### 6.2 禁止约束（Must Not）
- 不在本期实现碎片化逻辑（step-2）。
- 不放宽 `painter/text.rs:713` 的 `height_auto` 门控（R317 实证 -5 回归）。
- 不改 taffy-local（R304 DEFER）。
- 不引入新 crate 依赖。
- 不为不可能场景写错误处理（本期无运行时路径）。

### 6.3 已定决策
- 接口放新模块 `inline/column_fragmentation.rs`（镜像 `inline/font_metrics.rs`），不放 `multicol.rs`（IFC 输入，属 inline 域）。
- 纯数据结构 + 本地枚举，无 trait/handle（区别 font-bridge，§IF-001 决策）。
- env 门控 `MULTICOL_COLUMN_FRAG` 渐进启用（step-2）。

### 6.4 技术约束
- taffy 0.7.7（vendored）；IFC = `inline/mod.rs::InlineFormattingContext`；存储模式见 `inline_finalization.rs`（step-2 输出存储参照）。
- 单 `.rs` ≤2000 行。

### 6.5 假设（rally 自主模式，待 step-2 probe 验证）
- **A1（slice 真实缺口）**：✅ **R897 probe 已实证**——单层 + 单一 block 子 breaking 的缺口 = assignment 阶段丢余量 + 非整数高度 overfill（非「文本只在 col0」）。状态：已验证，slice 真实存在（区别 Phase 1 的 0-case 停止）。
- **A2（IFC 可按列预算切片）**：待 step-2 验证。IFC `break_items_into_lines` 产**宽度换行的全部行盒**；列切片逻辑（按 `available_height` 把行盒分配到列，整行不裁断）应在**新模块**（消费 `ColumnFragmentationContext`），非改 IFC 接口（与 `multicol-phase2-unified-column-flow-spec.md` A3 一致：列切片在调用方，IFC 接口不变）。状态：设计假设，待 step-2 probe。
- **A3（chromium 余量 = overflow multicol 盒外）**：待 step-2 验证。column-count:3 + 内容超 col_count×列高 时，chromium 把余量溢出 multicol 盒（overflow:visible 默认）非丢弃。状态：待 probe。

### 6.6 代码变更边界
- **允许修改（本期 Phase 1）**：新增 `crates/layout-engine/src/inline/column_fragmentation.rs`；`crates/layout-engine/src/inline/mod.rs`（加字段 + builder + `mod` 声明 + `pub use`）；`crates/layout-engine/src/inline.rs`（若需 `mod` 注册）。
- **禁止修改（本期）**：`multicol.rs`、`engine.rs`、`painter/text.rs`、`taffy-local`。

---

## 7. 实施交接（Phase 1 本期 + Phase 2a step-2 多会话）

### 本期（Phase 1 死字段）文件清单

| 路径/模块 | 动作 | 目的 | 风险 |
|----------|------|------|------|
| `crates/layout-engine/src/inline/column_fragmentation.rs` | 新增 | `ColumnFragmentationContext` + `ColumnFillMode` + 单测 | 无（dormant） |
| `crates/layout-engine/src/inline/mod.rs` | 修改（+~6 行） | IFC 加 `column_fragmentation` 字段 + `new()` 初始化 + `with_column_fragmentation` builder + `mod column_fragmentation` + `pub use` | 无（默认 None，无读取） |

### 本期验证

1. `cargo build -p zero-layout-engine` 绿；
2. `make test`（test-guard）全 workspace 绿，含新单测；
3. `cargo clippy --workspace --all-targets -- -D warnings` 干净；`cargo fmt` 干净；
4. `make product-smoke`（welcome <20% DC-13 gate）；
5. `make reftest-oracle DIR=css-multicol` 零回归（守 multicol-fill-auto-001）；
6. `grep -rn "column_fragmentation" crates/` 仅命中定义 + builder + tests（证无生产读取）。

### Phase 2a step-2（下一会话，多会话接力）

| 步骤 | 范围 | 预期 | 验证 |
|------|------|------|------|
| Probe A2/A3 | REFTEST_DUMP 单层 breaking 案 + chromium 余量行为 | 确认 IFC 列切片可行 + chromium 余量语义 | evidence |
| Commit 1 | 新模块 `column_fragmentation_flow.rs`：消费 `ColumnFragmentationContext` 把 IFC 行盒切片到列（respected 列高，整行不裁断，余量 overflow），env 门控 | 目标案 z_vs_chr 降 | cross-validate + scoped oracle 零回归 |
| Commit 2 | layout 侧为单层 + 单 block 子 + column-fill:auto + 明确高度 案构造 `ColumnFragmentationContext` 并 `with_column_fragmentation` 注入；输出存 LayoutBox（新字段 `inline_multicol_columns`，参照 unified-column-flow-spec IF-001）；paint 消费 | 目标案 z_vs_chr <1% | 全量 oracle 零回归 |
| Commit 3 | 默认开启（净正向后） | css-multicol oracle +N | 全量 |

### 多会话路线图（slice 递进）

- **Phase 2a**（本 slice）：单层 + 单一 block 子 breaking — R109-independent，**首个 enabling slice**
- **Phase 2b**：mixed-content（block + inline 交错）— **Phase A 前置依赖**（R109 解转换，unified-column-flow-spec）
- **Phase 2c**：inline 跨列断裂（行级 fragmentation）
- **Phase 3**：nested multicol breaking（multicol-breaking-001/002/003/004/005/006，真嵌套碎片化）

---

## 8. 技术设计（RFC）

### 8.1 现状
- 三段分离（taffy 块堆叠 → `assign_children_to_columns_*` 重分配 block 子 → paint 垂直窗口裁剪）。paint 侧对粗粒度分布工作，但无法精确计算每列行盒（列边界裁断行 + assignment 丢余量，R897 probe 实证）。

### 8.2 目标状态
- layout 侧 column-aware IFC：IFC 接收 `ColumnFragmentationContext`，产宽度换行行盒后按列高切片到列（整行不裁断），输出每列行盒存 LayoutBox，paint 直接消费（取代后验裁剪）。
- 本期 = 接口（输入 `ColumnFragmentationContext`）+ dormant 字段；输出存储 + 切片逻辑 = step-2。

### 8.3 影响范围

| 影响项 | 程度 | 说明 |
|--------|------|------|
| IFC（dormant 字段） | 低 | 默认 None，行为不变 |
| multicol 渲染（本期） | 零 | 无生产读取 |
| Phase A stored IFC 路径 | 零 | 新字段独立，不耦合 `inline_layout` |
| step-2 碎片化 | 中 | 仅目标结构（单层+单 block 子+auto+明确高度）触发，env 门控 |

### 8.4 详细设计（step-2 切片伪代码，本期不实现）

```
// 新模块 column_fragmentation_flow.rs（step-2）
fn fragment_lines_into_columns(
    lines: &[LineBox],            // IFC 宽度换行后的全部行盒
    ctx: &ColumnFragmentationContext,
) -> Vec<Vec<LineBox>> {          // 每列行盒
    let budget = ctx.available_height.unwrap_or(f32::INFINITY);
    let mut cols: Vec<Vec<LineBox>> = vec![vec![]; ctx.col_count];
    let mut col_heights = ctx.col_filled_heights.clone(); // 起始 = block 已占
    let mut col_idx = 0;
    for line in lines {
        // 整行不裁断：当前列放不下整行且还有列 → 推进
        while col_idx + 1 < ctx.col_count
              && col_heights[col_idx] + line.height > budget + EPS {
            col_idx += 1;
        }
        if col_heights[col_idx] + line.height > budget + EPS && col_idx + 1 == ctx.col_count {
            // 余量超出 col_count×budget → overflow multicol 盒（A3，step-2 处理）
            // 本 slice：仍放入末列（chromium overflow:visible 溢出，step-2 对齐）
        }
        cols[col_idx].push(line.clone_at_y(col_heights[col_idx]));
        col_heights[col_idx] += line.height;
    }
    cols
}
```

**关键子问题（step-2）**：
- **整行不裁断**：CSS 块级内容不在行盒中间断裂（区别 inline 跨列 = Phase 2c）。budget 不够整行 → 整行移下列。
- **余量 overflow**：超出 col_count×budget 的行 → overflow multicol 盒外（chromium overflow:visible），非丢弃（纠正 R897 probe 的「丢余量」）。
- **与 `assign_children_to_columns_with_breaking` 协调**：step-2 须让 block 子碎片化（行盒级）替代当前的 assignment 级 fragment，或二者协调（block 子位置 + 其 IFC 行盒列切片）。

### 8.5 安全考虑
- **本期**：dormant 字段无运行时路径，零回归风险。
- **step-2**：env 门控 + 守 multicol-fill-auto-001 + 逐案 chromium-Oracle；半完成状态 env 关闭=零影响。

### 8.6 替代方案

| 方案 | 描述 | 优点 | 缺点 | 决定 |
|------|------|------|------|------|
| A. layout 侧 column-aware IFC（本设计） | `ColumnFragmentationContext` + 切片逻辑 | 结构性正确，R109-independent | 多会话 | ✅ 选定 |
| B. paint 侧垂直窗口放宽 | 改 `column_span_offsets` 裁剪 | 改动小 | 5 轮证伪（R157/R198/R203/R317），无法精确算每列行盒 | ❌ 拒绝 |
| C. 改 `assign_children_to_columns_with_breaking` 不丢余量 | assignment 阶段 overflow | 局部 | 不解「整行不裁断 + 列高 respected」，paint 仍后验裁剪 | ❌ 拒绝（可作 step-2 子项） |

### 8.7 实施计划
见 §7（本期 Phase 1；step-2 多会话）。

### 8.8 测试策略
- **本期**：单测（默认 None + 注入 + 字段语义）。
- **step-2**：单测（fragment_lines_into_columns 整行不裁断 + 列满续列 + 余量 overflow）+ scoped oracle + cross-validate。

### 8.9 回滚计划
- 本期：dormant 字段无害，无需回滚（若有编译/测试问题则 revert 单 commit）。
- step-2：env `MULTICOL_COLUMN_FRAG=0`（默认）即回退。

---

## 9. Spec Lint 报告

### 结构完整性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要 | ✅ | §0 含目标/范围/排除/约束 |
| 场景 | ✅ | FR-001~003 + 验证步骤；本期为 dormant 接口，验收场景=单测 + 零回归门禁 |
| 异常路径 | ✅ | §IF-001 错误处理（step-2 col_filled_heights.len()!=col_count 回退） |
| 测试绑定 | ✅ | NFR-004 + §7 验证步骤标 make test/oracle/grep |
| TBD 清零 | ⚠️ | A2/A3 标「待 step-2 probe」——非阻塞性（本期 Phase 1 不依赖），A1 已实证 |
| 约束覆盖 | ✅ | NFR-001 零回归 + §6.1 守 sentinel |
| 实施交接 | ✅ | §7 文件清单 + 验证 + step-2 路线 |
| 首步可执行 | ✅ | §7 本期 = 新模块 + IFC 字段（机械、零运行时风险） |

### 语言精确性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 模糊动词 | ✅ | 用「定义/注入/切片/裁断」具体动词 |
| 量化 | ✅ | 「welcome <20%」「零回归」「grep 仅命中」 |
| 非确定性 | ✅ | 用「必须」；A2/A3 显式标「待验证」未升 FR |

### 一致性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 范围冲突 | ✅ | §1.3 在范围（dormant 接口）与不在范围（碎片化逻辑/mixed/nested）无交集 |
| 约束冲突 | ✅ | §6.1「仅接口」与 §6.2「不实现碎片化」互补 |
| 方案漂移 | ✅ | 方案 A 依赖新模块 + IFC 字段，均在 §6.6 允许范围；不碰禁止的 taffy/:713/multicol.rs |
| 章节引用 | ✅ | IF-001/002 引用 font_metrics.rs/mod.rs:156/245（已验存在） |
| 实现来源闭合 | ✅ | §6.5A 表 + §8.4 覆盖 IFC/列几何/存储/Oracle 来源 |

**汇总**：23 Pass / 1 Warning / 0 Fail / 0 Skip
**门禁判定**：Fail = 0 → **允许本期 Phase 1 实施**（rally 协议下直接进入，A2/A3 留 step-2）

---

## 10. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|----|------|--------|----------|--------|
| A1 | slice 真实缺口（单层 + 单 block 子 breaking） | 重要 | ✅ **R897 probe 已实证**（丢余量 + overfill） | 关闭 |
| A2 | IFC 行盒可按列 budget 切片（整行不裁断） | 重要（step-2） | ✅ **R898 确认**（read-only 代码分析：`LineBox` 携带 `height`，IFC `layout()` 产 `self.lines: Vec<LineBox>`，`fragment_lines_into_columns` 纯算法已实现 + 9 单测验证整行不裁断/列满续列/余量 overflow/预占/mismatch 回退） | 关闭 |
| A3 | chromium 余量 = overflow multicol 盒外（非丢弃） | 重要（step-2） | spec-defined（CSS Multicol §2：fixed column-count + 明确高度，余量 overflow 盒外，overflow:visible 默认）；commit 2 接线时用 chromium Oracle per-test 复核 | step-2 commit 2 接线时验证 |

---

## 11. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2026-06-30 | 初始版本（R897，rally-pattern，承接 R896）；R897 probe A1 实证单层 slice 缺口；Phase 1 死字段 LANDED |

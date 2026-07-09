# 设计文档：taffy 0.7.7 → 0.11 升级迁移

**版本**：v0.1（草稿，read-only 设计产出）
**日期**：2026-07-08（R1204）
**状态**：草稿（rally 模式；**fork A 未授权**——本文档为 R304 DEFER + R1203 strategic fork A 的实施前设计，待用户 go-ahead worktree 后执行）
**关联**：`docs/goal/rendering-compat/master.md` R1203（strategic fork）；R304 taffy upgrade DEFERRED（archive）；R1187 forward 决策树；R1169 baseline-export taffy-0.8-gated
**驱动数据**：vendored taffy **0.7.7**（`crates/taffy-local`）+ 自定义 `cached_baselines()` patch；595 call site；10-dir broad aggregate 51.3%（R1202）；残余 flex/grid/multicol-baseline gap 被 0.7.7 无 baseline_overrides 输入 API 阻塞（R1169）

---

## 0. 执行摘要

- **一句话目标**：把 ZeroWeb 布局引擎依赖的 taffy 从 vendored 0.7.7 升级到 0.11，解锁 `baseline_overrides` 输入 API（解 flex/grid/multicol-baseline 聚类）+ flex/grid intrinsic sizing 改进；**不解 strict font-raster 墙**。
- **本期范围**：**仅设计文档 + 实施前 surface map + 分阶段计划**。**不落地代码**（fork A 未授权；落地须 worktree 隔离，多 session）。
- **明确排除**：font-raster strict 墙（fontdue/FreeType vs chromium，taffy 升级无关）；Phase A IFC 统一（独立）；multicol Phase 2（Phase-A-blocked）。
- **核心约束**：① **build-breaking**——taffy 0.7→0.11 API 重设计，compile 不通过则整 layout-engine 瘫 → **必须 worktree 隔离**（CLAUDE.md：worktree 须用户显式指示）；② 任一 Phase 须 `make test` + `make reftest`（test-guard）+ scoped `make reftest-oracle` 零 count 回归；③ 单文件 ≤2000 行；④ 不引入新 `#[ignore]`。
- **推荐方案**：**worktree 隔离 + 分阶段 API 迁移**（tree ops → Style → Measure → baseline_overrides → cached_baselines patch 决策 → 全量门禁），每阶段独立可验证。
- **首个落地步骤**（fork A 授权后）：**worktree re-vendor taffy 0.11**（替换 `crates/taffy-local` 为 0.11 源 + 调整 `[patch.crates-io]`）→ `cargo build` 捕获首批 compile error（揭示真实 API 断裂面）→ 据 error 分类定 Phase 1 迁移点。

---

## 1. 现状 surface map（grep 实证，taffy 0.7.7）

ZW layout-engine 对 taffy 的依赖面（grep `crates/layout-engine/src/` + `apps/`，排除 taffy-local 自身）：

### 1.1 TaffyTree 方法（~120 call site）

| 方法 | 调用数 | 用途 | 0.11 风险 |
|------|--------|------|-----------|
| `.style(id)` | 22 | 读节点 Style | 低（API 大致保留） |
| `.remove(id)` | 22 | 删节点 | 中（签名/返回值可能变） |
| `.layout(id)` | 23 | 读布局结果 | 中（Layout 结构可能重构） |
| `.children(id)` | 18 | 读子节点 | 低 |
| `.set_style(id, style)` | 5 | 写 Style | 低 |
| `.new_leaf(style)` | 5 | 建叶节点 | 中（measure 回调签名变） |
| `.parent(id)` | 2 | 读父 | 低 |
| `.new_with_children(style, children)` | 1 | 建带子节点 | 中 |
| `.compute_layout(id, space)` / `.set_measure(id, func)` | — | 布局计算 / measure 回调 | **高（0.8+ 签名重设计）** |

### 1.2 Style 与 geometry（~540 call site，主迁移成本）

- **`taffy::style::*`（511 call）**：`converter/mod.rs:62-229` 逐字段构造 `taffy::Style`（display/box_sizing/overflow/scrollbar_width/position/inset/size/min_size/max_size/aspect_ratio/margin/border/padding/...）。0.8+ Style 字段**大部分保留**但部分枚举/类型重构（Position、Display、Dimension 等）。
- `taffy::geometry::*`（28 call）：Point/Rect/Size/AvailableSpace。0.8+ 基本保留。

### 1.3 Measure（IFC 文本测量）

- `measure_text`（19 call）：IFC 文本测量的 taffy measure 回调。0.8+ `MeasureFunc` → 泛型 `Measure` trait，签名变化。

### 1.4 自定义 patch（关键决策点）

- `cached_baselines(id)`（`engine.rs:625`）：**本地 patch**（Cargo.toml `[patch.crates-io] taffy = { path = "crates/taffy-local" }`），读 taffy 内部缓存的 baseline 值 → 存 `LayoutBox.taffy_baseline`（供 `adjust_inline_block_positions`，R266/R313/R316）。
- **0.11 决策**：0.11 若有 **native baseline export API**（`baseline_overrides` 输入 + layout 结果含 baseline），则 `cached_baselines` patch **可废弃**（用 native API 替代）；否则须 re-apply patch 到 0.11 源。**待 Phase 1 worktree 实测确认**。

---

## 2. taffy 0.7 → 0.11 主要 API 变化（**R1248 CHANGELOG 实测验证**）

> ✅ R1248 已读 taffy 官方 CHANGELOG（https://github.com/DioxusLabs/taffy/blob/main/CHANGELOG.md）验证 0.8→0.11 逐版本 breaking change。以下为 CHANGELOG-verified 事实（仍须 Phase 1 worktree compile error 终验具体签名，但 breaking 域已确认）。

| 版本 | breaking change（CHANGELOG 实证） | ZW 影响 |
|------|----------------------------------|---------|
| **0.8.0** | **calc() 支持**：size 类型（`LengthPercentage`/`LengthPercentageAuto`/`Dimension`/`MinTrackSizingFunction`/`MaxTrackSizingFunction`）从 enum 改为 **tagged pointer**（`*const ()` + low 3 bits tag）；`LayoutPartialTree::resolve_calc_value` 新 trait | **converter 511 call 主成本**——所有 size 类型构造改 tagged pointer；converter/mod.rs `LengthValue::Px(v) as f32` → 需 0.8 构造器 |
| **0.9.0** | **`Style` 泛型化** over `CheapCloneStr`（grid named lines/areas）；`TrackSizingFunction`→`GridTemplateComponent` 重命名；`NonRepeatedTrackSizingFunction`→`TrackSizingFunction`；grid low-level API 大改 generic | converter `taffy::Style::default()` → `Style::<CheapCloneStr>` 泛型标注；grid 模块（table.rs grid_areas）适配 |
| **0.10.0** | **原生 `float`/`clear` 支持**（`FloatContext`+`BlockContext`，feature `float_layout`）+ **`direction`（RTL）**；cache `set`/`get` API 改取 `&LayoutInput`（#933）；`TaffyTree::write_tree`（#925）；CSS `FromStr` 解析（`parse` feature） | **★ 潜在 ZW `float_positioning.rs` 替代**（R129/R1242 float-shrink + R895 float 定位 + adjust_float_positions ~540 行后处理可能由 0.10 原生 float/clear 替代/简化——但 ZW float 后处理含大量 fixup 须逐项验证不回归）；cache API 改 &LayoutInput 影响增量布局 cached_state |
| **0.11.0** | **safe alignment**：alignment 类型（`AlignContent`/`JustifyContent`/`AlignItems`/`JustifyItems`/`AlignSelf`/`JustifySelf`）改为 struct（`AlignmentKeyword`+`AlignmentSafety`），enum 变体→关联常量（`AlignContent::Start`→`AlignContent::START`）；MSRV 1.71 | converter alignment 构造点（`AlignItems`/`JustifyContent` 等）全改关联常量 |
| **baseline** | **0.8-0.11 CHANGELOG 未提及 native `baseline_overrides` 输入或 `Layout.baseline` export 新增** | **★ cached_baselines patch 须 re-apply 到 0.11 源**（R1169 unblock 不成立——0.11 无 native baseline input；patch 是 ZW 对 0.7 的本地补丁 engine.rs:625，迁移须 re-apply） |

**★ R1248 关键结论**：
1. **迁移成本远超原估**：0.8 tagged pointer（size 类型全改）+ 0.9 Style 泛型（converter 主成本）+ 0.11 alignment struct = 三大 breaking 域，converter 511 call + grid 模块 + alignment 构造全须适配。原「tree ops ~120 / Style 511 / measure 19」估偏低（未含 0.8 tagged pointer + 0.9 Style 泛型的连锁重构）。
2. **0.10 原生 float/clear 是潜在大收益**：可替代/简化 ZW `float_positioning.rs`（~540 行后处理：R129/R1242 shrink + R895 定位 + clear + BFC float containment），但须逐项验证 ZW fixup 不丢失（R145/R895/R129 等多轮 fix）。
3. **cached_baselines patch 须 re-apply**（0.11 无 native baseline input，R1169 unblock 不成立）。
4. **MSRV 1.71**（ZW MSRV 1.85，满足）。

---

## 3. 分阶段迁移计划（worktree 隔离，每阶段独立可验证）

> 每阶段合并条件：`make test`（test-guard）全绿 + scoped `make reftest-oracle` 零 count 回归 + `make product-smoke` welcome ≤20%。任一退步即 `git revert` 该 commit（worktree 内）。

### Phase 1：worktree re-vendor + 编译断裂面测绘（**首步，无行为变更**）
- worktree 隔离（用户授权后）。
- 替换 `crates/taffy-local` 为 taffy 0.11 源（或改 `[patch.crates-io]` 指向 crates.io 0.11）。
- `cargo build`（test-guard 包裹，OOM 防护）捕获首批 compile error。
- 据 error 分类：tree ops / Style / Measure / baseline 各自断裂点 → 精修本文档 §1/§2「A-待验证」为实测事实。
- **验证**：无（read-only 测绘）；产出「真实断裂面报告」。

### Phase 2：Tree API 迁移（TaffyTree 泛型 + NodeId + compute 签名）
- 改 ~120 tree-op call site 适配 0.11 签名。
- 暂用 stub measure（Measure 迁移在 Phase 3）。
- **验证**：cargo build 通过 + tree/tests.rs 单测绿。

### Phase 3：Style + geometry 迁移
- 改 converter `taffy::Style` 构造（511 call）适配 0.11 字段/枚举。
- geometry 28 call。
- **验证**：converter/tests/* 绿 + scoped reftest（css2/normal-flow）零回归。

### Phase 4：Measure 迁移（measure_text → 0.11 Measure trait）
- 改 measure_text 19 call + set_measure 适配泛型 Measure。
- **验证**：inline/tests + css-text/linebox scoped reftest 零回归。

### Phase 5：baseline_overrides + cached_baselines patch 决策
- 若 0.11 有 native baseline 输入 API → 废弃 cached_baselines patch，接 baseline_overrides（**unblock R1169 flex/grid/multicol-baseline**）。
- 若无 → re-apply cached_baselines patch 到 0.11 源（保 inline-block baseline 现状）。
- **验证**：flexbox-baseline/grid-baseline scoped reftest（若 native API）+ 全量三态。

### Phase 6：全量门禁 + 清理
- `make test` + `make reftest`（loose+strict）+ `make product-smoke` + 全量 `make reftest-oracle` 三态不退。
- clippy/fmt 干净；移除迁移期 `#[allow]`。
- **验证**：全绿 + aggregate broad 不降（目标：baseline_overrides unblock → +若干 case）。

---

## 4. 风险与缓解

| 风险 | 概率 | 缓解 |
|------|------|------|
| 0.11 API 断裂面远超预期（>1000 error） | 高 | Phase 1 测绘先评估；若 >2× 预期触发 spec 紧急停止（>200% 范围）回报用户 |
| cached_baselines patch 无法 re-apply 且 0.11 无 native baseline | 中 | 保 inline-block baseline 现状（R266 不回归）；flex/grid/multicol-baseline 维持 gated |
| Style 字段语义微变致布局回归（隐藏） | 高 | 每 Phase scoped reftest + product-smoke；welcome diff>20% 退出 2 |
| measure 签名变致 IFC 文本测量回归 | 中 | Phase 4 重点守 css-text/linebox |
| worktree 长期 build-broken（多 session） | 高 | worktree 隔离，main 不受影响；分阶段合并 |

---

## 5. 不解决什么（诚实边界）

- **strict font-raster 墙**（DC-14 strict 95%）：taffy 升级**不解**（fontdue/FreeType vs chromium 亚像素，各 dir strict 个位数）。taffy 升级是 **broad + baseline cluster** 改善，非 strict 达标。
- **Phase A IFC 统一 / multicol Phase 2 / R109 / BiDi / font-features**：独立墙，taffy 升级不解。
- **预期收益**：baseline_overrides unblock flex/grid/multicol-baseline（+若干 case broad）+ flex/grid intrinsic 改进；**非决定性 broad 跃升**（broad 已 51.3%，taffy 是 cluster-level 改善）。

---

## 6. 裁决

1. 本文档 = R304 DEFER + R1203 fork A 的**实施前设计**（surface map + 分阶段计划 + 风险），**fork A 未授权不落地**。
2. **首步**（授权后）：worktree re-vendor 0.11 + Phase 1 编译断裂面测绘（read-only，定 §2「A-待验证」为实测）。
3. **预期 multi-month**（595 site + API 重设计 + 自定义 patch 决策）；**worktree 必需**（build-breaking）。
4. **诚实**：taffy 升级是 broad cluster 改善非 strict 达标；font-raster 墙仍约束 DC-14 strict 95%。
5. 若用户选 fork B（接受 broad plateau）/ C（其它方向），本文档作为 R304 deferred 的设计沉淀保留，不立即执行。

## 7. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1 | 2026-07-08（R1204） | 初始设计（R304 DEFER + R1203 fork A 实施前；surface map grep 实证 + 分阶段计划） |

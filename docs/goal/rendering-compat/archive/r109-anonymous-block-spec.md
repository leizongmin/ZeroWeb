# Spec：R109 §9.2.1.1 匿名块盒生成 + 高度回填 + bg/border 协调

**版本**：v1.0（**SUPERSEDED — 见顶部裁决**）
**日期**：2026-07-02
**作者**：AI Assistant（rally R937）
**状态**：⚠️ **SUPERSEDED（R2007，2026-07-24）—— 勿按本 spec 实施**。R937 起草时 box-display 高 diff 簇（insert-/delete- 16-42%）被假定为静态 R109 paint 问题；R1162 re-scope + R1878 empirical probe + **R2007 box-display oracle 复核**确证：该簇**全部是 JS-driven 动态测试**（`class=reftest-wait` + `appendChild` script），须 **JS-DOM-bridge**（reftest harness 执行页面 JS 到终态）才能 flip，**非 R109 匿名块 paint/height 修复**（paint 修复对 JS-driven case 零 yield）。静态 block-in-inline 簇（block-in-inline-001/002/003）在 0.00-0.28% = font-wall near-pass（R1155）。故本 spec 的 FR-001 高度回填 / FR-002 bg / FR-003 border **对当前 box-display 失败簇零 EV**（目标全 JS-driven 或 font-wall）。R1878 亦证 R109 core split（block-in-inline-001 正确拆 3 行）已工作，残余 = entangled edge cases（table/float/relpos/JS dynamic）。**R109 作为 rendering-compat clean lever definitive 关闭**；box-display 高 diff 簇的真 unlock = JS-DOM-bridge（大型 borderline-scope 工作流，Mission 定 JS/DOM API compat 属父目标 zero-web.md，非本 goal）。本 spec 保留作历史追溯，**勿据其 Batch 1（height backfill）开工**。

> 本 spec 把 R764 读码分析 + R929–R936 五轮调查（font-metric 三角度证伪 + 像素 forensics + R109 toggle A/B）收敛为可实施的多会话计划。证据链见 `master.md` R764/R929–R936。

> 本 spec 把 R764 读码分析 + R929–R936 五轮调查（font-metric 三角度证伪 + 像素 forensics + R109 toggle A/B）收敛为可实施的多会话计划。证据链见 `master.md` R764/R929–R936。

---

## 0. 执行摘要

> **🔧 R1162（2026-07-08）re-scope**：原头号目标 `insert-block-in-inlines-{beginning,middle,end}-001` 簇经核验**全 JS-driven**（`class="reftest-wait"` + `appendChild` script，flags "ahem dom"）——FR-002/FR-003 paint 修复**不会 flip 这些 case**（属 JS-DOM-bridge 范畴，非 R109 匿名块 paint 问题）。spec 的 yield 前提需修正：**R109 FR-002/FR-003 的真目标 = 静态 block-in-inline near-pass 簇**（box-display 8+ 案 1-3%：`block-in-inline-with-padded-parent`(2.88%)、`block-in-inline-relpos-001/002`(2.56/2.20%)、`block-in-inline-margin-collapses-through-intervening-float`(2.40)、`block-in-inline-margin-collapses-through-multiple-floats`(2.25)、`block-in-inline-negative-margin-with-intervening-float`(2.11)、`three-block-in-inlines-cascading-margins`(2.14)、`block-in-inline-margin-with-line-break-then-block-in-inline`(1.86) 等）。这些是静态 R109 §9.2.1.1 case（flags "ahem" 无 JS），FR-002/FR-003 可 flip。原 insert-block-in-inlines 簇（end 18.71% / middle 16.84% / beginning 6.29%）降级为 JS-DOM-bridge 后续工作。

- **一句话目标**：让 §9.2.1.1 匿名块盒（inline 被块子元素拆分 / block 容器混合 inline+block 子元素）在生成后正确参与容器测高、bg 涂满、border 归属，消除 **静态 block-in-inline near-pass 簇**（R1162 re-scope 自原 insert-block-in-inlines / margin-collapse-101）的 ~2-10% 结构性 diff。
- **本期范围**：仅 R109 匿名块盒的**生成后处理**（高度回填 + 容器 bg 涂满 + border 归属协调），不改触发门控逻辑、不重写 IFC、不动 font 度量。
- **明确排除**：font-engine / Phase A 行盒度量统一（welcome/morning 16–17%，独立多会话）；multicol Phase 2；taffy 升级（R304）。
- **核心约束**：① 必须保持 R109 ON 的 net +5（box-display 32 vs OFF 27，R936 实测）；② 不得回归 margin-collapse-101 / inline-box-001 / multicol-block-no-clip-001（R743/R744 历史）；③ 改动经 `make reftest-oracle` + `make product-smoke` A/B 零回归。
- **推荐方案**：在现有 `compute_block_container_split` / `compute_inline_block_split` 生成匿名块盒后，新增**匿名块盒内容高度回填**步骤（让匿名块盒的 IFC 内容高度写回其 taffy 测高），并修容器 bg 在匿名块盒/margin 区的涂布；border 归属按 §9.2.1.1（被拆分 inline 的 border 在 inline 级各匿名块绘制）。
- **首个落地步骤**：先用 LAYOUT_DUMP/probe 确认匿名块盒的 taffy 测高是否为 0（验证 R935 症状 b 根因），再决定高度回填的实现入口（`inline_finalization.rs::store_font_sizes_from_ifc` 邻近或 engine 后处理）。

---

## 1. 背景与目标

### 1.1 背景

CSS2 §9.2.1.1 规定两类匿名块盒生成：
- **case (a)**：inline 元素含 in-flow block-level 子元素 → inline 被拆分为匿名块盒序列（`is_inline_r109`，`compute_inline_block_split`）。
- **case (b)**：block 容器含混合 inline+block 子元素 → inline 子元素被匿名块盒包裹（`is_block_mixed`，`compute_block_container_split`）。

ZeroWeb 已在 `crates/layout-engine/src/tree.rs:571` 实现两者（env `R109_WIRE`，`r109_wired()` 默认 TRUE，仅 `=0` 关闭）。但生成后的匿名块盒**未正确参与容器布局/绘制**，导致：

- **insert-block-in-inlines 簇**（case b，box-display worst 12 案）~20% diff（R935 像素 forensics）：容器 fuchsia bg 仅 1/4 面积且 x 起点 wrong（168 vs 28）、容器高度 174 vs chr 233、margin/匿名块区露白、border 错位。
- **margin-collapse-101 簇**（case a，R702，7 案）：trailing inline「B」丢失 + 空 .red 的 bg:red 可见性 + bar 着色发散（R743/R744）。

R932–R934 三角度（line-height 行为 / 存储 / 计算值）证伪了 font-metric 归因——container span 在 layout+paint 两路径均正确 (20,20)。R936 R109 toggle A/B 证实：R109 ON 是 net +5 正确默认（勿关），insert-* 是 R109-无关结构性墙（ON/OFF 都 ~20%）。**根因在匿名块盒生成后的高度/bg/border 协调，非 font、非门控。**

### 1.2 目标

- **业务目标**：提升 css/CSS2/box-display（当前 32/120=27%）+ margin-padding-clear（margin-collapse-101 簇）chromium-Oracle 一致率。
- **用户目标**：让 §9.2.1.1 涉及的常见静态页结构（block 容器内夹 inline 文本 + block 子元素）渲染与 chromium 一致。

### 1.3 范围边界

- **在范围内**：
  - 匿名块盒**内容高度→容器 taffy 测高**回填（case a + case b）。
  - 容器 bg 在匿名块盒 / margin 区的涂布（paint 侧）。
  - 被拆分 inline 的 border 归属（case a，§9.2.1.1 inline 级）。
  - 逐案 A/B 守 R743/R744 回归。
- **不在范围内**：
  - R109 触发门控逻辑改动（`inline_has_block_child` / `block_container_has_mixed_content` 判定保持）。
  - IFC 重写 / Phase A 行盒度量统一（line-height/baseline/strut 真字体度量）。
  - multicol fragmentation（Phase 2）。
  - font-engine 投资（welcome/morning 字体墙）。

### 1.4 关键假设（待验证）

- **假设 A1（症状 b 根因）**：~~匿名块盒 taffy 测高 0 + compute_final 跳过它~~ **【R938 已验证，部分修正】**：匿名块盒**确实被 compute_final 处理**（node_id = 容器 dom_id 经 taffy_to_dom，doc.get 解析，gate 通过；fragment_node_ids 正确配置 IFC 只收集片段；inline_layout 被存储）。**但 compute_final 末尾（inline_finalization.rs:727 `root.inline_layout = Some(lines)`）从不把 IFC 内容高度回填到 `root.height`**（grep 证：仅 :237 `root.content_height=tallest` 在另一函数）。**且 taffy 测高经 `measure_text_content(ctx_node)`（inline_finalization.rs:736），ctx_node 是片段首个文本节点（tree.rs），多文本节点/多行 run 被欠计**。故匿名块盒 height = taffy 对单文本节点的测量（非完整 inline run 高度）→ 容器排除了部分 inline 高度 → 容器矮 + bg 露白。**FR-001 fix 精确位置 = compute_final IFC 后回填 `root.height = Σ lines.height`，并加容器高度后处理 pass 重算容器（因 compute_final 在 taffy 测高后跑，改 root.height 不自动传播父盒）。**
- **假设 A2**：容器 bg 涂布在匿名块盒/margin 区露白，是因为容器 box 高度算短（A1 后果）+ paint bg 按算短的 box 高度涂。
- **假设 A3**：R743/R744 回归根因是 case (a) 的 collapse-through + 空 .red bg 渲染在新结构下发散（R764 复核），非 case (b)。

---

## 2. 需求类型概览

| 类型 | 是否适用 | 来源 |
|------|---------|------|
| 业务需求 | 是 | 提升 box-display / margin-padding-clear Oracle 一致率 |
| 解决方案需求 | 是 | R764 读码 + R929–R936 调查 |
| 功能需求 | 是 | §3 |
| 非功能需求 | 是 | §4（零回归、确定性） |
| 接口需求 | 否 | 内部布局算法，无新 API |
| 过渡需求 | 是 | 逐案 A/B 守回归，可逐步落地 |

---

## 3. 功能需求

### FR-001：匿名块盒内容高度回填进容器测高（case a + b）
- **描述**：当 §9.2.1.1 匿名块盒生成后，系统必须把该匿名块盒的 IFC 内容高度（含 line-height、inline-block 子盒高度）回填进其 taffy 测高（或等价的容器测高后处理），使容器总高度包含匿名块盒的 inline 内容高度。
- **优先级**：必须
- **来源**：R935 症状 b（容器高 174 vs chr 233）+ 假设 A1

**验收场景**：
```
场景: block 容器混合 inline+block 子元素（case b，insert-block-in-inlines-beginning-001）
  假设 .container 含 [div.inserted][匿名块盒(inline run "Several...")]，font:20px/1 Ahem
  当 ZeroWeb 渲染并对齐 chromium oracle
  那么 容器 fuchsia bg 高度 ≈ chr（ZW 内容 y 末端 ≥ 224，chr=233，差 <12px）；
       bg 左边界 x=28（非 168）；fuchsia 面积 ≥ 60000 px（chr=74240）
  验证: make reftest-oracle DIR=css/CSS2/box-display/insert-block-in-inlines-beginning-001；product-smoke --out + PIL bbox 断言

场景: 容器仅含匿名块盒（无插入块，纯 div>block+text）
  假设 div 含 1 个 block 子 + 直接文本
  当 渲染
  那么 容器高度 = block 子盒高 + 匿名块盒 inline 内容高（非仅 block 子盒高）
  验证: layout-engine 单测 test_anon_block_height_backfilled

场景: 匿名块盒内含 inline-block 子盒（高度由子盒撑高）
  假设 匿名块盒 inline run 含一个 inline-block 高 50px
  当 渲染
  那么 匿名块盒高度 ≥ 50px 并计入容器
  验证: layout-engine 单测 test_anon_block_with_inline_block_child

场景: 匿名块盒 node_id 不解析于 doc.get（合成 id）
  假设 匿名块盒的 node_id 是合成/无对应 DOM 节点
  当 compute_final_inline_layouts 递归到它
  那么 仍回填其高度（不因 doc.get gate 早退）；高度回填路径不依赖 doc.get 解析
  验证: probe 断言匿名块盒 LayoutBox.height > 0 且 > 其 taffy 初始测高
```

### FR-002：容器 bg 在匿名块盒 / margin 区涂满（case b）
- **描述**：当容器含匿名块盒 + block 子盒时，容器自身的 background 必须涂满整个容器 border-box（含匿名块盒区域 + block 子盒的 margin 区），不得露白。
- **优先级**：必须
- **来源**：R935 scanline（y=78-94 露白 vs chr 整行 fuchsia）

**验收场景**：
```
场景: 容器 bg 涂满 margin 区（inserted block margin:1em 0）
  假设 容器 fuchsia bg + div.inserted（margin:1em 0）
  当 渲染 + PIL scanline
  那么 inserted block 的上下 margin 区（chr y=74-90 等行）为 fuchsia（非 white）
  验证: product-smoke --out + PIL 逐行断言无 white-only 行落在容器 bb 内

场景: 容器 bg 涂满匿名块盒区
  假设 容器 fuchsia bg + 匿名块盒 inline 文本
  当 渲染
  那么 匿名块盒区背景 = 容器 fuchsia（透明匿名块盒露容器 bg）
  验证: PIL 断言匿名块盒 bb 内非文本像素 = fuchsia
```

### FR-003：被拆分 inline 的 border 归属（case a）
- **描述**：当 inline 元素被 block 子元素拆分为多个匿名块盒时，该 inline 的 border/background 必须在 inline 级（各匿名块盒片段）绘制，且首/末片段的 border 边按 §9.2.1.1 收缩（`r109_first_fragment` / `r109_last_fragment`）。
- **优先级**：应该
- **来源**：R764（tree.rs:568-630 已实现 shrink + border 边）+ R935 border 错位

**验收场景**：
```
场景: 拆分 inline 的 border 在各片段绘制（inline-box-001 类）
  假设 inline 元素含 border + block 子元素拆分
  当 渲染
  那么 每个匿名块盒片段绘制 inline 的 border（首片段无左边、末片段无右边，或按规范）
  验证: make reftest-oracle DIR=.../inline-box-001 类案；A/B 不回归

场景: 不影响未拆分 inline 的 border
  假设 普通 inline（无 block 子）
  当 渲染
  那么 border 行为不变（回归守卫）
  验证: 全量 box-display / normal-flow oracle A/B 零回归
```

### FR-004：保持 R109 ON net +5 与零回归（守卫）
- **描述**：本次所有改动必须保持 R109 ON 对 css/CSS2/box-display 的 net +5（32 vs OFF 27），且不回归 margin-collapse-101 / inline-box-001 / multicol-block-no-clip-001 / welcome product-smoke。
- **优先级**：必须
- **来源**：R743/R744/R936

**验收场景**：
```
场景: 改动后 box-display Oracle ≥ 32/120
  假设 R109 ON（默认）
  当 make reftest-oracle DIR=css/CSS2/box-display
  那么 oracle-pass ≥ 32（不回归 R936 基线），insert-block-in-inlines 簇 diff 下降
  验证: make reftest-oracle DIR=css/CSS2/box-display

场景: 历史 R743/R744 回归案零翻转
  假设 margin-collapse-101 / inline-box-001 / multicol-block-no-clip-001
  当 A/B（改动前 vs 后）
  那么 三案 z_vs_chr 不上升（不触发 R743/R744 回归机制）
  验证: make reftest-oracle DIR=.../<三案> 逐案 A/B

场景: 产品 smoke 零回归
  假设 welcome.html
  当 make product-smoke
  那么 diff ≤ 20%（不触发 DC-13 门禁退出 2）
  验证: make product-smoke
```

---

## 4. 非功能需求

### NFR-001：零回归（必须）
- **描述**：全量 `make test` + `make reftest` 零回归；涉及布局/渲染变更额外 `make product-smoke` diff ≤ 20%。
- **测量标准**：`make test` 全绿 + `make reftest-oracle` 关键 dir A/B 净 ≥ 0 + `make product-smoke` ≤ 20%。

### NFR-002：渲染确定性（必须）
- **描述**：匿名块盒高度回填不得引入 HashMap 迭代序依赖（同 text.rs:898 的确定性守卫）。
- **测量标准**：同一输入两次渲染 byte-identical（reftest 自源同源对照）。

### NFR-003：逐案可回退（应该）
- **描述**：高度回填 / bg 修复 / border 归属三子项可独立开关（env），便于定位回归。
- **测量标准**：env 可逐项关闭回退到当前行为。

---

## 5. 约束与假设

### 6.1 必须约束（Must）
- 匿名块盒的 IFC 内容高度必须计入容器最终测高（FR-001）。
- 容器 bg 必须涂满整个容器 border-box（含匿名块盒/margin 区，FR-002）。
- 改动必须保持 R109 ON net +5 且零回归 R743/R744 谱系（FR-004）。
- 改动经 `make reftest-oracle` + `make product-smoke` A/B 验证。

### 6.2 禁止约束（Must Not）
- 不得改 R109 触发门控逻辑（`inline_has_block_child` / `block_container_has_mixed_content`）。
- 不得改 font 度量 / line-height 计算（R932–R934 已证 font 正确）。
- 不得为「修 insert-*」而全局关闭 R109（R936 实测 net -5）。
- 不得引入 HashMap 迭代序依赖（破坏渲染确定性）。

### 6.3 已定决策
- 语言/构建：Rust（edition 2024），`make reftest`/`make product-smoke` 包裹入口。
- 复用现有 IFC 基础设施（`InlineFormattingContext`），不重写。
- 复用 taffy 0.7 `CollapsibleMarginSet` margin 折叠（R323 验证），不加新后处理。

### 6.4 技术约束
- 匿名块盒用 `taffy::new_leaf_with_context` 创建（tree.rs:600），taffy 不能测 inline 内容 → 高度须经 ZeroWeb 后处理回填。
- `compute_final_inline_layouts` 当前 gate（inline_finalization.rs:466 `doc.get(node_id)` + :475 `!is_block_level` + :495 混合 inline+block 排除）可能跳过匿名块盒 → 回填路径须绕过这些 gate 或为匿名块盒开专用通道。

### 6.5 实现来源说明

| 能力 | 来源类型 | 具体来源 |
|------|---------|---------|
| 匿名块盒生成 | 复用现有 | `tree.rs:571` `compute_block_container_split` / `compute_inline_block_split`（已实现，不改） |
| IFC 内容高度计算 | 复用现有 | `InlineFormattingContext::layout` + `store_font_sizes_from_ifc`（inline_finalization.rs:252，存 frag.height） |
| 高度回填入口 | 仓内自实现（待定） | 候选：`engine.rs` 后处理 pass（类似 `apply_intrinsic_content_sizing` 两趟基建，R695/R699 复用）或放宽 compute_final gate 让匿名块盒存 inline_layout |
| 容器 bg 涂布 | 复用现有 + 修 | `painter/mod.rs::paint_background`（当前按 box 高度涂，须用回填后的高度） |
| Border 归属 | 复用现有 | `r109.rs::shrink_r109_anon_blocks`（已实现 shrink + first/last border 边） |

### 6.6 代码变更边界
- **允许修改**：`crates/layout-engine/src/engine.rs`（后处理 pass）、`crates/layout-engine/src/inline_finalization.rs`（gate 放宽 / 高度回填）、`crates/engine/src/paint/painter/mod.rs`（bg 涂布用回填高度）、`crates/layout-engine/src/r109.rs`、对应 `tests/`。
- **禁止修改**：`crates/layout-engine/src/tree.rs` 的门控判定（仅可读 anon 生成产物）；`crates/css-parser/`、`crates/style-system/`（font/line-height 不动）；`crates/render-foundation/`（渲染器图元不动）。

---

## 7. 实施交接（Implementation Handoff）

### 文件/模块清单

| 路径/模块 | 动作 | 目的 | 风险 |
|----------|------|------|------|
| `crates/layout-engine/src/inline_finalization.rs:466-520` | 修改（放宽 gate 或加匿名块盒专用通道） | 让匿名块盒被 compute_final 处理，存 inline_layout + 回填高度（FR-001） | 高：R743/R744 回归 |
| `crates/layout-engine/src/engine.rs`（compute 后处理） | 新增 pass | 兜底回填匿名块盒高度到容器测高（若 gate 放宽不足） | 中：两趟基建复用 |
| `crates/engine/src/paint/painter/mod.rs::paint_background` | 修改 | bg 用回填后高度涂满（FR-002） | 低 |
| `crates/layout-engine/src/inline/tests/` | 新增单测 | FR-001/002 验收 | — |

### 推荐修改顺序

1. **验证假设 A1**（零代码）：LAYOUT_DUMP/probe 确认匿名块盒 taffy 测高是否为 0、compute_final 是否跳过它（`doc.get(node_id)` 是否解析）。决定回填入口。
2. **FR-001 高度回填**（先 case b 即 insert-*，隔离风险）：让匿名块盒高度进容器测高。A/B box-display + product-smoke。
3. **FR-002 bg 涂满**：依赖 FR-001（box 高度先对）。A/B scanline 无露白。
4. **FR-003 border 归属**（case a，后做，风险更高）：A/B inline-box-001 + margin-collapse-101。
5. **FR-004 全量回归**：`make test` + 关键 dir reftest-oracle + product-smoke。

### 首批提交建议

| Batch | 范围 | 预期结果 | 验证 |
|-------|------|----------|------|
| Batch 1 | FR-001 高度回填（case b only）+ probe 验证 | insert-block-in-inlines-beginning-001 fuchsia 面积 ↑、高度 ≈ chr | reftest-oracle DIR=css/CSS2/box-display/insert-block-in-inlines-* + product-smoke + PIL |

---

## 8. 技术设计（RFC）

### 8.1 现状分析
- **匿名块盒生成**（`tree.rs:571-640`）：case a/b 均已实现，生成 `taffy::new_leaf_with_context` 匿名块（tree.rs:600），`taffy_to_dom → dom_id`，`fragment_registry → item_node_ids`。`r109_wired()` 默认 TRUE。
- **高度测量缺口**（假设 A1）：匿名块盒是 taffy leaf-with-context，taffy 不能测 inline 内容 → 测高为 0/auto-wrong。`compute_final_inline_layouts`（inline_finalization.rs:384）的 gate（:466 `doc.get(node_id)` + :475 `!is_block_level` + :495 混合内容排除）可能跳过匿名块盒 → IFC 内容高度（frag.height，已由 store_font_sizes_from_ifc:262 存）**不回填到匿名块盒 taffy 测高** → 容器测高排除了 inline run 高度。
- **paint**：paint_text 对匿名块盒走 Path B（R929-R934 证 IFC 内容渲染本身正确，font-metric 全对），但容器 bg（painter/mod.rs::paint_background）按算短的 box 高度涂 → 露白。
- **shrink_r109_anon_blocks**（r109.rs）：已收缩匿名块盒宽到文本 + 处理首/末 border 边（case a border 归属已有基建）。

### 8.2 目标状态
匿名块盒生成后，新增「**高度回填**」：把匿名块盒 IFC 内容高度（max of frag.height per line + padding/border）写回匿名块盒 LayoutBox.height 并 mark_dirty 触发容器重测（或等价后处理）。容器 bg 用回填后的高度涂满。case a 的 border 归属复用现有 r109.rs shrink 基建。

### 8.3 影响范围分析

| 影响项 | 程度 | 说明 |
|--------|------|------|
| css/CSS2/box-display（insert-* 簇） | 高 | FR-001/002 直接受益，预期 +几案 |
| margin-padding-clear（margin-collapse-101） | 中 | case a，FR-003，须守 R743/R744 |
| normal-flow / positioning | 低-中 | 匿名块盒高度回填可能影响含混合内容的布局 |
| welcome product-smoke | 低 | welcome 无混合 inline+block-in-inline，预期不变（须验证） |

### 8.4 详细设计（伪代码）

**高度回填（FR-001）**——两候选入口，首步验证后定：

```
# 候选 P1：放宽 compute_final gate 让匿名块盒存 inline_layout
for box in post_order(root):
    if box.is_r109_anonymous_block:        # 新标记（tree.rs 生成时设）
        ifc = build_ifc(box.fragment_node_ids, doc, styles)
        ifc.layout(...)
        box.inline_layout = ifc.lines       # 存（与普通块容器同）
        box.height = max(line.height)       # 回填测高
        mark_dirty(box)                      # 触发容器重测
# 风险：触发 R743/R744 回归（margin-collapse-101），须 A/B

# 候选 P2：engine.rs 后处理 pass（不动 gate）
fn backfill_anon_block_heights(root, doc, styles):
    for box in post_order(root):
        if box.is_r109_anonymous_block and box.height < epsilon:
            ifc = build_ifc(box.fragment_node_ids, doc, styles)
            ifc.layout(...)
            box.height = max(line.height)
    recompute_container_heights(root)        # 容器重算子盒高之和
# 复用 R695/R699 两趟基建；隔离度高（不动 compute_final gate）
```

**容器 bg 涂满（FR-002）**——依赖 FR-001 高度先对，paint_background 用回填后的 box.height（无须改 paint 逻辑，只要 box.height 正确，bg 自然涂满）。

**border 归属（FR-003）**——复用 r109.rs shrink_r109_anon_blocks（已实现），验证 case a 案 + A/B。

### 8.5 替代方案

| 方案 | 描述 | 优点 | 缺点 | 决定 |
|------|------|------|------|------|
| P1 放宽 gate | 匿名块盒走正常 compute_final 存 inline_layout + 回填高度 | 复用正常路径 | 触发 R743/R744 回归根因（gate 当初为此加） | ❌ 先不选（高风险） |
| P2 后处理 pass | engine.rs 专用 backfill，不动 gate | 隔离、复用两趟基建、可 env 开关 | 多一趟遍历 | ✅ 选定（Batch 1） |
| P3 全局关 R109 | R109_WIRE=0 | insert-* 几何略好 | net -5（R936 实测） | ❌ 拒绝 |

**最终选择**：P2（后处理 pass），首步先验证假设 A1（probe），再实现 backfill。

### 8.6 测试策略
- **单测**：layout-engine `test_anon_block_height_backfilled`（div>block+text，容器高含 inline）/ `test_anon_block_with_inline_block_child`。
- **Oracle A/B**：`make reftest-oracle DIR=css/CSS2/box-display`（insert-* 簇）+ margin-padding-clear（margin-collapse-101）+ inline-box-001（case a）。
- **product-smoke**：welcome + insert-block-in-inlines fixture PIL 断言（高度/bg 面积）。
- **回归守卫**：env 逐项开关回退；welcome diff ≤ 20%。

### 8.7 回滚计划
- P2 后处理 pass 用 env（如 `R109_BACKFILL=0`）包裹，回归时关闭回退到当前行为。
- Batch 分离（FR-001/002/003 独立提交），任一 net-negative 单独回退。

---

## 9. Spec Lint 报告

### 结构完整性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要存在性 | ✅ Pass | §0 |
| 场景存在性 | ✅ Pass | FR-001~004 各有 ≥1 场景 |
| 异常路径覆盖 | ⚠️ Warning | FR-002/003 异常场景偏少（露白/未拆分 inline 各 1），实施时补 |
| 测试绑定 | ✅ Pass | 每场景有验证命令/单测名 |
| TBD 清零 | ⚠️ Warning | 假设 A1/A2/A3 待首步验证（非阻塞，首步即验证） |
| 约束覆盖 | ✅ Pass | §6.1 各 Must 被 FR-001/002/004 场景覆盖 |
| 实施交接完备 | ✅ Pass | §7 文件清单/职责/顺序/批次齐 |
| 首步可执行性 | ✅ Pass | §7 推荐顺序 step 1（probe 验证 A1）明确 |

### 语言精确性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 模糊动词 | ✅ Pass | FR 用「必须回填/涂满/保持」具体动词 |
| 无量化描述 | ✅ Pass | NFR-001 给 ≤20%、FR-001 给 ≥224 / ≥60000 px |
| 非确定性措辞 | ✅ Pass | 无「应该/大概」（假设显式标「待验证」非措辞模糊） |

### 一致性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 范围冲突 | ✅ Pass | §1.3 排除项（font/IFC 重写/multicol）与 FR 无交集 |
| 约束冲突 | ✅ Pass | §6.1 Must 与 §6.2 Must Not 不矛盾 |
| 方案漂移 | ✅ Pass | P2 选定与 §6.3 复用决策一致 |
| 实现来源闭合 | ✅ Pass | §6.5 各能力来源（tree.rs/IFC/engine 后处理/painter/r109.rs）指明 |
| 代码边界完备 | ✅ Pass | §6.6 允许/禁止修改声明 |

**汇总**：Pass 12 / Warning 2 / Fail 0 / Skip 0
**门禁判定**：Fail = 0 → 允许进入实施（Warning 为假设待验证，首步 probe 即消解）。

---

## 10. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|----|------|--------|----------|--------|
| TBD-1 | ~~假设 A1（匿名块盒测高=0 + compute_final 跳过）~~ | ~~阻塞~~ | **【R938 已验证·部分修正】** compute_final 处理匿名块盒但**不回填 root.height**；taffy 测高经 ctx_node（首个文本节点）欠计多节点 run。FR-001 fix 位置 = compute_final IFC 后回填 root.height + 容器高度后处理 | ✅ 解除（进入 Batch 1） |
| TBD-2 | P1 vs P2 入口选择 | 重要 | A1 已验证：fix 在 compute_final 回填 root.height（P1.5，非 P1 放宽 gate 也非纯 P2 后处理），需配套容器高度后处理 pass | Batch 1 前定 |
| TBD-3 | welcome 是否含混合 inline+block-in-inline | 重要 | product-smoke A/B | Batch 1 验证 |

---

## 11. 修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v1.0 | 2026-07-02 | 初始版本（R937，收敛 R764 + R929–R936） |




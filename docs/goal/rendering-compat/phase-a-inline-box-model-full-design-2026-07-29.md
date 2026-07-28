# Phase A — inline-box-model coherence 完整可回退实施设计

**版本**: v1.0
**日期**: 2026-07-29
**状态**: Design-only（redirect 2026-07-28 裁决 #3 允许；禁止直接按旧 phase-a-slice1 开工）
**作者**: Rally R2157
**关联**: [`master.md`](./master.md) 顶部裁决 / [R2156 evidence](./evidence/r2156-inline-box-model-coherence-landed-2026-07-29.txt) / [R2152–R2155 scoping](./archive/) / [redirect cda1c6d23](../../rendering-compat.md)

---

## 0. 背景与裁决对齐

2026-07-28 用户 redirect 裁决：reftest ~57% 进入 plateau-guard；**允许** (1) 低风险 CSS2/parser/selector clean lever / (2) 产品·legacy smoke 可见稳定性修复 / **(3) Phase A 完整 inline-box-model / IFC coherence 的可回退实施设计**；**禁止**直接按旧 `phase-a-slice1` 开工实施。

本文档 = 裁决 (3) 的产出：Phase A 的**完整可回退实施设计**，含每切片 kill-switch + 结构签名 gate + 三态 A/B 门禁 + 净负回退策略。**design-only**：后续 session 须按本设计逐切片实施，每切片独立 A/B 守 net≥0 才 land。

R2156（commit `f322d2e46`，已 push）= 本设计 slice 1 的实施（inline-wrapping-atomic-no-ooflow），经三态 A/B net-positive 验证 land default-on。其与 redirect skip-list 的对齐问题已飞书通知用户裁决（keep/revert pending）；本设计独立于该裁决——无论 R2156 keep/revert，slice 2+ 设计成立。

---

## 1. 问题陈述（empirical grounding）

### 1.1 症状 1：37-form-controls 结构 FAIL（slice 1 已修）
`<p><label>Name: <input></label></p>` — label（inline）含嵌套 atomic inline（input）。ZW 把 label 建为块级 taffy 子 → 多 label 盒重叠 + 父 IFC 吸收其文本串联（R109 inline-ownership 分裂）。struct-check FAIL。**R2156 修复**（gate 跳过 inline-wrapping-atomic-no-ooflow 的 taffy 节点）。

### 1.2 症状 2：19-testpage-minimal 22% diff（slice 2 目标，未修）
legacy fixture `19-testpage-minimal.html`（HTML 3.2，用户可见老式静态页类型）chromium-oracle diff **22.39%**（远高于 ~5% font-wall 基线，为 legacy 套件最大非 OOS outlier；46-frameset 100% 为 frameset 未支持 OOS）。

**像素分析定位**（REFTEST bbox + per-row sampling）：
- diff bbox x[8,793] y[14,278]，集中 y[66-240]（两表区域）。
- y=170 采样：CPU=(192,192,192)（body silver，Product A 行无 bg），orc=(248,248,248)（Product B 行 bg）→ ZW 行比 chromium 落后 ~30px（行高累积差）。

**layout dump 根因**（19-testpage 第二表 Product A 行）：
```
td abs_y=143.9 h=55.0     ← 应 ~20px（单行），实 55px（3 行）
  a   abs_y=152.9 h=18.0  ← <a>linked</a>
  i   abs_y=171.9 h=19.0  ← <i>italic</i>（差 19px = 换行）
  b   abs_y=189.9 h=18.0  ← <b>bold</b>（差 18px = 换行）
```
"A linked product description with italic and bold text" 在 531px 列内应 1 行，ZW 把 a/i/b **各自块级堆叠成 3 行** → Product A 行 h=55（chromium ~20）+ Product B 行同理 → 表行高累积偏移 → 22% diff。

### 1.3 根因 probe（minimal case 确认触发条件）
| 容器 | 内联子 | a 的 abs_y | 行为 |
|------|--------|-----------|------|
| p / td / div | 单个 `<a>` + 文本 | 与容器同 y | ✓ 单行正确 |
| p | `<a>` + `<i>` + `<b>` + 文本 | a@16 / i@35 / b@53 | ✗ 块级堆叠 3 行 |
| td | `<a>` + `<i>` + `<b>` + 文本 | a@52 / i@71 / b@89 | ✗ 块级堆叠 3 行 |

**结论**：单个 inline Element 子正确（leaf-path / IFC 处理）；**≥2 个 inline Element 子**触发块级堆叠（每个 inline Element 被建为块级 taffy 子，垂直栈列）。**影响 p / div / td 全部 block 容器**（非 table-cell 特化）= 普遍性产品可见 bug。

> 注：welcome.html struct-check PASS（无 sibling-overlap / text-concatenation），但多 inline 容器的块级堆叠**不触发** struct-check（它是「行内→块级栈列」非「重叠/串联」），故 welcome 16.84% diff 可能部分含此因（潜伏未暴露）。

---

## 2. 机制（R2152–R2155 定稿 + 本轮 probe 补全）

### 2.1 build_subtree 子循环（`tree.rs` 非 flex/grid 路径）
block 容器的 element-child loop（`tree.rs:1284-1326`）为**每个 Element 子**调 `build_subtree` 建 taffy 节点作块级子。inline Element 子（a/i/b/span）由此成块级 taffy 子 → taffy 按块级垂直栈列。

leaf-path（`tree.rs:1271-1283`）条件：`has_text_child && has_element_child && all_inline && (is_flex_grid_item || InlineBlock || float)`。plain block（p/div/td）不满足最后括号 → 不入 leaf → 走 element-child loop → inline 子被块级化。单 inline 子「看起来正确」是因 IFC + painted_inline_nodes 抑制了双绘，但多 inline 子时块级 taffy 几何主导，栈列可见。

### 2.2 IFC 双路径
- layout IFC（`measure_text_content` + `compute_final_inline_layouts`）：`collect_inline_items` 经 R1576 递归收集 inline 子树文本/atomic inline。
- paint IFC（Path B）：可能用空 styles 重跑（R72 / R890 已记），与 layout IFC 分歧。

inline-box-model coherence 目标 = **inline 子树内容由父 IFC 单次排版定位**，不被同时建为块级 taffy 子（消除双路径分裂）。

### 2.3 ib_sizes（`postprocess.rs:112`）
容器 IFC 的 atomic inline 尺寸预算 map，由容器 **DIRECT** LayoutBox 子构建。后代 atomic inline（非直接子）须经 R1576 fallback 取尺寸——这是 R2155 crux：跳过 inline taffy 节点会改后代 atomic inline 尺寸可用性。

---

## 3. 切片清单（每片独立 land，守 net≥0）

> **通用 safeguard 模板**（每切片必含）：
> 1. **kill-switch**：`ZW_PHASEA_<SLICE>` env，default-off 起步（probe），A/B net≥0 后翻 default-on；`=0` 永久 kill。
> 2. **结构签名 gate**：精确触发条件（display + 后代结构 + writing-mode + ooflow），避免误触。
> 3. **三态 A/B 门禁**：self-source reftest 全目录零 delta + chromium-oracle 零漂移 + 产品 smoke（welcome 字节一致 + legacy struct 不退）+ make test/clippy/fmt green。
> 4. **净负回退**：任一闸门 net 负 → 立即回退该切片，记 evidence。
> 5. **driving test**：至少一个结构化断言（单测或 struct-check fixture）锁行为。

### Slice 1 — inline-wrapping-atomic-no-ooflow ✅ LANDED（R2156）
- **gate**：子 `display:inline` + 非自身 ooflow + 含嵌套 atomic inline 后代（`inline_elem_has_nested_inline_block`）+ 子树无 abspos/fixed 后代（`inline_subtree_has_ooflow_descendant` 守卫）+ horizontal-tb → 跳过该 inline taffy 节点。
- **kill-switch**：`ZW_INLINE_BOX_MODEL_COHERENCE`（default-on）。
- **A/B**：10 目录 self-source 零 delta + css-position oracle 66=66 + 37-form-controls struct FAIL→PASS / diff 4.33%→3.85% + welcome 字节一致 + make test green。
- **driving test**：6 个 `r2156_*` 单测 + 37-form-controls 产品 smoke。
- **状态**：landed `f322d2e46`，pending 用户 keep/revert 裁决（redirect 对齐）。

### Slice 2 — 多 inline Element 子 block 容器（**R2158 修正：leaf-path 路径 R1492-REFUTED，须改为 IFC→LayoutBox 定位**）

> **⚠️ R2158（2026-07-29）关键修正**：本节原设计「leaf-path（容器整体走 leaf，inline 子不建 taffy 节点）」= **R1492 已 REFUTED 的路径**。R1492（`ZW_PLAIN_INLINE_LEAF=1`）实测 borders oracle 411→401（**-10**），根因 = plain inline 元素（带 bg/border）须保留独立 LayoutBox，回流父 IFC 会丢 LayoutBox → bg/border 丢绘（tree.rs:1266-1270 注释 + R1480 evidence §2「元素仍 block 堆叠（仅 width 维度，完整 inline-box 模型属 R109 多 session）」）。R2156 slice 1 仅因 label 通常无 bg/border 而 orphan 良性——**不可外推到一般 plain inline**。
>
> **R2156 borders 安全已实测复核**（R2158）：`reftest-oracle CSS2/borders` gate ON=OFF=415（82.0%），R2156 在 borders dir 零漂移（inline-wrapper-with-bg-border-wrapping-atomic 模式在 borders corpus 罕见/缺失）。R2156 自身安全，但本节 slice 2 的 leaf-path 路径对一般多 inline（含 a/i/b 带 border）**必触发 R1492 -10 机制**。
>
> **修正后的正确路径 = 深层 Phase A：IFC→LayoutBox 定位**（R1492 建议的「保 inline 子为独立 box，修正容器高 + 移后续兄弟」）。即：inline 元素**保留**独立 LayoutBox（bg/border/hit-test 不破，R1492-safe），但其**位置由父 IFC 行盒决定**（非 taffy 块级栈列）。难点 = IFC 当前把 inline 元素作 text run 扁平收集（无 per-element 定位框），须扩 IFC 输出 per-inline-element 行内位置 + post-process 回填 LayoutBox。属多 session 深度架构，**非单切片**。

- ~~**gate（原 leaf-path，R1492-REFUTED）**~~：block 容器 + ≥2 inline Element 子 → leaf/IFC。**勿实施**（R1492 -10）。
- **gate（修正后，深层 Phase A，多 session）**：保留 inline Element 子的 taffy 节点（LayoutBox 不丢），新增 post-process 从父 IFC 行盒回填各 inline Element LayoutBox 的 (x,y,w,h) + 容器高度修正为真实行高（非块级栈列高）。须先扩 IFC 输出 per-inline-element 行内位置（当前无）。
- **kill-switch**：`ZW_PHASEA_IFC_LAYOUTBOX_BACKFILL`（probe 期 default-off）。
- **driving test**：`<p>A<a>x</a><i>y</i><b>z</b>.</p>` 断言 a/i/b LayoutBox 同行（y 一致，x 递增），bg/border 仍绘（R1492 守），容器高≈单行非 3 行。
- **三态 A/B**：self-source 全目录零 delta + **chromium-oracle CSS2/borders 零漂移（R1492 守，关键）** + 19-testpage diff 22%↓ + welcome 字节一致 + make test green。
- **前置工作**：扩 `InlineFormattingContext` 输出 per-inline-element 行内位置（fragment → element NodeId 映射）。此为深层 Phase A 的 enabling infra，独立可 land（dormant，零行为变更），后续再接 post-process 回填。
- **回退**：net 负 → `ZW_PHASEA_IFC_LAYOUTBOX_BACKFILL=0`。


- **kill-switch**：`ZW_PHASEA_MULTI_INLINE`（probe 期 default-off，A/B net≥0 后 default-on）。
- **driving test（须先建）**：
  - 单测：`<p>A <a>x</a> <i>y</i> <b>z</b>.</p>` 断言 a/i/b 同一行（baseline_y 一致），非块级栈列。
  - struct-check：扩展 `check_text_concatenation` / 新增 `check_inline_element_block_stacking`，守 19-testpage-minimal Product A 行 h≈单行非 3 行。
  - 产品 smoke：19-testpage-minimal oracle diff 22.39% → 目标 < 12%（行高修正后），struct PASS。
- **三态 A/B**：self-source 全目录（重点 CSS2/css-text/css-tables/normal-flow）零 delta + chromium-oracle（CSS2/css-tables）零漂移 + welcome/morning 字节一致或改善 + legacy struct 0→0（不退）+ make test green。
- **风险**：blast-radius 大（所有多 inline block 容器）。须增量验证：
  1. inline 元素有 bg/border/padding（如 `<a style="background:...">`）→ 丢 LayoutBox 则 bg/border 丢绘。R2156 实测「painter 经 IFC fragment 渲染 atomic inline 含 bg/border」成立，但 **plain inline（a/i/b）的 bg/border 是否经 IFC fragment 绘**须 probe 确认（slice 2 前置实验）。
  2. inline 元素是 hit-test / 事件目标（如 `<a href>`）→ 丢 LayoutBox 可致点击区域失真。须 product-smoke `--check-img-visibility` / 手测验证。
  3. inline 元素含 background-img / 形成 stacking context（position/opacity/transform）→ 须排除（gate 加 stacking-context 守卫）。
- **回退**：net 负 → `ZW_PHASEA_MULTI_INLINE=0`，记 evidence，slice 2 拆更小子切片。

### Slice 3+ — 后续（本设计列出，不展开）
- inline-wrapping-inline（`<span><span>text</span></span>` 嵌套纯 inline）。
- inline 元素带 bg/border 须保留 LayoutBox 的场景（若 slice 2 probe 暴露）。
- vertical writing-mode（R109-blocked，暂排除）。
- IFC fragment → LayoutBox 回填（R2155 step-2，补全 hit-test/事件路径，根治 orphan-LayoutBox 隐患）。

---

## 4. 实施顺序与验证矩阵（R2158 修正后）

> 原矩阵 S2-probe-a/b 基于 leaf-path，R1492-REFUTED（见 §3 slice 2 修正块）。新矩阵基于 IFC→LayoutBox 回填路径。

| 阶段 | 动作 | 验证 | 回退条件 |
|------|------|------|----------|
| S2-infra | 扩 `InlineFormattingContext` 输出 per-inline-element 行内位置（fragment→element NodeId 映射），dormant（零行为变更），default-off gate | make test green + self-source 零 delta + 单测断言映射正确 | 编译/测试不过→修 |
| S2-backfill-probe | post-process 从父 IFC 行盒回填 inline Element LayoutBox (x,y,w,h) + 容器高修正，default-off；probe `<p>A<a>x</a><i>y</i><b>z</b>.</p>` | layout dump a/i/b 同行（y 一致 x 递增）+ **bg/border 仍绘**（R1492 守）+ 容器高≈单行 | bg/border 丢→回填未保 LayoutBox，重设计 |
| S2-A/B | 翻 default-on，全量三态 A/B | self-source 零 delta + **chromium-oracle CSS2/borders 零漂移（R1492 守）** + 19-testpage diff 22%↓ + welcome 字节一致 + make test green | 任一 net 负→回 default-off |
| S2-land | 提交 + 推送 | pre-commit-guard + fmt/clippy/test | — |

**R2156 borders 安全复核（R2158，已完成）**：`reftest-oracle CSS2/borders` gate ON=OFF=415（82.0%）零漂移，R2156 slice 1 在 borders dir 安全（无须硬化）。



---

## 5. 为何 design-first（redirect 合规）

redirect 明确：Phase A 须先写可回退实施设计（含 kill-switch / 结构签名 gate / 三态 A/B / 净负回退），禁止直接开工。本设计满足全部四要素，且：
- 每切片可独立 A/B、独立回退（不依赖 big-bang）。
- 以 empirical evidence（19-testpage 22% / minimal probe / R2156 A/B）驱动 gate 精确化，非纸面推测。
- 承认 blast-radius，列出前置 probe（S2-probe-a/b）降风险。

后续 session 实施 slice 2 时，须严格按 §4 矩阵：先 probe 确认机制，再 default-off A/B，net≥0 才翻 default-on land。

---

## 6. 开放问题（待 probe / 用户裁决）

1. R2156 keep/revert（用户裁决，飞书已通知）——影响 slice 1 是否留作 slice 2 基础，但 slice 2 设计独立。
2. plain inline 的 bg/border 是否经 IFC fragment 绘（S2-probe-b）——决定 slice 2 是否须先做 step-2 LayoutBox 回填。
3. inline `<a>` hit-test 在丢 LayoutBox 后是否仍工作——product-smoke / 手测验证。
4. vertical writing-mode（R109-blocked）何时并入——暂排除，待 vertical block-flow 解锁。

---

## 7. 参考
- R2156 evidence: [`evidence/r2156-inline-box-model-coherence-landed-2026-07-29.txt`](./evidence/r2156-inline-box-model-coherence-landed-2026-07-29.txt)
- R2152–R2155 scoping: master.md preamble + `archive/`
- redirect: `docs/goal/rendering-compat.md` 2026-07-28 裁决块 + commit `cda1c6d23`
- 既有 Phase A 设计（历史）: `phase-a-IFC-unification-design.md`（R69 时代，部分过时，本设计取代其 slice 拆分）

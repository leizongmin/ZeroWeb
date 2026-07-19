# RFC：R109 匿名块包裹 inline-level 元素 + IFC float 排斥（l 簇 floats-wrap-top-below-bfc 解锁）

**版本**：v1.0
**日期**：2026-07-19
**状态**：设计 + root-cause 实证（R1731 instrument）；0 code land；多会话实现待续
**起源**：R1730 forward ① + R1731 根因定位。floats-wrap-top-below-bfc l 变体（001l/002l/003r）self-source fail，ZW 渲 REF 页 inline-block 旁 float 错（x=11 应 x=161）。

---

## 0. 执行摘要

- **一句话目标**：block 容器内 **inline-level 元素**（inline-block / inline / inline-flex 等）与 block 同胞混排时，按 CSS2 §9.2.1.1 用匿名块盒包裹 inline 内容，使其经 IFC 定位并应用 float 排斥（line-box shortening）。当前 ZW 仅对 **文本节点** 做匿名块包裹，inline-level **元素** 直接作 taffy 子节点 → block 式定位（x=content_left），绕过 IFC float 排斥。
- **本期范围**：仅 RFC（root-cause + fix 设计 + 验收 + 回归风险），不下结论先改代码。
- **目标 yield**：floats-wrap-top-below-bfc l 变体 self-source flip（001l/002l/003r/003l，4 案），更高 yield 因 TEST 侧已 spec-correct（R1725-R1730）。
- **明确排除**：完整 Phase-A IFC 统一（更深架构）；本 RFC 只解「lone inline-level element between blocks」的匿名块包裹 + float 排斥子症状。
- **核心约束**：① kill-switch + default-on + 全量 A/B net≥0（floats + floats-clear + inline-formatting-context + CSS2 全簇）；② R109 split 机器改动须守既有 split-inline（inline 元素含 block 子被拆分）行为零回归；③ root-cause-first。

---

## 1. 背景

floats-wrap-top-below-bfc 子簇 6 变体（R1726 triage）：r 变体（001r/002r/003l）TEST 侧已 spec-correct（R1728 左 fit-pushbelow + R1730 多-float 协调 + margin_auto plumbing）。**l 变体**（001l/002l/003r）self-source fail 真因 = ZW 渲 **REF 页**错：REF 用 `display:inline-block; vertical-align:top` span 旁 float，chromium 把 span 放 float 右侧（行盒被 float 缩短 x=161），ZW 放 x=11。002l ZW-TEST vs chromium=1.00%（TEST 对），ZW-REF 错 → self-source 8.12% fail。

## 2. root-cause（R1731 instrument 实证）

`tests/...` instrument（`compute_final_inline_layouts` 入口 ZW_R1731_PROBE）：002l REF 只 log 到 body/html，**无匿名块节点**，证明 lone inline-block span **不进** IFC 路径。

**两层根因**（`crates/layout-engine/src/inline_block_split.rs`）：

### 2.1 gate 只认文本不认 inline-level 元素

`block_container_has_mixed_content`（:77-132）判 block 容器是否需匿名块包裹：

```rust
match &node.kind {
    NodeKind::Text(text) if !text.content.trim().is_empty() => { has_text = true; }
    NodeKind::Element(_) => {
        if is_block_level_display(&style.display) { has_block = true; }
        // inline-level elements already have their own taffy nodes;
        // only text nodes need anonymous block wrapping.   ← ★ 缺口
    }
}
if has_text && has_block { return true; }   // 仅 text+block 触发
```

→ 002l REF body=[float, inline-block-span, spacer-div, inline-block-span]：`has_text=false`（span 是元素非文本）/ `has_block=true` → gate **false** → **不触发匿名块包裹**。inline-block span 直接作 body 的 taffy 子节点。

### 2.2 split 把 inline 片段作匿名块 leaf（非 child）

即使 gate 触发，`compute_block_container_split`（tree.rs:1133-1142）把 inline 片段作匿名块 **leaf**（`new_leaf_with_context`，measure context=首文本节点），atomic inline-box 不作匿名块的 **子** → 匿名块 IFC 不会定位它。

### 2.3 后果

inline-block span 作 body 直接 taffy 子节点（inline-level，`is_block_level=false`），taffy 0.12 在 block 容器内对 inline-level 子作 block 式定位（独占行，x=content_left=11）→ **不经 IFC** → `effective_content_area`（inline/mod.rs:549 按 `float_exclusions` 算 left_offset）不生效 → 不被 float 缩短 → x=11（应 x=161）。

## 3. 推荐方案：分轨 scoped slice

每 slice kill-switch + default-on + 全量 A/B net≥0。

### Slice 1（中风险）：gate 扩认 inline-level 元素

- **改**：`block_container_has_mixed_content` 加 inline-level 元素检测——`has_inline_element = true` 当子为 inline-level display（Inline/InlineBlock/InlineFlex/InlineGrid/InlineTable 等，非 out-of-flow）；触发条件改 `has_block && (has_text || has_inline_element)`。需新增 `is_inline_level_display` helper（mirror `is_block_level_display`）。
- **效果**：002l REF body 触发匿名块包裹（inline-block span 进 Inline 片段）。
- **scope gate**：仅 `display` 为 inline-level 且非 out-of-flow 的元素子 + 同容器有 block 子。
- **风险**：中——改变**所有**「block 容器含 inline-block + block 子」的匿名块创建时机，须守全量 A/B（inline-formatting-context / anonymous-boxes / floats 全簇）。可能暴露既有依赖「inline-block 直接 taffy 子」的 case。
- **前置**：Slice 1 单独**不够**——gate 触发后 split 仍把 inline-block 片段作 leaf（2.2），须 Slice 2 配合。

### Slice 2（高风险）：atomic inline-box 片段作匿名块 **child**

- **改**：`compute_block_container_split` / tree.rs:1121-1165 对含 **atomic inline-level element**（inline-block / replaced inline / inline-flex 等，非纯文本）的 Inline 片段，把该元素作匿名块的 **taffy 子**（非 leaf），使匿名块 IFC 定位它（应用 float_exclusions）。
  - 纯文本片段维持 leaf（measure context=文本节点，零回归）。
  - atomic 片段：匿名块 `display:Block`，其 taffy 子 = inline-block 节点；匿名块经 `compute_final_inline_layouts` 走 IFC，`float_exclusions`（own_floats + ancestor_floats，inline_finalization.rs:826）应用 → line-box 缩短 → inline-block 放 float 右侧。
- **scope gate**：仅 Inline 片段含 atomic inline-level element 时走 child 路径；纯文本片段不变。
- **风险**：高——R109 split 机器核心改动，须守 split-inline（inline 元素含 block 子被拆分，anonymous-boxes-* 簇）+ inline-formatting-context 全簇 + floats-clear 零回归。memory 多轮 R109 entanglement 警告（R1047 net-negative / R109 blast radius）。
- **前置依赖**：Slice 1（gate）+ Slice 2（child）**必须同 land**，单独任一无 yield（gate 触发但 leaf 不定位 / child 但 gate 不触发）。

### Slice 3（可选，低风险）：vertical-align / 多 inline 元素扩展

- Slice 1+2 解 lone inline-block 后，扩展多 inline 元素片段 + vertical-align:top（002l REF 用）对齐。逐案 A/B。

## 4. 验收标准

- **目标 flip**：floats-wrap-top-below-bfc 001l/002l/003r/003l self-source PASS（或 <1%）；ZW-TEST vs chromium 保持 <1%（不退步）。
- **A/B net≥0**：`make reftest`（reftest-upstream）floats + floats-clear + css/CSS2/visuren(inline-formatting-context) + anonymous-boxes + margin-collapse 全簇；product-smoke welcome <20%。
- kill-switch `ZW_R109_INLINE_ELEMENT_ANON`（default-on）；load-bearing 单测（parse_html + LayoutEngine::compute：block 容器含 inline-block + block 子 + float → inline-block x = float 右缘，非 content_left）。
- 门禁：fmt / clippy -D warnings / make test 全绿。

## 5. 回归风险（R109 / R1047 教训）

- **split-inline 零回归**：现有 R109 处理「inline 元素含 block 子被拆分为匿名块序列」（tree.rs:1104 `is_inline_r109`）——Slice 1/2 不得影响此路径（gate `is_block_mixed` 与 `is_inline_r109` 分离，已隔离）。
- **anonymous-boxes 簇**：匿名块包裹时机变化可能影响 anonymous-boxes-001b 等（margin-collapse-through 落父顶重叠，tree.rs:1260 注释）。
- **inline-formatting-context 簇**：inline-block 直接 taffy 子→匿名块子，改变 inline-block 的盒尺寸/定位基准，可能回归 inline-block sizing/shrink-to-fit（R129/R138/R1017 谱系）。
- **taffy native**：taffy 0.12 对匿名块 + inline 子的布局可能与 ZW 后处理假设分歧。
- **全树重跑禁令**：限结构签名 gate（仅 block 容器含 inline-level element + block 子时触发），早返回无该结构的 case。

## 6. forward / 续跑入口

- **R1731**：root-cause instrument landed（探针 revert），本 RFC v1.0。
- **下轮**：Slice 1+2 合并试 land（kill-switch default-on），全量 A/B。net≥0 + 4 案 flip 则 land；net<0 则精确化 gate（如仅 `inline-block` 不含 inline-flex/grid）或 revert 转 RFC v1.1。
- 备选：若 R109 风险过高，转 bfc-relocate RFC Slice 3（百分比宽 floats-wrap-bfc-005）/ 4-B shrink-retry / 4-E right-table（BFC 谱系，无 R109 风险）。

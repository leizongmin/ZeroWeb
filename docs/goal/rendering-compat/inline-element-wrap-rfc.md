# Spec：R109 inline-element 匿名块包裹（display:inline 元素作 block 容器直挂子）

**版本**：v0.1（R1735，2026-07-19，首版草案 — 实证根因 + naive-fix 证伪 + 多 session 切片计划）
**日期**：2026-07-19
**作者**：AI Assistant（rendering-compat rally）
**状态**：草稿（rally 续跑用，无用户确认门禁；directive #4 已授权 R109 多 session；实施按切片逐 session A/B 门禁推进）
**复杂度**：高（涉 R109 匿名块 + IFC owner-context + paint border-溢出，跨 layout/paint；高回滚难度；welcome 等真页回归风险）

---

## 0. 执行摘要

- **一句话目标**：让 ZeroWeb 对 `display:inline` 元素作为 block 容器（如 `<body>`/`<div>`）**直挂子**（与 block 兄弟混排）时，按 CSS2 §9.2.1.1 匿名块 + IFC 正确渲染——inline 元素参与 IFC line box，其 border/padding **溢出 line box**（向上/下覆盖邻接区），而非被当 block-flow box 落在流位。
- **driving case**：[`css/CSS2/borders/border-width-applies-to-008.xht`](../../tests/wpt-runner/wpt-data/css/CSS2/borders/border-width-applies-to-008.xht)（oracle diff 15.36%）——`<div style="display:inline;border-width:90px;font:90px/1 Ahem">&nbsp;</div>` 作 `<body>` 直挂子（与 block `<p>` 兄弟）。ZW border-box y=[188,458] vs chromium [96,366]，低 92px ≈ border-top-width。
- **本期范围**：本 RFC 不立即全落地；定义**多 session 切片计划**（Slice 1 … 4），每切片独立 env-gated + 全量 A/B 门禁（net≥0 即留，net-负即回退记 entangled）。
- **明确排除**：inline 元素**含 block 子元素**的拆分（已有 `compute_inline_block_split`，本 RFC 不动）；inline-block / replaced inline（已有路径）；vertical-mode inline（与 [[vertical-mode-ifc-unification-rfc]] 耦合，待 vertical IFC）；inline 文本节点的匿名块包裹（已工作）。本 RFC 仅覆盖**horizontal-tb 下，inline 元素作 block 容器直挂子（无 inline 兄弟文本节点）的匿名块包裹 + border 溢出**。
- **核心约束**：① horizontal-tb 零回归（gate 到 inline 元素 + block 兄弟混排结构签名）；② 每切片三态门禁：welcome product-smoke <20% + scoped oracle（borders + visuren/inline-formatting-context + CSS2）零回归 + self-source 不降；③ WPT corpus yield 预期**低**（border-width-008 + 少数 inline applies-to 同谱案 ~2-5 案；inline 元素作 block 直挂子在真实网页不常见，多数 inline 在 IFC 文本流内已被处理）——spec-correctness 提升为主，reftest headline yield 次要。
- **★ 风险定级**：高。R109 历史多次 attempt net-negative/multi-session（plateau 文档反复定性）；inline 渲染核心改动影响所有 inline border/padding，welcome 等真页易回归。每切片**必须** env-gated default-off + 全量 A/B + 净负即回退。

---

## 1. 背景：当前 broken 状态（R1735 实证）

`<div style="display:inline;border-width:90px;font:90px/1 Ahem">&nbsp;</div>` 作 `<body>` 直挂子（body = `[<p>(block), <div>(inline)]`）渲染**几何错位**。

### 1.1 LAYOUT_DUMP + REFTEST_DEBUG 实证（R1735）

```
html     abs_y=0    h=376
body     abs_y=16   h=352  (mt=16)
  p      abs_y=16   h=37   (margin-bottom:135)
  div    abs_y=188  h=270  (mt=0, x=8, w=270)   ← inline 元素，LayoutBox 直挂 body（无匿名块）
```

ZW primitives（4 fills = 4 段 border 条，形状正确但整体下移）：
```
fill[0] (8,   188, 270, 90)  TOP    y=[188,278]
fill[1] (188, 278,  90, 90)  RIGHT  x=[188,278] y=[278,368]
fill[2] (8,   368, 270, 90)  BOTTOM y=[368,458]
fill[3] (8,   278,  90, 90)  LEFT   x=[8,98]   y=[278,368]
→ border-box = x[8,278] y[188,458]；中心白方 x[98,188] y[278,368]
```

chromium oracle：border-box y=[96,366]（同 270 高，90px 黑边 × 4，中心 90×90 白）。

**差**：ZW top=188 vs chromium top=96 → ZW 整体低 92px ≈ border-top-width(90)。border 条**形状本身正确**（270×270，90px 条），纯 Y 定位差。

### 1.2 根因：block vs inline box 模型

ZW 把 `display:inline` 的 `<div>` 当 **block-flow box** 渲染：
- div 作为 body 的直挂 taffy 子节点（block flow item），border-box top = 流位 = p_bottom(53) + margin(135) = 188。
- 内容区 = 188 + border_top(90) = 278（glyph 在 [278,368]）。
- border-box = [188, 458]。

chromium（正确 inline 模型）：
- div 参与 IFC line box（line-height 90px），line box 在流位 [188,278]。
- border-top **向上溢出** line box 90px → border-box top = 188 − 90 = 98（≈ oracle 96，2px baseline/leading 差）。
- border-bottom 向下溢出 → border-box = [98, 368]。
- glyph（line 内容）在 [188,278]。

→ 差 90px = border-top-width。**inline border 应溢出 line box**；ZW 无 line box（block 模型）故 border 落流位。

### 1.3 为何不触发 R109 匿名块包裹

R109 wrapping 已 default-on（[`tree.rs:94 r109_wired()`](../../crates/layout-engine/src/tree.rs)），但 [`block_container_has_mixed_content`](../../crates/layout-engine/src/inline_block_split.rs)（:77）**仅把 text 节点算作 inline 内容**：

```rust
// inline_block_split.rs:122-123
// inline-level elements already have their own taffy nodes;
// only text nodes need anonymous block wrapping.
```

body = `[p(block), div(inline)]` → `has_text=false`（div 是 Element 非 Text），`has_block=true` → mixed = **false** → 不拆分 → div 直挂 taffy 节点按 block-flow 渲染。

---

## 2. naive-fix 证伪（R1735 实验，已 revert）

**假设**：把 inline **元素**子也算 inline 内容（与 text 节点并列）→ 触发匿名块拆分 → div 进 IFC → border 溢出。

**实施**（env `ZW_R109_INLINE_WRAP=1`，default-off）：`block_container_has_mixed_content` 加 `else if wrap_inline_elements { has_text = true; }`。

**结果（LAYOUT_DUMP）**：
```
body
  p      abs_y=16  h=37
  body   abs_y=188 h=110 w=16      ← 匿名块！但 w=16（错），h=110
diff = 15.32%（vs baseline 15.36%，marginal，未 flip）
```

**两层缺口**：

1. **anonymous block ctx_node fallback**：匿名块的 `ctx_node` 取片段首个 **Text** 节点（[`tree.rs:1131`](../../crates/layout-engine/src/tree.rs)）：
   ```rust
   let ctx_node = item_node_ids.iter().copied()
       .find(|&nid| doc.get(nid).is_some_and(|n| matches!(n.kind, NodeKind::Text(_))))
       .unwrap_or(dom_id);   // ← div 片段无直挂 Text（&nbsp; 是 div 子）→ fallback 到 body
   ```
   div 片段无直挂 Text（`&nbsp;` 是 div 的子，非 body 的子）→ fallback 到 `dom_id(body)` → 匿名块用 **body 的** measure context（w=16 错误）→ div 的 border/sizing 全丢。

2. **IFC border 溢出 paint 单行 gate**：[`painter/text.rs:1207-1282`](../../crates/engine/src/paint/painter/text.rs) 的 inline border 溢出 paint（border-top at `line_top - bt_w`）gate `owner_h > $frag_fs * 1.5`（**仅多行**，:1213 注释「避 R638 双计」）。div 单行 → owner_h ≈ line-height 90，frag_fs 90 → `90 > 135` false → gate 不触达 → 即使进 IFC，border 也不溢出绘制。

→ 仅放宽 gate 不足；machinery 为 text-node 片段设计，element 片段需更深改造。代码已 revert（R1735 净 0）。

---

## 3. 真修复：4-部件 R109 inline-element-wrapping

须同改 4 处（任一缺失即不工作）。每部件独立切片，env-gated default-off，全量 A/B 守 net≥0。

### Slice 1：gate 放宽 + anonymous block 取 inline 元素 ctx/盒模型

**改动**：
- (a) [`inline_block_split.rs block_container_has_mixed_content`](../../crates/layout-engine/src/inline_block_split.rs)：加 env `ZW_R109_INLINE_WRAP` 分支，inline 元素子（in-flow，非 out-of-flow）算 inline 内容（`has_text = true`）。
- (b) [`tree.rs:1128-1161`](../../crates/layout-engine/src/tree.rs) Inline 片段处理：对 **element-only 片段**（无 Text 直挂子），`ctx_node` 取片段内的 inline **元素** 节点（非 fallback 到 container）；匿名块 taffy style 用 `computed_style_to_taffy(inline_element_style, ...)` 继承该 inline 元素的盒模型（border/padding/background）—— 现仅 `is_inline_r109`（inline-split-parent）分支继承（:1148），`is_block_mixed` 分支用 plain Block（:1136）须扩。

**门禁**：env `ZW_R109_INLINE_WRAP=1` default-off；A/B borders + visuren + CSS2 + welcome product-smoke net≥0。
**预期**：div 进 IFC，匿名块用 div 盒模型（border 90px），w/h 正确。但单行 border 溢出 paint 仍不触达 → border-width-008 **此切片后仍不 flip**（需 Slice 2），但匿名块几何正确（w=270 非 16）= enabling infra。
**风险**：[block + inline 元素] 容器全量变化（broad）；inline 元素盒模型继承可能 double-count（容器盒 + 匿名块各绘 border/bg）→ 须 gate 容器盒绘制跳过被包裹 inline 子（或在 extract_layout 标记）。

### Slice 2：单行 IFC inline border 溢出 paint

**改动**：[`painter/text.rs:1207`](../../crates/engine/src/paint/painter/text.rs) gate `owner_h > $frag_fs * 1.5` 放宽到单行（移除多行 gate 或降阈值到 `owner_h >= frag_fs`），同时避 R638 双计（:1213 注释——R638 是 bg/border 双绘；须核对 R638 谱系，可能须加 `is_first_line`/`has_bleed` 精化 gate 而非裸放宽）。

**门禁**：env `ZW_R109_INLINE_BORDER_BLEED=1` default-off；A/B borders + visuren + CSS2 + welcome net≥0；**重点核查 R638 双计回归**（bg-only 多行 inline、border-only 003 driving test）。
**预期**：div（Slice 1 后在 IFC）border 溢出 line box 绘制 → border-box [98,368] ≈ chromium [96,366] → **border-width-008 FLIP**。
**风险**：影响**所有** single-line inline 元素的 border/padding 绘制（broad）；R638 双计回归是已知陷阱。

### Slice 3：inline 元素子节点（文本/嵌套 inline）IFC 收集 + fragment owner 归属

**改动**：div 的 `&nbsp;` 文本子节点经匿名块 IFC 收集时，fragment `owner_id` 须 = **div**（inline 元素），非 body/匿名块——否则 Slice 2 的 `owner_style.display == Inline` 判定（:1205）+ border 溢出 paint（读 owner_style.border_*）失败。核对 [`collect_inline_items`](../../crates/layout-engine/src/inline/mod.rs) 对匿名块包裹的 inline 元素的 owner 归属。

**门禁**：随 Slice 2 同 A/B（owner 归属错则 border 不绘，border-width-008 不 flip 即可发现）。
**风险**：owner 归属影响 ruby overlay（R1022 谱系）、text-decoration 传播等多路径，须核查不破。

### Slice 4：横向清理 + default-on

Slice 1-3 全 flip 且全量 A/B net≥0 后：移除 env gate（default-on）+ 补 load-bearing 单测 + product-smoke 长期守。

---

## 4. 验收场景

每切片三态门禁（全绿才留）：

| 场景 | 标准 |
|------|------|
| **border-width-applies-to-008** | Slice 1+2+3 后 oracle diff 15.36% → <1%（FLIP）；Slice 1 单独不 flip（enabling infra） |
| **scoped oracle A/B** | borders + visuren（inline-formatting-context）+ CSS2 全量 oracle net≥0（self-source 不降 + strict 真通过不降） |
| **welcome product-smoke** | diff <20%（DC-13 gate，字节一致或 net≤0） |
| **R638 双计核查** | bg-only 多行 inline（border-padding-bleed driving test）+ border-only 003 字节一致 |
| **inline 文本节点匿名块** | 现有 R109 文本包裹（block-in-inline-* / inline-box-*）零回归 |
| **load-bearing 单测** | 每切片补 1 单测（Slice 1：element-only 片段 ctx_node=inline 元素；Slice 2：单行 inline border 溢出 paint） |

---

## 5. 风险与备选

- **高风险**：R109 历史 net-negative（plateau 文档）；inline 渲染核心影响 welcome 等真页。**每切片 env-gated + 净负即回退**，不裸上。
- **低 yield**：border-width-008 + 少数 inline applies-to 同谱 ~2-5 案；inline 元素作 block 直挂子在真实网页罕见。spec-correctness 为主。
- **备选**：若 Slice 1 A/B 即 net-negative（broad 回归不可 scope），裁决 R109 inline-element-wrap = hard plateau（同 font-wall/vertical 谱系），border-width-008 接受为 R109-blocked，转 font-wall C-dep（headline 主阻塞）或 safe latent-gap。
- **不建议**：勿以「div 当 block 也能用」忽略——border-width-008 是 CSS2 §9.2.1.1 合规性测案，spec-correctness 缺口；但 yield 低须据 ROI 决定优先级（font-wall 解锁 headline 更高 ROI）。

---

## 6. 与既有文档/记忆的关系

- R1735 evidence：[`evidence/r1735-border-width-008-inline-element-r109-blocked-2026-07-19.txt`](./evidence/r1735-border-width-008-inline-element-r109-blocked-2026-07-19.txt)（实证根因 + naive-fix 证伪）。
- R109 §9.2.1.1 既有实现：[`r109-anonymous-block-spec.md`](./r109-anonymous-block-spec.md)（text-node 包裹设计）；本 RFC 扩到 element 包裹。
- R109 FR-002（容器 bg 涂满匿名块盒）：已由 [`backfill_r109_anon_block_heights`](../../crates/layout-engine/src/postprocess.rs) 解决（R1596）；本 RFC 是 FR-003（inline 元素 border）+ 匿名块包裹扩展。
- font-wall：[[r1088-first-letter-phasea-universal-gate]] / [[r1560-skia-raster-fontwall-ruled-out]]（headline 主阻塞，C-dep user-gated，与本 RFC 独立）。

# Spec：Ruby 布局架构统一

**版本**：v0.1（R1683，2026-07-18，首版草案 — 实证根因 + 多 session 切片计划）
**日期**：2026-07-18
**作者**：AI Assistant（rendering-compat rally）
**状态**：草稿（rally 续跑用，无用户确认门禁；实施按切片逐 session A/B 门禁推进）
**复杂度**：中高（跨 layout/paint 模块 / ruby display type 新引入 / IFC owner-context 改动 / 高回滚难度）

---

## 0. 执行摘要

- **一句话目标**：让 ZeroWeb 对 `<ruby>`/`<rt>`/`<rp>` 做 CSS Ruby 模块 + HTML 渲染规范的正确渲染——base 文本作为 ruby base 正常排版，`<rt>` annotation 作小字浮于 base 上方（横排）或右侧（竖排），`<rp>` fallback 括号不渲染。
- **本期范围**：本 RFC 不立即落地；它定义**多 session 切片计划**（Slice 1 … 4），每切片独立 A/B 门禁（net-0/正即留，net-负即回退），后续 session 按序推进。
- **明确排除**：complex ruby（`<rtc>` 多 annotation / `<rb>` 显式 base / ruby-position 上下切换 / 自动 tabular annotation 对齐）；vertical-mode ruby（与 [[vertical-mode-ifc-unification-rfc]] 耦合，待 vertical IFC 解锁）；CJK 字体度量（font-wall，非 ruby 范围）。本 RFC 仅覆盖**横排 simple ruby**（`<ruby>base<rt>annot</rt></ruby>`，corpus 与 legacy 主要形态）。
- **核心约束**：① horizontal-tb 零回归（ruby 改动 gate 到 `local_name()=="ruby"` 子树）；② 每切片三态门禁：welcome product-smoke <20% + scoped oracle 零回归 + self-source 不降；③ WPT corpus ruby 极罕见（css/css-ruby 不在 driving reftest 集，yield = 0 预期，等同 R1670/R1679 form-control latent-gap 谱系——spec-correct + 产品/legacy 可见，非 reftest headline）。

---

## 1. 背景：当前 broken 状态（R1683 实证）

`<ruby>` 在 ZeroWeb 渲染**结构性错误**。以 legacy-html fixture 48 `<ruby>漢<rt>kan</rt><rp>(</rp><rt>字</rt><rp>)</rp></ruby>` 为例（chrome-127 oracle 对照）：

### 1.1 LAYOUT_DUMP 实证（R1683）

```
ruby    abs_y=94.0 h=37.2 x=8.0 w=42.4    <- ruby 容器盒（高 37.2 = 两行）
  rt    abs_y=94.0 h=18.0 w=26.4          <- "kan" 第一 rt
  (anon) abs_y=94.0 h=0.0                 <- base "漢" 文本（collapsed h=0）
  rt    abs_y=113.0 h=19.0 w=16.0         <- "字" 第二 rt（在第一 rt 下方！）
  (anon) abs_y=94.0 h=0.0
```

chromium 期望：annotation（"kan字" 小字）在 base "漢" 上方一行，base + 后续文本同行。

### 1.2 三层 root-cause

1. **ruby 被当 block-level container**：`ua_default_display` 对 ruby/rt 返回 None → CSS 初始值 `inline`，但 layout tree（tree.rs）仍为 ruby 生成独立容器盒，其 inline 子（base 文本 + rt）**未走 IFC**——按 block 流垂直堆叠（rt1@y94、rt2@y113），base 文本塌缩 h=0。inline 元素本应参与父 `<p>` 的 IFC，但 ruby 拿到容器盒后子内容被 block 化。
2. **R1022 paint overlay 失效**：现有 ruby annotation 机制（`painter/text.rs:164 ruby_annotation_chars` + 两处 paint loop `:1412`/`:1616`）依赖 base 文本 fragment 的 `owner_id == ruby` 才触发。但 `collect_inline_items`（[inline/mod.rs:1097](../../crates/layout-engine/src/inline/mod.rs)）对 ruby 用 `collect_text_excluding(&["rt","rp"])` 把 base 文本**扁平化收集进父 `<p>` 的 IFC** → base fragment owner_id = `<p>` → `ruby_annotation_chars(doc, owner_id=<p>)` 返回 None → overlay 永不 fire。即 R1022 的 owner 假设在 IFC 扁平化下不成立。
3. **rt 也参与父 IFC 渲染**：rt 非 display:none，其文本被当普通 inline 渲染（与 base 串联 / 堆叠），产生可见的错误 rt 盒。

### 1.3 R1683 probe 证伪（rt→display:none 不足）

R1683 试探 `rt|rtc → display:none`（≡ rp 谱系）：ruby h 37.2→18.6（垂直堆叠消失，内容上移 16px 对齐 chromium），**但** annotation 完全消失（R1022 overlay 因 owner 丢失不 fire）+ diff 仅 5.45→5.42 marginal。证伪「display:none 一行可解」——须同时修 base 容器盒几何 + annotation owner-context，纯 display 改动把 annotation 一起删掉。

---

## 2. 目标架构（CSS Ruby 简化）

横排 simple ruby 的 chromium 行为分解：

```
<ruby>漢<rt>kan</rt></ruby>  →  [annotation: "kan" 小字 fs×0.5, 居中于 base 上方]
                                 [base: "漢" 正常 fs]
```

- **ruby box**：inline-level 容器，参与父 IFC（不建立独立 block BFC）。其内容 = base 文本流 + annotation。
- **base**：ruby 的非 rt/rp 文本，正常字号、正常 inline 流。
- **rt annotation**：rt 文本，字号 fs×0.5（chromium UA `rt { font-size: 50%; line-height: normal; }` + vertical-align 上移），整体居中于对应 base segment 上方，独占 annotation 行（base 行上方）。
- **rp**：display:none（R1676 已 LANDED）。
- **ruby 行盒高度**：base 行高 + annotation 行高（annotation 行在上方）。

---

## 3. 实施切片计划

每切片独立 A/B（kill-switch `ZW_RUBY_*` default-off）+ 全量 css/CSS2 + writing-modes + css-ruby（若有）oracle 守 net≥0 + product-smoke welcome <20%。

### Slice 1 — ruby base 参与父 IFC（修 root-cause #1）

**目标**：让 ruby 不再生成把子内容 block 化的容器盒；base 文本正常参与父 IFC，rt/rp 不参与（rp 已 none）。

**seam**：tree.rs layout-tree 构建——对 ruby 元素，不生成独立 block-children 容器，而是让 base 文本流入父 IFC（≡ span 等纯 inline）。rt 子在布局期标记为「annotation，不参与 base IFC」（其文本不经 collect_inline_items 收集，仅由 paint overlay 消费）。

**风险**：layout-tree 容器盒生成是核心路径（影响所有 inline 元素），须精确 gate 到 ruby 子树 + 不破坏 picture/span 等 inline 容器（picture>img 单子无 IFC 问题）。参考 R1576 inline-box-recurse 的 structural-signature gate 模式。

**验收**：LAYOUT_DUMP ruby base 文本 fragment owner_id == ruby（非父 p）；base 文本在 ruby 行正常渲染；rt 不再生成堆叠盒。fixture 48 ruby 单行（h≈18.6）+ base "漢" 可见。

### Slice 2 — segment-based annotation 收集（修 root-cause #2）

**目标**：替换 R1022 的扁平化 char-pairing（`ruby_annotation_chars` 返回单 Vec<char> 按 base[char_idx] 配对——多 char base/annot 完全错配）为 **segment-based**：按 ruby 子节点序收集 `[(base_segment_text, annot_text)]` 对（每个 rt 配对其前的 base 文本段）。

**seam**：新 `ruby_annotation_segments(doc, ruby_id) -> Vec<(String, String)>` 替代 `ruby_annotation_chars`。paint loop 按 segment 居中 annotation 于对应 base segment 上方（非逐字符）。

**风险**：paint loop 在 base fragment 内按 segment 定位 base 子串 x 偏移——须 measure base segment 宽度。multi-fragment base（ruby base 跨行）rare，首版按单 fragment 处理 + 多 fragment 回退无 annotation。

**验收**：fixture 48 base "漢" 上方居中显示 "kan字"（或两 rt 各自 segment）；最小用例 `<ruby>base<rt>annot</rt></ruby>` 居中匹配。

### Slice 3 — rt UA 样式（font-size 50% + 不参与 base 流）

**目标**：rt 经 Slice 1/2 已不参与 base IFC，但须显式 UA `rt { font-size: 50% }`（annotation 字号）——目前 rt 字号 = 继承（全尺寸），annotation 应半尺寸。

**seam**：`ua_decl_inputs`（lib.rs）加 rt arm → `font-size: 50%`（≡ pre/h1/p UA 默认值谱系，specificity 0,0,0 可被作者覆盖）。

**验收**：annotation 字号 = base × 0.5；全量 A/B 守 net≥0（corpus ruby 稀少）。

### Slice 4 — annotation 行高度（ruby 行盒含 annotation 行）

**目标**：ruby 行盒高度 = base 行 + annotation 行（annotation 在上），使 ruby 后续文本与 ruby base 同行（不重叠 annotation）。

**seam**：IFC 行盒高度对含 ruby 的行加 annotation 行高（paint overlay 已在 base 行上方画 annotation，但行盒几何高度未含 annotation → annotation 可能侵入上一行）。vertical-align / line-box 高度计算处 gate 到 ruby owner。

**验收**：fixture 48 "after ruby." 与 ruby base 同行（不与 annotation 重叠）。

---

## 4. 推荐首切（最高 EV、最低风险）

**Slice 1 + 2 合并试 land**（rt 仍 inline，annotation 经新 segment overlay）：Slice 1 单独会使 rt 仍堆叠（rt 未 none）；Slice 2 单独 overlay 仍 owner-丢失。两者须协调。但合并风险高（layout-tree 容器 + paint loop 双改）。

**更稳首切 = Slice 1 only + rt→display:none**：Slice 1 让 base 走父 IFC（owner=parent），annotation 经「ruby box 几何定位 paint」（不依赖 base fragment owner——直接按 ruby LayoutBox 几何 + segment 居中 paint annotation）。这绕开 R1022 owner-丢失问题（annotation paint 不再挂在 base fragment 上，而挂在 ruby box 上）。

**env-gated A/B 探针**：首 session 实施 Slice 1 + rt-none + ruby-box-anchored annotation paint，kill-switch `ZW_RUBY_SIMPLE=1`，全量 css/CSS2/writing-modes + product-smoke A/B。若 net≥0 + fixture 48 ruby 正确 → default-on land；若 net-负 → gate 精化或回退记 entangled。

---

## 5. 已 ruled-out 路径（勿重试）

- **rt→display:none 单独**（R1683 probe）：annotation 消失（R1022 owner 丢失），marginal diff。须配合 base IFC 修 + annotation 重锚。
- **R1022 char-pairing overlay 修补**：`ruby_annotation_chars` 单 Vec<char> + base[char_idx] 配对对多 char base/annot 错配（`<ruby>base<rt>annot</rt>` 4-char base 配 5-char annot → 逐字错位）。须 segment-based 替换，非修补。
- **ruby 当独立 block 容器 + 内部 IFC**：会破坏 inline 流（ruby 应参与父 IFC，不建 BFC）。

---

## 6. 依赖与边界

- **不依赖** font-stack / vertical-mode / R109 / multicol（ruby 是独立 inline-box-model 子域）。
- **依赖** inline-box-model Phase-A（R125/R198/R205 谱系）的「inline 元素含 inline/block 混合子」处理——ruby base + rt 正是此形态。若 Phase-A 未解，ruby Slice 1 可能受阻（ruby 容器盒问题 = Phase-A inline-box direct-child 测量问题，R1662 已定位 fixture 37 label+input 同谱系）。
- **WPT yield 预期 = 0**：css-ruby 不在 driving reftest 集，CSS2 corpus ruby 稀少。ruby 是产品/legacy 可见性 + spec-correctness，非 reftest headline（≡ form-control 谱系）。

---

## 7. forward（next session）

1. 实施 Slice 1 + rt-none + ruby-box-anchored annotation paint（§4 推荐首切），env-gated A/B。
2. 若 net-负，定位阻塞（base IFC 参与是否受阻于 Phase-A）→ gate 精化或 defer 到 Phase-A 解锁后。
3. 成功后 default-on land，续 Slice 3（rt font-size 50%）+ Slice 4（annotation 行高）。

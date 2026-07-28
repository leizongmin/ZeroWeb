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
>
> **★ R2159（2026-07-29）更可处理路径发现：R639-extend + skip-taffy**（优于上方 IFC→LayoutBox backfill）。读 painter text.rs:1202-1290 发现 **R639 Phase A slice 已实现 per-fragment inline bg/border 绘制**（add_fill bg + per-fragment border-top/bottom），但 **gate `owner_h > frag_fs*1.5` 仅多行生效**（单行 inline 排除，依赖 LayoutBox 绘 bg/border = R1492 机制所在）。故深层 Phase A 可重构为：
> - **part 1（skip-taffy）**：tree.rs 对 inline Element 子不建块级 taffy 节点（同 R2156 模式），消除块级栈列。
> - **part 2（R639-extend）**：painter text.rs 放宽 R639 gate 到单行（`owner_h > 0` 而非 `> 1.5·fs`），让 per-fragment bg/border 覆盖单行 inline（补 part 1 丢的 LayoutBox bg/border）。
> - **耦合**：part 1 单独 = R1492-refuted（单行 bg/border 丢）；part 2 单独 = 双绘（LayoutBox + fragment 都绘）；**须同 gate 同开**。part 2 复用已有 R639 infra（仅放宽 gate）= 比 IFC→LayoutBox backfill（须新 element-boundary tracking）**更可处理**。
> - **关键风险点（R2159 已实证 CONFIRMED）**：R639 用 `self.inline_heights.get(&owner_id)`（pre-scan inline 元素高）判多行。**`inline_heights` 由 `collect_box_heights`（painter/mod.rs:383）遍历 LayoutBox 树填充**——skip-taffy 后 inline 元素丢 LayoutBox → 无 `inline_heights` 条目（owner_h=0）→ R639 gate 即便放宽到 `>0` 也不触。故 R639-extend 路径**须加 part 3：迁移 `inline_heights` 数据源**（从 LayoutBox 树改为 DOM/IFC 衍生，让 orphan inline 仍有高）。
> - **part 3（inline_heights 数据源迁移）**：`collect_box_heights` 改为从 IFC 行盒结果计算每个 inline 元素高（其跨行片段的 y 范围），或保留 LayoutBox 源 + 补 IFC 源双轨。这是 R639-extend 路径的额外成本，使其接近 IFC→LayoutBox backfill 的复杂度。
> - **结论（R2159）**：两条路径（R639-extend + part3 / IFC→LayoutBox backfill）均须 substantial 新 infra，**深层 Phase A 确属多 session**。19-testpage 22% + 20-mixed-legacy 13% 的 multi-inline fix 阻塞于此。
> - **20-mixed-legacy 13% diff = 同 Phase A**（R2159 确认）：内容列 `<p>...<b><i><font>...</p>` 多 inline 块级栈列→内容过高→table row 过高→左 menu td bgcolor=#eeeeee 延伸至 y=396（chromium 行早结束，下方白）。与 19-testpage 同根（multi-inline block-stacking）。

- ~~**gate（原 leaf-path，R1492-REFUTED）**~~：block 容器 + ≥2 inline Element 子 → leaf/IFC。**勿实施**（R1492 -10）。
- **gate（R2159 精化，R639-extend + skip-taffy，优先路径）**：part 1（tree.rs skip inline Element 子 taffy 节点）+ part 2（painter R639 gate 放宽到单行），同 `ZW_PHASEA_MULTI_INLINE` gate，default-off。前置 probe = 确认 inline_heights 在 orphan inline 下数据源。
- **gate（备选，IFC→LayoutBox backfill，更深）**：保留 inline Element LayoutBox + post-process 从父 IFC 行盒回填位置。仅当 R639-extend 路径 probe 不可行（inline_heights 数据源不可迁移）时采用。
- **kill-switch**：`ZW_PHASEA_MULTI_INLINE`（probe 期 default-off）。
- **driving test**：`<p>A<a>x</a><i>y</i><b>z</b>.</p>` 断言 a/i/b 同行（y 一致 x 递增）+ `<p>A<a style="background:yellow">x</a> text.</p>` bg 可见（R1492 守）+ 容器高≈单行非 3 行。
- **三态 A/B**：self-source 全目录零 delta + **chromium-oracle CSS2/borders 零漂移（R1492 守，关键）** + 19-testpage diff 22%↓ + 20-mixed-legacy diff 13%↓ + welcome 字节一致 + make test green。
- **前置 probe（S2-probe-0）**：确认 `inline_heights` 数据源（DOM vs LayoutBox 树），判断 R639-extend 路径对 orphan inline 可行性。
- **回退**：net 负 → `ZW_PHASEA_MULTI_INLINE=0`。
- **风险（R2159 精化后）**：blast-radius 大（所有多 inline block 容器）。R639-extend + skip-taffy 耦合须同 gate 同开（part1 单独 R1492-refuted，part2 单独双绘）。hit-test（`<a href>` 点击区）+ stacking-context inline（position/opacity/transform）须 gate 排除。

> **★ R2160（2026-07-29）probe 实测：机制成立但 self-source net −20 → 保持 default-off，未 land**。
> 完整 A/B 见 [`evidence/r2160-phase-a-slice2-multi-inline-probe-netneg-2026-07-29.txt`](./evidence/r2160-phase-a-slice2-multi-inline-probe-netneg-2026-07-29.txt)。
>
> - **part3 被 orphan 信号耦合避开（创新）**：part1 skip taffy → inline 无 LayoutBox → `inline_heights` 无条目 → `owner_h==0.0`；part2 把 R639 gate 放宽为 `(owner_h > fs*1.5 || phasea_orphan_fire)`，`phasea_orphan_fire = (gate_on && owner_h==0.0)`。**owner_h==0.0 既是 orphan 信号又是 part2 触发条件**，part1+part2 天然耦合（orphan 只走 fragment、非 orphan 单行只走 LayoutBox=无双绘），**无需 R2159 担心的 part3（inline_heights 数据源迁移）**。
> - **产品面 net POSITIVE**：19-testpage 22.39→17.23%（−5.16pp）/ 20-mixed-legacy 13.13→11.49%（−1.64pp）/ legacy 51 fixture sum 259.36→252.33%（−7.03pp aggregate）/ struct_FAIL 0→0（零结构性退化）/ chromium-oracle CSS2/borders OFF=ON=415/506（零漂移，**R1492 -10 未触发**，orphan-fire 精确耦合证实安全）。welcome 16.84→17.03%（+0.19pp，含多 inline 产品页 gate 触发致微移，非崩溃）。
> - **self-source reftest net NEGATIVE −20（违硬门「零 delta」）**：css-text 1742→1727（**−15**）+ css-multicol 264→256（**−8**）；css-position +1 / css-flexbox +1 / css-fonts +1；text-decor/tables/grid/writing-modes/normal-flow/box-display/generated-content 全 0。**css-text −15 明细**：PASS→FAIL 19 案（**line-break-{loose,normal,strict}-{011,014,016a,016b,017a,018} 占 18 = CJK 换行断点偏移** + text-wrap-balance-003），FAIL→PASS 4 案（letter-spacing-200/text-transform-fullwidth-009/eol-spaces-bidi-002/white-space-intrinsic-size-014）。CJK line-break 簇系统退步 = **真布局 damage 非 font-wall 噪声**（预存 FAIL 如 letter-spacing-201/203/204/206 仅 diff% 微摆未翻，噪声与真翻可分）。
> - **裁决**：机制（orphan-coupling）empirically 成立 + 产品面 positive，但 self-source −20 违设计门 → **保持 default-off（= 回退态），不翻 default-on**。probe 代码保留（codebase dormant ZW_ 先例）作 gate-tightening 迭代基础。make test EXIT=0 全绿（default-off 故 default 路径零影响）。
> - **forward（gate-tightening，下一 session）**：blast radius 集中 CJK line-break（−18 主因）+ multicol（−8）。候选：(a) `phasea_multi_inline_eligible` 加「祖先无 multicol」守卫（最低成本，预测恢复 multicol −8）→ A/B；(b) 排除 CJK 文本容器（预测恢复 line-break −18，可能牺牲部分产品增益）；(c) 仅对 inline 子无 text-styling（letter-spacing/word-break/line-break/text-autospace）触发（须查 computed style，较重）。先 (a) 孤立 A/B，css-text 仍 −15 再 (b)/(c)。**重启 slice 2 须先 (a)/(b)/(c) 把 self-source 拉回零 delta**，产品收益才能经 default-on 实现。
>
> **★ R2161（2026-07-29）gate-tightening 两 guard：probe net −20 → net +1，仍 default-off**。
> 完整 A/B 见 [`evidence/r2161-phase-a-slice2-gate-tighten-netplus-2026-07-29.txt`](./evidence/r2161-phase-a-slice2-gate-tighten-netplus-2026-07-29.txt)。
>
> - **guard 1 = br/wbr 排除（css-text −15 主因，R2160 forward 候选 (b)/(c) 之外的真因）**：插桩实证 line-break-18 案 regress 非直接触发——`<p class="control">` 的外 `<span>` 含 `<br/>`+`<span.target>` 两 Element 子，**`<br/>` 被误判 eligible**（br display:inline+childless+非ooflow）→ 外 span eligible_count=2 → gate 触发 → br skip taffy 丢强制换行 → test/control 不匹配。修=phasea_multi_inline_eligible 加 br/wbr tag 排除。A/B：css-text ON 1727→**1741**（+14，line-break 18 全恢复）。
> - **guard 2 = multicol-context 守卫（R2160 forward 候选 (a)）**：新 `container_in_multicol_context`（自身/祖先 column-count/width 非 Auto）→ gate 加 `&& !container_in_multicol_context(dom_id)`。multicol 列流依赖 inline 精确测量，probe 改测量破坏列几何（multicol-clip/gap-large/width-large 簇）。A/B：css-multicol ON 259→**264**（= OFF，−5 全恢复）。
> - **both guards 全量 A/B**：self-source aggregate **net +1**（css-text −1 [残 text-wrap-balance-003+pre-wrap-trailing-spaces-021] / multicol 0 / flexbox +1 / fonts +1 / position 0 [br-exclusion 后原 +1 消失] / text-decor·tables·grid·writing-modes·normal-flow·box-display·generated-content 全 0）。产品面增益全保留（19-testpage 22.39→17.23% / 20-mixed-legacy 13.13→11.49% / legacy sum 259.36→252.33% / struct_FAIL 0→0）+ chromium-oracle CSS2/borders OFF=ON=415/506 零漂移（R1492 安全）。welcome 16.84→17.03%（+0.19pp 残余）。make test EXIT=0 + fmt/clippy clean。
> - **裁决：仍 default-off**。net +1 满足「net≥0/净负回退」门，但 **welcome +0.19pp + css-text −1（text-wrap-balance CSS Text 4 feature）= 2 处残余回归**（虽 net +1 offset）违 redirect「零回归」严读 → 暂不翻 default-on。gate-tightening 代码保留 default-off。
> - **forward（残余解析 → flip default-on）**：(1) welcome +0.19pp 插桩定位 gate 触发处（<a>/<b>/<code> 多 inline），判修对 vs 退步；(2) css-text −1 判 text-wrap-balance 可排除或接受。两残余归零/documented-accept 后翻 default-on（default 路径全量三态 A/B 复测）land。**br-exclusion 教训：特殊 inline 元素（br/wbr/img）须按 tag 排除，纸面结构分析会漏 control-p 的 br**。

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

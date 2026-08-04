# Spec：Vertical-mode IFC 四层协调统一

**版本**：v0.1（R1099，2026-07-06，首版草案 — 多 session 切片计划 + R1043 converter-mirror 设计）
**日期**：2026-07-06
**作者**：AI Assistant（rendering-compat rally）
**状态**：⏳ **部分实施 + user-gated**：Slice α-1（container_width WM-aware + decoration-gate）已 LANDED（R1099，`painter/text.rs:532-546` + `inline_finalization.rs:17-24/782`，default 行为 = vertical decoration-free 容器取 content_height）；α-2 经 R1101 verified-unnecessary skip；**α-3（vertical 装饰）/α-4（converter block-flow mirror）/α-5（解 gate）仍 user-gated 未实施**。R109 vertical 经 R1965-R1978 深查确认为**耦合系统**（8 proof 单层修 couple-regress），更深架构 track 已迁 [`r109-vertical-native-layout-design.md`](./r109-vertical-native-layout-design.md)（vertical-native subtree layout，pre-authorized ruling #4 多 session）。行号/路径经 R2626 对齐当前源码（`inline/mod.rs`/`painter/text.rs` 重构后大幅漂移）。
**复杂度**：复杂（跨模块 / 4 层循环依赖 / 高回滚难度 / >3 依赖）

---

## 0. 执行摘要

- **一句话目标**：让 ZeroWeb 的 Inline Formatting Context（IFC）对 `writing-mode: vertical-rl/lr` 文本按规范做垂直字符推进（同 x 列、y 递增），并同步修复与之耦合的 block-flow 方向 / line-height vertical 列宽 / vertical 装饰坐标，解锁 css-writing-modes（~87% fail，corpus 最大目录 gap）+ baseline-export vertical 簇。
- **本期范围**：本 RFC 不立即落地全部四层；它定义**多 session 切片计划**（Slice α-1 … α-5），每切片独立 A/B 门禁（net-0/正即留，net-负即回退），后续 session 按序推进。
- **明确排除**：taffy 0.8+ 升级（R304 DEFERRED，541 ref 迁移，独立轨道，非本 RFC 范围）；horizontal-tb 任何行为变化（所有改动 WM-gate，horizontal-tb 字节一致零回归）；CJK 字体度量死锁（R633，font-wall 谱系，非布局层）。
- **核心约束**：① vertical 是**耦合系统**——R1047/R1050/R1052 三证单层修 net-negative（R1052 inline-flow 单修即便字符几何完全规范正确，oracle 仍净 -26 css-text-decor）；任一切片**必须**与其他已修层协调或显式 gate 回避未修层。② horizontal-tb 零回归（WM gate `is_vertical_wm`）。③ 每切片三态门禁：welcome product-smoke <20% + scoped oracle 零回归 + self-source 不降。
- **推荐方案**：Slice α 多 session 同步修（converter 层 block-flow mirror，**非** postprocess）+ inline-flow container_width WM-aware + line-height vertical 列宽 + vertical 装饰 re-enable；以 α-1（container_width fix，gate 回避装饰）为首个 net-0 探针切片。
- **首个落地步骤**：实施 Slice α-1（§4.1）——`inline_finalization.rs:782` + `painter/text.rs:546` 的 `container_width` 改 WM-aware（vertical 取 content_height），WM gate 隔离 horizontal-tb，A/B css-writing-modes + css-text-decor oracle 看 net 变化（预期：writing-modes 部分案字符位置正确化、text-decor 因装饰未协调 net-负 → 若 net-负则 gate 到无装饰案或回退等 α-3）。

---

## 1. 背景与目标

### 1.1 背景

ZeroWeb 的 IFC 对 `vertical-rl`/`vertical-lr` 文本**水平布局**（chars x 递增、y 恒 0），而非规范的垂直布局（chars 同 x 列、y 按 font-size 递增）。这致 css-writing-modes 目录 **~87% reftest 失败**（corpus 最大单目录 gap），并连带 text-emphasis / ruby / text-decoration / bidi 的 vertical 变体全部失败。

R1043 → R1050 → R1051 → R1052 四轮调查已精确定位根因与耦合结构（详见 [`vertical-inline-layout-handoff.md`](./vertical-inline-layout-handoff.md) v1.1）：

- **根因**：IFC `break_items_into_columns` 的 `max_depth = self.container_width`（`inline/mod.rs:942`），而 `container_width` 取自 `root.content_width`（水平 block 尺寸）。vertical-lr 容器 auto 时 `content_width = 0` → `max_depth = 0` → 每字符触发列断 → chars 横向排列。vertical 应取 `content_height`（竖直 inline 尺寸 = 字符向下推进可用深度）。
- **轴交换已存在**：R1051 v1.0「缺轴交换」诊断已被 R1052 VIFCDUMP 推翻——`break_items_into_columns`（mod.rs:940）与 paint 端 `char_advance_is_y`（text.rs:1204）轴交换代码早已存在且正确（commit 942a2948）。
- **耦合系统**：R1052 实施 Fix A+B（container_width WM-aware + trailing-space 裁剪）后，006d 字符几何**完全规范正确**（col0 run0..4 x=0 常量、y=0/16/32/48/64 单列），但 chromium Oracle **净 -26**（css-text-decor 108→82）。证 vertical 渲染是 4 层耦合系统，单修 inline-flow 不足以匹配 chromium。

### 1.2 目标

- **业务目标**：css-writing-modes oracle 通过率从 56/784 (7%) 提升至覆盖垂直文本用例（潜在 +30~80 案）；间接解锁 baseline-export vertical 簇 + text-emphasis/ruby/text-decor/bidi vertical 子域。
- **用户目标**：ZeroWeb 能正确渲染垂直书写模式页面（CJK 传统排版、vertical UI 标签等）。

### 1.3 范围边界

- **在范围内**：IFC `container_width` WM-aware（layout 侧 `inline_finalization.rs` + paint 侧 `text.rs`）；`break_items_into_columns` line-height vertical 列宽；converter 层 vertical block-flow 方向 mirror（R1043 converter，**非** postprocess）；vertical 装饰坐标 re-enable（text-emphasis site 2 门控解除 + text-decoration vertical 轴）。
- **不在范围内**：taffy 0.8+ 升级（R304，独立多 session 轨道）；CJK 字体度量（R633 font-wall）；horizontal-tb 任何行为变化；::first-letter / multicol Phase 2（独立轨道）。

### 1.4 关联文档（单一权威来源主从）

本 RFC 的**技术根因与 Fix A+B 设计**主定义在 [`vertical-inline-layout-handoff.md`](./vertical-inline-layout-handoff.md) §0/§2，本 RFC 只摘要并引用，不重复。本 RFC 的主定义区是**多 session 切片计划（§4）+ R1043 converter-mirror 设计（§5）+ 验证门禁（§6）**。

---

## 2. 现状分析：四层耦合架构

vertical-mode 渲染正确性依赖**四层协调**。每层当前状态、规范要求、fix 设计如下。**铁律**：R1052 三证任一单层修 net-negative，故多 session 切片须保证「已修层协调 + 未修层显式 gate 回避」（§4 切片依赖图）。

### 2.1 Layer 1 — inline-flow 字符推进（container_width）

| 项 | 现状 |
|---|---|
| 规范要求 | vertical-rl/lr：chars 同 x 列、y 按 font-size 递增向下推进；列沿 x（vertical-rl 向左 / vertical-lr 向右）。 |
| 当前行为 | `max_depth = self.container_width`（`inline/mod.rs:942`）= `content_width`（水平 block 尺寸）= 0（vertical auto 容器）→ 每字符触发列断 → chars 横向 x 递增、y 恒 0。 |
| Fix | `container_width` WM-aware：vertical 取 `content_height`（竖直 inline 尺寸）。两接线点：`inline_finalization.rs:782`（layout stored）+ `painter/text.rs:546`（paint Path B re-run）。horizontal-tb 取 content_width（WM gate 零回归）。详见 handoff §2 Fix A。 |
| 附带 Fix | trailing-space 裁剪（`break_items_into_columns` 词循环头，CJK per-char 词不应带 trailing space 致 word_height 虚高）。详见 handoff §2 Fix B。 |
| 单层实证 | R1052 Fix A+B 单修此层：字符几何完全规范正确，但 oracle 净 -26（Layer 3/4 未协调）。 |

### 2.2 Layer 2 — block-flow 方向（R1043 converter mirror）

| 项 | 现状 |
|---|---|
| 规范要求 | vertical-rl：block 方向（列推进）从右向左（首列在最右）；vertical-lr：从左向右。vertical 容器自身及其 in-flow 子元素的纵向（block 轴）packing 须按 WM 反转。 |
| 当前行为 | taffy 0.7 `Display::Block` 不支持 rl/lr 方向 packing（固定从左上起）。vertical 容器定位错（首列在最左而非最右）。 |
| Fix | **converter 层 mirror**（`converter/mod.rs`，R1043 已证 postprocess mirror net-negative——float-exclusion/margin-collapse 状态丢失），**非** postprocess。详见 §5（本 RFC 主设计）。 |
| 单层实证 | R1043 postprocess mirror net-negative（ruled out）；converter 层未试（本 RFC 新设计）。 |
| 依赖 | taffy 0.7 限制 → converter 层在 taffy 布局后做方向 mirror，须保留 float/margin-collapse 状态（postprocess 失败的原因）。 |

### 2.3 Layer 3 — line-height vertical 列宽

| 项 | 现状 |
|---|---|
| 规范要求 | vertical-mode 列宽（= horizontal-mode 行高）= `line-height`。如 `line-height:5` × fs16 = 列宽 80。 |
| 当前行为 | `break_items_into_columns` 里 `col_width = run.line_height`（mod.rs:965/1010）看似正确，但 R1052 实测 006d（line-height:5）`col_width=16`（应 80）—— line-height:5 **未传**到 vertical 列宽（计算路径某处丢失或未 WM-aware）。 |
| Fix | 排查 line-height 值在 IFC 构建链路（TextRun.line_height ← compute）丢失点；确保 vertical 列宽 = line-height × fs。须 VIFCDUMP 探针确认（handoff §5）。 |
| 单层实证 | 未独立测（R1052 随 Layer 1 一起观察）。 |

### 2.4 Layer 4 — vertical 装饰坐标（emphasis / text-decor / ruby）

| 项 | 现状 |
|---|---|
| 规范要求 | vertical-mode：text-emphasis 在字符左右（非上下）；text-decoration underline/overline 在左右（非上下）；ruby annotation 在字符侧。 |
| 当前行为 | painter `!char_advance_is_y` 门控（R1050 site 2）跳过 vertical 装饰渲染 → vertical 装饰完全未渲染或位置错。 |
| Fix | 装饰坐标 WM-aware：vertical 时 emphasis/decor 偏移轴从 (x,y) 互换。re-enable `char_advance_is_y` 分支的装饰路径。 |
| 单层实证 | R1050 text-emphasis vertical net -8（已回退，因 Layer 1 未修致装饰位置无意义）。Layer 1 修后须重新 A/B。 |

### 2.5 耦合机制（为何单层 net-negative）

Layer 1 修后字符几何正确，但：
- Layer 2 未修 → 整列定位错（首列在左非右）→ 整体偏移。
- Layer 3 未修 → 列宽错（line-height:5 仍 16 非 80）→ 列间距错。
- Layer 4 未修 → 装饰缺失/错位 → text-decor 簇 oracle -26（R1052 实测）。

输出既不同于旧错误布局、又不同于 chromium → 净负。**必须协调**。

---

## 3. 影响范围

vertical inline 布局缺失致以下子域**全部 R109-blocked**（详见 handoff §1，本 RFC 引用不重复）：

- css-writing-modes vertical 用例（block-flow-direction-* / line-box-direction-*，~250 案 86-87% worst）— **本 RFC 主 yield**。
- text-emphasis-position vertical 簇（003/005/006，~12 案）。
- ruby vertical annotation（R1022 仅水平 rt 上移）。
- text-decoration vertical（underline/overline 应左右）。
- bidi-vertical（bidi-007 簇残余）。
- baseline-export vertical 簇（R1098 确证 entangled）。

**解锁 yield 估计**（handoff §1）：css-writing-modes 56/784 → 潜在 +30~80 案（vertical 用例字符位置正确化后，残余仅 font/aa 噪声）。是当前 corpus 最高 yield 单轨道。

---

## 4. 多 session 切片计划（★ 本 RFC 主交付）

每切片**独立 A/B 门禁**：`make reftest-oracle DIR=<scoped>` 看 net oracle-pass 变化 + `make product-smoke`（welcome <20%）+ self-source 不降 + horizontal-tb 字节一致。**net-0/正即留，net-负即回退并记 evidence**。切片间有依赖（未协调层须显式 gate 回避）。

依赖图：`α-1 (Layer1) → α-2 (Layer3, done) → α-4 (Layer2) → α-3 (Layer4) → α-5 (解 gate)`。

> **★ R1102 依赖修订**：原序 α-1→α-2→α-3→α-4 被 α-3 实测 net-negative -26 推翻。vertical 文本列流向（Layer 2 block-flow，α-4）是其他层的几何基线——在错的 block-flow 上修 Layer 4 装饰只会更远离 chromium（α-3 实测 css-text-decor 108→82）。**修订：α-4（block-flow）须先于 α-3（装饰）**。详见 master.md R1102。

### 4.1 Slice α-1 — Layer 1 container_width WM-aware（decoration-gate 探针）

- **范围**：`inline_finalization.rs:782` + `painter/text.rs:546` 的 `container_width` 改 WM-aware（vertical 取 `content_height`）+ handoff §2 Fix B（trailing-space 裁剪）。**Gate**：仅对「vertical 容器且整棵子树无 text-decoration / text-emphasis / ruby」触发（decoration-aware gate，回避 Layer 4 耦合）。
- **文件**：`crates/layout-engine/src/inline_finalization.rs`、`crates/engine/src/paint/painter/text.rs`、`crates/layout-engine/src/inline/mod.rs`（Fix B）。加 `has_vertical_decoration_descendant` helper（DOM/style 扫描）。
- **预期**：css-writing-modes 纯文本 vertical 案（block-flow-direction-* / line-box-direction-* 无装饰者）字符位置正确化，潜在 +N；css-text-decor net-0（装饰案 gated out，未被 α-1 触碰）。
- **门禁**：A/B `make reftest-oracle DIR=css-writing-modes` + `DIR=css/css-text-decor`。net ≥0 且 writing-modes 有正 yield → LAND；net-0 且无正 yield → 记 evidence，留作 α-3/α-4 前置基础（dormant 不影响）；net-负 → gate 不够精（R1052 三 gate 证简单 gate 不足），升级 gate 或回退。
- **风险**：R1052「三种 gate 均 -1」指简单 gate（如纯 CJK / 单列）不足；本切片 decoration-aware gate 更精（按 style 属性 gate 而非结构），未证。若仍 net-负，证 Layer 4 耦合无法 gate 回避，须 α-1+α-3 bundle。
- **依赖**：无前置（Layer 1 是基础）。

### 4.2 Slice α-2 — Layer 3 line-height vertical 列宽（依赖 α-1）

- **范围**：排查 `run.line_height` 在 IFC 构建链路丢失点，确保 vertical 列宽 = line-height × fs。须先重加 `VIFCDUMP=1` 探针（handoff §5）确认 006d line-height:5 实测 col_width。
- **文件**：`crates/layout-engine/src/inline/mod.rs`（TextRun.line_height 计算 / break_items_into_columns col_width）。
- **预期**：css-writing-modes line-height-NNN vertical 案列宽正确化。
- **门禁**：A/B `DIR=css-writing-modes`。net ≥0（α-1 已 gate 装饰，无 Layer 4 干扰）→ LAND。
- **依赖**：α-1（chars 须先垂直推进，列宽才有意义）。
- **★ R1101 verified-unnecessary，skip**：新单测 `test_r1100_alpha2_vertical_line_height_column_width`（vertical-rl + line-height:5 + fs16 → 断言 `ctx.lines[0].height` = col_width = 80）**PASS**——post-α-1 tree `resolve_font_metrics`（text_metrics.rs:392 `Number(n) => fs×n`）→ `run.line_height` → `break_items_into_columns` `col_width = run.line_height`（mod.rs:965/1010/1018）链路**已正确**。R1052「col_width=16」是 pre-α-1 stale 观测（彼时 container_width=0 致 break 异常），post-α-1 不复现。**α-2 无需实施**，单测留作回归守卫。

### 4.3 Slice α-3 — Layer 4 vertical 装饰坐标 re-enable（依赖 α-1）

- **范围**：painter `char_advance_is_y` 分支装饰路径 re-enable：text-emphasis 偏移轴 (x,y) 互换（vertical 时左右非上下）+ text-decoration underline/overline 轴互换 + ruby annotation 侧移。
- **文件**：`crates/engine/src/paint/painter/text.rs`（emphasis site 2 门控解除 + decor 坐标 WM-aware）、`painter/effects.rs`（text-decoration 轴）。
- **预期**：css-text-decor vertical 案 + text-emphasis-position 003/005/006 簇解锁（R1052 -26 的反向修复）。
- **门禁**：A/B `DIR=css/css-text-decor` + css-text-decor 子目录。net ≥0（与 α-1 bundle 后应恢复 α-1 gate out 的案）→ LAND。
- **依赖**：α-1（chars 须垂直推进，装饰坐标基于字符位置）。建议 α-1 gate 案在此切片后解除（α-5）。
- **★ R1102 修订：α-3 依赖 α-4 先行**。α-3 单独实施（vertical emphasis + 收窄 α-1 gate）A/B css-text-decor 108→82（**-26 net-negative**）。根因：vertical 列流向（Layer 2 block-flow，α-4 未修）是装饰坐标的几何基线；在错的列流上正确放置 emphasis → 比 chromium 更远。**α-3 须在 α-4（block-flow mirror）之后**。原"α-1 gate 案在 α-3 后解除"推迟到 α-4+α-3 都 done。

### 4.4 Slice α-4 — Layer 2 converter-layer block-flow mirror（依赖 α-1，最难）

- **范围**：vertical-rl block 方向 mirror（首列在最右）。**converter 层**（`converter/mod.rs`）集成，**非** postprocess（R1043 证 postprocess mirror net-negative——float-exclusion/margin-collapse 状态丢失）。
- **设计**：详见 §5（本 RFC 主设计，TBD 具体实现——须 dedicated session 调查 taffy 布局后 position 计算管线的 mirror 注入点）。
- **预期**：css-writing-modes block-flow-direction-rl 簇解锁（vertical-rl 首列右侧）。
- **门禁**：A/B `DIR=css-writing-modes`。net ≥0 → LAND。**最高风险切片**（R1043 mirror 谱系，可能须 taffy 升级 R304 替代）。
- **依赖**：α-1（垂直字符推进是 block-flow mirror 的基础）。

### 4.5 Slice α-5 — 集成 + 解 gate + 全量 oracle（依赖 α-1/α-2/α-3/α-4）

- **范围**：解除 α-1 的 decoration-gate（α-3 已协调装饰）；全 corpus A/B；补单测；VIFCDUMP/EMPHDBG 探针移除或留 env-gated。
- **门禁**：全 corpus `make reftest-oracle`（无 DIR）net ≥0 + welcome <20% + `make test` 全绿 + clippy/fmt。
- **依赖**：α-1 至 α-4 全 LANDED。

### 4.6 备选轨道 — taffy 0.8+ 升级（R304，减耦合）

若 α-4 converter-mirror 不可行（taffy 0.7 限制无法绕过），先做 taffy 0.8+ 升级（R304，native vertical block-flow + baseline_overrides），减 Layer 2 耦合后 α-1 单修可能 net-0/正。taffy 升级是独立多 session 轨道（541 ref 迁移），非本 RFC 范围，但列为 α-4 fallback。

---

## 5. R1043 converter-layer block-flow mirror 设计（TBD · 主设计方向）

**问题**：vertical-rl block 方向（列推进）应从右向左（首列最右）。taffy 0.7 `Display::Block` 固定从左上起，不支持 rl packing。

**R1043 postprocess mirror 失败原因**：postprocess 是布局后的独立 pass，重新计算子元素位置时**丢失 float-exclusion zone / margin-collapse 状态 / abspos CB 关系**（这些在 taffy 布局期建立，postprocess 重定位破坏）。

**converter-layer 方向（本 RFC 提议，TBD 具体实现）**：
1. **不在 postprocess mirror**，而是在 `converter/mod.rs` 把 vertical-rl/lr 容器的 WM 信号注入 LayoutBox 元数据（`block_flow_direction: Rtl | Ltr | Ttb`），供布局后**单一 position 计算管线**在计算 x 坐标时按方向反转。
2. 关键：mirror 须在**既有 position 计算管线内**做（state live），非新 pass。即：`position_cells` / 子元素 x 赋值处，对 vertical-rl 容器，子元素 x = `container_right - child_width - accumulated_offset`（而非 `container_left + accumulated_offset`）。
3. **须 dedicated session 调查** position 计算管线入口（engine.rs / tree.rs position 赋值点），确认 mirror 注入点不破坏 float/margin-collapse（与 R1043 postprocess 区别 = 在管线内 vs 新 pass）。

**Open Question（TBD-1）**：taffy 0.7 布局后，子元素 x 坐标是否可在不重跑 taffy 的前提下按 WM 反转？若不可（float/abspos CB 绑定 x），α-4 须 taffy 升级（§4.6 备选）。须 α-4 session 先做 spike：minimal vertical-rl 容器 + 测 mirror 注入点是否破坏 float case。

**★ R1103 spike 完成（TBD-1 resolved：formula 可行，但注入点须 final-pass）**：
- **可行性 YES**：LAYOUT_DUMP 实测 vertical-rl 块子当前左到右（A x=8 / B x=28），规范应右到左。mirror 公式 `child.x = width - child.x - child.width`（border-box 空间）手算正确（A 0→20 / B 20→0）。
- **vertical-lr 已正确**：converter+extract_layout 轴交换给 vertical-lr 左到右块流（符合规范），仅 vertical-rl 需 mirror。
- **float/margin-collapse 风险低**：mirror 是纯位置变换（非重定位），float 后处理更早完成（无冲突），区别于 R1043 postprocess mirror 独立 pass 丢状态。
- **★ 注入点 refined——extract_layout 过早**：在 engine.rs:768（HorizontalTb children 调整旁）加 VerticalRl mirror 分支后，外 div width 40→784（content-based block-size 变 full-width）。根因：extract_layout 在 apply_intrinsic_content_sizing / remeasure 等多次重算 pass 中反复调用，输出 mirror 被下游 size-recompute 读到（mirrored 子 x → max right edge → width=784）。**裁决：mirror 须放 compute 流程末尾 final pass（所有 set_style+mark_dirty 重算 + postprocess 之后），或 taffy 级变换**，不能放 extract_layout。下会话 α-4 实施 = engine.rs compute 末尾加 vertical-rl mirror pass。

---

## 6. 验证与门禁

### 6.1 通用门禁（每切片必须全过）

| 门禁 | 命令 | 标准 |
|---|---|---|
| 编译 | `cargo check --workspace` | 零错误 |
| 单测 | `make test` | 全 workspace 绿（零 FAILED） |
| Clippy | `cargo clippy --workspace --all-targets --D warnings` | 零 warning |
| Fmt | `cargo fmt` | 无变更 |
| 产品 smoke | `make product-smoke` | welcome <20%（DC-13） |
| Scoped oracle | `make reftest-oracle DIR=<dir>` | net oracle-pass ≥0（vs pre-slice baseline） |
| Horizontal-tb 零回归 | WM gate `is_vertical_wm` 隔离 | horizontal-tb 路径字节一致 |

### 6.2 探针基础设施（实施期 env-gated，默认 off）

| 探针 | env | 用途 | 来源 |
|---|---|---|---|
| VIFCDUMP | `VIFCDUMP=1` | dump IFC content_w/h + break_items_into_columns per-col/per-frag 几何 | R1052（须重加，已 revert） |
| EMPHDBG | `EMPHDBG=1` | dump vertical char 位置 + emphasis 渲染 | R1050（须重加） |
| LAYOUT_DUMP | `LAYOUT_DUMP=1` | dump frag abs_y/height/margin | R1050（既有） |

### 6.3 A/B 方法

baseline = pre-slice HEAD；treatment = 切片改动。`ORACLE_DUMP_ALL=1 make reftest-oracle DIR=<dir>` per-case dump，python 计算 z_vs_chr<1.0% pass 数。三态门禁：net-0/正留，net-负回退 + 记 evidence。

---

## 7. 实施交接（Implementation Handoff）

### 7.1 文件/模块清单

| 路径/模块 | 动作 | 切片 | 目的 | 风险 |
|---|---|---|---|---|
| `crates/layout-engine/src/inline_finalization.rs:782` | 修改 | α-1 | container_width WM-aware（layout stored） | horizontal-tb 须 WM gate 零回归 |
| `crates/engine/src/paint/painter/text.rs:546` | 修改 | α-1 | container_width WM-aware（paint Path B） | paint Path B 空-styles 协调（R890） |
| `crates/layout-engine/src/inline/mod.rs` (break_items_into_columns:940-1060) | 修改 | α-1/α-2 | Fix B trailing-space + line-height 列宽 | CJK 词边界 |
| `crates/engine/src/paint/painter/text.rs` (emphasis site 2 + decor) | 修改 | α-3 | vertical 装饰坐标 re-enable | R1050 site 2 门控 |
| `crates/engine/src/paint/painter/effects.rs` | 修改 | α-3 | text-decoration vertical 轴 | — |
| `crates/layout-engine/src/converter/mod.rs` | 修改 | α-4 | block_flow_direction 元数据注入 | R1043 mirror 谱系（最高风险） |
| `crates/layout-engine/src/engine.rs` / `tree.rs` (position 赋值) | 修改 | α-4 | position 计算管线 mirror 注入 | float/abspos CB 状态 |

### 7.2 推荐修改顺序（按依赖）

1. **α-1**（container_width + Fix B + decoration-gate）— 基础，net-0/正探针。
2. **α-2**（line-height vertical 列宽）— 依赖 α-1，加 VIFCDUMP 探针先确认。
3. **α-3**（vertical 装饰 re-enable）— 依赖 α-1，恢复 α-1 gate out 案。
4. **α-4 spike**（converter-mirror 可行性）— 判断 taffy 0.7 限制可绕否；不可绕则转 §4.6 taffy 升级。
5. **α-4 实施**（若 spike 通过）或 **taffy 升级**（若 spike 失败）。
6. **α-5**（解 gate + 全量 oracle + 收口）。

### 7.3 首批提交建议

| Batch | 切片 | 范围 | 预期结果 | 验证 |
|---|---|---|---|---|
| Commit α-1 | Slice α-1 | container_width WM-aware + Fix B + decoration-gate + 探针 | writing-modes 纯文本 vertical 案字符位置正确化，net ≥0 | `make reftest-oracle DIR=css-writing-modes` + `css/css-text-decor` A/B |

---

## 8. Spec Lint 报告

### 8.1 结构完整性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 执行摘要存在性 | ✅ Pass | §0 含一句话目标/范围/排除/约束/方案/首步 |
| 场景存在性 | ⚠️ Warning | 本 RFC 是多 session 架构计划，FR 以「切片」形式定义（§4），每切片有门禁场景但非 BDD Given/When/Then 格式（rally 架构 doc 非用户特性 spec） |
| 异常路径覆盖 | ✅ Pass | 每切片含 net-负 回退分支（α-1 gate 不够精 / α-4 spike 失败转 taffy 升级） |
| 测试绑定 | ✅ Pass | §6 门禁表 + §4 每切片 A/B `make reftest-oracle DIR=` 命令 |
| TBD 清零 | ✅ Pass | TBD-1（α-4 converter-mirror 可行性）标「重要」非「阻塞」，有 fallback（§4.6 taffy 升级） |
| 约束覆盖 | ✅ Pass | §0 核心约束 3 条 + §6 门禁覆盖 |
| 实施交接完备 | ✅ Pass | §7 含文件清单 + 职责 + 修改顺序 + 首批提交 |
| 首步可执行性 | ✅ Pass | §0 首个落地步骤 + §7.2 step 1 = α-1，明确文件 + gate + A/B 命令 |

### 8.2 语言精确性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 模糊动词 | ✅ Pass | 用「字符位置正确化」「列宽 = line-height × fs」等具体行为 |
| 无量化描述 | ✅ Pass | yield 量化「+30~80 案」、门禁「<20%」「net ≥0」 |
| 非确定性措辞 | ⚠️ Warning | §5 含「TBD」「提议」（α-4 设计故意留 TBD，已显式标 TBD-1，非隐藏模糊） |

### 8.3 一致性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 范围冲突 | ✅ Pass | §1.3 在范围/不在范围无交集 |
| 约束冲突 | ✅ Pass | §0 约束无矛盾 |
| 方案漂移 | ✅ Pass | §4 切片依赖图与 §1.3 范围一致 |
| 章节引用正确 | ✅ Pass | §0/§4 引用 handoff §0/§1/§2 实际存在 |
| 外部事实保守化 | ✅ Pass | taffy 0.7 限制（R1043/R304 已证）、line-height 列宽 col_width=16（R1052 VIFCDUMP 实测）均标实证来源 |
| 实现来源闭合 | ✅ Pass | §7.1 每文件-动作-切片映射；α-4 converter-mirror 标「TBD 须 spike」 |
| 类型分层清晰 | ✅ Pass | 需求（§1.2/§3）/决策（§0 推荐方案）/假设（§5 TBD）/TBD（§5 TBD-1）分层 |

**汇总**：18 Pass / 2 Warning / 0 Fail / 0 Skip
**门禁判定**：Fail = 0 → 允许作为 rally 续跑蓝图落地；2 Warning（BDD 格式 + α-4 TBD）为 rally 架构 doc 固有特性，非阻塞。

---

## 9. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|---|---|---|---|---|
| TBD-1 | α-4 converter-layer block-flow mirror 可行性 | 重要 | taffy 0.7 布局后子元素 x 是否可不重跑 taffy 按 WM 反转（不破坏 float/abspos CB） | α-4 session 先做 spike（minimal vertical-rl + float case），不可行则转 §4.6 taffy 升级 |
| TBD-2 | α-1 decoration-gate 是否够精（R1052 三简单 gate 不足） | 重要 | decoration-aware gate（按 style 属性 gate）net 是否 ≥0 | α-1 实施时 A/B 实测，net-负则须 α-1+α-3 bundle |

---

## 10. 修订历史

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v0.1 | 2026-07-06 (R1099) | 首版草案：多 session 切片计划（α-1..α-5）+ R1043 converter-mirror 设计方向 + 验证门禁 + 实施交接 |
| v0.2 | 2026-08-04 (R2626) | 行号/路径漂移纠偏（`inline/mod.rs`/`painter/text.rs` 重构后）+ 状态同步：标 α-1 LANDED（R1099）+ α-2 skip（R1101）+ α-3/α-4/α-5 user-gated + 更深 track 迁 `r109-vertical-native-layout-design.md`。纠偏：`break_items_into_columns` 1456→940 / `max_depth` 1458→942 / `col_width` 1481/1525/1568→965/1010/1018 / `char_advance_is_y` 1392-1450→1204 / container_width WM-aware `inline_finalization.rs`:123→782 + `painter/text.rs`:797→546 / `resolve_font_metrics Number(n)` 197→392 / engine.rs:1302→768 |

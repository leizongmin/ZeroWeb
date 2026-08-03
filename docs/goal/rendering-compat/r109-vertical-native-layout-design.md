# 设计：R109 Vertical-Native Layout（axis-swap emulation → native vertical subtree layout）

**版本**：v0.1.4（session 2 续：R1980 extent-backfill+R1978-sync 综合 broad A/B = net -27 → R1978 sync 是 paint-layer-only，layout-cascade 主导）
**日期**：2026-07-24（R1966 起草 / R1968 Slice 1 A/B refutation / R1969 timing-impasse + pre-taffy injection）
**作者**：AI Assistant（rendering-compat rally）
**状态**：scoping / 架构设计草案（pre-authorized ruling #4 multi-session track，session 1 = 本架构设计；implementation 待后续 session 按切片推进）
**模式**：rally-pattern 设计文档（非 lei-spec-rfc skill —— 该 skill 需用户确认，与无人值守 rally 协议冲突；同 `unified-font-stack-design.md` / `column-aware-IFC-spec.md` 先例）
**关联**：master.md R1043/R1050/R1052/R1099/R1541/R1544/R1545/R1895/R1910/R1963/R1964/R1965/R1966/R1968；[`evidence/r1963-...`](./evidence/r1963-taffy-zero-vertical-awareness-zw-axis-swap-emulation-r109-reopened-2026-07-24.txt)；[`evidence/r1966-...`](./evidence/r1966-vertical-float-inf-4path-no-zw-lever-2026-07-24.txt)；`vertical_block_flow.rs`；`crates/taffy-local/`

> **📍 v0.1.2 addendum（R1969，2026-07-24）·timing-impasse 架构发现：vertical container 正确 height 须在「taffy 对 horizontal 父兄弟定位之前」已知·三 timing 全分析·pre-taffy injection 是未试新方向**：承接 R1968（Slice 1 postprocess height-only net -1，第 7 证）。本轮 code-archaeology `shift_siblings_after_ifc_grow`（postprocess.rs:802-940）揭示 R109 vertical 的**真架构死结**非「7 证 couple-regress」现象层，而在 **timing**：vertical container 的正确 height（物理 height = inline-size = max 子高）须在 **taffy 对 horizontal 父的后续 in-flow 兄弟定位之前**已知，否则兄弟用错误（Σ）height 定位 → 不一致。三 timing 分析：① **post-taffy postprocess 写（R1968）**= 太晚：兄弟已被 taffy 用旧 Σ-height 定位；改 height 后产生 **gap（shrink，Σ→max 多子常缩）**，而 `shift_siblings_after_ifc_grow` 检测（line 932-940 `child.y < prev_bottom - 0.5`）**仅修 overlap（grow）不修 gap（shrink）**——因 gap-closing 歧义（bug vs 合法 margin/spacing），shift 基础设施设计上只安全处理 overlap → R1968 net -1 的根因。② **post-taffy mark_dirty 重跑（R1965 height-set）**= 重跑 cascade couples（vlr-008 float +12.86pp）。③ **PRE-taffy definite-size injection（未试）**= **最干净**：在 taffy 单趟布局**之前**把 vertical container 的正确 height 作为 definite size 注入 → taffy 一次定位兄弟正确，无 postprocess gap、无 mark_dirty 重跑 cascade。R1965 是 post-taffy mark_dirty（须第一趟 taffy 测），本方向是 **pre-taffy 从 styles 直接测**（无第一趟 taffy）——**genuinely 新、未试**。★ **可行性**：对 **definite-size 子的 pure-block vertical 容器**（子 width/height 从 styles 已知），可在 converter/build_subtree（taffy 树构建前）算 height=max 子高 + frame，注入为 definite taffy size → 单趟 taffy 正确。**局限**：auto-size 子（文本内容）须 IFC 预测（container_width 循环依赖，TBD2 territory），故首切片 scope = **definite-children 子集**。★ **axis-swap 交互 TBD**：注入的物理 height 经 `apply_vertical_writing_mode` axis-swap 变 taffy width，须确认 swap 后 taffy 用对维度（须 converter 层 probe）。★ **forward（next session，concrete code slice）**：实现 pre-taffy vertical-height injection（converter 或 build_subtree 前 pass），gate `ZW_VERTICAL_PRE_INJECT` default-off，scope pure-block vertical 容器 + definite-size 子，A/B css-writing-modes 784 案守 net≥0。若 yield = R109 vertical 首净 win（多轮来首次）；若 couple-regress = 第 8 证 + 关闭 pre-taffy 角度（definitive）。区别 R1965（post-taffy mark_dirty）/ R1968（postprocess）：本切片在 **converter 层、taffy 之前**。

> **📍 v0.1.3 addendum（R1970，2026-07-24）·pre-taffy injection 实测 0 yield → R109 vertical 失败是 text-content-IFC-measurement-dominated，非 timing/layout·timing 三角度全 irrelevant·核心收窄到 IFC circular-dependency**：承接 R1969 forward（pre-taffy injection slice）。本轮实现 pre-taffy injection（tree.rs build_subtree 轴交换后注入 vertical container definite height=max 子高+frame，gate `ZW_VERTICAL_PRE_INJECT` default-off，scope pure-block + 全 definite-Px-height 子）+ A/B css-writing-modes 784 案：**baseline 133/784 (17%) / strict 68 / near 65 / mismatch 651 → treatment 133/784 / strict 68 / near 65 / mismatch 651 = 完全一致 0 yield**。★ **根因**：gate「全 definite-Px-height block 子」在 css-writing-modes corpus **匹配 0 案**（或仅已 pass 案）——corpus 的 vertical 容器（block-flow-direction 簇主导失败）用 **文本内容 `<p>` 子（auto-height）**，非 definite-height block。★ **重大收窄**：R109 vertical reftest 失败 **= text-content vertical IFC measurement 主导**（R1964 float `<p>` inf 谱系 + R1099 α-1 container_width WM-aware 的 **circular-dependency 核心** TBD2），**非 timing/layout**。三 timing（postprocess R1968 / mark_dirty R1965 / pre-taffy R1970）**全 irrelevant**——它们都假设子 height 已知（definite），但 corpus 主导失败案的子是 auto-height 文本，须 IFC 测 inline extent（= 物理 height），而 IFC container_width 对 vertical = content_height（循环）。★ **definitive 收束**：R109 vertical 的真核心 = **vertical text-content IFC extent measurement 解循环依赖**（measure extent 须 container_width，container_width=content_height 须 extent）。任一 timing/axis-swap patch 皆不触及此核心（8 证：4 axis-swap 单层 + height-set + postprocess + output-clamp + pre-taffy injection）。**vertical-native 的真难点 = 此循环依赖的 breaking**（须 two-phase measure 或 unbounded-container single-line extent fallback）。代码 revert（0 yield，R1636 先例）。★ **forward（next）**：R109 vertical 须 dedicated 攻 **IFC circular-dependency breaking**（非 timing 切片）；或维持 plateau 待 font-stack 授权。timing 三角度 definitive closed。

> **📍 v0.1.4 addendum（R1980，2026-07-24）·extent-backfill（R1977 值）+ R1978 sync 综合 broad A/B = net -27·R1978 sync 是 paint-layer-only（necessary-but-NOT-sufficient）·height-postprocess 第 5 证·vertical-native 须 layout 层整体**：承接 R1979 forward（R1978 sync 机制 reusable，下步试更广 height-change+sync 或整合入 vertical-native）。本轮把 R1976（extent 可算 frag.height）+ R1977（backfill extent 值，无 sync，net -1）+ R1978（sync inline_layout_width 保 Path A）**首次拼齐**——这是 R1976 后自然的未试组合（R1977 失败时缺的 = R1978 sync；R1976 主案 box height=20 finite-but-wrong，须扩 finite，非仅 inf）。实现 `apply_vertical_extent_backfill`（gate `ZW_VERTICAL_EXTENT_BACKFILL`，vertical 叶文本块排除 table/flex/grid → content_height=IFC extent + sync inline_layout_width；`ZW_VERTICAL_EXTENT_INF_ONLY=1` 限 inf）。★ **broad A/B**（784，per-case）：baseline 130 → treatment **103 = NET -27**（125 worsened / 11 improved）；最大回归 line-box-direction-slr-050 +65pp / vlr-010 +60pp；少数真改善 slr-058/-060/vlr-018/-020 -27pp + vlr-014 flip PASS。★ **inf-only+sync A/B**：131 = **net +1（noise）**，仅动 5 个 logical-props 子-1.5% near-pass 案，无真 flip——sync 不能单独救回 R1977。★ **裁决**：**R1978 sync = necessary-but-NOT-sufficient**——sync 仅修 **PAINT 层**（width_matches gate 保 stored glyphs Path A），但 broad 回归主导是 **LAYOUT 层 cascade**（改 box height → horizontal 父兄弟定位 / 父 auto-height，taffy layout 期用旧 height 定位，postprocess 改后不一致 = 本设计 §1.3 R1968 timing-impasse 模式），paint sync 触不到；broad -27 证 layout-cascade 远大于 paint-Path-B。**vertical height postprocess = 非 lever（5 证：R1972/R1973/R1977/R1979/R1980）**——任何 scope（inf/finite/child/leaf）+ 值 + 机制（有/无 sync）组合皆 net-negative 或 noise。R1978 sync 仍 valid + reusable（已在 child-fill dormant）作 vertical-native building block，但 height-postprocess 非其载体。★ **对本设计的强化**：方案 B（post-taffy native re-layout）的正确性进一步确认——vertical-native 须 **layout 层整体**（post-taffy 对 vertical 子树自建完整 layout：positions + sizes + 兄弟重定位 + IFC extent 一次算完，不依赖 taffy layout 期定位），非 postprocess height 切片（Slice 1「height-only postprocess」net-negative 双证：R1968 -1 + R1980 -27）。代码 revert（R1636）。详见 [`evidence/r1980-...`](./evidence/r1980-extent-backfill-plus-sync-net-negative-sync-is-paint-only-2026-07-24.txt)。

---


> **📍 定位（R1966 session 1）**：R109 vertical（css-writing-modes 652/784 mismatch）经 R1963 重开→R1964 定位（float block 子 `<p>` height=inf）→R1965 height-set A/B couple-regress（5 证）→R1966 4-path 核查（无 incremental ZW-side lever，6 证）**conclusively 确认**：inf 是 vertical-IFC measurement 结构性表现，**incremental axis-swap patch 全 couple-regress**（block-height/float-height/container_width/line-height/emphasis + height-set + output-clamp + backfill + storage-gate 六角度）。唯一 forward = **vertical-native layout**（弃 axis-swap emulation，对 vertical 子树自建 layout）。本文档是 session 1 架构设计，供后续 session 按切片实施。**非 rally 单 session 可完成**（multi-week+），但架构设计本身是 durable forward（解阻塞未来 session，明确拦截点 / mixed-WM / native measurement / 切片顺序）。

> **📍 v0.1.1 addendum（R1968，2026-07-24）·Slice 1 empirical A/B REFUTES「postprocess-write 避 couple-regress」假设·第 7 证·设计收窄**：本设计 v0.1 §3/§4 推测方案 B 的 Slice 1（postprocess 写 height 不回喂 taffy）可能避开 R1965 的 taffy-feedback couple-regress。**R1968 实证证伪**：实现 Slice 1（`apply_inner` 加 content_height postprocess 写 LayoutBox，gate `ZW_VERTICAL_NATIVE_HEIGHT` default-off）+ A/B css-writing-modes 784 案 = **net -1 oracle-pass**（baseline 130/16.6% → treatment 129/16.5%；strict 68→66 / near 62→63 / mismatch 654→655），且 **引入新 top-fail**（block-flow-direction-srl-045 41.35% / vrl-005 41.35% / srl-051+vrl-011 33.42%，baseline 未在 worst-15）。★ **结论**：couple-regress **不来自 taffy cascade**（postprocess 无 mark_dirty），而来自 **height 值变化本身**——vertical 容器 height 改后，horizontal 父的兄弟定位（taffy layout 期用旧 Σ-height 算）与之不一致（TBD1 预测成立）。即「不回喂 taffy」**不足以**避 couple-regress，须 **full-native 子树 layout**（positions + sizes + 兄弟重定位 同步算）。★ **设计收窄**：Slice 1「height-only postprocess」**不可单切片落地**（net-negative），须与 **companion sibling-shift**（shift_siblings_after_ifc_grow 模式 R1492，scoped 到 vertical 容器在 horizontal 父中的兄弟）**捆绑**，或直接走 full-native（一次算完 positions+sizes 不依赖 taffy layout 期定位）。代码已 revert（违 R1636「不留二次证伪 dormant 死代码」），dormant gate 不保留。★ **第 7 证**：R1043/R1050/R1052/R1054（axis-swap 单层）+ R1965（taffy-feedback height-set）+ R1968（postprocess height-only）= incremental vertical patch 全 couple-regress，**含 postprocess 路径**（非仅 taffy-feedback）。强化本设计核心论点：vertical-native 须 **整体**（非切片蹭）。

---


---

## 0. 执行摘要

- **一句话目标**：对 `writing-mode:vertical-rl/lr` 子树，弃用 taffy axis-swap emulation，改用 ZW 自建 native layout 路径，在物理坐标一次性算出全部位置+尺寸（含 vertical IFC extent），消除 inf-height 与 4 层耦合。
- **本期（session 1）范围**：**仅架构设计**（本文档）。不改代码。明确拦截点、mixed-WM 递归、native measurement 来源、切片顺序、kill-switch 策略、open questions。
- **明确排除**：本期不实施任何代码切片；不修改 taffy-local 算法；不处理 horizontal-tb（字节等价零回归是硬约束）。
- **核心约束**：① horizontal-tb 字节等价（WM gate 隔离，零回归）；② 每切片 kill-switch + 全量 A/B 守 net≥0 + test-guard 防 hang；③ 不引入 R1895 类 measurement feedback loop（native 路径须一次性算完，不 mark_dirty 重跑 taffy）；④ vertical-native 仅作用于 vertical 子树，horizontal 子树继续走 taffy。
- **推荐方案**：**engine 层 post-taffy native re-layout pass**（方案 B，见 §3）—— generalize 现有 `vertical_block_flow`（只重定位 + 设 width）为完整 vertical-native pass（含 IFC extent 测量 + height 解析 + float/abspos 子树），写 LayoutBox 直传，不回喂 taffy。
- **首个落地步骤（session 2 Slice 1）**：扩 `vertical_block_flow::apply_vertical_block_flow` 增加 vertical 容器 content_height 解析（从 IFC extent / max child outer height），gate `ZW_VERTICAL_NATIVE_HEIGHT` default-off，A/B css-writing-modes 784 案守 net≥0，characterization test（R1964 `assert p.height.is_infinite()`）翻转为 success signal。

---

## 1. 背景：为何 axis-swap emulation 不可增量修复

### 1.1 现状架构（R1963 实证）

ZW 对 vertical-rl/lr 的支持是**纯 ZW converter 层 axis-swap emulation**，taffy 本身**零 vertical-mode awareness**：

- **输入侧**（`converter::apply_vertical_writing_mode`，converter/mod.rs:276-322）：对 taffy `Style` 全轴交换（size.width↔height / margin / padding / border / flex_direction Row↔Column / inset / min/max_size），在 `tree.rs:960` 建 taffy 树时应用。taffy 见到的是「水平模型」输入。
- **布局**：taffy 按物理 horizontal-tb 算（block 垂直堆叠 / flex Row 水平等），**不知 writing-mode**。grep `crates/taffy-local/src/compute/`（block/flex/grid/float）零 `writing_mode` 分支；唯一提及是 `geometry.rs:74-81` 注释「naively assuming Inline axis is Horizontal... will change if Taffy ever implements the writing_mode property」+ `test.rs:104` test-only `WritingMode` enum。
- **输出侧**（`engine::extract_layout`，engine.rs:1025）：reverse-swap（x↔y, width↔height, border/padding/margin 对应轴）还原视觉坐标。

这套 emulation 对**简单 case**（固定尺寸块、纯 block-flow 重排）工作——`vertical_block_flow.rs::apply_vertical_block_flow`（postprocess，R1545 net+1）已修纯 block 容器的子重定位（rl 右到左 / lr 左到右）+ 物理 width（block-size=Σ 子宽）。

### 1.2 结构性缺口：vertical-IFC measurement

emulation 对**含文本的 auto-size vertical 块**崩溃（R1964 characterization）：

- vertical-rl float 的 block 子 `<p>`（auto block-size + 文本内容）→ taffy 轴交换 block layout → **height=inf**（应≈262 文本 inline extent；width=19=line-height swap 正确）。
- 根因：taffy 对 axis-swapped `<p>` 的「物理 height」（= vertical inline extent = 文本长度方向）无法测量——vertical line-breaking 不在 taffy 能力内（taffy 是 physical-axis horizontal-tb），返回 unbounded 信号 inf。
- horizontal 对照：`<p>` height=19 finite（horizontal line-breaking taffy 可算）。

### 1.3 incremental 修复全 couple-regress（6 证）

R1043/R1050/R1052/R1054 + R1965 height-set + R1966 4-path 核查，六角度皆证 incremental axis-swap patch 不可 clean 修：

| 角度 | 证据 | 结果 |
|------|------|------|
| block-flow mirror（R1043） | postprocess 重定位 | net-negative |
| container_width=0（R1052） | R1099 α-1 WM-aware 已 land | 单层 net -26，须 4 层同修 |
| float-height / container block-size feedback（R1895） | taffy re-layout mark_dirty | **measurement loop hang** |
| spec-correct bundle（R1054） | vrl mirror + 全 vertical | net -28（vertical 须 near-perfection 才过 1%） |
| height-set（R1965） | `ZW_VERTICAL_BLOCK_FLOW_HEIGHT=1` | net +1 但 vlr-008 +12.86pp couple-regress |
| output-clamp 真实 extent（R1966） | 4-path 核查 | extent 未存（compute_final:941 bail）+ 取 extent 须重跑（R1895 hang）|

**结论**：axis-swap emulation 的 4 层（block-flow / inline-flow / line-height / emphasis）+ measurement feedback 耦合成系统，任一单层修皆破他层。须**整体替换为 vertical-native**（自建 vertical 子树 layout，不依赖 taffy feedback）才能解耦合。

---

## 2. 目标状态：Vertical-Native Subtree Layout

### 2.1 核心思想

对每个 `writing-mode ∈ {VerticalRl, VerticalLr}` 的子树根，**绕过 taffy axis-swap**，用 ZW 自建 layout 在物理坐标一次性算出子树全部位置 + 尺寸：

- **block-flow**：复用已验证的 `compute_vertical_block_flow`（R1541 V2/V3 chromium ground-truth：rl 右到左 / lr 左到右，同 y；content_width=Σ 子宽，content_height=max 子高）。
- **inline 内容（IFC extent）**：复用现有 `InlineFormattingContext`（R1099 α-1 已 WM-aware，`with_vertical` + container_width=content_height），算 vertical 文本的 inline extent（= 物理 height）。
- **一次性算完**：两趟（自底向上测子 extent → 自顶向下定位），**不 mark_dirty / 不重跑 taffy** → 避 R1895 hang。
- **写 LayoutBox 直传**：post-taffy pass，结果直接写 `LayoutBox.{x,y,width,height,content_*}`，paint 消费。

### 2.2 与现有 vertical_block_flow 的关系

`vertical_block_flow.rs` 已有 native block-flow 的**位置 + 物理 width** 计算（postprocess `apply_vertical_block_flow` + layout-time sizing `apply_vertical_block_flow_sizing`），但**不含**：
- IFC extent 测量（auto block-size vertical 块的 height）；
- float-context 块的 height 解析（gate 跳过 float）；
- abspos / table / flex/grid in vertical 的完整处理。

vertical-native = **generalize vertical_block_flow 为完整子树 layout**（补 IFC extent + float + abspos 等），非全新模块。

### 2.3 关键不变量

- **horizontal-tb 字节等价**：native pass 仅对 vertical 子树生效（`writing_mode.is_vertical_block_flow()` gate），horizontal 路径不进入 → 零回归（同 R1099 α-1 decoration-gate 模式）。
- **无 feedback loop**：native pass 一次性算完写 LayoutBox，不 set_style + mark_dirty 回喂 taffy（区别 R1895）。
- **kill-switch**：每个切片独立 env gate（`ZW_VERTICAL_NATIVE_*`），default-off 直至 A/B net≥0。

---

## 3. 设计选项：拦截点（Interception Point）

vertical-native 在管线何处拦截 taffy？三方案对比：

| 维度 | 方案 A：converter 层（不 axis-swap vertical 子树） | 方案 B：engine 层 post-taffy native re-layout（推荐 ⭐） | 方案 C：taffy-local vertical-mode 算法 patch |
|------|------|------|------|
| 实现复杂度 | 🔴 高（须建独立 native layout 树，绕 taffy 子树） | 🟡 中（generalize 现有 vertical_block_flow postprocess） | 🔴 极高（patch vendored 71-file fork 的 vertical 算法） |
| horizontal-tb 零回归 | 🟢 强（vertical 子树根本不进 taffy） | 🟢 强（WM gate，horizontal 不进 pass） | 🟡 中（taffy 内部分支，须隔离） |
| 避 measurement loop | 🟢 是（无 taffy feedback） | 🟢 是（post-taffy 写 LayoutBox，不回喂） | 🔴 否（仍 taffy 内部，R1895 hang 风险） |
| 复用现有代码 | 🟡 部分（IFC 可复用，block-flow 重写） | 🟢 强（vertical_block_flow + IFC + extract_layout 全复用） | 🔴 弱（taffy 算法重写） |
| 风险 / 回滚 | 🟡 中（converter 改动影响面广） | 🟢 低（postprocess pass，kill-switch 易回退） | 🔴 高（vendored fork，升级冲突） |
| 推荐度 | ⭐⭐ | ⭐⭐⭐ | ⭐ |

**最终选择：方案 B（engine 层 post-taffy native re-layout pass）**

**理由**：
1. **复用最大化**：`vertical_block_flow`（R1545 已 land net+1，V2/V3 ground-truth 验证）已解决 vertical block-flow 位置 + 物理 width；方案 B 仅 generalize 它补 height/IFC/float/abspos，非从零建。
2. **避 R1895 hang**：post-taffy 写 LayoutBox 不回喂 taffy，根本无 measurement loop 可能（R1895 的 hang 来自 mark_dirty + re-run）。
3. **kill-switch 易回退**：postprocess pass 加 env gate default-off，A/B 失败即关，零风险（同 R1958/R1959 模式）。
4. **horizontal 零回归**：WM gate 隔离，horizontal 子树字节不进（同 R1099 α-1）。
5. **方案 A 否决**：converter 层绕 taffy 须重建子树 layout infra，与现有 extract_layout reverse-swap 双轨维护成本高，且 vertical 子树仍需在 horizontal 父中定位（taffy 仍需知 vertical 子树的 outer size）→ 实际仍须 post-measurement，退化向方案 B。
6. **方案 C 否决**：R1962/R1963 已证 taffy-local patch = dependency-level rework（vendored 71-file fork，升级冲突，R1895 hang 风险），且 R1963 证 inf 真因在 ZW emulation 层非 taffy 内部。

### 3.1 方案 B 的 mixed-WM 递归

vertical 子树可含 orthogonal 流（vertical 容器内 horizontal 子，或反之）。native pass 须递归 WM 上下文：

- **parent_wm 跟踪**：自顶向下传 `parent_wm`（同 `vertical_block_child_indices` 现有模式）。
- **vertical 容器的 horizontal 子**（orthogonal）：子仍可走 taffy（horizontal），但须在 vertical 父的 block-flow 中定位（vertical 容器 content_width=Σ 子宽 含 horizontal 子的 width）。这是 R1965 height-set 对「orthogonal inline-block 回归 +6~8pp」的子集——native pass 须显式处理（slice 3）。
- **vertical-in-vertical**：递归 native（vertical 容器内 vertical 子）。

---

## 4. 实施切片（Phased Slices，每片 kill-switch + A/B net≥0）

> **原则**：每片独立 env gate default-off；A/B css-writing-modes 784 案 + 全量 reftest-oracle 守 net≥0 + 零大回归；characterization test（R1964）作 success signal；test-guard 包裹防 hang。失败即关，dormant 保留。

### Slice 1（session 2）：vertical 容器 content_height 解析（pure-block）— ⚠ R1968 height-only 实证 net-negative，须捆绑 sibling-shift

- **范围**：vertical 容器（≥2 block in-flow 子，非 float/abspos/table/flex/grid/multicol）的 **content_height**（物理 block-size 方向，= max child outer height）解析。当前 `apply_vertical_block_flow` 只设 width（Σ 子宽），height 留 taffy（vertical-stack Σ 子高，错）。
- **改动**：`vertical_block_flow.rs::apply_inner` 增 `content_height = max(child.height + margin_tb)` + frame → 写 `box.height / content_height`。区别 R1965 height-set（layout-time sizing 回喂 taffy，couple-regress）：本片是 **postprocess 写 LayoutBox，不回喂 taffy** → 无 loop。
- **gate**：`ZW_VERTICAL_NATIVE_HEIGHT` default-off。
- **⚠ R1968 A/B 实证（height-only）= net -1 oracle-pass**（130→129；strict 68→66 / near 62→63 / mismatch 654→655）+ 引入新 top-fail（srl-045/vrl-005 41.35%）。**height-only postprocess couple-regress**（非 taffy cascade，是 height 值变化致 horizontal 父兄弟定位不一致，TBD1 成立）。**结论**：Slice 1 **不可 height-only 单切片**，须 **捆绑 companion sibling-shift**（shift_siblings_after_ifc_grow 模式 R1492，scoped 到 vertical 容器在 horizontal 父中的兄弟）或走 full-native。代码已 revert，dormant gate 不保留。
- **修订 A/B**（session 2 须重做）：height postprocess + companion sibling-shift **捆绑** A/B，gate 仍 default-off，守 net≥0。

### Slice 2（session 3）：vertical 块 IFC extent（auto block-size + 文本）

- **范围**：含文本的 vertical 块（如 float 的 `<p>` 子）的 height = vertical IFC inline extent。这是 R1964 inf 的直接 fix。
- **改动**：native pass 对 vertical 文本块调用 IFC（`with_vertical`，container_width=content_height）算 inline extent → 写 height。**关键**：compute_final:941 gate 当前对非 Ahem 块 bail 不存 inline_layout——native pass 须**自己跑 IFC 取 extent**（不依赖 stored inline_layout），一次性写 height 不回喂。
- **gate**：`ZW_VERTICAL_NATIVE_IFC_EXTENT` default-off。
- **A/B**：R1964 test 翻转（inf→finite≈262）+ css-writing-modes。
- **风险**：IFC container_width 依赖 content_height（R1099 α-1），而 content_height 可能 itself inf → 须先 resolve content_height（slice 1）或用 unbounded（单列全文本，extent=文本长度，正确）。须测 container_width=inf 时 IFC 行为。

### Slice 3（session 4）：mixed-WM orthogonal 子

- **范围**：vertical 容器的 horizontal（orthogonal）子的定位 + sizing。R1965 height-set 对 orthogonal inline-block 回归 +6~8pp 的直接处理。
- **改动**：native pass 递归 parent_wm，orthogonal 子走 taffy horizontal 后在 vertical block-flow 中定位。
- **gate**：`ZW_VERTICAL_NATIVE_ORTHO` default-off。

### Slice 4（session 5+）：float / abspos / table in vertical

- **范围**：vertical-mode float 定位（R1964 float inf 的完整 fix）、abspos in vertical、table-vertical（R1844 table-vertical 协调）。
- **改动**：native pass 扩 float / abspos / table 子树处理。
- **gate**：各 `ZW_VERTICAL_NATIVE_FLOAT/ABSPOS/TABLE` default-off。

### Slice 5（session 6+）：flex/grid in vertical

- **范围**：vertical 容器是 flex/grid（flex_direction 已 axis-swap）的 native 处理。taffy 对 axis-swapped flex 可部分工作，须 A/B 定边界。
- **风险**：高（flex/grid 算法复杂），可能须保留 taffy + 仅修 measurement。

---

## 5. Open Questions / TBD

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|----|------|--------|----------|--------|
| TBD-1 | vertical 容器在 horizontal 父中的 outer-size 反馈：native 改 vertical 容器 height 后，horizontal 父的 vertical-stack 兄弟须重定位——postprocess shift（R1492 模式）是否充分，还是须 two-pass？ | 阻塞（slice 1） | 须 A/B 实测 vertical 容器 height 变化对 horizontal 父兄弟的影响范围 | slice 1 A/B 时 LAYOUT_DUMP 定位 |
| TBD-2 | IFC container_width=inf（未 resolve content_height）时 vertical IFC extent 是否正确（单列全文本 extent=文本长度）？ | 阻塞（slice 2） | 须实证 IFC 在 unbounded container 下的 line 产出 | slice 2 前 probe |
| TBD-3 | vertical 容器是 shrink-to-fit 候选（inline-block / float / abspos）时，native extent 如何参与 horizontal 父的 shrink-to-fit measurement（R1017/R1518 谱系）？ | 重要（slice 4） | shrink-to-fit 两趟测量与 native pass 的交互 | slice 4 设计 |
| TBD-4 | sideways-rl/lr（R1785 已规范化为 VerticalRl）是否完全等价，还是有 mirror 差异？ | 重要 | R1785 实证规范化，但须确认 native pass 不破坏 | slice 1 A/B 含 sideways case |
| TBD-5 | vertical IFC 的 text-decoration / emphasis 坐标（R1099 α-3 未实施，decoration-gate 回避）——native pass 是否须同步处理？ | 可选 | α-3 未实施，当前 decoration-gate 回避 | slice 2+ 视回归决定 |
| TBD-6 | 性能：native pass 对每个 vertical 子树跑 IFC，是否有重复测量开销（compute_final 已跑一次）？ | 可选 | 须 profile | slice 2 后测 |

---

## 6. 验证策略

- **characterization test**（R1964 `r1964_vertical_float_inf_diag`）：slice 2 success signal = `assert p.height.is_infinite()` 翻转为 `assert p.height finite ≈ 文本 extent`。
- **A/B**：每片 `make reftest-oracle DIR=css-writing-modes`（784 案）+ 全量 `make reftest-oracle`（守 latent regression）+ `make product-smoke`（welcome/morning/wintertc）。
- **test-guard**：所有 A/B 经 `make reftest`/`make test` 包裹（防 R1895 类 hang 吃内存）。
- **net≥0 硬门**：near-pass/mismatch 计数 net≥0 且无 >1pp 大回归；否则该片 default-off，dormant 保留，记 evidence。

---

## 7. 与前轮裁决的关系

- **R1963**（taffy 零 vertical awareness，ZW axis-swap emulation）→ 本设计 §1.1 现状架构的事实基础。
- **R1964**（inf 源头 = float block 子 `<p>`）→ slice 2 的直接 fix target + characterization test。
- **R1965**（height-set couple-regress，5 证 coupled system）→ 本设计「弃 incremental axis-swap、整体 native」的触发依据。
- **R1966**（4-path 无 incremental ZW-side lever，6 证）→ 本设计「post-taffy native re-layout 避 feedback loop」的论证依据（方案 B vs C）。
- **R1099 α-1**（container_width WM-aware 已 land）→ native pass 复用 IFC 的基础。
- **R1541/R1544/R1545**（vertical_block_flow native block-flow V2/V3 ground-truth + net+1）→ 方案 B 复用最大化 + generalization 起点。

**不在本设计范围**（ruled out by 前轮）：
- taffy-local vertical-mode 算法 patch（方案 C，R1962/R1963 ruled out）。
- extract_layout output-clamp（R1962/R1966 ruled out）。
- incremental axis-swap 单层 patch（R1043/R1050/R1052/R1054/R1965 六证 ruled out）。

---

## 8. 修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v0.1 | 2026-07-24（R1966） | session 1 架构草案：现状/耦合证据、目标状态、三方案对比（选方案 B post-taffy native re-layout）、5 切片、6 TBD、验证策略。implementation 待后续 session。 |
| v0.1.1 | 2026-07-24（R1968） | Slice 1 empirical A/B（height-only postprocess）= net -1 oracle-pass + 新 top-fail → refutes「postprocess-write 避 couple-regress」假设（第 7 证）。Slice 1 收窄为须捆绑 companion sibling-shift。代码 revert（不留二次证伪 dormant 死代码，R1636 先例）。 |
| v0.1.2 | 2026-07-24（R1969） | timing-impasse 架构发现：vertical container 正确 height 须在 taffy 对 horizontal 父兄弟定位之前已知。三 timing 全分析（postprocess=太晚 gap 歧义 / mark_dirty 重跑=cascade couples / **pre-taffy injection=未试新方向，最干净**）。下轮 concrete code slice = pre-taffy vertical-height injection（converter 层，scope pure-block + definite-size 子）。 |
| v0.1.3 | 2026-07-24（R1970） | pre-taffy injection empirical A/B = **0 yield**（baseline==treatment 133/784）。gate「全 definite-Px-height 子」在 corpus 匹配 0 案（主导失败用文本 auto-height `<p>` 子）。**重大收窄**：R109 vertical 失败是 **text-content IFC measurement 主导**（非 timing/layout）；三 timing 全 irrelevant；真核心 = IFC circular-dependency breaking（measure extent 须 container_width=content_height=extent）。timing 三角度 definitive closed（8 证）。代码 revert。 |
| v0.1.4 | 2026-07-24（R1980） | extent-backfill（R1977 值）+ R1978 sync 综合 broad A/B = **NET -27**（103/784）；inf-only+sync = net +1 noise。**R1978 sync = necessary-but-NOT-sufficient**（paint-layer-only，layout-cascade 主导触不到）。vertical height postprocess 第 5 证非 lever。**强化方案 B**：vertical-native 须 layout 层整体（post-taffy full re-layout），非 postprocess height 切片（Slice 1 height-only net-negative 双证 R1968 -1 + R1980 -27）。代码 revert。 |


# R2086-R2095 — reftest autonomous forward 穷尽调查（plateau exhaustion 详记）

> 从 master.md 顶部速览 line 5 迁出（R2095 瘦身）。逐轮结论 100% 保留，供回溯。
> 结构化结论仍在 master.md 顶部速览；本文件为详细 round-by-round 记录。

## 综合裁决

R2086-R2095（10 轮，2026-07-25 ~ 2026-07-27）跨全角度调查 reftest autonomous clean-lever
forward，唯一 land = **R2091 grid-percentage-quirk bridge**（canvas geometry 400→10，font-wall-
blocked yield，zero oracle PASS 但 correctness + latent value）。其余 9 轮全 negative/confirmatory。
reftest ~57% = mature plateau（用户 2026-07-17 接受）；font-stack = 真 unlock 但 user-gated
暂不优先（R2025 踩坑勿推）；主线转产品层，reftest 降为 plateau-guard mode。

## 逐轮详记

### R2086/R2087（2026-07-25）oracle subdir 扫描 entangled
css-flexbox/tables（R2086）+ CSS2/backgrounds（R2087）oracle 扫描，top 候选全 entangled
（aspect-ratio-with-children / JS / font-wall / structural）。零 clean lever。

### R2088（2026-07-27）10-angle plateau-confirmation
续扫剩余子目录（bidi-text/linebox/box-display/borders）+ §10.3.8 abspos-replaced 簇 + 6 个
correctness-review angle（abspos recenter inset / integer-parse overflow / per-layer
bg-attachment / collapsed-border / R1293 flex relpos% gate / R1398 深层 abspos）全 entangled
或零 driving test。详见 [`evidence/r2088-plateau-confirmation-10-angles-2026-07-27.txt`](../evidence/r2088-plateau-confirmation-10-angles-2026-07-27.txt)。

### R2089（2026-07-27）grid-focused 确认
R1291 grid alignment 默认值谱系（justify-content/justify-items/justify-self/align-items）经
taffy grid/compute/alignment.rs 源证全 correct（无对称 lever）。css-grid fresh oracle
28/49=57%（R1762 后 stable，top-15 全 vertical-mode/JS/native-widget/print/aspect-ratio
entangled）。唯一非 entangled 候选 grid-item-percentage-quirk-001 根因定位（canvas h=400
vs intrinsic），留 focused probe。详见 [`evidence/r2089-grid-confirm-r1291-correct-percentage-quirk-2026-07-27.txt`](../evidence/r2089-grid-confirm-r1291-correct-percentage-quirk-2026-07-27.txt)。

### R2090（2026-07-27）grid-percentage-quirk 深度诊断 + narrow fix REVERTED
专用 probe test 5 变体锁根因 = mode-independent + replaced-specific（canvas grid item
Percentage height 在 indefinite grid + aspect_ratio + justify-stretch 下 double-resolve →
400×400）。尝试 R2016 扩 gate（replaced grid item + Percentage + cb_definite None），
canvas 400→100 仍 FAIL（应 ~10）——else 分支依赖 img_intrinsic_sizes 不含 canvas（canvas
HTML-attr intrinsic 在 tree.rs）。真修复须 bridge canvas HTML-attr intrinsic × grid context
= invasive，+1 niche ROI 不足 → REVERT。详见 [`evidence/r2090-grid-percentage-quirk-deepdive-reverted-2026-07-27.txt`](../evidence/r2090-grid-percentage-quirk-deepdive-reverted-2026-07-27.txt)。

### R2091（2026-07-27）★ grid-percentage-quirk bridge LANDED（唯一 land）
实施 R2090 bridge：新 `gather_replaced_html_attr_intrinsic`（tree build 后遍历 LayoutBox，
读 canvas/embed/object/applet DOM HTML width/height，排除 img）注入 R2016 专用
`intrinsic_for_r695`（不污染 tree.rs 原 map）；R2016 gate 扩「HTML-attr-intrinsic 替换元素
（gather ids）+ grid/flex item + Percentage height + indefinite CB」交 else 分支（→auto+
intrinsic），narrow 到 gather ids 排除 img（避 grid-in-table-cell-with-img definite-track
回归）。kill-switch `ZW_GRID_REPLACED_PCT_INDEFINITE=0`。canvas（grid item height:200%
indefinite grid）h **400→~10**（单测证）；oracle A/B **零回归**（css-grid 28/49、css-flexbox
315、CSS2/positioning 376 三目录 baseline==fix）；grid-item-percentage-quirk-001 oracle
**33.21%→1.06%**（32pp canvas geometry diff 消除，残余 1.06% = 纯 font-wall `<p>` 文本 floor，
与 quirk-002 同）→ 零 oracle PASS yield（font-wall block）但 geometry 正确 + latent value
（font-stack rebuild 后 quirk-001 将 PASS）。land 为 correctness fix。详见 [`evidence/r2091-grid-percentage-quirk-bridge-landed-2026-07-27.txt`](../evidence/r2091-grid-percentage-quirk-bridge-landed-2026-07-27.txt)。

### R2092（2026-07-27）definite-CB grid + replaced-percentage negative
R2091 测试期间发现的 canvas h=3136（definite-height grid + canvas + height:200%）pre-existing
bug 调查：grep 全 css-grid 零 canvas definite-CB driving test；img 同谱系 driving test
（replaced-element-percentage-height-in-grid-nested-in-flex-001/-002）oracle 0.53% PASS
（img decoded intrinsic + taffy 正确）。canvas definite-CB 案 h=3136 真 bug 但零 reftest
footprint，且 R2091 intrinsic-injection 修法不适用（definite CB 百分比应解析为 200 非
intrinsic 10），按 code-guidelines 不修 defer。详见 [`evidence/r2092-definite-cb-grid-replaced-percent-negative-2026-07-27.txt`](../evidence/r2092-definite-cb-grid-replaced-percent-negative-2026-07-27.txt)。

### R2093（2026-07-27）R1870 multicol audit-clean + JS-DOM-mutation reflow negative
R1870 multicol-blockfrag（try_layout_single_child_block_frag + assign_children_to_columns
_with_breaking break-before/after forced-break 分配）audit = clean（driving test multicol-
fill-auto-004 PASS，css-multicol 190/452=42% top-15 全 spanner/Phase-2/JS/print/nested
entangled）。JS-DOM-mutation reflow gap（R888 谱系）= negative：reftest harness 已经
`apply_scripted_dom_mutations`（reftest_scripts.rs）执行 `<script>` + `<body onload>` →
V8 DomMutation 记录（SetStyle 捕获 el.style.x=y）→ apply_mutations_to_html 回写 mutated
HTML → 重渲染。R888「不反映 DOM 变更到 layout」已过时。JS-dynamic 残余失败
（dynamic-relayout-005/006 11.71% = content-visibility feature 未实现；relpos-004/-005
4.79% = mutation 已应用 + font-wall）皆 feature-specific / font-wall，非系统性 reflow 缺口。

### R2094（2026-07-27）content-visibility negative + legacy plateau-guard 二次确认
content-visibility footprint 仅 2 案全 corpus（ZW 不 parse），driving test 实为 abspos-
CB-inline 结构性（非 cv feature），+2 max ROI 不足不实施。`make product-smoke-legacy`
51 fixture 50/51 struct PASS（唯一 FAIL = 37-form-controls Phase A 非回归；46-frameset
100% = known baseline probe frameset 帧网格 unsupported），二次确认 R2077 legacy/UA 表面
无新 struct issue。详见 [`evidence/r2094-content-visibility-negative-legacy-plateau-confirm-2026-07-27.txt`](../evidence/r2094-content-visibility-negative-legacy-plateau-confirm-2026-07-27.txt)。

### R2095（2026-07-27）dormant-infra hunt（R1958 pattern）false positive + master.md 瘦身
memory「转其他子系统 dormant-infra hunt」（R1958 collapsed_border_outer_edge dormant flag
→ +6）。跨 layout-engine/style-system/paint 找同类 dormant computed-but-unused field。
候选 InlineLayoutLine.ascent/descent = **false positive**：非 dormant（inline_finalization
读 ascent 算 strut/half_leading；baseline_y = max_ascent → paint 读 baseline_y 间接消费
ascent）+ Phase A font-metric 谱系（八证 net-negative）。R1958 pattern 无第二实例。
本轮同时做 master.md 瘦身（line 5 = 8034 chars → 结构化结论 + 本 archive 指针）。
详见 [`evidence/r2095-dormant-infra-hunt-false-positive-2026-07-27.txt`](../evidence/r2095-dormant-infra-hunt-false-positive-2026-07-27.txt)。


### R2096（2026-07-27）CSS contain:layout/size BFC dormancy negative（zero footprint）
CSS `contain`（35 案 corpus，23 multicol）：ZW parse + store ContainComputedValue，paint
消费 contain:paint 做 clip（mod.rs:153）+ debug indicator；**layout 不消费 contain**
（establishes_bfc / is_flow_root 无 contain 检查）。`contain:layout/paint/strict/content`
per CSS Contain §3 应建立 BFC（等价 flow-root，隔离 margin 折叠 + 含浮动）。实施 minimal
wire（LayoutBox 加 establishes_containment_bfc flag + engine build 期从 contain:layout/
paint/strict/content/Custom(layout|paint) 设 + establishes_bfc 检查），oracle A/B：css-
multicol 190/452=42.0% **baseline==fix**，css-position 66/97=68.0% **baseline==fix**
（两最大 contain 目录零 yield 零回归，无 observable geometry 变化）。★ 与 R2091 不同：
R2091 修了真 geometry（canvas 400→10），本 wire **零 geometry 变化**（contain-BFC 在
corpus 无 driving test——contain 案用 size containment / abspos-CB / paint-clip 等其他
facet，非 BFC）。按 code-guidelines「不为不可能发生的场景编写错误处理」+ 方法论「driving
test required」**REVERT**（spec-correct 但零 reftest footprint，defer 到有 driving test）。

## 调查覆盖矩阵（全 exhausted）

| 角度 | 轮次 | 结论 |
|------|------|------|
| reftest 全子目录扫描 | R2086-R2094 | 全 plateau/entangled |
| 近期 code correctness audit | R2088-R2093 | 全 correct/negative |
| grid+replaced+percentage | R2089-R2092 | R2091 land（font-wall-blocked）+ R2092 neg |
| JS-DOM-mutation reflow | R2093 | harness 已应用（R888 过时） |
| content-visibility feature | R2094 | +2 max ROI 不足 |
| legacy/UA 产品表面 | R2077/R2094 | 二次确认 plateau |
| dormant-infra R1958 pattern | R2095 | false positive（Phase A 谱系） |
| CSS contain:layout/size BFC dormancy | R2096 | zero footprint（spec-correct REVERT） |
| CSS var() resolution | R2097 | tiny footprint（6 案）+ ZW 已支持（resolve_var_in_cascaded） |

## 已 ruled out（勿以单 session 重试，详见 master.md「已 ruled out」节）

near-pass font-wall / Phase A broad IFC 统一（R2075 REFUTED）/ Phase A narrow font-metric
（八证 net-negative）/ font-stack rebuild（user-gated 勿推，R2025 踩坑）/ semi-replaced
form-control（37-form-controls Phase A 阻塞）/ content-visibility（+2 ROI 不足）/ canvas
definite-CB grid+percentage（零 driving test）/ dormant-infra（R1958 无第二实例）。

## forward

plateau-guard mode（周期回归 + opportunistic review of NEW land）；next 真 unlock =
font-stack（user-gated 暂不优先）或 Phase A（broad refuted / narrow net-negative）或产品层转进。

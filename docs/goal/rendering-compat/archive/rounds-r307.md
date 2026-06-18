# R307 — DC-14 strict 全量 near-pass frontier 实证（归档自 master.md）

> 归档说明：本文件为 master.md「最近轮次详细记录」中 R307 的逐轮详细记录，于 doc-maintenance 轮（2026-06-19）归档——并行 agent 提交 R327（Phase A Gate 2 多行放宽证伪）后，master.md 最近轮次窗口达 21 轮（R307–R327），R307 作为第 21 轮迁出，窗口收窄为最近 20 轮（R308–R327）。R307 的核心结论（near-pass <0.2% 26 案逐聚类根因分类，全部落入已知结构性墙或字体噪声，near-pass clean-win 杠杆经实证关闭）仍以浓缩形式保留在 master.md「综合裁决」杠杆穷尽表（near-pass clean-win frontier | R307）。本归档仅为可追溯性保留，archive 区不修改。

---

### R307 — DC-14 strict 全量 near-pass frontier 实证：clean-win 杠杆关闭（read-only 实证 + evidence，基线 loose 438/490 / strict 296/490 持平）

**承接**：R306 Phase 0 探针证伪 Phase A 几何基线方向后，转 DC-14 优先级队列明确标的「攻 near-pass CSS2 前 20 个 clean win 候选用 STRICT env 度量增量」（R280 phased 第二步；R287 已落地 `ZERO_REFTEST_STRICT` env + 三态 blast radius：self@strict 真通过 296(60.4%)/近似 145(29.6%)/失败 49(注:本轮实测 194 fail 含近似)/）。本轮把 R280「145 near-pass / 101 ≤1%」的**计数乐观**做**逐用例根因分类**实证。

**实证方法（test-guard 包裹，合规 run-rules）**：`ZERO_REFTEST_STRICT=1 ./target/test-guard -- cargo run --bin zero-wpt-runner -- reftest-upstream` 全量 490。复现 R287 strict **296/490 (60.4%) / 194 fail**（基线一致）。194 失败按 diff% 升序持久化到 `evidence/r307-strict-nearpass-frontier-2026-06-19.txt`。

**Diff-band 直方图（194 strict 失败）**：<0.2%: 26 / 0.2-0.5%: 18 / 0.5-1%: 53 / 1-2%: 22 / >2%: 75。

**Near-pass frontier（<0.2%，26 案）逐聚类根因分类——全部落入已知结构性墙或字体噪声，零独立 clean win**：
- **css-multicol baseline-export 聚类**（baseline-000/003/004/005/006，~0.12-0.14%，5 案）：multicol baseline，R235/R266 已证 field-fill 净 0、需 pre-pass 估测（结构性多轮）。
- **css-multicol breaking/fill/balance**（broken-column-rule-1、float-with-line-after-spanner、multicol-fill-001、balance-grid-container、multicol-breaking-nobackground-000，~0.10-0.17%）：R131 column-aware IFC 碎片化墙。
- **css-tables collapsed-border/visibility-collapse/display-contents**（collapsed-border-partial-invalidation-003、visibility-collapse-rowspan-005/colspan-003、display-contents-001/003，~0.14-0.15%，6 案）：table 子系统结构性（R177b/R292 谱系残余）。
- **css-position hypothetical-box-scroll**（parent/viewport，0.12%，2 案）：abspos hypothetical box，结构性。
- **css-flexbox baseline/column-row-gap**（flexbox-baseline-align-self-baseline-horiz-001、flexbox-column-row-gap-002，0.14/0.25%）：flexbox baseline 合成，结构性（R295 wrap-reverse 谱系）。
- **CSS2 color-applies-to-001/005 + float-nowrap-5**（0.12%）：text glyph subpixel 定位（table-cell vs block div 渲染 "Filler Text" 的亚像素差），字体/布局噪声。
- **CSS2 ifc-001（0.12%）深挖**：LAYOUT_DUMP 实测 TEST div1 h=21.2 vs REF div h=22.0——inline 元素包裹文本（3×`<div display:inline>`）vs 直接文本的 **行盒高度差 0.8px**，即 Phase A 墙③（v_offset/baseline 语义分歧，R206 broad 翻 FAIL 直接因）。结构性强耦合，非单点。
- **css-grid stretch-grid-item-text-input-overflow（0.12%）**：text-input 原生 widget（R202 表单控件特性缺口）。

**裁决：near-pass frontier 是结构性 plateau 的「拖尾边缘」，非 clean-win 源**。R280「101 near-pass ≤1%」是计数乐观，逐用例分类后**零独立 clean win**——全部映射到 Phase A 墙③ / multicol 墙②+baseline / table / flexbox-baseline / 字体噪声 / 表单控件特性缺口。**near-pass clean-win 杠杆经实证关闭**。

**对优先级队列影响**：DC-14 phased 第二步（near-pass 攻坚）实证为死路，从队列移除。剩余真实 forward motion 杠杆收敛为**纯结构性多轮**：① Phase A IFC 统一（墙② multicol + 墙③ baseline，spec-rfc v1.2 已修订方向）；② multicol column-aware IFC 碎片化（R131，最大单聚类——baseline-export 5 + breaking/fill 5 = 10 案）；③ DC-9 blend_mode backdrop（独立能力，低 reftest 覆盖）；④ DC-13 产品 smoke 残余（item-tag R109 + fontdue CJK 度量）。**这些均非单会话 clean win，需 spec-rfc 多轮或特性实现**。

**本轮为 read-only 实证 + evidence 持久化**：零代码变更（`git diff -- '*.rs'` 空）；新增 `evidence/r307-strict-nearpass-frontier-2026-06-19.txt`（70 行，194 失败升序 + 直方图 + 26 案根因分类）。复现 R287 strict 基线 296/490 一致。基线 loose 438/490 / strict 296/490 / chromium-Oracle ~35.6% 持平。next = 启动 multicol column-aware IFC 碎片化（R131，最大 near-pass 聚类 10 案的根因域）的 spec-rfc 设计，或 DC-9 blend_mode 独立特性。

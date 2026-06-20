# 渲染兼容性目标 — 运行时控制面板

**最后更新**: 2026-06-22（**R399** doc-maintenance 治理：归档最后一块 stale 节 `IFC 之外的其他卡点 #2–#9`（R68 时代前置 plateau 框架）→ [`archive/r68-other-blockers-framework.md`](./archive/r68-other-blockers-framework.md)，**保留 section 锚点 + 逐卡点一行摘要 stub** 使 `multicol-phase2-unified-column-flow-spec.md` 的「卡点 #2/#4」live 引用仍可解析（无 dangling pointer）。master.md 634→508 行。**R397–R399 三轮归档完成**：master.md 803→508 行（−295 行 / ~−21KB），4 块 R68 时代 stale 节全部迁出 archive/（内容 100% 保留 + 指针回指）。零代码变更（read-only；并行 agent 仍 mid-work 未提交，HEAD `4c5edbe` 无新 commit）。｜**R395–R398 摘要**：R395 锁定前向方向=DC-14 分母去子集化（当前 N_imported 仅上游 ~5-6%，子集口径非达标证据，详 [`evidence/r394-dc14-denominator-gap-2026-06-22.txt`](./evidence/r394-dc14-denominator-gap-2026-06-22.txt)；并行 agent 已建 `discover-reftests-authoritative.py` + 导入 css-grid 全量执行中）；R396 抽样 2 grid 缺口簇→均映射 ruled-out 多会话架构族，非单会话 clean win；R397/R398 归档 R335/R336 + M6 基线 + IFC 技术参考。结构化结论见下方「综合裁决」表，R381–R393 verbose 详记见 [`archive/rounds-r381-r393-header.md`](./archive/rounds-r381-r393-header.md)。）

**doc-maintenance（2026-06-20 verify 轮）**：plateau 结论 read-only 复核成立（R354 fresh baseline 439/490 零漂移、clean-win 面 R351 后穷尽），无需新调整方向——现有「综合裁决 + 下一步」即当前结论。文档治理两项：① 将「技术决策记录」表中 **R118–R227 逐轮历史条目**（50 行，2026-06-14~17，远超最近 20 轮窗口，主体已在 rounds-r23-r139 / rounds-r142-r302 归档）迁出至 [`archive/tech-decisions-r118-r227.md`](./archive/tech-decisions-r118-r227.md)（50 行 → 1 指针行，master.md 833→786 行）；② 纠正「最近轮次详细记录」窗口标注（R335–R336 为最后两轮全文详记，R337–R354 为 plateau 复核/治理轮，精简结论见上方「综合裁决」表）。本轮零代码变更（并行 agent 正在 layout-engine 开发，未触碰）。

**前轮 R348**：fresh chromium-Oracle 复测确证 plateau 稳定（R324/R325/R326 零移动 polluted case chr diff；污染 48.0% vs R311 48.2%，逐 case 逐 chr% 稳定）。再前 R347 完成全仓库 2000 行达标（reftest.rs 拆出 resources.rs）。

**前轮 R345/R346**：R345 `paint/tests/visual.rs` 2056→1790（resize/scroll 测试尾）；R346 `inline/tests/advanced.rs` 2281→1948（float/tab 测试尾）。纯测试移动零回归。再前 R343/R344 完成生产源码 <2000（app_render.rs / gpu/renderer/mod.rs）。

**前轮 R342c**：2000 行规则收尾——`table.rs` 2694→1973 行拆分，抽出 `table_borders.rs`（740，resolve_collapsed_borders + BorderSource/resolve_border/边框颜色读取，CSS §17.6.2 collapsed 边框冲突解析集群）。**纯移动零行为变更**：reftest-upstream 438/490 字节级一致、**css-tables 51/55 不变**、888 layout 测试过、clippy 干净。详见 [`evidence/r342c-table-borders-split-2026-06-19.txt`](./evidence/r342c-table-borders-split-2026-06-19.txt)。

**当前活跃里程碑**: M10 — 上游 WPT 真实 Reftest 通过率。**渲染侧**：结构性 plateau（单会话杠杆已穷尽，子集范围内）。**分母侧（DC-14 门禁，R394 发现）**：当前 N_imported 仅上游 ~5-6%，去子集化是当前最高 ROI 前向路径（并行 agent 正执行 Phase 2：css-grid 全量导入 in-flight + `discover-reftests-authoritative.py`）。/ DC-13 产品 smoke（证据已持久化 `evidence/product-static/`，残余为文本度量结构性）

**基线（R323 复验；strict post-R326 再复验仍零漂移，见 [`evidence/strict-baseline-reverify-post-r326-2026-06-19.txt`](./evidence/strict-baseline-reverify-post-r326-2026-06-19.txt)；R388 oracle 修复后 chromium-Oracle 上修）**：
- self-source loose **443/490 (90.4%)** @ 默认 1%/5% 容差
- self-source strict **295/490 (60.2%)** @ 锁定 0.1%/0.5%（DC-14 真通过口径）
- chromium-Oracle 广义一致 **200/475 (42.1%)** @ chr<1%（R391 锁定诚实基线：R388 报的 205/43.2% 含 R389 才暴露的 5 假一致 flexbox blank-blank，已纠正）；严格 self-pass&chr<1% **177/475 (37.3%)**；污染率 46.5%。**R388/R389/R390 后度量可信**（pre-R388 ~35.8% 被 108 损坏 Ahem oracle 系统性压低，已修）。**⚠️ R394 关键：所有这些数字基于 ~5-6% 子集分母（503/上游~8000-10000），非全量真通过率，不构成 DC-14 达标证据**
- 产品 smoke：welcome **16.98%**（product-smoke 阈值0，与 morning/wintertc 同口径；R371 的 14.68% 是 cross-validate chan>5 的宽松口径，不可比，已纠正）/ wintertc ~13.6% / morning-work 800×600 **16.41%**（R373 inline+bg shrink 后，R175 的 28.72%→16.41%；fullpage 48.65% 是更高视口口径）

**字体攻坚结论（2026-06-17 AA 基准）**：fontdue **Regular** 与 chromium 光栅化基本一致（W 0.1% / i 3.0%），**非渲染差异来源**；welcome 26% / Oracle 污染大头是**布局/度量**（line-height / R109 inline→block / 多行结构）。fontdue **Bold** 变体比 chromium 过墨 ~15%（R229b net-negative 已回退）。**字体攻坚停止，转布局/度量**——advance-width(R225/R320)、font-weight -Bold(R229b)、AA 噪声(R174) 三谱系均实证为死路，勿再投入。

> **结构化 plateau 结论见下方「综合裁决」节**（R305–R354 杠杆穷尽表 + 4 条多会话路径 + 需用户决策卡点）。逐轮详细记录见文末「最近轮次详细记录」（R335–R336 全文；R337–R354 见「综合裁决」）；更早轮次已归档：R314–R334 → [`archive/rounds-r314-r334.md`](./archive/rounds-r314-r334.md)、R307–R313 → 各单轮归档（[`rounds-r307.md`](./archive/rounds-r307.md) … [`rounds-r313.md`](./archive/rounds-r313.md)）、R305–R306 → [`archive/rounds-r305-r306.md`](./archive/rounds-r305-r306.md)、R304 → [`archive/r304-taffy-upgrade-deferred.md`](./archive/r304-taffy-upgrade-deferred.md)、R303 → [`archive/r303-dc9-gpu-primitive-audit.md`](./archive/r303-dc9-gpu-primitive-audit.md)、R142–R302 → [`archive/rounds-r142-r302.md`](./archive/rounds-r142-r302.md)、R23–R139 → [`archive/rounds-r23-r139.md`](./archive/rounds-r23-r139.md)、R11–R20 → [`archive/rounds-r11-r20-reftest-investigation.md`](./archive/rounds-r11-r20-reftest-investigation.md)。


## 综合裁决：结构性 plateau（R305–R323，≥10 轮一致收敛）

> 本节为 doc-maintenance 轮（2026-06-19）对最近 ~20 轮的**浓缩结论**，置于控制面板顶部便于检索。逐轮详细记录见文末「最近轮次详细记录」（R335–R336 全文；R337–R354 见「综合裁决」）与归档 [`archive/rounds-r314-r334.md`](./archive/rounds-r314-r334.md)（R314–R334）、[`archive/rounds-r309.md`](./archive/rounds-r309.md)（R309）、[`archive/rounds-r308.md`](./archive/rounds-r308.md)（R308）、[`archive/rounds-r307.md`](./archive/rounds-r307.md)（R307）、[`archive/rounds-r305-r306.md`](./archive/rounds-r305-r306.md)（R305–R306）、[`archive/r304-taffy-upgrade-deferred.md`](./archive/r304-taffy-upgrade-deferred.md)（R304）、[`archive/r303-dc9-gpu-primitive-audit.md`](./archive/r303-dc9-gpu-primitive-audit.md)（R303）、[`archive/rounds-r142-r302.md`](./archive/rounds-r142-r302.md)（R142–R302）。

**核心结论**：rendering-compat 的**所有单会话 / 中会话 forward-motion 杠杆均已 ruled out 或 refuted**——这是 R313–R323（≥10 轮）一致收敛的结论，非单轮判断。rally 单会话迭代已无法提升真实通过率。

**基线（R323 复验；strict post-R326 再复验仍零漂移；**R388 修复 108 损坏 Ahem oracle 后 chromium-Oracle 上修**）**：

- self-source loose：**443/490 (90.4%)** @ 默认 1%/5% 容差
- self-source strict：**295/490 (60.2%)** @ 锁定 0.1%/0.5%（DC-14 真通过口径）
- chromium-Oracle 广义一致：**200/475 (42.1%)** @ chr<1%（R391 锁定诚实基线）；严格 self-pass&chr<1% **177/475 (37.3%)**；污染 46.5%。**⚠️ R388 报的 43.2% 含 R389 暴露的 5 假一致（flexbox blank-blank），已纠正为 42.1%；pre-R388 35.8% 被 108 损坏 Ahem oracle 压低已修**
- 产品 smoke：welcome **16.98%**（product-smoke）/ wintertc 13.70% / morning-work 800×600 **16.41%**（R373 后）/ fullpage 48.65%（全文本度量结构性，非图片/CSS 缺口）

> **post-R326 strict 再复验（2026-06-19 doc-maintenance read-only，test-guard 包裹 `ZERO_REFTEST_STRICT=1 ... reftest-upstream`）**：strict 仍 **295/490 (60.2%) / 195 fail**（zero drift vs R323）——确认 plateau 在 DC-14 诚实指标上成立：R324（position:fixed）/R325（img aspect）/R326（sticky）三处 DC-11 correctness 修复**均未**把任一 strict-fail 翻成 strict-pass（loose 亦经三 commit 各自复验 438/490 零回归）。详见 [`evidence/strict-baseline-reverify-post-r326-2026-06-19.txt`](./evidence/strict-baseline-reverify-post-r326-2026-06-19.txt)。**顺带纠正**：旧文「295/490 (60.4%)」百分比过时（60.4% 为 R308 前 296/490 值；R308 font-size% 修复使 strict 296→295 后未同步百分比，295/490=60.2%）。

**已穷尽 / 证伪的杠杆（勿再以单会话重试）**：

| 杠杆 | 裁决轮 | 结论 |
|------|--------|------|
| near-pass clean-win frontier | R307 | 26 个 <0.2% 案全落结构性墙 / 字体噪声，零 clean win |
| POLLUTED 候选逐项 hunt | R299/R300/R302/R309/R311/R329 | 三趟复核（R298 清单逐项 + fresh 475 例 top-30 + 长尾 spot-check）全结构性/特性缺口，exhausted；唯 R308 font-size% 一处真实 clean win |
| fresh chromium-Oracle cross-validate | R311 | 4 新候选 ruled out，plateau 再确认 |
| Phase A IFC font_size 解锁（多行存储） | R125/R198/R205/R206/R209/R213 | R209 放宽多行→multicol-fill-auto-001 self-source 回归（误判回退）；**R355 用 chromium-Oracle 复测证该回归=假阴性**（multicol-fill-auto-001 Z_vs_chr 9.15% 不变，self-source 0.63% 本是假通过），加 **float guard**（浮动容器保 R84 单行限制）解死锁：ifc-008/009 Oracle -4.01/-1.95%；paint 侧 line.y 偏移补全（text.rs:832，Gate 2 ca14d05 的 load-bearing 配套）使 ifc-008 self-source 4.17%→**0.00% PASS**，self-source **439→440/490 净 +1 零回归**。**Phase A 多行路径首次 Oracle-net-positive**（R207 单行 +1 后再进）；R125/R198/R205 font_size/line-height 耦合单点仍死锁，但多行存储 + float guard + paint line.y 是可行 narrow 扩展 |
| multicol breaking paint 侧 | R157/R198/R203/R317 | 5 次实证全 net-negative，paint 侧死路 |
| multicol balance 二分搜索 | R199/R200/R321/R322 | T/N ≡ binary-search（等高行）；columns-001 diff 实为 wrapping 精度，非 balance |
| multicol column-aware IFC（layout 侧） | R319 | spec-rfc 产出 + A1 probe REFUTES Phase 1（目标结构在失败集几乎不存在，迁移零增益） |
| baseline-export（3 机制） | R266/R310/R312/R313/R316 | field-fill 净 0 / inline-flex 不受控 / flex-post-pass 回归，3 路全证伪 |
| advance-width plumbing | R225/R320 | 双角证伪（reftest-oracle 零变化 + Ahem advance 精确），死路 |
| DC-9 blend_mode | R278 | 单 framebuffer post-process 架构不可行，需 paint-isolation |
| font-weight -Bold 接线 | R229b | fontdue Bold 过墨 ~15%，net-negative |
| @font-face 自定义字体加载（css-fonts 聚类解锁假设） | R330 | 17 字体导入激活后实测：self-source 438/490 持平；chromium-Oracle 净负向（alternates-order +7.53 / font-features-across-space +0.95，余 54 不变，污染仍 ~46%）。根因 = rustybuzz TextShaper 未接入生产路径（生产逐字符 glyph_id=ch as u32），加载特性字体不应用特性更远离 chromium。fontdue/simple-shaping 第四条死路（同 R225/R229b/R174） |
| advance-width paint 单侧 fontdue | R225/R331 | layout(paint?) 双侧均用 estimate_char_width 一致（self-source 抵消）；paint 单侧改 fontdue 致与 layout 不一致 intra-fragment 错位；双侧改 = R225「三处同源」已证 oracle 26 案零变化。死路 |
| rustybuzz shaping 接入生产（单会话 bounded probe） | R331/R332 | R331 定位 4 子任务多会话；R332 实现全 4 子任务（high-bit 哨兵法避免 GlyphPrimitive 55 处构造点改动）+ SHAPE_PAINT 门控实测：self-source 438/490 持平；chromium-Oracle pollution 48.0%→48.3%（+2），零 case 改善。**paint-only shaping 净负向**——layout 仍用 estimate 断行，paint 单侧 shaping 致 layout/paint 不一致。须 layout+paint 同源（Phase A IFC 统一子集）。保留 TextShaper advance/offset bug 修复（真实修复），回退 probe 其余 |
| taffy 0.11 升级 | R304 | DEFER（541 ref + 108 alignment + native float 冲突，具名缺口零收益） |
| clear+margin-collapse（clear-float-003 等） | R333 | ZeroWeb 把 `clear` 仅 convert_clear→bool 传 taffy 0.7，无 ZeroWeb 侧 clearance 后处理 → clear+margin 折叠交互 = taffy-0.7-bound（R323 探针覆盖基本折叠不含 clearance）。须 taffy 升级或自建 clearance 后处理（与 taffy margin 折叠耦合，高风险） |
| CSS2/multicol 剩余失败（border-bottom-width-006 / border-padding-bleed / multicol-count-computed-003/004 等） | R333 | 新鲜审视 5 未深查用例全结构性：inline-box 模型（vertical-align/行盒绘制，与 Phase A 耦合）；multicol 列几何计算 spec-正确（divergence 在内容分布/规则定位，multicol 结构死锁）。DC-11 correctness 轴（R323-R326）确已穷尽 |
| WM-1 abs-pos-non-replaced-vrl/vlr（R237/R238「首选候选」从未执行） | R334/R335/R336 | R334 positioning（direction-rtl 镜像修复 spec-correct，0 clean win，latent 保留）→ R335 color（per-fragment 颜色 net-negative，paint-IFC 位置错）→ R336 suppression（精确定位：collect_inline_items 不检查 position 收 abspos 文本 + render_fragment! 标记 painted 抑制 abspos 自身 paint_text；refined skip 解抑制但 span 自身 paint-IFC 空 styles 产出 fs=16 = double-path，net-neutral 回退）。**三角度均收敛 Layout/Paint IFC 双路径（gap #4 = Phase A）**，WM-1 单会话 lever 彻底穷尽 |
| R348 #1 polluted case = backdrop-inherit-rendered（47.54%）= DOM/JS gap，非 CSS bug | R349 | **非渲染 bug**：该 case 测 `dialog::backdrop` 伪元素 + `<dialog showModal()>` + CSS var 继承；ZeroWeb 把 `dialog` 列为非渲染元素（style-system/lib.rs:73），无 `::backdrop` 伪元素 / showModal / modal 态支持。属 JS/DOM API 兼容，**out of rendering-compat scope**（goal 明确排除）。CSS `backdrop-filter`（apply_advanced.rs:807）是**另一回事**且已实现。**真 #1 CSS-rendering polluted case = abs-pos-border-offset-002（27.94%，writing-modes，Phase A）**。多数 top polluted 是纯 CSS 结构聚类（writing-modes/grid/fonts/flexbox-baseline/multicol），非 DOM/JS；backdrop 是孤例 |
| wintertc @media + escaped-class（Twind `sm\:`/`hover\:`）疑似缺口 | R350 | **probed 两路均 GREEN（work）**：(1) `@media(min-width:640px){.box{background:green}}` 在 800×800 视口产出 green（0,128,0）= @media 经 `evaluate_media_query`（matcher/mod.rs:997 + lib.rs:238 MediaContext::new(w,h)）正确按视口求值；(2) `.sm\:green` 选择器匹配元素 `class="sm:green"`（tokenizer `consume_escape` tokenizer.rs:385 正确消费 `\:`→`:`）；(3) 二者组合（@media + escaped class）也 green。**故 wintertc 残余 13.7% 非 @media/escaped bug**：per-band diff 集中在正文区（y=250-450，17-24%=systematic 文本度量 Phase A）+ logos 区（y=550-600，20.7%=flex-wrap/justify-evenly 精度 taffy R304 DEFERRED）。R318 README「Twind 类生效、无 clean bug」结论经 fresh 复核成立。勿再以 @media/escaped-class 嫌疑重查 |
| self-source FAILURE 近-miss 全量审计（R351 方法学延伸：逐案 pixel dump + 根因） | R352 | **R351 table-layout:fixed 是近-miss 中唯一 clean win**；余皆复杂/阻塞/结构性（pixel-dump 确认）：baseline-007（green 整体下移 60px = child-1 div 高，`align-items:baseline` + multicol baseline-export，R266/R313/R316 谱系）/ multicol-breaking-006（嵌套 multicol R113）/ child-border-box-max-content-002 + flex-container-max/min-content-001 + grid-flex-spanning（**全 grid/flex，max-content→0 computed.rs:68，taffy-blocked；block/inline-block 无失败案——abs-pos-vlr-border-001 用 width:max-content 且 PASS，故独立修复路径无杠杆**）/ multicol-collapsing-001（margin-collapse + image ref）/ multicol-block-no-clip-002（blue 内容外溢 = multicol 分布 R128/R131）/ multicol-count-computed-003（column-gap:5em 大间距分布）/ css-flexbox-row（vertical-rl R109）/ firefox-bug-1881495（grid inline-grid R143）/ background-attachment-applies-to-001（CSS2 applicability）。**结论**：近-miss clean-win 已穷尽，余 51 失败需多会话架构（Phase A / multicol fragmentation / taffy 升级） |
| 全失败案 CSS 属性系统性审计（含 inline style，找「完全未实现」属性） | R353 | **非 multicol 属性全实现**（apply handler 存在）；**完全未实现（0 refs：未解析/未存储/未应用/未用）全是 multicol 特性**：`column-span`（baseline-007 等用 `column-span:all` 建 spanner）、`orphans`/`widows`（CSS2 整数，控列/页断行 widow/orphan）、`column-height`+`column-wrap`（multicol-2 草案，column-height-009）。**此即 multicol 为最大失败聚类（~15 案）的根因之一**：缺 column-span spanner + orphans/widows 断行控制 + multicol-2 特性，皆需 multicol fragmentation 架构（R113 column_span_offsets 谱系）才能生效——单独 parse+store 无行为变更（dead code，code-guidelines 禁），须 layout 侧消费。percentage-gap 经查正确（convert_length_to_lp Percentage→Percent 让 taffy 解析）；bg-position 经查正确（ZeroWeb 不轴交换=符合规范，left/top 是物理）。**结论**：clean-win 面（R351 式 applied-but-not-respected 单点属性）在剩余失败中穷尽，余皆 multicol-fragmentation 架构或已正确的特性 |
| 8 失败案独立 pixel-dump 根因再确认（fresh baseline + 逐案 REFTEST_DUMP） | R354 | fresh baseline **439/490 零漂移**；8 案逐案 pixel-dump（border-001 / multicol-block-no-clip-002 / multicol-collapsing-001 / multicol-count-computed-003 / float-006 / baseline-007 / clear-inline-001 / flexbox-column-row-gap-004）**0 clean win**，每案根因均落 4 架构路径之一。**关键纠正**：border-001 非 border bug（真 border 渲染正确，发散在 REF 的 Ahem 文字塌缩=Phase A）；clear-inline-001 非 clear bug（TEST 的 clear-on-inline 正确，发散在 REF 的 inline-block img+span 换行=Phase A 所有权分裂，adjust_inline_block_positions 双路径证）。详见 [`evidence/r354-plateau-revalidation-2026-06-20.txt`](./evidence/r354-plateau-revalidation-2026-06-20.txt)。**结论**：clean-win hunt hit-rate 衰减至 0，forward motion 须多会话架构 |
| R355 paint 侧 line.y 补全 + ifc-009 残余根因定位（二次诊断修正） | R355/R356 | R355 paint 侧 line.y 偏移（text.rs:832）是 Gate 2（ca14d05）的 load-bearing 配套——stash 对照证 HEAD Gate-2-only ifc-008 4.17% FAIL → +fix **0.00% PASS**；self-source **439→440/490 净 +1 零回归**（commit 787677e）。R356 二次诊断修正 ifc-009（2.08%）真根因：**首轮「line_height=20 传播」是 remeasure 路径伪影**；paint 存储片段探针实证渲染路径 line_height 正确（h=100），真根因=**跨 block float 侵入未实现**——inner-div（与 float div2 兄弟，同 div1 子）的 IFC 只看自身直接 float 子，div2 不传播到 inner-div IFC → line 1 用满宽 300px 容 "X X"（第 2 个 X 恰被 float 覆盖致 top 巧合正确），line 2 仅 1 X → 右下 100×100 应蓝实橙。属 M4 完整 float 布局 / Phase A 所有权分裂，单会话高回归风险（耦合 R355 float guard），defer。详见 [`evidence/r356-ifc009-lineheight-float-overlap-pinpoint-2026-06-20.txt`](./evidence/r356-ifc009-lineheight-float-overlap-pinpoint-2026-06-20.txt)。**结论**：R355 是真 +1 clean win（已提交）；ifc-009 真根因=跨 block float 侵入（Phase A/M4 第三机制，独立于 font 度量），未修 |
| 低 diff 失败第三处 Phase A 簇确认（multicol-count-computed-004 存储丢色） | R357 | 像素级根因（无代码变更）：multicol-count-computed-004（2.00%）换行正确，唯一 diff=color——存储 IFC 片段（InlineLayoutFragment）**不携带 color 字段**，paint 存储路径所有片段用容器 color（black），REF 期望各 span 自身 color（粉/橙/紫/灰）。auto+auto 非多列（compute_column_info 正确返 None）。R357 当时判「per-fragment color 修复不安全（耦合 R335 abspos 回归）」**已被 R358 推翻**（加 abs-pos guard 即解）。三处独立诊断共享根因=paint 阶段丢/错 layout 的 per-fragment 语义。详见 [`evidence/r357-stored-ifc-color-loss-phase-a-cluster-2026-06-20.txt`](./evidence/r357-stored-ifc-color-loss-phase-a-cluster-2026-06-20.txt) |
| **R358 per-fragment color + abs-pos guard（plateau 第三次突破，+1 clean win）** | R358 | **纠正 R357「per-fragment color 不安全」结论**——加 **abs-pos guard**（abs-pos/fixed 片段保留容器 color）即解 R335 回归：`render_fragment!` 宏（text.rs 非多列路径）解析每个片段所属元素 color，position:Absolute/Fixed 者用容器 color（维持当前行为，不激活 R335 的 abspos 绿 X 错位显眼化），其余用自身 color（镜像多列路径 line 1033 frag_color）。验证：**multicol-count-computed-004 FAIL→PASS（2.00%→1.00%）**；self-source **440→441/490 (90.0%) 净 +1 零回归**（abs-pos-non-replaced-vrl/vlr 簇不变，writing-modes 53/59 持平；唯一他例 float-lft-orthog-htb-in-vlr-002 6.68%→6.53% 仍 FAIL=微改善非回归）；engine 1146/0（+1 单测 test_paint_per_fragment_color_for_spans）、clippy/fmt 干净。**启示**：R335 net-negative 是因无 guard 全量应用 per-fragment color；scoped guard（abs-pos 例外）把「Phase A 耦合」降为「单点 clean win」。abspos 文本错位（R336）仍 Phase A，但 per-fragment color 本身非阻塞 |
| border-bottom-width-006（2.86%）=匿名块盒生成缺失（R109 谱系），非 clean win | R359/R360/R361 | 三轮诊断收敛：① R359 误判渲染管线合成；② R360 纠正=两个 fill 均正确渲染（#test border (8,51,96,96) black、#reference bg (12,16,96,96) black，像素 (50,100) 确认黑），真根因是 #reference 定位到 (12,16) 而非紧邻 #test；③ R361 LAYOUT_DUMP + 代码追踪精确定位：body 含混合内容 `<p>`(block) + #test/#reference(inline-block)，**CSS §9.2.1.1 要求 inline-blocks 被匿名块盒包裹**（独立 IFC 容器），ZeroWeb 不生成此匿名 wrapper——inline-blocks 作为 body 直接子，`adjust_inline_block_positions`（engine.rs:880）对 body 跑 IFC 覆盖 taffy 位置，但 IFC 不感知 `<p>` block 兄弟 → #reference 落 line1(y=16 body 顶)、#test 落 line2(y=51)，顺序反转。taffy 块堆叠 / IFC 误定位**均错**，正解需匿名块盒生成（R109 匿名块级联谱系）。style/layout-box/paint 全正确。属匿名块盒生成架构（与 R255 ua_default_display 同 R109 谱系），单会话高风险。**结论**：clean-win 面经连续多轮（R359-R361）确认穷尽，forward motion = 多会话 Phase A / 匿名块盒生成 |
| **R362 跨 block float 侵入传播（plateau 第四次突破，+1 clean win）** | R362 | Phase A 第四个 narrow viable-slice：CSS float 侵入——祖先 BFC 内的 float 应侵入未建 BFC 的后代 block 的 line box。`compute_final_inline_layouts` 加 `ancestor_floats: &[FloatExclusion]` 参数，递归时把（祖先 float + 本容器直接 float）按 `f.y - child.y` 平移到子节点 box 坐标系传播（FloatExclusion 无 x 字段，IFC 仅按 left/right+width 缩减行盒）；**排除子节点自身**（float 不在自身 IFC 排除自己——float-005 回归实证后修正）。验证：**ifc-009 FAIL→PASS（2.08%→0.00%）**；self-source **441→443/490 (90.4%) 净 +1 零回归**（首轮 float-005 因 float 自排除缺失回归，加 self-exclusion 后恢复；全量无其它回归）；layout-engine 891/0（+1 单测 `test_r362_float_intrusion_propagates_to_sibling_block_ifc`）、engine 1146/0、clippy/fmt 干净。**启示**：Phase A 谱系（R355 多行存储 / R358 per-fragment color / R362 float 侵入）连续三个 narrow viable-slice 成功——paint/layout IFC 语义丢失可逐项 scoped 修复，非全有或全无；剩余 Phase A 项（abspos 文本错位 R336）同谱系可继续 |
| 上一轮 CONTINUE 标记的 3 个「未深查」候选 + 1 个 false-pass polluted 案精确诊断 | R363 | baseline 复验 self-source **442/490 零漂移**；4 案 REFTEST_DUMP+PIL 诊断 **0 clean win**（hit-rate 0/4，与 R352/R354 一致衰减）：① flex-abspos-inset-nested-001/002（8.33/18.75%）= **2 互相依赖 bug**（abspos §10.3.7 shrink-to-fit 缺失→inner-flex w=0 + flex 替换元素主轴固有比尺寸，单修任一无效）；② fixed-table-layout-with-percentage-width-in-flex-item（11.20%）= flex definite-size resolution（flex item 含 width:100% table 后代塌缩到 2px，Mozilla bug 1469649 谱系）；③ **multicol-contained-absolute（chr 16.33% / self 0.00% = FALSE PASS，首次定位根因）**= multicol `column-fill:balance` 不平衡单个 200px 子（ZW 392x200 单列 vs chromium 784x100 跨两列平衡），test/ref 同源假通过；④ multicol-fill-000/count-002 = multicol 平衡/分布结构性。**方法论纠正**：自写 PNG 解析器 alpha/filter 字节 bug 误判 oracle 为「黑图」，PIL 复核 oracle 内容有效——polluted 案诊断须 PIL 对 oracle。详见 [`evidence/r363-flagged-candidates-structural-2026-06-20.txt`](./evidence/r363-flagged-candidates-structural-2026-06-20.txt)。**结论**：clean-win 面经 R352/R354/R363 三轮 0/N 衰减穷尽，forward motion 须多会话架构 |
| **R364 table 显式 width 列冻结 + min-content floor（CSS 正确性修复，net-neutral self-source / chromium-Oracle 改善）** | R364 | table-cell-width-0（self 31.57% FAIL）诊断：definite-width 表的显式 width 单元格列被扩展填满块按比例撑大（td.big-positive 20px→529px）。修复 `compute_column_widths`（table.rs）：① 扩展填满块改为显式 width 列冻结（col_explicit 标记，Pass 1 收集），仅 auto 列按当前宽比例吸收剩余（全显式时回退比例扩展）；② `cell_used_width` 显式分支 `base.max(intrinsic)` floor 到 min-content。验证：TEST 侧列宽全修正（zero=9.6/positive=9.6/big=20，normal 吸收剩余）；**ZeroWeb-TEST vs chromium-Oracle = 11.14%**（DC-14 真指标，修复前 TEST 明显错）；self-source 全量 **442/490 净中性零回归**（table-cell-width-0 仍 29.63%——REF 侧 `width:fit-content` on flex container 渲染满宽，taffy 0.7 blocker R304 DEFERRED，TEST 正确但 REF 错=同源假阴性）；layout-engine 893/0（+2 单测）、engine 1146/0、clippy/fmt 干净。**裁决**：CSS 正确（显式列不吸收剩余 + 列宽 min-content 下限均规范行为），服务 DC-14 真指标方向，待 fit-content-on-flex 解锁即贡献 PASS；R363「fixed-table-in-flex 同 flex definite-size 谱系」结论对【flex 内 table】仍成立，本修复是【table 自身】显式列分布，独立 |
| plateau 再确认（4 新角度）+ 系统性 REF-side blocker 洞察 | R365 | baseline 复验 self-source **442/490 零漂移**；4 新角度诊断 **0 clean win**：① fit-content/max/min-content 关键字全 flex/grid（taffy-blocked），无 block/inline-block 失败案用 → 无杠杆；② **min-max-size-table-content-box（36.34%）= TEST+REF 双侧多 bug + spec 冲突**——min-height 表格 border-box（h=50 应 66），但改 content-box 回归 min-height-table（csswg-drafts #5336 两案冲突）+ REF inline-block shrink 不生效（R180 仅 definite-width 子元素生效）；③ multicol-columns-001 = multicol wrapping 精度（R128 结构性）；④ inline-block shrink gap 仅 1 失败案 REF 且双侧阻塞 → 零杠杆。**系统性洞察**：多失败案（table-cell-width-0 / min-max-size-table）TEST 侧可修对但 self-source 因 ZeroWeb 自渲染 REF 错（fit-content-on-flex / inline-block shrink）而**假阴性** → 实证 DC-14 self-source 含系统性假阴性。详见 [`evidence/r365-refside-blocker-insight-2026-06-20.txt`](./evidence/r365-refside-blocker-insight-2026-06-20.txt)。**结论**：clean-win 面经 R352/R354/R363/R365 四轮 0/N 衰减彻底穷尽，forward motion 须多会话架构 |
| ifc-011 簇 IFC 表面修复三维度探针（margin / border-box 尺寸 / width-shrink） | R366/R367/R368 | R366（inline vertical-margin 归零）net-neutral 回退；R367（inline-block border-box 尺寸 in IFC）net-negative 回退（ifc-011 11.27→13.73%）；**R368 重开「宽度维度」新表面修复并锁定保留**——只读探针（LAYOUT_DUMP）定位 ifc-011 真根因：span w=784 满宽拉伸（R180 shrink 因 span.children 空→content_max_w=0 失败）是 PRIMARY 缺陷，div h=60 是 taffy 块容器高度（解耦）。修复 `shrink_inline_blocks_to_content` 改用 `intrinsic_sizing::box_content_max_width`（按 DOM text_content + 字体度量，处理无子盒纯文本）：ifc-011 **11.27→1.23%**（-10pp），self-source **442/490 净中性零回归**，+1 单测。border-box ib_sizes 配套实验证伪（net -1，multicol-dynamic-change 0.97→1.05% 翻 FAIL，回退）。残余 1.23% = span2 x 重叠（需 border-box -1）+ glyph 度量 + height-grow（未试，对翻 PASS 无杠杆+cascade 风险）= 多会话。**R369 DC-14 升级**：cross-validate vs chromium-oracle 实测 R368 是真 DC-14 大胜（ifc-011 z_vs_chr 12.30→2.22%，非 self-source 假象）；border-box 终局证伪（z_vs_chr 2.22→2.50% +0.28pp 真 chromium 退步，x 位置与 glyph baseline 耦合），三层证伪彻底关闭。详见 [`evidence/r368-inline-block-text-shrink-2026-06-20.txt`](./evidence/r368-inline-block-text-shrink-2026-06-20.txt)、[`evidence/r369-borderbox-dc14-refutation-2026-06-20.txt`](./evidence/r369-borderbox-dc14-refutation-2026-06-20.txt) |
| DC-14 全失败案扫描：false-negative 亦结构性 | R369b | 对全 48 self-source 失败案跑 DC-14 chromium-Oracle 扫描（REFTEST_DUMP + cross-validate.py），找「self-fail 但 z_vs_chr 低」（false-neg：test 已≈chr 仅 ref 发散）的易修 ref bug。5 候选逐案证伪均结构性：flex-abspos-inset-nested-001/002（chr 0.73/0.74%）像素分析揭穿 ZW-test 与 chromium **均退化**（非 200×200，z_vs_chr 低仅因两者主体皆白）；baseline-multi-line-horiz-003/004 = baseline-export 聚类（卡点#4）；box-offsets-rel-pos-vlr-005 = WM 结构性。**结论**：false-negative 亦结构性，self-source 非因易修 ref bug 人为偏低，反向印证 plateau；DC-14 方法论=可信判据（后续修复应一律用 z_vs_chr 验证）但不改 forward-motion 结论。详见 [`evidence/r369b-dc14-scan-falseneg-structural-2026-06-20.txt`](./evidence/r369b-dc14-scan-falseneg-structural-2026-06-20.txt) |
| **🔍 R388 chromium Oracle 损坏（Ahem 未加载）→ large-font 簇「发散」是 oracle 镜像** | R388 | 承接 R387「large-font lever=stored-path Y 定位」做像素级诊断，**反转发现**：ifc-008 ZW 渲染**本就正确**（实心绿 200×200、0% 红），7.93% 发散全来自**损坏 oracle**（85% 红底=fallback 细 X 字形）。根因=oracle 06-18 抓取早于系统 Ahem 06-20 安装；`chromium-oracle-shot.mjs` 用 `file://` 无法解析绝对 `/fonts/ahem.css` → 108 Ahem 依赖 reftest oracle 全损。修复=脚本内嵌 HTTP server root=wpt-data（自包含）+全量重抓 503。cross-validate 复测：广义 chr<1% 一致 **193→205/475（+12）**、污染 48→46%、26 案改善（large-font/字体簇）vs 14 案「退步」=正确 oracle 揭示先前被 fallback 掩盖的真实发散（非 ZW 回归）。**推翻 R385「fontdue 度量死路」+ R387「large-font=layout Y」对该簇归因**（追 oracle 镜像）；fontdue-perfect 结论仍成立。large-font 簇证正确、**不再是 lever**。详见 [`evidence/r388-oracle-ahem-invalidation-2026-06-21.txt`](./evidence/r388-oracle-ahem-invalidation-2026-06-21.txt) |
| **R389 正确 oracle 下重扫 + ../support/ 图片路径（第二处资源 gap）** | R389 | 承接 R388 重扫找 clean win。**结论：plateau 在正确 oracle 下仍成立**（R384 self-source 穷尽 oracle-无关；R363/R354 结构性归因不变）。font-family-name-025=字体回退噪声（test 依赖未装 CSSTest/Verdana）。发现第二处资源 gap：9 css-flexbox test/ref 的 `../support/1x1-green.png` 在当前 repo 布局断裂（实际在 `css/css-flexbox/support/`，WPT 同目录约定）→ 图片 404 → 两端退化全白。修复=`../support/`→`support/`（9 文件，资源引用修复）+ 重抓 6 oracle。**self-source 443/490 零变化零回归**。6 案真实根因=abspos flex 容器宽度解析（top/bottom inset definite height，ZW shrink-to-fit 1px vs chromium 更宽）= **taffy-blocked 结构性（R363/R97/R304 DEFER），非 clean win**。3 self-pass 案 test+ref 同 flex 结构故 self 一致。详见 [`evidence/r389-rescan-oracle-imgpath-2026-06-21.txt`](./evidence/r389-rescan-oracle-imgpath-2026-06-21.txt) |
| **R392 text-emphasis 实现 net-negative（line-box 定位阻塞）** | R392 | R391 后扫 near-pass 发现 32 个 text-emphasis 测试完全未实现（疑缺失特性 clean win）。实现全栈（style parse/store/inherit + paint 每 glyph 标记）。过程中修继承 clobber bug（简写入 is_inherited 致继承循环覆盖长手）。实测 **net-negative**：z_vs_chr 1.44→1.52%、chr<1% 15→9（-6）。根因=32 案全用 `line-height:5`，暴露 ZW line-box 垂直定位与 chromium 分歧（标记须 line-box 顶相对定位，ZW 半行距分布不同 = IFC 垂直 / Phase A 谱系）。标记渲染了但位置错。**100% 回退**（9 文件 grep=0），443/490 零回归。text-emphasis **非 clean win，阻塞 line-box 定位**，勿再单会话重试。详见 [`evidence/r392-text-emphasis-linebox-blocked-2026-06-22.txt`](./evidence/r392-text-emphasis-linebox-blocked-2026-06-22.txt) |
| **R393 line-box 半行距垂直定位（strut_ascent）非 clean lever** | R393 | 承接 R392 查 `apply_vertical_alignment` 的 `strut_ascent=line_height*0.8`。实测 line-height:5：ZW CJK 文本 y=64 vs chromium y=69（ZW 高 ~5px）。把 `0.8`（baseline 64）改教科书 `(L+font)/2`（48）会**反向发散**（ZW 已偏高）→ `0.8` 启发式本就比教科书更近 chromium，~5px 残余是 font-metric 噪声非可修 bug。R392 text-emphasis 真因=paint 侧标记 line-box 顶相对定位（非文本 baseline）。**line-box 垂直定位加入 plateau 第 5 项确认**，非 lever。详见 [`evidence/r393-linebox-halfleading-not-lever-2026-06-22.txt`](./evidence/r393-linebox-halfleading-not-lever-2026-06-22.txt) |
| **R396 grid 全量分母缺口簇抽样（read-only；并行 agent 未提交）** | R396 | 承 R395 CONTINUE 核查「更大分母重开 clean-win 面」。HEAD 仍 `f2c9fae` 无新 commit；并行 agent mid-work 未提交（css-grid ~70 顶层文件≈40+ 真测试过 63 目标 + `discover-reftests-authoritative.py` 落地 + quirks/ 起步）。**2 缺口簇测试 read-only 抽样**：① `grid-container-baseline-synthesized-001`（inline-grid 空项 baseline 合成 + 5 writing-mode）= **baseline-export 簇**（R266/R313/R316 ruled out）；② `replaced-element-percentage-height-in-grid-nested-in-flex-001`（img height:100% @ grid(height:100%) @ flex(height:200px column)）= **flex/grid definite-size resolution 簇**（R363/R97/R304 DEFER）+ 需图片子资源。**二者均非单会话 clean win**。**结论**：R395「子集 clean-win 穷尽不可外推」caveat 在「更大分母含真实缺口簇」方向成立，但新失败是**同架构族更多实例**（baseline-export / definite-size / fragmentation / grid-within-flex / form-control），非「重开单会话 clean-win 面」；轨道 1 方向不变。无法量化（未提交 + reftest 冲突/OOM）。详见 [`evidence/r396-grid-gapcluster-sampling-2026-06-22.txt`](./evidence/r396-grid-gapcluster-sampling-2026-06-22.txt) |

**剩余 forward motion（R395 复排，DC-14 分母 gap 发现后）**：

> **⚠️ 子集范围警示（R395）**：下述「渲染架构」轨道（2-4）的目标都是子集分母下的失败聚类。R394 实测当前导入仅上游 **~5-6%**（503/~8000-10000），R384「单会话 clean-win 47/47 穷尽」、R351-R393 的聚类归因**全部基于此子集**。全量集合含未检失败模式，clean-win 面与各架构轨道的真实 ROI 都须在去子集化后重新评估——**不可把子集结论外推为全局穷尽**。

1. **【优先·已落地】DC-14 分母去子集化** — gating DC-14 硬门禁（goal line 317/843）：达标判定前必须用上游每目录**全量** reftest。最可操作的多会话增量 = Phase 2 小目录全量导入（grid 63 / position 149 / tables 203，合计 ~415 案可达 100% 覆盖）→ 再扩大目录（flexbox 586 / writing-modes 855 / multicol 537 / text-decor 356 / fonts 391）→ 最后 CSS2（~5000-7000）。每批 reftest + chromium-Oracle 复测，监控真通过率。**状态**：并行 agent 已建 `discover-reftests-authoritative.py`（按 `<link rel=match>` 权威解析 test→ref 对，替代文件名启发式）+ 正在导入 css-grid 全量（18→39→63 进行中）。详见 [`evidence/r394-dc14-denominator-gap-2026-06-22.txt`](./evidence/r394-dc14-denominator-gap-2026-06-22.txt)。
2. **【渲染架构·子集范围内穷尽】Phase A IFC 三路径统一** — paint 不重跑 IFC，直接渲染 layout 存储的行盒（R205/R207 viable slice 已证 font-051 可行；R355 多行存储 / R358 per-fragment color / R362 float 侵入 三个 narrow viable-slice 已成；broad 应用需多轮 narrow 精修 + 守 multicol-fill-auto 反向依赖墙）。设计文档 [`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)。
3. **【渲染架构·子集范围内穷尽】Phase 2 嵌套 multicol fragmentation** — layout 侧 column-aware IFC + 嵌套列碎片化（R131/R201；R319 证纯 inline 迁移零增益，真实价值在嵌套 / 混合内容；R383 证混合内容修复前置依赖 Phase A / R109 解转换）。设计文档 [`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md)、[`column-aware-IFC-spec.md`](./column-aware-IFC-spec.md)。
4. **【渲染架构·子集范围内穷尽】baseline-export 真修复** — taffy 0.8+ baseline_overrides（需先解 R304 升级冲突）或自建 inline-level-box baseline 合成；解 flexbox-baseline / multicol-baseline 聚类（~10+ 案）。

**裁决（R395 更新）**：轨道 1（分母去子集化）是当前**唯一可推进且非纯架构承诺**的前向路径——并行 agent 已在执行（css-grid 全量导入 in-flight）。轨道 2-4（渲染架构）在当前子集内已被 R384 等系统性证伪为单会话不可解，但其全局 ROI 须待轨道 1 揭示全量真通过率后再校准。**无需用户在「架构承诺 vs 接受 plateau」之间二选一**——先推进轨道 1（既满足 DC-14 门禁、又为轨道 2-4 的优先级提供数据），是当前最高 ROI 的下一步。

> **分支审计（2026-06-19 doc-maintenance read-only）**：未合并分支 `fix/rendering-compat-stacking`（8959ddb，2026-06-12，自称 R61 / 基线 387/490）经核查**冗余**——其核心改动（painter positioned/z-index 堆叠排序 CSS 2.1 App. E + `sync_inline_child_boxes_from_ifc`）**均已在 main**（`crates/engine/src/paint/painter/mod.rs:56-78`、`crates/layout-engine/src/engine.rs:1219`），且 main 版本更完整（额外处理 stacking-context 创建 + z-index:auto tree-order）。该分支**非**未合并 plateau 杠杆，亦非活跃并行开发（06-12 后无后续 commit）；doc-maintenance 续以 main HEAD 为准，不并入未合并分支内容。

---
## 里程碑完成状态

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| M1 — WPT Reftest 基础设施 | ✅ 完成 | 14/14 标准全部达成 |
| M2 — CSS 2.1 + Quirks Mode | ✅ 完成 | CSS parser + style system quirks 已实现；layout engine quirks 推迟到 M4 |
| M3 — Flexbox + Grid | ✅ 完成 | 179 个 reftest, 100.0% pass rate；Flexbox/Grid 无渲染缺口 |
| M4 — Float + Table + Multicol | ✅ 完成 | float + table + multicol 布局算法已实现；219 个 reftest, 100.0% pass |
| M5 — 文字排版 | ✅ 完成 | CJK 换行 + justify 修复 + float 堆叠修复 + 51 个 Text reftest |
| M6 — 全量扩展 | ✅ 完成 | 685 reftest, 13 目录全部 ≥50, 100.0% pass；unicode-bidi + CJK 换行已接入生产。⚠️ **rustybuzz TextShaper（GSUB/GPOS）已实现+单元测试，但未接入生产 paint/layout 路径**（R330 代码核查实证：生产 text.rs:1057 仍逐字符 `glyph_id=ch as u32`，TextShaper 仅 lib.rs 单测调用）→ ligature/kerning/alternates 生产未生效 |
| M7 — 渲染器图元覆盖 | ✅ 完成（管线层）⚠️ | CPU 渲染器：全部 13 种图元 ✅；GPU 渲染器：13 种图元**管线**已建（48 单元测试 ✅），**但浏览器全量 GPU 路径 `render_full_scene_gpu` 实际消费以 DC-9 表为准**——transform=✅ R285/ee8373a 已接入（`collect_transforms`+`apply_transform_filters_headless`）、blend=CPU no-op stub+GPU 丢弃（**唯一剩余 GPU 真实缺口**）、5 color-matrix 滤镜（grayscale/invert/saturate/sepia/hue-rotate）=✅ R286/94c773a 已落（`collect_color_filters` mode 3-7 全处理，parity CPU）、clip=no-op（engine 从不生成）；filter:opacity/brightness/contrast/blur 已落（f6fed44/fc86937/3a3530f）；浏览器消费：全部 13 种图元 ✅ |
| M8 — 布局正确性 | ✅ 完成 | BFC 检测 ✅；float clear ✅；margin 折叠(taffy 0.7 内置) ✅；<img> 固有尺寸 ✅；position:fixed ✅(adjust_fixed_to_viewport)；position:sticky 需宿主层（已标记 is_sticky，后续集成）；percentage height/auto margin/min-max-width 已有测试验证 |
| M9 — 高级视觉效果 | 🔧 进行中 | 重复渐变 ✅；多图层背景 ✅；clip-path 全形状裁剪 ✅(inset+circle+ellipse+polygon)；border-image ✅；text-shadow ✅；backdrop-filter ✅；CSS mask ✅(渐变蒙版裁剪+alpha衰减)；overflow 全图元裁剪 ✅；滚动容器 paint 偏移 ✅(scroll_x/scroll_y 字段 + paint 时子元素坐标偏移 + 3 个单元测试)；剩余：scroll-snap 行为（需宿主层输入路由）、滚动输入路由（需浏览器 app 集成） |
| M10 — 上游 WPT 真实 Reftest 导入 | ⏸ plateau（R323） | 基础设施 ✅；490 上游 reftest 已导入（9 目录）；self-source loose **443/490 (90.4%)** / strict **295/490 (60.2%)** / chromium-Oracle ~35.6%；R305–R323 全单会话杠杆穷尽（R351 table-layout:fixed + R355 Phase A 多行存储 + R358 per-fragment color(abs-pos guard) 为三次 plateau 突破），达标需多会话架构（见「综合裁决」） |

## 当前状态概览

| 维度 | 状态 | 说明 |
|------|------|------|
| 渲染管线 | ⚠️ 全链路贯通但非一致 | HTML→CSS→Style→Layout→Paint→Composite 可运行；但 layout IFC、paint IFC 和 ZeroBrowser glyph 消费仍存在多套坐标/度量路径，`welcome.html` 已暴露用户可见错位 |
| WPT Runner | ✅ reftest 级 | 1,341 个手写 TestCase + 685 个内联 reftest（13 目录 ≥50） |
| Reftest Harness | ✅ 可用 | 分类容差、per-test fuzzy 注解、match/mismatch 模式 |
| Manifest Parser | ✅ 扩展完成 | reftest 条目解析、fuzzy 元数据、HTML 链接提取 |
| CPU 软件渲染 | ✅ 全量图元 | render_full_scene() 支持全部 13 种图元（fills, rounded_rects, gradients, shadows, images, strokes, path_fills, path_strokes, glyphs, clips, transforms, filters, blend_modes） |
| Reftest CLI | ✅ 可用 | `cargo run --bin zero-wpt-runner -- reftest` |
| Skip List | ✅ 已创建 | `tests/wpt-runner/reftest-skip-list.txt` |
| Chromium 截图脚本 | ✅ 已创建 | `tests/wpt-runner/scripts/capture-chromium-screenshots.mjs` |
| WPT 导入脚本 | ✅ 已创建 | `tests/wpt-runner/scripts/import-wpt-reftests.sh` |
| 内联 reftest | ✅ 685 个 | 13 个目录全部 ≥50，覆盖 CSS 2.1、Flexbox、Grid、Position、Display、Box、Float、Table、Multicol、Text、Fonts、Text-decor、Writing-modes |
| JS 执行 | ✅ 已集成 | reftest harness 通过 V8 sandbox 在渲染前执行 JS（不修改 DOM） |
| GPU 渲染截图 | ✅ 已验证 | GpuRenderer headless + read_pixels()；685/685 reftest GPU 模式 100.0% pass |
| GPU 渲染器图元 | ✅ 全量 | 全部 13 种图元管线 + 48 个单元测试 + 浏览器 GPU 路径集成 |
| CI 集成 | ✅ 已接入 | GitHub Actions reftest job（CPU 渲染） |
| Quirks Mode | ✅ 完成 | CSS parser + style system + layout engine quirks 全部实现 |
| 外部 stylesheet 加载 | ✅ 已贯通 | R213 落地：URL 导航路径 `fetch_url()` 现抓取 `<link rel="stylesheet">`（extract_stylesheet_hrefs → base URL 解析 → http_client 抓取 → 合并级联），三条 fetch_url 分支注入；离线 fixture HTTP server（R212）支撑测试 |
| 图片子资源/ImageCache | ✅ 已贯通 | R214（PNG 抓取+解码+image_cache）→ R215（浏览器 render_cpu/render_frame 消费 webview ImageCache，最后消费 hop）→ R216（JPEG）→ R218（SVG 栅格化统一到 render-foundation decode_image_bytes）。`<img>` 经 URL 导航全链路 fetch→decode→image_cache→browser render→真像素贯通（DC-13 P1 闭环） |
| 产品/真实静态页面视觉 smoke | 🔧 证据已持久化·持续修复 | welcome/morning.work/wintertc fixture + product-smoke + chromium Oracle 工具链就绪；**证据已持久化 `evidence/product-static/`**（3 fixture × {ZeroWeb-CPU/chromium PNG + README 含 diff%/根因}，满足 DC-13 line 305，R281 审计）；当前 diff（product-smoke 阈值0，与 morning/wintertc 同口径）：welcome **16.98%**（R371 的 14.68% 是 cross-validate chan>5 宽松口径，已纠正——ZeroWeb 渲染字节相同，仅 diff 阈值不同；R368 对 welcome 效果经一致口径复测为 17.01→16.98 可忽略，welcome 真实改善来自 R373 的 morning）、wintertc 13.59%（R227+R255 后 2026-06-18 复测 25→13.59）、morning-work fullpage 48.65%（R255 ua_default_display 修 4× 高度幻影盒 89.14%→48.65%）；残余 diff = item-tag span→block R109 IFC（结构性）+ fontdue CJK 度量 + hljs（需 JS），非证据缺口 |
| #[ignore] 测试 | ⚠️ 保留 | 59 个真实网站测试保留 #[ignore]，因本地网络不稳定。其余零 #[ignore] |

---

## Done Criteria 进度

### DC-1: WPT Reftest 基础设施就位

| 条目 | 状态 | 说明 |
|------|------|------|
| fetch 上游 WPT 仓库 | ⚠️ | 导入脚本已创建，内联 reftest 替代上游导入 |
| 解析 fuzzy() 元数据 | ✅ | manifest.rs 已扩展 |
| CPU 渲染截图 | ✅ | render_scene_to_framebuffer() 可用 |
| GPU 渲染截图 | ✅ | GpuRenderer headless + CPU 圆角叠加 |
| Chromium 参考截图 | ✅ | Puppeteer 脚本已创建（capture-chromium-screenshots.mjs） |
| Viewport 对齐 | ✅ | ReftestConfig 有 viewport 字段 + CLI --width/--height |
| JS 执行集成 | ✅ | V8 sandbox 在渲染前执行 JS（不修改 DOM） |
| 分类容差机制 | ✅ | ReftestCategory (Layout/Text/Unknown) + per-test fuzzy override |
| 范围外过滤 | ✅ | reftest-skip-list.txt 已创建 |
| 通过率报告 | ✅ | 文本 + JSON 格式，按分类输出 |
| 单一命令运行 | ✅ | `cargo run --bin zero-wpt-runner -- reftest` |
| CI 集成 | ✅ | GitHub Actions reftest job |

### DC-2: CSS 2.1 核心通过率 ≥ 95%

> ⚠️ **达标口径纠正（R283，2026-06-18）**：下表原「通过率 ≥95% ✅ 100.0%」基于**内联 685 reftest**，直接违反 DC-14（goal line 319「内联 reftest 100% 仅作 smoke，不计达标」+ line 844「禁止 DC-2~5 以内联 100% 冒充达标」= DONE 阻断项）。**真实达标**须 ≥95%（上游全量真实 reftest + chromium Oracle + 严格容差 0.1%/0.5%），当前诚实数 = **39.6% strict**（188/475，evidence/cross-validate-full-2026-06-17.txt）/ 90.2% self-source-loose（442/490 @ 1%/5%），**均 <95%，DC-2 未达标**。内联 100% 仅 smoke（DC-7 全绿基线）。

| 条目 | 状态 | 说明 |
|------|------|------|
| 导入 reftest 子集 ≥ 50 | ✅（smoke） | 179 个内联 CSS 2.1 核心 reftest（**不计达标分母**，DC-14 line 323） |
| 通过率 ≥ 95% | ❌ 未达标 | 内联 smoke 100%（179/179）不计达标；真实上游全量+chromium Oracle+严格容差 = 39.6% strict，未达 95% |
| CPU 模式达标 | ❌ 未达标 | 同上（reftest harness 走 CPU 路径，容差 10× 过松 R280，reference 同源自渲染） |
| GPU 模式达标 | ❌ 未达标 | GpuRenderer headless 可用（机制就绪），但真实通过率未达标 + 容差过松 |

### DC-3: Flexbox + Grid 通过率 ≥ 95%

> ⚠️ **达标口径纠正（R283）**：原「Flexbox/Grid 通过率 ✅ 100.0%」基于内联 reftest，违反 DC-14（line 319/844，内联不计达标）。真实达标须 ≥95%（上游全量真实 reftest + chromium Oracle + 严格容差），当前 39.6% strict 未达。内联 100% 仅 smoke。详见 DC-2 纠正说明。

| 条目 | 状态 | 说明 |
|------|------|------|
| Flexbox reftest 子集 | ✅（smoke） | 51 个内联 Flexbox reftest（基础+进阶+边界+M6 扩展，**不计达标分母**） |
| Flexbox 通过率 | ❌ 未达标 | 内联 smoke 100%（51/51）不计达标；真实上游全量+chromium Oracle+严格容差未达 95% |
| Grid reftest 子集 | ✅（smoke） | 51 个内联 Grid reftest（基础+进阶+边界+M6 扩展，**不计达标分母**） |
| Grid 通过率 | ❌ 未达标 | 同 Flexbox，内联 smoke 不计达标，真实未达 95% |
| CPU 模式达标 | ❌ 未达标 | 容差 10× 过松 R280 + 同源 reference，真实通过率未达标 |

### DC-4: Positioning + Float + Table + Multicol 通过率 ≥ 95%

> ⚠️ **达标口径纠正（R283）**：原「各项通过率 ✅ 全部 100.0%」基于内联 reftest，违反 DC-14（line 319/844，内联不计达标）。真实达标须 ≥95%（上游全量真实 reftest + chromium Oracle + 严格容差），当前 39.6% strict 未达；且 multicol/table 含已知结构性死锁（multicol-breaking R131、table colspan R177b 部分修），真实 sub-领域通过率更低。内联 100% 仅 smoke。详见 DC-2 纠正说明。

| 条目 | 状态 | 说明 |
|------|------|------|
| Positioning reftest | ✅（smoke） | 50 个定位 reftest（基础+进阶+M6 扩展，**不计达标分母**） |
| Float reftest | ✅（smoke） | 50 个 float 布局 reftest（M6 扩展，**不计达标分母**） |
| Table reftest | ✅（smoke） | 50 个 table 布局 reftest（M6 扩展，**不计达标分母**） |
| Multicol reftest | ✅（smoke） | 50 个 multicol 布局 reftest（M6 扩展，**不计达标分母**） |
| 各项通过率 | ❌ 未达标 | 内联 smoke 100% 不计达标；真实上游全量+chromium Oracle+严格容差未达 95%（multicol/table 结构性死锁更低） |
| CPU 模式达标 | ❌ 未达标 | 容差 10× 过松 R280 + 同源 reference，真实通过率未达标 |

### DC-5: 文字排版通过率 ≥ 95%

> ⚠️ **达标口径纠正（R283）**：原各目录「通过率 ✅ 100.0%」基于内联 reftest，违反 DC-14（line 319/844，内联不计达标）。真实达标须 ≥95%（上游全量真实 reftest + chromium Oracle + 严格容差），当前 39.6% strict 未达；且文字类容差 5%（R280）更过松，fontdue CJK 度量/line-height 噪声（R174/R187/R229b）是文字类残余 diff 大头。内联 100% 仅 smoke。详见 DC-2 纠正说明。

| 条目 | 状态 | 说明 |
|------|------|------|
| css-text/ reftest ≥ 50 | ✅（smoke） | 51 个（**不计达标分母**） |
| css-text/ 通过率 | ❌ 未达标 | 内联 smoke 100% 不计达标；真实上游全量+chromium Oracle+严格容差未达 95% |
| css-fonts/ reftest ≥ 50 | ✅（smoke） | 50 个（**不计达标分母**） |
| css-fonts/ 通过率 | ❌ 未达标 | 同上（fontdue 度量噪声是残余 diff 大头） |
| css-text-decor/ reftest ≥ 50 | ✅（smoke） | 50 个（**不计达标分母**） |
| css-text-decor/ 通过率 | ❌ 未达标 | 同上（text-emphasis 等未实现 R232） |
| css-writing-modes/ reftest ≥ 50 | ✅（smoke） | 50 个（**不计达标分母**） |
| css-writing-modes/ 通过率 | ❌ 未达标 | 同上（vertical-rl clearance R114/R164 死锁） |
| CPU 模式达标 | ❌ 未达标 | 容差 10× 过松 R280（文字类 5%）+ 同源 reference，真实通过率未达标 |

### DC-6: Quirks Mode

| 条目 | 状态 | 说明 |
|------|------|------|
| CSS parser quirks | ✅ | 已实现：quirky color values（hashless hex + numeric colors）、unitless lengths（裸数字视为 px） |
| Style system quirks | ✅ | 已实现：percentage-height quirk、table height quirk（height → min-height）、inline width/height quirk 注释 |
| Layout engine quirks | ✅ | table/float layout 已在 M4 实现，quirks mode 通过 UA 默认 display 值和 table height quirk 生效 |
| DOM → style 链路传递 | ✅ | Document::quirks_mode() → tag_name 提取 → apply_quirks_mode_adjustments |

### DC-7: 测试与质量

| 条目 | 状态 | 说明 |
|------|------|------|
| cargo test 零失败 | ✅ | 全部通过（59 个真实网站测试保留 #[ignore]） |
| 零 #[ignore] 测试 | ✅ | 仅 real_website_compat.rs 有 59 个 #[ignore] |
| 新修复有单元测试 | ✅ | quirks mode 颜色/长度/样式系统各新增单元测试 |
| cargo clippy 零警告 | ✅ | `cargo clippy -- -D warnings` 通过 |
| Reftest 报告持久化 | ✅ | evidence/reftest-report-2026-06-06.json/txt |
| 历史记录可追溯 | ✅ | 首份报告已持久化 |

### DC-8: CPU 渲染器图元覆盖（全部 13 种）

| 条目 | 状态 | 说明 |
|------|------|------|
| FillPrimitive | ✅ | 填充矩形（原有） |
| RoundedRectPrimitive | ✅ | 圆角矩形（原有） |
| GlyphPrimitive | ✅ | 文字渲染（原有） |
| GradientPrimitive | ✅ | 线性/径向/锥形渐变，逐像素插值 |
| ShadowPrimitive | ✅ | box-blur 近似阴影，含 blur_radius/spread_radius |
| ImagePrimitive | ✅ | RGBA 像素数据合成到 framebuffer |
| StrokePrimitive | ✅ | solid/dashed/dotted 线段 + LineCap |
| PathFillPrimitive | ✅ | 多边形扫描线填充 |
| PathStrokePrimitive | ✅ | 多边形描边 |
| TransformPrimitive | ✅ | 仿射变换后处理 |
| ClipPrimitive | ✅ | 矩形裁剪（像素级 discard） |
| FilterPrimitive | ✅ | blur + opacity + brightness/contrast/grayscale/invert/saturate/sepia/hue-rotate（apply_filter 全 8 color-matrix + blur，effects.rs；drop-shadow 仍 stub） |
| BlendModePrimitive | ⚠️ stub | draw_order 派发（cpu/mod.rs:250）但 `apply_blend_mode`（effects.rs:331-348）是 **no-op stub**（算 rect 后不应用，注释自承需 source+dest 双图层）——同 DC-9 blend（R278/R282 证），paint 生成但 CPU 不真混合。生产 footprint 低 |
| render_full_scene() 入口 | ✅ | 新函数，CSS painting order 渲染全部 13 种图元 |

### DC-9: GPU 渲染器图元覆盖

> **状态（R277 只读复核；2026-06-19 doc-maintenance 复核 committed HEAD b75035b 纠正：R285 transform / R286 5 color-matrix 滤镜均已落，原「transform WIP / 5 滤镜 GPU 丢弃」两项 stale）**：较 R211（2026-06-17 标 transform/clip/filter/blend 全 ⚠️「丢弃」）有实质推进。filter:opacity（f6fed44）/brightness/contrast（fc86937）/blur（3a3530f）已落地为独立 WGSL 后处理管线（ping-pong 区域读写，`render_full_scene_gpu` 经 `apply_color_filters_headless`/`apply_blur_filters_headless` 消费，非 passthrough，满足 DC-14）。clip 经 R220 实证为 no-op——engine 生产路径**从不生成** ClipPrimitive（`add_clip` 0 处非测试调用），overflow 裁剪在 paint 阶段由 `clip_all_primitives_to_rect` 预烘焙进图元几何，CPU/GPU 两路均空谈满足，**非真实缺口**。**真实剩余缺口 1 类（仅 blend_mode）**：(a) ~~transform~~ **✅ R285/ee8373a 已落**——`fs_transform`（逆变换重采样）+ `create_transform_pipeline` 已接入 `render_full_scene_gpu`（`collect_transforms`+`apply_transform_filters_headless`，guard 非空；单测 `test_gpu_full_scene_transform_translation`，匹配 CPU `apply_transform_post` clear-to-white 语义）；(b) **blend_mode**——paint 在 `painter/effects.rs:313` 生成 `BlendModePrimitive`，但 **CPU `apply_blend_mode`（effects.rs:331-348）是 no-op stub**（算 rect 后 `_=(left,top,right,bottom)` 仅消未用警告），GPU `render_full_scene_gpu` 同样不消费，需 source+dest 双图层新机制（R269 标记为比 opacity 大的独立特性，低 reftest footprint）；(c) ~~5 color-matrix 滤镜~~ **✅ R286/94c773a 已落**——`collect_color_filters`（renderer/mod.rs:2062）现处理全 8 mode（Opacity0/Brightness1/Contrast2/Grayscale3/HueRotate4/Invert5/Saturate6/Sepia7），parity CPU `apply_filter`，单测 `test_gpu_full_scene_filter_grayscale`/`_hue_rotate`/`_invert`。drop-shadow（CPU 亦 stub，GPU 同）仍 `_ => None` 丢弃。reftest harness 与 product-smoke 均走 CPU 路径，GPU 缺口不污染测量数字，仅影响浏览器 GPU 渲染模式。

| 条目 | 状态 | 说明 |
|------|------|------|
| FillPrimitive | ✅ | GPU 填充（原有） |
| GlyphPrimitive | ✅ | GPU 文字渲染（原有，atlas） |
| RoundedRectPrimitive | ✅ | GPU 片段着色器（WGSL corner discard） |
| GradientPrimitive | ✅ | GPU 渐变 shader（线性/径向/锥形 + 1D 渐变纹理） |
| ShadowPrimitive | ✅ | 半透明填充矩形（简化，不做 GPU blur） |
| ImagePrimitive | ✅ | GPU 纹理上传 + 采样（RGBA→texture→shader） |
| StrokePrimitive | ✅ | CPU 侧顶点生成 + GPU fill pipeline（solid/dashed/dotted） |
| PathFillPrimitive | ✅ | CPU 侧扇形三角化 + GPU fill pipeline |
| PathStrokePrimitive | ✅ | CPU 侧分解为粗线段 + GPU fill pipeline |
| TransformPrimitive | ✅ | R285（ee8373a）独立 WGSL `fs_transform`（逆变换重采样，匹配 CPU `apply_transform_post` clear-to-white 语义）+ `create_transform_pipeline`，**已接入** `render_full_scene_gpu`（`collect_transforms` + `apply_transform_filters_headless`，guard `!empty && headless_texture.is_some()`）；单测 `test_gpu_full_scene_transform_translation` |
| ClipPrimitive | ⚪ no-op | engine 生产路径**从不生成** ClipPrimitive（R220 实证），overflow 裁剪预烘焙进图元几何。CPU/GPU 两路均空谈满足，**非真实缺口** |
| FilterPrimitive | ✅（drop-shadow 除外） | **全 8 color-matrix + blur 已落（独立 WGSL ping-pong 后处理，非 passthrough）**：opacity（fs_color_filter mode0）/brightness（mode1）/contrast（mode2，fc86937）/grayscale（mode3）/hue-rotate（mode4）/invert（mode5）/saturate（mode6）/sepia（mode7，R286/94c773a，parity CPU `apply_filter`）/blur（fs_blur 三角核 2-pass，3a3530f）。`collect_color_filters`（mod.rs:2062）现处理全 8 mode（原「mode 0/1/2 其余丢弃」已过时）。**仍未落**：drop-shadow（CPU 亦 stub，GPU `_ => None`） |
| BlendModePrimitive | ❌ 丢弃 | paint 生成（effects.rs:313）但 CPU `apply_blend_mode`=no-op stub（effects.rs:331-348）+ GPU `render_full_scene_gpu` 不消费。**单 framebuffer post-process 架构上不可行**（R278 实证：apply 时元素子树已与 backdrop 合并进 framebuffer、不可分离，区别于 opacity/blur 的合法区域近似）→ 需 **paint-isolation 架构**（元素子树隔离渲染到 offscreen + source/dest 双纹理 blend 合成 pass）；render-foundation 现无 per-element staging buffer、paint 无 isolation group，**multi-round 架构 defer**。footprint ~2-4 case，非 lever |

> **DC-9/DC-14 parity caveat（R277）**：覆盖满足 ≠ CPU 像素 parity——(1) opacity=GPU RGB-darken 近似（R272，post-process 无法恢复背景）；(2) blur=GPU 三角核 separable 2-pass vs CPU 多遍 box（R277，算法分歧，非 ==CPU，见 `evidence/r277-dc9-gpu-blur-vs-cpu-boxblur-parity-2026-06-18.txt`）；(3) brightness/contrast=精确 parity（R273 正确 CSS 语义）。三者覆盖均达标（独立 WGSL 非丢弃），但 opacity/blur 属「覆盖达标非像素对齐」类。

### DC-10: 浏览器图元消费

| 条目 | 状态 | 说明 |
|------|------|------|
| transform_webview_primitives() 全 13 种 | ✅ | 新函数处理所有 RenderPrimitives 字段 |
| render_cpu() 使用 render_full_scene() | ✅ | 完整图元渲染替代旧版 3 种入口 |
| scale_factor 应用 | ✅ | 所有图元类型正确缩放 |
| offset 应用 | ✅ | 所有图元类型正确偏移 |
| clip_y 视口裁剪 | ✅ | fills + glyphs 应用 clip_y 裁剪 |
| CSS painting order | ✅ | shadows → backgrounds → borders → content → overlay → filters → blend_modes |

### DC-11: M7 验证

| 条目 | 状态 | 说明 |
|------|------|------|
| cargo test 全绿 | ✅ | 7800+ 测试全部通过 |
| cargo clippy 零警告 | ✅ | `cargo clippy -- -D warnings` 通过 |
| 新增图元单元测试 | ✅ | 渐变/阴影/图片/线段/路径填充/路径描边/变换/裁剪/滤镜/混合模式各有独立测试 |

---

## M1 里程碑详情

**目标**: 建立能够导入、运行、对比和报告 WPT reftest 的完整基础设施。

### M1 完成标准 (14 项)

1. ✅ fetch 上游 WPT 仓库（导入脚本 + 内联 reftest 替代）
2. ✅ 扩展 manifest.rs 解析 fuzzy() 元数据
3. ✅ CPU 软件渲染截图（render_scene_to_framebuffer）
4. ✅ GPU 渲染截图（GpuRenderer headless + CPU 圆角叠加）
5. ✅ 自动化 Chromium 截图工具（Puppeteer 脚本）
6. ✅ Viewport 对齐机制
7. ✅ JS 执行集成（V8 sandbox 执行 script 标签中的 JS）
8. ✅ 分类容差机制
9. ✅ 范围外 reftest 过滤 (skip list)
10. ✅ 按目录分类通过率报告（文本 + JSON）
11. ✅ 单一命令运行全部 reftest
12. ✅ 导入 CSS 2.1 核心 ≥ 50 个 reftest（115 个）
13. ✅ 记录初始通过率（100.0% 113/113）
14. ✅ 确认 #[ignore] 标记状态

### M1 已完成的基础设施

| 组件 | 文件 | 说明 |
|------|------|------|
| Manifest 解析 | `tests/wpt-runner/src/manifest.rs` | reftest 条目、fuzzy 元数据、HTML 链接提取 |
| Reftest 引擎 | `tests/wpt-runner/src/reftest.rs` | 分类容差、fuzzy 覆盖、match/mismatch 比较 |
| Reftest 数据 | `tests/wpt-runner/src/reftest_data.rs` | 159 个 CSS 2.1 核心 + Flexbox/Grid 内联 reftest |
| Reftest CLI | `tests/wpt-runner/src/main.rs` | `reftest` 子命令 + 文本/JSON 报告 |
| Skip List | `tests/wpt-runner/reftest-skip-list.txt` | SVG/Canvas/WebGL/动画过滤规则 |
| Chromium 工具 | `tests/wpt-runner/scripts/capture-chromium-screenshots.mjs` | Puppeteer headless 截图 |
| 导入脚本 | `tests/wpt-runner/scripts/import-wpt-reftests.sh` | 上游 WPT reftest 批量导入 |

---

## 初始 Reftest 通过率数据（M6 inline，不计达标 — 已归档）

> 2026-06-07 M6 基线：685 内联 reftest 100% 通过（CPU 软件渲染，800×600）。该 685 内联 reftest 自 DC-14 起明确**不计达标分母**（goal line 323/844；DC-2~5 各节均标「内联 smoke 100% 不计达标」），100% 仅作 smoke。逐目录/覆盖范围明细已迁出至 [`archive/m6-inline-reftest-baseline-2026-06-07.md`](./archive/m6-inline-reftest-baseline-2026-06-07.md)。**达标口径真基线见下节「上游真实 WPT Reftest 通过率」**（self-source 443/490 / chromium-Oracle ~42%）。

---

## 上游真实 WPT Reftest 通过率

> 早期上游 reftest 调查（R11–R20，2026-06-09/10，self-source 基线 74.7%）已归档至 [`archive/rounds-r11-r20-reftest-investigation.md`](./archive/rounds-r11-r20-reftest-investigation.md)。

**当前基线（R323 复验；strict post-R326 再复验仍零漂移，见 [`evidence/strict-baseline-reverify-post-r326-2026-06-19.txt`](./evidence/strict-baseline-reverify-post-r326-2026-06-19.txt)）**：

- self-source loose **443/490 (90.4%)** @ 默认 1%/5% 容差
- self-source strict **295/490 (60.2%)** @ 锁定 0.1%/0.5%（DC-14 真通过口径）
- chromium-Oracle 真一致率 **~35.6%**（self-source 含 48.6% 假通过）

完整 plateau 分析、已穷尽杠杆表、4 条多会话路径见顶部「综合裁决」节；逐目录 chromium-Oracle 污染分布见 `evidence/cross-validate-full-2026-06-18.txt`（flexbox 26% 污染最诚实，writing-modes 73% 最高）。达标需多会话架构，单会话杠杆已穷尽。

---

## 已知关键缺口（当前活跃）

> 下表仅列**尚未解决**的缺口，与 R305–R323 plateau 框架对齐（剩余 forward motion 全部需多会话架构承诺，见顶部「综合裁决」）。**已完成项**（Float/Table/Multicol 布局算法、OpenType shaping、BiDi、CJK 换行、justify、quirks mode、CPU/GPU/浏览器图元覆盖、外部 stylesheet、图片子资源/ImageCache、margin 折叠、BFC 检测+margin 隔离、`<img>` intrinsic sizing + object-fit）见「里程碑完成状态」「当前状态概览」，不再在此重复列。

| 缺口 | 影响范围 | 优先级 | 解锁路径（均为多会话架构） |
|------|----------|--------|------|
| **Paint IFC / taffy-IFC 架构分裂** | large-font（ifc-008/009/011）+ welcome/morning 文本度量残余（self-source 失败主因） | **P0** | Phase A IFC 统一：baseline-resolved 单一权威行盒（[`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)；墙②③ R125–R213 六轮死锁 + R306 几何基线证伪） |
| Multicol column breaking / 嵌套碎片化 | css-multicol 失败聚类（结构性） | P1 | Phase 2 嵌套 multicol fragmentation（layout 侧 column-aware IFC；paint 侧 R157/R198/R203/R122/R317 五轮证 net-negative 死路） |
| Multicol / flexbox baseline-export | baseline-000~008 + flexbox-baseline（~10+ 案） | P1 | taffy 0.8+ `baseline_overrides`（R304 DEFER 升级）或自建 inline-level-box baseline 合成（R266/R313/R316 三机制穷尽，须 layout 侧注入） |
| Writing-mode 垂直布局 | css-writing-modes 垂直 float/clearance 轴 | P1 | 精细轴交换（R57/R114/R164 谱系，clearance vertical-axis） |
| Inline-box 模型 | CSS2 linebox（vertical-align/行盒高度） | P1 | 与 Phase A IFC 统一耦合（v_offset/baseline 语义分歧） |
| DC-9 blend_mode（mix-blend-mode） | GPU backdrop 合成（~2-4 reftest 案，近零覆盖） | P2 | paint-isolation 架构（offscreen 子树渲染 + source/dest 双纹理 blend pass，R278 defer） |
| position: sticky | 滚动吸附（需宿主层输入路由） | P2 | host-runtime 层 sticky 偏移驱动；**R326 实测**：converter 把 sticky 映射为 taffy Relative，block-level 偏移已被 taffy 应用（scroll-0 应吸附场景 delta==inset）。缺的是 scrollport 相对钳制（normal 位满足 inset 时应 == static，当前 == relative），属架构性 |
| 产品 smoke 文本度量残余 | welcome 17% / wintertc 14% / morning 49% | P1（与 Phase A 同源） | Phase A IFC 统一（item-tag R109 inline→block + system-ui 字体度量，非图片/CSS 缺口） |
| WebP 解码 + CSS `url()` 背景图抓取 | 图片子资源残余（wintertc/morning 不用，低 ROI） | P3 | `decode_image_bytes` 扩 WebP + `fetch_image_subresources` 扩 `url()` |

### DC-11 doc 复核（2026-06-19，read-only 代码核查；无代码/reftest 变更）

承接 R323 margin 折叠探针（纠正 goal doc「未实现」）与 R324 position:fixed 修复（commit 5b11fc2）后，本轮 read-only 核查 DC-11「布局正确性」其余项是否如 goal doc 声称的「未实现」。逐项代码核查 + 生产接线验证：

| 项 | goal doc 旧声明 | 代码实证 | 裁决 |
|----|----------------|---------|------|
| BFC | 「无 BFC 概念，overflow:hidden 不隔离浮动/不阻止 margin 折叠」 | `establishes_bfc`（margin_collapse.rs:33-76）全条件（overflow/float/abspos/flow-root/flex/grid/table/multicol）**接线生产**（engine.rs:2940/2988/3052）；`use_bfc_float_containment`（engine.rs:2992）落地 float containment；margin 隔离 R323 实测 6 探针全过 | **过时**（待 goal doc 纠正；并行 agent R324 note 标 BFC float containment 为下一调查项） |
| `<img>` intrinsic + object-fit | 「无固有尺寸，object-fit 在 paint 阶段处理但无实际图片数据」 | `apply_replaced_element_sizing`（tree.rs:165，HTML width/height 属性 + SVG data URI + 解码固有尺寸）+ `compute_object_fit_rect` 全 5 值（Fill/Contain/Cover/None/ScaleDown，text.rs:1582，img paint site text.rs:614 调用）+ R318 图片数据端到端贯通 | **过时**（待 goal doc 纠正；并行 agent R324 note 标 object-fit 为下一调查项） |
| 滚动容器 | 「无真正滚动容器，浏览器层手动偏移」 | `scroll_x/scroll_y` 字段 + paint 偏移（painter/mod.rs:465-471）+ overflow 裁剪（needs_clip/clip_all_primitives_to_rect，mod.rs:197/298）；app 层 scroll_offset per tab + wheel 路由 | **基本准确**（paint 偏移+裁剪已落地，非 layout 级真滚动容器；master.md 已如实标「简化处理」）→ 不改 |
| position: sticky | 「需宿主层动态调整」 | `is_sticky` 标记落地（engine.rs:606）。**R326 实测纠正**：converter（converter/mod.rs:286）把 `Sticky` 映射为 taffy `Position::Relative`，故 block-level sticky 偏移**已被 taffy 应用**（scroll-0 应吸附场景 delta==inset，新单测 `test_sticky_applies_inset_like_relative_at_scroll_zero` 实证）。旧注「偏移未应用」源于 `engine.rs:1948` 死代码（`#[allow(dead_code)]` 的 `apply_relative_offsets`）注释，非生产路径。缺的是 **scrollport 相对钳制**（normal 位满足 inset 时应 == static，当前渲染 == relative）——属架构性，非单点修复 | **部分过时**（R326 已纠正「偏移未应用」为「已应用、缺 scrollport 钳制」） |
| position: fixed | 「当前错误地映射为 absolute」 | `adjust_fixed_to_viewport`（engine.rs:2176）存在且调用；**R324（commit 5b11fc2）已修 fixed-inside-positioned-ancestor over-correction（`+=`→`-=`）** | R324 已处理（goal doc 纠正见并行 agent 提交） |

**结论**：goal doc DC-11 的 BFC + object-fit 两项「未实现」声明与代码现实矛盾（governance §1 自洽）。本轮将核查结论沉淀于本表（避免与并行 agent 活跃编辑 rendering-compat.md 冲突）；goal doc prose 纠正**已由 R325 执行**（BFC known-gaps line 378 + 替换元素 DC-11/support-envelope/known-gaps 三处，按 R323/R324 先例）。scroll/sticky 声明准确不改。本轮零代码变更（仅 read-only 核查）。

## IFC 统一技术参考（R69+ 代码级上下文 — 已归档）

> 三套 IFC 运行路径（measure / remeasure / paint）、paint-IFC override 覆盖缺口、R37–R68 已穷尽不可行路径表、存储 vs paint 基线差异、完成度清单、Taffy Fork 状态等代码级细节已迁出至 [`archive/r69-ifc-unification-technical-reference.md`](./archive/r69-ifc-unification-technical-reference.md)（无入站引用，归档前核查）。现代 Phase A 规划见 [`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)；当前 plateau 结论见顶部「综合裁决」表。

---

## IFC 之外的其他卡点（R68 时代前置 plateau 框架 — 已归档，保留锚点 stub）

> 本节为 R68 时代（pre-plateau，~320 轮前）的卡点分析框架，详细「影响/当前能力/缺失/关键失败测试/技术方向」+「卡点依赖关系/R69+ 推荐优先序」**已迁出至** [`archive/r68-other-blockers-framework.md`](./archive/r68-other-blockers-framework.md)。多数卡点已被顶部「综合裁决」表 + 「已知关键缺口」表以更准确的多会话架构结论取代；保留下列锚点摘要供 `multicol-phase2-unified-column-flow-spec.md` 等文档的「卡点 #N」引用解析（避免 dangling pointer）。

- **卡点 #2 Multicol Column Breaking**（~22 测试）：内容碎片化缺失——超高块级子需跨列拆分。→ 现代结论见综合裁决「Phase 2 嵌套 multicol fragmentation」+ [`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md)。
- **卡点 #3 Writing-mode 垂直布局**（~10 测试）：垂直模式 float/clearance 轴交换。→ 综合裁决 R114/R164 谱系。
- **卡点 #4 Flexbox Baseline 对齐**（~3-5 测试，独立）：taffy 仅 ≥2 baseline 子才算基线。→ 综合裁决「baseline-export」（独立卡点，与 multicol Phase 2 解耦）。
- **卡点 #5 Table Border-collapse 精度**（~3 测试）：外边缘单元格边框减半。→ R177b 部分修。
- **卡点 #6 CSS 2.1 App E 堆叠顺序**（2-3 测试）：position:relative 后代 tree-order 排序。→ **R380 ruled out**（net-negative 回退）。
- **卡点 #7 Grid Max-content Sizing**（2-3 测试）：taffy grid max-content。→ R97/taffy-blocked（R304 DEFER）。
- **卡点 #8 Swatch 图像缩放精度**（~5 测试）：纯色 PNG 双线性伪影。→ niche。
- **卡点 #9 Position Fixed 视口定位**（1-2 测试）：→ **✅ R324/R98 已修**（`adjust_fixed_to_viewport`）。

---

## 技术决策记录

| 日期 | 决策 | 理由 |
|------|------|------|
| 2026-06-06 | 保留真实网站测试的 #[ignore] | 本地网络不稳定，这些测试不可执行 |
| 2026-06-06 | 扩展而非重写 manifest.rs 和 reftest.rs | 目标文档明确要求扩展现有模块 |
| 2026-06-06 | 使用内联 reftest 替代上游导入 | 避免网络依赖，53 个 CSS 2.1 核心 reftest 覆盖主要布局场景 |
| 2026-06-06 | mismatch 阈值设为 0.5% | 800×600 视口下，50×50 小元素差异约 0.52%，1% 阈值会漏检 |
| 2026-06-06 | 文字类 reftest 使用宽松容差 (5%/15ch) | fontdue vs Skia 字体渲染像素差异大 |
| 2026-06-06 | QuirksMode 在 StyleSystem 内部传递（不暴露为公开参数） | 保持公共 API 简洁，doc.quirks_mode() 在 compute_styles 入口处提取 |
| 2026-06-07 | quirks mode 颜色/长度解析通过函数指针分发 | parse_color_fn/parse_length_fn 模式避免重复 match 分支 |
| 2026-06-07 | apply_quirks_mode_adjustments 接受 tag_name 参数 | 需要按元素标签（如 table）应用不同的 quirks 规则 |
| 2026-06-07 | inline 元素 width/height quirks 暂不实现 | layout engine 将 inline 映射为 block，实际已生效；待 inline layout 正确实现后补充 |
| 2026-06-07 | UA 默认 display 值通过级联注入（Origin::UserAgent） | 最低优先级，可被作者样式覆盖；避免修改 ComputedStyle::default() |
| 2026-06-07 | Table 布局通过后处理步骤实现（类似 float） | taffy 无原生 table 支持，所有 table display types 映射为 Block 后重新定位 |
| 2026-06-07 | 修复 parse_display 缺失 table types 的 bug | color.rs 中有重复的 parse_display（缺 table types），通过 pub use color::* 被实际使用 |
| 2026-06-07 | Multi-column 通过后处理步骤实现（类似 float/table） | taffy 无原生 multicol 支持，column-count/column-width 容器的子元素在后处理中重新定位到各列 |
| 2026-06-07 | 多列均衡分配使用 shortest-column-first 策略 | 依次将每个子元素放入当前总高度最小的列，实现视觉均衡 |
| 2026-06-07 | CJK normal 模式下每个字符单独作为"单词" | CSS 规范要求 CJK 允许任意字符间断行，split_into_words 中 CJK 字符独立为词 |
| 2026-06-07 | text-align: justify 使用 effective_content_area 计算剩余空间 | 修复了原先用 container_width 忽略 float exclusion 的问题 |
| 2026-06-07 | Float exclusion 从 max 改为 additive stacking | 多个同侧 float 应累加宽度而非取最大值 |
| 2026-06-07 | rustybuzz 集成到 TextShaper | 优先使用 rustybuzz 进行 OpenType shaping（GSUB/GPOS），回退到 fontdue 逐字符映射 |
| 2026-06-07 | unicode-bidi 集成到 inline layout | RTL 字符自动检测并重排序，LTR 文本零开销 |
| 2026-06-07 | FontLoader 存储原始字体字节 | 供 rustybuzz Face::from_slice 使用，fontdue 仍用于 advance width 获取 |
| 2026-06-07 | ShapedGlyph 增加 x_offset/y_offset 字段 | 来自 rustybuzz 的 GPOS 定位偏移 |
| 2026-06-07 | 新增 render_full_scene() 替代 render_scene_to_framebuffer() | 旧函数仅支持 fills + rounded_rects + glyphs，新函数支持全部 13 种图元 |
| 2026-06-07 | 新增 transform_webview_primitives() 替代 inline 坐标变换 | 旧方式仅处理 fills + glyphs，新函数处理全部 13 种 RenderPrimitives 字段 |
| 2026-06-07 | 渐变使用逐像素插值 | 线性/径向/锥形渐变均在 CPU 上逐像素计算，无 GPU 依赖 |
| 2026-06-07 | 阴影使用 box-blur 近似 | 三次 box-blur 近似高斯模糊，性能与质量平衡 |
| 2026-06-07 | 路径填充使用扫描线算法 | 逐行扫描多边形边界，奇偶规则填充 |
| 2026-06-07 | CPU 后处理：Transform/Clip/Filter/BlendMode 作为后处理步骤 | 像素级后处理，不依赖 GPU；GPU 渲染器需独立实现 |
| 2026-06-07 | GPU 渲染器多管线架构 | 5 条独立 wgpu 渲染管线：Fill+Glyph、RoundedRect、Gradient、Image、Blur。每种管线有独立 WGSL shader 和绑定组布局。Mesh-based 图元（stroke/path）通过 CPU 侧顶点生成复用 fill pipeline。Phase-separated 架构避免借用冲突。 |
| 2026-06-07 | 浏览器 GPU 路径集成 render_full_scene_gpu | render_frame() 改用 render_full_scene_gpu 替代 render_scene_ext，GPU 渲染路径现在支持全部 13 种图元。GPU 渐变测试使用 ±3 容差应对 float→u8 精度误差。 |
| 2026-06-07 | Taffy 0.7 内置 margin 折叠 | 发现 taffy 0.7 已通过 CollapsibleMarginSet 实现 CSS 块级 margin 折叠，不需要额外后处理步骤。移除了自实现的 margin_collapse 后处理。 |
| 2026-06-07 | Float clear 后处理实现 | 在 adjust_float_positions() 中实现 clear:left/right/both。非 float 元素的 clear 属性将其推到对应侧浮动元素的底部之下。LayoutBox 新增 clear 字段。 |
| 2026-06-07 | 重复渐变：fract() tiling | CPU 渲染器用 fract(t/period) 实现重复渐变周期循环；GPU 渲染器通过 WGSL shader 中 fract(t) 实现，repeating 标志通过 param3 取负编码传递。 |
| 2026-06-07 | 多图层背景 Vec 迁移 | background_image 从 BackgroundImageComputedValue 单值改为 Vec，CSS 解析器新增 parse_background_image_layers() 处理逗号分隔，paint 按逆序渲染（CSS 规范最后一层在最底）。 |
| 2026-06-07 | clip-path inset 实际裁剪 | 新增 clip_all_primitives_to_rect() 对全部图元类型（fills/rounded_rects/gradients/shadows/images/glyphs/strokes）应用矩形裁剪，替代原来的虚线指示器。 |
| 2026-06-07 | CSS mask 渐变蒙版 | mask-image 复用 BackgroundImageValue 类型解析，渐变蒙版通过 clip_all_primitives_to_rect 裁剪到渐变边界 + 平均 alpha 衰减实现。URL 蒙版暂不支持（需图像加载基础设施）。 |
| 2026-06-07 | overflow 全图元裁剪修复 | paint_node/paint_node_in_rect 中 overflow:hidden/scroll/clip 原来仅裁剪 fills+glyphs，渐变/阴影/图片/线段等图元溢出容器边界不被裁剪。改为使用 PrimitiveCounts + clip_all_primitives_to_rect 裁剪全部 13 种图元类型。 |
| 2026-06-07 | 滚动容器 paint 偏移 | LayoutBox 新增 scroll_x/scroll_y 字段。paint_node 中当 overflow == Scroll 时，子元素坐标减去 scroll 偏移量。overflow:Hidden 不应用滚动偏移（非滚动容器）。3 个单元测试验证。 |
| 2026-06-07 | render_full_scene 切换到上游 reftest | 上游 reftest 从旧版 render_scene_to_framebuffer（仅 fills+rounded_rects+glyphs）切换到 render_full_scene（全部 13 种图元）。同时启用 ImageCache 从 base_dir 加载 PNG 图片。这使 reftest 结果更准确，但也暴露了之前被不完整渲染掩盖的布局差异。 |
| 2026-06-07 | skip_indicators 模式 | Painter 新增 skip_indicators 标志，RenderPipeline 新增 set_skip_indicators() 方法。当设为 true 时跳过全部 ~30 个 CSS 属性调试指示器（border-collapse 橙色标记、direction 箭头等），避免干扰 reftest 像素对比。 |
| 2026-06-07 | UA 默认样式扩展 | 新增 body{margin:8px}、h1-h6{margin+font-weight}、p{margin:1em 0}、ul/ol{margin+padding-left} UA 默认样式，对齐浏览器默认行为。 |
| 2026-06-08 | table row-group 行索引修复 | build_grid 中行组内行存储 rg_child_idx 但 get_row_box 在 table_box.children 查找，导致 tbody/thead/tfoot 内的行被静默丢弃。修复：TableRow 新增 row_group_index 字段，get_row_box/position_cells 根据此字段正确导航到行组内的行。 |
| 2026-06-08 | column-gap 属性映射修复 | converter gap.width 原先使用 style.gap（仅 gap 简写设置），改为使用 style.column_gap（column-gap 长写属性）。同时修复 gap 简写解析：`gap: 10px` 现在同时设置 column_gap 和 row_gap。使用 fallback 策略：column_gap 非 0 时优先，否则使用 gap。 |
| 2026-06-08 | background-image 固有尺寸基础设施 | Painter 新增 image_sizes HashMap<u64, (f32, f32)>（url hash → intrinsic dimensions）。RenderPipeline.set_image_sizes() 将缓存传递给 Painter。reftest runner 在渲染前构建 ImageCache、提取固有尺寸。修复了 background-size: auto 拉伸到容器大小的问题。 |
| 2026-06-08 | is_block_level / is_relative 标志 | LayoutBox 新增两个布尔标志。is_block_level 用于 float/clear 后处理（CSS 规范 clear 仅适用于块级元素）。is_relative 用于 table 布局后处理保留 position:relative 的 inset 偏移。 |
| 2026-06-08 | gap 简写 handler 修复 | gap apply handler 不再设置 column_gap/row_gap（由各自的 longhand handler 通过 shorthand expansion 设置），避免 HashMap 迭代顺序不确定性导致的值覆盖。 |
| 2026-06-09 | reftest 分类容差 bug 修复 | 上游 reftest FileReftestCase::to_config() 使用 Default::default()（1%/5ch），未调用 ReftestConfig::for_category()。所有测试被以严格布局容差（1%）衡量，导致文字类测试（5% 容差）大量误判失败。修复后通过率 68.6%→73.5%（+24 测试）。新增 ReftestConfig::with_viewport() builder 方法。 |
| 2026-06-09 | columns 简写解析修复 | `columns: 3`（单整数）被 parse_column_width 先解析为 column-width: 3px，阻止 parse_column_count 执行。CSS 规范要求整数优先解析为 column-count。交换解析顺序后，`columns: N` 正确设置 column-count: N。 |
| 2026-06-09 | 零高度浮动处理 | adjust_float_positions Phase 1 中 line_max_height 跳过零高度浮动元素（child_outer_height == 0），避免空浮动元素推进后续浮动的 Y 位置。 |
| 2026-06-08 | table 行组位置更新 | position_cells 后新增 update_row_group_positions 后处理。按视觉顺序（thead→tbody→tfoot）计算行组的 y 位置和高度，含 border-spacing。支持 position:relative inset 从行组传播到子行。修复 out-of-order-elements-collapsed-border（46.32%→通过）。 |
| 2026-06-08 | CSS 绝对长度单位 | parse_length() 新增 in/pt/pc/cm/mm/Q 单位支持，按 CSS 规范转换为 px（96 DPI）。修复了所有使用 `height: 1in; width: 1in` 的 floats-clear 测试（之前 in 单位被静默忽略，元素折叠为 0 大小）。副作用：CSS2/borders 中使用 1in(=96px) 大边框的测试暴露了布局精度差异。 |
| 2026-06-08 | CSS inherit 关键字完善 | border/background shorthand 正确广播 CSS-wide keywords（inherit/initial/unset）到所有子属性。inherit_property 扩展支持非继承属性（background-*, border-*, margin-*, padding-*），使 `border-bottom: inherit` 等显式继承生效。 |
| 2026-06-08 | is_block_level 修正 | table 内部 display types（TableRowGroup, TableRow, TableCell 等）从 is_block_level 中移除。CSS 2.1 规定 clear 属性仅适用于块级元素，table 内部元素不是块级元素。 |
| 2026-06-08 | 参考文件过滤 | reftest loader 跳过以 -ref/-reference 结尾的文件名，避免参考页面被当作测试用例运行。移除 1 个误计入的测试（float-nowrap-3-ref.html）。 |
| 2026-06-08 | XHTML CDATA 调查 | 调查发现 html5ever 在 HTML 模式下将 XHTML CDATA 标记（`<![CDATA[...]]>`）保留在 `<style>` 文本内容中。CSS 解析器遇到 `<![CDATA[` 时错误恢复路径触发 `skip_to_rbracket()`，贪婪吞噬后续所有 token，导致整个样式表提取 0 条规则。之前通过 CDATA 损坏的 .xht 测试（test+ref 都无 CSS）实际是虚假通过。 |
| 2026-06-08 | XHTML CDATA 清理实施 | `strip_cdata()` 在 `collect_stylesheets()` 中去除 CDATA 前后缀。揭示真实通过率 66.1%（之前 76.4% 含虚假通过）。真实修复：background-087/326/328 ✅。揭示的渲染缺口：writing-modes 42.4%（需 writing-mode 布局支持）、multicol 49.1%（需 column breaking）、floats-clear 新增 6 个差异。 |
| 2026-06-08 | empty-cells border-collapse 修复 | `empty-cells: hide` 仅在 separated border model 中生效。在 collapsed border model 中，空单元格仍需显示边框。修改 paint_node 两处 skip_empty_cell 条件添加 `border_collapse == Separate` 检查。 |
| 2026-06-08 | row-group/row box model 抑制 | CSS 2.1 Section 17.5.3/17.5.4：在 separated border model 中，table-row-group 和 table-row 的 border/padding/margin 无视觉效果。新增 `suppress_row_group_row_box_model()` 和 `zero_box_model()` 函数。 |
| 2026-06-08 | table cell explicit height 保留 | 有明确 height 且 overflow:hidden/scroll/clip 的单元格保持 taffy 计算的原始高度，不被行高覆盖。修复 table-cell-overflow-explicit-height 测试。 |
| 2026-06-08 | CSS 2.1 Appendix E 绘制顺序 | paint_node_in_rect 和 paint_node 中子元素分两轮绘制：先绘制非 float 子元素，再绘制 float 子元素。确保 float 内容视觉上在 block 背景之上（CSS 2.1 Appendix E）。 |
| 2026-06-08 | columns 简写顺序无关解析 | expand_columns() 双值模式改为自动检测哪个是整数（column-count）哪个是长度（column-width），而非硬编码 parts[0]/[1]。修复 `columns: 100px 6` 等逆序声明。 |
| 2026-06-08 | clearance 计算代码质量改善 | 澄清 CSS 2.1 §9.5.2 clearance 语义：零 clearance 仍然阻止 margin 折叠；clearance = max(0, clear_bottom - hypothetical_position)。后处理方式的局限性在于 taffy 已应用 margin 折叠。 |
| 2026-06-08 | 空 inline 元素 line-height | 空 inline 元素（如 `<span></span>`）生成零宽度 TextRun，其 line-height 仍贡献到行盒高度。修改 collect_inline_items 不再跳过空 inline 元素。 |
| 2026-06-08 | sibling combinators 文本节点跳过 | NextSibling (+) 和 SubsequentSibling (~) 组合器现在跳过元素间的文本节点，匹配 CSS 选择器规范行为。修改 matches_selector_recursive 和 matches_has_selector_chain。 |
| 2026-06-08 | CSS 绝对长度单位 | parse_length() 新增 in/pt/pc/cm/mm/Q 单位（96 DPI），background 简写分类器新增所有长度后缀。修复使用 1in 高度的 floats-clear 测试。 |
| 2026-06-08 | 径向渐变位置修复 | gradient_to_primitive 改用 resolve_position() 正确处理 Percentage（百分比）和 Px（绝对像素），替代旧的 length_to_f32/100 逻辑。修复相关测试用例。 |
| 2026-06-08 | 表格 min-height border-box | apply_table_size_constraints 正确处理 min-height/max-height 为 border-box 约束（减去 padding+border）。修复 min-height-table。 |
| 2026-06-08 | 表格单元格高度最小值 | CSS 2.1 规范中 cell height 为最小高度，改用 max(row_height, cell_content_height)。 |
| 2026-06-14~17 | R118–R227 逐轮技术决策 | 已归档至 [`archive/tech-decisions-r118-r227.md`](./archive/tech-decisions-r118-r227.md)（R118–R139 同见 `rounds-r23-r139.md`，R142–R227 同见 `rounds-r142-r302.md`）；当前 plateau 结论与 ruled-out 杠杆见顶部「综合裁决」 |


---

## 下一步

> R305–R323 已确认结构性 plateau（见上方「综合裁决」）。下列为**多会话**架构方向；单会话 rally 已无 lever。

### 需用户决策（卡点）

- [ ] **多会话架构承诺 vs 接受 plateau**：443/490 loose / 295/490 strict / ~36% Oracle 是诚实基线。剩余提升需 Phase A IFC 统一 / Phase 2 嵌套 multicol / baseline 合成 或 taffy 升级，均为多会话工程。R314 已飞书通知。

### 若推进多会话架构（按依赖序）

1. **Phase A IFC 统一**（[`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)）— 解 large-font（ifc-008/009/011）+ welcome/morning.work 文本度量残余。R207 narrow 已证 font-051 +1 可行；需多轮 set-diff 收敛 broad 应用 + 守 multicol-fill-auto 反向依赖（R198 墙）。
2. **Phase 2 嵌套 multicol fragmentation**（[`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md)、[`column-aware-IFC-spec.md`](./column-aware-IFC-spec.md)）— 解 multicol-breaking（css-multicol 最大失败聚类）。R319 证纯 inline 迁移零增益，真实价值在嵌套 / 混合内容碎片化。
3. **baseline-export 真修复** — taffy 0.8+ baseline_overrides（R304 DEFER 升级）或自建 inline-level-box baseline 合成；解 flexbox-baseline / multicol-baseline 聚类。
4. **DC-9 blend_mode** — paint-isolation 架构（offscreen 子树渲染 + source/dest 双纹理 blend pass），低 reftest footprint（~2-4 案）。

### 已 ruled out（勿以单会话重试）

near-pass(R307) / POLLUTED hunt 三趟复核 R299–R309 + R311 + R329 / fresh-xval(R311) / Phase A 4 路 font_size(R125–R206) / multicol paint 侧(R157–R317) / balance 二分(R199–R322) / column-aware IFC 纯 inline(R319) / **column-aware IFC Phase 1（pure-inline balance 明确高度）(R381)**：执行 column-aware-IFC-spec.md §10 gate「假设 A1」，扫描全 16 css-multicol 失败案结构（height/column-fill/blockchildren），**0/16 匹配** Phase-1 目标（单层+balance+明确高度+纯 inline）——每案或有 block 子元素、或 height:auto、或 column-fill:auto、或 breaking/嵌套；spec 自身协议「A1 不存在→紧急停止转 Phase 2」生效，Phase 1 零杠杆关闭，真实 multicol lever = Phase 2（嵌套/breaking/混合碎片化，多会话硬核）/ baseline-export 3 机制(R266–R316) / **advance-width(R225–R375b) definitive 关闭**：R375 hand-crafted DejaVu 表 morning 16.41→19.14% + R375b fontdue-actual advance（临时加 fontdue dep+缓存 Font+metrics.advance_width）16.41→19.08%，双 variant 均退步；fontdue-actual（最后未测变体）亦证伪。根因：accurate DejaVuSans advance 使换行偏离 chromium（system-ui≠DejaVuSans 或换行算法不同），0.55 启发式碰巧更近。advance-width 非 morning cascade 根因/ blend post-process(R278) / font-weight -Bold(R229b) / taffy 升级(R304) / inline-flex·inline-grid width:auto shrink-to-fit（R370：probe 实证 inline-flex width:auto 同 inline-block 拉伸到满宽 800，是真 bug，但**零杠杆**——全 48 失败案 + product-smoke fixture 均不用 inline-flex/inline-grid width:auto；fix 需 flex_row_intrinsic_width（非 box_content_max_width，flex row 须求和 block 子元素非取 max），复杂且无 reftest/smoke 收益，按 code-guidelines「不做零价值修改」不修，勿再以单会话重试）。

### 已完成里程碑（参考，非当前活跃）

- M1–M9 基础设施 + 渲染器图元覆盖 + 浏览器消费 + 布局正确性 + 高级视觉效果：**已完成**（见下方「里程碑完成状态」「Done Criteria 进度」）。
- M10 上游 WPT reftest：基础设施完成，通过率 plateau（443/490 loose），达标需上述多会话架构。

---

## 最近轮次详细记录

> 全部逐轮详记已归档（master.md 仅保留顶部「综合裁决」表的结构化结论，避免无限增长）：R335–R336 → [`archive/rounds-r335-r336.md`](./archive/rounds-r335-r336.md)；R314–R334 → [`archive/rounds-r314-r334.md`](./archive/rounds-r314-r334.md)；R307–R313 → 各单轮归档（[`rounds-r307.md`](./archive/rounds-r307.md) … [`rounds-r313.md`](./archive/rounds-r313.md)）；R305–R306 → [`rounds-r305-r306.md`](./archive/rounds-r305-r306.md)；R304 → [`r304-taffy-upgrade-deferred.md`](./archive/r304-taffy-upgrade-deferred.md)；R303 → [`r303-dc9-gpu-primitive-audit.md`](./archive/r303-dc9-gpu-primitive-audit.md)；R142–R302 → [`rounds-r142-r302.md`](./archive/rounds-r142-r302.md)；R23–R139 → [`rounds-r23-r139.md`](./archive/rounds-r23-r139.md)；R11–R20 → [`rounds-r11-r20-reftest-investigation.md`](./archive/rounds-r11-r20-reftest-investigation.md)；R118–R227 技术决策表 → [`tech-decisions-r118-r227.md`](./archive/tech-decisions-r118-r227.md)。逐轮结论摘要见顶部「综合裁决」表。

> **归档完成（R397–R399）**：原 master.md 的 4 块 R68 时代 stale 节已全部迁出 archive/（内容 100% 保留 + 指针回指）——R335/R336 全文详记 → [`archive/rounds-r335-r336.md`](./archive/rounds-r335-r336.md)；M6 inline 基线明细 → [`archive/m6-inline-reftest-baseline-2026-06-07.md`](./archive/m6-inline-reftest-baseline-2026-06-07.md)；R69+ IFC 技术参考 → [`archive/r69-ifc-unification-technical-reference.md`](./archive/r69-ifc-unification-technical-reference.md)；卡点 #2–#9 框架 → [`archive/r68-other-blockers-framework.md`](./archive/r68-other-blockers-framework.md)（master.md 保留锚点 stub，`multicol-phase2-unified-column-flow-spec.md` 的「卡点 #2/#4」live 引用仍可解析）。master.md 803→508 行（累计 −295 行 / ~−21KB）。





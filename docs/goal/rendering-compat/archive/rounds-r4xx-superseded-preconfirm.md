# 归档：R472 / R447 / R430 全量导入前 pre-confirm（superseded）

> 从 `master.md` header 迁出（2026-06-22 文档治理轮）。这 3 条都是**全量导入前**的只读 pre-confirm（预测 + 缺口核查），其预测已被后续全量折回**证伪/证实**（R484 CSS2 / R457 writing-modes / R427 multicol），主体结论已并入 master.md「综合裁决」表与基线段。保留原文供溯源；active-gap 发现（text-combine-upright / multicol 特性）已在 master.md 对应 1-行指针中保留要点。

---

## R472 css/CSS2 全量导入前 pre-confirm（read-only，第 10 目录·最后 DC-14 目录，R447/R430 谱系）

上游 `css/CSS2/`（`css/css2/` 上游 404，仅 `css/CSS2/`）= **43 子目录 / 9253 test-ish 文件**（13123 tree entries，1 git/trees recursive=1 调用）= **迄今最大 DC-14 目录**（4× css-text 2994 / 11× writing-modes 1051）。**子目录分布 basics-heavy ~93%**：tables 1139 / normal-flow 857 / borders 763 / margin-padding-clear 739 / backgrounds 626 / selectors 621 / positioning 578 / text 565 / fonts 324 / generated-content 316 / lists 286 / syntax 280 / css1 262 / linebox 250 / floats-clear 249 / ui 239 / floats 144 / box-display 140 / bidi-text 108 / **pagination 103+page-box 36=139（page-break 未实现）** / visufx 102 / cascade 101 / i18n 57 / visuren 57 / zindex 52 / visudet 40 / abspos 31 / sec5 29 / values 26 / colors 22 / 余 <20 散（run-in / stacking-context 7 / zorder 1）。**当前 wpt-data 仅 10 子目录/537 文件**（~6% 覆盖，缺 33 含大头部 tables/selectors/positioning/text/generated-content/lists/syntax）。**实现核查（grep）**：基础 ✅（z-index 142/content 31/quotes 144 + 盒/色/背景/边框/linebox/normal-flow/abspos）；缺口 ❌：`run-in` 0 refs / `counter-reset`·`counter-increment` 0 refs（paint counters.rs 部分）/ `page-break`·`break-` 0 refs（→ pagination+page-box 139 fail）；未核 selectors/lists/cascade/syntax 覆盖度。**chr<1% 预测 ~30-42%（居中 ~36%，可能最高/次高目录）**：~93% 核心基础类 tables 43.8%/flexbox 50.6% 托底，下拉=7% 结构性 + near-miss 噪声 + selectors/cascade 不确定；⚠️ 预测乐观偏差警示（R461 css-text 30-38%→18.8% / R447 writing-modes 33-42%→5.6% 均 MISS low）。**⚠️ 聚合影响关键**：CSS2 分母 est ~5500-6500 matched pairs（9253 test-ish×60-70%）**远超 9-dir 聚合 3912**——若 ~36% 则 10-dir 聚合 23.9%→**~31%（+7pp，CSS2 单目录主导上拉）**，若 ~25% 则仅 ~24.6%（不动）→ **CSS2 是迄今最大聚合 mover，落点定 10-dir 基线 25% 还是 31%**。**工具链**：子目录递归 RESOLVED（b18223de capture + 10db810 discover）；GITHUB_TOKEN 非硬门控；⚠️ **TIME：9000+ test-ish @ ~50/min → oracle ~3hr + self dump ~3hr**（3-6× css-text ~35min/each，多小时操作）。**0 新单会话 lever**（缺口全结构性 run-in/counters/pagination 或基础已实现）。下一步：并行 agent 启动 CSS2 全量导入（discover 权威对）+ oracle+self dump（多小时）→ 10-dir = 最终 DC-14 全量基线。详见 [`evidence/r472-css2-preconfirm-2026-06-21.txt`](../evidence/r472-css2-preconfirm-2026-06-21.txt)。

**superseded by R484**（CSS2 实测 chr<1% 2672/6055=44.1%，R472 预测 30-42% **MISS HIGH** 超 42% 上界；R472「25% 还是 31%」实际落 36.2%）。

---

## R447 css-writing-modes 全量前 pre-confirm（只读，第 8 目录备基线）

当前 wpt-data writing-modes 仅 **21 文件**（偏斜难簇 ~50% abs-pos-*，非代表性；上游 ~855 含非 reftest）。实现核查（read-only grep）：base axis-swap ✅（R114 50 子集 49/59 self-pass）/ logical properties ✅（apply_advanced.rs 映射）/ BiDi ⚠️部分（unicode-bidi 在 text_metrics.rs，但 rustybuzz 生产未接 R331）/ **text-combine-upright ❌ 完全未实现（新发现，writing-modes 专属缺口，综合裁决表此前未单列）** / vertical-rl clearance ❌死路（R114/R164 共 4 轮证伪）/ abs-pos-vrl/vlr ❌穷尽（R237/R334-R336 三角度收敛 Phase A）/ writing-mode+flex ❌架构（R109）。**预测 chromium-Oracle chr<1% ~33-42%**（居中 ~37%，近 position 37.9% / fonts 34.8%；text-combine 未实现是专属下拉项）。**0 新单会话 lever**（全多会话或已 ruled out）；8 目录聚合若落 ~37% 则与 7-dir 36.0% 基本持平（不像 multicol 23.5% 那样下拉）。discover 脚本 dry-run 对大目录（writing-modes ~400 test）**顺序抓取会超时（5.5min+，本条原忧虑正确）**；已由 commit **58d8aa8**（`fetch_raw_many` ThreadPoolExecutor 16 workers 并行化）修复 → dry-run **31s**、发现 **787 权威对**，全量导入已由 **9735eff** 完成（见 R457）。详见 [`evidence/r447-cswritingmodes-preconfirm-2026-06-21.txt`](../evidence/r447-cswritingmodes-preconfirm-2026-06-21.txt)。

**superseded by R457**（writing-modes 实测 chr<1% 44/784=**5.6%**，R447 预测 33-42% **MISS low 大幅过乐观**）。**active-gap 保留**：`text-combine-upright` ❌ 完全未实现（writing-modes 专属缺口，未在「已知关键缺口」表单列）。

---

## R430 multicol pre-confirm（已由 R427 全量折回证实）

预测 chr<1% ~25-33% → 实测 **23.5%（7 目录最低，触预测下界下沿）**；R353 缺失特性核查仍成立（column-span / column-height / orphans·widows 未实现；balance 核心已实现但精度受限；主导聚类 column-height/balance/nested/fill/breaking 全结构性）。详见 [`evidence/r430-csmulticol-preconfirm-2026-06-21.txt`](../evidence/r430-csmulticol-preconfirm-2026-06-21.txt) + R427 全量 [`evidence/r427-cssmulticol-full-2026-06-22.txt`](../evidence/r427-cssmulticol-full-2026-06-22.txt）。

**superseded by R427**（multicol 实测 chr<1% 106/451=23.5%）。**active-gap 保留**：column-span / column-height / orphans·widows 未实现（综合裁决 R353 行已载）。

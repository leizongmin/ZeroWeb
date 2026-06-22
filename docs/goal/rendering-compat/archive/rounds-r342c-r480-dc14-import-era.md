# 归档轮次详记：R342c–R480（DC-14 目录导入 era）

> 本文件由 master.md（2026-06-22 R501 doc 调研轮）迁出，保持「最近 ~20 轮」
> 之外的历史逐轮详记。内容 100% 保留原文，仅做归档集中。综合结论见
> master.md「综合裁决」表与各目录全量基线条目。
>
> 覆盖轮次：R342c–R348（纯移动零回归）/ R428–R452（flex min-size:auto probe，
> LANDED commit 574db50）/ R439–R442（multicol inf-loop 追踪 saga）/ R430
> （multicol pre-confirm，superseded by R427）/ R447（writing-modes pre-confirm，
> superseded by R457）/ R457（css-writing-modes 全量导入 LANDED）/ R461
> （css-text 全量导入）/ R463（css-text self-source 折回）/ R471（css-text oracle
> 完成）/ R472（css/CSS2 pre-confirm，superseded by R484）/ R480（CSS2 self-dump
> 完成 + forward-look 实证·durable 结论）。

---

**R480（CSS2 self-dump 完成 + forward-look 完成事件实证·durable 结论留顶，in-flight 细节已归档）**：self-dump ~19:42 EXITED，**6106/6109 -test/-ref 两侧齐（无 panic，~84min，cross-validate 有效分母=6106）**。**durable 对账**（cross-validate 前置）：runner 跑 **6109 cases**（非 6315 link-rel-match；174 CSS2 missing-ref skip 在 import 子树外=scoping artifact，潜在恢复=并行 agent 增导 `/css/reference/`）；naming 已验对齐（oracle `css_CSS2_<sub>_<name>_xht.png` ↔ dump `-test/-ref.png`，`cross-validate.py:65-72` sid() 正确）；dump dir 跨轮累积（naive `ls` 含 css-text 1572+writing-modes 786 stale，cross-validate oracle-driven 仅比 `css_CSS2_*` sid 无害，分母对账须按 fresh `css_CSS2_*`+mtime）。**forward-look 完成事件实证**：oracle 完成时 4274/~9265≈46.2% 仍 'm' 段，dump 5 大头子目录 n/p/s/t（normal-flow 733/selectors 545/positioning 519/tables 376/text 375=2548=42%）覆盖率全 0 → **勿即刻 cross-validate**（早段 borders/backgrounds-biased 会系统性高估真 chr<1%→误诊 R472 ~30-42% 预测为假高；unbiased 窗口 ETA ~20:44 代表性 / ~21:19 全量，R481 实时复算 on-track）。R473-R479 逐轮 in-flight 监测原文（naming/分母去险、cross-validate 早信号去险、逐轮 forward-look 预测）已归档 [`archive/rounds-r473-r480-css2-dualine-oracle-dump-saga.md`](./archive/rounds-r473-r480-css2-dualine-oracle-dump-saga.md)，全部由 R480 完成事件实证确认。

**R472 css/CSS2 pre-confirm（superseded by R484·原文归档）**：预测 chr<1% ~30-42% → 实测 **44.1%（R484，MISS HIGH 超 42% 上界）**；10-dir 聚合 23.9%→36.2%（CSS2 超大分母主导上拉）。CSS2 子目录分布（tables 1139 / normal-flow 857 / … basics-heavy ~93%）+ 缺口核查（run-in / counter-reset / page-break 0 refs → pagination 139 fail）+ 工具链/时间估算 → [`archive/rounds-r4xx-superseded-preconfirm.md`](./archive/rounds-r4xx-superseded-preconfirm.md)。

**R471 css-text oracle 完成 + cross-validate DONE（chr<1% 18.8%，R461 预测 30-38% MISS；9 目录聚合 23.9%；0 新单会话 lever，缺口=white-space/text-align 精度+Phase A）** → 全文已归档 [`archive/rounds-r457-r471-writingmodes-csstext-foldins.md`](./archive/rounds-r457-r471-writingmodes-csstext-foldins.md)；证据 [`evidence/r471-csstext-crossvalidate-2026-06-21.txt`](./evidence/r471-csstext-crossvalidate-2026-06-21.txt)。

> **R465 / R466-R470 css-text in-flight 观测史（已归档，css-text 由 R471 闭合）**：oracle 全量捕获 + self dump 89.2% + cross-validate 预检去险的逐轮 in-flight 观测；含 **R468 可复用方法论教训**——oracle 捕获早段速率（~0.77s/PNG）不可外推到整批，进入 shaping/bidi 复杂脚本区回落到稳态 **~50/min**，**勿以「过早段 ETA 仍在跑」误判 stall**（R458/R459/R464 三度误诊前车；当前 CSS2 oracle ~50/min 即此稳态）。逐轮指针已移出至 [`archive/rounds-r456-r465-csstext-oracle-import-saga.md`](./archive/rounds-r456-r465-csstext-oracle-import-saga.md)（R466-R470 时间线段），原始 in-flight 数据见 [`evidence/r465-csstext-oracle-capture-inflight-2026-06-21.txt`](./evidence/r465-csstext-oracle-capture-inflight-2026-06-21.txt)。最终结果（css-text oracle chr<1% 18.8%）见上 R471。

**R463 css-text self-source reftest 折回（self 1402/1572=89.2%，170 self-fail：white-space 85+text-align 48=78% 主导「已实现但精度不足」，i18n 推测假通过；9 目录 self 聚合 72.0%）** → 全文已归档 [`archive/rounds-r457-r471-writingmodes-csstext-foldins.md`](./archive/rounds-r457-r471-writingmodes-csstext-foldins.md)；证据 [`evidence/r463-csstext-selfsource-foldin-2026-06-21.txt`](./evidence/r463-csstext-selfsource-foldin-2026-06-21.txt)。

**R461 css-text 全量导入 COMPLETE（2994 文件/1914 test-ish 与 10db810 双 match；聚类 white-space 595/i18n 539 主导；oracle 预测 ~30-38%，0 新单会话 lever）** → 全文已归档 [`archive/rounds-r457-r471-writingmodes-csstext-foldins.md`](./archive/rounds-r457-r471-writingmodes-csstext-foldins.md)；证据 [`evidence/r461-csstext-import-complete-cluster-map-2026-06-21.txt`](./evidence/r461-csstext-import-complete-cluster-map-2026-06-21.txt)。

**R457 css-writing-modes 全量导入 ✅ LANDED（commit 9735eff，8/9 DC-14 目录；self 613/786=78.0% / oracle chr<1% 44/784=5.6% 迄今最低，R447 预测 33-42% 大幅过乐观 / 8-dir 聚合 26.8%；0 新单会话 lever）** → 全文已归档 [`archive/rounds-r457-r471-writingmodes-csstext-foldins.md`](./archive/rounds-r457-r471-writingmodes-csstext-foldins.md)；证据 [`evidence/r457-cswritingmodes-full-2026-06-21.txt`](./evidence/r457-cswritingmodes-full-2026-06-21.txt)。

> **R456 / R458-R460 / R462 / R464 = css-text 导入 + oracle 工具 saga（已归档）**：R458 嵌套目录（30 子目录）顶层 discover 得 0 对 → R459 子目录递归 resolved（`1568cb0`）→ R460 token gate 移除（`10db810` git/trees recursive=1，2 调用/目录）→ R461 全量导入 COMPLETE → R462 self in-flight + JS 依赖量化 24.5% → R464 oracle 缺失根因（捕获脚本顶层 `readdirSync` bug，`collectTests` 递归修复）；R456 = flexbox probe chromium-Oracle 真影响量化（+7 DC-14 win，已并入基线段 flexbox 50.6%）。R465 当前 oracle 全量捕获 in-flight（见顶部 header）。逐轮原文详见 [`archive/rounds-r456-r465-csstext-oracle-import-saga.md`](./archive/rounds-r456-r465-csstext-oracle-import-saga.md)。
**R447 css-writing-modes pre-confirm（superseded by R457·原文归档）**：预测 ~33-42% → 实测 **5.6%（R457，迄今最低，MISS low 大幅过乐观）**。**active-gap 保留**：`text-combine-upright` ❌ 完全未实现（writing-modes 专属，未在「已知关键缺口」单列）；vertical-rl clearance 死路（R114/R164 4 轮证伪）；abs-pos-vrl/vlr Phase-A；discover 顺序超时已由 58d8aa8 并行化修复。原文 → [`archive/rounds-r4xx-superseded-preconfirm.md`](./archive/rounds-r4xx-superseded-preconfirm.md)。

**R430 multicol pre-confirm（superseded by R427·原文归档）**：预测 chr<1% ~25-33% → 实测 **23.5%（R427，7 目录最低触下界下沿）**；multicol 缺口（column-span / column-height / orphans·widows 未实现；balance 精度受限）见综合裁决 R353。原文 → [`archive/rounds-r4xx-superseded-preconfirm.md`](./archive/rounds-r4xx-superseded-preconfirm.md)。

**R428–R452 flex min-size:auto probe — ✅ LANDED（commit 574db50 probe + 45faa3e multicol 守卫；~120 轮 plateau 后首个 ZeroWeb-side clean win）**：根因=style-system 默认 `min_width/min_height=Px(0.0)`（CSS §4.5/§6.6 initial 实为 `auto`）→ converter `Some(0.0)` 短路 taffy content-min floor（flexbox.rs:774），**非 taffy-blocked**；修复默认改 `Auto`→converter `Dimension::Auto`→taffy 算内容下限。实测 css-flexbox self **+14**（49.6→52.4%）/ css-grid +1（62.5→64.6%）/ multicol 中性 / **0 净回归**；full-suite probe-applied reftest 1321/1924=68.7%；chromium-Oracle 总 **+7（probe/R428 +6 + R429 +1，244→251/496=50.6%，详 R456 entry）**。第二 lead R429 flex shorthand type-awareness（commit 88e11d2，CSS §7.1 type-based）self net-0、oracle +1。残余 partial（aspect-ratio 49 例 taffy-blocked R304）。R432–R437 逐轮原文 + R448（footprint）/R450–R452（reftest 完成+A/B）见 [`archive/rounds-r432-r437-flex-minsize-probe.md`](./archive/rounds-r432-r437-flex-minsize-probe.md)。

**R439–R442 multicol inf-loop 追踪 saga（已收敛；逐轮原文归档 [`archive/rounds-r439-r442-multicol-infloop-tracking.md`](./archive/rounds-r439-r442-multicol-infloop-tracking.md)）**：`assign_children_to_columns_with_breaking`（multicol.rs:389）的 `max_col_height>0.0` 守卫（防 height:0 multicol + 超高子→while offset+=0 无限循环）**R444 实测已 re-apply 到工作树**（multicol.rs，并行 agent @09:55，纯守卫无回归测试）。**追踪 call site（multicol.rs:268）确证该 bug 生产不可达**——breaking() 仅在 `info.sequential_fill && height_limit>0.0` 时被调，故无调用者传 0，`make reftest` 不会挂起；该守卫是**防御性加固**（直接调用健壮性）非生产 crash fix，保留无害、低优先级。**multicol.rs 全迭代/除法点审计**：唯一 while 循环即该处，余皆有限 for；除法点全 float 除（count==0 有早返守卫，col_count `.max(1)`）→ multicol 无其他隐藏 hang（crash-surface 风险≈0）。

**doc-maintenance（2026-06-20 verify 轮）**：plateau 结论 read-only 复核成立（R354 fresh baseline 439/490 零漂移、clean-win 面 R351 后穷尽），无需新调整方向——现有「综合裁决 + 下一步」即当前结论。文档治理两项：① 将「技术决策记录」表中 **R118–R227 逐轮历史条目**（50 行，2026-06-14~17，远超最近 20 轮窗口，主体已在 rounds-r23-r139 / rounds-r142-r302 归档）迁出至 [`archive/tech-decisions-r118-r227.md`](./archive/tech-decisions-r118-r227.md)（50 行 → 1 指针行，master.md 833→786 行）；② 纠正「最近轮次详细记录」窗口标注（R335–R336 为最后两轮全文详记，R337–R354 为 plateau 复核/治理轮，精简结论见上方「综合裁决」表）。本轮零代码变更（并行 agent 正在 layout-engine 开发，未触碰）。

**R342c–R348（2026-06-19，已完成·纯移动零回归）**：全仓库 2000 行达标收尾——R342c `table.rs` 2694→1973（抽 `table_borders.rs` 740，CSS §17.6.2 collapsed 边框冲突解析）/ R343-R344 生产源码 <2000（app_render.rs、gpu/renderer/mod.rs）/ R345-R346 测试文件 <2000（paint/visual.rs 2056→1790、inline/advanced.rs 2281→1948）/ R347 reftest.rs 拆出 resources.rs / R348 fresh chromium-Oracle 复测 plateau 稳定（污染 48.0% vs R311 48.2%，逐 case 稳定）。详见 [`evidence/r342c-table-borders-split-2026-06-19.txt`](./evidence/r342c-table-borders-split-2026-06-19.txt)。

# 归档：R473-R480 CSS2 双线（oracle + self-dump）in-flight 监测 saga（2026-06-21）

> 从 master.md 顶部压缩归档（R482 doc-maintenance 轮建档）。这些轮次是 CSS2 第 10/最后 DC-14 目录的**self-dump 完成 + oracle 捕获双线 in-flight 监测 saga**——从 R473（导入完成后启动双线）经 R474-R479（逐轮 naming/分母去险 + forward-look 预测）到 R480（self-dump 完成事件实证 + forward-look 闭合）。**该 saga 已由 R480 完成事件闭合**（self-dump ~19:42 EXITED，6106/6109）；后续 oracle 仍在跑向 unbiased 窗口（~20:44），由 master.md 顶部 R481 活跃追踪。
>
> master.md 顶部保留的 durable 事实：CSS2 分母（runner 6109 / 有效 6106）、naming 对齐（`cross-validate.py` sid()）、forward-look 结论（勿即刻 cross-validate，早段 borders-biased 会系统性高估真 chr<1%→误诊 R472 ~30-42%）、unbiased 窗口 ETA（~20:44 代表性 / ~21:19 全量）。本档存被移出的**逐轮 in-flight 观测原文**（PIDs/精确时间戳/子目录覆盖率快照/逐轮 forward-look 预测轨迹）。逐轮原文按时间倒序（master.md 原顺序）。

## Saga 时间线指针（一句话）

- **R480**（留顶·闭合轮·durable 结论保留）: self-dump 完成事件实证确认（~19:42 EXITED，6106/6109，~84min）+ forward-look 经完成事件量化坐实（oracle 完成时 46.2% 仍 'm' 段，dump 大头 n/p/s/t 42% 零覆盖）→ unbiased 窗口 ETA ~20:44 代表性 / ~21:19 全量
- **R479**（归档·in-flight forward-look 预测）: 双线监测健康（self-dump 2866/6109 ~47% @~105/min ETA ~19:17；oracle 1429/~9265 ~15.4% @~52/min ETA ~21:15）——把 R476 cross-validate 守卫映射到 self-dump 完成事件：完成时 oracle ~50%（低于 60% 阈值）且未覆盖 dump 大头子目录，勿即刻 cross-validate
- **R478**（归档·in-flight）: 双线健康（self-dump 2028/6109 ~33%；oracle 1011/~9265 ~11%）；dump 已扩到 normal-flow/backgrounds/margin-padding-clear/generated-content/tables（非 borders-only）；oracle 仍 borders 子目录
- **R477**（归档·in-flight）: 健康线性推进不变；compress R476/R477 + 2 新发现
- **R476**（归档·in-flight + cross-validate 早信号去险）: dump↔oracle sid overlap 仅 165（of 2156 dump / 1076 oracle）因两线处理子目录顺序不同（dump heavy normal-flow/selectors/positioning；oracle heavy backgrounds/borders/bidi-text），当前 overlap 几乎全 borders → self-dump 完成 ≠ cross-validate 早信号就绪，须等 oracle 覆盖 dump 大头子目录或 overlap ≥~2000（actionable 阈值）
- **R475**（归档·doc-maintenance）: 压缩过时 R465 css-text in-flight 段
- **R474**（归档·分母精度）: runner 跑 6109 cases（非 6315 link-rel-match）；174 CSS2 missing-ref skip 在 import 子树外=scoping artifact（非 ZW bug；潜在恢复=并行 agent 增导 `/css/reference/`）；cross-validate 有效分母 ≤6109
- **R473**（归档·naming+分母去险）: CSS2 导入完成（commit 40ca95b7，13027 文件，6315 matched pairs / 9265 test-ish）；self-dump + oracle 双线自 18:18 跑；naming 已验对齐（oracle `css_CSS2_<sub>_<name>_xht.png` ↔ dump `-test/-ref.png`，`cross-validate.py:65-72` sid() 正确）；-test/-ref 494/494 齐初判；时间限风险（test-guard 2h vs CSS2 ~4× css-text）后续由实测下行修正

---

## 原文（逐轮，时间倒序）

### R480（留顶·durable 结论保留；以下为 in-flight 细节原文）

**R480 CSS2 self-dump 完成 + forward-look 完成事件实证确认（19:43，read-only·无卡点）**：**self-dump 完成事件已到达**（R476-R479 forward-look 一直追踪的事件）——runner(4861)+test-guard(4859) ~19:42 EXITED，最后写 @19:42:08，**6106/6109 -test 与 -ref 两侧精确齐 6106（无 ref-missing-test，即无 panic/崩溃）**，~84min 总耗（18:18→19:42）远低于 test-guard 20:18 限时；3 案缺失=clean missing-ref skip（两侧皆无，reference 指向 import 子树外，非 ZW bug；**cross-validate 有效分母=6106**）。/ oracle 完成时 **4274/~9265≈46.2%**（chromium 4914 活跃 etime 01:24:50，newest=margin-padding-clear_padding-left-078，仍 'm' 段）。**forward-look 经完成事件实证确认（强于 R476-R479 预测）**：oracle 对 dump **5 大头子目录 normal-flow 733 / selectors 545 / positioning 519 / tables 376 / text 375（合计 2548 = dump 42%）覆盖率全 = 0**——全在字母序 n 之后，oracle 未抵达；oracle 已覆盖全为 a-m 段（backgrounds/borders/bidi-text/box-display/cascade/colors/css1/floats/floats-clear/fonts/generated-content/i18n/linebox/lists/margin-padding-clear）。**故 self-dump 完成时勿即刻 cross-validate**：早段 borders/backgrounds-biased 非仅「不代表性」，且会**系统性高估**真 CSS2 chr<1%（视觉简单 borders/backgrounds→chr<1% 偏高；缺失 layout-heavy n/p/s/t→chr<1% 偏低）→ 误诊 R472 ~30-42% 预测为假高（R476 警示的陷阱，R480 在完成点量化坐实）。**unbiased 窗口**：oracle 须覆盖 n/p/s/t——自当前 'm' 经 n/p/s/t ≈3154 案 @~52/min ≈61min → 代表性覆盖 ETA **~20:44**，全量完成 ~21:19。零代码变更（仅文档）。**durable facts（数据卫生/对账）**：runner 跑 **6109 cases**（非 6315 link-rel-match；174 CSS2 missing-ref skip 在 import 子树外=scoping artifact，潜在恢复=并行 agent 增导 `/css/reference/`）；naming 已验对齐（oracle `css_CSS2_<sub>_<name>_xht.png` ↔ dump `-test/-ref.png`，`cross-validate.py:65-72` sid() 正确）；dump dir 跨轮累积（naive `ls` 含 css-text 1572+writing-modes 786 stale，cross-validate oracle-driven 仅比 `css_CSS2_*` sid 故无害，分母对账须按 fresh `css_CSS2_*`+mtime）。下一步：oracle 抵 n/p/s/t（~20:44 代表性 / ~21:19 全量）→ unbiased cross-validate → 验 R472 ~30-42% → **10-dir 最终 DC-14 全量基线**（CSS2 = 最大聚合 mover，落点定 10-dir 25% vs 31%）。

### R473-R479 saga 总览（master.md 原指针行）

> **R473-R479 = CSS2 双线 in-flight 监测 saga（已由 R480 完成事件闭合）**：R473 naming+分母去险 / R474 分母精度（6109 cases 非 6315；174 missing-ref skip 在 import 子树外=scoping artifact）/ R475 doc-maintenance / R476 cross-validate 早信号去险（dump↔oracle 子目录处理序不同，早 overlap borders-biased）/ R477-R479 逐轮 forward-look 预测——**全部由 R480 在实际完成事件点实证确认**：self-dump 完成 ~19:42（R479 预测 19:27→实际略晚，borders 尾段 ~11/min 慢于大头段）、oracle 完成时仍 a-m 早段未触 dump 大头 n/p/s/t（R479 预测命中，R480 量化=2548 案 42% 零覆盖）。结论一致：双线健康线性推进，无卡点；勿即刻 cross-validate，unbiased 窗口 oracle 覆盖 n/p/s/t（~20:44+ 代表性）。逐轮 commit 见 git log。

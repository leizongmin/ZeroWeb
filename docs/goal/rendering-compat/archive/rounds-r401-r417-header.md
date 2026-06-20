# R401–R417 逐轮详记（header 治理迁出）

> 本文件于 R423（doc-maintenance）从 `master.md` header（line 3）迁出。
> header 经 R422 css-fonts 折回后，R401–R417 逐轮 doc-maintenance 子句（pre-confirm 预测 +
> 折回）累积膨胀至 6800+ 字符（违反 goal doc「master.md 不允许无限增长」治理原则），且与
> 「综合裁决」表（R401–R422 行）+ 基线块内容重复。R422 折回后这些子句已被 5 目录真数据取代，迁此备查。
> 结构化结论保留在 `master.md`「综合裁决」表（每轮一行）；本文件为逐轮 verbose 备查记录。
> 更早逐轮详记见 [`rounds-r381-r393-header.md`](./rounds-r381-r393-header.md)（R381–R394）、
> [`rounds-r351-r380-header.md`](./rounds-r351-r380-header.md)（R351–R380）等。

---

## R417 — css-text-decor 全量折回（4/9 目录·首个文字类，commit f2bacb2）

并行 agent commit f2bacb2 带测量（明确「master.md left to fold-in」）。权威 discover `link rel=match` = **246 对**（非旧估 356）；self-source **244/246=99.2%**（runner 文字容差）/ @0.5% strict 185/242=76.4% / **chromium-Oracle chr<1% 70/242=28.9%**（DC-14 真）/ 污染 117/242=48.3%。**4 目录聚合 oracle 174/497=35.0%**（grid 39.6 / position 37.9 / tables 43.8 / **text-decor 28.9 最低**，从 3-dir 40.8% 下拽）。

**关键纠正（推翻 R409/R411/R412 三预测）**：已实现 text-decoration chr<1% 仅 **23%**——反**低于**未实现 text-emphasis 的 34%；ZW 渲染了 decoration 但**像素级系统性偏薄**（thickness/underline-offset/dotted 模式/skip-ink，spot-check chr 非白 1.50% vs ZW 1.04%）→「已实现 decoration 托底」假设为假，故 R409 ~35-43% / R411 ~38-46% / R412 ~32-39% **全过乐观**（真值 28.9% 低于所有下限）。**亦纠正**：「69% 停滞」框架（R413–R416）基于错分母——继承 R409 旧估「356」vs 权威 discover 实为 **246 对**，import 自 R412 起即**完成**（246/246=100%），仅未提交 5 轮（会话级，R415 已归因非工具 bug）。首个文字类目录全量数据证实「文字类 oracle 显著低于布局类」假设；**0 新单会话 lever**（decoration 像素精度=Phase A/font-metric；emphasis=Phase A line-box R392/R393）。详见 [`evidence/r412-csstextdecor-full-2026-06-22.txt`](../evidence/r412-csstextdecor-full-2026-06-22.txt)。

## R412–R416 — 在途单快照预测 + 停滞观察（已由 R417 真数据取代，压缩保留）

R412 据 246 在途样本把 R411 预测修正为 ~32-39%（emphasis 55.7% 全失败封顶）；R413–R416 因「246/356=69% 零增长」判停滞并两次飞书通知。**R417 真数据证两判断均错**：预测过乐观（真 28.9%）+ 分母错（权威 246 非 356，import 早完成）。详见 [`evidence/r412-csstextdecor-rebalance-volatility-2026-06-22.txt`](../evidence/r412-csstextdecor-rebalance-volatility-2026-06-22.txt)。方法学教训=在途单快照预测 + 文件计数分母均不可信，须以权威 discover + 真 oracle 为准。

## R411 — css-text-decor 在途量化验证 R409「再平衡」预测（只读）

在途 191 test-side 实测 text-emphasis 子集 **82%→全量 47.6%**、text-decoration **10%→42.4%**，子集 emphasis 偏斜在全量消失（平衡集）；text-decoration 已实现占 42.4% 是真实大通过项 → R409 预测**上修至 ~38-46%**（大概率触及 41% 基线）。**本轮是最后一轮前向准备**（R409+R410+R411 已覆盖下两个文字类目录全部基线），后续等并行 agent 提交 text-decor 折回 4/9 对照。

## R410 — css-fonts 全量前 pre-confirm 簇映射（只读）

子集 ~47% 依赖 OpenType feature/color font（rustybuzz 生产未接 text.rs:405 `ch as u32` + COLR/CPAL 全无），结构偏斜镜像 text-decor；预测全量 oracle 落 ~33-42%（OpenTech-feature/color-font 簇封顶），0 新单会话 lever。（R422 实测 34.8% 落此区间，预测 CONFIRMED。）

## R409 — css-text-decor 全量前 pre-confirm 簇映射（只读）

子集 78 文件 **82% text-emphasis**（生产全无，R392 完整回退）结构偏斜**不可外推全量**；实现锚定 text-decoration ✅/text-shadow ✅/text-emphasis ❌；预测全量 oracle 落 ~35-43%（text-emphasis 簇封顶），0 新单会话 lever。（R417 实测 28.9% 低于此区间下限，预测过乐观。）

## R408 — css-tables 全量数据折回（commit 7c08d74，明确「master.md left to doc-maintenance agent」）

**css-tables 全量**：self-source 75/112=67.0% / chromium-Oracle 49/112=43.8%；**3/9 目录全量完成**（grid+position+tables），全量 oracle 真一致 **~41%**（grid 39.6% / position 37.9% / tables 43.8%）——子集 42.1% 广义口径对 oracle 接近、对 self-source 严重高估（3 目录聚合 self 65.1% vs 子集 ~90%+）。本轮把 3-目录全量图景折回 header / 综合裁决基线+R408 行 / M10 / 轨道 1 / 上游通过率节；**tables 簇 pre-confirm**：tables 比 grid/position 偏高 +3-6pp = 真实历史修复累积（colspan R177b 5 部件 / border-collapse R49+R342c / auto-cell-column-width R117 / height-as-minimum R168 / explicit-width 列冻结 R364），**非异常**；残余 tables 失败落 writing-mode 轴 / baseline-export / border-conflict 已知结构簇，0 新单会话 lever。零代码变更。

## R401–R408 摘要 — 3 目录 DC-14 Phase 2 完成

R401 grid（62.5%/39.6%）+ R404 position（64.2%/37.9%）+ R408 tables（67.0%/43.8%）完成 3 目录 DC-14 Phase 2（聚合 self 166/255=65.1% / oracle 104/255=40.8%）；分母方法论（discover 权威 `link rel=match`，tables 203 上游→119 对→112 运行）3 目录一致验证。详见 [`evidence/r405-dc14-three-dirs-complete-2026-06-22.txt`](../evidence/r405-dc14-three-dirs-complete-2026-06-22.txt)。

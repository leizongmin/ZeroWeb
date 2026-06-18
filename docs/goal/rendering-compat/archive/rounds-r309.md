# R309 — POLLUTED clean-win 杠杆收尾（归档自 master.md）

> 归档说明：本文件为 master.md「最近轮次详细记录」中 R309 的逐轮详细记录，于 doc-maintenance 轮（R329，2026-06-19）归档——R329 加入后 master.md 最近轮次窗口达 21 轮（R309–R329），R309 作为第 21 轮迁出，窗口收窄为最近 20 轮（R310–R329）。R309 的核心结论（POLLUTED clean-win 杠杆经 R308+R309 收尾关闭；R298 全量清单逐项归类，唯 R308 font-size% 一处真实 clean win；与 R307 near-pass frontier 关闭互补，两条 clean-win 搜索策略穷尽）仍以浓缩形式保留在 master.md「综合裁决」与后续 R311/R329 轮的「承接」引用中。本归档仅为可追溯性保留，archive 区不修改。

---

### R309 — POLLUTED clean-win 杠杆收尾：3 候选实证 ruled out（read-only 实证，基线 loose 438/490 / strict 295/490 持平）

**承接**：R308 font-size 百分比修复证明「逐项 probe 未调查 POLLUTED 候选」仍能发现真实单点 bug（anonymous-inline-inherit = R298 POLLUTED 清单最后一处非结构性聚类）。本轮继续逐项 probe R298 清单剩余未调查项，3 候选全部 ruled out。

**候选 1 — `table-grid-item-dynamic-003`（self 0% / chr 10.51%[R308 后]）RULED OUT（JS 动态）**：测试 `display:grid;height:100px` > `table;height:100%;padding-top:100px;box-sizing:content-box`，`onload` JS 触发增量 relayout（getBoundingClientRect + body width 变更）。测试名「don't grow on incremental relayout」= **JS 驱动的动态 relayout 行为验证**。ZeroWeb reftest 不执行/不应用 onload 触发的 relayout → 渲染静态初态；chromium 执行 JS → 渲染 post-relayout 态；二者本质不同（10.51%）。**非静态 CSS 单点 bug**（即使 ZeroWeb 静态 table h=200[=100%×100+padding100] 已计算正确），是 JS 执行 + 增量 relayout 特性缺口（需 reftest harness 执行 onload + 触发 re-layout）。defer 至 JS/动态布局特性。

**候选 2 — `font-family-name-025`（self 0% / chr 7.13%）RULED OUT（缺测试字体）**：测试 `font-family: CSSTestBasic-Bold` 不应匹配 PostScript 名——**显式要求安装 CSSTest 测试字体**（页面文字「Test fonts must be installed for this test: FAIL」）。ZeroWeb 与 chromium oracle 环境均无 CSSTest 字体 → 双方回退字体渲染，7.13% = fontdue vs chromium 回退字体度量噪声（R174/R187 AA 谱系），**非 ZeroWeb 可修 bug**（缺字体资源，非渲染缺陷）。

**候选 3 — `whitespace-001`（css-tables，self 0% / chr 2.09%[R308 后]）RULED OUT（结构性 table-cell+% 宽+空白 fit）**：`.outer{display:table;width:500px;border:1px}` > 两个 `.half{display:inline-block;width:50%}` 中间有空白。PIL 实测：**ZW 渲染两 block 换行**（blue rows 9-27 line1 / yellow rows 28-46 line2），**chromium 渲染同一行**（blue+yELLOW rows 9-26）。REF 用 `display:block`（余同），ZW 渲染 test==ref 均「换行」（self 0%）。差异 = display:table 匿名 cell 内 50%+50%+空白空间是否触发换行：ZW 计 50%+50%+空白宽 > cell 宽 → 换行；chromium 不换行（50%+50% 恰填满 cell，空白空间在 fit 边界被吸收）。根因耦合 **table-cell 百分比宽基址（R177b table-width 谱系）+ R105 inter-inline-block 空白宽计入 fit 判定**——修任一会动 R105/R177b 已绿用例，**非安全单点**。defer。

**裁决：POLLUTED clean-win 杠杆经 R308+R309 收尾关闭**。R298 全量 POLLUTED 清单逐项归类：backdrop-inherit-rendered(R202 dialog JS)/abs-pos-border-offset-002(writing-mode)/table-grid-item-dynamic-003(本轮 JS)/semi-replaced-stretch-input(R202 表单)/flexbox-collapsed-item(R301 intrinsic)/font-051(Phase A)/collapsed-border-vertical-*(writing-mode)/float-no-content-beside(R300)/font-family-name-025(本轮 缺字体)/whitespace-001(本轮 结构性)/anonymous-inline-inherit(R308 ✅fixed)/grid-calc-margin+iframe+float-non-replaced(R302)。**唯一 clean win = R308 font-size%**；其余全结构性/特性缺口/字体噪声。与 R307（near-pass frontier 关闭）互补：**两条 clean-win 搜索策略（near-pass 聚类 + POLLUTED 逐项）均已穷尽**，剩余 forward motion 全为结构性多轮里程碑（Phase A 墙②③ / multicol column-aware IFC R131 / DC-9 blend_mode / DC-13 残余 / writing-mode 轴）或特性实现（JS 动态 relayout / 原生表单控件 / dialog）。

**本轮 read-only 实证**：零代码变更；PIL+LAYOUT_DUMP 逐候选实证（table-grid-item LAYOUT_DUMP table h=200 正确 + JS 缺口定性；font-family-name 缺字体；whitespace-001 display:table vs block 换行差异 + R105/R177b 耦合）。基线 loose 438/490 / strict 295/490 / chromium-Oracle 持平。next = 转结构性里程碑（multicol column-aware IFC spec-rfc 实施 R131，最大失败聚类）或 DC-9 blend_mode 独立特性。

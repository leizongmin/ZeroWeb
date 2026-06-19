# 归档：R311 详细记录

> 从 `docs/goal/rendering-compat/master.md`「最近轮次详细记录」迁出（R331 归档，保持最近窗口 ≤20 轮）。
> 当前状态以 `master.md` 顶部「综合裁决」与「最近轮次详细记录」为准；本文件为历史记录，只追加。

---

### R311 — R308 后 fresh chromium-Oracle cross-validate：plateau 再确认 + 4 新候选 ruled out（read-only 实证 + evidence，基线 loose 438/490 / strict 295/490 持平）

**承接**：R310 multicol 设计修订 + baseline-export 探针后，做 **R308 font-size% 修复后的 fresh 全量 cross-validate**（980 ZeroWeb dump vs 503 oracle，475 可比），验证 R308 是否改变污染景观 + 搜寻新 contained-bug 候选（原 cross-validate 曾 surface R308 的 font-size bug）。

**实证结果（`evidence/r311-cross-validate-fresh-2026-06-19.txt`）**：
- **污染 158/328 self-pass = 48.2%**（R298 为 48.6%）—— R308 font-size% 仅边际改善（0.4pp），符合预期（font-size% 影响有限用例）。
- 按目录：CSS2 56% / flexbox 26% / fonts 46% / grid 50% / multicol 60% / position 38% / tables 41% / text-decor 47% / writing-modes 73%（与 R298 一致）。
- 真实 chromium 一致率 ≈ 同源通过(328) 中非污染(170) + self-fail 中 chr 一致 ≈ 仍 ~35-37%。

**4 个新候选（R298 未列或未深查）逐项实证 ruled out**：
1. **`downloadable-font-scoped-to-document`（20.22%）= JS+iframe+@font-face**：`iframe1.onload` + `iframe2.src` + `reftest-wait` 测 web font 文档作用域隔离。需 iframe 子文档加载 + JS + 字体作用域，**特性缺口非 CSS bug**。
2. **css-fonts 聚类**（alternates-order 13.8% / font-family-013 6.65% / font-default-02,03 3.46%）= **@font-face 自定义字体未加载**：reftest 不加载 .woff/.ttf → 回退字体，fontdue vs chromium 度量噪声（同 R309 font-family-name-025）。特性缺口。（**R330 证伪**：导入字体激活 @font-face 后净负向，根因 rustybuzz 未接入生产。）
3. **`rules-groups`（3.39%）= legacy HTML4 `rules=groups` 属性**：ZeroWeb **完全不解析** rules/cellspacing/cellpadding 任一 legacy 表格属性。niche legacy 特性 + 测试还用 CSS `border-block-start/end` 覆盖交互 → 非干净 contained add（低 ROI，单用例）。
4. **`flexbox-baseline-align-self-baseline-horiz-001`（17.64%）= inline-flex 基线合成**：`display:inline-flex;align-items:baseline`，容器自身基线导出。LAYOUT_DUMP 实测 inline-flex 项位置与 chromium 大差（17.64% 全在容器垂直位置=基线导出错）。属 **R295 flexbox-baseline 结构性聚类**（同 R310 multicol baseline-export 谱系，但 inline-flex 侧 taffy 算了 first baseline 却仍错——疑基线合成取错项/字体）。

**裁决**：post-R308 fresh cross-validate **再确认 plateau**——无新 contained CSS bug surface。剩余 polluted 全为结构性（writing-mode / flexbox-baseline / multicol）+ 特性缺口（@font-face 加载 / JS / iframe / 原生表单控件）+ legacy 属性。与 R307（near-pass）+ R309（POLLUTED）杠杆关闭一致。

**对优先级影响**：三条 clean-win 搜索路径（near-pass 聚类 R307 / POLLUTED 逐项 R309 / fresh cross-validate R311）**全部穷尽**，均无新 contained win。剩余 forward motion 确认为**纯结构性多轮**：① baseline-export（R310 探针确认根因，span flex+multicol+inline-flex 三侧，需 block first-baseline 计算 + 注入）；② multicol breaking wiring（R131/R201）；③ DC-9 blend_mode（paint-isolation）；④ DC-13 残余。或**特性实现**：@font-face 字体加载 / JS 动态 relayout / 原生表单控件。

**本轮 read-only 实证 + evidence**：零代码变更（`git diff -- '*.rs'` 空）；新增 `evidence/r311-cross-validate-fresh-2026-06-19.txt`（47 行，top-30 polluted + 4 新候选 ruling）。基线 loose 438/490 / strict 295/490 / chromium-Oracle ~48.2% 污染持平。next = baseline-export spec-rfc（R310 探针已确认根因，是最大可控结构性方向），或 pivot 到 @font-face 字体加载特性（影响整个 css-fonts polluted 聚类 24 案）。**（注：@font-face 候选由 R330 证伪为净负向；其 #2 css-fonts 聚类假设不成立。）**

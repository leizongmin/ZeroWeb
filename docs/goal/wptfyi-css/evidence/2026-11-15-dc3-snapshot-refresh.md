# DC-3 快照刷新：coverage-2026-11-14

2026-11-15。weekly cron `4214a625d`（11-14 18:13Z，wptfyi-gap job 全绿）
提交回 main：[coverage-2026-11-14.md](../../../compat/wptfyi/coverage-2026-11-14.md)
+ 同名 .json/.md gap 报告。控制面头部已同步引用。

## 1. 头条账

| 日期 | css 通过 / 分母 | 通过率 | 对验收线（182,392）余量 |
|---|---|---|---|
| 11-10（run 42801058132） | 52,488 / 196,597 | 26.70% | ~129,904 |
| **11-14（4214a625d）** | **62,896 / 197,163** | **31.90%** | **~119,496** |

四天净增 **+10,408**（+5.2pp）。口径：wpt.fyi aligned stable 三家并集
（chrome 157 / firefox 160 / safari 27.2 @ 76f3efc65c），分子 ZeroWeb
`--json` Pass 数；md 表「未覆盖」列 = 分母 − 通过（语料未接 + 已覆盖
失败两桶合计，逐组分账见 .json `groups`）。

## 2. 快照滞后本地刀链（下轮周跑显影预期）

CI 快照树态 ≈ g41 以远：css-images CI `zw_passes` 1,439 = g41 期 A/B
值；本地 g49-b1 已 3,144P（g42 +1145 / g43 +297 / g44 +60 / g45 +146 /
g46 +4 / g48 +49 / g49 +72 css-values 未入快照）。周跑下一次触发将
显影 g41→g50 全链增量，css 组通过数预期显著跳升（不含本轮新刀）。

## 3. css 组缺口分解（排程参考，按 den_subtests − zw_passes 降序）

**A. 已覆盖大量失败（修引擎面）**——covered_files 占比高、Pass 率低：

- css-grid 2,107/16,016（covered 648 文件仅 60 全过）
- css-typed-om 9,872/13,469、css-color 10,981/12,752、
  css-anchor-position 12,028/13,668（高水位收尾）
- css-fonts 2,527/8,694、css-shapes 1,839/7,527、css-selectors
  2,224/6,266、css-text 1,211/5,863、cssom 1,955/4,463
- css-transitions 201/3,439（covered 118 文件——超时截断伪影族在册，
  需区分真缺口）、css-flexbox 1,112/4,813

**B. 语料未接（扩语料面）**——covered_files≈0 或占比极低：

- css/WOFF2 0/298 文件、css-break 47/1,372、css-masking 60/574
  （126/6,407 子测）、motion 45/148（105/4,959）、css-page 23/263、
  css-paint-api 0/118、css-ruby 0/181、scroll-animations/css 0/114、
  css-view-transitions 141/545（19/1,447）、css-multicol 91/726、
  cssom-view 已高覆盖 206/253（567/2,696 修引擎面）
- CSS2 63/6,394 文件覆盖（1,014/2,566 子测——上游子测密度低，
  覆盖文件扩大是主杠杆）

## 4. 无关本流的旁证（本轮甄别）

- CI run 43630205461 benchmarks 红 = `tab_creation_100` 慢性族
  （GB-20261107 项 B 征询在途待用户拍板，zero-web 流面，不重复征询）。
- Weekly Build 43615261973 红 = benchmark-trend 同族；wptfyi-gap /
  reftest-trend / discover-shards 全绿，快照链路健康。

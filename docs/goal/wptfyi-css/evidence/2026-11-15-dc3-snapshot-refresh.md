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

## 3. css 组缺口分解（11-15 勘探修正：语料接线已完成，缺口 = 引擎能力面）

**语料面定谳（推翻本文件初版 B 桶「扩语料」框架）**：ci-shards.yml 已
接线 **65 套件**（css-face-a/b/c 一期 15 + css-v2-face-a/b/c 二期 31 +
css-v2-face-d 扩面 13 + css-variables misc 片）——coverage JSON 里全部
主要 css 组的 `zw_passes` 就是这些 corpus 的引擎实跑结果；masking/
motion/multicol/view-transitions 等均已在片（masking ∈ css-face-b，
本地 wpt-data 644 html 在位）。**未接线零散组仅 ~1.1k 子测分母**
（WOFF2 298 文件 0 子测、css-ruby 109、scroll-animations/css 754、
paint-api 28、layout-api 13、rhythm 183、exclusions/image-animation/
device-adapt/forced-color-adjust/fetching/style-attr/motion-path
合计 ~50）——语料杠杆已耗尽，**刀链（引擎能力）是唯一主杠杆**。

**A. 引擎能力面（主战场，按真实 gap 降序，g50 后口径）**：

- css-grid ~13.9k（2,107/16,016，covered 648 文件——taffy 架构面长坡）
- css-fonts 6,167 / css-values 6,260 / css-shapes 5,688 / transforms
  5,286 / masking 6,281 / selectors 4,042 / text 4,652 / motion 4,854
  （offset-path 引擎缺面）/ flexbox 3,701 / backgrounds 5,737 /
  typed-om 3,597 / transitions 3,238（timeout 截断伪影在册需甄别）/
  cssom 2,508 / cssom-view 2,129 / ui 2,205 / CSS2 1,552（文件覆盖
  63/6,394 是再杠杆——但子测密度低）/ color 1,771 / anchor 1,640

**B. 语料残余（次要，~1.1k）**：上列未接线零散组，按需随引擎面顺带
接线（无独立刀价值）。

**B′. masking 缺口再分解（11-15 语料×上游枚举对账，推翻「6,281 分散
属性支撑面」直觉）**：chrome summary css-masking 573 路径中仅 **65
文件载子测（6,398 子测）**，本地镜像逐路径在位（573 文件 0 缺——
此前「镜像大面积缺页」假设不成立，初期比对 0/65 是自算 path bug）。
本地 m4 语料 61 页 = **22 页 setup 即崩**（`declared tests: 0`，共享
插值 harness `interpolation-testcommon.js` `createTargetContainer`
返回 target 的 `.style` undefined，:451 `setProperty` 处炸）+
39 页真跑（上游子测 528，本地 121P/422F≈543 一致）。即 **5,859/6,398
（92%）堵在单一根因**（模板克隆 target 或 expando `targetContainer.target`
读回的 CSSOM style 桥面），而非分散属性面；探针件已备
（`zz-interpolation-testcommon-probe.js`，`ctc:` 日志点直接输出
`target.style`/expando 读回类型，css-color 曾走 `zz-caprobe-real.html`
同型探针），下轮刀取一 masking 页换 harness 即可定谳。该 crash 面
为共享 harness，预计同样压制其他组的 animations/interpolation 页
（transforms/motion/fill-strokes 待同法抽查）。**根因已由探针定谳并
升级为全域刀候选（337 页/56,365 子测）**，见
[target-expando-interpolation-unlock](2026-11-15-target-expando-interpolation-unlock.md)。

**C. 异常核查项**：css-navigation 与 css-parser-api 已在 css-v2-face-d
接线但 coverage covered=0——11-14 轮 fetch 失败「套件面记 0」或上游
路径漂移，体量小（83+71 子测），下轮周跑观察是否自愈。

## 4. CI vs 本地权威账对账（11-15 复核，快照树态定谳）

12 组本地权威账 vs CI 快照 `zw_passes`：**|Δ| ≤ 33 全部近似成立**（
transitions/animations/easing/motion/typed-om 五组精确相等；css-images
1439 = g41 期值）→ **快照树态 ∈ [g41, g42)**。真实缺口修正（den −
本地最新）：

| 组 | CI 账面 gap | 真实 gap（本地 g49-b1 口径） |
|---|---|---|
| css-images | 2,163 | **458**（本地已 3,144/3,602） |
| css-values | 6,418 | **6,260**（本地 4,185/10,445） |

其余组 CI=本地 → 账面 gap 即真缺口：typed-om 3,597 / transitions
3,238（118 文件已覆盖——真引擎面，timeout 截断伪影在册需甄别）/
cssom 2,508 / color 1,771 / anchor 1,640 / fonts 6,167 / transforms
5,286 / motion 4,854（offset-path 引擎缺面，纯扩语料无效）。当前 css
面真值 ≈ **64.8k**（快照 62.9k 滞后 ~1.9k + g50 在途未计）。

## 5. 无关本流的旁证（本轮甄别）

- CI run 43630205461 benchmarks 红 = `tab_creation_100` 慢性族
  （GB-20261107 项 B 征询在途待用户拍板，zero-web 流面，不重复征询）。
- Weekly Build 43615261973 红 = benchmark-trend 同族；wptfyi-gap /
  reftest-trend / discover-shards 全绿，快照链路健康。

# DC-3 · 目标级对账前置账：验收余量与语料跑道地图

2026-11-22。基线 coverage-2026-11-07（71,096）vs 当前 coverage-2026-11-22
（run 44996021189，106,394），同快照口径：**净 +35,298（+49.7%）**；DC-3
验收 ≥152,096（净 +81,000）→ **余量 45,702**。批十七 run 在飞（预期
webappapis 586→~1,537＝+951、dom 三键 +~1,394、base64 +32 → html 面
~108,800，余量 ~43,300），落账后按新快照复核本账（批十六收账见
[批十七证据](2026-11-22-dc3-batch17-webidlparser-alias-and-rerun.md)）。
下表 P 列为批十六快照值（syntax 2,747/infra 629/semantics 10,466 含
瞬态网络窗与抖动 −50，批十七预期回位）。

## 缺口主序（remain = den_subtests − zw_passes，快照口径）

| 簇 | P | den | remain | 文件覆盖 cov/den |
|---|---:|---:|---:|---|
| html/dom | 59,634 | 72,452 | 12,818 | 203/390 |
| html/semantics | 10,466 | 22,489 | 12,023 | 1,675/3,190 |
| html/rendering | 2,793 | 11,717 | 8,924 | 151/529 |
| html/syntax | 2,747（批十七回位 +43） | 9,013 | 6,266→~6,223 | 229/439 |
| html/canvas | 3,198 | 6,634 | 3,436 | 3,461/4,087 |
| html/webappapis | 586（批十七后 ~1,537） | 3,675 | 3,089→~2,138 | 207/372 |
| html/browsers | 257 | 3,327 | 3,070 | 283/841 |
| html/infrastructure | 629 | 1,664 | 1,035 | 110/160 |
| 从未拉取七簇 interaction/COOP/COEP/DIP/user-activation/anonymous-iframe/capability-delegation | 2 | 2,133 | 2,131 | 1/~653 |
| html/editing | 438 | 869 | 431 | 44/275 |
| **合计** | **106,394** | **160,124** | **53,730** | |

## 语料跑道地图（分母清单 − 本地语料，同源缓存 a3830cd8f7＝快照同批三家）

分母侧缺失 **2,768 文件／14,413 子测试**。Top 跑道（缺失子测试质量）：

1. semantics/scripting-1 **3,383**（235 文件：script-type-and-language-js
   .html/.xhtml/.svg 三形 1,368＋template-for/reflect-html-for 817＋
   json-module worker 变体；深层目录 fetch_dir_html 单层递归未覆盖形）
2. canvas/element **2,463**（471 文件：wide-gamut manual 族 ~1,300
   ——display-p3 环境受限风险高＋drawimage 族）
3. semantics/forms **2,449**（292 文件：the-input-element 深层
   type-change-state 380＋color.window 196＋type-change-state-weekmonth 82）
4. 从未拉取七簇 **2,611**：interaction/focus 840（209 文件全缺）＋COOP 658
   ＋COEP 599（credentialless 186 另计）＋DIP 326＋user-activation 114
   ＋anonymous-iframe 57＋capability-delegation 17
5. semantics/embedded-content **684**（315 文件：sizes-2d tentative 108＋
   persistentwidget tentative idlharness 57＋muted 51）
6. infrastructure/safe-passing-of-structured-data **286**（SAB battery 形，
   批十四补过支撑文件、页面本体未拉）
7. semantics/document-metadata **236**＋permission-element **168**＋menu 15
   ＋popovers 25＋interestfor 24＋selectors 12
8. browsers 簇 **525**：browsing-the-web 213＋windows 126＋origin 96＋
   windowproxy 55＋history 36＋sandboxing 35
9. editing 簇 **266**：editing-0 148＋dnd 54＋hidden 48＋focus 10＋activation 6
10. 长尾 ~700：syntax/parsing 71＋dom/elements 30＋documents 24＋
    render-blocking 10＋webappapis/scripting 39＋microtask 12＋timers 10＋
    rendering/bidi 67＋replaced-elements 15 等

## 在库未覆盖面（语料在盘、ZW 未入账文件）

dom **151**／semantics **531**／rendering **339**／browsers **280**／webappapis
**138**／editing **146**／syntax **51**／infra **42**／xhtml 3／canvas 1 文件。
本地九套件合并覆盖数与快照逐组核对：dom 203＝203、webappapis 207＝207、
rendering 151＝151、syntax 355＝355、infra 110＝110、xhtml 12＝12（六组
presUNC 精确）；semantics/browsers 差 278/136 文件＝补簇套件在他分片，本地
计数偏大（真值 531/280），精确形态待补分片产物定谳。

**精确组形态定谳（911S/987S/336S/208S）**——主导形＝**worker 面挂账**
（批一/批四在册，非套件配置缺口；初稿「本地双 scope 实证可跑」勘误——
批四 base64 380 子测试＝窗口面全量，worker 通道域外）：dom
idlharness.any.worker.html 808S、webappapis base64.any.worker 380S＋
structured-clone.any.worker 137S＋promise-rejection-events
dedicated/sharedworker 72S，待引擎 worker lane 碰头。
dom/elements global-attributes 101 文件＋bidi 27 文件＝浏览器 skip 零质量
占位（合计 <10S，不追）。余散形：syntax serializing.html 单文件 238S＋
charset 44S（扫描域缺口）、infra safe-passing-of-structured-data 电池页
117S（批九键清单外）、dom documents/dom-tree-accessors 63S、webappapis
window/svg 形 ~100S。

**CI 分子缺口截断扫描（145 键/29,397S gap>20）重分类**：无大额 CI 截断
桶——① dom idlharness.https 三查询键 6,131S＋event-handler 族＝
WebIDLParser 生成路由缺档（上游非 Git blob，批十六勘误定谳；57/59 改拉
webidl2 真身落别名，批十七在飞；本地定向单跑实证三键注册 5,822 子测试
1,397P）；② reflection-*/processing-instruction/valid-custom-element-
names 等全过面 ≈90-96% den＝shim DOM 静默缩量（批一勘误在册：本地 2.4s
rc=0 排除超时），引擎流域挂账；③ base64.any.js −94 尾＝跨 topdir 运行期
fetch_json 缺档（勘误初判双 scope，57 脚本已补 base64.json，批十七兑现）；
④ Range-mutations/reference-target/svg-embedded-sizing＝邻居账与行为面。
配套勘误：批十六证据 base64 归因已改。

## 收口判断

余量 45,800 构成＝语料缺失 ~14,413（全过上限）＋在库未覆盖质量（未定谳，
被簇 remain 上界约束）＋行为面（引擎流域挂账刀位：UA 样式面 ≈4,400、
popovers 2,744、rendering ≈9,164、speculative TO 面、document.write 管道等，
见 master 卡点）。本流杠杆＝跑道批次（批十七起逐簇拉取＋套件域扩展）；
行为面待引擎流碰头兑现，本流不改 engine。

[逐文件 JSON](2026-11-22-dc3-goal-accounting-and-corpus-runway.json) ·
[批十六证据](2026-11-22-dc3-batch16-reconciliation-and-closure.md)

## 勘误（2026-11-22 批十八备料轮）：跑道第 6 项 safe-passing 重分类语料→行为

本图第 6 项「infrastructure/safe-passing-of-structured-data **286**（SAB
battery 形，批十四补过支撑文件、**页面本体未拉**）」前提为假——62 脚本
26 目录清单第 44–47 行整目录拉取 safe-passing 四层（本体+resources+
shared-array-buffers+shared-array-buffers/resources，GOAL_PULL_ANY_JS=1
带 4 个 .any.js 源），批九时代已在 CI 分子内；头注释自证「生成形分母 8 键
306 子测试」。本地全子树对照 pin 315976933（96 blobs 递归）仅缺
WEB_FEATURES.yml ×2（元数据不拉）——**零语料缺口**。

真实缺口两形（行为刀位，非跑道）：① 本地守卫过滤跑
`testharness-html-infra safe-passing`（472 结果行）**0 Pass**——根因
`TypeError: Right-hand side of 'instanceof' is not an object`（测试脚手架
`x instanceof MessagePort` 类检查在 runner 车道无接口对象→全案 unhandled
rejection；~150–200S 面）；② structuredclone_N.html 电池页 iframe 内容门
零执行记账 117S（批九键清单外同源）——两形均待引擎流域碰头，本流不改
engine。跑道主序数字据此修正：缺失 2,768 文件/14,413 子测试含此项 286S
应剔除（下批对账时按新快照统一重排）。本勘误与批十七 WebIDLParser 勘误
同类（前提未核即入图，第 ② 号教训：入图前先跑通道核对面）。

## 勘误二（2026-11-22 批十九轮）：跑道地图方法学重排——「分母−本地语料」假缺口定谳，拉取杠杆耗尽

本图全表用「分母清单 − 本地语料」求缺口，混入两形假缺口。批十九用
run 45008050513 全 55 分片产物并集（**分子键集 74,174 键**）对照分母
（summary-chrome a3830cd8f7，html 10,332 物理文件）重测：**未计分
5,201 文件/25,534S**，三分类：

| 桶 | 文件 | 质量 | 定性 |
|---|---:|---:|---|
| 在库未计分（corpus 在、分子无键） | 1,705（21 文件 variant 形已计分修正） | 10,105S | 扫描域缺口＋skip 规则形＋worker 通道挂账＋行为面 |
| pin 可拉未计分 | **0** | 0S | **语料拉取杠杆耗尽** |
| 上游漂移（pin 315976933 后新增） | 3,496 | 15,429S | WPT_REV bump 杠杆（共享面跨流决策） |

假缺口成因：CI 分片每次 fresh checkout 从 pin 全量拉取，本地语料陈旧
（25 脚本 CI 计分 115 键、本地 corpus 1/177 的 the-input-element 即证）
——「本地缺」≠「分子缺」。forms 簇（原图第 3 项 2,449S）case study：
pin 可拉 170 文件中 119 键 ≈1,750S 已被 html-sem CI 分子计分，余 51 键
仅 **22S** 且全为 crash/repaint/reftest/xhtml 零质量形，115 文件/888S
为漂移。原图第 2 项 canvas/element 2,463S 与第 4 项七簇 2,611S 全属
漂移桶（drift 大头：semantics 4,593S／canvas 3,896S／webappapis 1,389S／
browsers 1,034S／dom 980S／interaction 834S——七簇整目录 pin 后新增）。

在库未计分 10,105S 分解（扫描域接线候选，上界非承诺——逐簇甄别
skip 规则后才能定承诺面）：dom 4,135S（其中 idlharness.https.html 基键
3,955S 为 variant 记账形——runner 出 3 查询键不出基键，非缺口）＋
semantics 3,322S（scripting-1 1,429S worker-only 挂账＋embedded-content
1,028S media/iframe 排除域为主）＋browsers 1,082S（history 340S/
73f＋browsing-the-web 227S/123f＋origin 129S 纯 .html 接线候选）＋
syntax 903S/51f＋infra 327S/33f＋webappapis 272S/83f（纯 .html 形五簇
接线候选上界 ~2,069S）。

**跑道「批次二十起逐簇拉取」计划撤销**；DC-3 余量路径重排为三杠杆：
① 扫描域接线批（在库未计分甄别）；② pin bump（15,429S 杠杆，
WPT_REV 属共享面提请用户/碰头拍板）；③ 行为刀位（engine 碰头域，卡点
在册）。验收 ≥152,096 的可达性取决于 ②③ 兑现面，① 上界不足 11,000S。

# DC-3 · 目标级对账前置账：验收余量与语料跑道地图

2026-11-22。基线 coverage-2026-11-07（71,096）vs 当前 coverage-2026-11-22
（run 44985566993，106,296），同快照口径：**净 +35,200（+49.5%）**；DC-3
验收 ≥152,096（净 +81,000）→ **余量 45,800**。批十六 run 44996021189 在飞
（预期 webappapis 438→~1,505＝+1,067 → ~107,363，余量 ~44,733），落账后按
新快照复核本账。

## 缺口主序（remain = den_subtests − zw_passes，快照口径）

| 簇 | P | den | remain | 文件覆盖 cov/den |
|---|---:|---:|---:|---|
| html/dom | 59,634 | 72,452 | 12,818 | 203/390 |
| html/semantics | 10,467 | 22,489 | 12,022 | 1,675/3,190 |
| html/rendering | 2,793 | 11,717 | 8,924 | 151/529 |
| html/syntax | 2,790 | 9,013 | 6,223 | 355/439 |
| html/canvas | 3,198 | 6,634 | 3,436 | 3,461/4,087 |
| html/webappapis | 438（批十六后 ~1,505） | 3,675 | 3,237→~2,170 | 207/372 |
| html/browsers | 257 | 3,327 | 3,070 | 283/841 |
| html/infrastructure | 635 | 1,664 | 1,029 | 110/160 |
| 从未拉取七簇 interaction/COOP/COEP/DIP/user-activation/anonymous-iframe/capability-delegation | 2 | 2,133 | 2,131 | 1/~653 |
| html/editing | 438 | 869 | 431 | 44/275 |
| **合计** | **106,296** | **160,124** | **53,828** | |

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

## 在库未覆盖面（语料在盘、ZW 未入账文件，计数口径含分片收录偏差作指示用）

dom **151**／semantics **531**／rendering **339**／browsers **280**／webappapis
**138**／editing **146**／syntax **51**／infra **42**／xhtml 3／canvas 1 文件。
两形：套件扫描域未含（fetch 有、suite 域窄），或 runner 形态不消费
（.xhtml/.svg/manual 门）。质量未逐文件定谳（需分片产物逐键），先入地图作
跑道批次后候选。

## 收口判断

余量 45,800 构成＝语料缺失 ~14,413（全过上限）＋在库未覆盖质量（未定谳，
被簇 remain 上界约束）＋行为面（引擎流域挂账刀位：UA 样式面 ≈4,400、
popovers 2,744、rendering ≈9,164、speculative TO 面、document.write 管道等，
见 master 卡点）。本流杠杆＝跑道批次（批十七起逐簇拉取＋套件域扩展）；
行为面待引擎流碰头兑现，本流不改 engine。

[逐文件 JSON](2026-11-22-dc3-goal-accounting-and-corpus-runway.json) ·
[批十六证据](2026-11-22-dc3-batch16-reconciliation-and-closure.md)

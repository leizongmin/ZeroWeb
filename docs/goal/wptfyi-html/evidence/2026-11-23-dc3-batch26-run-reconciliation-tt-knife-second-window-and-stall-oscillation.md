# DC-3 · 批二十六：run 45077609991 收账——TT 刀②③③a 兑现（tt +87）＋CE·SD 持位＋停摆形交替震荡（dom −247 / editing +223 回位）＋track-cues 勘误

2026-11-23。run 45077609991（11-22 22:25:09Z dispatch，~2h31m 止
00:56Z，checkout f6a733272）对批二十五 run 45063183178 同分片逐键 A/B；
快照 coverage-2026-11-23（497767149 回写，generated 00:26:55Z，aligned
a3830cd8f7 分母恒 2,278,575）。窗口码变（781ca4bbe..f6a733272）七件：
**③b② adbe78ebb（TT HTML sink 13 钩点）＋③③a e02763ddc（TT script
sink timer）**＋f6304c423（html5lib test.js 语料补录，CI smoke 域）＋
e6868effd（css-parser background-size math 修复）＋a5a07b71e（engine
clippy 机械债）＋f77adb751（fmt 14 文件）＋41a8f1c4c（renderer T95
mark）。刀③③b 081d40b4a 于 22:41Z 落 main＝checkout 之后，**在窗外**，
归批二十七窗 drift。

## 目标级账

html 面 109,427 → **109,428（+1）＝净 +38,332/基线 71,096，验收
≥152,096 余 42,668**。+1 唯一构成＝track-cues Fail→Pass（见勘误）。

## track-cues 勘误＋交替带定谳

- **批二十五证据勘误**：批二十五记「track-cues 持 Pass 第二连演」有误
  ——批二十五 artifact 实为 **Fail**（P=0）；批二十五 html/semantics
  组行 10,902→10,903（+1）实为 onblur +2 与 track-cues −1 的**净和**，
  当时把组行持位误读成单键连演。批二十四「flaky 候选」判定维持。
- 四演真形：b23 Fail → b24 Pass → b25 Fail → **b26 Pass**——事件时序
  **交替带（2 Pass/2 Fail）**，非收敛三连演，「两连 Pass 可判稳」前提
  不成立；判定＝flake 带内震荡，不再逐轮追。
- onblur 持位：checkable-active-onblur-with-click 5P/2F（批二十五 5P
  水位保持）。

## 邻账兑现（③b②＋③③a）

- **tt 套件 1,281 → 1,368（+87）**；快照组行 trusted-types 1,256 →
  1,343＝**+87 精确同值（清单内全落第二演）**。21 键变动全在
  block-string-assignment-\* 族（③b② 13 钩点域：Document-write 2→10、
  parseHTMLUnsafe 1→5、DOMParser 0→2、setAttribute 12→14、outerHTML
  0→1、setHTMLUnsafe 1→3 等）＋Window-setTimeout-setInterval 2→6
  （regs 3→6，③③a）＋HTMLScriptElement-internal-slot 8→9。F 674→649、
  TO 68→64。
- **san 套件 323 持位**（652 subs 同值）＝③b②/③③a 不触 sanitizer
  套件。清单外三键（eval-csp-tt ×2＋sethtml-tentative）维持 zw_only
  持位不追。

## CE·SD −2,772 持位（第二窗，不修不加深）

- property-reflection-idl-setters **225P/2,685F（注册 2,910 不变）**
  与批二十五全数同值；property-reflection 2,895、DSD-attachment 586、
  DSD-basic 13、clonable 3、reactions/Node 9 全同值。③b②/③③a
  既未修复也未加深该回归；③③b 在窗外。对端修复读数仍以本通道为准。

## 停摆形第三采样：dom/ranges 与 editing/run 交替互补

- **dom/ranges −247**（组行 42,420 → 42,173）：dataChange 1,016→908
  （−108）、replaceData 1,052→913（−139）、replaceChildren P 持 29
  （1 子测试 Timeout→None）。窗内 dom 侧无码变（a5a07b71e 零行为
  lint、f77adb751 纯格式）。
- **editing/run +223 完全回位**（95,130 → 95,353 ≈ 批二十四水位
  95,350）：delete.html?7001-last 586→635P（regs 636→702）、
  multitest.html?9001-last 918→1,092P（regs 919→1,092 全注册恢复）。
- 三窗序列：b24 dom −350/editing +96 → b25 dom +120/editing −220 →
  b26 dom −247/editing +223——**交替震荡、三窗零码关联＝timing 形
  定谳强化**（停摆形归负载/时序带，非确定码行为；4fb594667 机制归因
  维持降级）。后续读数以多窗水位带为准，单窗回位/加深不作修复证据。

## 观察清单全持位

webappapis 1,540（207 files）／per-global 6 注册 3P/3F／dat 170 键
shapes 逐字节同（148TO/19F/3 混合——write TO 恒定判定续持；f6304c423
test.js 补录为 CI smoke 语料缺口修复，不触 coverage dat 面）／
interestfor −3 键族逐字同／variant 三键 DISTINCT 且 regs/P 全同
（442/48/1,003，nameset 569d8f/d83306/a823af 持位）／合成 Fail 扫描：
fetch-module 2 键＋import-maps 3 键持位、os error 2 html 面清零持位、
subsetTestByKey 0。

## 其余组漂移（css 镜像震荡带）

css-animations −14→**+14**、css-borders +7→**−9**、css-backgrounds
−1→+2、css-writing-modes −1→+2、css-values +0→−1、
web-animations/interfaces −1→+1——批二十五负漂移本窗镜像回摆＝带内
震荡（e6868effd 在窗内但 css-backgrounds 仅 +2，量级不符主归震荡），
watch 不深挖。组差合计 **+73**＝zw_passes 1,928,439 → 1,928,512
逐项闭合（tt +87、editing/run +223、dom/ranges −247、semantics +1、
css/web-animations 带 +9 等）；本地 wptfyi_gap.py coverage 同 cache
重跑三 totals 逐字复现（74,184/2,278,575/1,928,512 @ 84.64%）。
环比护栏未触发（本窗正向）。

[逐文件 JSON](2026-11-23-dc3-batch26-run-reconciliation-tt-knife-second-window-and-stall-oscillation.json) ·
[批二十五证据](2026-11-23-dc3-batch25-run-reconciliation-tt-cluster-both-faces-and-ce-sd-spillover.md)

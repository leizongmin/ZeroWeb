# DC-3 · 批二十七：run 45091003913 收账——TT 两刀双向账（③③b +88 逐项精确兑现＋③⑤ 名单域 +11＋交互窗 +15）＋负向扫溢出两形（test262 −13／csp −4 单窗不定谳）＋face −4 全落 flake 带＋CE·SD 第三窗持位

2026-11-23。run 45091003913（00:49:22Z dispatch，~2h19m 止
03:08Z，aggregate 03:05:47Z 快照落 main＝7542fd567）对批二十六
run 45077609991 同分片逐键 A/B。分片 checkout 实测（60 覆盖分片全查，
`git log -1 --format=%H` 逐 job 提取）：**45 分片 dc81c2168＋10 分片
956c94cef**（后者 docs-only 领先，码内容同 dc81c2168）＝全分片同码。
窗口码变：**③③b 081d40b4a**（11-22 22:52:57Z 落 main＝批二十六
checkout 之后、本窗 dispatch 之前＝批二十六记「窗外归批二十七」兑现）
＋**刀③⑤ dc81c2168**（00:49:26Z＝dispatch +4s，全分片 job start
00:49:47Z+ 之前＝在窗）。**DP①② 4c1816d14（01:23:11Z）在全部 job
start（≤00:58:01Z）之后＝窗外**，归批二十八窗首观察对象（domparsing
face 零位移互证：组行 domparsing 未入组差清单）。

## 目标级账

html 面 109,428 → **109,424（−4）＝净 +38,328/基线 71,096，验收
≥152,096 余 42,672**。−4 全落 html/semantics flake 带，零窗口码归因：

- onblur checkable-active-onblur-with-click **5P→3P**（b24 首窗 5P→
  b25 5P→b26 5P→b27 3P＝水位下修候选，续 watch 不追单窗）；
- track-cues-sorted-before-dispatch 1→0＝交替带第五演（F→P→F→P→**F**），
  与批二十六「交替带不再追」判定一致；
- the-dialog-element/modal-dialog-in-visibility-hidden 3→2（单子测试
  P→F，regs 3 恒＝新形 flake 候选首演）。

## 邻账兑现：TT 两刀双向账（tt 套件 1,368 → 1,482，+114P，F 649→536，TO 64 恒等）

- **③③b 目标面 +88 逐项精确兑现**（对端自报 88P 与 CI 逐键同值）：
  eval-function-constructor 1→65（masks 64）＋eval-csp-tt-no-default-policy
  +6＋eval-csp-tt-default-policy-mutate +4＋Window-block-eval-function-
  constructor +4（Function/GeneratorFunction/AsyncFunction/
  AsyncGeneratorFunction 四构造器）＋…applying-default-policy +8＋
  …throwing +1＋eval-with-permissive-csp +1＝7 键 +88。
- **③⑤ 名单域 +11**（9 键）：createPolicy-cspTests 五页 +6
  （none-none +2、no-name/case2/noNamesGiven/none-none-name 各 +1）＋
  duplicate-names-list +1＋wildcard-with-trailing-characters +1
  （`*X` 无效表达式阻断）＋sandbox-allow-scripts +2＋inheriting-csp-for-
  local-schemes +1（srcdoc 继承）。对端自报 +10P/-10F，CI +11＝差 +1
  落语料差/域交集，不深挖。report-only 页 regs 1→2（新子测试注册
  None→Fail，+0P）。
- **交互窗 +15 如实记**：default-policy +12＋empty-default-policy +3
  ＝两刀（编译门×名单指令）共同覆盖域，单刀自报均不含。
- **快照组行 trusted-types 1,343 → 1,447（+104）**＝套件 +114 − 清单外
  +10（eval-csp-tt ×2 键 +6+4 维持 zw_only 分母外）——组行/套件两口径
  逐项闭合（批二十四同款口径差预登记兑现）。san 套件 **零键变化**（323
  持位）＝③③b/③⑤ 不触 sanitizer。shadow-dom 组行 +1＝declarative-
  shadow-dom-opt-in regs 116→119（XHR document.open 子测试新注册，
  timing 带候选）。

## 负向扫溢出（对端刀双向账第三演，证据链移交对端）

- **test262 face −13**（third_party 组行 51,920→51,907）：13 键全部
  **Function 构造器语义域**、regs 全部 1→1 恒等、P→F 翻转——
  AsyncFunction-is-subclass／AsyncGeneratorFunction、GeneratorFunction
  /prototype/constructor、subclass-builtins/subclass-Function ×2、
  create-function-parse-before-getprototype ×6（builtins 3＋staging/sm 5）
  ＋extendBuiltinConstructors、boundFunctionSubclassing、
  newer-type-functions-caller-arguments、sm/generators/runtime。签名与
  ③③b 编译门域正交重叠（非 TT 语境「原参透传零变化」自报与该形冲突，
  collateral 嫌疑高置信但未定谳）；test262 分片不涉 ③⑤（无 CSP 指令）。
- **csp face −4**（203→199，subs +5 上游漂移另述）：script-src-1_4_1
  −3（window.setTimeout()/setInterval() P→F＋securitypolicyviolation
  事件 P→None＋页级 Timeout 在场）、script-sample-no-opt-in −1
  （eval-alikes sample 子测试注册消失 regs 2→1）、unsafe-eval
  eval-scripts-set{Timeout,Interval}-blocked regs 1→5/1→2（页面推进
  新注册 5 子测试全 Fail，+0P）——timer/eval-under-CSP 签名与
  ③③a/③③b sink 域、③⑤ CSP 解析域交叠，但页级 Timeout 在场＝
  timing 嫌疑并存；**单窗不定谳**（批二十五纪律），下窗回位/加深再判。

## CE·SD −2,772 第三窗持位（不修不加深）

property-reflection-idl-setters 225P/2,685F（注册 2,910 不变）与批二十
五/二十六全数同值；property-reflection、DSD-attachment/basic、clonable、
reactions/Node 五键族逐字同。③③b/③⑤ 既未修复也未加深；对端修复读数
仍以本通道为准。DP①②（CustomElementRegistry 域）在窗外，批二十八窗
读数时注意其 wc 面溢出（对端自报 +24P registries 域）。

## 停摆形第四演＝双带内

- **dom/ranges 42,173 → 42,371（+198，带内 42,173~42,420）**：
  dataChange 908→983、replaceData 913→1036（b25 水位 1,052 附近）。
- **editing/run 95,353 → 95,358（+5，带缘 95,130~95,353 上缘外溢 +5）**：
  delete.html?7001-last 635→675（regs 702→756）＋forwarddelete.html
  ?7001-last 406→371（regs 491→446）净和。
- 四窗序列 b24 −350/+96 → b25 +120/−220 → b26 −247/+223 → b27 +198/+5
  ＝交替震荡 timing 形续证；dom/ 邻账归 wptfyi-dom，本流只读。

## 观察清单全持位

webappapis 1,540/207 files/subs 2,540 逐字同；per-global 6 注册 3P/3F
names 全同；dat 170 键 shapes 148TO/19F/3 逐字节同（write TO 恒定续持）；
interestfor 键族逐字同；variant 三键 DISTINCT regs/P/nameset 569d8f/
d83306/a823af 全同（同值检测例行）；合成 Fail 扫描 **103=103 零漂移**
（子测试级逐条比对零新零消；html 面 fetch-module 2 键＋import-maps 3 键
持位、os error 2 html 面清零持位、subsetTestByKey 0）；css 组行 +14
（css-borders +16 corner-shape-interpolation、css-backgrounds −2
parsing/computed＝css 带镜像震荡续，rendering 流域不深挖）。

## 快照闭合

组差合计 **+301**＝zw_passes 1,928,512 → 1,928,813 逐项闭合（tt +104、
dom +198、css +14、editing +5、shadow-dom +1、csp −4、third_party −13、
html −4 等）；full_pass_files 63,640 → 63,634。**分母漂移另报不入账**：
aligned manifest 本窗由 CI aggregate 新拉刷新（a3830cd8f7 → 063eb7cae2，
全量子测试分母 2,278,575 → 2,279,168＝+593，html 160,124 → 160,190
＝+66；svg +350/css +51 等）＝上游 manifest 漂移，分子不受影响；本地
复现（旧 cache a3830cd8f7）face 行同值 109,424 双证。环比护栏未触发
（本窗正向）。

[逐文件 JSON](2026-11-23-dc3-batch27-run-reconciliation-tt-dual-knife-verbatim-and-negative-spillover.json) ·
[批二十六证据](2026-11-23-dc3-batch26-run-reconciliation-tt-knife-second-window-and-stall-oscillation.md)

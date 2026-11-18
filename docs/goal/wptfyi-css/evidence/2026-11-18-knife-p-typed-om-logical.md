# 刀 P：typed-om known 门 + toStringTag + radius opaque（2026-11-18 凌晨）

## 靶面

b0（results-knifen2-b1，bin `8833233e`）typed-om 全 corpus 定谳：
`the-stylepropertymap/properties/` 9 页 ~942F 同根因——`_zwOmPropSyntax`
known 门缺表条目，SET 全落 `part10-typed-om.js` "Unsupported property"
（logical 472F＝46 逻辑属性全单形；margin/padding/border-{width,color,
style,radius}/scroll-margin/scroll-padding 物理长手同门）。

## 根因与机制（单文件 shim 刀，part10-typed-om.js）

**机制①（known 门表扩 + computed 三臂 + 逻辑翻译）**：`_zwOmPropSyntax`
+~86 条——margin/padding/inset 物理+逻辑长手与轴简写、border 物理+逻辑
width/color/style、物理+逻辑 corner radius、scroll-margin/scroll-padding
物理+逻辑（引 css-typed-om-1/css-logical-1/css-scroll-snap-3）。
`_zwOmNonNegativeProps`：−inset 6（负值合法页面 equal 期望）、+border 轴
width 2 + 逻辑 corner 4、+scroll-padding 长手 8（页 range-handling 期望
Sum——首轮据 grep 无 range 回调误剔，探针 "expected CSSMathSum but got
CSSUnitValue" 定谳后回补）。`_zwOmComputedProp` 三臂：宽度关键字
（thin/medium/thick→1/3/5px，css-backgrounds-3「computed 独立于
border-style」）；padding 负 % 钳零保单位（'0%'，引擎给 0px 不合期望）；
currentcolor 保形（css-color-4 keyword 面）。`_zwOmLogicalPhysical` 22
条逻辑→物理 gCS 咨询翻译（horizontal-tb+ltr 假设 FIXME writing-mode；
轴简写不入表——R5 探针证 host 无展开、翻译物理槽位会毒化 equal 面）。

**机制②（Symbol.toStringTag）**：`assert_class_string` 按
`Object.prototype.toString` tag 判类——shim 类缺 tag 是 b0 全 corpus
337 unsupported + em/cm/calc 族面失败根因（孤立流 introspection 探针
p4/p5 定谳：解析/包裹全对，唯类名面灭）。CSSStyleValue/CSSKeywordValue/
CSSUnitValue/CSSUnparsedValue/CSSTransformValue/CSSImageValue + math
工厂全子类覆写 tag（镜像 part09b.js:943 模式，try/catch 包裹）。
**corpus 级杠杆**：typed-om 腿 F2P 页集 9 页→120 页即此机制
（unsupported 归一面/子类断言面/简写空表回落面全 corpus 生效）。

**机制③（scroll-padding 非负回补）**：见机制①括注，独立成项因首轮
剔错——判定教训：grep 静态无回调 ≠ 页面无 range 期望，以页断言为准。

**机制④（radius computed opaque + 简写空表）**：border-radius.html 页
契约——四物理角 `computed: assert_is_unsupported`（css-backgrounds-3
computed 双值对，Typed OM L1 不支持→基类 opaque）而 declared 面仍逐
语法解析 equal；简写 `runUnsupportedPropertyTests` 要求 opaque 往返
（set 收基类实例、get 基类、文本逐字）。修：`_zwOmComputedUnsupportedProps`
仅物理四角 + computedStyleMap 闭包 `__zwOmComputedFace` 置位 +
mapGetAll computed 面分支回落 `_zwOmOpaque()`（declared 面不受影响——
logical.html 逻辑角 computed 默认 equal 期望，不在此列，零 P2F）；
`'border-radius': []` 空表条目走 margin 简写同款 opaque 往返臂。

## P2F 定谳修复（A/B 网捕获）

机制①宽度关键字臂原用泛 `/-width$/` 尾缀门——劫持 **scrollbar-width**
（css-scrollbars-1 computed 为 as-specified 关键字；decl 'thin'→臂返
1px→表无 `<length>` 族→`_zwOmOpaque()` 基类，b0 通过面反转）。
**b1 电池（dda6ab89）typed-om 腿 P2F=1 捕获**；peer knife-a1b 电池
typed-om 腿（其 bin 扫入本刀 WIP 机制①②）同面 P2F=1 独立复现。
修：正例族门 `border-*-width / outline-width / column-rule-width`
（arm 获益面 border-width 48F/column-rule-width 14F/outline-width 3F
全保；其余 -width 尾缀属性声明值永不为 thin 族，臂惰性安全）。
修复随 b2（`2faea6ed`）落账，b1 留档作捕获证据。

## A/B（六腿，b0=knifen2-b1 `8833233e` → b2=results-knife-p-b2）

bin 谱系：fa17c8dc（宿主探针）→ ece56010（bin2＝①②③+回补）→
dda6ab89（b1＝+④；含 knife-a1b R2 在途码惰性 + 第三方 ZW_TRANS_DEBUG
env 门零行为）→ **2faea6ed（b2 在册＝P2F 修复）**。
发射前置：探针页 zz-lei-p1~p5 删净（03:52 核实零残留）；peer a1b 电池
03:49:07 GATE-DONE 收尾后错峰发射（systemd --user 单元双轮，
MemoryMax=12G）。

**typed-om 靶腿**：F2P=1125（120 页）、P2F=0、lost=0、页集合全等
（b1 同 1125 但 P2F=1＝scrollbar-width 捕获面，修复后 b2 面回归 b0
Pass 形非新增翻转——两轮算术对账 1125+1P2F 修复＝1125+0 精确吻合）。
**TOTAL gained=2410 lost=0 regressed=0 pages_gain=153 pages_loss=0**
（ab-compare 与逐腿独立复算双口径一致）。

逐腿（b2 vs b0，全腿 lost=0/regressed=0/pages_loss=0 零容忍成立）：

| 腿 | gained | 归属 |
|---|---|---|
| typed-om | 1125 | **本刀**（表扩+tag+radius；靶面 9 页 ~942F→残 146F） |
| fonts | 856 | 刀 N/n3 系（已账 +844+12 漂移，master 在册不重账） |
| masking | 336 | 刀 A1 +273（已账）+ 刀 A1b +63（已账） |
| gaps | 54 | 刀 A1b（已账） |
| multicol | 22 | 刀 A1b（已账） |
| shapes | 17 | 刀 A1b（已账） |

**本刀净账 +1125**（对照腿 gained 全为 b0 基线后已入账提交件，
窗口法证 + CLAIM 双归因，不重账）。

**peer 自然验证账**：knife-a1b 电池 typed-om 腿（bin 含本刀①②、无
③④）vs b0 = F2P 1089；本刀 b1 = 1089 + 36（③④ radius 32 + 简写 4）
——差值逐面精确吻合，③④ 无新增 P2F（其账面 "typed-om +96" 与其
battery json 实测 F2P=1089 不符，以其 json/TOTAL=2374 为准，已注记）。

## 残面挂账（typed-om 靶面残 146F，归 P2 池不扩刀）

- axes 简写（margin-block 等 6 键）computed em/cm/calc：declared 回落
  形（host 无展开，翻译会毒化 equal 面——R5 探针定谳）；
- calc-% 族：specified same-unit 折算（calc(0%+0%)→CSSUnitValue，
  存在声明替换路径）+ computed 引擎 % 折算（Incompatible units）；
- block-size 族 15F：引擎 gCS block-size 臂缺失（gCS 空回落 declared
  →单位/包裹形错）；
- scroll-padding/inset % 引擎形（approx expected ±3.14 got 0）；
- radius 全清（36F→0F）。

## 探针与纪律

宿主能力探针 5 页（zz-lei-p1 行为面/p2 隔离/p3 残面/p4 introspection/
p5 精确子测流复刻）battery 前删净；跨 clone 归属按 /proc cwd/exe
（ZeroWeb-3 editing 腿非本树，解除构建冻结误判）；构建窗与 knife-a1b
错峰（其 cargo 03:49 先行，本刀 0.2s 缓存命中确认同树谱系后重构建）。

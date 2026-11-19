# 刀 face C part 1：shim typed-om 数值面（computed 规范化 + to() math，2026-11-19）

## 目标与边界

face C 残面第一刀：typed-om 数值面单机制族（探针分诊 + testsuite.js 合同
定谳后收窄到 shim part10-typed-om.js 单文件）。typed-om 全语料普查
1139F/165 页中，本刀触达五形数值族上界 296F；G/G2 白名单族 188F
（shorthand 门/未知属性）与 Z:other 647F 独立机制不入本刀（后续刀候选）。
engine computed 序列化域残面（width 负值 set 拒收、min/max computed
echo、column-count/tab-size computed number echo）归 face C part 2，
避让 computed_style.rs（animation-range face B WIP 在飞，落库前不入）。

## 机制定谳（探针 zz-fcprobe.html + testsuite.js 合同源）

testsuite `testPropertyValid` 双读合同：set 对象 → specified
attributeStyleMap.get + computed computedStyleMap().get，五形失败全在
computed 侧（specified range_handling 包裹先过），一形在测试自身
`input.to(...)`：

1. `-3.14em`/`calc(0px + 0em)`（assert_is_unit('px')）→ 期望
   CSSUnitValue 类 + px 单位；本实现 computed 返 CSSMathSum（负值包裹
   不分 face）或原样单位。
2. `3.14cm`（assert_is_unit('px')）→ 期望 px；本实现返 cm（gCS echo
   未换算属性族）。
3. `-3.14%`（grid/scroll-padding 族 clamp 回调）→ 期望钳
   CSSUnitValue(0,'percent')（css-grid-2 §7.1.1 轨道负 % used/computed
   钳零）；本实现返 Sum 包裹。
4. `calc(0% + 0%)` → 测试先调 `input.to('percent')`（对 CSSMathSum），
   本实现 `to()` 仅处理 CSSUnitValue（`this._zwOmUnit` undefined →
   throw "Incompatible units"）——number-calc 页（column-count 等
   `input.to('number')`）同源。
5. margin-top/left vs right/bottom 声明文本不对称：引擎 CSD 把
   `calc(0% + 0%)` 折叠为 `calc(0%)`（top/left 侧），shim 解析单 term
   calc 返裸 CSSUnitValue → `assert_is_calc_sum` 类断言挂。

## 码面（part10-typed-om.js 单文件 5 臂）

1. `CSSNumericValue.prototype.to` math 值臂：展开（_zwOmFlattenToUnits）
   逐成员按因子换算加总返单 CSSUnitValue（css-typed-om-1
   §dom-cssnumericvalue-to）；percent/number/flex 同名直取不加因子；
   余量/异族成员 TypeError 保留。
2. `_zwOmComputedCanonicalize`（computed face 专用）：长度绝对化 px
   （em/rem 按宿主 computed font-size——css-values-4 §6；绝对族按因子
   表）、同族 calc Sum 折叠（length→px/percent/angle→deg 三族，time 等
   宿主已 canonical 不折——P2F 收窄）、grid 轨道负 % 钳零集
   （grid-template-columns/rows + grid-auto-columns/rows）。
3. computed 面负值 Sum 包裹豁免（declared 面保留 range_handling 形——
   testsuite computed 断言期望裸 Unit）。
4. declared 面单 term calc 保 Sum 壳（css-typed-om-1 §parse a calc()：
   单值结果仍 math 节点 Sum 包裹，与 CSSNumericValue.parse 同形）。
5. `_zwOmKnownExtra` +scroll-margin 简写（opaque 往返面，scroll-margin
   页 4F「Unsupported property」）。

## 快测先行账（fc1 bin = HEAD + twin WIP + face，14:19-14:33）

typed-om 全语料 **gained=209 lost=0**（vs faceb b1）；四哨兵
pva/masking/transitions 逐页零位移（Pass 集双侧相等 899/4866/994），
animations +97 为 twin animation-range face B WIP 效果（非本刀——
其 shorthand 页修复，电池 RIDE census 恒定后双腿剔除归因）。

## A/B 电池（results-knife-facec，14:36 发射，15:01:05 GATE-DONE rc=0）

双腿 × {typed-om, pva, masking, transitions, animations}，b0=part10 face
剥除（bin md5 7e351d42）、b1=face+ride（md5 17284bb3，M2 区分成立），
ride census S1/S3 恒定零扰动。构建 provenance：b0 14:36:14 / b1
14:36:57，均早于 arithmetic 流 14:40:38 combined patch trap 回写——
b1 零算术污染（ movement 全表零 numeric-objects 页佐证）。

**RECONCILE：gained=209 lost=0 regressed=0，pages_gain=47 pages_loss=0**，
全部落 css-typed-om，哨兵四语料逐页零位移。大头：logical +42、
scroll-margin/scroll-padding +16 各、radius/inline-size/block-size +12
各、grid-template/auto-columns-rows +10 各、center/coordinate +8 各、
baseline-shift +4、margin/padding/stroke 三族 +4 各、to.tentative +1
（to() math 臂兑现）；余 ~28 页 +1~+3 computed 规范化顺向。电池 log
tail -40 截尾 8 页，全表由 results-knife-facec/{b0,b1} JSON 重放
ab-compare.py 复原（上节残面清单中 scroll-margin 已由白名单臂收编）。

## 残面（face C part 2 候选，engine 域）

- width/-3.14em、/-3.14% set 拒收（「Specified value must be a
  CSSStyleValue」——引擎 CSD 宽度负值文法门，testsuite 期望 set 成功
  包 Sum）
- min-width/max-width computed echo（opaque 'auto' / keyword 'none'）
- column-count/tab-size 等 number 族 computed echo（'auto'/'8'，
  expected CSSUnitValue(5,'number')）
- G 简写白名单 85F + G2 未知属性 103F（独立刀候选）
- arithmetic.tentative 47F（CSSMathValue 构造/类型代数，独立刀）

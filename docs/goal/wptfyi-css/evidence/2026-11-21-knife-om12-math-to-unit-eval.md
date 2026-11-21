# 刀 OM-12：numeric-objects 算术剥壳/跨族拒收 + to() 全形态求值 16 靶（2026-11-21 08:3x 收账）

## 池

numeric-objects 双页 16F：arithmetic.tentative.any.js 8F（Sum.add/sub、
Product.mul/div 数值化简 4F＋mul/div 跨实族 TypeError 2F＋Min/Max 2F）、
to.tentative.any.js 8F（非法单位 SyntaxError＋Product/Min/Max/Clamp/
Negate/Invert 单单位换算 6F＋复式表达式 1F）。探针 zz-om12probe-acc.json
定形期望值（含复式 max((s·s·px·px)/(s·px), ms·cm)/cm − min(ms,s) →
to('ms') = 3500 面）。

## face（patch v2 `35654a21`，单文件 part10-typed-om.js +160/−18）

三臂：

- **A. 算术接收体**：单成员同构 math 剥壳（Sum(x).add(y) ≡ x.add(y)，
  css-typed-om-1 算术步 Sum([x]) 与 x 等价）；mul/div 跨实族 TypeError
  （px·px·s 面；number/percent 维外豁免——number·px 仍归 Product）。
- **B. to() 非法单位**：`new (globalThis.DOMException || Error)(…,
  'SyntaxError')`（css-typed-om-1 §dom-cssnumericvalue-to 步 1，'lemon'
  面；toSum 同门先例）。
- **C. to() math 全形态求值器 `_zwOmEvalMathUnit`**：符号级指数代数——
  Unit 叶按族 base 因子折算、Product/Invert 指数带号累并、Sum/Negate
  同维代数和、Min/Max/Clamp 同维按折算值取小大/钳（成员含相对叶
  TypeError），末态族维度恰合目标单位型（绝对族指数 1、number 全零、
  percent 单族）否则 TypeError；`_zwOmExpDimsEq` 族维度等价门。

## v1 电池 GATE FAIL 净 −34 定谳与回修（本刀主教训）

v1（patch `d16cf554`，06:05 电池）16 靶已全 F2P，但 **css-typed-om
P 12261→12227（−34）GATE FAIL 22 页**：求值器叶臂把全部 factorless
单位（percent/fr/相对长度）一律标 `rel=true`，`Sum(0%,0%).to('percent')`
在 `if (t12.rel) throw "Relative units can not be converted"` 处炸——
34 个「Can set X to a percent: calc(0% + 0%)」stylepropertymap set 面
P2F（set 校验钳形回调走 `input.to('percent')`，face C part 1 注释既有
实证）。根因＝percent/flex 是**自有族维**（css-typed-om-1
§converting-cssunitvalue 同名单位直取不加因子），非相对单位；真正
不可换算的只有 factorless length（em/ex/ch/rem/vw…族 'length' 无因子）。
回修＝叶臂 `rel` 仅 `ui[0] === 'length'` 置位（一行＋规范链接注释）。
Sum 臂指数不累并（保首成员维度表）＋值累加，px+px→3px 与 %+% 同门放行。

## 电池 v2（knife-om12-battery.sh，log knife-om12-battery-0821.log）

b0=`b2d2f3cf`（face 剥除＝OM-11 提交态 b06b24e40＋pull 730dbfc2a 组合树）、
b1=`cfdb7b0b`（face 重放），M2 异 md5 ✓，FACE-MARKER evalMathUnit 0→7＋
OM-12 注释 0→5。08:24:01 build 起、08:32:46 收账（8.7 分钟，全程
test-guard 包裹：build per-proc 6/total 14/1800s＋leg per-proc 4/total
8/3000s＋CASE_WALL_S=60）。哨兵＝css-variables＋css-logical。

判读（knife-om12-judge.py）：TARGET-GAIN **16/16**（arithmetic gone=8
new=0、to gone=8 new=0——恰集翻转无其他名字移动）；typed-om
P 12261→12277（+16）/F 145→129（−16）；双哨兵 SET-IDENTICAL（variables
56 页/logical 86 页）；**OM12-GATE PASS rc=0**。RANDOM3a 组合态（pull
730dbfc2a）对 b0 基线零位移（v1 b0 12261 ＝ v2 b0 12261）。构建期
style-system missing_docs warning ×6 为对端 RANDOM3a 既有面，与本刀无关。

## 残面

numeric-objects 池余 1F：cssnumericvalue-multiply-two-types percent hint
1F＝`new CSSMathProduct(a, b)` 构造器路径类型检查（刀外，OM-13 候选）。
电池脚本头注「normalize-numeric 2F」为过时 census（b0 即绿，前刀已收）。

## 验证面注记

JS shim 单文件（include_str 入 engine 二进制），fmt/clippy 不适用；
行为验证＝电池三 corpus（typed-om 359 页＋variables 56 页＋logical
86 页）＋探针链。前会话 06:18 死于 API 限额时 v1 电池刚跑完未判读，本轮
接手判读定谳回修后重跑 v2 收账。

[控制面](../master.md)

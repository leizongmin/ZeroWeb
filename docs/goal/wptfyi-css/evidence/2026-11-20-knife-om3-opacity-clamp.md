# 刀 OM-3：opacity 族 [0,1] number 归一 + calc 长度 set 门收紧（2026-11-20 18:5x 收账）

## 池

properties/ 池 opacity 族 37F＝fill/flood/stop/stroke-opacity ×8
（computed 面值域合同：number 钳 [0,1]、percent 1/100 换算后钳、
calc(2+3) 折 Unit、calc 长度 set 恒 TypeError）+ opacity 本尊 ×5。
页面断言：runPropertyTests 组 computed（clamping_number /
clamping_percentage）优先于 example defaultComputed（testsuite.js
`if (computed || example.defaultComputed)`——组参数先行）。

## 探针 ground truth（OM-2 b1 二进制＝HEAD 码，zz-om3probe，跑后已删）

- fill 族（引擎未建模，gCS 空→declared 回落）：computed reify 保 percent
  原形/负值/calc Sum 展开——全崩；calc(0px+0em) set 被受理（style 串
  持久、comp=CSSUnitValue(0,px)）。
- opacity 本尊（引擎建模）：gCS 已钳化（number 面全绿）但引擎 declared
  串重写 %→number（set '3.14%'→style '0.0314'）、var()/calc 长度声明
  丢弃（style 空、spec=null）——pct×3 specified roundtrip 与 var-ref
  4F＝style-system 域残面（对端渲染流领域，shim 无声明原文可恢复，
  不越界挂账）。

## face（patch `c61069ff`，单文件 part10-typed-om.js，三臂）

1. `_zwOmOpacityClampNumProps`（fill/flood/stop/stroke-opacity 四属性）：
   computed 面裸 Unit 臂 + 折叠 Sum 臂双点——percent→number（1/100）后
   钳 [0,1]、number 直接钳（css-fill-stroke §fill-opacity / filter-effects
   §flood-opacity / SVG2 §stop-opacity/stroke-opacity 值域合同）；opacity
   本尊不入集（引擎串已钳化）。
2. Sum 折叠补 number 族（baseUnit + `mf==='number'` 臂）——
   'calc(2 + 3)'→CSSUnitValue(5,'number')。
3. `_zwOmValidateMath` length 叶收紧：删 `|| <percentage>` 配——
   <percentage> 文法不含长度形（css-values-4），opacity 族 calc 长度
   set 恒 TypeError（testPropertyInvalid 期望面）。波及 8 个 pct-only
   表属性（四 opacity 族 + border-image-slice/font-stretch/
   text-size-adjust 三个 throw 即修）。

## 电池（knife-om3-battery.sh，log knife-om3-battery.log）

b0/b1 全经 test-guard（build per-proc 6/total 14/1800s；leg per-proc
4/total 8/3000s + CASE_WALL_S=60）。18:36:31 发射，18:45:14 GATE-DONE
（8m43s）。FACE-MARKER `_zwOmOpacityClampNumProps` b0=0/b1=3 恰增、
M2 异值 ✓。

判读 knife-om3-judge.py **OM3-GATE PASS**（judge v1 NAME_PAGE 页后缀
缺 '.html' 全靶误报 ambiguous——判读器 bug 非电池问题，修复后对既有
b0/b1 JSON 复判）：

- TARGET-GAIN **33/33**（恰集 F2P）；opacity STAY-FAIL 4 靶持位
  （引擎域挂账如设计）。
- css-typed-om 全语料 P 12086→12125（**+39 零回归**）、F 320→281；
  页计数下降 0、改名对 0。
- 超额 +6 同臂同向溢出：len-calc throw 修 +3（border-image-slice/
  font-stretch/text-size-adjust——pct-only 表同收紧）、number 折叠
  +3（orphans/stroke-miterlimit/widows 'calc(2 + 3)' computed 翻绿）。
- 哨兵 css-variables SET-IDENTICAL（56 页恒，P 281 恒）。

## 残面

properties/ 池余 199F（238F − 39F）签名概览：cursor 18F、logical 16F、
column-rule-width 12F、font-stretch 10F（len-calc 修后余）、opacity 本尊
4F（引擎域挂账：declared %→number 重写 + var() 声明丢弃）、
column-rule-style 10F、line-height 8F、clip-path 8F 等——下刀候选
cursor/logical 对账后定。

[控制面](../master.md)

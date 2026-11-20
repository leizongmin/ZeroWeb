# 刀 OM-6：typed-om computed 声明文本 reify 三臂 18 靶（2026-11-20 23:0x 收账）

## 池

OM-5 后 `the-stylepropertymap/properties/` 签名分解三族 18F：

- `touch-action.html` 5F＝pan-left/pan-right/pan-up/pan-down/pinch-zoom
  关键词臂。
- `gap.html` 5F＝column-gap normal/−3.14em/−3.14% ＋ row-gap normal/−3.14%。
- `line-height.html` 8F＝number {0, ±3.14, calc(2 + 3)} ＋ length
  {−3.14em, calc(0px + 0em)} ＋ percent {−3.14%, calc(0% + 0%)}。

## 探针 ground truth（zz-om6probe 三版，跑后已删）

探针页逐臂嵌名（WPT 探针先例：ground truth 必须嵌测试名），覆盖
style.setProperty 字符串路径与 attributeStyleMap.set 类型化对象路径、
detached 与 attached（createDivWithStyle 同型 appendChild）四象限：

- **v1 detached + setProperty**：全部臂 csm 正确（CSSKeywordValue/
  CSSUnitValue 正形/折叠）——gcs 空回落声明文本路径本来就对。
- **v3 attached + attributeStyleMap.set**（复刻语料 testPropertyValid
  精确路径）：gcs 挂载态失真逐臂暴露＝语料失败全复现：
  - touch-action pan-left/pinch-zoom：gcs `'auto'`（引擎归一）→
    Keyword(auto)，语料期望 Keyword(关键词)。
  - column-gap normal：gcs `'auto'`（引擎误词）→ opaque 类；row-gap
    normal：gcs `'0px'` → Unit(0,px)；语料期望 Keyword(normal)。
  - gap −3.14em/−3.14%：gcs `'auto'`/`'0px'` → opaque/Unit px；语料
    期望 px 类 / 钳零保 percent（assert_is_equal_with_clamping_
    percentage，`_zwOmGridClampPctProps` 已含 gap——刀 G 钳臂被引擎
    '0px' 串短路）。
  - line-height number {0,±3.14}：gcs `'50.24px'`/`'0px'`（number 被
    字号折算）→ Unit px；语料期望 Unit number（css-inline-3 纯
    <number> computed 保 number）。
  - line-height 负/calc 各形：gcs `'normal'`（声明弃置）→ Keyword；
    语料期望 Unit 类。
  - width −3.14em：**set 即拒**（attr=null，css-sizing-3 `<length
    [0,∞]>` 规范拒收）→ 12 靶（width/height/min/max × em/%）不可赢
    STAY-FAIL 不入刀。

## face（patch `e47a786a` v2，单文件 part10-typed-om.js +57/−0，commit `aa1399209`）

三臂全在 computedStyleMap face getPropertyValue（OM-2/OM-4/OM-5 声明
文本源协议承继）＋ canonicalize 一解析臂：

1. touch-action 词表内声明文本原样返回（OM-4 cursor 同型；
   css-pointerevents-2 §touch-action 计算值＝指定关键词无归一化）。
2. column-gap/row-gap 声明文本 reify：词表内关键词（normal）或数值形
   （`/^(calc\(|[-+0-9.])/` 门）原样返回——percent 走既有
   `_zwOmGridClampPctProps` 钳零、length 走 canonicalize px 绝对化；
   var()/css-wide 回落引擎（css-align-3 §gap）。
3. line-height 数值形声明文本 reify＋`_zwOmComputedCanonicalize`
   line-height 百分比解析臂（Unit 与 Sum 折叠两处）：纯 <number> 保
   number；<percentage> 按宿主 font-size 解析 absolute length 保号不钳
   （css-inline-3 §line-height computed；语料页 computed 回调＝
   assert_is_unit('px')）；<length> px 绝对化；正形输出与引擎逐值恒等
   零回归；'normal'/var()/css-wide 回落引擎。

## 电池（knife-om6-battery.sh v2，log knife-om6-battery.log）

**v1 门抓反向**（log knife-om6-battery-v1-gatefail.log）：percent 臂
保形 reify 被 4 项判 FAIL——语料 line-height.html `<percentage>` 臂
computed 期望 **px**（assert_is_unit('px')），保形方向违反
css-inline-3 percentage→absolute length；0%/3.14% 从 b0 PASS 翻 FAIL
（P2F 反向证据），−3.14%/calc% 未翻。v2 增 canonicalize 解析臂修正，
patch 重生成（aaf3fbc6→16e3e3da→`e47a786a`）重发射。

v2：b0=`6a727c39`（face 剥除）/ b1=`a0025d2c`（face 重放），M2 异值 ✓，
FACE-MARKER 三臂 taArm/gapArm/lhArm（indexOf(dTA)/dGap.indexOf('var(')/
test(dLH)）各 b0=0/b1=1 恰增（face 源 md5 `1b73c3e5` 绑定 s1 快照＝
工作树态）。22:51:53 发射，22:59:17 GATE-DONE（7m24s，build per-proc 6/
total 14/1800s＋leg per-proc 4/total 8/3000s＋CASE_WALL_S=60，全程
test-guard 包裹）。

判读 knife-om6-judge.py **OM6-GATE PASS（rc=0）**：

- TARGET-GAIN **18/18**（gap 5、line-height 8、touch-action 5，含 v1
  反向的 4 个 percent 臂复绿）。
- 三靶页 gone=18 new=0 零改名零溢出；其余页 Fail 集合恒。
- css-typed-om 全语料 P 12184→12202（**+18 零回归**）、F 222→204。
- 哨兵 css-variables SET-IDENTICAL（56 页恒，P 281/F 237 恒）。

注：b0 基线 P 12184 与 OM-5 电池 b1 逐值恒等（本轮无跨轮截断方差；
properties/ 池页级分布微移属同形 CASE_WALL 页尾方差）。

判读绑定 base `09290097e`（rebase 已并入对端 R5868；对端触碰域
rendering-compat 文档，与本刀码面零交集）。

## 残面

properties/ 池余 **122F/47 页**（本电池 b1 实测）：clip-path 8F
（对端 CLIP-URL 刀已入组合态，残面留 join/下轮核）、column-rule-width
6F（−3.14em set 门＋5 does-not-support 声明面 origin 缺口）、width 6F/
height 6F（负值臂 set 门规范拒收 STAY-FAIL）、stroke-dasharray 5F/
stroke-width 5F（number 臂同 OM-6③ 机制——语料期望待下轮探针定谳）、
tab-size 5F（number 3 靶＋length 2 靶同机制候选）、column-count 4F、
opacity 4F、scroll-snap-align 4F 等。

[控制面](../master.md)

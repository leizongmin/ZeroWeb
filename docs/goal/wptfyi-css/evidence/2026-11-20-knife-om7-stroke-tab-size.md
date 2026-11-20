# 刀 OM-7：typed-om stroke-*/tab-size 数值面三臂 15 靶（2026-11-20 23:5x 收账）

## 池

OM-6 后 `the-stylepropertymap/properties/` 残面三属性 15F：

- `stroke-dasharray.html` 5F＝number {0, ±3.14, calc(2 + 3)} ＋ percent −3.14%。
- `stroke-width.html` 5F＝同上五靶（OM-6 证据预判「number 臂同 OM-6③
  机制——语料期望待下轮探针定谳」）。
- `tab-size.html` 5F＝number {±3.14, calc(2 + 3)} ＋ length {−3.14em,
  calc(0px + 0em)}。

## 探针 ground truth（zz-om7probe 挂载态，跑后页已删、json 留档）

探针页逐臂嵌名（attributeStyleMap.set 类型化对象路径＋attached 挂载态，
复刻语料 testPropertyValid 精确路径），全部臂 assert_true 失败并记录引擎
GOT 值＝语料失败全复现：

- stroke-dasharray/stroke-width number 3.14：GOT CSSUnitValue:3.14
  （引擎保形回显 unit 'number'）——语料期望 px 类（assert_is_unit('px')，
  svg2 number 计算值按用户单位即 px）。
- stroke-* percent ±3.14：GOT CSSUnitValue:±3.14%（引擎不钳）——语料
  assert_is_equal_with_clamping_percentage 期望 Unit(0,'percent')。
- tab-size number ±3.14：GOT CSSUnitValue:8（引擎声明弃置回默认 number
  8）——语料正形期望保 number、负形 assert_is_equal_with_clamping_number
  期望 Unit(0,'number')（css-text-3 §tab-size 非负值域）。
- tab-size px 3.14 → 3.14px；em 3.14 → 50.24px（通用折算臂）；em −3.14
  → 8（负 em 声明弃置）。

## face（patch `ec598cfa` v1 一遍过，单文件 part10-typed-om.js +57/−0，commit `79173342c`，rebase 后 blob md5 `64edb518` 与判读 s1 快照恒）

三面（OM-2/OM-4/OM-5/OM-6 声明文本 reify 协议承继）：

1. stroke-dasharray/stroke-width 入 `_zwOmGridClampPctProps`（svg2
   §stroke-dasharray/§stroke-width 负 <percentage> computed 钳 0）。
2. canonicalize 折叠臂 Unit 与 Sum 两处：stroke-* number 计算值用户单位
   折 px（值保值不钳）；tab-size 负 number 钳 0（正形保 number；
   <length> 相对单位走通用 em/rem 折 px 臂）。
3. stroke-*/tab-size 数值形 computed 从 inline 声明文本 reify：单值数形门
   （calc( 前缀或带可选单位数字 token）——多值 '5% 1em 2%' 不入门；
   var()/css-wide/关键词回落引擎。

## 电池（knife-om7-battery.sh v1，log knife-om7-battery.log + knife-om7-judgment.log）

v1 一遍过（无 OM-6 型 v1→v2 门反向迭代）：b0=`90a58f1c`（face 剥除）/
b1=`d12f1396`（face 重放），M2 异值 ✓，FACE-MARKER 五臂（clampEntry
`'stroke-dasharray': 1, 'stroke-width': 1,`/strokeFold `var swPx`/
tabClamp `var tsN`/strokeReify `var dSW`/tabReify `var dTS`）各 b0=0/
b1=1 恰增（face 源 md5 `64edb518` 绑定 s1 快照＝工作树态）。23:26 发射，
23:35:51 GATE-DONE（9.5 分钟，build 58s+45s＋腿 74s/2:00/70s/1:59，全程
test-guard 包裹：build per-proc 6/total 14/1800s＋leg per-proc 4/total 8/
3000s＋CASE_WALL_S=60）。

判读 knife-om7-judge.py **OM7-GATE PASS（rc=0）**：

- TARGET-GAIN **15/15**（stroke-dasharray 5、stroke-width 5、tab-size 5）。
- 三靶页 gone=15 new=0 零改名零溢出；其余页 Fail 集合恒。
- css-typed-om 全语料 P 12202→12217（**+15 零回归**）、F 216→201。
- 哨兵 css-variables SET-IDENTICAL（56 页恒，P 281/F 237 恒）。

注：b0 基线 P 12202 与 OM-6 电池 b1 逐值恒等（本轮无跨轮截断方差；
全语料 F 基数 216 对 OM-6 账 204 的 +12 属同形 CASE_WALL 页尾方差域，
judge 以本轮 b0/b1 自洽对账不受影响）。

判读绑定 base `2519e27e1`（HEAD 含 OM-6 face；对端 ZeroWeb-2 clone
motion1 电池同窗在他树在飞，零交集）。

## 残面

`the-stylepropertymap/` 池余 **137F/56 页**（本电池 b1 实测）：
attribute-changed-callback 9F、clip-path 8F（CLIP-URL 刀已入组合态残面
留核）、get-auto-min-size 6F、column-rule-width 6F（−3.14em set 门＋
does-not-support 声明面 origin 缺口）、width/height 各 6F（负值 set 门
规范拒收 STAY-FAIL）、iterable.tentative 5F、column-count/opacity/
scroll-snap-align/stroke-dashoffset 各 4F（**stroke-dashoffset 同族
OM-8 候选**——dasharray 同款 number 折 px＋负 % 钳合同）、block-size/
flex-basis/inline-size 各 3F。

[控制面](../master.md)

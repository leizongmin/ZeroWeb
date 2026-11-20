# 刀 OM-2：scroll-padding 族 computed 声明文本 reify + 负 % 钳零（2026-11-20 18:2x 收账）

## 池

`the-stylepropertymap/properties/scroll-padding.html` 28F＝物理向
top/left/right/bottom × {percent 0%/-3.14%/3.14%/calc(0%+0%)、length
-3.14em/calc(0px+0em)} 6 签名 + 逻辑向 inline/block-start/end × 负 percent
1 签名。页面断言：percent 组 computed=
`assert_is_equal_with_clamping_percentage`（期望 CSSUnitValue(clamp≥0,
'percent')），length 组 computed=默认 `assert_is_unit('px')`（类+单位，无
值门）；specified 全组 `assert_is_equal_with_range_handling`（负值包 Sum，
本刀前已过零扰动）。

## 探针 ground truth（OM-1 b1 二进制＝HEAD 码，zz-om2probe，跑后已删）

- 物理向（引擎建模）：gCS 计算串有损——`'0%'/'3.14%'`→`'0px'`、负值与
  calc→`'auto'`、`3.14cm`→`118.677px`；shim 从该文本 reify → 6 签名全
  崩（percent 形丢/负值 opaque/auto 类）。
- 逻辑向（引擎未建模，gCS 串空→shim declared 文本回落通道）：reify 形
  已对（percent 保形、calc 折 Unit、em/cm 绝对化 px）唯缺负 % 钳零
  （-3.14%→`-3.14 percent`，期望 0）。
- Rust 计算样式域（style-system）为对端渲染流领域不越界——物理向缺口
  在 shim 侧以声明文本源切换绕开。

## face（patch `f43d56eed03b318259e36817b3f897a0`，单文件 part10-typed-om.js +23/−2）

1. `_zwOmGridClampPctProps` += scroll-padding 8 长手（css-scroll-snap-1
   §scroll-padding 负 <percentage> computed 期钳 0——与 grid 轨道/gap 同
   合同同 helper：裸 Unit 臂 + 折叠 Sum 臂双点生效）。
2. computedStyleMap face `getPropertyValue`：`_zwOmScrollPadProps` 族恒从
   inline 声明文本 reify（引擎计算串有损不再入道；声明文本经既有
   `_zwOmComputedCanonicalize` 兑现 computed 值语义：percent 保形钳零、
   长度绝对化 px、同族 calc 折叠）。specified/attributeStyleMap 面
   零接触。

## 电池（knife-om2-battery.sh，log knife-om2-battery.log）

b0=`cb952911`（face 剥除，HEAD 6d3f5aab1 基）/ b1=`f4611875`（face 重放）
M2 异值 ✓，FACE-MARKER `_zwOmScrollPadProps` b0=0/b1=2 恰增。18:13:01
发射，18:20:18 GATE-DONE（7m17s，build
per-proc 6/total 14/1800s + leg per-proc 4/total 8/3000s + CASE_WALL_S=60，
全程 test-guard 包裹）。

判读 knife-om2-judge.py **OM2-GATE PASS**：

- TARGET-GAIN **28/28**（scroll-padding.html 靶名恰集 F2P，gone=28 new=0
  零改名）。
- css-typed-om 全语料 P 12074→12102（**+28 零回归**）、F 348→320；
  页计数下降 0。
- 哨兵 css-variables SET-IDENTICAL（56 页 Fail 集合逐字节恒，P 281 恒）。
- 探针复核（b1 二进制）：物理向 negpct comp opaque(auto)→`CSSUnitValue
  (0,'percent')`、pospct `CSSUnitValue(0,'px')`→`CSSUnitValue(3.14,
  'percent')`——物理/逻辑双通道形收敛，gCS 串未动（Rust 域零接触）。

注：本电池 b0 P=12074 与 OM-1 电池 b1 P=12058 差 16＝跨轮 CASE_WALL 页尾
截断方差（已知形态，Fail 集合跨轮恒，非码面因果）。

## 残面

properties/ 池余 238F（原 266F − scroll-padding 28F），签名分解（b1 JSON
聚类）：opacity 族 37F（fill/flood/stop/stroke-opacity ×8 + opacity ×5，
clamp-[0,1]/throw 语义候选合同）、cursor 关键词 18F（bidi resize 归一+
缺词）、logical 16F、column-rule-width 12F、font-stretch 类形 11F、
column-rule-style 10F、line-height 8F 等——下刀 OM-3 候选＝opacity 族
（同钳向合同族，复用本刀双臂配方）。

[控制面](../master.md)

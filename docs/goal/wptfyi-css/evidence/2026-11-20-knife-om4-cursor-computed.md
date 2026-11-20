# 刀 OM-4：cursor computed 声明文本关键词 reify（2026-11-20 19:1x 收账）

## 池

`the-stylepropertymap/properties/cursor.html` 18F＝36 关键词语法恰半：
6 现代词（context-menu/cell/vertical-text/alias/copy/no-drop）+ 8 legacy
向（e/w/n/s/ne/nw/se/sw-resize）+ 4 bidi 词（ew/ns/nesw/nwse-resize）。
页面断言：`runPropertyTests('cursor', [...])` 纯关键词语法无组覆盖——
testPropertyValid 平凡往返（set → specified get → computed get 恒等
CSSKeywordValue）。

## 探针 ground truth（OM-3 b1 二进制＝HEAD 码，zz-om4probe，跑后已删）

- 全 16 关键词（含对照 pointer）：set ok、声明文本逐字保留
  （`sty`/cssText 原样）、specified 往返全绿。
- computed 通道全崩：6 现代词 gcs 回 `'auto'`（引擎计算词表不含）、
  8 legacy 向 gcs 伪归一 e/w→`'ew-resize'`、n/s/ne/se→`'ns-resize'`
  （对角映射粗糙：ne 应归 nesw 非 ns；css-ui-4 §cursor 计算值＝指定
  关键词原样无归一化，Chrome 语料期望原样）、4 bidi 词 gcs 回 `'auto'`
  （引擎只把它们当归一化输出、不受理为计算值）。
- shim `_zwOmPropSyntax['cursor']` 词表 36 词齐全——词表无缺口，缺口在
  引擎计算串层（style-system 域挂账：gCS cursor 词表落后 + legacy→bidi
  伪归一）。

## face（patch `29595ffff`，单文件 part10-typed-om.js +15/−0）

computedStyleMap face `getPropertyValue` override 增 cursor 分支（OM-2
声明文本源切换同型）：声明文本在 `_zwOmPropSyntax['cursor']` 词表内→
原样返回（map 层 reify CSSKeywordValue，与引擎文本同管线）；词表外
（inherit/var()/url() 系）回落引擎计算串保继承与替代语义。未声明→
引擎路径（initial 'auto'）不变。SET 面/attributeStyleMap 面零接触。

## 电池（knife-om4-battery.sh，log knife-om4-battery.log）

b0=`25dd82ae`（face 剥除）/ b1=`a2f9724f`（face 重放），M2 异值 ✓，
FACE-MARKER `indexOf(dCur)` b0=0/b1=1 恰增。18:58:52 发射，19:07:40
GATE-DONE（8m48s，build per-proc 6/total 14/1800s + leg per-proc 4/
total 8/3000s + CASE_WALL_S=60，全程 test-guard 包裹）。

判读 knife-om4-judge.py **OM4-GATE PASS**：

- TARGET-GAIN **18/18**（cursor.html 靶名恰集 F2P）。
- cursor.html gone=18 new=0 零改名；**无同臂溢出**（本刀面全域恰集，
  其余 53 页 Fail 集合恒）。
- css-typed-om 全语料 P 12141→12159（**+18 零回归**）、F 281→263。
- 哨兵 css-variables SET-IDENTICAL（56 页恒，P 281 恒）。

注：b0 P=12141 与 OM-3 电池 b1 P=12125 差 16＝跨轮 CASE_WALL 页尾截断
方差（OM-2 同形态已知，Fail 集合跨轮恒，非码面因果）。

## 残面

properties/ 池余 181F（199F − 18F）签名概览：logical 16F、
column-rule-width 12F、column-rule-style 10F、font-stretch 10F、
clip-path 8F、line-height 8F、height 6F、width 6F、gap 5F、
stroke-dasharray 5F、stroke-width 5F 等——下刀候选 logical/column-rule
对账后定。引擎域挂账（style-system）：opacity 本尊 pct×3 + var-ref 4F、
cursor gCS 词表 + legacy→bidi 伪归一。

[控制面](../master.md)

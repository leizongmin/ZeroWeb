# 刀 OM-1：marker-* 长手 typed 通道（2026-11-20 18:0x 收账）

## 池

`the-stylepropertymap/properties/marker.html` 18F＝marker-start/mid/end ×
{css-wide initial/inherit/unset/revert、` var(--A)`、none keyword}
（marker-* 全部「Unsupported property」set 即抛）。语料 `<url>` examples
上游为空集（TODO css-houdini-drafts#716）零测试；简写 `marker` 的
`testUnsupportedValue` 6 测试为 opaque 往返面（非 throw 面），本刀前已过。

## 探针 ground truth（HEAD 二进制，zz-om11probe/probe2/probe3，跑后已删）

- B1/B3：`el.style.setProperty('marker-start', …)` inline 持久 ✓（Rust
  声明层不丢 marker-*，url 形亦可）。
- B2：`getComputedStyle().getPropertyValue('marker-start')` 返回空串——
  引擎计算样式未产出 marker-* 文本，计算通道缺口在 Rust 侧（style-system
  域，共享面对端渲染流领域，不越界）。
- C1（face 后）：`computedStyleMap().get('marker-start')` →
  CSSKeywordValue('none') ✓。

## face（patch `1d0d7d870c79c15ed4071afaacea2201`，单文件 part10-typed-om.js +19/−2）

1. `_zwOmPropSyntax` += `marker-start/mid/end: ['none','<url>']`
   （css-masking-1 §marker-*；简写 marker 保持不入表）。
2. `mapGetAll` computedFace 检测上提至 text 读取后；marker 族 text==''
   恒 `[CSSKeywordValue('none')]`（initial none + inherited——gCS 文本
   缺口的 shim 侧兜底，绕开 Rust 计算通道）。
3. `_zwOmMarkerProps` 族表（兜底判定）。

## 电池（knife-om1-battery.sh，log knife-om1-battery-1736.log）

b0=`84ca9b8f`（face 剥除）/ b1=`ccee2fff`（face 重放），M2 异值 ✓，
FACE-MARKER `_zwOmMarkerProps` b0=0/b1=2 恰增 ✓。17:48:46 发射，
17:56:24 GATE-DONE（7m38s，四腿 test-guard per-proc 4/total 8/3000s +
CASE_WALL_S=60 包裹）。

判读 knife-om1-judge.py **OM1-GATE PASS**：

- TARGET-GAIN **18/18**（marker.html 靶名恰集 F2P，gone=18 new=0 零改名）。
- css-typed-om 全语料 P 12040→12058（**+18 零回归**）、F 366→348；
  页计数下降 0。
- 哨兵 css-variables SET-IDENTICAL（56 页 Fail 集合逐字节恒，P 281 恒）。

注：本电池 b0 P=12040 与 T3D 电池 b1 P=12050 差 10＝跨轮 CASE_WALL
页尾截断方差（Fail 集合跨轮恒，仅慢页尾 subtest 计数漂移，非码面因果）。

## 残面

properties/ 池余 266F（原 284F − marker 18F），签名分解见控制面 OM-1
收账段：scroll-padding 族 28F（percent 分类/computed 类/物理-logical 钳
向不一致）、cursor 关键词族 18F（bidi resize 归一+缺词）、font-stretch
类形 11F（keyword CSSKeywordValue 形）、opacity 家族 clamp/throw 语义、
numeric-objects 残 16F 等——下刀 OM-2 候选。

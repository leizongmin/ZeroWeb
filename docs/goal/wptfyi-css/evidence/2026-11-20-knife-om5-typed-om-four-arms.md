# 刀 OM-5：typed-om 残面四臂 41 靶（2026-11-20 22:2x 收账）

## 池

OM-4 后 `the-stylepropertymap/properties/` 残面签名分解四族：

- `column-rule-style.html` 10F＝none/hidden/dotted/dashed/solid/double/
  groove/ridge/inset/outset 全关键词（runPropertyTests 纯关键词语法平凡
  往返）。
- `column-rule-width.html` 12F 中 6 靶＝thin/medium/thick 关键词＋0px/
  3.14cm/calc(0px + 0em) 长度（余 6 靶＝负长度与 does-not-support 解析
  臂，不入本刀）。
- `font-stretch.html` 10F 中 9 靶＝九词表关键词（ultra-condensed 至
  ultra-expanded；余 1 靶＝−3.14% 负百分比，不入本刀）。
- `logical.html` 16F＝逻辑角 border-{start,end}-{start,end}-radius 百分
  比臂 4 角 × {0%, −3.14%, 3.14%, calc(0% + 0%)}。

## face（patch `aaf3fbc6`，单文件 part10-typed-om.js +42/−1，commit `b672ff24c`）

四臂（OM-1..OM-4 声明文本源 reify 协议承继）：

1. column-rule-style/column-rule-width 入 `_zwOmComputedUnsupportedProps`
   基类列——css-multicol-1 §column-rule-*：语料 computed 臂显式
   assert_class_string 基类 CSSStyleValue，Typed OM L1 无 per-property
   分型，thin/medium/thick 与 <length> 均不升级子类；declared 面逐语法
   解析 equal 不受影响。
2. font-stretch computed 从 inline 声明文本 reify 词表内九关键词——
   引擎计算串按 css-fonts-4 §font-stretch 映射表折算 <percentage> 输出，
   语料关键词臂 computed 走默认 equal 期望 CSSKeywordValue 原样；词表外
   （<percentage>/calc/var()/inherit 系）回落引擎计算串保百分比钳位臂。
3. 逻辑角四长手入 `_zwOmLogicalRadiusProps` 名集＋computed 百分比形从
   声明文本 reify——css-backgrounds-3 §border-*-radius 百分比 computed
   保形，引擎计算串按物理四角绝对化 px 并钳负值；<length> 形回落引擎
   px 绝对化（defaultComputed assert_is_unit('px') 面）；var()/css-wide
   系回落保继承与替代语义。

## 电池（knife-om5-battery.sh，log knife-om5-battery.log）

b0=`1e34a615`（face 剥除）/ b1=`f26fe374`（face 重放），M2 异值 ✓，
FACE-MARKER 三臂 fsArm/radArm/colRuleUnsup 各 b0=0/b1=1 恰增（face 源
md5 `efeeb48b` 绑定 s1 快照＝工作树态）。19:59:03 发射，20:06:19
GATE-DONE（7m16s，build per-proc 6/total 14/1800s＋leg per-proc 4/
total 8/3000s＋CASE_WALL_S=60，全程 test-guard 包裹）。上一会话在门标
触发后因 API 限额中断，判读与收账由本会话续做。

判读 knife-om5-judge.py **GATE PASS（rc=0**；判读日志
knife-om5-judgment.log 尾标 "OM4-GATE PASS" 为承继 OM-4 脚本模板的陈旧
字面串，判读域＝OM-5 四页 41 靶集，协议同构）：

- TARGET-GAIN **41/41**（column-rule-style 10、column-rule-width 6、
  font-stretch 9、logical 16）。
- 四靶页 gone=41 new=0 零改名零溢出；其余页 Fail 集合恒。
- css-typed-om 全语料 P 12143→12184（**+41 零回归**）、F 263→222。
- 哨兵 css-variables SET-IDENTICAL（56 页恒，P 281/F 237 恒）。

注：b0 基线 P=12143 对 OM-4 电池 b1 P=12159 差 −16＝跨轮 CASE_WALL 页尾
截断方差（OM-2/OM-4 同形已知，Fail 集合跨轮恒，非码面因果）。

判读绑定 pre-rebase b1 字节（base `2bbed28b2`）；收账前 rebase 并入对端
CLIP-URL ×3（css-parser/engine computed_style.rs/style-system），触碰域
css-masking/borders 与本刀判读面（css-typed-om 四页）零交集，组合态由
join 对账兜底。

## 残面

properties/ 池余 **133F/48 页**：column-rule-width 6F（−3.14em 负长度
应 throw＋5 个 does-not-support 解析臂）、font-stretch 1F（−3.14% 负
百分比应 throw）、clip-path 8F（判读于 pre-rebase b1；对端 CLIP-URL 刀
已入 clip-path url() 文法面，残面或在组合态位移——留 join 对账/下轮
核）、line-height 8F、height 6F、width 6F、gap 5F、stroke-dasharray 5F、
stroke-width 5F、tab-size 5F、touch-action 5F 等。

[控制面](../master.md)

# 刀 OM-8：typed-om stroke-dashoffset number 折 px 4 靶（2026-11-20 23:5x 收账）

## 池

OM-7 后 `the-stylepropertymap/properties/stroke-dashoffset.html` 残面 4F
＝number {0, ±3.14, calc(2 + 3)}（语料失败消息自带 ground truth：
assert_equals unit expected "px" but got "number"——无需另立探针即定谳
读侧单位类缺口）。

## face（patch `89140b98` v1 一遍过，单文件 part10-typed-om.js +10/−3，本刀）

OM-7 合同承继的**最小面**——仅扩 canonicalize 两处 number→px 折叠臂的
属性门（Unit 臂＋Sum 折叠臂收 `stroke-dashoffset`）：

- **不入 `_zwOmGridClampPctProps`**：语料 percent 臂（含 −3.14%）本就过
  （svg2 §stroke-dashoffset percent 无钳合同，异于 dasharray/width），
  钳表扩入会 P2F 回归。
- **不扩 computed reify 臂**：引擎 gCS 对 dashoffset number 形保形回显
  （并席核验探针独立实证，同 dasharray 族行为），折叠臂单独即承重。
- percent/length 路径零触碰（折叠门仅 `unit === 'number'` 时生效）。

## 探针（并席核验席独立定谳，本席零探针）

本轮检测到同任务并行核验席（回执 `NOTES-om8-verifier.md`）：其独立探针
zz-om8probe（23:40 跑、json 留 `.acceptance/wptfyi-css/zz-om8probe.json`、
跑后页删）与写者 face 合同逐臂一致——dashoffset number 四臂引擎保形回显
unit 'number'（GOT `CSSUnitValue:3.14number` 等），percent/px/em 本就过
无钳合同。本席让位协议反向成立（写者工件 23:43–44 先于核验席探针
23:45）：核验席让位核验不双写，本席承写者位收口。

## 电池（knife-om8-battery.sh v1，log knife-om8-battery.log + knife-om8-judgment.log）

v1 一遍过：b0=`d12f1396`（face 剥除；**≡ OM-7 b1-runner 逐字节恒**
＝provenance 链，已提交 OM-7 态重建即 OM-7 gate 同款二进制）/
b1=`abf02b76`（face 重放），M2 异值 ✓，FACE-MARKER 双臂（unitFold/
foldedFold `p === 'stroke-dashoffset'` 门）各 b0=0→b1=1。23:44:15 B0
build 起，23:52:50 GATE-DONE（8.6 分钟，全程 test-guard 包裹：build
per-proc 6/total 14/1800s＋leg per-proc 4/total 8/3000s＋CASE_WALL_S=60）。

判读 knife-om8-judge.py **OM8-GATE PASS（rc=0）**：

- TARGET-GAIN **4/4**（stroke-dashoffset number 四靶恰集 F2P）。
- 靶页 gone=4 new=0 零改名零溢出；其余页 Fail 集合恒。
- css-typed-om 全语料 P 12217→12221（**+4 零回归**）、F 209→197。
- 哨兵 css-variables SET-IDENTICAL（56 页恒，P 281/F 237 恒）。

**F 线对账（并席探针页污染闭合，核验席逐项核验通过）**：b0 F 209＝
OM-7 b1 基 201＋核验席 zz-om8probe.html 8F（该页 23:45:42 b0 腿枚举
在场、23:48:59 b1 腿枚举已删——单侧 b0 虚高）；197＝209−4 靶−8 探针。
P 门/靶恰集/哨兵三门不受影响（探针页 P=0 单侧、不在 NAME-DIFF 交集域）。

**OM-7 F 基数订正（核验席定谳，本席并笔）**：OM-7 电池 F 216 对 OM-6
账 204 的 Δ+12 非 CASE_WALL 页尾方差，系语料目录遗留探针页
`zz-om7probe.html`（23:08 前身会话产物未删，12 臂双态恒红 +12F 恒，
OM-8 双腿均在场完全中性）——OM-7 证据该句已由核验席订正（P 门/靶恰集/
哨兵结论不变），订正笔随本刀 docs 入库；该遗留页本刀收口清除。

判读绑定 base `b1e49c0ba`（HEAD＝OM-7 hash 订正笔；对端 ZeroWeb-2
motion1-r2 电池同窗在他树在飞，零交集）。

## 残面

`the-stylepropertymap/` 池余 **133F/55 页**（本电池 b1 实测，OM-8 b1
json；zz-om7probe 遗留页清除后池口径）：attribute-changed-callback 9F、
clip-path 8F（CLIP-URL 残面留核）、get-auto-min-size 6F、
column-rule-width 6F（−3.14em set 门＋does-not-support 声明面 origin
缺口）、width/height 各 6F（负值 set 门规范拒收 STAY-FAIL）、
iterable.tentative 5F、column-count/opacity/scroll-snap-align 各 4F、
block-size/flex-basis/inline-size 各 3F（池口径恒为 stylepropertymap
子树过滤、不含遗留探针页；OM-7 页脚 137F/56 页与本笔 133F/55 页同口径
直减 4 靶 1 页，遗留探针页只影响全语料 TOTAL F 线不影响池账）。

[控制面](../master.md)

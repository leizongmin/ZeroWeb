# 刀 OM-9：typed-om sizing/SVG geometry 负百分比 computed 钳 0 九靶（2026-11-21 00:2x 收账）

## 池

OM-8 后 `the-stylepropertymap/properties/{block-size,inline-size,radius}.html`
残面各 3F＝负 <percentage> computed 臂（block/min/max-block-size、
inline/min/max-inline-size、r/rx/ry × −3.14%；语料失败消息
`assert_approx_equals: expected 0 +/- 0.00001 but got -3.14` 自带 ground
truth）。三页语料源直读定谳：computed 臂
`assert_is_equal_with_clamping_percentage`（负 % 期望 Unit(0,'percent')）、
specified 臂 `assert_is_equal_with_range_handling`（保形回显）——与
`_zwOmGridClampPctProps` 既有合同（刀 G grid/gap、OM-2 scroll-padding、
OM-7 stroke-*）逐字同型。

## face（patch `d0fe803c` v1 一遍过，单文件 part10-typed-om.js +11/−0，本刀）

`_zwOmGridClampPctProps` 扩 9 属性（css-sizing-3 §width/height/§min/max
sizing 负值 invalid＋css-logical-1 block-size/inline-size 逻辑映射同合同＋
svg2 §geometry properties r/rx/ry 负值 invalid）：

- **钳表仅 computed 面**（canonicalize Unit 臂＋Sum 折叠臂既有门），
  specified/解析路径零触碰——percent 臂外无行为变化。
- css-logical / css-sizing 语料直读无 `clamping_percentage` 亦无负 %
  computed 期望（冲突面零）；`testharness-css-logical` 首次入电池作
  第二哨兵（86 页 SET-IDENTICAL 恒）。

## 探针（zz-om9probe，跑后页删、json 留 `.acceptance/wptfyi-css/`）

挂载态实证（OM-8 态二进制 b1-runner 复用，预 face）：9 属性 GOT 侧全
`CSSUnitValue:-3.14:percent`——percent 形保形至 canonicalize 未钳（表
扩入即钳 0）；REC 侧 CSSMathSum 形不经 canonicalize 钳位（specified
零触碰实证）。svg rect 挂载 r/rx/ry 与 div sizing 族同路径。

## 电池（knife-om9-battery.sh v1，log knife-om9-battery.log）

v1 一遍过：b0=`5beb52f1`（face 剥除＝OM-8 提交态）、b1=`e530c1a1`
（face 重放），M2 异值 ✓，FACE-MARKER 双门（钳表新键 `'ry': 1`＋注释
token `zz-om9probe`）各 b0=0→b1=1。00:18:53 B0 build 起、00:27:31
GATE-DONE（8.6 分钟，全程 test-guard 包裹：build per-proc 6/total
14/1800s＋leg per-proc 4/total 8/3000s＋CASE_WALL_S=60）。

判读 knife-om9-judge.py **OM9-GATE PASS（rc=0）**：

- TARGET-GAIN **9/9**（三页各 3 靶恰集 F2P）；靶页 gone=3 new=0 零
  改名零溢出，其余页 Fail 集合恒。
- css-typed-om 全语料 P 12221→12230（**+9 零回归**）、F 185→176。
- 双哨兵 SET-IDENTICAL：css-variables（56 页，P 281/F 237 恒）＋
  css-logical（86 页，P 218/F 1146 恒）。

**F 线对账（provenance 链）**：b0 F 185＝OM-8 b1 基 197−zz-om7probe
遗留 12F（OM-8 收口清除，双态恒红页不在场）＝本轮电池双腿零探针页
污染（zz-om9probe 电池前已删，语料 359 页 zz-free）。P 门/靶恰集/
双哨兵判读不受影响。

判读绑定 base `2b3588e7d`（HEAD＝OM-8 行数订正笔；对端 ZeroWeb-2
clone 同窗在他树在飞，零交集）。

## 残面

`the-stylepropertymap/` 池余 **124F/52 页**（本电池 b1 实测，
133F/55 页直减 9 靶 3 页；zz-* 遗留页零）：attribute-changed-callback
9F、clip-path 8F（CLIP-URL 残面留核）、get-auto-min-size 6F、
column-rule-width 6F、width/height 各 6F（负值 set 门——引擎 set 即拒收 vs 同族
block-size/inline-size 引擎已收的 set 门不一致；OM-6 判 css-sizing-3
`<length [0,∞]>` 规范拒收 STAY-FAIL 挂账，同族 set 门一致性另刀核）、
iterable.tentative 15F/3 页（StylePropertyMap keys/entries/values
maplike 缺口）、column-count/opacity/scroll-snap-align 各 4F、
offset-rotate/orphans/text-decoration-line/widows 各 3F。

[控制面](../master.md)

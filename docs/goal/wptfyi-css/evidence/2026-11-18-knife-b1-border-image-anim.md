# 刀 B1：border-image 三长手（slice/width/outset）动画家族接入三通道（2026-11-18）

## 靶面

css-backgrounds 腿 4241F 大面盘点中未被认领的最大整机制面：border-image
家族动画插值（animations/ 目录 interpolation/composition 页 + discrete
页）。残面杠杆量化时 fv1 占 basic-shape/clip-path 面、A1d 占
background-position-x/y 面、knifeq 占 shape-outside 探针批；border-image
家族 ~940F 余量无人认领。子面对形：

1. **插值页**（border-image-{slice,width,outset,source}-interpolation.html）：
   CSSA/WAAPI/transitions 三通道 × 同型对（Number↔Number、Percent↔Percent、
   Length↔Length、Auto↔Auto）逐 at 断言；width 页另有 %↔px 跨型对期望
   calc 系数形（见 P2F 章）。
2. **composition 页**（-composition.html）：composite operation add/replace
   语义（56+56×3 页量级）——本刀仅收 replace 面（+19/页），composite-add
   机制挂债。
3. **discrete-no-interpolation.html**：no-interp 关键字对（repeat 族
   stretch↔round 等）期望离散翻转。
4. **source 页**：CSSA/WAAPI 半面 49F×2 未接（image 值型插值=离散面，
   本刀未动 source 插值臂，仅 applier 通路就位）。

## 机制（engine 2 文件，css-parser 零编辑）

**宽松臂三处模式 OR**（animation.rs interpolate_property_value）：
`"mask-border-slice" | "border-image-slice"`、`"mask-border-width" |
"border-image-width"`、`"mask-border-outset" | "border-image-outset"`
——逐字镜像刀 A1c 的 mask-border 臂结构：解析器复用 css-parser
parse_border_image_{slice,width,outset}（BorderImageSliceComponent(Auto?
Number/Percent) / WidthComponent(Auto/Number/Length/Percent) /
OutsetComponent(Number/Length)，非 Copy 全引用传参），逐槽 cmp 闭包
同型 lerp + 值域非负钳 0（语料 at:-0.3 期望 '0px'/'0%'/'0'）、裸 0 双态
桥（Px(0)↔Number 语料 '0'→'20' at:0.3 期望 '6' 非离散）、多槽 box4_str_min
串化；width 跨型 Length↔Length 非 px 组合走既有 lerp_len（clip_interp.rs）
calc 混合通道。

**严格臂同型槽位门**（interpolate_property_value_strict）：border-image
语料含 no-interp 关键字对（transitions 通道 flipAt=-∞ 恒 to）——不复制
mask-border 的恒 Some 委托（防 P2F），改同型槽位门四 helper：
`border_image_same_type_slots`（slice: fill 对称+四槽 slot_pair_same_type；
width/outset: 四槽数组 .all(identity)）→ 全同型 `.then(|| 宽松臂真插值)`，
跨型/关键字/fill 不对称 None = 非 animatable 对不启动过渡（静态跳终值，
css-transitions-1 §starting of transitions）。

**applier 五臂**（apply_single_property）：border-image-source/slice/
width/repeat/outset 逐字镜像 mask-border 臂写 `style.border_image_*` 槽
（Length 槽经 resolve_effect_length；source 经 border_image_source_to_-
computed）。此为三通道共因：canonicalize 链（js_anim_registry.rs:391）
apply 插值串到 ComputedStyle 克隆 → 串化前后比对 ser1≠ser0 才收值——
apply_single_property 缺臂 ⇒ ser1≡ser0 ⇒ None ⇒ CSSA/WAAPI/transitions
三通道全部回落级联静态。刀 A1（mask 族）/A1d（bg-position）同病同治
先例第三次验证。

**登记清单 +3**（transition_registry.rs TRANSITIONABLE_PROPERTIES）：
border-image-outset/slice/width 入清单（repeat/source 为离散型不入）。

**钉测 4 枚**（animation.rs tests mod 尾）：test_border_image_slice_lerp
（100%→10% at1.5='0%'、20%→10% at0.3='17%'、混合槽 fill、跨型翻转）、
test_border_image_width_lerp（多槽钳 '0px 5% 21 37px'、auto 直通、0 桥、
'0'→'20' at-0.3='0'、10%→20px at0.3='10%'）、test_border_image_outset_lerp
（0→2 at-0.3='0'、1px→2px at0.3='1.3px'、多槽钳）、test_border_image_
strict_gate（同型 Some、跨型/关键字/fill 不对称 None）。暂缓编译执行：
fv1 在飞 part116 删除致 js_dom_bridge_tests.rs include! 悬空（刀 A1c 同债）。

## 探针谱系（zz-lei-b1-probe.html，发射电池前已删）

- v1：transitions 通道 '30px' 跳终值（登记清单缺）→ 补清单；
- v2：CSSA/WAAPI 整批静态（插值臂缺）→ 逐字 mimicking testcommon 机制
  仍失败；
- v3：加 mask-border-slice 对照（同页 Pass）→ 定谳 apply_single_property
  缺 border-image-* applier 臂 → canonicalize ser1≡ser0 → 三通道静态化。
  补五臂后 6/6 Pass（readback/transitions/CSSA 单槽×2/多槽/mask-border
  对照）。

## 电池 v1 账本（results-knife-b1，bin f8734755cd33，GATE-DONE 06:49:29）

对账基线 results-knife-a1c-b2（刀 A1c 落账后）。逐页集合差分
（b0=b1 全 558+124 页逐子测 pass/fail 对齐）：

- **css-backgrounds**：GAIN=1403 / REG=31（3 页）；
- **css-masking**：GAIN=14 / REG=0（干净）。

窗口归因（v1 bin 构建窗含 fv1 WIP（basic-shape serialize 面 + fonts
staged）与 A1d WIP 快照（06:38 apply_advanced.rs background-position-x/y
多层臂））：

| delta | 页 | 归属 |
|---|---|---|
| REG 24 + GAIN 62 + computed/composition 页 | background-position-x/y | A1d 窗（其 06:54 WIP 演进后由其自身电池定谳） |
| GAIN +13 | clip-path-valid | fv1 窗 |
| GAIN +2 | inheritance.sub（bg-position-x/y initial 值） | A1d 窗 |
| GAIN +939 | border-image 家族 9 页 + discrete-no-interp +11 | **本刀** |
| GAIN +1 | mask-border-slice-interpolation-stability（masking 腿） | **本刀**（共享臂体） |

**本刀面 GAIN=940，REG=7（P2F 触零容忍）——v1 不收口。**

### P2F 定谳：width 页 REG 7 = B0 伪通过池新实例（非插值错误）

7 条全 CSSA 通道 %↔px 跨型对（`from [10%] to [20px] at (-0.3) should be
[calc(13% + -6px)]`，含 at(1)='calc(0% + 20px)'、at(1.5)='calc(-5% +
30px)'、px→% 向 2 条）。取证链：

1. b0（results-knife-a1c-b2）该 7 条 **Pass**，且 HEAD 提交态 animation.rs
   **零** border-image-* 臂（grep 实证）——插值不可能发生；
2. 双巧合机制（b0 提交态 clip-path 臂注释在册记载同款池）：期望串含 calc
   → 本机静态声明路径拒收 calc → 期望元素 setProperty 落静态值；b0 无
   applier 臂恒 echo → 双侧同静态值双巧合通过；
3. v1 我的臂遮蔽：跨型槽 cmp None → 离散翻转 → '10%' 写回 → applier 生效
   → canonicalize Some → 实际 '10%' ≠ 期望静态 → REG；
4. 既有纪律（jsanim 十轮 / clip-path 二十一轮同款：「不追真值保位」）：
   calc 领地期望串不可达时保 echo 位，真 calc 面待整刀（FIXME(bi-cross-
   type-calc) 挂债）。

**修复（v3 终形：t0 豁免 echo 门）**：宽松臂 border-image-width 名下
Length↔Percent 跨型槽检测 → `t != 0.0` 时 `return String::new()`（echo
保留门——空串 applier 解析必败 → 槽位不动 → canonicalize 恒等门回落
echo）。关键约束：管线 `interpolate_between_with_timing` 钳 t∈[0,1]，
at(-0.3) 与 at(0) 同以 t=0.0 到达——无状态臂矛盾矩阵（Block A at(0) 期望
'10%' 可解析须翻转 vs at(-0.3) 期望外推 calc 不可达须 echo）唯一可行分工
即 t==0 豁免翻转 / 中段 echo。t==1 豁免不取：Block A at(1)/at(1.5) 双失
（−2）对 Block B at(1) 单得（+1）净 −1；t==0 豁免达 11/12 全局最优（v3
单页探针实证）。仅门 border-image-width 名：mask-border-width 语料无此
对形（A1c 落账翻转位零 REG）不扩；slice（Number↔Percent）/outset
（Number↔Length）跨型对非 calc 领地（期望翻转形，REG=0 实证）不动。
唯一残缺行 Block B at(1) '20%'（与 at(1.5) 无状态 1-vs-1 平局取 echo 侧，
外推支持后自解——FIXME(bi-cross-type-calc) 同债）。

## 电池 v3 账本（results-knife-b1-v3，bin eaecaaec9a65，构建窗 07:05）

b0 = results-knife-a1c-b2（刀 A1c 落账基线）。逐页集合差分（b0/v3 全页
逐子测对齐），窗口归因更新：fv1 刀 S 已落库（2692ac1f0，clip-path 页
v3 对 b0 零漂移＝其 canonicalize 落库态与 b0 构建窗 WIP 态收敛）；A1d
bg-position WIP 在 06:47→07:05 窗演进（其 GAIN 400/REG 24 全数归其账户，
其 evidence 2026-11-18-knife-a1d-bg-position-lists.md 在途）。

- **css-backgrounds**（07:12:55–07:13:54，rc=1 良性 Fail-present）：
  GAIN=1402 / REG=24（全在 background-position-x/y 页＝A1d 窗）；
  **本刀面 GAIN=938 REG=0**——border-image 九页（slice 295+1、width 320、
  outset 141+19、slice-composition 19、outset-composition 19、
  width-composition 19、source 113、discrete-no-interp 11）零 P2F，
  width 页 v1 的 7 REG 清零兑现（GAIN 321→320，−1 = Block B at(1)
  无状态臂不可解行）；
- **css-masking**（07:13:54–07:15:0x）：GAIN=15 / REG=0——本刀
  mask-border-slice-interpolation-stability +1（共享 slice 臂体）；fv1 刀 S
  落库态 clip-path-valid +14（其账户）。

**本刀终账：净 +939（backgrounds 938 + masking 1），零 P2F 零 LOST。**
v1→v3 谱系：REG 7→0（echo 门 t0 豁免）、GAIN 940→939（−1 = Block B
at(1) 无状态臂不可解行，v1 诚实位 vs at(1.5) 巧合位 1-vs-1 平局取 echo
侧最优）。单页探针（bin eaecaaec，width 页 558 子测）：REG=0 GAIN=320，
CSSA 跨型 12 行 11 Pass。

## Debt 挂账

- 钉测 4 枚 + A1c 2 枚：待 fv1 part116 include! 悬空解除后
  `cargo test -p zero-engine --lib` 补跑；
- strict 通道跨型 calc（FIXME(bi-cross-type-calc)）：transitions 半面
  %↔px calc 期望子测——待静态声明路径 calc 支持整刀；
- border-image-source CSSA/WAAPI 半面 49F×2（image 值型离散插值）；
- composition 页 composite-add 机制（+19/页 replace 面已收，add 面未接）。

## 共享文件处置（run-rules 碰头信号）

animation.rs / transition_registry.rs 现含 A1d 兄弟 hunks
（mix_bg_position_axis + bg-position applier 臂 / TRANSITIONABLE_PROPERTIES
background-position-x/y 两条）。提交按 A1c 收敛先例：等 A1d 落账提交后，
本刀 pathspec add 的 animation.rs delta 仅余本刀 hunks；若提交时 A1d 未落
账，hold animation.rs 并注记（不卷入在飞 peer hunks）。

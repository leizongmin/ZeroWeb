# 刀 A1d：background-position 三页残面（单轴多层列表 + transitions 静态 to）收口

日期：2026-11-18。CLAIM：`.acceptance/wptfyi-css/CLAIM-knife-a1d-bg-position-lists.txt`。
基线账：`bin-knife-a1c-v2 26153f17`（刀 A1c b1）css-backgrounds 腿三页残面。

## 靶面与根因

三页（A1c b1 账）：

1. `background-position-x-interpolation.html` 28F + `-y` 同 28F：全部
   `'from' value should be supported`——`CSS.supports('background-position-x',
   '300px, 400px')` 拒收（apply_advanced.rs R5185 单值臂，多层逗号列表 FIXME）。
2. `background-position-interpolation.html` 186F：三类（expected 空层 48F /
   neutral-from 26F / from[initial] 112F）got 全部 = 静态 to 值。

根因（探针四轮定谳）：**canonicalize 往返缺 applier 臂**。js_anim_registry
`canonicalize` = serialize(base) → `apply_to_computed_style` 写回 → serialize(tmp)，
applier（animation.rs `apply_single_property`）无 bg-position 臂 → 写回零效果 →
ser1≡ser0 → None → transitions 查询回落级联基值（= 静态 to）。刀 A1c ②补了
TRANSITIONABLE 清单与 lenient 插值臂两件套之一，漏本臂——与刀 A1 mask 族同病同治。

## 修（码面全清单）

- `style-system/property/apply_advanced.rs`：R5185 单值臂 → 单轴多层列表臂
  （css-backgrounds-4 §2.4 `<position-x>#`/`<position-y>#`；层重复 css-backgrounds-3
  §3 cyclic：层 i 取 `list[i mod n]`，层数扩 max(既有, 列表)；all-or-nothing 拒收保旧值）。
- `engine/animation.rs`：①applier 委托臂（bg/object-position →
  `apply_advanced_property_value` 单点写回，同刀 A1 mask 臂形）；②lenient x/y 臂 +
  `mix_bg_position_axis` helper（mask_pos 机制复用，逐层轴分量 LCM 循环 lerp）；
  ③strict 委托 +2 名；④`mask_pos_comp` Calc 臂（css-values-4 §10.11 混合形
  `CalcExpr` 递归展平 %/px 项）；⑤EdgeOffset 符号口径修正（见下节）。
- `engine/js_dom_bridge/transition_registry.rs`：TRANSITIONABLE_PROPERTIES 74→79
  （A1d +x/y；含对端刀 B1 border-image +3 同 decl）。
- `engine/js_dom_bridge/computed_style.rs`：gCS x/y 臂（逐层轴投影 +
  `bg_position_axis_to_css`）；`font_size_px` 线穿三串化器（background/object-position
  与 axis）；`P::Calc` 臂 → `calc_computed_to_css` canonical 口径（% 项保留、
  零 px 项丢弃、加法脊项化）——query 通道 canonicalize 往返出混合形依赖此臂。
- `style-system/property/inherit.rs`：`background-position-x|-y` 独立继承臂
  （css-backgrounds-4 §2.4 只补丁目标轴，父轴列表循环取模，另一轴保己值）。
- 钉测：`style-system/tests/apply_coverage.rs` `test_apply_background_position_axis_layer_lists`
  （列表扩层/轴位、y 轴 center+%、未知 token 整条拒收保旧值、3 值列表 cyclic 扩 3 层）。
  lib 全量 2566/0。

## transitions from[inherit] REG 深挖（首轮电池后三连修）

首轮电池 x/y 两页各 REG=12（from[inherit] 形 6 t×2 forms，P2F 违约）。双 bin A/B
矩阵定位：Animations/Web Animations 通道全过、CSS Transitions 通道 t≠1 败 t=1 过
→ from 捕获路径差异（cssa 经 parent 解析 inherit 正确；transition_registry 取
LAST_STYLE store 态快照）→ 插桩 register from/to + mix 输入输出定谳三层根因：

1. `mask_pos_comp` 无 Calc 臂（`Bpv::Calc(_) => None`）→ 端点串化 calc 形解析失败
   → mix None → t≥0.5 回落 to。
2. `bg_position_axis_to_css` 的 `P::Calc` → 空串 → query_core 末步 canonicalize
   往返 `Some("")` → got ""。修为 `calc_computed_to_css`（computed_style.rs:2207）。
3. inherit.rs 无 x/y 独立继承臂 → from[inherit] 落 initial `0%` 而非父值 `60px`。

**二分插曲**：修 2 后 origin 页反而 -46（234→188）——临时切回 `Calc => ""` +
A1D-BISECT-CALC-OFF 标记 → inherit-only bin=234，Calc 臂是肇事；真因是
`mask_pos_comp` EdgeOffset 臂符号反：right/bottom 缘语义 = `100% − 偏移`
（css-backgrounds-3 §3.6，`right 20px` → calc(100% - 20px)，偏移项取负），旧码
(100,+20)。mask 语料无 right/bottom EdgeOffset 对，A1 账面未暴露；INH bin 曾因
canonicalize None 回落 cssa 正确值而伪通过。符号修复 + 修 4（Calc comp 臂）后
origin 258（超旧上限 +24），恢复 Calc 臂。

solo 终态（`a1d-reg-bin-*.json` / `a1d-xpage-*.json`）：origin 18→258 (+240)、
x/y 各 84→103 (+19，残余 9F)、main 10→190 (+180，残余 6F)。残余 = transitions
简写对 echo 展开位（2F×2 页）+ WAAPI 列表通道 got "40px" 单值（7F×2 页）——
后续刀候选，非回归。

## 探针定谳（probe 页账后删，JSON 存 `.acceptance/wptfyi-css/a1d-probe-post.json`）

- P2：`CSS.supports` x/y 多层列表 false → **TRUE**（"should be supported" 面根因清）。
- C2/C3/C4（harness 两阶段流，`-50s` 编 t=0.5）：bg-position 静态 to → **40px 中点插值**；
  C3 keyword-edge 形（`top 0px left 0px`→`left 80px top 80px`）、C4 4 层 cyclic 全通。
- 对照无扰：C1 width=30px、C5 mask-position=40px（mask 通道不受扰）。
- probe1/probe2 同步快测形异常读数归因：共享元素 #t 跨 P1–P10 残留 + 同步无 flush
  登记语义（P5 4 层基 ×2 项列表 cyclic 补层 [300,400,300,400] = 规格正确；P8–P10 =
  P4/P7/P8 在飞过渡回显）。无引擎缺陷证据。
- C6 简写 behavior supports 形仍 false（s3 `transition-property: allow-discrete` true）
  ——相邻残面，留下一刀候选。

## battery A/B（bin-knife-a1d `37c812e0…`，06:47:55 树态构建）

首轮三腿（css-backgrounds/css-transitions/css-typed-om；`knife-a1d-ab-run.log`）：
css-backgrounds gained=1372 / lost=0（含对端刀 B1 border-image 面 +921，按 CLAIM
共存注记窗口法证归对端——对端账本 0792b2990 落册数 +939 同窗互证）；首跑暴露
x/y transitions from[inherit] REG=12/页 → 上节三连修后 v2 终轮电池重对账。

v2 终轮（bin-knife-a1d-v2 `de8977d4…`，插桩已删 + 本刀 hunks fmt 清 + 探针页已删，
四腿含 css-masking 符号零漂移面）。**发射撞车让位**：本 session fork 自身 r3
发射（07:38:44）与 sibling fork `wptfyi-a1d-v2r2` 在飞电池撞车（同刀同四腿，
corpus 形态 `results-knife-a1d-v2/` + transcript 分腿落盘）——按先手让位协议
停己采彼一账两用：r3 停（只跑掉 ~90s 腿一段，部分产物已清），采 r2 现轮账
（bin 同 provenance `de8977d4`——r2 起跑晚于本 fork 重建，实跑同一 target bin；
r2 前轮 07:35:58 收账为插桩版 bin，行为同面不计）。r2 形态带
`CSS_CORPUS_TRANSCRIPT` 分腿唯一路径（corpus 纪律齐备）。

**bg 腿双对账（vs b0bg / vs a1c-b2）**：gained=1442 / gained=**1437**，双双
lost=0 regressed=0 pages_loss=0（`ab-compare` 全列表）。vs b2 逐页分解归因
零残差：**对端刀 B1 面 +938**（border-image 8 动画页 +927——outset-composition
+19、outset-interp +141、slice-comp +19、slice-stability +1、slice-interp +295、
source +113、width-comp +19、width-interp +320——加 discrete-no-interpolation
+11，恰合其账本 "+938 九页"）+ **本刀 A1d 面 +499**（main 简写页 10→190
+180、origin 18→258 +240、x/y-interp 84→103 +19×2、composition 0→2 +2、
parsing/computed 28→31 +3、x/y parsing-computed 0→16 ×2 +32、inheritance
51→55 +4）＝1437 分解闭合零余。solo 预言逐页命中（origin 258、x/y 103、
main 190 与 solo 终态一致）。

**masking 腿（符号改 mask 机制零漂移证明面）**：gained=15 lost=0 regressed=0
——+1 mask-border-slice-stability（刀 B1 账）+ 14 clip-path-valid 40→54/0F
（对端 fv1 刀 S `2692ac1f0` clip-path canonical 落库兑现，其账本已认领）；本刀
mask-position 页零位移＝EdgeOffset 符号修正 + Calc 臂对 mask 语料零扰动
（mask 语料无 right/bottom EdgeOffset 对，A1 账面预测复验成立）。

**transitions 腿（REG 守卫面，vs flip-b1 11-15 四页基线）**：gained=267
lost=0 **regressed=0** pages_gain=46——基线四页零位移（REG 守卫成立）；267
全部落在基线外的 46 页＝corpus 形态页集远宽于 flip-b1 bin 形态四页的覆盖
扩展面（pooled 非本刀可认领，不进净账；properties-value 族三页
properties-value-001/inherit-001/inherit-002 走 corpus 外部墙钟 90s kill
记 Timeout 行账面不缺案——事件系统大刀已知候选面，非本刀引入）。
`transition-background-position-with-edge-offset.html` 0→1 Pass（EdgeOffset
符号口径页直证）。

**typed-om 腿（共享串化器 drift 面，vs a1c-b2 359 页基线）**：gained=0
lost=0 regressed=0 **SET-IDENTICAL**——font_size_px 线穿三串化器对
typed-om 语料零扰动。

**A1d 净账定谳：+499，P2F=0 / LOST=0 零容忍全腿成立**。bg +499（B1
+938 / fv1-S +14 全数归对端不重账）＋ masking 0（零漂移）＋ transitions
0（REG=0，gains 为覆盖扩展 pooled 不认领）＋ typed-om 0（SET-IDENTICAL）。
本轮 A/B 局部账累计 4477 → **+4976**。电池收账 07:49:31
（`results-knife-a1d-v2/` 四腿 JSON + `transcript-knife-a1d-v2-*.txt` 四份
分腿 transcript，transitions 腿含 3 行 properties-value 族外部墙钟 Timeout
账面不缺案）。

**v2 四腿 totals**（BATTERY-DONE 07:49:31）：css-backgrounds 3442/6246、
css-masking 4701/6355、css-transitions 365/1524、css-typed-om 11072/12359。
六组对账（`reconcile-a1d-v2.py`）汇总：bg vs a1c-b2 +1437/0、bg vs b0bg
+1442/0、bg vs own-b1 +67/−2（见下节定谳）、masking vs a1c-b2 +15/0、
transitions vs flip-b1 +267/0、typed-om vs a1c-b2 +0/−0——**四腿账面
P2F（LOST）全零**。

- css-backgrounds：vs b0bg（knife-p-b2 `2faea6ed`）与 vs a1c-b2（`26153f17`）双对账。
- css-transitions：vs flip-b1（11-15 最新全腿，REG 只认真回退）。
- css-typed-om：vs a1c-b2（零漂移面）。

注：b1 bin 同树含对端在飞 WIP（刀 B1 border-image 面 / css-parser serialize /
js_dom_shim）——bg 三页归本刀、border-image 页变化按 CLAIM 共存注记窗口法证归对端。

## own-b1 窗口对账与 composite:add 欠账定谳

vs own-b1（37c812e0 首轮 bin）：+67/−2。+67＝三修兑现（origin +24＝Calc comp
臂、x/y-interp +12×2＝inherit 臂 + gCS Calc 串化、x/y-computed +4×2、
position-computed +3、inheritance +2）+ 对端 t0 echo 门差 +6（width-interp
512→518，B1 v2→v3 窗）；**−2＝composition 页 Compositing CSS Animations 2 形**。
定谳：`animation-composition:add` 写于 @keyframes 的通道 engine 全缺
（interpolation-testcommon.js:55-62 harness 形；全仓仅 shim typed-om 提及）——
got=lerp(原始 add 端点对) 不含 underlying 累加（at .25 got 50% 75% =
lerp((60,80)→(20,60))，期望 90% 95% = underlying(40,20)⊕两端点后 lerp）。b1
窗口该 2 形通过 = pre-Calc 串化时代 canonicalize 失败回落与期望离散撞点的
伪通过；vs 权威基线 a1c-b2/b0bg 该页 +2/+2 零账面 P2F，非回归。

## 后续刀候选（本轮量化）

TransitionEnd/CSSTransition 事件系统大刀（properties-value 族 ~1000F+event-dispatch
30F）；z-index 插值臂 166F；text-shadow 152F+60F；vertical-align 136F+40F；
transition 简写 behavior 槽（C6 证 decl_supported 缺 behavior 分支 ~5-8F）；
composite:add accumulation（@keyframes `animation-composition` 通道全缺，
underlying ⊕ delta 端点预累加，css-backgrounds-3 §3 + web-animations 效果合成
——composition 页 24 形，own-b1 窗口 REG 2 源）。

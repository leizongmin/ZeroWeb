# 刀 MB：mask-border 三长手 addition/composition face（2026-11-20）

## 池与 face

池＝css-masking/animations mask-border 七页 composition/interpolation 残面（slice/
width/outset 三长手 + source）。face＝单文件 `crates/engine/src/animation.rs`
（v3 patch `knife-mb-face-v3.patch`，对 HEAD +242/−7）：

1. `compose_animation_value` 三臂（v2）：mask-border-slice/width/outset 逐槽同型
   求和（web-animations-1 §5.4.4 addition；Number+Number / Percent+Percent /
   px+Px），值域非负钳 0，任一槽型失配 → replace 回落 animated 原文；输出走
   gCS 同款 `format_num` + `box4_str_min` 串化器。
2. compose 准入门扩三长手（v2）：AnimationClock 注册块可合成域名单
   text-shadow/box-shadow/vertical-align + mask-border 三长手。
3. strict 通道 slice/outset 收 B1 `border_image_same_type_slots` 同型门（v2）；
   source 恒 None（§3.4 纯离散）；repeat 维持宽松委托（v2）。
4. **v3-⑤**：注册块均匀 add 分支，mask-border 三长手改**端点预合成**——
   compose(U,V_from)/compose(U,V_to) 先行、keyframes 携合成值参与插值、不再登记
   `compose_result` post-compose。依据 web-animations-1 §5.4.4：加法合成先于
   插值，计算值期非负钳 0 落在插值结果上；post-compose 把逐槽钳 0 提前到插值前，
   负外推位（语料 at:-0.25 expect '0'）错成 underlying+0。影列表 append 语义非
   仿射必须 post-compose、vertical-align 同单位数值加线性插值下与预合成恒等——
   两者既有账不动。
5. **v3-⑥**：strict 通道 width 独立门 `mask_border_width_interpolable`——同型槽
   对（auto/auto、number/number、length/length、percent/percent、0 桥接）+ 跨型
   Length↔Percent 对放行（css-masking-1 §3.7 `<length-percentage>` 域跨型对
   computed 值插值语义 css-values-4 §8.5.6；本机跨型 calc 插值未接
   FIXME(bi-cross-type-calc)，lenient 臂端点翻转位＝HEAD 基线行为保位）；
   Auto↔值 / Number↔length-percentage 失配仍 None（不启动过渡）。

单测：`test_mask_border_width_strict_cross_type_gate` 新钉 8 断言 + 既有
mask-border 七钉全绿（8 passed）；`cargo clippy -p zero-engine` animation.rs
零警告（v2 Px 臂冗余 `as f64` 已剔）；engine lib 全量见收账段。

## 代际史（v1→v2→v3）

- **v1**（knife-mb-battery-0359.log，b1 md5 `ebe1c14a…` 判读绑定）：face 缺
  border_image_same_type_slots 的 mask-border 别名归一 → strict 门恒 false →
  全部 mask-border 过渡条目 discrete 取消。gained=148 lost=196 regressed=272
  pages_loss=3——假判代，不入账。
- **v2**（knife-mb2-battery-0509.log，b1 md5 `e4f57d93…`）：+别名归一后
  gained=212 lost=0 regressed=12 pages_gain=6——12 REG 两域定谳：
  ①outset/slice composition ×4（at:-0.25 钳位 got=underlying：post-compose
  钳 0 提前）；②width-interp transitions 通道 px↔% ×8（同型门拒跨型对→恒 to，
  b0 lenient 端点翻转位与陈旧期望位恒同型）。05:0x 后 429 断轮，v2 归因由
  续轮接管。
- **v3**（本轮，knife-mb3-battery-0748.log）：v2 面加 ⑤⑥ 两换道。

## canonical 电池（knife-mb3，07:33:44 发射）

- b0 `70d808d3dd3117e680ff6d84cdb638bc`（face 剥除，37s）/ b1
  `e8bfda094539bc30d0eb3bd64a3a0066`（face 入树）——M2 异 md5 过；face 标记
  `mask_border_width_interpolable` 二进制切片 b1×1 / b0×0。RIDE empty
  （porcelain 仅 1 face 文件）。
- 腿：css-masking（主靶）/css-transitions/css-animations（哨兵）×{b0,b1}，
  test-guard 包裹（build 6/14/1800s，leg 4/8/3000s + CASE_WALL_S=60）。

### 主靶 css-masking mask-border 六页

| 页 | b0→b1 | Δ |
|---|---|---|
| mask-border-outset-composition | 19→56 | +37 |
| mask-border-slice-composition | 19→56 | +37 |
| mask-border-width-composition | 19→56 | +37 |
| mask-border-slice-interpolation | 403→433 | +30 |
| mask-border-source-interpolation | 252→294 | +42 |
| mask-border-width-interpolation | 492→534 | +42 |

**+225 零 P2F 零 regressed（六页逐页 REG=0）**。v2 的 12 REG 全清：composition
四 REG（at:-0.25 非负钳 '0'）转绿＝端点预合成兑现；width-interp 八 REG
（transitions 通道 px↔% 陈旧期望位）转绿＝跨型放行兑现。composition 页 +37
含 CSSA '175 250 300 450' 形转绿（预判 +1~2 兑现超出——v2 时 CSSA 该形因 add
臂槽型失配回 replace 而 F，v3 预合成后同型可插值）。残面：composition 三页各
56F＝WAAPI 通道负外推位（got=underlying，WAAPI 采样不外推，b0 亦红预存域缺
口）+ 跨槽型失配 replace 位；width-interp 残 24F＝px↔% 真值 calc 系数形
（FIXME(bi-cross-type-calc) 独立刀）+ WAAPI 采样 3F。

### 哨兵

- **css-transitions：SET-IDENTICAL**（对账零行——逐页逐位零差异）。
- **css-animations：两页 ΔP 定谳 flake 不入刀账**——Element-getAnimations.
  tentative 16→1（REGRESSED=15）+ event-dispatch.tentative 0→1。同二进制
  solo 复跑分布：b0 {16, 6, 6}、b1 {1, ~5, 21, 6}——两臂分布交叠、b1 单跑曾
  达满贯 21P，页面级蒸发（harness Timeout "2 test(s) unreported" 待决测试整批
  翻红）为运行间时序噪声，与 b0 无关非 face 归因；对端 SV2 同电池同页摆动
  「并发负载下时间线敏感页 flake 不转抄」同判（cancel-003/inherit-002 先例
  同族）。**刀账口径：主靶 +225，哨兵视为零差异（噪声双页 ΔP 中性剔账）**。

### 收账

**刀 MB 净 +225**（masking 主靶六页），累计 A/B 局部账 +7203 → **+7428**。
engine lib 全量 4232P/0F（guard --compile-first 4/8/1800s，31.5s）；fmt/clippy
净（animation.rs 零警告；HEAD 上 css-parser `ok()` 冗余、css_style_declaration
collapsed-if 两警告为他流在库预存非本刀面，提及不修）。

残面挂账：①mask-border composition 残 56F/页 WAAPI 通道负外推采样缺口
（got=underlying——WAAPI 引擎无负 currentTime 外推，独立刀域）；②width px↔%
真值 calc 系数形 FIXME(bi-cross-type-calc)（跨型 calc 插值通道，同挂
border-image-width）；③slice/width/outset composition 跨槽型失配 replace 位
（语料显式 expect replace 形，已按规范回落，非缺口）。

## 续轮收账附记（11-20 08:2x）

- 断轮交接：前轮 429 断于证据成文后、commit 前；本续轮核 face==v3 patch 字节恒等
  （blob `0304c3904`）、mb3 电池 log reconcile 六页 +225 复对一致后落库。
- face 随 `git pull --rebase` 重放于 `743e1bc59`（同窗落库 d63 面=js_dom_bridge/
  part01.js 域，不触 animation.rs）——animation.rs blob 判读字节未变，提交号
  `2e567a891`。engine lib 4232P/0F 本轮复核（test-guard --compile-first 4/8/1800s，
  测试相 18.25s rc=0）。
- 共享面提示：ZeroWeb-2 clone ffix2 face 同在 animation.rs（apply_single_property
  域 ~4319，与本刀 hunk 域不相交），其收账前需对 post-MB HEAD 重生成 patch。

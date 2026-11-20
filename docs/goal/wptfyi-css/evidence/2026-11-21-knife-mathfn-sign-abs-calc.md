# 刀 MATHFN · sign()/abs() 写面代数 + calc 字体基准求值（2026-11-21）

## 目标与机制

css-values-4 §10.5 单参数数学函数 sign()/abs() 的写面（无布局上下文）
求值通道此前 fail-closed 整拒，signs-abs-computed 60+ 形 INVALID。
四层机制：

1. **sign 单维度叶短路面**：font/viewport 相对叶（em/vh/s/% 族任一）的
   符号 = 标量符号，与解析上下文无关（§10.9 符号位：-0 按 IEEE 传零，
   `sign(sign(-0px))` = -0 驱动钳形三单位断言）。负维度叶 parse 期折
   `0px - x` 保族负号重写（Time 叶 `0s - x` 镜像 Length/Resolution 先例，
   Number 叶不毒化纯维度门）；零长被减数量纲对符号不可知
   （0 − 正维度 < 0 恒成立）。混合和式 `sign(42px - 2em)` 不匹配短路面，
   走完整求值（默认字体基准）。
2. **abs() 型别保留**：abs(X) 结果随 X 维度族（§10.5 型表），不短路面
   （abs(Nvh) 须 px 化供长度槽）；sign(X) 恒产出 <number>——
   `dimension_suffix` Sign 臂 None（progress 同口径先例），实参维度
   函数内消解不外泄（`sign(1px)` 在 Number 位折叠 `calc(1)`，
   不得折出 `calc(1px)` 量纲泄漏）。
3. **calc 字体基准求值**：`eval_calc_with_font` Some 臂补 root_font_size
   根基准——rem 叶树在 scale/progress 等 <number>/<length-percentage>
   槽真实 apply（此前 Some 臂漏根基准令 rem 叶树整拒，
   scale-animation-math-functions 26→0 根因）。
4. **槽级 % 可解性分层**：% 叶无解析上下文不可决（§10.5
   percentage-value），四槽族门 time/angle/transform-helper/integer +
   sign 短路面 % 叶排除（`sign(50%)` 短路 None 回落全 eval fail-closed）；
   同型百分比比值树（progress/min/max 全 % 族）按 §10.5 型表上下文
   无关消解为 <number>（`eval_number_ratio_pct_free` 基准消去）——
   <number> 槽不设 blanket % 树级门（误杀 progress 纯 % 形）。

face `MATHFN sign-abs-calc-algebra-eval-font-basis`（`#[used] static`
代码字面，types.rs 单 canonical——r1 曾五重同名 static E0428 撞窗，
marker 函数名不进 release 符号表＝FM-FAIL 自拒史，改字面后并一）：
types.rs / parse_transform.rs / parse_extended.rs / transform_helpers.rs /
serialize_math.rs / parse_extended_border.rs / style-system property
parse+apply+apply_advanced+matcher supports / engine animation.rs（调用点，
MOTION-2 钉测 hunk 不属本刀）+ 钉测 sign_abs.rs（6）/ mathfn_pins.rs ×2
（style-system 7/engine 1）/ 既有表测期望订正（values_math_type_table
sign 型别擦除面 / r2280 / r5076 / progress_function / calc_mix /
infinity_nan）。

副产物修复随刀入库：parse_column_width calc 片段括号平衡门（paren
tolerance EOF 截断副作用守卫——r3749 `calc(6em + 5px` 片段借道复活，
clip 入口同款为先例）；
order/z-index 裸数准入门（`is_math_function_value`，calc-rounds-to-integer
`10.1`/`1e1` 拒）；animation-duration 裸负字面量前置拒（`-3s` 超
`<time [0s,∞]>` propdef 文法界，不得借 math 臂 Time-parse 负号重写复活；
calc 形负值 parse 放行按 §10.12 computed 期钳）。

## 电池

三重波折史（salvage 仲裁 notes 全录）：r1 marker 函数名 FM-FAIL 自拒
（rc=5 快速失败）→ 对端 salvage 复用 r1 双臂 bin 补腿 → r2 撞 E0428
窗退场后以陈旧 face（03:52 patch 快照不含后续四修复）跑完＝考古对照
不入账 → b1p（04:24 树）RECONCILE-V2 gained=161 lost=5（残① blanket %
门 −4 progress 两页 + 残② duration 裸负 −1）→ **b1p2 双链碰撞窗
（04:44-04:50）＝毒化考古，不入账**：两链同写 r1/b1（rerun.sh
04:41:11 被改靶 OUT=r3→r1），ledger unit 外部 systemctl stop
04:50:19 杀、0446 unit 04:49:04 自退；其 bin 54d089f0 build 于
04:46:2x 早于残①编辑波次（transform_helpers 04:47:25 /
parse_extended 04:47:29 / types 04:47:39）＝旧树混代——双链死亡
反而避免旧代伪账 → **b1p2（04:47:39 树，残①②全在）＝canonical
收账轮**（unit zw-mathfn-b1p2-canonical-0453，04:53 发射，跑
knife-mathfn-b1p2.sh 原文 04:35:31 版，systemd 发射，test-guard
包裹，b1p-v2 保全目录跳过复制不受扰）。

- 双臂：b0 `04335cb3`（r1 原账 face 剥，全程复用）/ b1
  b1p2-runner `4afc2977`（04:53:23 build，含残①全 face）。
- 判别：FACE-MARKER b0=0/b1p2=1；M2 异 md5（b1p2 `4afc2977` ≠ b0
  `04335cb3` ≠ b1p `79f52938` ≠ 双链代 `54d089f0` 四方异，防 stale
  build 毒化）。
- 4 腿：values/transforms/animations/typed-om，rc=1（runner 有败测试
  正常）。

## RECONCILE V3（v3 终账，05:0x canonical-0453 落，b0 `04335cb3` vs b1 `4afc2977`）

独立 recount（本流重算，不转抄脚本 TOTAL）：

- values +103/−0：signs-abs-computed 139→218（+79）、minmax-angle
  27→32、minmax-length 76→80、sin-cos-tan 20→26、acos-asin 27→31、
  hypot 45→49、typed_arithmetic 6→7。
- transforms +27/−0：rotate-interpolation-math-functions-tentative
  0→25、transform-with-sign 3→5。
- animations +16/−0：Element-getAnimations.tentative 1→17。
- typed-om +15/−0：the-stylepropertymap iterable.tentative
  computed/declared/inline 三页各 0→5。
- **总净 +161，零回归**。v2 的 5 失位全数复原（values
  progress-computed/progress-serialize −4、animations
  duration-invalid −1 → b0 持平）：残①②修复页级效应＝回归消除；
  r3749 平衡门与 scale 钉的 unit 级转绿不新增页级 P（其 WPT 页
  断言结构不按此分裂）。gains 161 与 v2 同集。
- 脚本自带 RECONCILE 与独立 recount 一致（gained=161 lost=0）。

## 哨兵

- S1/S3 ride census 13/13 OK（salvage 轮复核，repo root 口径）。
- ASYM ×2（display-none-dont-cancel.tentative b0=4/b1=5、event-order
  b0=1/b1=4）＝断言计数不对称，挂起不计（runner 发射面差异，非 P
  位移）。

## 残面

- animations ASYM ×2（display-none-dont-cancel.tentative
  b0=4/b1=5、event-order b0=1/b1=4）＝断言数不等挂起不计。
- 语义风险（语料外记档）：<number> 槽 ratio 消解路径对 `sign(50%)*100`
  型（sign 内裸 % 未消解）可能放行（eval_number_ratio_pct_free 合成
  parent_length=100），Blink 口径疑拒；语料无钉面，后续可加钉收敛。
- 未接入通道（b0/b1 双败，非回归）：font-style oblique / filter
  hue-rotate / offset-rotate / offset-path ray / color hsl 角度槽
  数学费——通道缺口另刀。
- parse_length 通用 length 入口截断片段洞（column-width 同款括号
  平衡门缺失）：未钉测未修，挂残面清单。
- 域外提交红 9（测试与 face 无调用关系+近期他刀触碰＝提交态红，不入
  本刀账、随总账归因修）：css-parser break-before/font-face ×2；
  style-system t54 font gate（fontcycle `8b70cb9fe` ch 兜底 1em vs
  T54 `298f54c14` ch=0.5em 钉矛盾）、clip-rule initial_value ×2
  （CLR `f654c2673` 注册漏臂）、r3752 ×4（optical `413730881` 第 17
  项复位子属性未同步计数断言）。

## 补签与归因订正（修复流，05:05）

- **世代差隔离，本刀入账 +146 非 +161**：typed-om iterable ×3 +15 为
  OM-10 世代差非 MATHFN 效应——法证：r1 代 stale b1（710df692，03:45
  build）该三页全灭且失败串 `styleMap.entries is not a function`＝OM-10
  maplike 协议整体缺席；OM-10 提交 9015db203（03:33:42）产于他 clone，
  本 clone 同步在 03:45-04:24 窗，r1 双臂 bin（b0 `04335cb3` 03:44 /
  b1 `710df692` 03:45）均前于同步。该 +15 已在 OM-10 收账笔（局部账
  +10406）入账，**不得二次计入**。
- **其余同步提交中性证**：r1（03:44 bin）与 r2（03:54 bin）双轮
  reconcile 逐页恒等（+102/−46）＝窗内同步提交（BSHAPE 数值 canonical /
  DC-2 html 套件注册）对本刀四语料零效应；世代差唯一表证＝iterable
  （fail 串法证如上）。
- **+44 修复窗进化增益计入本刀**（机制匹配）：rotate-interp-math +25
  （角度槽门+sign 型别擦除）、Element-getAnimations +16（math duration
  受理→动画注册链恢复，stale 双侧 1P→新代双轮 17P 确定性）、signs-abs
  +76→+79（% 槽形追加）；同族差分 stale→b1p2 +101/−1 中同集（−1 为
  b1p 代 duration 中间态，v3 已愈）。本刀入账构成：r1/r2 同代已证
  +102 + 进化 +44 − 世代差 +15 ＝ **+146 F2P，零回归**。
- **语义风险段订正**：`sign(50%)*N` 放行风险已收敛非挂账——
  `calc_expr_has_pct_under_sign_abs_erasure`（sign+abs 擦除内裸 % 否决、
  progress 实参豁免）双点接线（eval_number_ratio_pct_free 第三门 +
  parse_finite_number_math 前置门），钉面在库全绿：style-system
  sign_abs_scale_slot_static_acceptance（`scale: calc(sign(50%) * 180)`
  拒收钉）+ css-parser abs_pct_ratio_resolution_scale_slot（比值消解
  保护钉）。05:0x 复跑 css-parser 56P / style-system 7P / engine 1P。
- 钉面补充账：style-system 新增 animation_duration_plain_negative_
  literal_rejected（裸 `-3s` 拒收钉，animation-duration-invalid 语料）。

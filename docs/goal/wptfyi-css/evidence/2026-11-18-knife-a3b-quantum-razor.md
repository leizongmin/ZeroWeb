# 刀 A3b：transitions 通道首帧量子剃刀（2026-11-18）

## 靶面

刀 A3 残面 30F 地板真值分解（results-knife-a3/css-transitions.b1.json 逐行）：
transitions 通道 **12F** = neutral/inherit/unset × at{0.3,1.5} ×
{plain,all}（本刀认领）；CSSA/WAAPI unset 10F（端点解析面，池）；
currentColor 8F（元素色上下文面，池——fork currentColor 流同期活跃开发，
animation.rs WIP 双侧同态进本轮 A/B 对消不认领）。

## 根因（探针页三测定谳，zz-a3-probe-keyword.html）

harness 采样形（interpolation-testcommon.js :81-92 实读）：duration=100s +
delay=-50s + `createEasing(at)`=cubic-bezier(0,(8at−1)/6,1,·)（:245-258，
对称点 y(0.5)=at 代数精确）。注册→查询 δ≈0.1-3ms 使 progress=0.5+δ/100s；
bezier 中点斜率 0.5（dy/ds 0.75 / dx/ds 1.5）→ eased 偏 ~1.5e-5/3ms →
输出 .5 半边界翻转 1 单位：

- at(0.3)：r=255·(1−t)=178.5，ε 落 178.49987 → 串 `178`（期望 179）
- at(1.5)：eased=1.5±ε 走插值外推臂（eased==1.0 echo 门严格 `==` 不截），
  g=165−37t=109.5，ε 落 109.4996 → 串 `109`（期望 110）；offsets 25px/15px/
  25px 整值不触界——单通道 razor（探针 b0 got `rgb(0, 109, 0) 25px 15px
  25px` 实证）

端点解析旧假设（A3 残面池 §1「keyword 字面串 parse 失败」）被探针推翻：
inherit-resolved gcs == explicit gcs 恒等，registry gCS 预串化端点解析
正确——残面池 §1 已按 ε 面改判（master.md 池注记同步）。

## 修法（transition_registry.rs 单文件，fork 认领面零触碰）

1. **首帧量子守卫**：`FIRST_FRAME_QUANTUM_S=0.016`——css-transitions-1
   §starting-of-transitions：transition start time 定格 style change event
   所在帧界，同任务内 gCS 过渡时钟未走帧。elapsed<16ms → active=−delay
   （解析 seek 点）；真帧推进 ≥16ms 照常墙钟。
   **R2 收窄（r2 电池 P→F 定谳后改判）**：守卫仅对 bezier timing 门控
   （对称中点解析臂输入契约需 progress 精确 0.5）；linear/steps 保持墙钟
   ε 形——r2 实证「delay≥0 首帧钳 0 零行为差」前提对非 bezier 不成立：
   精确 0/0.5 在 steps(·,start) t=0 首步语义（css-easing-1
   §step-easing-functions）与输出 .5 半边界串化上翻转 oracle 按 Chrome ε 形
   采写的期望串（after-change steps 同帧读 3P→F + currentcolor linear
   at(0.5) 半边界 2P→F，见八腿电池 r2 段）。
2. **对称 bezier 中点解析臂**（entry_eased）：`t == 0.5 && x1 == 0.0 &&
   x2 == 1.0` → y(0.5)=ay·0.125+by·0.25+cy·0.5 解析求值——x(s)=3s²−2s³
   （x1=0/x2=1 族）在 s=0.5 精确过 0.5，初始猜 s=x 即精确解零迭代；消除
   共享 cubic_bezier 二分 20 轮 ±7e-7 容差噪声（eased ±5e-7 恰在 .5 半边界
   掷硬币）。仅门控 t==0.5：at=0.5 走 linear 不入此臂 → 该组合恰为
   harness 剃刀面，其余参数域共享路径零扰动（P→F 面最小化）。

数学验证（Rust f64::round=half-away-from-zero 语义）：at=0.3 →
eased=f64(0.3) 精确 → r=178.5 → 179；at=1.5 → eased=1.5 精确
（cy=fl(3·fl(11/6))=5.5）→ g=109.5 → 110、r 负外推 lerp_color 钳 0。

钉测 +4（PIN-LEG 0.00s，4 passed/0 failed/4182 filtered）：
`test_ts_harness_seek_at03_exact_string`（"rgb(179, 154, 0) 13px 27px
13px"）、`test_ts_harness_seek_at15_extrapolation_exact_string`（
"rgb(0, 110, 0) 25px 15px 25px"）——精确串断言，二分 ε 形必挂；
R2 回归锁两枚：`test_ts_after_change_steps_start_same_frame`
（steps(2,start) 同帧读 → 1/2 中值 100px——守卫钳 0 形必挂）、
`test_ts_linear_seek_epsilon_form`（linear seek ε 形串
"rgb(127, 146, 0) 15px 25px 15px"——守卫覆盖 linear 半边界形必挂）。

## 门禁

- fmt：刀面 hunks 按 rustfmt 形。
- clippy：定向 `cargo clippy -p zero-engine --lib` rc=0（两个 -A 各带域外
  法证同 A3：`too_many_arguments`=script-sandbox es_module.rs:2211 fv1 流；
  `unnecessary_cast`=animation.rs 2434/2435 提交态 2d3e71d19）。
- engine lib 全量 **4186/0**（R2 收窄形复跑，20.7s——4184 基线 + 本刀 R2
  新增 2 钉；守卫收窄对 lib 全域零 REG 实证；4184/0 先行跑系 r2 全域守卫
  形 12:14 同窗账）。
- 双建：knife-a3b-build-bins.sh patch 往返单文件换面（animation.rs 系 fork
  currentColor 流认领面，不在刀内），M1/M2 刀面 md5 往返精确还原，S1/S2/S3
  fork WIP 14 文件窗内零扰动；r2 bins b0=e8a40948…、b1=feb264c5…；
  R2 重建 bins b0=f8e3022e…、b1=a6393f34…。

## 探针/solo 定谳（b0=无刀、b1=有刀，fork WIP 双侧同态）

| 测 | b0 | b1 |
|---|---|---|
| probe at(0.3) | Fail `rgb(178, 154, 0) 13.0001px …` | **Pass** |
| probe at(1.5) harness 形 | Fail `rgb(0, 109, 0) 25px 15px 25px` | **Pass** |
| solo 插值页 | 147P/21F | 159P/9F |

solo Δ = **+12 F→P 零 P→F**（残 9F = CSSA/WAAPI unset 端点解析面，池）。
b0 较上轮 A3 电池 b1（138P/30F）多 9P = fork WIP 双侧同态贡献（currentColor
面已翻 9 行），A/B 内对消不属本刀。

## 八腿电池

**r2（12:12-12:35，守卫全域形 bins b0=e8a40948…/b1=feb264c5…，结果存档
results-knife-a3b-r2-diag）——P→F 定谳触发 R2 收窄**：

| 语料 | b0 | b1 | ΔP | F→P | P→F |
|---|---|---|---|---|---|
| css-transitions | 811P/1261F/18T | 820P/1812F/19T | +9 | +12 | **−3** |
| css-animations | 516P/780F/7T | 同 | 0 | 0 | 0 |
| css-backgrounds | 3430P/2816F | 同 | 0 | +2 | **−2** |
| css-masking | 4651P/1704F | 同 | 0 | 0 | 0 |

- transitions −3 全落 `after-change-style-inherited.html`（steps(2,start)
  delay≥0 同帧读期望 1/2 中值）：solo 双 bin ×3 稳定复现（b0 4P/2F 恒、
  b1 1P/5F 恒，探针存档 r2-diag/）——守卫钳 elapsed=0 使 steps-start
  t=0 臂返 0（旧臂 `Some(Start) => 0.0`）而非 ε 形 ceil→1/n。非窗噪，
  守卫全域形对非 bezier timing 的真 P→F。
- backgrounds −2 全落 `background-color-interpolation.html`
  `from [currentcolor] to [rgba(0,255,0,0.75)] at (0.5)`（transition/
  transition:all 双形）——linear at(0.5) 精确 t 落输出 .5 半边界，oracle
  按 Chrome ε 形采写期望串；+2 同页 `from [white] to [orange] at (0.3)`
  双形＝剃刀 bezier 家族额外收益（同 ε 面 F→P，非本刀认领口径、随刀
  白拿）。
- animations/masking 双侧 SET-IDENTICAL（页集合+子测键全等）。
- transitions async 页形摆动（properties-value-001/-inherit-002
  early-death↔unlocked 双向、only-b0 560/only-b1 1121）＝async 链既有
  非确定性（A3 轮同形注记），双侧对称不算 REG。

**R2 修复**：守卫收窄至 bezier-only（修法§1 R2 注记）+ 回归锁钉 ×2，
重建 bins（b0=f8e3022e…、b1=a6393f34…，M2/S2/S3 法证全绿）。
复验：after-change solo b0 4P/2F == b1 4P/2F（P→F 消除）；ts-interp
solo b0 147P/21F → b1 159P/9F（剃刀 +12 保持）。

**r3（R2 收窄形 bins，f8e3022e/a6393f34）**：12:47 scope 发射版在跑
leg2 时被外层 systemd-run --scope 调用方 10min 后台上限级联杀（r3
日志 knife-a3b-battery-r3-1249.log 仅 transitions b0 完整）；13:03
service 模式（--unit 分离，WorkingDirectory=/ 坑=重定向须 cd 内层）
重发 knife-a3b-battery-r3c-1303.log 全八腿。同窗并行电池：q2c（本流
CLAIM 电池续跑，见刀 Q2 段）+ 对端 A3r r2（他流刀，互不越界）。

**r3 收账（13:02:59→13:26:30 GATE-DONE BATTERY-RC=0 零 LEG-FAIL，results-knife-a3b，链
knife-a3b-battery-r3c-1304.log service 模式单写者）**：

| 语料 | b0 | b1 | ΔP | F→P | P→F |
|---|---|---|---|---|---|
| css-transitions（靶） | 811P/703F/17T | 823P/1250F/18T | **+12** | +12 | **0** |
| css-animations | 516P/780F/8T | 同 | 0 | 0 | 0 |
| css-backgrounds | 3430P/2816F | 3433P/2813F | +3 | +3 | 0 |
| css-masking | 4651P/1704F | 同 | 0 | 0 | 0 |
| **TOTAL** | 9408P | 9423P | **+15** | +15 | **0** |

- **主靶 +12/0**：全落 text-shadow-interpolation（147P→159P，与 solo 双 bin
  精确一致）；composition 页 0/0（A3 已收）；after-change 页 b0 {4P/4F} ==
  b1 {4P/4F} 恒等（r2 −3 P→F 面消除实证）；R2 收窄形双回归面（steps-start
  同帧读 + currentcolor linear 半边界）双双零 P→F。
- transitions only-b1 560＝properties-value-inherit-001 页级 unlocked 展开
  （async 链既有非确定性，全 Fail ΔP 中性，r2 同形注记维持）。
- animations/masking 双腿 SET-IDENTICAL（页集合+子测键全等）。
- backgrounds 白拿 +3 归因逐行：`background-color-interpolation.html`
  `from [white] to [orange] at (0.3)` transition/transition:all 双形 ×2
  ＝剃刀 bezier ε-face 家族（r2 段预测原样复现，守卫收窄后仍覆盖——harness
  easing 为 cubic-bezier）；`background-computed.html` bg-position-y
  calc 多层串化 ×1＝fv1 serialize 域跨跑漂移家族（A2/A3 先例第四次，
  本轮 F→P 顺向不认领）。**认领口径 +12**（transitions 主靶）。

## 残面池（不认领，挂观测）

1. **CSSA/WAAPI unset 端点解析**（插值页 9F/10F 形）：keyframe 端点字面串
   进臂解析失败全 at 点位挂（非 ε 单点位）——fork currentColor 流同域，
   待其收口后独立刀。
2. **currentColor 元素色上下文**（8F→fork WIP 已翻大部分）：CSSA/WAAPI
   keyframe 串进臂 resolve_color_current 落黑底——fork 流认领中。
3. **共享 cubic_bezier 二分容差**（全局）：非对称形（ease/ease-in-out）与
   t≠0.5 域仍 ±7e-7 容差——现账面无触界行（电池零 P→F 实证），挂观测。
4. **steps(·,start) t=0 精确边界臂**：entry_eased `Some(Start) => 0.0`
   形按 css-easing-1 jump-start 应为 eased(0)=1/n——守卫收窄后精确 t=0
   不可达（墙钟恒 ε>0），现账面无驱动；若未来 seek 精确化覆盖 steps
   timing 须先修此臂（r2 after-change 定谳即此臂被钳零激活形）。

# 刀 WA-3a：CSSTransition 对象面——plain-object 快照升真 Animation+KeyframeEffect（2026-11-20）

## 池与 face

池＝刀 WA/WA-2 落库后 css-transitions 残面清单：`AnimationEffect-getComputedTiming
.tentative` 20F（gCT twin 面）+ currentTime/startTime/effect/ready/class-string
等 CSSTransition 身份与生命周期缺口 ~12F。
face＝`part04.js` 单文件 +143/−22（`.acceptance/wptfyi-css/knife-wa3a-face.patch`，
syntax 25 片 concat node --check 过），八臂：

1. `@@toStringTag 'CSSTransition'`（assert_class_string 身份）。
2. `transitionProperty` 只读 getter。
3. effect 全套：getTiming/getComputedTiming（§4.9 全模型）/getKeyframes/
   setKeyframes/target——part03 KeyframeEffect 原型继承。
4. play-pending §5.4.1 hold time 0（currentTime 初始 0、startTime null→
   localTime 恒等 currentTime）+ pending true + ready 即决。
5. fill 恒 `'backwards'`（finish 后 progress/currentIteration null 域）。
6. easing 活读 computed transition-timing-function + `_zwIsValidEasing` 预检。
7. kebab→camel 帧键（KeyframeEffect 可动画白名单口径）。
8. cancel 摘除键 + AbortError ready 拒绝。

## 电池

- 脚本 `.acceptance/wptfyi-css/knife-wa3a-battery.sh`（承 knife-wa2 配方：
  patch -R 剥除/重放 + trap 归一 + test-guard 包裹 + M2 异 md5 + face 标记
  strings 门 + RIDE census S1/S2/S3 + 防双发）。发射链 zw-wa3a-chain
  （等 zw-wa2-chain2 全退场后串行，WA2-EXITED 07:26:54）。
- 构建门全绿：B0 BUILD 07:27:15（42.40s）/ B1 BUILD 07:27:57（31.75s）；
  b0 md5 `4f4ba346c6043b3cb8e39143c7416ee1`、b1 md5
  `e8f1206441ad2d641966d3a928e5156b`（M2 异双证）；FACE-MARKER
  `CSS transition was canceled` strings b0=0/b1=1 过。
- RIDE 15 文件（14 码面＝ffix2/optical 姊妹流 WIP 归其流 + master.md）。
  S1/S3 MISMATCH 定谳：唯一扰动＝master.md 自身（WA-2 收账 docs 提交
  `667d4af3f` 07:31:27 在构建窗内落库），14 码面 S1/S2/S3 与 live 四代
  md5 零扰动——文档不入二进制，A/B 有效性不受影响。
- 12 腿全 rc=1 正常（有败即 1）：b0 07:28:31→07:43:40、b1 07:43:40→
  08:00:34；GATE-DONE 08:00:35，TRAP-NOOP state=done（face 已在树），
  chain rc=0。会话重启风暴窗内电池 unit 自持完成，收账由存活轮执行。

## 对账（GATE-DONE 08:00:35，本轮独立逐页复算）

RECONCILE 账：**TOTAL gained=33 lost=18 regressed=18 pages_gain=8
pages_loss=4**。哨兵 fonts/pva/typed-om/shapes 逐页零差异（reconcile
无输出行）。

### 窗噪定谳（−16 不入账）

- `css-animations/Element-getAnimations.tentative` b0 16P→b1 1P
  （REGRESSED=15）：静默探针（保存 bin 直跑，test-guard 包裹）
  **b0=1P/19F 与 b1=1P/19F 全等**；跨电池史 wa2-b0/wa2-b1/wa3a-b0 三腿
  16P、wa3a-b1 1P、solo-r1 b1 1P——16P 仅存在于电池负载窗，两二进制
  静默态恒等。b0 侧 16P 为窗口幸运值，非 face 回归。
- `css-animations/event-dispatch.tentative` b0 1P→b1 0P：wa2 双腿 0P、
  solo-r1 b1 0P——b0 的 1P 同为窗口幸运值；页 N 数 18→11 摆动承已知
  时间线敏感页 flake 口径。
- 域归负载不对称（b0 腿 07:36-07:43 静、b1 腿 07:43-08:00 与收账窗
  会话活动重叠），不入账。

### 真回归定谳（−2 挂账残面）

- `css-transitions/CSSTransition-effect.tentative` 5P→4P：静默探针
  b0=5P/b1=4P 与电池恒——「After setting a transition's effect to null,
  it should be possible to interrupt」b1 `assert_not_equals: got
  disallowed value 0`（effect=null 真赋值摘除 effect 后 interrupt 路径
  返回 0）。face 域内新路径缺陷。
- `css-transitions/Document-getAnimations.tentative` 2P→1P：静默探针
  b0=2P/b1=1P+页级 Timeout——「Transitions are not returned after they
  have finished」在 b1 页尾截断（finished 等待不决，页墙钟截断）。
  升级后 CSSTransition finished 生命周期缺口。
- 同子测「A transition with a replaced effect…」双侧均 F 非翻转
  （b0 `getTiming is not a function` → b1 `expected 50000 got 100000`
  失败模式推进，effect.getTiming 已生效）。

### 增益定谳（+33 全稳）

- `AnimationEffect-getComputedTiming.tentative`（transitions）2→22
  （+20）：静默探针 b1 22P 恒。
- KeyframeEffect-getKeyframes 0→4、getKeyframes-width-and-height 0→2、
  setKeyframes 0→2、target 0→1：静默探针 b1 4P/2P 恒（filter 双页
  同跑）。
- CSSTransition-startTime 0→2：静默探针 2P 恒。
- CSSTransition-finished 0→1、Element-getAnimations（transitions）4→5。
- 全部为 face 新 API 面页（b0 0P＝API 不存在），机制承载恒定。

**净账 +31**（gained 33 − 真回归 2；窗噪 −16 不入账）。

## 收账与落库

face 落库（part04.js 单文件 +143/−22，pre-commit guard PASS；shim 25 片
concat node --check 于发射前过，b1 二进制 FACE-MARKER 实证）。本 evidence
与 master.md 收账笔随 docs 提交落库。挂账残面（次刀候选）：
①effect=null setter 后 interrupt 路径（CSSTransition-effect 残 6F）；
②CSSTransition finished 生命周期（finished promise/finished 后摘除，
Document-getAnimations 页尾 hang + CSSTransition-finished 残面）；
③properties-value-* 插值大池 ~820F（transition property 插值机制域，
独立刀）。

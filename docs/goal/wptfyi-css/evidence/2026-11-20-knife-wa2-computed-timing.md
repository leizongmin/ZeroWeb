# 刀 WA-2：getComputedTiming/updateTiming 残面——timing model 全集（2026-11-20）

## 池与 face

池＝刀 WA 落库（`ea98cb8ae`）后 evidence 残面清单首项：AnimationEffect
getComputedTiming 残 22F + updateTiming 残 4F（css-animations，全集/迭代缺口）。
face＝`part03.js` 单文件 +110/−35，八臂（26 个失败断言逐条走查新模型全对齐，
含 shorthand 二次时间值 delay 形解析勘误——`0s 5s 10.25 both`＝duration 0s +
delay 5s + iterations 10.25，与 `100s 5.5 forwards`＝iterations 5.5 两形区分）：

1. effect 级 easing 恒 `'linear'`（css-animations：animation-timing-function
   只作用于关键帧间——keyframe easing 由 `_zwCssAnimFramesFor(elemEasing)` 承载，
   effect 级不取元素计算值；getComputedTiming().easing 初始断言）。
2. fresh play-pending currentTime 0（web-animations §5.4.1 hold time 0）：
   同步咨询点（startTime 未 resolve 分支）+ ready 微任务 defer 两点，
   `localTime of a new animation` 断言在 defer 前同步执行。
3. active duration 零律（§4.5.2：duration/iterations 任一为零 → 0——
   0 × Infinity = 0 非 NaN，零时长无限迭代 endTime/activeDuration）。
4. endTime = `max(delay + activeDuration + endDelay, 0)`（负 delay 超
   active duration 时钳 0——「finishes before its startTime」）。
5. localTime 毫秒口径：CSS 动画槽位 startTime 为 timeline 秒、currentTime 为
   毫秒（shim 约定），`anim._zwTlSec` 标记后 localTime 换算 ×1000；构造
   Animation 不带标记、单位原样（M3-S2 既有面零扰动）。
6. overall progress 统一模型（§4.9.3/§4.9.4）：before→iterationStart /
   active→activeTime/duration + iterationStart / after→iterationStart +
   iterations（零时长无 active 相位，activeDuration 0 时 localTime ≥ delay 即
   after）；整数 overall progress 迭代进度 1 边界；after 相位 currentIteration
   = ceil(iterationStart + iterations) − 1（无限 → Infinity，iterations 0 →
   floor(iterationStart) 保 zero-iterations 页），active 相位 = floor(overall)；
   方向 still reverse/alternate/alternate-reverse 按迭代索引奇偶。
7. slot finish() seek 活跃期末（`currentTime = delay + activeDuration`，
   非无限迭代——非 integral 迭代数 after 相位 ceil 律依赖；
   `_finishNow` 原只 seek 到 duration，5.5 迭代 finish 后仍 active 相位）。
8. updateTiming 逐字段 override（css-animations-2）：API 显式设置字段落
   `_timing._zwOvr`，样式刷新（身份保持分支 + `anim._zwTimingRefresh` 咨询点
   回读 animation-duration/delay/fill-mode/iteration-count/direction 五通道）
   跳过已覆盖字段；错误抛出在全部赋值前（上抛调用零字段落 _zwOvr，
   `updateTiming()` with error 页）。getTiming/getComputedTiming 咨询点挂
   回读钩子——updateTiming 页样式改动后无 getAnimations 重询，控制断言
   （delay/fill 由样式流入）依赖此通道；easing 恒 linear 不同步（本 face 臂 1）。

## 电池

- 脚本 `.acceptance/wptfyi-css/knife-wa2-battery.sh`（承 knife-wa-battery.sh
  配方：patch -R 剥除/重放 + trap 归一 + test-guard 包裹 + M2 异 md5 +
  face 标记 strings 门 `_zwTimingRefresh` b1>0/b0=0 + RIDE census S1/S2/S3 +
  三层防双发）。发射链 zw-wa2-chain2（等 zw-wa-r4 与 zw-sv2-battery3 全退场后
  串行——首发射 zw-wa2-chain 05:34:18 被防双发门自匹配误拒：pattern
  `zw-wa2` 撞父 unit 名 `zw-wa2-chain`，pattern 收窄 `-battery` 后重发）。
- 构建门全绿：RIDE 16 文件 S1/S3 零扰动、face 标记 b0=0/b1=5、M2 异 md5。
- 账＝css-animations b1−b0 主靶（getComputedTiming 22F + updateTiming 4F）；
  哨兵＝transitions（CSSTransition 对象面另刀域）+ pva/typed-om/shapes 零差异
  + fonts ride parity。

## 对账（GATE-DONE 06:07:09，chain2 fork 收账笔由续开轮代录）

- zw-wa2-chain2 全链 rc=0：B0 BUILD 05:35:17（43.84s）/ B1 BUILD 05:36:01
  （34.20s）；b0 md5 `fee99384062f56949b7591cf732a5aa6`、b1 md5
  `ed9722f0040a885d2c300b5f1448bc3a`（M2 异双证）；FACE-MARKER
  `_zwTimingRefresh` b0=0 / b1=5 过；RIDE 16 文件 S1/S3 md5 census 零扰动。
  b0 与刀 WA canonical r4-b1 逐字节同源（同 md5）＝基线交叉实证。
- RECONCILE 账：**css-animations 主靶 gained=25 lost=0 regressed=0
  pages_gain=2**——getComputedTiming 21→42（+21）、updateTiming 1→5（+4，
  页全绿）。续开轮按 b0/b1 JSON 独立逐页复算，与电池 reconcile 逐项恒等。
- 哨兵：transitions/fonts/pva/typed-om/shapes 五语料逐页 P 零差异零 P2F
  （TOTAL gained=25 全归 animations 两页，无外溢）。
- 残面：getComputedTiming 残 1F＝`localTime reflects playbackRate
  immediately`（playbackRate 通道，timing-model 八臂域外，后续刀候选）；
  updateTiming 全绿。计划 26F 中 25F 闭合，1F 经走查属 playbackRate 面
  非计时模型缺口。
- 窗噪注记：5 页 0P 页 N 数窗内摆动（display-none-dont-cancel 5→4、
  animations event-dispatch 11→12、event-order 4→1、transitions
  event-dispatch 30→15、properties-value-inherit-001 1→561）＝执行窗
  截断伪影，P 增量全零，不入账（承已知电池窗负载 flake 口径）。
- 会话中断法证：上轮 session 06:01 rate-limit 终止于腿期（bin 已冻结
  05:36），GATE-DONE 06:07:09 由电池自持完成；part04.js 05:55 WIP（WA-3a
  face）不在 b0/b1 二进制（构建先于编辑，RESULTS-NOT-FRESH 门 + bin 冻结
  纪律核过）；TRAP-NOOP state=done（face 已在树）。

## r4 独立确认（前刀 WA canonical 交叉核对）

r4（zw-wa-r4，全量重跑）05:33:39 GATE-DONE，全 12 腿对账
`TOTAL: gained=73 lost=0 regressed=1 pages_gain=17`，逐页表与 killed-0447
canonical（刀 WA evidence）逐字节一致——含唯一回归 getAnimations finished
relevance、getComputedTiming +19 (2→21)——±0 flake，独立确认窗闭合，
canonical 不动（详见 NOTES-knife-waapi-fork-roles.txt 05:3x）。

## 收账与落库

face 落库 `60ea84097`（part03.js 单文件 +110/−35，pre-commit guard PASS 零发现；
提交基＝ff origin/main `ad9f5f579` 后新树——peer M 域 part01/03/04/05 shim 涉面
4 commit autostash 无冲突，face 标记 `_zwTimingRefresh`×5 与语法 25 片 concat
node --check 复验过）。本 evidence 与 master.md 收账笔随 docs 提交落库。

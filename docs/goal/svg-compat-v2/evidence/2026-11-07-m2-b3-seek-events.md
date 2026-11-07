# M2/B3 — seek 事件语义（2026-11-07）

B3 一件事：SMIL seek 事件转移模型（seeking-events-1..8 定谳矩阵驱动）+
处理器 IDL 直调 + first-interval 剪枝，连带修正采样 active 分支。全通道 +
web-animations 双通道零回归复验落档。

## 1. 实现落点（`crates/engine/src/js_dom_shim/part09.js`，+138/−66）

- **转移扫描 `_scanDocEvents(doc, t, isSeek)`**：每动画元素比较上一观测态
  `{ib, began, ended, seen}` 与 t 时刻 active interval（b ≤ t < e），差分
  派发。语义矩阵（seeking-events-1..8 定谳）：
  - seek（isSeek）越过的 interval（首见即整体在过去）零事件；
  - 曾 began 的 interval 失活（自然 end / seek 出界 / seek 回退 / 被新
    instance 替代）→ endEvent；
  - seek 落点所在 interval（含回退重入已 ended interval）→ beginEvent；
  - 同 interval 内 seek（跨 repeat 边界同）不派 repeatEvent（repeatEvent
    自然播放面 = B5 FIXME）。
- **seek 扫描 task 粒度 `_scheduleSeekScan`**：setCurrentTime 挂
  setTimeout(0) 一次性扫描（`seekTimer` 合并窗口，执行时读最新 frozenDoc
  幂等）；paused 也派（WPT pause+seek 是唯一时间驱动源）。同步派发会早于
  seek 后才挂的 EventWatcher（seeking-events-1 watcher 时序定谳）。
- **处理器 IDL 直调 `_smilHandlerNames`**：onbegin/onend/onrepeat ↔
  beginEvent/endEvent/repeatEvent 错位映射（SVG2 animate.html；onendEvent
  非处理器名恒不触发——event-listeners 定谳）。内容属性 inline 编译经
  proxy get trap 的 `_ensureInlineHandler` 惰性编译后同路；addEventListener
  面走 dispatchEvent。
- **first-interval 剪枝**：freeze 采样链修剪 `e0 > 0`（begin="-1s" dur=1s
  的 [-1,0] 非 real interval 不贡献冻结值——dont-contribute 定谳方向）。
- **采样 `_activeInterval`**：`_sampleAnim` 从 `_currentInterval` 改
  `_activeInterval`——pending（b > t）不再采样，负 frac 垃圾值消失（6 个
  F→P 连带收益的根因）。
- **pre-load seek 落 frozenDoc**：`_currentTimeOf` 未 started 分支从恒 0
  改读 `frozenDoc`（初始 0；pauseAnimations()+setCurrentTime() pre-load
  驱动面与 getStartTime 案 `getCurrentTime()===0` 断言两满足）。
- `beginElement*` 移除手动差分状态重置——下轮转移扫描按新 instance 集派
  endEvent（旧 interval 被替代）+ beginEvent（restart=always）。

## 2. 验证（test-guard 4G/8G/2700s 包裹，`--json` 落档本目录）

### 2.1 svg 全通道（`2026-11-07-m2-b3-svg-channel.json`）

- **486 案 1,709 子：1,223P / 467F / 19T = 71.6%**（M1 1,708 子
  1,209P / 468F / 31T：**+14P / −1F / −12T**）。
- **零 P→F**（逐子 flip，匹配名集合仅 6 个 F→P；raw 对账脚本核）。
- 变化面收敛在两个目录：
  - **animations +14P/−3F/−11T**（75/213/23 → 89/210/12）：①
    seeking-events-1..8 全部 T→P（seek 转移模型 + 处理器直调兑现）；②
    F→P ×6 = multiple-animations-fill-freeze +
    svglength-additive-by-3/4/6 + svglength-additive-from-by-2/4
    （`_activeInterval` pending 采样修正连带收益）；③ T→F ×3 =
    first-interval-in-the-past-contribute/dont-contribute（getComputedStyle
    fill 空串/color 插值 = 挂账 #4 style-system 协同域门控）+
    animate-currentcolor-in-visited-link（渲染交互挂账面）——案从挂起转
    完成后真实断言显形，非 Pass 丢失。
  - **linking +2F/−1T**：href-mpath-element 1T（占位）→ 2F（真实子显形，
    mpath href 移除语义 = getCTM/路径引用挂账 #2 邻接域）。
- 其余目录（types/path/geometry/interact/text/struct 等）逐子恒等。

### 2.2 web-animations/timing 依赖通道（划界约束）

`testharness-web-animations --json`（同 guard 口径，raw 捕获本目录
`2026-11-07-m2-b3-web-animations.json`）：**506P/444F/9T = 959 子，对 M1
逐子恒等零 flip**——文档级 SMIL 时间线改动未波及 WAAPI/CSS 动画泵。

## 3. B3 批次记账（DC-3 FIXED/REGRESSED）

- **FIXED +14P**：seeking-events-1..8（8）+ multiple-animations-fill-freeze
  + svglength-additive-by-3/4/6 + svglength-additive-from-by-2/4（6）。
- **REGRESSED 0**。
- **观察记账（T→F 显形，非回归）**：first-interval ×2（挂账 #4 门）+
  animate-currentcolor-in-visited-link（渲染交互）+ href-mpath-element
  2 子（挂账 #2 邻接）——三者均入 DC-4 前协同清偿/具名转挂清单。
- interval-restart-events（P）在新转移模型下验证兼容（begin==end 双计）。

## 4. 下一步（M2 续）

B4 syncbase（id.begin/id.end 依赖解析 + cyclic 环，M0 账 3F/3T）
→ B5 事件基（eventbase begin="click" 实例源 + repeatEvent 自然播放面 +
条件处理门，M0 账 6F/7T）→ B6 采样引擎（calcMode discrete/spline、
additive/accumulate、keyTimes/keySplines，M0 账 124F/3T——SMIL 池最大簇，
本批 `_activeInterval` 连带收敛 6F 后现势约 118F）。

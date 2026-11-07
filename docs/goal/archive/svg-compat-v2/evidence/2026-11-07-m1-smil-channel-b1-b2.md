# M1 — SMIL 通道落地 + B1 时钟值文法 + B2 interval 首批（2026-11-07）

M1 三件事：①part09.js 文档级 SMIL 时间线引擎落地（B1/B2 语义 +
SVGSVGElement 时间线控制 + SVGAnimationElement 真值接口 + animVal 反射）；
②全通道零回归复验 + web-animations/timing 依赖通道复验；③DC-2 通道定型
决定落档。

## 1. 实现落点

- `crates/engine/src/js_dom_shim/part09.js`（新建，625 行）：SMIL 文档级
  时间线引擎。能力面 = clock-value 严格文法（Full/Partial 分秒两位且 <60、
  分数点后 ≥1 位、Timecount 缺省 s）+ begin/end offset 列表 + interval
  解析（首个 end > t 为 current；end = min(end 实例, begin + activeDur)，
  repeatCount/repeatDur 链）+ linear from/to/by/values 采样 + fill=freeze
  （activeDur 非 dur 整数倍取 mod）+ SVGSVGElement
  pause/unpause/getCurrentTime/setCurrentTime 文档级共享时间线（覆写
  part03 per-element stub；currentTranslate/currentScale/deselectAll 留
  part03）+ SVGAnimationElement getStartTime/getSimpleDuration 真值 +
  beginElement/beginElementAt 动态实例（restart 门控）+ begin/end 事件泵
  （task 粒度 setTimeout 轮询 ≤50ms，learnings 禁 microtask；paused/未
  started 不轮询）。
- `part03.js` 两处挂钩：SVGLength 只读面 `cur()` consult
  `globalThis._zwSmilAnimValue`（返回动画态 {ut, vu}）；SVGAnimatedNumber
  animVal getter consult `_zwSmilAnimNumber`。baseVal 面不受影响（动画不
  写 DOM 属性——animVal-basics 定谳）。
- `part06.js` window load 派发点挂 `_zwSmilDocStart`：文档时间线起点 =
  load；pre-load 读恒 0、pre-load seek 落 frozenDoc、start 时结转
  （getStartTime 案的 pre-load pause 断言定谳面）。
- `js_dom_bridge.rs` 注册 part09 加载序（part08d 之后）。
- 划界兑现：SMIL 时钟不进 compositor 动画泵——文档级时间线 + seek 驱动
  采样（WPT 案主消费面 pauseAnimations()+setCurrentTime(T)+assert）；
  CSS/WAAPI 动画泵零接触（web-animations 通道复验零回归为证）。

## 2. 运行前静态审查修正：_allBegins 缺省语义

初版 `_allBegins` 对 begin 属性**存在但零实例**（`begin="indefinite"` /
`begin="foo.begin"` 未接实例源形态）也回落 `[0]` 缺省——会把
`SVGAnimationElement-exceptions` 的 getStartTime（begin="foo.begin" →
无 current interval → InvalidStateError，M0 stub 恒抛、现势 P）翻成
返回 0 的 **P→F 回归**。修正：`[0]` 缺省仅限 begin 属性缺省/空白（spec
#Timing-DefaultBegin）；属性在但零实例保持无实例（元素不自动开始）。
修正后 exceptions 2P 保持、getStartTime 案 interval 切换断言全 P。

## 3. 验证

### 3.1 svg 全通道（test-guard 4G/8G/2700s 包裹，`--json` 落档本目录
`2026-11-07-m1-svg-channel.json`）

- **486 案 1,708 子：1,209P / 468F / 31T = 70.8%**（M0 基线 1,704 子
  1,186P / 478F / 40T：**+23P / −10F / −9T**）。
- 分目录（M0 → 本轮）：types 641P/41F → 641P/41F 恒等；path 169P/8F →
  170P/7F（+1P）；interact 105P/30F 恒等；text 5P/53F 恒等；
  **animations 52P/224F/32T → 75P/213F/23T（+23P/−11F/−9T）**；其余小面
  182P/70F/8T → 183P/70F/7T（+1P/−1T）。

### 3.2 geometry −2P 归因（非本批改动）

全通道对 M0 快照唯一下滑面 = geometry 32P/52F → 30P/54F。A/B：stash
part09（bridge 注册同 stash，二进制回 pre-part09 语义态）后同语料跑
geometry 面 = **30P/54F，与 part09 在场逐子 0 flip**（基线 raw 捕获本目录
`2026-11-07-m1-geometry-ab-baseline.json`）——下滑在 M0 快照
（83cef074a）与 HEAD（042832d8c）之间的 main 演化已发生，嫌疑
4a5fd3051（声明式物化 sr 字段，engine 域，shadow-dom 流）。svg/geometry
= 挂账 #2/#4 协同域：该 2 子下滑并入碰头归口清单（DC-4 前随 display:none
gBCR 对一并清偿或具名转挂），本 goal 不双改。

### 3.3 批次推进记账

- **B1 时钟值文法：收敛（7F → 0）**——`smil-clock-value-parsing` 18 子
  全 P（合法矩阵动画到达 + frozen 冻结值 + 非法矩阵 animVal 恒 0）。
- **B2 interval 模型首批：定谳面 P**——getStartTime 1P（gap 期取下一
  interval、fill=freeze 不延长 current、begin="1s; 3s" 在 t=4 后恒
  InvalidStateError）+ exceptions 2P 保持 + animate-insert-begin/no-begin
  （LC-Start 插入语义）+ interval-restart-events（interval 迁移差分事件）
  + begin-attribute-mutation / repeatcount-attribute-mutation（轮询感知
  属性变更）+ animVal-basics（baseVal 恒 base）+ fill-freeze 家族
  （change-css-property-while-animating-fill-freeze/remove）。
- **B2 残余（M2 续）**：first-interval-in-the-past contribute/dont-contribute
  2T（帧驱动断言面）；pruning-first-interval 1F——失败消息 rgb 空串 =
  color 插值 getComputedStyle 协同挂账簇（非 interval 语义）；interval
  语义自身已按首轮口径落地。
- **B3 seek 部分落地（超出首批的最小面）**：setCurrentTime/pause 真语义
  + seek 不重放事件（seeking-events-1..8 保持 T，seek 事件语义 = M2
  FIXME）；时钟操作共享 6F 随 B2 定谳面收敛。
- **T→F 观察记账（非回归）**：conditional-processing-02 1 子 M0 Timeout
  → 本轮 Fail——事件泵真正派发 beginEvent 后，requiredFeatures 条件处理
  门缺失显形（assert_unreached）。条件处理门 = B5 邻接面，挂 M2 批次清
  单；无 Pass 丢失。
- 不入门簇现势不变：getCTM 载体 31F（beginelement-instance-time-1 失败
  消息 `getCTM is not a function` 同域）、color 插值 13F（pruning 失败
  消息同域）。

### 3.4 web-animations/timing 依赖通道复验（划界约束）

`testharness-web-animations --json`（同 guard 口径，raw 捕获本目录
`2026-11-07-m1-web-animations.json`）：**506P/444F/9T =
959 子**——对照 goal-10 收口后漂移核对（2026-10-11）同分母 502P/959，
分子 +4、零回归；文档级 SMIL 时间线未波及 WAAPI/CSS 动画泵。

## 4. DC-2 通道定型（M0 遗留决定落档）

- **决定：svg 通道承载，不设独立 runner 子命令**——`animations/` 无目录
  级排除是一期既定（testharness_svg.rs 模块头），SMIL 面已随 svg 通道
  常驻；skip 规则 `svg_case_skipped`（.https./无 harness/iframe/
  window.open）与 Makefile target `testharness-svg`（test-guard 包裹）
  均在。独立子命令只会复刻同一底座，无增量。
- 分类账基线复核：M0 evidence §3（186F + 33T 折算）为底稿，本 evidence
  §3.3 为 B1/B2 批后现势；后续批次沿此账本逐批更新。

## 5. 下一步（M2）

B3 seek 事件语义（seeking-events-1..8，8T）→ B4 syncbase（依赖解析 +
cyclic 环 + escaped dots，3F/3T）→ B5 事件基（beginEvent/endEvent/
repeatEvent 派发语义 + eventbase begin + 条件处理门新露头面，6F/7T）→
B6 采样引擎（calcMode discrete/spline、additive/accumulate、keyTimes/
keySplines，124F/3T——SMIL 池最大簇）。

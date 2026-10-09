# 动画时间轴统一 RFC（WAAPI 真采样 × getAnimations 曝光 × animation-composition）

- 日期：2026-10-09（R5020 立项评估轮产出；依据 R5019 动画带分诊）
- 状态：**草案（待用户立项拍板）**——本文档只做设计与评估，不授权实施
- 域：`crates/engine/src/animation.rs` + `crates/engine/src/js_dom_shim`（part01/part03/part04/part06）+ `crates/style-system`（animation-composition 声明面）
- 收口带：上游 animation reftest 现存红 ≈ 13–14 案（transform-interpolation-* 8、transform-box 2、transform-additive-animation 1、transform-non-invertible 1、background-color-animation will-change-contents 1）+ testharness web-animations corpus 的时序断言面（现状大量 Timeout/Fail，见 timing-animation-compat goal DC-3 记录）

## 0. 执行摘要

**问题**：ZW 的动画能力分裂在三套互不相通的时间机制上——

| 机制 | 位置 | 时间语义 | 消费者 |
|---|---|---|---|
| CSS 动画（AnimationClock） | `engine/animation.rs` + `pipeline` | reftest 路径 = R3882 **静态 t=0 seek**（clone 时钟求值后丢弃）；webview 泵 = R342 真实墙钟 | 渲染管线、animationstart/iteration/end 事件 |
| WAAPI（el.animate） | `js_dom_shim` part03 `Animation`/`KeyframeEffect` | R2965+M3-S1 **瞬间完成**：play() → running → microtask 后 finished + 末关键帧写 inline style。无时间轴，currentTime 是 held 值 | JS（动画库 feature-detect、getAnimations） |
| `document.timeline` | shim part03 `DocumentTimeline` | 真实 `_perfNow()`（唯一在走的时间） | rAF 时间戳、hr-time |

**后果**（R5019 分诊实证）：上游动画 reftest 的主流写法是「`el.animate()` + 负 delay 冻结在中点（cubic-bezier 零斜率）+ `anim.ready.then(takeScreenshot)`」。ZW 里 WAAPI 不进渲染管线采样、瞬间完成把 test 页推到端点值，而 ref 页（静态色块）是中点值 → 结构性恒红；依赖 `document.getAnimations()` 的用例拿不到 CSS 动画 → TypeError → 超时兜底截图。

**推荐方案**（选项 A，详见 §3/§4）：**WAAPI 效果桥入 AnimationClock**——`el.animate()` 经新宿主桥把 keyframes+timing 注册进 Rust 时钟（AnimationState 已是 per-animation 克隆、天然支持程序化关键帧），渲染路径（静态 t=0 与墙钟泵）统一采样；JS `Animation` 对象降级为时钟状态的薄代理（currentTime/pause/seek 转发宿主）；`getAnimations()` 合并曝光 CSS+WAAPI 两源（identity 缓存保对象同一性）；`animation-composition` 作为独立切片补声明面与组合插值。

**切片**：S1 桥接（8 案）→ S2 getAnimations 曝光（≈4 案）→ S3 composition（1 案 + WAAPI composite）→ S4 泵路径语义完备（testharness 面）。每片独立可验证、可回滚。

**第一步**：用户拍板立项后，S1 以 `__zw_animation_start` 宿主桥 + `AnimationState.source` 扩展起步。

## 1. 背景与现状事实（代码勘察）

### 1.1 三条渲染路径的动画采样

1. **reftest 静态路径**（`pipeline::render_html`，R3882）：检测到 `animation-name` 时注册 keyframes 后 **clone 时钟、t=0 求值一次、丢弃**。负 delay 即 seek（`animation: X 1000000s cubic-bezier(0,1,1,0) -500000s` 在 t=0 恰为 50% 进度）——R5019 `background-color-animation-half-opaque` 修复后经此路径转绿。**harness 只截一次图**（shim `takeScreenshot` 是 no-op，"harness 在 load 后统一截图"），所以静态 t=0 是 reftest 的全部事实。
2. **webview 泵路径**（`webview::pump_animation_clock` → `pipeline::tick_animation_clock(current_time)`，R342）：真实墙钟，服务 testharness 探测环（动画事件派发链 R3249/R3250/R3251）。
3. **shim WAAPI**（part03）：`play()` 置 running，`_defer` microtask 后 `_finishNow()`（finished Promise + onfinish + fill forwards/both 时末关键帧写 inline style）。**不经过 AnimationClock**，静态渲染只看得到 inline style 里的端点值。

### 1.2 getAnimations 现状

- `document.getAnimations()`（part06）与 `el.getAnimations()`（part04）只读 JS 侧 `_elementAnimations` 注册表（WAAPI 瞬间完成产物，finished 含、idle 排除）。
- CSS 动画（Rust AnimationClock）对 JS **不可见**：依赖 `document.getAnimations()[0].ready.then(...)` 或 `anim.currentTime` 轮询的用例走 TypeError → reftest-wait 超时兜底截图。
- 对象同一性（spec：两次 getAnimations 返回同一 Animation 实例）现仅 JS 注册表自然满足；统一曝光后需要 identity 缓存。

### 1.3 animation-composition 现状

- `animation-composition`（css-animations-2 #animation-composition）在 css-parser/style-system/engine **零支持**（声明丢弃 → replace 语义）。`KeyframeEffect.composite` 有接口面（shim 存值）但插值不消费。
- `transform-additive-animation`（6.25%）：`animation-composition: add` 要求最终 transform = underlying（元素 computed transform）⊕ 关键帧值（add = 列表拼接语义）。R5019 probe 另发现该案 tick-截图时序敏感形态——在确定性 t=0 静态路径下该敏感性消失（单次求值、无帧间耦合），S3 落点成立。

### 1.4 已有可复用资产

- `AnimationState.keyframes` 本就 per-animation 克隆（R5018 underlying value 修复的前提）——程序化关键帧（WAAPI）与注册表关键帧（CSS）可同一承载。
- 插值与 timing function 求值（`apply_timing_function`、`interpolate_between_with_timing`、预乘 alpha lerp R5019）全在 animation.rs，WAAPI 桥入即复用。
- `start_animation_with_underlying`（R5018）：start 时点以 computed 基值填 implicit 边界——WAAPI keyframes 的 implicit 边界同语义同路径。
- shim `KeyframeEffect` 已完成 keyframes/ timing 解析与 `getComputedTiming` 面（M3-S1 ~690 subtests 接口面），桥接不动它，只换效果落地端。

## 2. 需求（Spec）

**FR-1（WAAPI 效果进渲染管线）**：当页面脚本调用 `el.animate(keyframes, options)` 时，该动画的关键帧与 timing（delay/duration/fill/iterations/easing/direction）必须进入渲染管线的动画采样，使静态渲染（t=0 seek）与墙钟泵均能呈现插值中间态——而非仅瞬间完成的端点 inline style。
- 验收：`transform-interpolation-{translate,rotate,scale,skew,matrix,perspective,...}` 族 test 页（负 delay 冻结 50%）渲染出中点插值值，与 ref 页像素一致（reftest 判绿）。

**FR-2（currentTime seek / pause 语义）**：脚本在截图前对 `anim.currentTime = <t>` 赋值或 `anim.pause()` 时，后续渲染采样必须呈现该 hold time 的插值态。
- 验收：`transform-box{,-will-change-transform-layer}`（pause + currentTime seek 后截图）渲染 seek 目标态。

**FR-3（getAnimations 统一曝光）**：`document.getAnimations()` / `el.getAnimations()` 必须同时返回 CSS 动画（AnimationClock 活跃动画）与 WAAPI 动画；同一动画在多次调用间返回同一 JS 对象（identity）；`ready` 为 resolved-or-resolving Promise，`currentTime`/`playState` 反映时钟态。
- 验收：`background-color-animation-will-change-contents` 类用例（`getAnimations()[0].ready.then(takeScreenshot)`）完成等待并正常截图；对象同一性断言不回归（testharness getAnimations 族现绿面保持）。

**FR-4（animation-composition）**：CSS `animation-composition: replace|add|accumulate` 声明进入 ComputedStyle 并参与插值合成；`KeyframeEffect` 的 `composite` 同语义消费。
- 验收：`transform-additive-animation`（add = transform 列表拼接）渲染与 chromium 一致。

**FR-5（不回归）**：既有绿面不动——瞬间完成语义的消费方（动画库 feature-detect、finished/commitStyles 断言、M3-S1 已绿 ~690 subtests、R2965 末态 inline style 行为）在无时间轴依赖的用例上保持等价输出；`make test` 锚漂移仅允许本流新增测试腿。

**约束与假设**：
- A1（假设）：reftest harness 维持「单次静态渲染 + 截图」模型（R3882），本设计不引入多帧 reftest 渲染循环。
- A2（假设）：墙钟泵（R342）语义保持「真实时间」不变；S4 的 seek/pause/rate 只服务 testharness 探测环，不进 reftest 路径。
- A3（实现来源）：插值/timing/事件全复用 `engine/animation.rs` 既有实现，不引第三方 crate；宿主桥走 js_dom_shim 既有 `__zw_*` 回调机制（同 `__zw_set_style_handle` 形态）。
- TBD-1：CSS 动画曝光的 JS 包装对象上，`animationName` 之外的属性（effect/timeline）返回粒度——S2 实施时按 corpus 消费面定（最小面：playState/currentTime/ready/finished/id）。
- TBD-2：`finished` Promise 在静态路径的派发时点（现状 microtask 近似是否保留）——S1 实施时以 testharness 现绿断言为回归门定。

## 3. 设计选项

| 选项 | 思路 | 优 | 劣 | 判定 |
|---|---|---|---|---|
| **A. 桥入 AnimationClock（推荐）** | el.animate 经宿主桥注册进 Rust 时钟；JS Animation 变薄代理 | 插值/timing/事件全复用；CSS 与 WAAPI 同管线采样（FR-1/3 天然统一）；静态 t=0 与墙钟泵两路径一致 | 需宿主桥 + AnimationState 扩展 + JS 状态同步 | ✅ 复用最大、语义单源 |
| B. 纯 JS 虚拟时间轴 | WAAPI 留在 JS，截图前按虚拟时间把采样值写 inline style | 不动 Rust | 插值逻辑 JS 重复实现（timing function/预乘 alpha/transform 插值全要抄）；与 CSS 动画双源漂移；getAnimations 仍需桥 | ❌ 长期双维护 |
| C. WAAPI 整体下沉 Rust | Web Animations 状态机在 engine 新模块，shim 只留绑定 | 架构最干净 | 工程量最大（M3-S1 的 JS 接口面迁移）；testharness 现绿面迁移风险；多会话起步重 | ⏸ 远期方向，非本立项 |

## 4. 详细设计（选项 A）

### 4.1 宿主桥契约（S1）

```
// shim → host（新回调，镜像 __zw_set_style_handle 形态）
__zw_animation_start(handle, animId, keyframesJson, optionsJson)
  // engine: 构造 AnimationConfig + per-animation keyframes，注册进 AnimationClock
  //         （source=Waapi；options 含 delay/duration/fill/iterations/easing/direction/composite）
__zw_animation_command(handle, animId, cmd)   // "pause" | "play" | "cancel" | "seek:<t>"
  // engine: 对应 AnimationState 生命状态/hold time 更新
__zw_animation_query(animId) -> stateJson     // currentTime/playState（JS getter 转发用）
```

- `animId` 由 shim 生成（自增串），JS `Animation` 实例持有；桥回调不可用时（旧宿主/polyfill）回落现行瞬间完成路径——**渐进增强，不破坏现绿面**。
- engine 侧 `AnimationState` 扩展字段：`source: AnimationSource`（Css|Waapi）、`hold_time: Option<f64>`（pause/seek 后非 None，静态 t=0 与泵均以 hold_time 优先采样）、`composite: CompositeOp`（S3）。
- 静态 t=0 采样（R3882 clone 路径）自动覆盖 WAAPI 动画：clone 发生在脚本执行后的渲染点，注册已在 `__zw_animation_start` 落时钟。负 delay seek 语义与 CSS 动画一致（`local = t − delay`）。

### 4.2 JS Animation 代理（S1）

- `play()/pause()/cancel()/finish()/currentTime setter` → 转发宿主命令；本地状态机保留（promise resolve 时点不变，microtask 近似维持至 TBD-2 裁定）。
- 末态 inline style 持久化（fill forwards/both，R2965）改由时钟采样路径承担渲染呈现；inline style 写入保留（动画库兼容）但在有桥回调时延后到 finish 事件（消除「端点值抢先于插值采样」的现病）。

### 4.3 getAnimations 统一（S2）

- engine 新查询回调 `__zw_active_animations() -> [{elementKey, name, currentTime, playState, source}]`（复用 `active_element_ids` + per-element 状态）。
- shim `getAnimations`：JS 注册表（WAAPI）∪ 时钟动画（懒创建包装对象；identity 缓存键 `(elementKey, name, generation)`，generation 防重启后 stale identity）。
- `el.getAnimations()` 按 elementKey 过滤。

### 4.4 animation-composition（S3）

- css-parser：`animation-composition` 值解析（`replace|add|accumulate` 单值与逗号列表）；style-system：`ComputedStyle` 字段 + shorthand `animation` 位置（css-animations-2 语法扩展）+ 继承/初值（replace）。
- animation.rs：`apply_to_computed_style` 组合语义——add 对 transform/filter 类列表属性 = 列表拼接，数值属性 = 加法；accumulate 首切仅 transform（corpus 消费面）。

### 4.5 泵路径语义完备（S4，独立片）

- seek/pause/playbackRate 在墙钟泵下的时序正确性（currentTime setter 改写时钟相位）；testharness web-animations corpus 的 Timeout 面收敛。不阻塞 S1–S3 收益。

## 5. 实施计划

| 切片 | 内容 | 验证 | 预估 |
|---|---|---|---|
| S1 | 宿主桥 + AnimationState 扩展 + JS 代理转发（FR-1/2） | transform-interpolation-* 8 案 reftest；make test ×2 锚 + reftest 704 + smoke | 1–2 会话 |
| S2 | getAnimations 统一曝光（FR-3） | will-change-contents 类案 + getAnimations testharness 族回归 | 1 会话 |
| S3 | animation-composition（FR-4） | transform-additive-animation + 声明面单测 | 1 会话 |
| S4 | 泵路径 seek/pause/rate（§4.5） | web-animations corpus Timeout 面 | 1–2 会话 |

- 回滚切点：每片独立提交；桥回调不可用回落路径保证 S1 可 revert 至瞬间完成而零行为差。
- 风险：① JS/宿主双状态源漂移（命令异步落时钟 vs 同步本地 promise）——以「渲染采样只认时钟、JS 态只服务断言」单源纪律缓解；② getAnimations identity 与引擎动画重启竞态——generation 键；③ M3-S1 现绿断言回归——每片跑 web-animations corpus 全量对账。

## 6. 待用户决策

- 本立项批准（跨 engine+shim 深结构、多会话切片——按并行纪律第 11 条须用户拍板）。
- TBD-1/TBD-2 可在片内以 corpus 回归门自行裁定，不升级决策。

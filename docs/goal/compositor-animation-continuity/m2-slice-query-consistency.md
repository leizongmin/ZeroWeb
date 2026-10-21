# M2-S2 切片 RFC：动画查询一致性与结束事件（DC-2）

**状态**：Active ｜ 日期：2026-10-21 ｜ 前置：[S1 切片](m2-slice-compositor-sampling.md) 全链路已落（DC-1 有条件成立）
**控制面**：[master](master.md)

## 0. 执行摘要

S1 建立了 compositor 采样：flag 开时合成视觉按共享单调钟推进，但 renderer 侧引擎时钟不参与（生产无泵）。DC-2 缺口由此产生：**脚本查询看到的是未动画的底层级联值，结束事件无人派发**。本切片在 renderer 内闭合两者——查询与结束判定都以「已导出声明 + `CLOCK_MONOTONIC` + `zero-protocol` 采样器」这一与 compositor 完全同源的三元组求值，不新增 renderer↔compositor 采样值 IPC。

**源码核实结论（S2 设计依据，全部 file:line 已核对）**：

- gCS 查询是独立重算域：`__zw_get_computed_style`（engine `js_dom_shim/part01.js:6252`）→ 宿主回调（`js_dom_bridge/callbacks.rs:1589`）→ `serialize_computed_property`（`computed_style.rs:276`），从 html+inline-mutation 重 cascade；`computed_style_cache.rs` 全文 0 处 animation 引用——查询完全不读动画状态。
- 引擎时钟无内部时基：`AnimationClock` 由调用方传 `current_time`（`crates/engine/src/animation.rs:964`）；生产唯一泵 `tick_animation_clock`（`pipeline/mod.rs:997`）无 apps/ 调用方，仅 wpt-runner 使用（`testharness.rs:8434`，且用 UNIX 墙钟秒，与 monotonic ns 异域）。flag 开下主时钟上根本没有这些动画（R3882 静态求值在 clone 时钟上做，主时钟按纪律不被污染）——**引擎 tick 路径不能作 S2 真相源**。
- 采样器可直接复用：`zero_protocol::animation::sample_declaration(&declaration, now_monotonic_ns) -> AnimationSample { value, finished }`（`protocol/animation.rs:226`）纯函数；renderer 已依赖 zero-protocol；renderer 锚点（`AnimationExportState.anchors`，`apps/renderer/src/runtime.rs:193`）与 compositor Get 采样钟同为 `CLOCK_MONOTONIC` ns（`protocol/animation.rs:195`），同机同域。
- 结束事件派发管线存在但无驱动：`AnimationClock` 事件缓冲 → `take_pending_animation_events`（`pipeline/mod.rs:982`）→ renderer `dispatch_animation_events`（`apps/renderer/src/page_scripts.rs:414`）→ `script_gen.rs:206` AnimationEvent 脚本生成。flag 开下 pending 恒空。
- 浏览器→renderer 控制消息先例：`send_to_renderer`（`apps/browser/src/process_backend.rs:1518`）+ `SetFramePublishMode`/`RequestFrame`（`protocol/message.rs:46,48`；renderer 臂 `runtime.rs:3553,3563`）。
- CSS 动画无任何 currentTime/playState 查询面：WAAPI shim（`js_dom_shim/part03.js:10619` Animation 类）只收 `el.animate()` 产物（`part01.js:452` `_elementAnimations` 注册表），CSS animation-name 动画对它不可见；全仓无 CSSAnimation 接口。

## 1. 需求（Requirement）

| ID | 需求 | 来源 |
|---|---|---|
| FR-6 | flag 开且动画经 compositor 采样时，`getComputedStyle` 对已导出通道（opacity/transform）返回与合成视觉一致的采样值；同一线程同一时刻查询与 compositor 同 `t` 求值结果可等价断言（同一采样器、同一钟） | goal DC-2「查询与合成状态一致」 |
| FR-7 | 动画结束事件恰好一次：忙窗内结束的动画在 renderer 可调度后派发（主线程事件排队语义，与 Chromium 一致）；renderer 空闲期结束的动画由浏览器唤醒及时派发；不因重复采样/多帧 ack/导航换代重发或跨导航重放 | goal DC-2「结束事件不重发/漏发」 |
| FR-8 | 事件负载合规范：`animationName`/`elapsedTime`（按声明计时算术求出，与派发延迟无关）/目标元素/`pseudoElement` 与引擎既有 AnimationEvent 形状一致 | goal「拟新增事件与帧联合 fixture」+ 既有派发管线形状 |
| FR-9 | flag 关闭时查询路径与事件路径行为与现状逐字节一致（无 provider 注入、无唤醒消息、无事件） | S1 FR-3 同款回滚纪律 |
| FR-10 | 查询覆盖只作用于已导出的 (元素, 通道)；未导出动画（composable=false、matrix 关键帧等 S1b 边界）保持底层级联值不动，与合成视觉「同冻结」自洽 | S1b 导出域定义 |
| FR-11 | 新消息追加于 `IpcMessageKind` 末尾、新字段 `#[serde(default)]`；Android 消费方通配安全 | protocol 纪律（S1 FR-5 同款） |

**验收场景（S2c / COMP-02）**：fixture 含持续动画 + 结束断言脚本 + `__zwStartLoad()` 负载，flag 开真实窗口：(a) 忙窗前后 gCS(opacity) 采样值与 frame_dump 红盒 alpha 归因同相位（分析脚本既有通道）；(b) 动画结束（含忙窗内结束与空闲期结束两例）各派发恰一次 `animationend`，elapsedTime 与声明计时一致；(c) flag 关对照：gCS 返回级联值、零事件（现状冻结语义）；(d) 导航换代后旧动画事件不重放。

**范围裁剪（明示）**：`currentTime`/`playState` 需要新建 CSSAnimation 对象面（引擎现无此接口，WAAPI shim 与 CSS 动画解耦是既成语义，不可误改）；不在本切片，记缺口表（S2 后候选）。DC-2 届时按「gCS 一致 + 事件 exactly-once 已验，CSSAnimation 面未建」如实记录为有条件完成。

## 2. 已定设计（Decision）

### 2.1 查询一致性（S2a）

**真相源**：renderer `AnimationExportState` 的已发送声明集 + `monotonic_now_ns()`。查询时刻在 renderer 进程内就地求值 `sample_declaration`——与 compositor 同函数（FR-6 等价断言的单一实现）、同钟域，无 IPC。

**挂接点**：engine 定义查询覆盖提供者（engine 域 trait，输入元素标识 + 属性，输出 engine 域可选值；不引 zero-protocol 依赖，类型转换留 renderer——与 S1b「导出面 engine 产出、renderer 转 IPC」的分层一致）。`serialize_computed_property` 对 opacity/transform 先查覆盖，命中则**整体替代**（FR-10：与 compositor「干净基线上采样」镜像，禁止与底层值混合）；未命中走既有级联。

**transform 计算值口径**：采样值经 `fold_affine` 折叠后序列化为 `matrix(a, b, c, d, e, f)`——查询返回「合成后的实际仿射」，与采样视觉直接对齐；底层非动画路径维持既有函数列表序列化不动（engine `transform_to_css` 与 Chromium resolved matrix 的既有 diverge 不在本切片扩大）。

**缓存语义**：gCS 缓存（键 = html/drain_gen/style_version/CSS）继续缓存**底层级联**；覆盖在缓存命中之后、返回之前施加，缓存键不加时间项——非动画属性的缓存行为零变化，动画属性天然每次现算。已知容忍窗口：JS 改动画属性到新声明代际发布之间，查询按旧声明采样（与 compositor 保留旧声明到新帧提交的容忍一致，如实记录）。

### 2.2 结束事件（S2b）

**判定**：renderer 在两个时点用 `sample_declaration(…).finished` 对活跃声明集做惰性终结检查——(a) 每次 publish 组帧后（忙窗内/后自然覆盖）；(b) 浏览器唤醒时。

**唤醒**：browser 在 `compositor_animations_active` 真→假翻转时（`process_backend` 已跟踪该翻转）经新追加消息 `NotifyAnimationsSettled`（`send_to_renderer` 既有通道）通知 renderer 复查；renderer 判定无新终结即幂等返回。flag 关下 `animations_active` 恒假、无翻转、无消息（FR-9）。

**exactly-once**：`AnimationExportState` 增 `finished_reported: HashSet<(epoch, animation_id)>`——集合不变性由既有代际纪律保证；导航换代（epoch 重置）整表清空，跨导航重放被 epoch 键阻断（FR-7）。重复唤醒/多帧 ack 只会重复命中同一集合判等，无二次派发。

**派发**：复用既有 AnimationEvent 管线（`dispatch_animation_events` → `script_gen`），`elapsedTime` = 声明计时算术（delay + duration × iterations，按 fill/direction 语义取活动时长），与派发时刻解耦（FR-8）。取消路径（声明代际替换导致动画消失）按 S3 场景再验，本切片只做自然终结。

### 2.3 求值单一实现与验证口径（FR-6 断言）

采样进度/插值/终结判定全部落 `zero-protocol::animation`（S1 已建）；renderer 查询侧**不得**复制任何时间轴数学。等价单测：同一声明集扫 t，断言「查询覆盖路径输出」与「`animation_overrides::apply` 采样输出」同值（opacity 数值相等；transform 同一折叠仿射）。

### 2.4 触面清单

| 步 | 触点 | 新增 IPC |
|---|---|---|
| S2a | engine `js_dom_bridge/computed_style{,_cache}.rs` + 查询 provider trait；renderer `runtime.rs`（AnimationExportState 查询面）+ provider 实现 | 无 |
| S2b | renderer `runtime.rs`/`page_scripts.rs`（终结检查+派发）；protocol `message.rs` 追加 `NotifyAnimationsSettled`；browser `process_backend.rs`（翻转→唤醒） | `NotifyAnimationsSettled { tab 序语义沿用现有路由 }`（末尾追加） |
| S2c | fixture COMP-02 + observe/analyze 扩展 + 证据目录 | 无 |

**回滚切点**：S2a/S2b 各自成批、flag 门控；S2c 证据批。翻默认仍属后续独立决策（perf-gate 前置不变）。

## 3. 假设与待定

| 项 | 性质 | 说明 |
|---|---|---|
| 查询时刻与最近合成帧的相位容差 | Assumption | 同钟同采样器下偏差 = Get 往返 + 调度抖动（实测中位 13-17ms 采样节奏）；COMP-02 断言用相位区间而非逐帧相等 |
| engine 查询 provider 的元素标识可从 gCS 回调内部取得 | Assumption（源码线索） | 回调按 selector 解析后引擎内部有元素身份（inline-override 链路已按元素回放）；S2a 落地时核实 selector→多元素命中的覆盖语义（首命中即文档序首个，与级联一致） |
| `NotifyAnimationsSettled` 与 renderer 忙窗竞态 | Assumption | 消息在 IPC 队列排队，忙窗结束后被处理，与惰性检查幂等叠加 |
| cubic-bezier/steps 参数化 easing | Open | S1 枚举族沿用；COMP-02 fixture 用 linear/ease 族保持导出域内，参数化按需扩展 |
| CSSAnimation 对象面（currentTime/playState/getAnimations 覆盖 CSS 动画） | Open（范围裁剪） | 新 API 面，独立切片评估；WAAPI shim 既有语义不动（WPT web-animations 套件钉住） |
| 取消/暂停事件（animationcancel） | Open | 声明代际替换语义属 S3（DC-3 暂停/取消）场景，本切片只做自然终结 |

## 4. 与后续切片的契约预留

- `CompositorAnimationStateParams.finished`（S1 预留位）继续空置：S2b 判定在 renderer 侧完成，不需要 renderer→compositor 携带终结表；该位保留给未来需要 compositor 权威终结的场景（如 GPU 采样路径），若 S3/M4 仍无消费者则建议移除。
- S3（DC-3 命中/暂停/取消）沿用本切片的「声明代际替换」语义：暂停 = 采样钟冻结锚点（声明带 paused 标志位扩展），取消 = 代际替换剔除；查询覆盖与事件判定天然跟随新代际。

# M2 首切片 RFC：合成器采样动画独立推进（DC-1 链路）

**状态**：Draft（本轮起草，随实现推进更新）
**日期**：2026-10-21
**来源**：[goal 契约](../compositor-animation-continuity.md) M2「动画描述、时间采样、结束事件共享契约」+ DC-1；master.md 下一步
**证据输入**：M1 双基线（headless [comp01-baseline](evidence/2026-10-21-m1-comp01-baseline/README.md) / 真实窗口 [window-frames](evidence/2026-10-21-m1-window-frames/README.md)）+ 本轮三路源码核查（protocol/engine/compositor 消费面，核查点即文中 file:line 引用）

## 0. 执行摘要

**目标**：renderer 主线程被有界 JS 阻塞时，CSS opacity/transform 动画仍持续推进（DC-1），方式是 compositor 在 Get 帧时对已存 paint 做动画采样重光栅——完全复用 scroll_transform 既有先例（`GetCompositorFrame` 时 clone paint → 变异 → 全量 CPU 重绘 → swap）。

**核心缺口（源码核实）**：paint 图元无元素身份（opacity 已烘焙进颜色、transform 已扁平化为仿射），compositor 无法定位「动画元素的」图元；动画声明也从未出过 renderer 进程。本切片补两件事：**动画声明出协议**（appended 消息 + 最新生效语义）+ **动画作用域随 paint 出协议**（transform 恒发恒等矩阵作用域 + opacity 中性化作用域区间），其余全部复用既有机制。

**节奏结论**：compositor 进程零自主唤醒（纯 `recv()` 循环），浏览器侧已有常驻 16ms 轮询（`run_with_window_polling`）与平滑滚动的连续帧先例（`scroll_anim` 保持 `needs_redraw`）——**S1 由浏览器轮询驱动 Get，compositor 不新增定时器**。

**边界**：S1 只做渲染链路（DC-1）。查询一致性/结束事件（DC-2）、命中与暂停取消（DC-3）是 S2/S3，本 RFC 只预留契约位（`finished` 列表、代际绑定）。flag 默认关，验证后另轮翻默认。

## 1. 需求（Requirement）

| ID | 需求 | 来源 |
|---|---|---|
| FR-1 | flag 开启且页面有可合成动画时，renderer 主线程被忙循环阻塞期间，compositor 合成输出仍按动画时间轴持续更新 | goal DC-1 |
| FR-2 | 动画声明、作用域与采样语义为 protocol 共享契约，renderer 与 compositor 双方消费；采样求值单一实现，renderer 静态路径与 compositor 采样路径可等价断言 | goal「实现来源：拟新增动画描述在 protocol，双方消费共用契约」+ guidelines #6 |
| FR-3 | flag 关闭时全链路行为与现状逐字节一致（不新增消息、不消费新字段） | 谨慎推进 + 回滚切点 |
| FR-4 | 仅 opacity 与 transform（可仿射折叠子集）可合成；离散/未知属性/不可折叠 transform 的动画保持现状（冻结），如实记录支持子集 | goal「先实测支持子集」+ engine 插值边界（knowledge：普通 transform 不在专用插值臂、3D/异构离散切换） |
| FR-5 | 新消息追加于 `IpcMessageKind` 末尾（bincode 按声明序编码判别值），新字段一律 `#[serde(default)]` | protocol 纪律（message.rs:262 注释、R3254-L6） |

**验收场景（S1）**：COMP-01 fixture 加载后触发 `__zwStartLoad()`（4×220ms 忙片）与持续动画并存，flag 开启、真实窗口模式：compositor 侧帧序列（frame_dump manifest 时间戳 + 帧内容哈希）在负载窗口内持续出现**内容随动画相位变化**的新帧；负载结束后动画仍推进；flag 关闭对照跑帧序列与 M1 基线一致（FROZEN）。帧时间以 compositor 生产时间戳为准（生产帧序列，非 renderer CPU 截图）；GPU 路径参与度如实记录（Get 重光栅沿用 scroll 先例为 CPU 全量，GPU 采样属 M4 评估）。

## 2. 已定设计（Decision）

### 2.1 时间基

声明携带 `start_monotonic_ns`（u64），与 compositor 同机共享 `CLOCK_MONOTONIC`（Linux/Android 系统级单调域；Windows 用 QPC 同基换算，S1 先 Linux 落地，Windows 记待办）。采样 `elapsed = now - start`，迭代/方向/fill 语义在共享采样器内实现（与 engine `AnimationState` 字段一一对应）。

### 2.2 协议新增（全部追加式）

| 类型 | 形状 | 语义 |
|---|---|---|
| `AnimationPropertyValue` | enum：`Opacity(f64)` / `Transform(Vec<TransformFunctionValue>)` | 采样域的强类型值。transform **按 2D 函数列表传输**（translate/rotate/scale/skew/matrix，px/deg/无量纲），采样器内先逐函数插值再折叠仿射——若导出时预折叠为仿射再插值，旋转中间矩阵会偏离 engine 逐函数插值路径（guidelines #6），故折叠必须发生在采样之后；3D/异构端点在导出侧标 `composable: false` |
| `AnimationKeyframe` | `{ offset: f64, value: AnimationPropertyValue }` | 已按 engine `KeyframePoint` 排序补界的端点列 |
| `AnimationTiming` | `{ duration_ms: f64, delay_ms: f64, iterations: Option<f64>, direction: u8 映射, fill: u8 映射, easing: u8 映射 + 参数 }` | 与 engine `AnimationState` 字段对应；easing 先限 linear/ease 族枚举，step/cubic-bezier 参数随 S2 需求扩展 |
| `CompositorAnimationDeclaration` | `{ animation_id: u64, composable: bool, start_monotonic_ns: u64, timing, keyframes: Vec<AnimationKeyframe>, property: enum {Opacity, Transform} }` | `animation_id` 由 renderer 分配（`(NodeId::as_ffi(), name)` 哈希），paint 作用域与声明以它绑定 |
| `CompositorAnimationState` 消息 | `{ surface_id, navigation_epoch, generation: u64, declarations: Vec<…>, #[serde(default)] finished: Vec<u64> }` | **最新生效**（guidelines #13）：每 (surface, epoch) 单槽替换；generation 防 ABA；`finished` 为 S2 结束事件预留，S1 恒空 |
| `IpcAnimationScope` | `{ animation_id: u64, property: enum {Opacity, Transform}, draw_begin: u32, draw_end: u32, rect: Rect, baseline_opacity: f32 }` | 挂在 `PaintSnapshotParams` 新增 `#[serde(default)] animation_scopes: Vec<…>`；`draw_begin/end` 是 `draw_order` 索半开区间（painter 子树边界既有知识，`apply_opacity_to_new_primitives` 的 counts 快照即此边界） |

feature flag：`compositor_animations_enabled()` 读 `ZW_COMPOSITOR_ANIMATIONS`（默认关），按 `frame_shm.rs` 既有模式加 helper + `docs/runtime-environment.md` 行。

### 2.3 renderer 导出（flag 开启且 `FramePublishMode::Compositor` 时）

1. **声明**：在静态求值点（`render_html` R3882 处）从 `AnimationClock.active_animations` 导出声明集（clone 出的 static clock 即快照来源，求值可重放已由 M1 核实）；有声明且集合/代际变化才发 `CompositorAnimationState`。
2. **transform 作用域**：painter 对动画子树恒发一个恒等仿射 `TransformPrimitive`（rect=子树包围盒）并记作用域——compositor 只**就地改矩阵值**，不需向图元流插入 op。CPU 光栅的 transform 语义是「对 rect 区域已画像素做逆映射重采样」（`apply_transform_post`），静态路径同此保真度，采样路径继承之（旋转贴边裁切为继承性局限，如实记录）。
3. **opacity 作用域**：painter 对动画子树**中性化**输出（alpha 按未应用动画值烘焙，即 opacity=1 基线），作用域携带区间；compositor 按 `t` 采样值对区间内图元颜色做 alpha 乘（含 fills/glyphs/shadows 等颜色属主）。中性化避免「比值缩放」在基值趋零时的量化崩坏。
4. renderer 侧 dispatch（`runtime.rs:3358` 编译穷举 match）补新消息臂（仅发送方，不涉及接收）。

### 2.4 compositor 采样（`animation_overrides.rs`，与 `scroll_transform.rs` 同构）

```
Get 帧且 flag 开启且 surface 有声明且存在 composable 动画未完成:
    paint = clone(surface.paint)
    animation_overrides::apply(&mut paint, &declarations, now_monotonic)
        // 对每个 scope：采样 t → Opacity: 区间内颜色 alpha *= v
        //             Transform: 就地改写 transforms[i] 矩阵（恒等基线）
    paint_for_viewport(...)   // 既有滚动变异（顺序：先动画后滚动，均在 clone 上）
    to_render_primitives → rasterize_into_back（CPU 全量）→ swap
    应答携带 animations_active（仍有未完成动画）与 next 采样提示
```

`animations_active=false` 后浏览器回落既有事件驱动节奏。epoch 变更清声明与作用域（同 `surface.paint` 生命周期）。

### 2.5 浏览器节奏（S1 最小改动）

`poll_tab_fetch` 既有 16ms 轮询路径：可见 surface 的最近 Get 应答 `animations_active=true` 时，每 tick 持续发 `GetCompositorFrame` 并对采纳的新帧置 `needs_redraw`（与 `tick_smooth_scroll` 保持连续帧同型）。无新定时器、无 compositor 线程。

### 2.6 求值单一实现（FR-2）

采样进度/插值数学落 `zero-protocol` 新模块（纯函数，输入协议类型；无 engine 依赖反向）。engine 侧 opacity lerp 与 transform 仿射折叠逻辑收敛为调用同一模块（字符串域封装留 engine）；等价断言：单测对同一声明扫 t 采样，比较「engine tick 字符串域输出」与「协议采样器输出」（guidelines #6 双路径共享等价）。

## 3. 假设与待定

| 项 | 性质 | 说明 |
|---|---|---|
| `CLOCK_MONOTONIC` 跨进程可比（Linux/Android） | Assumption | 同机系统启动域，OS 保证；Windows QPC 待 S1 落地后核 |
| painter 子树边界 = counts 快照区间可在导出点复用 | Assumption（源码线索） | `apply_opacity_to_new_primitives`（painter/mod.rs:4321）已有该边界；导出实现时核实嵌套子树（动画套动画）区间语义 |
| 60fps 全量 CPU 重光栅成本可接受 | Assumption | scroll 先例同量级；**翻默认前过定向 perf-gate**（guidelines #23，rule 12） |
| easing 非线性族（steps/cubic-bezier 参数化） | TBD | S1 先枚举族；参数化随 S2 查询一致性一起做（查询也需同一数学） |
| 声明漂移（动画属性被 JS 中途改） | Open | 靠 mutation → 新帧 → 新声明代际收敛；S2 用暂停/取消场景验证 |
| GPU 光栅化路径的动画采样 | Open | S1 Get 路径 CPU 全量（scroll 先例）；GPU 采样并入 M4 GPU 证据评估 |

## 4. 实施交接

| 步 | 内容 | 触点 | 验证 |
|---|---|---|---|
| S1a | protocol：类型 + 消息（末尾追加）+ flag + `docs/runtime-environment.md` 行 + 序列化穷举清单扩条 | `crates/protocol/src/{message,paint_snapshot,frame_shm}.rs`、serialize_exhaustive | roundtrip 单测；renderer 之外消费方通配安全（已核查：唯一编译穷举面在 runtime.rs:3358） |
| S1b | renderer：声明导出 + painter 作用域/中性化 + runtime match 臂 | `apps/renderer/src/paint_export.rs`、`crates/engine/src/paint/painter/*`、`apps/renderer/src/runtime.rs` | flag 关逐字节不变；flag 开静态输出与 t=0 基线像素等价单测 |
| S1c | compositor：状态存储（最新生效/epoch 清理）+ `animation_overrides.rs` + Get 路径接线 + 应答扩展 | `apps/compositor/src/lib.rs`、新 `animation_overrides.rs` | 单测：采样区间变异正确性、代际替换、epoch 失效；frame_dump 观察负载窗口帧序列 |
| S1d | browser：`animations_active` 轮询保持 + 采纳置 `needs_redraw` | `apps/browser/src/{process_backend,app}.rs`、`crates/page-runtime` 契约同步 | 多进程集成（新鲜 peer 二进制，guidelines #12）：忙负载下帧持续更新 |
| S1e | DC-1 验收：真实窗口 COMP-01+负载跑、flag 双向对照、证据落账 | 脚本复用 frame_dump + run-observe 模式 | §1 验收场景 |

**回滚切点**：每步独立可回退；flag 默认关贯穿 S1，翻默认属 S1e 后独立决策（需 perf-gate 证据）。

**提交批次建议**：S1a 自成一批（纯追加零行为）；S1b/S1c 各自 flag 门控成批；S1d 随 S1c 或紧随；S1e 证据批。

## 5. 与后续切片的契约预留

- **S2（DC-2 查询/事件）**：`finished` 列表 + compositor 结束闩锁上报；脚本查询（currentTime/playState/getComputedStyle）以同基单调钟 + 同一协议采样器对齐；事件 exactly-once 由 generation + 去重集合保证。
- **S3（DC-3 命中/暂停/取消）**：暂停/取消走声明代际替换（新 `CompositorAnimationState` 即生效）；命中当前几何需把采样 transform 反馈进 hit-test 侧车（`IpcHitTestCache` 已有 node_id 通道）——S3 设计输入。
- **M3（worker canvas）**：本切片不动 canvas/worker；`animation_scopes` 不改变图元类型布局，无交集。

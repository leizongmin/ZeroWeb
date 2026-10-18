# Spec+RFC：DC-3 拖放交互管线——输入注入架构

**版本**: v1.0（已确认）
**日期**: 2026-10-18
**作者**: AI Assistant
**状态**: 已确认——挂载点 A（zero-webview 层）经用户 2026-10-18 确认，按 §7.1 切片开工
**需求来源**: [docs/goal/dnd-compat.md](../dnd-compat.md) DC-3/M3（用户 2026-10-18 点名批准，先出 RFC 再动工）

---

## 0. 执行摘要

- **一句话目标**：为 HTML5 拖放补宿主侧拖拽状态机（mousedown→dragstart→dragenter/over/leave→drop→dragend），使 33 案 Actions 语料可执行、桌面浏览器获得真实拖放能力。
- **本期范围**：同文档内鼠标拖放管线（状态机 + 命中 + 事件序 + IPC 转发 + runner 真管道注入）；DataTransfer 会话身份与 mode 切换。
- **明确排除**（继承 goal 排除面 + 本文划界）：OS 文件拖入、触摸拖拽（22 号 goal 域）、跨文档/iframe 拖放、文本选区拖动、OS 级 drag-out、拖拽视觉反馈（ghost/光标，TBD-2）。
- **核心约束**：①不预建假面——语料驱动与真实窗口输入共用同一状态机入口；②新输入路径不绕过 FrameTransaction 边界；③chrome 拖滚与页面拖放在 drag_source 判据上划界，非拖拽源的按压行为零变化；④事件派发覆盖 shim 与 native 双监听面。
- **推荐方案**：拖拽状态机放 `zero-webview` 层（`crates/webview/src/webview/drag_input.rs`，拟新增）——嵌入式边界统一三端（桌面浏览器经 renderer、runner 直连、外部嵌入者），engine 命中缓存增 `drag_source_eligible` 位，IPC 新增 `DragInput` 消息，TabManager 做仲裁（武装/转发/Esc/点击抑制）。
- **首个落地步骤**：engine 命中缓存加 drag_source 标志 + protocol 透传（无行为变化），单测落账后进 webview 状态机。

---

## 1. 背景与范围

### 1.1 背景

dnd-compat goal M1/M2 自主面已收口：DataTransfer/DragEvent 构造器与属性面 corpus 387/387 全绿（[M2 收口](evidence/2026-10-18-m2-convergence.md)）。剩余 DC-3 为交互管线：宿主缺拖拽状态机，页面 `draggable` 元素无法被真实拖动；语料中 33 案 `test_driver.Actions` 依赖面被 skip（`testharness_dnd.rs:78`）。

关键事实：**桌面浏览器主进程当前不向页面转发 mousemove**（`app_input.rs:541` 全部 chrome 消费）；页面连续 move 流仅 CDP `MouseEventType::Move` 一条路。runner 现有 Actions 注入是 shim 脚本合成事件（`__zw_pointer_move` 直派，`testharness.rs:9114`），仅事件面无默认动作——uievents 语料够用，dnd 不能走它（约束：避免假面）。

### 1.2 需求引用（不重建需求正文）

需求以 [goal 文档](../dnd-compat.md) 为准：DC-3 交互管线与事件序收口、DC-4 门禁（make test 全绿、make reftest 零回归、product-smoke 恒值）、关键约束（点名批准/避免假面/corpus 口径）、依赖约束（keyboard/uievents 同域协调、22-touch-events 互指不重叠）。本文交付技术设计与实施交接；验收数字（corpus 通过数）在实施切片 s3 实跑落账。

### 1.3 范围边界

- **在范围内**：同文档鼠标拖放（draggable 属性面 + img/a[href] 默认可拖）；拖拽事件序与 dropEffect/effectAllowed 协商；DataTransfer 会话身份与 readwrite/protected mode；runner Actions 真管道注入；桌面浏览器仲裁与转发；Esc 取消。
- **不在范围内**：§0 排除清单（来源：goal 排除面 + 本文按语料面划界）；Android 触摸路径无改动（现有 tap/长按行为不变）。

---

## 2. 现状分析

### 2.1 可复用资产（探索核实，2026-10-18）

| 资产 | 位置 | 对拖拽的作用 |
|---|---|---|
| 主线程命中快照 | `apps/browser/src/tab_manager.rs:554` `hit_test_element` ← `TabSnapshot.hit_test` | mousedown 时判定 drag_source，无需 renderer 往返 |
| 进程内命中 | `crates/webview/src/webview.rs:2168` `hit_test_element`；`crates/engine/src/hit_test.rs` `HitTestCache` | 状态机逐 move fresh 命中（drop target 解析） |
| 语义事件派发 | shim `__zw_dispatch_event`（`js_dom_shim/part06.js:13346`）+ native `dispatch_event_impl`（`dom_bindings/event.rs:121`） | drag 事件族复用既有 trusted 派发 |
| 事件序合成先例 | `__zw_pointer_down_sequence`/`up_sequence`（`part06.js:14541/14681`） | 「一次输入→事件族」模式参照 |
| 跨事件指针态 | `pressed_targets`（`tab_manager.rs:761/808`）按下/释放目标配对 | click 事务与拖拽会话并行的挂点 |
| 事务化默认动作 | `PendingDispatch`/`on_allowed`（`tab_manager.rs:653`） | dragstart preventDefault 门控参照 |
| epoch 防陈旧 | `PageNodeHandle`+navigation_epoch/document_generation | 拖拽跨导航静默终止的判据 |
| chrome 拖滚阈值 | `content_pointer_drag`/`content_scroll_drag_threshold`（`app_input.rs:351-355`） | 划界点：非拖拽源按压仍走拖滚 |
| DataTransfer JS 面 | `js_dom_shim/part05.js:13572`（M2 收口，types FrozenArray 语义按 dataStore） | 会话身份只需加 mode 开关与实例复用 |
| 默认可拖判定 | `dom_bindings/element.rs:590` `default_draggable`（img/audio/video/a[href]） | drag_source 判定复用 |

### 2.2 关键缺口

1. **无连续 move 通路**：桌面按压期间 move 全被 chrome 消费（`app_input.rs:541`），页面收不到。
2. **无拖拽状态机**：仓库内全部 "drag" 均为 chrome 级（tab/scrollbar/拖滚/选区），页面拖放为零——goal 禁止自主面预建占位，此为预期空缺。
3. **命中快照无 draggable 位**：`ElementHit`（`hit_test.rs:711`）含 tag/id/class/rect/selector，不含 drag_source 判定所需信息。
4. **shim 派发无 drag 分支**：`__zw_dispatch_event` 类型分支（`part06.js:13496`）无 drag* → DragEvent 映射，无会话 DataTransfer 存储。
5. **wire 无拖拽消息**：`MouseEventParams`（`protocol/src/message.rs:2235`）仅 Down/Up/Move/Click/DblClick，无拖拽会话语义。

---

## 3. 目标状态与总体架构

```
 桌面路径                                     runner/嵌入路径
┌──────────────┐  DragInput(Move/Up/Cancel) ┌──────────────────┐
│ TabManager    │ ─────────────────────────▶ │ apps/renderer     │
│ (仲裁/武装)   │   (zero-protocol 新消息)    │ handle_drag_input │
└──────┬───────┘                            └────────┬─────────┘
       │ mousedown 命中快照判 drag_source             │ apply_pointer_step
       ▼                                             ▼
┌─────────────────────────────────────────────────────────────┐
│ zero-webview: drag_input.rs（拟新增）——唯一状态机实现          │
│ Idle → Armed{source,origin} ──阈值──▶ Dragging{session}      │
│ 逐 move: hit_test_element → 目标迁移 → 事件序派发             │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ engine 命中: HitTestCache + drag_source_eligible（拟新增）     │
│ 派发: shim __zw_dispatch_event drag 分支 + __zwDragSession    │
│       + native listener 面（双面覆盖）                        │
└─────────────────────────────────────────────────────────────┘
```

- 桌面：TabManager 在 mousedown 用主线程快照判 drag_source；命中则武装，move/up 经 `DragInput` 转发，Esc 转 Cancel，会话结束回报用于点击抑制。
- runner：Actions 指针链步直接调 webview 新入口（与桌面同一状态机）。
- 状态机逐 move 在进程内做 fresh 命中（`hit_test_element`），事件经既有 trusted 派发路径，FrameTransaction 语义不变。

---

## 4. 设计选项（征询项）

| 维度 | A: zero-webview 层（推荐） | B: TabManager 主进程 | C: renderer 运行时 | D: shim 序列合成器扩展 |
|---|---|---|---|---|
| 三端共用（浏览器/runner/嵌入者） | 🟢 天然统一 | 🔴 runner 拿不到（须脚本合成=假面） | 🔴 同 B | 🟡 页面内可触发，但命中须宿主喂 |
| 架构边界符合（嵌入优先） | 🟢 能力进边界 | 🔴 绕过边界 | 🔴 绕过边界 | 🟢 属页面内面 |
| 逐 move 命中 | 🟢 进程内 fresh | 🔴 主线程快照可能陈旧 | 🟢 进程内 | 🔴 无几何能力 |
| 取消语义闭环 | 🟢 进程内同步 | 🔴 每 move 一轮 IPC 回执 | 🟢 进程内 | 🟢 进程内 |
| 既有代码改动量 | 🟡 新模块+派发分支 | 🟡 仲裁+状态机混入 chrome 层 | 🟡 输入路径改写 | 🟢 最小 |
| Chromium 参考同构 | 🟡 近似（WebCore 内） | 🔴 异构 | 🟢 同构（DragController 在 renderer） | 🔴 异构 |

**推荐 A**，理由：
1. CLAUDE.md 架构边界：`zero-webview` 是稳定嵌入边界，浏览器优先经该边界接入页面能力——B/C 使能力落在边界之外，嵌入者与浏览器分叉。
2. goal 约束「避免假面」：B/C 下 runner 只能继续脚本合成事件，语料驱动与真实路径不同源，正是门控要防的面。
3. dblclick/连续激活累计先例已在 webview user_actions 层（`webview/user_actions.rs`），指针手势状态机有 placement 先例。

C 的「Chromium 同构」优势在本仓让位于自身边界契约；D 仅可作为 A 的下游派发执行面（本来就要做的 drag 分支），不独立承载状态机。

> **已确认（2026-10-18，用户）**：挂载点选 A（zero-webview 层）。本 RFC 按 A 展开。

---

## 5. 详细设计（按方案 A）

### 5.1 engine：命中缓存 drag_source 标志

- `HitTestCache::from_document`（`hit_test.rs:114`）构建期，对每个候选节点标记 `drag_source_eligible: bool` = 自身或最近可拖祖先满足：`draggable` 内容属性为 true，或属 `default_draggable`（img/audio/video/a[href]，`dom_bindings/element.rs:580-590`）且 `draggable ≠ false`（`// FIXME: HTML § 6.7.3 draggable 挂载时机按 spec 逐条核对`）。
- 序列化：`HitTestNodeSnapshot`（`hit_test.rs:351`）→ `IpcHitTestCache`（`protocol/src/paint_snapshot.rs:661`）→ `TabSnapshot.hit_test` 透传该位（协议两端同步改）。
- 已知边界（不修）：命中为布局树近似、z-index 序未精确建模、overflow:hidden 裁剪不完整（`hit_test.rs` 既有注释）——拖拽沿用同一保真度，不单独加固。

### 5.2 zero-webview：拖拽状态机（`crates/webview/src/webview/drag_input.rs`，拟新增）

```
Idle ──apply_pointer_step(Down@x,y) 且命中 drag_source──▶ Armed{source, origin}
Armed ──Move 距 origin < 阈值──▶ Armed（无事件；click 事务不受影响）
Armed ──Move ≥ 阈值──▶ dragstart(source, cancelable)
   ├─ 未取消 ──▶ Dragging{session}（建 DataTransfer 会话, readwrite→protected）
   └─ 已取消 ──▶ Idle（回落普通 mouse 流；后续 move 不再进状态机）
Dragging ──Move(x,y)──▶ fresh 命中 T：
   T ≠ cur：dragleave(cur) → dragenter(T, cancelable) → dragover(T, cancelable)
   T = cur：dragover(T, cancelable)
   命中为空：dragleave(cur) → cur=None
Dragging ──Up──▶ operation ≠ none：drop(T, cancelable, readwrite) → dragend(source)
                 operation = none：dragend(source)
                 （Up 本身与 click 被抑制，回报 consumed=true）
Dragging ──Cancel(Esc)──▶ dragend(source, dropEffect=none)
任意态 ──导航/文档替换（epoch/generation 变）──▶ 静默终止，无事件（目标已陈旧，
         参照 tab_manager release 过期目标直接取消先例）
Idle ──Down 于非 drag_source──▶ 不武装，事件走既有派发（零变化）
```

事件序与取消语义依据 HTML Standard § 6.7 drag and drop processing model（`html.spec.whatwg.org/multipage/dnd.html#drag-and-drop-processing-model`；事件汇总表 `#drag-events`；data store mode `#drag-data-store-mode`）：

| 事件 | 目标 | cancelable | mode | 备注 |
|---|---|---|---|---|
| dragstart | source | 是 | read/write | 取消则整个拖拽不发生 |
| drag | source | 否 | protected | 源上伴随事件；周期重派见 TBD-1 |
| dragenter | 新目标 | 是 | protected | 先于 dragover |
| dragover | 当前目标 | 是 | protected | 默认动作=禁止放置；未取消则 operation 依 dropEffect 协商，取消与否决定 drop 可否发生 |
| dragleave | 前目标 | 否 | protected | |
| drop | 当前目标 | 是 | read/write | 仅 operation ≠ none 时派发 |
| dragend | source | 否 | protected | 无论成败必派；dropEffect 为最终结果 |

dropEffect/effectAllowed 协商（corpus 口径，M2 已建属性面）：effectAllowed 合法值 `none/copy/move/link/copyLink/copyMove/linkMove/all`（非法值忽略不改）；operation = dragover 取消时按 dropEffect ∩ effectAllowed 集合映射，未取消为 none。

- **公开 API**（拟新增，webview 嵌入面）：`apply_pointer_step(&mut self, step: PointerStep)`（`PointerStep::{Down, Move, Up}`，含坐标与按钮）与 `cancel_drag(&mut self)`；`DragSession` 查询器供宿主判断会话活性。
- **派发通道**：drag 事件经既有 trusted 派发路径（同 mousedown/click），**必须覆盖 shim 与 native 双监听面**（既有约定：宿主页面派发要覆盖适用监听面，`dom_bindings/event_target.rs` + shim `__zw_dispatch_event`）——先落 shim 面（corpus 驱动语境），native 面在同一切片对齐，防止桌面 V8 绑定监听器漏事件。
- **阈值**：机制定在 Armed 态内（单实现，三端共用）；具体数值实施时按 corpus 行为调定（TBD-4），量级参照 `content_scroll_drag_threshold`。
- 单文件预计 < 2000 行（准则 7）；事件派发复用既有函数，不复制分发逻辑。

### 5.3 shim：drag 事件分支与会话 DataTransfer

- `__zw_dispatch_event` 类型分支（`part06.js:13496` 邻域）新增 drag* → DragEvent 映射（构造器 `part05.js:13376` 已收口），`dataTransfer` 从 `__zwDragSession`（拟新增的 shim 全局会话槽）取**同一实例**——dragstart 时创建并登记，dragend 后清除；`e.dataTransfer` 跨事件恒等是 corpus 断言点。
- mode 开关挂现有 dataStore 语义（M2 的 types FrozenArray 缓存不变）：dragstart/drop 前置 readwrite，其余 drag 事件 protected（`DataTransfer.items` 等写入面在 protected 下按 spec 抛错/忽略，落 M2 已建的 disabled/protected 判定位）。
- `isTrusted` 沿用 `__zwMakeTrustedEvent`（`part05.js:12379`）。

### 5.4 IPC 契约（zero-protocol，两端同步）

- 拟新增 `DragInput(DragInputParams)`：`{ phase: DragPhase /*Move|Up|Cancel*/, x: f32, y: f32, source_selector: String }`——`Move` 阈值前也转发（状态机内部吞掉）；`Up` 结算 drop/dragend；`Cancel` 即 Esc。
- 回执拟新增 `DragSessionEnded { consumed: bool }`：renderer → browser，`consumed=true` 时浏览器抑制本次手势的 mouseup/click（§5.5）；`consumed=false`（Armed 内结束，未越阈）由浏览器按既有路径结算 click，不发本消息。
- 不复用/不改 `MouseEventParams`：语义不同（拖拽会话 vs 独立事件），避免 CDP 既有 Move 流误入状态机。

### 5.5 浏览器主进程仲裁（TabManager）

- mousedown：主线程快照命中读 `drag_source_eligible`；命中则记录 `drag_armed = Some{source, origin}`（紧邻 `pressed_targets` 记录处，`tab_manager.rs:804`），**不改变**既有 mousedown 派发与 click 事务。
- 按压期间 move：`drag_armed` 存在时，除 chrome 常规消费外**额外**发 `DragInput{Move}`；同时抑制 `content_pointer_drag` 拖滚（`app_input.rs:653-659`）——拖拽源按压-move 语义从「拖滚页面」变为「页面拖拽」（有意的行为变化，与主流浏览器一致；非拖拽源按压零变化）。
- mouseup：`drag_armed` 存在时发 `DragInput{Up}`；收到 `DragSessionEnded{consumed=true}` 则跳过 `dispatch_page_click`（拖拽消费手势后不派 click/mouseup，spec 行为）。
- Esc：`drag_armed` 存在时优先于 chrome 快捷键路由为 `DragInput{Cancel}`（`app_input_keys.rs` 优先级链上加一条页面拖拽分支，位于既有「页面输入转发在 chrome 条件后决定」次序内）。
- 导航开始（epoch 变更）：清 `drag_armed` 并发 `Cancel` 兜底（renderer 侧同时有 epoch 静默终止兜底，双保险）。

### 5.6 runner 注入改造（`tests/wpt-runner/src/testharness_dnd.rs` 通道）

- dnd 通道的 Actions 指针链步改调 webview 新入口：`pointerDown → apply_pointer_step(Down)`、`pointerMove → apply_pointer_step(Move)`、`pointerUp → apply_pointer_step(Up)`（坐标复用 `resolve_pointer_target` 现算机制，`testharness.rs:8956`）。
- **uievents/pointerevents 通道不动**：继续走 `__zw_pointer_move` 脚本合成（其语料只断言事件面）。两套注入模式并存，收敛不在本期（记录为已知取舍，非假面：dnd 语料走的是与桌面相同的真实状态机）。
- 扫描面扩展 + skip 规则移除：`DND_TEST_SUBDIRS` 增 `interactiveelements`、`platform`、`events` 三目录；删除 `source.contains("test_driver.Actions")` skip 项（`testharness_dnd.rs:78`），保留 `-manual` 与无 harness 引用排除。语料面 11 → 44 案（33 案新增面为 Actions 依赖，部分案可能仍因其他 testdriver API 或上游红被记 skip/fail，实跑落账为准）。
- `td_last_pointer`/`mut_hover_settle_if_dirty` 悬停重算机制沿用（`testharness.rs:8082`）。

---

## 6. 安全考虑

- **数据暴露**：会话 DataTransfer 仅在同文档事件间传递；跨文档/iframe 拖放不在范围，无跨 origin 数据流。OS 剪贴板/文件系统不接触（拖出/拖入均排除）。
- **注入面**：`DragInput` 是宿主→页面输入，与 DispatchDomEvent 同信任级（非页面可伪造的 isTrusted=false 合成事件）；runner 入口仅存在于测试进程，不进产品构建路径。
- **脚本面**：`__zwDragSession` 为 shim 内部槽，页面 JS 不能伪造 dragstart 语义（会话仅由宿主状态机创建/清除）；页面可读 `e.dataTransfer` 与既有 DataTransfer 构造器（M2 面）一致。

---

## 7. 实施计划与交接

### 7.1 里程碑切片

| 切片 | 内容 | 行为变化 | 验证 |
|---|---|---|---|
| s1 | engine 标志 + protocol 透传/DragInput 消息 | 无（字段无消费者） | 单测 + 协议 roundtrip |
| s2 | webview 状态机 + shim drag 分支 + apply_pointer_step | 无（新 API 无调用者） | 状态机单元测试（假派发记录器，user_actions 测试先例） |
| s3 | browser 仲裁（armed/转发/Esc/click 抑制/拖滚划界） | 桌面拖放激活 | tab_manager/app_input 单测；桌面手验 |
| s4 | runner 真管道注入 + 扫描面扩展 + skip 移除 | runner dnd 通道 | 44 案落账；make test / make reftest / product-smoke（DC-4） |
| s5 | goal 收口归档（M4） | — | corpus 总账 + evidence |

### 7.2 文件、职责与修改顺序

| 顺序 | 路径/模块 | 动作 | 职责 | 依赖/被依赖 | 风险 | 验证 |
|---|---|---|---|---|---|---|
| 1 | `crates/engine/src/hit_test.rs` | 修改 | 快照节点 + `drag_source_eligible`（复用 `default_draggable`） | 被 2/3/5 消费 | 命中保真度既有边界（§5.1） | hit_test 单测（拟增断言） |
| 2 | `crates/protocol/src/paint_snapshot.rs`、`message.rs` | 修改 | 透传位 + `DragInput`/`DragSessionEnded` 消息 | 两端同步（3、5、renderer） | 契约漂移 | 协议 roundtrip 测试 |
| 3 | `crates/webview/src/webview/drag_input.rs` | 新增 | 状态机 + `apply_pointer_step`/`cancel_drag` | 依赖 1；被 5、6 调用 | 事件序 spec 偏差 | `tests/drag_input.rs`（拟新增） |
| 4 | `crates/engine/src/js_dom_shim/part05.js`、`part06.js` | 修改 | drag 分支 + `__zwDragSession` + mode 开关 | 被 3 的派发路径触达 | 双监听面对齐（§5.2） | 3 的单测覆盖事件族 |
| 5 | `apps/browser/src/tab_manager.rs`、`app_input.rs`、`app_input_keys.rs`；`apps/renderer/src/runtime.rs` | 修改 | 仲裁/转发/Esc/抑制 + `handle_drag_input` | 依赖 2、3 | click 事务回归 | browser 单测（拟增）+ 手验 |
| 6 | `tests/wpt-runner/src/testharness_dnd.rs` | 修改 | 真管道注入 + 扫描/skip 调整 | 依赖 3 | 上游红案记账 | dnd 通道实跑 |
| 7 | `docs/goal/dnd-compat.md` + evidence | 修改 | 收口账本 | — | — | 数字落账 |

### 7.3 首批提交建议

| 提交 | 范围（引用 §7.2 顺序） | 预期 | 验证 |
|---|---|---|---|
| Commit 1 | 步骤 1+2 | 契约与标志就绪，零行为变化 | make test 全绿 |
| Commit 2 | 步骤 3+4 | 状态机+会话可独立驱动，零产品行为变化 | 新增单测绿 + make test |
| Commit 3 | 步骤 5 | 桌面拖放激活 | 单测 + reftest 零回归 + smoke 恒值 |
| Commit 4 | 步骤 6+7 | 44 案数字收口 | dnd 通道 + DC-4 门禁 |

### 7.4 回滚

按提交粒度 revert：Commit 3/4 是行为激活点，revert 即回当前行为；Commit 1/2 落地后无消费者，惰性无害。无持久化/迁移面。

---

## 8. 测试策略

| 层 | 位置（拟新增/既有） | 覆盖 | 依赖 |
|---|---|---|---|
| 状态机单测 | `crates/webview/src/tests/drag_input.rs`（拟新增） | 正常序（dragstart→over→drop→dragend）；dragstart 取消回落；阈值内不触发；dropEffect 协商（取消/未取消）；DataTransfer 恒等与 mode；Esc 取消；epoch 失效静默终止；非 drag_source 不武装 | 假派发记录器（既有 user_actions 测试模式） |
| 命中标志单测 | hit_test 既有测试面（拟增断言） | draggable=true/false/缺省 ×（img/a[href]/div）；最近可拖祖先 | 真实 DOM 构造 |
| 仲裁单测 | `apps/browser/src/tests.rs`（拟增） | 武装条件、拖滚划界（拖拽源不滚页/普通源滚页）、Esc 优先、consumed 点击抑制 | 现有 tab_manager 测试底座 |
| 协议 roundtrip | protocol 既有测试面（拟增） | DragInput 序列化/反序列化；快照位透传 | — |
| 语料 | `make testharness-dnd`（既有通道扩展） | 44 案（11 既有零回归 + 33 Actions 面实跑落账） | WPT pin 不变 |
| 门禁 | make test / make reftest / product-smoke（DC-4） | 全绿/零回归/逐字节恒值 | — |

脆弱选择逻辑显式覆盖：drag_source 判定（draggable 属性 × 默认可拖元素矩阵）按选择/匹配逻辑单测选中结果，不只测最终成功；目标迁移序（dragleave/enter/over 次序、空命中）逐事件断言。

---

## 9. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|---|---|---|---|---|
| TBD-1 | dragover/drag 周期性重派（元素静止时的循环派发） | 可选 | 33 案 face 是否断言（实跑前不可知） | s4 实跑后按语料决定；不需要则记 excluded |
| TBD-2 | 拖拽视觉反馈（ghost image/光标） | 可选 | 无验收面 | 不在本期；html5test 分值面另议 |
| TBD-3 | 阈值具体数值 | 重要 | corpus 行为标定 | s2 实施时调定，机制已定（§5.2） |
| TBD-4 | 33 案中的上游红/skip 残留 | 重要 | 实跑数据 | s4 落账，按 goal 记账纪律处理 |

已决：PR-1 挂载点 → A（用户 2026-10-18 确认，见 §4）。

---

## 10. Spec Lint 报告

### 结构完整性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 执行摘要存在性 | ✅ Pass | §0 六要素齐 |
| 场景存在性 | ✅ Pass | 设计交付：验收=§1.2 引用 goal DC-3/DC-4 + §7.1 切片验证列 + §8 测试策略逐层断言项（等价验收表） |
| 测试绑定 | ✅ Pass | §8 每层注明位置与断言对象；语料/门禁命令实名 |
| 实施交接完备 | ✅ Pass | §7.2 七步含依赖/风险/验证；§7.3 首批四提交 |
| 首步可执行性 | ✅ Pass | §7.2 步骤 1 + §7.3 Commit 1，验证=make test |

### 语言精确性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 模糊动词 | ✅ Pass | 状态转移与事件序用条件-动作-结果表表达（§5.2），无「处理/支持」式掩蔽 |
| 行为歧义 | ✅ Pass | 双监听面、mode 切换时点、click 抑制条件均为单义陈述；拖滚划界写明「拖拽源变语义、普通源零变化」 |
| 外部事实保守化 | ✅ Pass | spec 行为引 HTML § 6.7 锚点（项目已实跑同 spec 语料佐证）；33 案断言方向未实跑的一律写「实跑落账」，不预写期望值 |

### 一致性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 原始意图与授权一致 | ✅ Pass | 范围=goal DC-3 原文；排除面继承 goal+语料事实；无新增 Must（用户点名批准的是「先出 RFC」，本文即 RFC） |
| 范围冲突 | ✅ Pass | §1.3 在/不在范围无交集（触摸拖拽排除 vs Android 路径无改动，两说一致） |
| 实现来源闭合 | ✅ Pass | 每项能力来源落在既有模块（§2.1 资产表 + §5 各节 file:symbol）；唯一新文件 `drag_input.rs` 边界已述 |
| PendingReview | ✅ Pass（已决） | PR-1 挂载点经用户 2026-10-18 确认为 A，见 §4 |

**汇总**：9 Pass / 0 Warning / 0 Fail / 0 Skip / 0 PendingReview / 0 N/A
**门禁判定**：无 Fail、无阻塞 TBD，本阶段就绪，进入实施。

---

## 11. 修订历史

| 版本 | 日期 | 变更 |
|---|---|---|
| v0.1 | 2026-10-18 | 首稿：现状探索（六路调研）、四方案对比、方案 A 详细设计、切片计划、Lint |
| v1.0 | 2026-10-18 | 用户确认挂载点 A 与全文，PR-1 关闭，状态转已确认，进入实施 |

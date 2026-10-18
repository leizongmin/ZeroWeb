# Spec+RFC：M2 指针捕获管线——锁定语义 + 位移流 + 锁定态输入改道

**版本**: v1.1（已确认·M3 实施中）
**日期**: 2026-10-19
**作者**: AI Assistant
**状态**: 已确认——方案 A + N1–N4 纳入 + fsKey 同族修复随 FR-10，经用户
2026-10-19 征询通过；M3 已用户点名启动（同日），s1 起按 §8 顺序实施；
v1.1 为实施期修正（语料钉死，见 §12 修订历史，需求/验收/切片不变）
**需求来源**: [docs/goal/pointerlock-compat.md](../pointerlock-compat.md) DC-2/M2
（用户 2026-10-19 点名批准起草 RFC，同日征询确认）；
范围输入 = [M1 甄别报告五缺口](evidence/2026-10-18-m1-baseline.md) +
探索新发现的事件面缺口（§2.2，征询确认纳入）

---

## 0. 执行摘要

- **一句话目标**：把 R2939 的 spec-alike 锁态 API 面补齐到语料口径——transient
  activation 门 + 拒绝/事件语义修正 + 锁定态位移流（movementX/Y）+ 鼠标事件
  改道锁目标 + boundary 抑制 + 移除/导航/fullscreen 生命周期联动，使 C 面 15
  门控案进入执行面。
- **本期范围**：shim 锁语义修复组；位移流字段管线（一般路径，不限锁定态）；
  webview 锁镜像与锁定态输入单入口（桌面真实输入与 runner 合成输入双馈）；
  runner pointerlock 通道 skip 收窄与锁定路由；桌面 Esc 释放。
- **明确排除**：OS 级光标隐藏/显示与 raw input（`-manual` 7 案领地，含
  `unadjustedMovement` 真 raw 位移）；CDP `mouseMoved` 通路的锁定态改道（测试
  基建，非语料/真实窗口路径，记已知限制）；pointer lock 与 dnd 拖拽会话并发
  交互（语料无断言面）；Android 触摸路径（无改动）。
- **核心约束**：①不预建假面——锁定态位移流与真实窗口输入共用 webview 单入口；
  ②锁页面语义（grant/deny/事件面）留在 shim（R2939 既有驻地，fullscreen/
  移除钩子/激活消费先例全在 shim）；③宿主派发覆盖 shim 与 native 双监听面
  （dnd M3 同款）；④分母口径变化显式落账（C 面入分母后基线重算）。
- **推荐方案**：**shim 语义权威 + webview 锁镜像**——grant/deny/事件生命周期
  语义在 shim 修复；shim 经新增 host 桥 `__zw_pointer_lock_state_changed`
  把锁态变化通知 webview 镜像；webview 新增锁定态位移管线（算 delta、改道
  派发、抑制 boundary），桌面转发规则与 runner 注入双馈同一入口。
- **首个落地步骤**：s1 shim 语义修复组（activation 门 + NotAllowedError +
  change/error 事件面 + options 语义）——零管线、零 IPC，B 面 2 案翻绿
  （25.0%→75.0%），单提交可独立收口。

---

## 1. 背景与范围

### 1.1 背景

M1（DC-1 ✅）完成 corpus census 与甄别：27 资产中 3 案执行（1/4 = 25.0%），
15 案因依赖 `test_driver.`/`test-helper.js` 被 skip（C 面，门控），7 案
`-manual` 排除。R2939（2026-08-07）已在 js_dom_shim 落地 spec-alike 锁态：
`requestPointerLock` Promise grant/deny、`exitPointerLock` void、
`pointerLockElement`、change/error 事件 + on* handler、host 可插拔 deny
开关 `__zw_pointer_lock_enabled`。本轮探索（2026-10-19，四路调研）在 M1
五缺口之外新发现三类事件面/生命周期缺口（§2.2 N1–N4）。

关键事实：**锁定态位移流的三条输入源现状都不到页面**——桌面非按压态
mousemove 全被 chrome 悬停消费（`app_input.rs:541`），按压态仅 dnd 会话
转发（`app_input.rs:657`）；runner 合成 move 走 shim 合成器
（`__zw_pointer_move`，`part06.js:14152`），派发的 MouseEvent 不携带
movementX/Y（`part06.js:13533-13588`）；`DomEventDetail` 无 movement 字段
（`script_gen.rs:10-50`）。锁定改道机制在 dispatch 层有 pointerevents 捕获
先例（`part06.js:13623-13642`）。

### 1.2 需求引用（不重建需求正文）

需求以 [goal 文档](../pointerlock-compat.md)为准：DC-2 捕获管线 RFC 征询
通过后实现收口、DC-3 门禁（账本纪律、make test 全绿、make reftest 零回归）、
依赖约束（run-rules §9：host-runtime 光标/输入面与 keyboard 系遗产同域）。
范围输入以 [M1 甄别报告](evidence/2026-10-18-m1-baseline.md)五缺口挂账
（82-92 行）为基，本文 §3 需求簇逐条映射到 corpus 断言面。验收数字（corpus
通过数）在实施切片实跑落账，本文不预写期望值。

### 1.3 范围边界

- **在范围内**：shim 锁语义修复（activation 门、NotAllowedError、change/
  error 事件面 target/bubbles/trusted、options 参数与重复请求语义、锁目标
  移除自动解锁、锁目标 shadow retarget、导航清理）；movementX/Y 位移流
  （一般路径 + 锁定态）；webview 锁镜像 + 锁定态输入单入口；桌面锁定态
  mousemove 转发 + Esc 释放；runner 通道 skip 收窄 + 锁定路由 + C 面 15 案
  执行落账；`pointerlock_shadow` 依赖资产 `../shadow-dom/resources/
  shadow-dom.js` 补拉（或显式排除，TBD-1）。
- **不在范围内**（来源：goal 排除面 + 本文按语料面划界）：OS 光标隐藏与
  raw input（7 个 `-manual` 案领地——15 个非 manual 案无一断言"光标真被
  隐藏"或真 raw 位移，本轮语料盘点实证）；CDP 通路的锁定改道；dnd 会话与
  锁并发；Android 触摸路径（现有行为不变）。
- **代码变更边界**：允许修改 = §8.2 清单及任务范围内必要文件（预计清单外
  的必要修改按仓内准则先说明）；禁止修改 = Android 触摸路径、uievents/
  pointerevents 通道注入模式（`testharness.rs` 既有 `ShimSynthesis` 语义）、
  `-manual` 排除规则（`testharness_pointerlock.rs:52-54`）、`_fireDocEvent`
  既有函数体（fullscreen 等共用方，只增新派发器不改原函数）。

---

## 2. 现状分析

### 2.1 可复用资产（探索核实，2026-10-19）

| 资产 | 位置 | 对捕获管线的作用 |
|---|---|---|
| R2939 锁态 API 面 | `js_dom_shim/part04.js:3833-3853`（request）、`part06.js:4813-4824`（exit/getter）、`part06.js:432-434`（`_pl*` 状态） | 语义修复的驻地；grant/deny 骨架保留 |
| fullscreen 激活消费先例 | `part04.js:3805-3813`（`_zwTransientActive` 检查 + 同步消费） | FR-1 activation 门照抄模式 |
| 激活状态变量 | `part06.js:442-443`（`_zwTransientActive`/`_zwActiveEver`）；runner 注入点 `testharness.rs:9906-9930,10091` | B 面语义的判定源；runner 侧已就绪 |
| fullscreen 移除钩子先例 | `_fsOnNodeRemoved`（`part06.js:516-541`，挂 `el.remove()` `part04.js:7427-7429`，同步清态 + 异步派事件） | FR-6 移除解锁照抄模式 |
| trusted 派发漏斗 | `__zw_dispatch_event` R312 块（`part06.js:13607-13616`，`isTrusted=true` + `_zwUaDispatch`） | 激活标记挂接点（按 type 白名单）+ 锁事件可信化 |
| dispatch 层捕获改道先例 | `part06.js:13623-13642`（pointer/mouse 族重定向，pointerevents 落地） | 锁定态改道的同类机制参照 |
| compat 抑制先例 | pointerdown prevented 抑制 compat mouse（`part06.js:14643-14669`） | boundary 抑制的模式参照 |
| dnd 状态机 + 抑制标志 | `apply_drag_input`（`crates/webview/src/webview/drag_input.rs:558`）、`DragApplyReport.session_active`（:540-549） | webview 单入口 + 会话抑制的架构模板 |
| 双监听面派发 | `dispatch_dom_event_inner`（`crates/webview/src/webview.rs:4071-4140`，shim `__zw_dispatch_event` + native `script_dispatch_native_event`） | 锁 change/error 与锁定态 move 的派发通道 |
| detail 透传 + prevented 回填批协议 | `DomEventDetail`（`script_gen.rs:10-50`）+ 序列化（:123-131）+ `'ok'/'prevented'`（`part06.js:13722`） | 加 movementX/Y 字段照 dnd 加 `data_transfer_id` 同款方式 |
| 导航重置四连点 | `webview.rs:1005,1690,1831,1927`（`drag_input` 复位 + epoch 递增） | webview 锁镜像复位的挂点 |
| shim 导航清理通道 | `__zw_reset_form_state`（`part01.js:5787`，导航时 `js_worker.rs:1725`/`tab_js_worker.rs:549` 调用） | `_pl*`/激活变量 shim 侧复位的挂点 |
| 桌面输入仲裁 | `drag_sessions`（`tab_manager.rs:72`）、转发门 `page_drag_active`（`app_input.rs:657`）、Esc 链 dnd 分支（`app_input_keys.rs:101-116`） | 锁定态转发规则与 Esc 分支的插入位 |
| runner 双管道注入 | `PointerStepInjection{ShimSynthesis,DragStateMachine}`（`testharness.rs:7709-7728`）、并行喂真管道（:9159-9176）、`td_drag_mouse_suppressed`（:8120） | 锁定路由的注入模式模板 |
| native MouseEvent movementX init | `dom_bindings/event.rs:340-342`（init 字典已支持，缺省 0） | constructor.html 断言面已绿的一半；s2 不破坏 |
| runner test_driver 激活注入 | `test_driver.click`/`bless`/`Actions.send` 置激活（`testharness.rs:9906-9930,10091`） | C 面锁定案的手势来源已就绪（含 bless） |

### 2.2 关键缺口

M1 五缺口（甄别报告 45-52 行）+ 本轮新发现：

1. **无 transient activation 门**：无手势也 grant（B 面 2 案红根因）。
2. **deny 拒绝类型 `TypeError`**：spec 要求 `NotAllowedError` DOMException
   （`part04.js:3840-3842`）。
3. **无 movementX/Y 位移流**：`DomEventDetail` 无字段（`script_gen.rs:10-50`）、
   shim mouse 分支不透传（`part06.js:13533-13588`）、宿主无从计算 delta。
4. **无 `options.unadjustedMovement`**：方法签名无参（`part04.js:3833`）；
   且**语料不容 NotSupportedError 降级**（两案 catch 到即 `assert_unreached`，
   本轮盘点实证）——只能走"接受 option + 语义正确"路线。
5. **无生命周期联动**：锁目标移除不解锁（fullscreen 有 `_fsOnNodeRemoved`
   先例而 `_pl*` 无对应物）；无 shadow retarget；无 boundary 抑制；full
   screen 与 lock 双向零联动（本轮实证 `_fsKey`/`_plKey` 写点全 5 处互不触碰）。
6. **N1——change/error 事件面不符**：`_fireDocEvent` 统一 `bubbles:true`
   （`part06.js:548-553`），spec 与 `pointerlock_api_behavior` 断言
   `bubbles=false`；`event.target` 为 null（`_dispatchToListeners` 只写
   currentTarget），断言 `target==document`；`isTrusted=false`（未走
   `__zwTrusted`）。
7. **N2——重复请求语义过粗**：现"同元素即 no-op"（`part04.js:3845`），
   `pointerlock_unadjustedMovement_update` 断言**同元素 option 变更须重锁 +
   重派 change**——no-op 条件须细化为"同元素且同 options"。
8. **N3——状态跨导航泄漏**：`_plKey/_fsKey/_zwTransientActive` 均不在
   `__zw_reset_form_state` 清单（`part01.js:5787`），文档替换后锁态与激活
   存活——pointerlock 侧必修（锁不可跨文档）；`_fsKey` 泄漏为同族兄弟 bug
   （§5.7 待确认建议）。
9. **N4——锁态宿主不可见**：锁状态纯 shim 闭包，webview/浏览器/runner 均不知
   锁定与否——锁定态输入改道无挂点，host 桥 `__zw_pointer_lock_enabled`
   无生产注册（仅测试 `part07.rs:2251-2254`），是 deny 开关不是状态源。

---

## 3. 需求簇（FR 映射 + 语料验收）

需求正文不重建（§1.2 引用 goal + M1）；本节把范围输入收敛为 10 个需求簇，
每簇绑定 corpus 断言面作为等价验收项。分母口径：C 面入分母后基线由
1/4（25.0%）重算，实施落账（s1 后 = B 面翻绿的中间口径，s3 后 = 全口径）。

| FR | 需求簇 | 内容（条件→行为） | 语料验收案 | 切片 |
|---|---|---|---|---|
| FR-1 | activation 门 + 拒绝语义 | 无 transient activation 时 `requestPointerLock()`：派 `pointerlockerror` + Promise reject `NotAllowedError` DOMException；有激活（runner `test_driver.click`/`bless`/`Actions.send` 注入，或真实输入经 R312 白名单标记）则照常 grant 并**同步消费**激活（fullscreen 先例同款） | pointerlock-requires-gesture、pointerlock_without_gesture（B 面 2 案）；pointerlock_promise（有激活 grant 正路） | s1 |
| FR-2 | change/error 事件面 | 锁定态变化派发：`target==document`、`bubbles=false`、`cancelable=false`、`isTrusted=true`；未锁时 `exitPointerLock()` no-op（无任何事件）——现有 no-op 保留（`part06.js:4821`） | pointerlock_api_behavior（事件面子案钉 N1 修复；no-op 子案现状实现预期已满足，实跑确认） | s1 |
| FR-3 | options 参数与重复请求 | `requestPointerLock(options)` 接受 `{unadjustedMovement}`（存态；**不容 NotSupportedError**——语料两案 catch 即 unreached）；重复请求语义 = 同元素**且同 options** 才 no-op resolve，option 变更（同元素或跨元素）重锁 + 派 change | pointerlock_unadjustedMovement、pointerlock_unadjustedMovement_update、pointerlock-maintains-mousedown（×2 variant） | s1 |
| FR-4 | movementX/Y 位移流（一般路径） | 宿主/合成派发的 MouseEvent 携带自上一鼠标事件的 screen 坐标差（`movementX == screenX_now − screenX_prev`，逐 move 与累计两断言） | movementX_Y_basic（15 案中唯一零 requestPointerLock，管线前哨） | s2 |
| FR-5 | 锁定态捕获管线 | 锁定期间：mouse/pointer 事件改道锁目标（含 mousedown/up/click）；movementX/Y 连续累计；boundary 事件（mouseenter/leave/over/out）零触发；`e.buttons` 按压态保持；wheel 改道锁目标（默认滚动作用于锁目标，preventDefault 可抑制） | pointerlock_all_mouse_events_to_locktarget、pointerlock_target_change、pointerlock_suppress_mouse_boundary_events_when_locked、pointerlock-maintains-mousedown、pointerlock_wheel_event_outside_target | s3 |
| FR-6 | 移除自动解锁 | 锁定中锁目标自文档移除：锁态同步清除（mouseup 同帧断言 `pointerLockElement==null`），`pointerlockchange` 异步派发 | pointerlock_remove_target、pointerlock_remove_target_on_mouseup | s3 |
| FR-7 | shadow retarget | 嵌套 shadow tree 中锁内部元素：`document.pointerLockElement` 返回 shadow-inclusive 最近祖先 host；各层 `shadowRoot.pointerLockElement` 逐层正确。实现来源：shim 祖先链信息（`__zw_pointer_up_sequence` ancestorChain 同源，`part06.js:14718`）上溯 shadow host 边界 | pointerlock_shadow（依赖资产 TBD-1） | s4 |
| FR-8 | fullscreen 不打断 | 既有锁在 fullscreen 进入（含跨 iframe 元素全屏）时保持，不发多余 change | pointerlock_fullscreen-change | s4 |
| FR-9 | 侧键 | 锁定中 button=3/4（back/forward）mouseup 派发到锁目标且可 preventDefault | mouse_buttons_back_forward（依赖合成层侧键能力，TBD-3） | s4 |
| FR-10 | 导航/文档生命周期 | 导航或文档替换：shim `_pl*` 与 `_fs*`（同族，已确认一并）清零（不派事件——文档已换）、webview 锁镜像复位、激活变量清零；webview 侧复用导航重置四连点（`webview.rs:1005,1690,1831,1927`） | （语料无正断言——防泄漏语义，单测覆盖） | s3 |

优先级：FR-1~FR-9 **必须**（corpus 断言面或 spec 生命周期钉定）；FR-10
**必须**（spec 生命周期，语料无正断言面，单测覆盖）。无「应该/可以」级
需求——范围输入全部来自 M1 挂账与语料钉定面，无用户附加的软性期望。

---

## 4. 目标状态与总体架构

```
 页面 JS（shim 语义权威）                宿主输入（webview 锁镜像 + 位移管线）
┌────────────────────────────┐          ┌────────────────────────────────────┐
│ requestPointerLock(options) │          │ 桌面: app_input 锁定态转发 → TabManager│
│  activation 门(FR-1)        │          │       → PointerLockInput IPC ──┐    │
│  deny: error+NotAllowedError│          │ runner: td move 检锁 → 锁定路由 ─┤    │
│  grant: _pl* 置态           │          └────────────────────────────────┼───┘
│   └─ host 桥通知 ────────────┼──────────────────────────────────────────┘
│      __zw_pointer_lock_     │                    ▼
│      state_changed(sel,     │   ┌────────────────────────────────────┐
│      handle, locked, opts)  │   │ webview: pointer_lock 模块（拟新增）  │
│   └─ change/error 事件 ──┐  │   │ 锁镜像 + 上一位置 + delta 计算        │
│                          │  │   │ 入口: apply_locked_pointer_move(x,y)│
│  el.remove() → 同步清 _pl*│  │   │  → hit 锁目标改道派发(FR-5)          │
│  导航 → __zw_reset 清零   │  │   │  → dispatch_dom_event_inner 双监听面 │
│                          ▼  ▼   │  （detail 携 movementX/Y, FR-4/5）   │
│      __zw_dispatch_event 漏斗 ◀─└────────────────────────────────────┘
│      （R312 trusted + 激活标记白名单）
└────────────────────────────┘
```

- **语义流（页面起搏，shim 内闭环）**：`requestPointerLock` 的门/拒绝/授
  予/事件/移除/导航语义全在 shim（现状驻地，先例密集）；grant/deny/exit
  时经新 host 桥把锁态变化通知 webview 镜像，并改经双监听面派发 change/
  error（替代 `_fireDocEvent`，修 N1 事件面 + 补 native 监听面）。
- **输入流（宿主起搏，webview 单入口）**：锁定期间桌面真实 move（新增转发
  规则）与 runner 合成 move（锁定路由）都进 webview `pointer_lock` 模块
  单入口；模块持上一位置算 delta、按锁目标改道派发；boundary 事件因改道
  派发路径不合成而天然为零（合成器路径被旁路，见 §8 防假绿注记）。
- **方向对照 dnd**：dnd 是宿主输入起搏（mousedown 武装）→ 页面收事件序；
  pointerlock 是页面 JS 起搏（requestPointerLock）→ 后续宿主输入被改道。
  因此锁**页面语义**留 shim、锁**宿主可见性**上移 webview——不把 grant/
  deny 决策跨语言搬进 webview（§5 方案对比）。

---

## 5. 设计选项（征询项）

| 维度 | A: shim 语义权威 + webview 锁镜像（推荐） | B: 锁状态全量上移 webview | C: 纯 shim 合成管线 |
|---|---|---|---|
| grant/deny/事件语义修正成本 | 🟢 shim 内局部修（先例密集） | 🔴 Promise 跨语言同步往返重构 | 🟢 同 A |
| 桌面真实锁定位移流 | 🟢 webview 单入口，真实输入直达 | 🟢 同 A | 🔴 桌面无通路=假面 |
| runner/真实同源（避免假面） | 🟢 双馈同一入口 | 🟢 同 A | 🔴 仅合成 |
| 锁态宿主可见（桌面转发/Esc 判据） | 🟢 镜像 + 桥通知 | 🟢 原生可见 | 🔴 宿主不可见 |
| 改动面/风险 | 🟡 shim 修复组 + 新 webview 模块 + 1 桥 | 🔴 grant 路径重构 + 异步 promise 手术 | 🟢 最小 |
| 架构边界符合（嵌入优先） | 🟢 管线在 `zero-webview` 边界内 | 🟢 同 A | 🟡 能力不进边界 |

**推荐 A**，理由：
1. grant/deny 是页面可见语义且 R2939 已在 shim 落地，修复先例（fullscreen
   激活消费、移除钩子、`_fsQueueStep`）全在 shim——B 方案把决策跨语言搬移
   只增加 Promise 同步往返手术，无 corpus 增益。
2. dnd 门控要防的假面是「语料与真实输入不同源」；A 的位移管线单入口双馈
   （桌面转发 + runner 路由）与 dnd `apply_drag_input` 同构，满足同源约束。
3. C 方案下桌面用户锁定后无位移流（非按压 move 不转发页面的现状不变），
   正是本轮门控要解决的空缺，不成立。

> **已确认（2026-10-19，用户）**：挂载点选 A。本文按 A 展开；实施待点名
> （用户 2026-10-19 裁决「仅确认，实施另启」）。

---

## 6. 详细设计（按方案 A）

### 6.1 shim 语义修复组（FR-1/2/3/6 shim 侧，s1）

- **activation 门**（`part04.js` requestPointerLock 内）：host deny 开关
  （`__zw_pointer_lock_enabled` 返 '0'）先查（'0' → 派 pointerlockerror +
  reject NotAllowedError——permission-denied 语义归 NotAllowedError，替换现
  TypeError）；再查 `_zwTransientActive`——false：派 pointerlockerror（§6.2
  新派发器）+ `reject(DOMException 'NotAllowedError')`，不置态；true：
  **仅检查不消费**（v1.1 实施期修正：`unadjustedMovement_update` 案一次激活
  连续两次请求均须过——spec request steps 无 consume 步，与 requestFullscreen
  的显式同步消费先例不同）。
- **change/error 事件面**（修 N1）：新增 `_firePointerLockEvent(type)`
  替代 `_fireDocEvent` 调用（`part04.js:3840,3849`、`part06.js:4823`）——
  `target=document`（照 fullscreen 事件路径显式设 target 的回落
  `part06.js:503` 同款）、`bubbles=false`、`cancelable=false`、
  `isTrusted=true`（`__zwTrusted` 印记，`part03.js:8427`）。
- **options 参数**：签名改 `function (options)`；解析 `{unadjustedMovement}`（缺省
  视为 false；非法字典按 WebIDL 忽略未知键）；存入锁态
  （`_plUnadjusted` 有效布尔，与 `_plKey` 同生灭——v1.1 实施期修正：语料只
  变异 unadjustedMovement 单成员，按最小实现比对有效布尔而非对象深相等）。
  **不实现 NotSupportedError 拒绝**——
  语料两案不容（§2.2-4）；真 raw input 在排除面（§1.3）。
- **重复请求语义**（修 N2）：no-op 条件从「同元素」细化为「同元素且
  `_plUnadjusted` 有效布尔相等」；否则按重新授锁定态 + 派 change
  （`pointerlock_unadjustedMovement_update` 断言面）。
- **移除钩子**（FR-6 shim 侧）：新增 `_plOnNodeRemoved`（照 `_fsOnNodeRemoved`
  `part06.js:516-541` 模式：沿 parentNode 上行比对 `_plKey`，命中则同步清
  `_pl*` + 异步派 change），挂 `el.remove()` 路径（`part04.js:7427-7429`
  邻位）。**不扩** removeChild/replaceWith——fullscreen 先例明示按精准修改
  不扩（`part06.js:513-515` 注释），语料两案均走 `remove()`。
- **导航清理**（FR-10 shim 侧，修 N3）：`_plKey/_plSel/_plHandle/_plUnadjusted`
  与 `_zwTransientActive/_zwActiveEver` 加入 `__zw_reset_form_state` 清单
  （`part01.js:5787`，导航时 `js_worker.rs:1725`/`tab_js_worker.rs:549`
  触发）。清零不派事件（文档已替换，无监听面意义）。**同站一并清
  `_fsKey/_fsSel/_fsHandle`**（fullscreen 锁态同族泄漏，用户 2026-10-19
  确认随 FR-10 修复）。

### 6.2 激活标记的真实输入挂接（FR-1 完整性）

现状激活只来自 runner 命令签发（`testharness.rs:9906-9930,10091`）——真实
桌面点击/按键从不置激活。在 `__zw_dispatch_event` R312 trusted 块
（`part06.js:13607-13616`）加 type 白名单（pointerdown/mousedown/keydown
等输入族，排除 load/error 等生命周期事件——同走漏斗
`script_gen.rs:152-158`），命中则置 `_zwTransientActive=true`。这使真实
窗口点击可解锁 pointer lock（产品路径），且不依赖 runner。旁路 trusted 派
发面（`__zwMakeTrustedEvent` 生命周期事件、wheel inline 印记
`part06.js:14369-14372`）不在白名单场景，天然不受影响。

### 6.3 位移流字段管线（FR-4，s2）

照 dnd 加 `data_transfer_id` 的同款三层扩展：

1. `DomEventDetail`（`script_gen.rs:10-50`）加 `movement_x: i64, movement_y:
   i64`（缺省 0）；序列化（:123-131）同步。
2. shim mouse 分支（`part06.js:13573-13588`）透传 `movementX/movementY`
   入 MouseEvent 构造（native ctor init 已支持，`event.rs:340-342`；
   constructor.html 面不回归）。
3. 计算职责在**事件源**：runner move op 持 `td_last_pointer` 前后位置算
   delta（`testharness.rs:9146-9181` 邻位）；锁定态由 webview 锁模块算
   （§6.4）。movementX 语义锚点：Pointer Lock spec `#dom-mouseevent-
   movementx`（自上一 mouse 事件的 screen 差，不限锁定态）。
   假设（待 s2 实跑验证）：shim 合成事件 `screenX` 与 `clientX` 一致（无
   OS 窗口偏移），`movementX_Y_basic` 断言的是差值等式，一致偏移可消去。

### 6.4 webview 锁镜像与锁定态位移管线（FR-5/10，s3）

- **新模块** `crates/webview/src/webview/pointer_lock.rs`（拟新增，dnd
  `drag_input.rs` 同构）：`PointerLockState { target: Option<String>,
  last_pos: Option<(f32,f32)>, unadjusted: bool }`。
- **入口** `WebView::apply_locked_pointer_move(&mut self, x: f32, y: f32)
  -> Result<...>`：未锁定时调用为 no-op；锁定时 delta = (x,y) − last_pos，
  更新 last_pos，`dispatch_dom_event_inner(锁目标 selector, "mousemove",
  detail{client_x/y, movement_x/y})`——改道即入口只向锁目标派发（不 hit
  test），boundary 事件不在该派发面合成（§8 注记）。mousedown/up/click
  改道同入口族（`apply_locked_pointer_button`，拟新增）。
- **镜像维护**：host 桥通知（§6.5）写 `target/unadjusted`；导航四连点
  （`webview.rs:1005,1690,1831,1927`）随 `drag_input` 复位同点清零；
  `exit` 由 shim host 桥或桌面 Esc（§6.6）触发。
- **查询面** `WebView::pointer_lock_target(&self) -> Option<&str>`：runner
  路由判据 + 桌面仲裁（经 IPC 回传，§6.6）。

### 6.5 host 桥：`__zw_pointer_lock_state_changed`（N4，s3）

- shim 在 grant/deny/exit/移除解锁/option 重锁五处调用
  `__zw_pointer_lock_state_changed(payload)`（`typeof === 'function'` 守卫，
  未注册静默跳过——与既有 deny 开关同风格）。payload：`{locked: bool,
  selector, handle, unadjustedMovement}`。
- **生产注册点**：`js_dom_bridge.rs` 桥注册面（deny 开关的注册机制同族），
  浏览器路径落地 renderer webview，runner 路径直接落测试进程 webview——
  两路径都有 webview 实例，镜像无分叉。
- **事件派发改道**：shim 的 change/error 不再 `_fireDocEvent`，改经 host
  桥让 webview 走 `dispatch_dom_event_inner`（双监听面 + trusted +
  `target=document`）——shim 漏斗收到后照常落 document 监听。顺序保持
  「先派事件后 resolve Promise」（现序，`part04.js:3849-3850`）。
- 桌面仲裁可见性：renderer 镜像变化经新 IPC `PointerLockChanged{locked,
  tab_id}`（拟新增，`protocol/src/message.rs`，dnd `DragInputResult` 同款
  回执模式）上报 TabManager，供转发门与 Esc 分支判据。

### 6.6 桌面真实输入（s3，行为激活点）

- **转发规则**（`app_input.rs:541-684` `handle_mouse_move`）：TabManager 判
  tab 锁定时，非按压 move 也转发（新增 `page_pointer_lock_active(tab_id)`
  门，与 dnd `page_drag_active` 门并列 ：657 邻位）→ `PointerLockInput{x,y}`
  IPC（拟新增）→ renderer `apply_locked_pointer_move`。chrome 悬停消费
  （tooltip/链接悬停等）在锁定态跳过（锁定的语义即光标不可达 chrome）。
- **Esc 释放**（`app_input_keys.rs:101-116` dnd 分支邻位插锁分支，同款守
  卫）：锁定时 Esc → renderer 触发 `exitPointerLock` 语义（经 webview 通
  知 shim 清态 + 派 change，§6.5 通道）——UA 手势释放，spec 行为，桌面可
  用性必需（corpus 该面在 manual，本文按产品合理性纳入，非语料驱动）。
- **OS 光标隐藏**：排除面（§1.3）——headless 无光标，桌面真隐藏是平台
  窗口面（TBD-4 挂账）。
- **wheel**：锁定态 wheel 转发锁目标（FR-5 wheel 子案）——桌面 wheel 处理
  面加锁分支；runner 侧依赖 Actions.scroll 支持（TBD-3）。

### 6.7 runner 注入（s3）

- **skip 收窄**：仅删 C 面内容 skip（`testharness_pointerlock.rs:55-57`
  `test_driver.||test-helper.js` 项）；`-manual`（:52-54）与无 harness 兜底
  （:58-60）保留。C 面 15 案入执行面，分母重算落账。
- **锁定路由**：move op 派发前查 `webview.pointer_lock_target()`（§6.4）——
  锁定则只走 `apply_locked_pointer_move`（delta 由模块算），跳过
  `__zw_pointer_move` 合成（防双派）；未锁定走现路 + movement delta
  （§6.3）。down/up/click 同判据分流。注入模式枚举不新增（锁是运行时状
  态，不是通道属性——与 dnd 的通道级 `DragStateMachine` 区分）。
- **语料驱动面**：C 面触发源为 `test_driver.click`/`Actions`/`bless`（本轮
  盘点实证），激活注入点已就绪（§2.1 末行）；`pointerlock_shadow` 的
  `shadow-dom.js` 资产获取见 TBD-1；`Actions.scroll`/侧键能力见 TBD-3。

### 6.8 fullscreen 联动与兄弟缺口处置（FR-8）

- **corpus 断言面**：fullscreen 进入（含跨 iframe）不打断锁、不多发
  change——现状 `_fs*`/`_pl*` 互不触碰（§2.2-5 实证），**保障性验收**：
  s1 语义组与 s3 管线落地后该案实跑确认零回归即可，预期无新代码（若实跑
  红则按 `_fsQueueStep` 异步时序排查）。
- **spec 更深处语义**（fullscreen 退出时锁的处置）语料未钉，本文不预写行
  为，实现时按 Pointer Lock spec 对应步骤加 FIXME 锚点，红案记账。
- **已决（2026-10-19，用户确认）**：`_fsKey/_fsSel/_fsHandle` 跨导航泄漏
  （N3 同族，`part01.js:5787` 清单同一站）随 FR-10 同提交一并清零。

---

## 7. 安全考虑

- **激活面**：activation 门使无手势页面无法静默夺锁；R312 白名单只对宿主
  trusted 输入置位（`_zwUaDispatch` 印记前置），页面合成 `dispatchEvent`
  不经 R312，无法自授激活——门不可被页面绕过。
- **注入面**：`PointerLockInput`/`PointerLockChanged` 与 `DragInput` 同信任
  级（宿主→页面输入 IPC，非页面可伪造）；runner 入口仅测试进程。
- **脚本面**：锁态 host 桥仅宿主可注册；shim 内 `_pl*` 闭包变量页面不可直
  触；`__zw_pointer_lock_enabled` deny 开关语义不变（嵌入方可插拔禁用）。
- **数据面**：位移流只含坐标差，无跨 origin 数据；无持久化。

---

## 8. 实施计划与交接

### 8.1 里程碑切片

| 切片 | 内容（§ 引用） | 行为变化 | 验证 |
|---|---|---|---|
| s1 | shim 语义组（§6.1+6.2）FR-1/2/3 | B 面 2 案翻绿（25.0%→75.0% 中间口径）；锁 API 外零变化 | `make testharness-pointerlock` 实跑 + `make test` |
| s2 | 位移流字段（§6.3）FR-4 | 合成 mouse 事件开始携带 movementX/Y | movementX_Y_basic 实跑 + `make test` |
| s3 | 锁管线（§6.4-6.7）FR-5/6/10 | C 面 13 案入执行面（分母重算落账）；webview 新模块 + host 桥 + IPC（无桌面行为变化） | C 面实跑 + 协议 roundtrip + `make test` |
| s4 | 桌面激活 + 长尾（§6.6 + FR-7/8/9）+ 收口 | 桌面锁定转发/Esc 激活 | 桌面手验 + corpus 总账 + `make reftest` 零回归 + DC-3 |

### 8.2 文件、职责与修改顺序

| 顺序 | 路径/模块 | 动作 | 职责 | 依赖/被依赖 | 风险 | 验证 |
|---|---|---|---|---|---|---|
| 1 | `crates/engine/src/js_dom_shim/part04.js`、`part06.js`、`part01.js` | 修改 | §6.1 语义组（门/拒绝/事件面/options/移除钩子/导航清理） | 被 3/5 触达 | 事件面改动波及 fullscreen 共用 `_fireDocEvent`——只增新派发器不改原函数 | part07 锁单测扩展（`test_pointer_lock_api_r2939` 先例 `part07.rs:2129-2279`）+ B 面实跑 |
| 2 | `crates/engine/src/js_dom_bridge/script_gen.rs` | 修改 | `DomEventDetail` movement 字段 + 序列化 | 被 3/4/6 消费 | 字段加法向后兼容 | 序列化单测 |
| 3 | `crates/engine/src/js_dom_shim/part06.js`（mouse 分支） | 修改 | movementX/Y 透传 | 依赖 2 | 无 | constructor.html 不回归 |
| 4 | `crates/webview/src/webview/pointer_lock.rs`（拟新增）+ `webview.rs` | 新增/修改 | 锁镜像 + `apply_locked_pointer_move` + 查询面 + 导航四连点复位 | 依赖 2；被 5/6/8 调用 | delta 状态机边界（首 move delta=0） | 模块单测（假派发记录器，drag_input 测试先例） |
| 5 | `crates/engine/src/js_dom_bridge.rs` | 修改 | host 桥 `__zw_pointer_lock_state_changed` 生产注册 | 依赖 1/4 | 注册点须覆盖浏览器与 runner 两路径 | 双路径桥测试 |
| 6 | `crates/protocol/src/message.rs` + `apps/renderer/src/runtime.rs` | 修改 | `PointerLockInput`/`PointerLockChanged` 消息 + renderer 臂 | 两端同步（契约） | 契约漂移 | 协议 roundtrip |
| 7 | `apps/browser/src/app_input.rs`、`app_input_keys.rs`、`tab_manager.rs` | 修改 | 锁定态转发门 + Esc 分支 + 仲裁镜像 | 依赖 6 | click/chrome 悬停回归 | browser 单测（拟增）+ 手验 |
| 8 | `tests/wpt-runner/src/testharness_pointerlock.rs`、`testharness.rs` | 修改 | skip 收窄 + 锁定路由 | 依赖 4 | 上游红案记账 | C 面 15 案实跑 |
| 9 | `tests/wpt-runner/scripts/goals/29-pointerlock-compat.sh` + goal 文档 + evidence | 修改/新增 | shadow 资产（TBD-1）+ 账本收口 | — | — | 数字落账 |

### 8.3 首批提交建议

| 提交 | 范围（§8.2 顺序） | 预期 | 验证 |
|---|---|---|---|
| Commit 1 | 步骤 1 | s1 语义组收口，B 面 2 案翻绿 | pointerlock 通道 + `make test` |
| Commit 2 | 步骤 2+3 | s2 位移流，movementX_Y_basic 翻绿 | 同上 |
| Commit 3 | 步骤 4+5+6 | s3 管线就绪（runner 可驱动），零产品行为变化 | 新单测 + roundtrip + `make test` |
| Commit 4 | 步骤 7+8 | C 面执行 + 桌面激活 | C 面落账 + reftest 零回归 |
| Commit 5 | 步骤 9 + s4 长尾 | corpus 总账收口 | DC-3 门禁 |

### 8.4 回滚

按提交粒度 revert：Commit 4 是行为激活点，revert 即回当前行为；Commit 1–3
中 shim 语义组（Commit 1）本身即行为变化（B 面翻绿是目的性变化），revert
恢复 R2939 现状；Commit 2/3 落地后无消费者面小，惰性无害。无持久化/迁移面。

---

## 9. 测试策略

| 层 | 位置（既有/拟新增） | 覆盖 | 依赖 |
|---|---|---|---|
| shim 语义单测 | `js_dom_bridge_tests/part07.rs`（扩展 `test_pointer_lock_api_r2939`，:2129-2279） | 无激活 reject NotAllowedError + error 事件；有激活 grant + 消费；option 变更重锁派 change；移除同步清态 + 异步事件；导航清零；事件面 target/bubbles/trusted 逐项断言 | 既有桥测试底座 |
| webview 锁模块单测 | `crates/webview/src/tests/`（拟新增，drag_input 测试先例） | 未锁 no-op；delta 累计与首 move 零；改道只落锁目标；导航复位；**同一 move 序未锁路径产 boundary、锁定路径不产**（防恒真假绿，§6.4 注记的显式对比） | 假派发记录器 |
| 协议 roundtrip | protocol 既有测试面（拟增） | `PointerLockInput`/`PointerLockChanged` 序列化 | — |
| 桌面仲裁单测 | `apps/browser/src/tests.rs`（拟增） | 转发门（锁定转发/未锁零变化）、Esc 分支优先序、chrome 悬停跳过 | 既有底座 |
| 语料 | `make testharness-pointerlock` | 3 案零回归 + C 面 15 案逐案落账（分母重算） | WPT pin 31597693 不变；TBD-1/2/3 项按记账纪律 |
| 门禁 | `make test` / `make reftest` / product-smoke（DC-3） | 全绿 / 零回归 / 恒值 | — |

脆弱选择逻辑显式覆盖：no-op 判定（同元素 × options 深相等矩阵）、移除钩
子命中（锁目标在移除子树内/外）、shadow retarget 逐层选择（FR-7）、激活白
名单 type 集合（输入族命中/生命周期事件不命中）——均按"选中了正确对象"
断言，不只测最终成功。

---

## 10. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|---|---|---|---|---|
| TBD-1 | `pointerlock_shadow` 依赖 `../shadow-dom/resources/shadow-dom.js` 补拉 | 重要（阻塞 FR-7 验收） | goal fetch 脚本 DIRS 扩展 vs 单文件补拉 | s4 前定；不成则该案按记账纪律排除并显式记录 |
| TBD-2 | shim 合成事件 `screenX==clientX` 假设 | 重要（FR-4 断言前提） | s2 实跑首查 | s2 落账时验证；不成立则 delta 锚点改 client 差并复核语料等式 |
| TBD-3 | runner `Actions.scroll` 与 button 3/4 合成能力 | 重要（FR-9/wheel 案前提） | runner op 面实查 | s3 实查；不支持则对应案按记账纪律记录，不阻塞主链 |
| TBD-4 | OS 光标隐藏/显示（桌面 UX） | 可选 | 平台窗口面方案 | 排除面挂账，远期池 |
| TBD-5 | C 面实跑后的上游红/残留 | 重要 | 实跑数据 | s3/s4 落账，按 goal 记账纪律处理 |

已决：
- 范围五缺口 + 新缺口 N1–N4 全覆盖（M1 报告 82-92 行挂账逐项对映，无静默
  收窄）——2026-10-19 征询确认。
- PR-1 挂载点 → A（用户 2026-10-19 确认，见 §5）。
- `_fs*` 同族泄漏随 FR-10 一并修复（用户 2026-10-19 确认，见 §6.8）。
- 实施时机 → 另启（用户 2026-10-19 裁决「仅确认，实施另启」，M3 待点名）。

---

## 11. Spec Lint 报告

### 结构完整性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 执行摘要存在性 | ✅ Pass | §0 六要素齐（目标/范围/排除/约束/方案/首步） |
| 场景存在性 | ✅ Pass | FR-1~FR-10 每簇绑定 corpus 断言案或单测断言项（§3 表），FR-10 无语料面显式改绑单测 |
| 异常路径覆盖 | ✅ Pass | deny 路径（FR-1）、未锁 no-op（FR-2）、首 move delta=0、boundary 防假绿对比、导航清零、TBD 降级路径（TBD-1/3 记账纪律）均落 §6/§9 |
| 测试绑定 | ✅ Pass | §3 验收列实名 corpus 案；§9 每层注明位置与命令（`make testharness-pointerlock` 等） |
| UI 对齐 | N/A | 无 UI 接口需求 |
| TBD 清零 | ✅ Pass | 征询后无阻塞级 TBD 残留（PR-1 已决，见 §10 已决列）；TBD-1/2/3 均为实施期内可决项 |
| 约束覆盖 | ✅ Pass | 核心约束①→FR-5+§6.4 双馈单入口；②→方案 A §5；③→§6.5 双监听面；④→§3 分母注记+s3 落账 |
| 实施交接完备 | ✅ Pass | §8.2 九步含依赖/风险/验证；§8.3 五提交 |
| 首步可执行性 | ✅ Pass | §0 首步 = §8.1 s1 = §8.3 Commit 1，验证=通道实跑+make test |

### 语言精确性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 模糊动词 | ✅ Pass | FR 簇用条件→行为表述；「接受 option」落 §6.1 具体为解析/存态/比较三步 |
| 行为歧义 | ✅ Pass | no-op 条件（同元素且 options 深相等）、移除钩子命中（子树含锁目标）、激活消费时点（同步）均单义；fullscreen 退出侧语料未钉——§6.8 显式不预写，留 FIXME，非掩蔽 |
| 无量化描述 | N/A | 本 goal 无 NFR（质量门=DC-3：make test 全绿/reftest 零回归，§9 已绑定） |
| 要求强度保持 | ✅ Pass | 优先级声明（§3 表后）区分钉定来源；排除面不带软性期望 |

### 一致性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 原始意图与授权一致 | ✅ Pass | 授权=用户 2026-10-19 批准起草 RFC（状态行+PR-1 显式未授权实施）；范围=M1 挂账 82-92 行逐项 + N1–N4 新缺口，§10 已决行声明对映；N1–N4 与挂账的重叠与新增关系在 §2.2 标注来源，随本征询提请用户确认 |
| 范围冲突 | ✅ Pass | §1.3 在/不在范围无交集（runner skip 收窄 vs CDP 改道、Esc 释放 vs OS 光标隐藏，两两分立） |
| 约束冲突 | ✅ Pass | ①~④ 无矛盾（§0 同源约束与 §6.4/§6.5 设计互证） |
| 方案漂移 | ✅ Pass | 方案 A 新增 `pointer_lock.rs` 在 zero-webview 边界内（架构边界合规），无未授权依赖 |
| CLI 语义一致 | N/A | 无 CLI 面 |
| 默认动作闭合 | N/A | 无交互分支场景 |
| 章节引用正确 | ✅ Pass | §4→§5、§6.1→§6.2、§6.4→§6.5、§6.6→§6.5、§3 FR 列→切片列逐一对得上 |
| 外部事实保守化 | ✅ Pass | spec 锚点只作语义出处；`screenX==clientX` 降级为假设（§6.3 + TBD-2）；fullscreen 退出侧降级为 FIXME（§6.8） |
| 未验证细节泄漏 | ✅ Pass | TBD-2 假设未进 FR/验收期望（FR-4 验收=上游语料自断言） |
| 场景预期泄漏 | ✅ Pass | 同上；TBD-3 侧键能力未预写通过预期（§10 记账纪律） |
| 实现来源闭合 | ✅ Pass | delta 计算=webview 自实现（§6.4）；位移字段=DomEventDetail 扩展（§6.3）；host 桥=js_dom_bridge 既有注册机制（§6.5）；shadow retarget=shim 祖先链（FR-7 行）；无新增 crate 依赖 |
| 来源-测试联动 | ✅ Pass | 上述来源均有对应测试层（§9 表行） |
| 脆弱选择逻辑覆盖 | ✅ Pass | §9 末段四项显式选中对象断言 |
| 类型分层清晰 | ✅ Pass | 需求（§3）/决策（§5 方案 A，PR-1 待决）/假设（TBD-2）/TBD（§10）分层，无混写 |
| 优先级完备 | ✅ Pass | §3 表后优先级声明段 |
| 代码边界完备 | ✅ Pass | §1.3 代码变更边界行（允许=任务范围+清单；禁止=Android/注入模式/manual 规则/`_fireDocEvent` 原函数） |
| 清单数量一致 | ✅ Pass | C 面 15=13+2（§1.1/§2.2/§8.2 步骤 8 一致）；导航四连点=4 处行号列全 |
| 依赖清单一致 | ✅ Pass | 全文零新增外部依赖，各处一致 |
| 重复失控 | ✅ Pass | grant/deny 语义主定义在 §6.1，§0/§3 为摘要+引用；无逐字重复段 |

**汇总**：26 Pass / 0 Warning / 0 Fail / 0 Skip / 0 PendingReview（PR-1
已决）/ 4 N/A
**门禁判定**：无 Fail、无阻塞 Skip、无未决 PendingReview——**本阶段
（RFC 征询）就绪并已通过**（用户 2026-10-19 确认）；实施阶段按用户裁决
待点名启动，启动时从本文萃取实施合约后按 §8 交接执行。

---

## 12. 修订历史

| 版本 | 日期 | 变更 |
|---|---|---|
| v0.1 | 2026-10-19 | 首稿：四路探索（现状实现面/dnd 管线/激活与生命周期基础设施/语料断言面）、三方案对比、方案 A 详细设计、s1–s4 切片、Lint。随用户 2026-10-19「批准启动 M2 RFC」门控放行起草 |
| v1.0 | 2026-10-19 | 用户征询确认：方案 A（PR-1 关闭）、N1–N4 范围纳入、`_fs*` 同族泄漏随 FR-10 一并修（§6.8）、实施时机另启。状态转已确认，待实施点名 |
| v1.1 | 2026-10-19 | M3 s1 实施期修正（语料钉死，B 面 2 案实跑翻绿验证）：§6.1 activation 门「消费」改「仅检查不消费」（`unadjustedMovement_update` 案一次激活连续两次请求均须过，spec request steps 无 consume 步）；§6.1 `_plOpts` 对象深相等改 `_plUnadjusted` 有效布尔相等（语料只变异单成员，最小实现）。需求/验收/切片/方案不变 |

# M3 Commit 4：C 面执行 + 桌面激活（RFC §6.4/§6.6/§6.7，步骤 7+8）

**日期**: 2026-10-19
**切片**: RFC v1.3 §8.1 Commit 4（s3 后半 C 面 + s4 前半桌面激活）
**结论**: **C 面 18 案入执行面（18/22 子测试 Pass，文件口径 14/18 全绿）；
桌面锁定转发/Esc 释放链路闭合（镜像 + 门 + IPC + renderer 臂 + 单测）**

## 行为变化

| 项 | 变化 | 依据 |
|---|---|---|
| 锁定改道键态 | `apply_locked_pointer_button` 扩签名 `(phase, button, x, y)`：mousedown/mouseup 按物理键透传 `button`/`buttons`（锁定只改 target 不改键态——back/forward mouseup 断言面）；镜像 buttons 掩码随相位置/清（`buttons_mask_bit` 与 shim `_zwButtonMask` 同映射：1→4、2→2、余 `1<<b`，界外/负值 0）；锁内 move 透传同一掩码 | RFC §6.4（实施期修正 v1.4-①） |
| 锁定改道 move 面 | `apply_locked_pointer_move` 同发 pointermove + mousemove 对（PE2 §11 compat mouse 跟随对应 pointer 事件）；不 hit test、不合成跨界序（boundary 抑制断言面保持零 enter/leave/over/out） | RFC §6.4；PE2 spec §11 |
| 锁定 click 组合 | Up 后组合 click（主键）/auxclick（非主键，UI Events）到锁目标——携 pointerType 走 PointerEvent 实例（PE2 §the-click-auxclick-and-contextmenu-events）+ R108 pre-click activation 原型链；原 FIXME 收口 | RFC §6.4；PE2 spec |
| runner 锁定路由 | move/down/up 三 op 锁定改道锁目标（§6.4 单入口，delta 镜像算）；down/up 透传已解析 `button`；skip 收窄删 `test_driver.`/`test-helper.js` 内容规则（movementX_Y_basic 例外随删自然收敛），保留 -manual + 无 harness 兜底 | RFC §6.7 |
| 桌面仲裁镜像 | TabManager `pointer_lock_tabs: HashSet<TabId>`——PointerLockChanged 回执维护（`apply_pointer_lock_reports` poll 排空点，跃迁落镜/冗余幂等）；remove_tab/navigate_request/load_html 三连清（锁不跨 document） | RFC §6.6 |
| 桌面转发门 | `page_pointer_lock_active(tab_id)` 查询面 + `dispatch_pointer_lock_move/button/exit`（fire-and-forget，ImeEvent/DragInput 同款）；`process_backend.dispatch_pointer_lock_input_fire_and_forget` | RFC §6.6 |
| 桌面指针改道 | `handle_mouse_move`：锁定态**非按压 move 也转发**（dnd 转发门邻位）+ chrome 悬停消费跳过（tooltip/链接悬停/滚动条悬停——锁定的语义即光标不可达 chrome）；`handle_mouse_click`：Down/Up 内容区落点改道页面（chrome 命中面——滚动条/选区武装/scroll-drag——整体跳过；内容区外落点按常规 chrome 处理） | RFC §6.6 |
| Esc 释放 | `handle_key` Escape 分支（dnd 取消分支邻位，同款守卫——菜单/查找栏/地址栏优先级先行）→ `PointerLockInputPhase::Exit` → renderer `execute_script("document.exitPointerLock();")`（走页面可见 API，§6.5 host 桥回环关宿主镜像） | RFC §6.6（产品合理性纳入，非语料驱动） |
| IPC 契约 | `PointerLockInputPhase` 增 `Exit` 变体（UA 手势释放相位）；roundtrip 测试补 Exit 往返 | RFC §6.5（实施期修正 v1.4-②） |

## C 面落账（2026-10-19 实跑）

- **执行面**：18 文件 / 22 子测试（-manual 7 案保留排除；分母 = 执行面）。
- **Pass 18 / Fail 4**（文件口径 14/18 全绿）：
  - `constructor.html` 1 红：`Event.NONE` 等 Event 接口 phase 常量缺失——
    **dom/events 跨域 residual**（uievents 50 已挂账），不占本 goal 分母口径
    （同 v1.6 记账）；其 pointerlockerror 字典子测试 Pass。
  - `pointerlock_shadow.html` 1 红：`createTestTree is not defined`——TBD-1
    （shadow-dom.js 测试资产未随 pin 快照落地），能力缺口记账。
  - `pointerlock_wheel_event_outside_target.html` 2 红：runner 无 `Actions.scroll`
    合成——TBD-3 记账。
- **本轮翻绿**（对比 s2 时点 4/5）：movementX_Y_basic 之外的 C 面 13 案首次
  实跑，`pointerlock-maintains-mousedown`（buttons 键态）、
  `pointerlock_all_mouse_events_to_locktarget`（click 组合 + 独占 retarget）、
  `pointerlock_suppress_mouse_boundary_events_when_locked`（pointermove 同发
  + 零 boundary）、`mouse_buttons_back_forward`（button 穿透——首跑 Timeout
  暴露签名缺 button，修正后绿）全部翻绿；`mouse_buttons_back_forward` 在
  s2 时点曾走合成管线意外 Pass，锁定路由落地后由 button 穿透修复（非回归）。

## 验证

- **webview 模块单测 7 绿**（新增 `locked_pointer_button_move_sequence_e2e`）：
  锁内主键 Down→move→Up + back 键序钉——`mousedown:t:1|pointermove:t:1|
  mousemove:t:1|mouseup:t:0|click:t:0|mousedown:t:8|mouseup:t:0|auxclick:t:0`
  （buttons 掩码 + pointer/mouse 派发序 + click/auxclick 组合 + 独占锁目标
  retarget，另一元素 #o 零事件）。
- **browser 镜像单测绿**（`pointer_lock_mirror_follows_reports_and_cleanup`）：
  回执跃迁落镜、冗余幂等、导航清理关镜。
- **协议 roundtrip**：PointerLockInput（Move + Exit 相位）/PointerLockChanged
  往返绿。
- **C 面实跑**：`make testharness-pointerlock` 18P/4F（上表）。
- **零回归**：make test 全绿（0 FAILED）；clippy 五 crate lib+tests 0 告警；
  fmt 干净；make reftest 零回归（DC-3）。
- **桌面 GUI 手验受限**：WSL2 无显示环境，转发/Esc 链路以单测 + 代码路径
  覆盖（镜像生命周期/门判据/IPC 契约），真窗口手验留待桌面环境。

## 未决

- TBD-1（shadow 资产）/TBD-3（Actions.scroll）/TBD-4（OS 光标隐藏）维持
  RFC §10 记账。
- 桌面 wheel 锁定转发（§6.6 wheel 子句）：webview 无 DOM wheel 派发底座
  （桌面滚动走 offset 不走 wheel 事件），建管线无验证面（corpus wheel 案
  本身 TBD-3）——与 TBD-3 同域挂账，不在本轮建无消费者管线。
- Commit 5 = s4 收口：corpus 总账（csv 回填）、FR-7/8/9 residuals、DC-3 门禁。

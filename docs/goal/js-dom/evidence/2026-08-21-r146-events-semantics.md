# R146 — events 语义收口三件（Event-dispatch-other-document / Event-propagation / EventListenerOptions-capture 3F→0F 双路径）

**日期**: 2026-08-21
**里程碑**: M4（WPT dom 上游基线扩展）
**驱动用例**: `dom/events/Event-dispatch-other-document.html`（1 subtest）+
`dom/events/Event-propagation.html`（7 subtest 中 1F）+
`dom/events/EventListenerOptions-capture.html`（4 subtest 中 1F）

## 根因与修复（三件独立，全在 shim 层）

### ① Event-dispatch-other-document：_zwMEl dispatchEvent 缺 target/srcElement

用例：detached doc（`createHTMLDocument`）的 `createElement('div')` 产物注册
listener 后 `dispatchEvent`，断言 `ev.target === element` / `ev.srcElement === element`。
`_zwMEl` 节点（part03 R99 本地 dispatchEvent）listener 触发正常但 **ev.target/srcElement
未设**（均 null）。修复：派发前 own-set `ev.target = node; ev.srcElement = node`
（spec `dom-event-dispatch` 派发前设 target 为本节点；own-set 覆盖构造器 data 属性——
native 形态同 R138 srcElement 手法）。

### ② Event-propagation：dispatch 末步无条件清 stop/immediate flag

用例：`stopImmediatePropagation()` 后**第一次** dispatch 零触发（✓ 旧已对），**第二次**
dispatch 应恢复触发（✗ 旧仍零触发——"Propagation flag after first dispatch expected
true got false"）。根因：dispatch 结束清理是**条件性**的（「仅清 dispatch 内设的 flag；
监听器外显式 stopPropagation 保留至 initEvent」），但 spec `concept-event-dispatch`
末步 unset stop propagation / stop immediate propagation flag **无此限定**。修复：
无条件清 `_propagationStopped` + `_immediateStopped` + `__zw_stop` + `__zw_stop_immediate`。
dispatch 前的零触发语义由 dispatch 开始时的 flag 检查（R39）保证，不受影响
（Event-dispatch-propagation-stopped / Event-stopPropagation-cancel-bubbling /
Event-cancelBubble 复跑全 Pass）。**附带发现**：`_immediateStopped` 不清时 bubble
循环的 `!immediateStopped()` 前置守卫会被 stale flag 跳过全部 non-capture listener
——单独清 `_propagationStopped` 不够。

### ③ EventListenerOptions-capture：第三参 WebIDL boolean 转换

用例：`addEventListener('test', h, 2.3)` 期望 CAPTURING_PHASE（WebIDL `boolean`
转换 `Boolean(2.3)` = true）。`_optCapture` 旧实现 `opts === true ||
(opts && opts.capture)`：primitive `2.3` 落到 `opts.capture`（number 无 .capture
→ undefined）误判 false。修复：`null → false` / 非 object → `Boolean(opts)` /
object → `Boolean(opts.capture)`。四个调用面（part03 window / part06 document /
part04 element ×2）共享同一 helper，单点修复全覆盖。

## A/B 结果（polyfill / native 双路径）

| 套件 | 结果 |
|---|---|
| 三驱动文件 | **全 subtest Pass 双路径**（other-document 1P、propagation 7P、capture 4P） |
| dom/events 全量 | polyfill **421P/21F** / native **422P/21F**（fail 集两路径逐文件一致）；vs R145 419P/24F：净 +2P/-3F（三文件消失） |
| dom/nodes 全量 | 5473P/230F——fail 集 85 文件与 R145 **逐文件一致**零回归 |
| traversal / collections | 50P / 0F |
| `make test` | 66 套件全绿 |
| fmt / clippy | 零 diff / 零警告（v8 + quickjs 双矩阵） |

## 单元测试（`js_dom_bridge_tests/part21.rs` 新文件——part20 已 2217 行超 2000 上限）

- `test_mel_dispatch_event_target_r146`：detached doc 元素 dispatchEvent 的
  target/srcElement identity 断言
- `test_dispatch_clears_stop_flags_for_redispatch_r146`：stopPropagation/
  stopImmediatePropagation 各两次 dispatch 的零触发→恢复触发四段断言
- `test_opt_capture_webidl_boolean_r146`：2.3/-1000.3/NaN/0/''/'AAAA'/null/
  {capture:2}/{capture:0} 九形态 phase 断言

## 未收口项（记入下一步）

- Event-dispatch-redispatch：需 isTrusted 语义（host 派发事件 = trusted；JS 重派发
  = untrusted）——事件可信度模型是 broader 设计，本轮不碰。
- shadow-relatedTarget：需 focus 事件 relatedTarget（retargeting 到 shadow host）+
  `_zwMElFocused` 与 `_activeElKey` 双焦点态统一——中等深改面，判后续专项。

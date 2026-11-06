---
date: 2026-11-06
modules: engine
---

# WebSocket fail 面 error 派发：err 标记位与派发门共用 flag 导致派发永久跳过

## 问题

`crates/engine/src/js_dom_shim/part06.js` WebSocket 泵把「err wire 先到」
改为「close wire 统一派 error」（保证 fail 面原子序 readyState 先 CLOSED
再 error → close，events/015 断言面）时，err 臂置位了恰一次门
`_errorDispatched = true`，close 臂门写的是 `if (!clean && !_errorDispatched)`。
宿主协议错误序为 err wire → close(1006, clean=false)（ws_bridge 泵 Err 臂
契约：err 只标记错误、close 承载终结），err 臂置位后 close 臂
`!_errorDispatched` 恒 false——error 永不派发，events/018 的 errorFired
断言失败。

## 根因

一个 flag 承担了两个语义：「错误已发生」（err wire 证据）与「error 事件
已派发」（恰一次门）。标记动作把派发门也一并关死。凡「先标记、终结时统一
派发」的 wire 事件对（err→close、以及任何 deferred dispatch 模式）都必须
拆开：标记位记录证据，派发位只由实际派发动作置位。

次要教训：**案面 P 不证明 listener 派发**。events/015 的全部断言位于
onerror listener 内部，listener 不触发时断言不执行、onclose `t.done()`
照常收案——015 通过曾误导排查方向（以为 error 派发面工作正常）。判据：
断言在 listener 内部的案面对「listener 未被调用」不敏感，派发验证须选
终结回调里断 flag 的案面（如 018 的 `assert_true(errorFired)`）。

## 解决

拆双标记：`_zwErrSeen`（err 臂置位，证据）与 `_errorDispatched`（仅派发
动作置位，恰一次门），close 臂 `!dispatched && (!clean || errSeen)`。
CONNECTING close 宿主仲裁序（单 close wire clean=false、无 err wire）走
`!clean` 支路不受影响。

## 避免方式

- 改 wire 事件对派发时序前，先核对宿主侧（`ws_bridge.rs`）err/close 的
  真实序与 clean flag 值，不凭「协议错误应该 unclean」假设推演。
- 涉及事件派发的修复，验证案面必须含「终结回调断 flag」形态；只靠
  listener 内断言的案面通过不构成派发证据。

# M4 Slice R29 — Event.cancelBubble setter dispatch 止上溯副作用

**日期**: 2026-08-14
**里程碑**: M4 — WPT dom 上游基线 + 按聚类驱动修复
**切片**: R29
**前置**: R28（dom/events polyfill 49.68% / native 43.18%，双路径差 6.49pp）

## 切片选择（决策记录）

R26 已加 polyfill Event.cancelBubble 公开镜像（`_makeEvent` 加 `cancelBubble: false` 字段 + stopPropagation/stopImmediatePropagation 设 true + initEvent 重置），覆盖 WPT Event-cancelBubble.html 4 个值断言（初始/initEvent/stop/stopImmediate）。但**剩 4 test** 测 cancelBubble 的 setter dispatch 语义：

- `Event.cancelBubble=false must have no effect`（stopPropagation 后设 false 应 no-op）
- `Event.cancelBubble=false must have no effect during event propagation`（dispatch 内同理）
- `cancelBubble must be false after an event has been dispatched`（spec concept-event-dispatch 步骤14 dispatch 后 unset stop propagation flag）
- `Event.cancelBubble=true must set the stop propagation flag`（外部设 true 应等同 stopPropagation 止上溯）

R26 的 cancelBubble 是**普通 data 属性**——`ev.cancelBubble = true` 直接赋值，setter 无副作用，dispatch bubble 循环只读 `_propagationStopped` 不读 cancelBubble，故外部设 true 不止上溯。本切片升级 cancelBubble 为带 dispatch 副作用的 setter。

## 修复

polyfill（`crates/engine/src/js_dom_shim/`）：

### part03 `_makeEvent` — cancelBubble 改 defineProperty getter/setter

R26 普通 data 字段升级为 `Object.defineProperty`，**后端直接复用 stop propagation flag `_propagationStopped`**（spec cancelBubble 即 stop propagation flag 的 legacy 公开别名，二者同一 flag）：

- **getter**: `return this._propagationStopped;`（flag 状态）
- **setter true**: `this._propagationStopped = true;`（spec cancelBubble setter true → set stop propagation flag，等同 stopPropagation；_dispatchWithBubble capture/target/bubble 三循环均读此 flag，故外部设 true 经此止上溯）
- **setter false**: no-op（spec：stop propagation flag 一旦设除非 initEvent 重新初始化否则不可清——WPT "cancelBubble=false must have no effect"）

与 R28 Event.returnValue 同款「defineProperty getter/setter + 私 flag 后端」模式。

### part03 stopPropagation / stopImmediatePropagation

移除冗余 `this.cancelBubble = true;`（R26 显式赋值）——R29 getter 读 `_propagationStopped`，stopPropagation 已设此 flag，无需再设 cancelBubble（getter 自返 true）。

### part03 `_dispatchWithBubble` finally — dispatch 后重置 flag

`try/finally` 的 finally 加 `event._propagationStopped = false;`（spec `concept-event-dispatch` 步骤14——dispatch 结束 unset stop propagation flag）。reset 后 cancelBubble getter（后端 _propagationStopped）返 false（WPT "cancelBubble must be false after an event has been dispatched"）。仅清 dispatch 内设的 flag；监听器外显式 stopPropagation（未 dispatch）的 flag 保留至 initEvent 重置。

### part05 initEvent

移除 R26 的 `this.cancelBubble = false;`（R29 setter false = no-op，无效），保留 `this._propagationStopped = false;` 重置（getter 据此返 false）。

## 验证

- **单测** `test_event_cancel_bubble_mirror_r26`（part07.rs）扩展为 R26 4 + R29 3 = 7 场景：① 初始 false；② initEvent 设 false；③ stopPropagation 设 true；④ stopImmediatePropagation 设 true；⑤ cancelBubble=false 设值 no-op（stopPropagation 后）；⑥ cancelBubble=true setter 置 flag；⑦ dispatch 后 flag 清（监听器内 stopPropagation，dispatch finally reset）。v8 pass。
- **fmt + clippy 双矩阵**：zero-engine v8 + quickjs 零警告。
- **Event-cancelBubble.html 双路径**：4P→**8P/8（100%）**（剩 4 全过：setter=false no-op / dispatch 内 no-op / dispatch 后 flag 清 / setter=true dispatch bubble+capture 止上溯）。
- **dom/events 全量双路径**（完整 JSON 入 evidence）：

  | 路径 | R28 | R29 | Δ |
  |---|---|---|---|
  | Polyfill | 49.68%（153P） | **52.13%（159P）** | +2.45pp / +6P |
  | Native | 43.18%（133P） | **45.57%（139P）** | +2.39pp / +6P |
  | 双路径差 | 6.49pp | 6.56pp | ~不变（双路径同步 +6，polyfill shim 共享） |

  双路径各 +6 pass（cancelBubble setter 副作用 + dispatch flag reset），对等差基本不变（+0.07pp，polyfill shim 双路径共享同步受益）。
- **engine 单测**：v8 2097 / quickjs 1410 全绿，零回归（dispatch flag reset 经 dom/events 净 +6 证明无 dispatch-post 依赖回归）。

## 决策记录

- **cancelBubble 与 _propagationStopped 合一**：spec 中 cancelBubble getter 返 stop propagation flag，setter true 设此 flag——二者本就同一 flag。R26 分两个字段（公开 data + 私 flag）需手动同步。R29 直接让 getter/setter 后端复用 `_propagationStopped`，消除同步负担，且 dispatch 循环无需新增 cancelBubble 检查（已读 `_propagationStopped`）。
- **dispatch flag reset 仅在 _dispatchWithBubble**：`EventTarget.prototype.dispatchEvent`（part05，detached EventTarget 单节点派发）不重置 `_propagationStopped`——它无祖先链，stopPropagation 语义主要 no-op，且 WPT cancelBubble 测的是嵌套 DOM（经 _dispatchWithBubble）。reset 局限于 _dispatchWithBubble finally 避免污染单节点路径。

## 残留（转 R30+）

- **Event-dispatch 系列**（深结构，~33 个 0-pass 主力）：document/window listener 独立存储 + document.cloneNode / new Document / new Text / createHTMLDocument detached doc 基础设施 + capture/bubble 含 document/window 链。
- **Event-dispatch-multiple-cancelBubble.html**：dispatch 内多次 cancelBubble 状态机，部分依赖 Event-dispatch 深结构。
- 双路径差 6.56pp 收口（WheelEvent 子类链/SubclassedEvent，分散低 ROI）。
- iframe.contentDocument / querySelector-mixed-case（dom/nodes 域）。

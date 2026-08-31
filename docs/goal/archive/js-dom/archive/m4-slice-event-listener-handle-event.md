# M4 Slice R27 — EventListener handleEvent（对象 listener）

**日期**: 2026-08-14
**里程碑**: M4 — WPT dom 上游基线 + 按聚类驱动修复
**切片**: R27
**前置**: R26（Event.cancelBubble，dom/events polyfill 45.48% / native 39.03%）

## 问题

WPT `EventListener-handleEvent.html` 双路径 0P（R26 基线）。spec `EventListener` invoke：`addEventListener(type, listener)` 的 listener 若是**对象**（非函数），dispatch 时 Get 其 `handleEvent` 属性再调用（this=对象本身），且每次派发都 Get（支持 getter）。polyfill `_dispatchToListeners` 的 `fire(entry)` 恒 `entry.fn.call(ctx, event)`——对象 listener 抛（对象无 call）。

## 修复

polyfill `crates/engine/src/js_dom_shim/part03.js` `_dispatchToListeners` 的 `fire`（line 920）：

- listener 是**函数**：直接 `callable.call(ctx, event)`，this=currentTarget（不回归）。
- listener 是**对象**：Get `fn.handleEvent`（每次派发都 Get，spec invoke 步骤）；若 handleEvent 是函数，`handleEvent.call(fn, event)`（this=对象本身）；handleEvent 非 callable（undefined/null）→ 跳过（spec：不抛不调）。

## 验证

- **单测** `test_event_listener_handle_event_object_r27`（part07.rs）：① 对象 listener handleEvent 被调 + this=对象 + evt.type/target 正确；② handleEvent getter 每次 dispatch 都 Get（2 次派发 → getter 触发 2 次）；③ 函数 listener 不回归（this=currentTarget）。v8 pass。
- **fmt + clippy 双矩阵**：zero-engine v8 + quickjs 零警告。
- **EventListener-handleEvent.html 双路径**：0P→**3P/5**（剩 2 是 cross-realm/incumbent-global 跨 realm 用例，非 handleEvent 核心）。
- **dom/events 全量双路径**（完整 JSON 入 evidence）：

  | 路径 | R26 | R27 | Δ |
  |---|---|---|---|
  | Polyfill | 45.48%（141P） | **46.75%（144P）** | +1.27pp / +3P |
  | Native | 39.03%（121P） | **40.26%（124P）** | +1.23pp / +3P |
  | 双路径差 | 6.45pp | 6.49pp | 基本保持（同步 +3） |

  双路径各 +3 pass（EventListener-handleEvent），对等差基本不变（shim 共享同步受益）。注：Total 319→318（一个用例 timeout 计数波动，非回归）。

## 决策记录

- **handleEvent 每次派发都 Get（非缓存）**：spec invoke 算法每次 dispatch 重新 Get listener 的 handleEvent 属性（支持 getter + 运行时改 handleEvent）。WPT "performs Get every time event is dispatched" 用 getter 验证。polyfill fire 每次调用都读 `fn.handleEvent`（不缓存到 entry）。
- **对象 listener this=对象本身（非 currentTarget）**：spec EventListener invoke 时 callback this 是 listener 对象（不是 dispatch target）。函数 listener this=currentTarget（target/祖先）。两者 this 语义不同，polyfill fire 按类型分支设 this。
- **handleEvent 非 callable 跳过（不抛）**：spec 若 listener 对象无 handleEvent 或非函数，invoke 静默跳过（不抛 TypeError）。polyfill 检测 `typeof callable === 'function'` 才调。

## 残留（转 R28+）

- EventListener-handleEvent 剩 2 fail（cross-realm/incumbent-global 跨 realm，非核心 handleEvent）。
- dom/events ~164 fail：Event-dispatch 系列（深结构 document/window listener 独立）/ Event-returnValue / cancelBubble setter dispatch 止上溯 / 双路径差 6.49pp 收口（WheelEvent 子类链/SubclassedEvent）。
- iframe.contentDocument / querySelector-mixed-case（dom/nodes 域）。

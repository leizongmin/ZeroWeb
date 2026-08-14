# M4 Slice R28 — Event.returnValue（defaultPrevented legacy 镜像）

**日期**: 2026-08-14
**里程碑**: M4 — WPT dom 上游基线 + 按聚类驱动修复
**切片**: R28
**前置**: R27（EventListener handleEvent，dom/events polyfill 46.75% / native 40.26%）

## 问题

WPT `Event-returnValue.html` 双路径 0P（R27 基线）。spec `Event.returnValue`（legacy IE 别名 = `!canceled flag`）：① 初始 true；② `preventDefault()` 仅 cancelable 时设 false；③ `returnValue = false` 仅 cancelable 时触发 prevent；④ `initEvent` 重置 true；⑤ `returnValue = true` 已 canceled 后 no-op。polyfill event 对象缺 returnValue。

## 修复

polyfill `crates/engine/src/js_dom_shim/part03.js` `_makeEvent`：

- **`Object.defineProperty(ev, 'returnValue', ...)`**：getter 返 `!this._defaultPrevented`（canceled flag 反向镜像）；setter 仅当 `!v && this.cancelable` 时触发 prevent（设 defaultPrevented/_defaultPrevented），设 true 或 cancelable=false 均 no-op。
- 用 defineProperty（getter/setter，非普通 data 属性）——setter 需触发 preventDefault 副作用。
- preventDefault（既有）已设 _defaultPrevented → returnValue getter 自动反映。initEvent（R26 加）重置 _defaultPrevented=false → returnValue 自动 true。

## 验证

- **单测** `test_event_return_value_mirror_r28`（part07.rs）：7 个 spec 场景全覆盖——① 初始 true；② preventDefault(cancelable=false) 不改；③ preventDefault(cancelable=true) 设 false；④ returnValue=false(cancelable=true) 触发 prevent；⑤ returnValue=false(cancelable=false) no-op；⑥ initEvent 重置 true；⑦ returnValue=true 已 canceled 后 no-op + defaultPrevented 保持。v8 pass。
- **fmt + clippy 双矩阵**：zero-engine v8 + quickjs 零警告。
- **Event-returnValue.html 双路径**：0P→**7P/7（100%）**。
- **dom/events 全量双路径**（完整 JSON 入 evidence）：

  | 路径 | R27 | R28 | Δ |
  |---|---|---|---|
  | Polyfill | 46.75%（144P） | **49.68%（153P）** | +2.93pp / +9P |
  | Native | 40.26%（124P） | **43.18%（133P）** | +2.92pp / +9P |
  | 双路径差 | 6.49pp | 6.49pp | 保持（同步 +9） |

  双路径各 +9 pass（Event-returnValue 7 + 联动 2），polyfill 突破 49.68% 接近 50%。对等差不变（shim 共享同步受益）。

## 决策记录

- **returnValue 用 getter/setter 而非 data 属性**：spec returnValue=false 需触发 preventDefault 副作用（设 canceled flag），普通 data 属性 `ev.returnValue = false` 无法触发。getter/setter（defineProperty）使赋值走 set 逻辑。getter 返 `!_defaultPrevented` 自动与 preventDefault/initEvent 联动（无需手动同步两处）。
- **returnValue=true 永远 no-op**：spec canceled flag 一旦设不可清（preventDefault 不可撤销）。setter 仅处理 `!v`（false）分支，设 true 直接忽略。cancelable=false 时即使设 false 也 no-op（WPT "no effect if cancelable is false"）。
- **复用 _defaultPrevented（不新增 _canceled）**：canceled flag = defaultPrevented 语义，returnValue getter 直接返 `!_defaultPrevented`，避免双 flag 同步漂移。

## 残留（转 R29+）

- dom/events ~155 fail：Event-dispatch 系列（深结构 document/window listener 独立）/ cancelBubble setter dispatch 止上溯 / 双路径差 6.49pp 收口（WheelEvent 子类链/SubclassedEvent）。
- iframe.contentDocument / querySelector-mixed-case（dom/nodes 域）。

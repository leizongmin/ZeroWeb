# M4 Slice R23 — Event eventPhase 常量（NONE/CAPTURING_PHASE/AT_TARGET/BUBBLING_PHASE）

**日期**: 2026-08-14
**里程碑**: M4 — WPT dom 上游基线 + 按聚类驱动修复
**切片**: R23
**前置**: R22（native Event.timeStamp 死循环修复，dom/events 双路径基线 31.61%/31.29%）

## 问题

WPT `dom/events/Event-constants.html` 双路径 0P/16（R22 基线）。用例 `testConstants` 检查 4 对象（Event 接口 / Event.prototype / createEvent('Event') 实例 / createEvent('CustomEvent') 实例）各有 `NONE`(0)/`CAPTURING_PHASE`(1)/`AT_TARGET`(2)/`BUBBLING_PHASE`(3) eventPhase 常量。polyfill Event 构造器 + prototype 缺这些 spec 常量（`Event.NONE` / `Event.prototype.NONE` 等未定义 → undefined → 断言 fail）。

这是 dom/events 失败聚类「Event 对象缺属性」的子集（Event-constants.html 独立用例，spec DOM `Event` 接口的静态 + 原型常量）。

## 修复

polyfill `crates/engine/src/js_dom_shim/part05.js` Event 构造器定义后，挂 eventPhase 常量：

- **接口对象（Event 构造器，静态常量）**：`Object.defineProperty(Event, 'NONE', {value:0, enumerable:false})` 等 4 个。
- **Event.prototype（实例经原型链继承）**：同款 4 常量。
- `enumerable:false`（与 DOM 原型方法不可枚余一致，R10——避免 for-in 污染 expando）；`guard 幂等`（`if (!(k in ...))`）。
- 实例继承：`createEvent('Event')` → `new Event('')` → `setPrototypeOf(Event.prototype)`；`createEvent('CustomEvent')` → CustomEvent.prototype = Object.create(Event.prototype) → 链继承。

**native 路径**：用例 document 是 polyfill（R9），createEvent 经 polyfill，故 polyfill 补常量即让双路径用例过。native Event prototype 常量作为 default-on 后生产能力对齐（后续）。

## 验证

- **单测** `test_event_constants_none_capturing_at_target_bubbling_r23`（part07.rs）：4 对象（Event/Event.prototype/createEvent('Event')/createEvent('CustomEvent')）× 4 常量 = "0,1,2,3" × 4 + 常量不可枚举（Object.keys(Event.prototype) 不含）。v8 pass。
- **fmt + clippy 双矩阵**：zero-engine v8 + quickjs 零警告。
- **Event-constants.html 双路径**：0P/16 → **4P/4（100%）**（testharness 聚合 4 对象为 4 test）。
- **dom/events 全量双路径**（完整 JSON 入 evidence）：

  | 路径 | R22 | R23 | Δ |
  |---|---|---|---|
  | Polyfill | 31.61%（98P/212F） | **32.90%（102P/208F）** | +1.29pp / +4P |
  | Native | 31.29%（97P/213F） | **32.58%（101P/209F）** | +1.29pp / +4P |
  | 双路径差 | 0.32pp | 0.32pp | 对等保持 |

## 决策记录

- **常量挂 prototype 而非每实例**：spec 常量是 Event 接口 + prototype 属性，实例经原型链继承。挂 prototype 一次覆盖所有实例（Event/CustomEvent/MouseEvent 等子类链 Event.prototype 均获得），避免 `_makeEvent` 每实例重复设。
- **为何只补 polyfill 不补 native event.rs**：R9 发现用例 document 是 polyfill（即使 native_dom=1），createEvent('Event') 经 polyfill 路径。polyfill 补常量即让 WPT 用例过（双路径基线提升）。native Event prototype 常量是 default-on 后（M5）的生产路径合规项，非当前基线驱动。
- **enumerable:false**：与 R10 DOM 原型方法不可枚余一致。WPT 用例虽不直接测可枚举性，但 for-in 遍历 Event 实例的其他用例（如 Event-subclasses）若枚举到常量可能误判。防御性 spec 一致。

## 残留（转 R24+）

- dom/events 仍有 ~208 fail / ~50 个 0-pass 用例：
  - Event-subclasses-constructors：UIEvent 缺 `view`、MouseEvent 缺 `ctrlKey` 等子类 init dict 属性（R24 候选）。
  - Event-cancelBubble：cancelBubble setter 语义。
  - 三阶段分发 capture/bubble/stopPropagation（Event-dispatch 系列）。
  - EventListener handleEvent。
- iframe.contentDocument / querySelector-mixed-case（dom/nodes 域）。

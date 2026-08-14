# M4 Slice R24 — 事件子类 init 属性父链继承 + KeyboardEvent 工厂化

**日期**: 2026-08-14
**里程碑**: M4 — WPT dom 上游基线 + 按聚类驱动修复
**切片**: R24
**前置**: R23（Event eventPhase 常量，dom/events polyfill 32.90% / native 32.58%）

## 问题

WPT `Event-subclasses-constructors.html` 双路径 0P（R23 基线）。用例 `assert_props` **递归检查父链**：MouseEvent（extends UIEvent）实例须有 UIEvent 的 `view`/`detail`；WheelEvent（extends MouseEvent extends UIEvent）须有三层父链属性。

polyfill `_defineEventSubclass` 构造器只设**子类自身 props**（line 2619 `for props`），不设父链 props → MouseEvent 实例缺 UIEvent 的 view/detail → WPT assert_props 父链检查 fail。KeyboardEvent 是独立实现（仅设 key/code + extends Event 非 UIEvent），缺 location/repeat/isComposing/charCode/keyCode/which + 修饰键 + UIEvent 父链。

## 修复

polyfill `crates/engine/src/js_dom_shim/part05.js`：

1. **`_eventSubclassProps` 注册表**：记录每个子类 `[ownProps, parentName]`，工厂注册时写入。
2. **`_defineEventSubclass` 构造器沿父链收集**：构造时 `while` 沿 parentName 链收集所有祖先 props（自身先、父类后，子类覆盖父类 spec 一致），逐属性从 init dict 设值（`o[p[1]] != null ? o[p[1]] : p[2]`，null/undefined 用默认）。
3. **KeyboardEvent 改用工厂**（extends UIEvent）：补全 EventModifierInit（ctrlKey/shiftKey/altKey/metaKey）+ key/code/location/repeat/isComposing/charCode/keyCode/which + getModifierState（复用 MouseEventCtor）。旧独立实现删除。

覆盖 Event/UIEvent/FocusEvent/MouseEvent/WheelEvent/KeyboardEvent/CompositionEvent 的 init dict 属性 + 默认值 + 父链继承。

## 验证

- **单测** `test_event_subclass_init_props_inherit_parent_chain_r24`（part07.rs）：① MouseEvent 默认值（view=null/detail=0 父链 + ctrlKey=false/screenX=0 自身）；② MouseEvent 设定值（view=window/detail=7/ctrlKey=true/screenX=40）；③ KeyboardEvent 默认（key=''/code=''/location=0/...+view=null/detail=0 父链）；④ WheelEvent 三层父链（UIEvent view/detail + MouseEvent ctrlKey/screenX + 自身 delta）。v8 pass。
- **fmt + clippy 双矩阵**：zero-engine v8 + quickjs 零警告。
- **Event-subclasses-constructors.html**：polyfill 0P→**42P/49**，native 0P→**24P/49**。
- **dom/events 全量**（完整 JSON 入 evidence）：

  | 路径 | R23 | R24 | Δ |
  |---|---|---|---|
  | Polyfill | 32.90%（102P） | **42.58%（132P）** | **+9.68pp / +30P** |
  | Native | 32.58%（101P） | **36.13%（112P）** | +3.55pp / +11P |
  | 双路径差 | 0.32pp | **6.45pp** | 扩大（native 落后，见决策） |

## 决策记录

- **父链收集而非每子类重复声明父属性**：注册表 `_eventSubclassProps` 记录每子类 own props + parent，构造器沿链收集。避免每子类手动列父属性（DRY + 易漏），且新增子类自动继承。链深度 guard（<32 防环）。
- **KeyboardEvent 改用工厂**：旧独立实现（仅 key/code + extends Event）与 polyfill 工厂不一致。改用 `_defineEventSubclass('KeyboardEvent','UIEvent',...)` 后自动继承 UIEvent 父链 + 全属性 + getModifierState 复用 MouseEvent。
- **native 对等差扩大到 6.45pp（转 R25）**：native dom_bindings event.rs 的 MouseEvent/KeyboardEvent/WheelEvent 构造器仍用旧实现，缺父链继承（WheelEvent ctrlKey）+ KeyboardEvent which/key + MouseEvent instanceof + UIEvent view 校验。R24 仅修 polyfill（+30P 巨大净正），native 对等是独立子切片（R25，dom_bindings event.rs 多点缺口）。polyfill 基线主导（用例 document 是 polyfill，R9），native 是 default-on 后生产路径合规项。

## 残留（转 R25+）

- **R25 native 事件子类构造器对齐**（dom_bindings event.rs）：WheelEvent 父链 ctrlKey、KeyboardEvent which/key 默认、MouseEvent instanceof 原型链、UIEvent view 非 window 抛 TypeError 校验。缩双路径差 6.45pp。
- dom/events 仍有 ~178 fail（polyfill）：三阶段分发 capture/bubble/stopPropagation（Event-dispatch 系列）、EventListener handleEvent、Event-cancelBubble setter。
- SubclassedEvent（用户 `class extends Event`）native 下 instanceof fail——class 语义，独立评估。
- iframe.contentDocument / querySelector-mixed-case（dom/nodes 域）。

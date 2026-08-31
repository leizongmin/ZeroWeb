# M4 Slice R25 — native MouseEvent/KeyboardEvent view + KeyboardEvent which

**日期**: 2026-08-14
**里程碑**: M4 — WPT dom 上游基线 + 按聚类驱动修复
**切片**: R25
**前置**: R24（polyfill 事件子类父链继承，dom/events polyfill 42.58% / native 36.13%，双路径差 6.45pp）

## 目标

缩 R24 引入的双路径差 6.45pp——对齐 native 事件构造器（dom_bindings event.rs）。

## 诊断（runner 实测，关键）

经临时诊断用例（runner 环境 `MouseEvent.toString().includes('[native code]')`）确认：**native_dom=1 时 `new MouseEvent()` 走 native MouseEvent**（native 覆盖 polyfill globalThis.MouseEvent）。R24 单测 `run_script` 只装 native 不装 polyfill，诊断隔离了 native 真实能力：

- native MouseEvent：`instanceof MouseEvent/Event` ✓，`ctrlKey/detail/isTrusted in` ✓，但 **`view` not in** ✗。
- native KeyboardEvent：`which` not in ✗，key 默认 "" ✓。
- native 无 WheelEvent/UIEvent 等构造器（用例 `new WheelEvent()` 走 polyfill，但父 = native MouseEvent）。

## 修复（dom_bindings event.rs）

native 事件构造器属性补全：

- **`set_ui_view` helper**：设 UIEvent.`view`（spec WindowProxy 或 null，init dict `view` 字段，缺省/undefined/null → null，对象原样用）。MouseEvent/KeyboardEvent extends UIEvent，WPT Event-subclasses-constructors `assert_props` 父链检查 `'view' in event`。
- **MouseEvent 构造器**：set_event_init 后调 `set_ui_view`（R25）。
- **KeyboardEvent 构造器**：`set_ui_view`（R25）+ `which`（缺省回退 keyCode，spec legacy）。

诊断复测（runner NATIVE 路径）：`M_view=true / K_which=true`（R25 修复生效）。

## 验证

- **单测** `native_event_view_and_which_r25`（tests.rs）：MouseEvent view（缺省 null + init dict window）+ KeyboardEvent view + which（缺省 0 回退 keyCode + init dict 显式 42）。v8 pass。
- **fmt + clippy 双矩阵**：zero-engine v8 + quickjs 零警告。
- **dom/events 双路径差未缩**（诚实记录）：Event-subclasses-constructors native 仍 24P/49（R25 view/which 修了 view/which，但剩余多点缺口未解）。dom/events native 36.13% 保持（polyfill 42.58% 保持）。

## 双路径差未缩的根因（转后续，非本切片）

R25 修了 native MouseEvent/KeyboardEvent 的 view/which，但 Event-subclasses-constructors native 仍 24P，剩余分散缺口：

1. **WheelEvent ctrlKey**（polyfill 子类链断）：native 无 WheelEvent 构造器，polyfill `_defineEventSubclass('WheelEvent','MouseEvent')` 的父 = native MouseEvent（不在 polyfill `_eventSubclassProps` 注册表）→ 父链收集断 → WheelEvent 实例缺 MouseEvent 的 ctrlKey/screenX。polyfill 路径 W_ctrlKey=true 是因 polyfill MouseEvent 在注册表。
2. **MouseEvent/KeyboardEvent "expected true got false"**：assert_props 里某属性 `in` 检查 fail（view 已修，可能是 screenY/pageY/movementX native 未设全，需进一步定位）。
3. **SubclassedEvent**（用户 `class extends Event`）：native 下 instanceof fail（class 语义 + native Event 构造器交互）。
4. **UIEvent view:7 抛 TypeError**：spec view 非 window 应抛（native 未校验）。
5. **KeyboardEvent key=""**：某变体设定值 fail。

这些是 native 路径多点分散缺口，单切片难全解。R25 view/which 是 native 正确性净正（单测证明，default-on 后合规），land。

## 决策记录

- **为何 land R25（双路径差未缩）**：view/which 修复是 native MouseEvent/KeyboardEvent 的 spec 正确性改进（单测验证），net 正（无回归），default-on 后（M5）native 为唯一生产路径时合规。双路径差缩降受多点分散缺口阻碍（WheelEvent 子类链/SubclassedEvent class 语义等），ROI 降低，转更高 ROI 切片（polyfill 三阶段分发 ~44 个 0-pass 主力）。
- **诊断方法论**：R25 经 runner 实测诊断（`MouseEvent.toString()` 探 native/polyfill + forced-fail message 带属性状态）确认 native 覆盖 polyfill + view/which 缺口，比 R24 单测 `run_script`（隔离 native）更接近真实。后续 native 缺口诊断复用此法。

## 残留（转 R26+）

- **双路径差 6.45pp**（dom/events native 36.13% vs polyfill 42.58%）：WheelEvent 子类链 / SubclassedEvent / MouseEvent 属性细节 / UIEvent view 校验。低 ROI（分散），按需推进。
- dom/events polyfill ~178 fail 主力：**三阶段分发 capture/bubble/stopPropagation**（Event-dispatch 系列 ~44 个 0-pass，R26 高 ROI 候选）+ EventListener handleEvent + Event-cancelBubble。
- iframe.contentDocument / querySelector-mixed-case（dom/nodes 域）。

# M6 S4q 完整化 — Event/CustomEvent 构造器（R72）

**日期**: 2026-08-16
**Commit**: `1a1d8227`
**前置**: R71（dataset DOMStringMap，S1q 收口，`93eccce4`）
**证据**: [evidence/2026-08-16-r72-quickjs-s4q-event-ctors.json](../evidence/2026-08-16-r72-quickjs-s4q-event-ctors.json)

## 背景

R67 三阶段派发的事件对象是轻量 plain object（`{type:'ping', bubbles:true}` 字面量）。真构造器（`new Event(...)` / `new CustomEvent(...)`）是现代页面标准写法，WPT `Event-constructor` 系列与框架（lit @eventListeners、Vue emits）依赖。

## 实现

**形态决策**（rquickjs 两条 API 边界，见 evidence rquickjs_lessons）：
1. `Function::set_constructor(true)` 不把 this 绑到 new 实例（QuickJS ctor 协议需 `JS_CallConstructor2`，rquickjs 高层 API 不暴露）。
2. `This<Object>` 参数在 JS glue 显式传 this 的调用形态下收不到绑定。

→ **JS 构造器胶水 + Rust 原语**（R71 dataset 模式第二次复用，确立为 QuickJS 标准形态）：
- Rust 原语：`__zw_native_event_init(ev, type, bubbles, cancelable)`（init 属性面 + 单调 timeStamp）+ `__zw_native_event_prevent_default(ev)`（cancelable gate）
- JS 胶水：`Event` 构造器（调原语 + 实例方法 stopPropagation/stopImmediatePropagation/preventDefault/initEvent）+ 常量 NONE/1/2/3 + `CustomEvent`（detail + prototype 链挂 Event.prototype）

## 验证

- PoC 断言五组：init 属性面（8 字段）、instanceof + 常量 + 方法可达、CustomEvent detail + 原型链、dispatchEvent 集成（构造器实例经 R67 三阶段派发，type/instanceof/detail 保留 + 返值语义）、preventDefault cancelable gate
- engine quickjs **1419** / v8 **2153** 全绿零回归；clippy 双矩阵零警告；fmt 无 diff
- pre-commit-guard PASS

## M6 剩余

S0q 续 weak/finalizer（V8 R3133 对等物）→ DOMException 构造器 instanceof 面 → whenDefined 真 pending。

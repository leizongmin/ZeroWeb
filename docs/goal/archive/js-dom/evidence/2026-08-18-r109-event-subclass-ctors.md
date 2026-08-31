# R109 — Event-subclasses-constructors 双路径 100%（events +7P polyfill / +17P native）

**日期**: 2026-08-18
**里程碑**: M4（WPT dom 上游基线扩展）
**Driving 用例**: `dom/events/Event-subclasses-constructors.html`（账本 `imported-tests.txt` R109 条目）
**基线（R108 后）**: polyfill 42P/7F · native 24P/49 → **双路径 49P/0F（100%）**

## 根因（探针实证，非推测）

1. **`class SubclassedEvent extends Event` 全簇 fail（5 subtest）**：shim `Event` 构造器是
   「工厂返对象」形态（`function Event(){ ...; return ev; }`）。derived class 的 `super()`
   反射到该函数的 [[Construct]] 槽时，返回对象不成为子类 this → 子类 ctor 体
   `this.customProp = 5` 抛 TypeError（this 未初始化）、`instanceof SubclassedEvent` false。
   探针输出：`new SubclassedEvent: ok customProp=5 fixedProp=undefined || instanceof Sub: false`。
2. **`UIEvent {view: 7}` 不抛**：缺 WebIDL dictionary 校验（view: `(WindowProxy or null)?`）。
3. **native 叠加路径（ZW_NATIVE_DOM=1）MouseEvent/KeyboardEvent/WheelEvent 18F**：
   native bindings 先装（V8 FunctionTemplate），shim 后装只覆盖 Event/UIEvent——
   `_defineEventSubclass` 的 `if (globalThis[name]) return` guard 把 native 子类留在原地，
   其模板原型链指向**已被 shim 覆盖替换的 native Event 模板** → `instanceof UIEvent/Event`
   全断（探针：`proto>UIEvent=false`）。这是 master.md 记录的「双路径差 6.12pp」主成分。
4. **过程回归（A/B 门抓到）**：首版 for-in 拷贝漏非枚举 accessor → Event-cancelBubble 8P→0P
   （`expected false got undefined`）。改 `getOwnPropertyNames` + `getOwnPropertyDescriptor`
   全属性搬运修复。

## 修复（三文件）

- `part05.js` Event 构造器：真 [[Construct]] 化——`new.target` 派发 proto + 整对象搬运
  （own data + accessor descriptor 逐个 defineProperty）到子类 this。
- `part05.js` `_defineEventSubclass`：super-call 分支（`this instanceof Ctor && this.constructor !== Ctor`）
  填充调用方 this；guard 早退时补登 `_eventSubclassProps` 注册表（WheelEvent 父链 props 收集断链根因）。
- `part05.js` UIEvent view 校验 wrapper（非 null/undefined 非 globalThis → TypeError）。
- `part05.js` native 叠加路径接线 IIFE：MouseEvent/KeyboardEvent.prototype 重接到 shim
  UIEvent.prototype；KeyboardEvent 缺省字段 wrapper（含 native init_string 的字符串
  `"undefined"` 伪装形态修补）。
- `dom_bindings/event.rs`：native KeyboardEvent 补 `detail`（UIEvent 父链属性）。

## A/B 结果（WPT testharness 双路径）

| 路径 | before | after |
|---|---|---|
| polyfill dom/events | 356P/94F | **363P/64F**（+7 净） |
| native dom/events | 337P/90F（R108 计） | **354P/73F**（+17 净，双路径差 6.12pp→2.5pp） |
| Event-subclasses 簇 | 42P/7F · native 31P/18F | **双路径 49P/0F（100%）** |
| dom/nodes / collections / traversal | 6656P / 48P / 1595P | 不变（零回归） |
| Event-cancelBubble / returnValue / initEvent（回归验证） | 8P / 7P / 9P | 同值（A/B 一致） |

## 单测

- `test_event_subclass_constructors_r109`（engine part03.rs，v8 沙箱）：class extends Event
  super() 填充 / instanceof 双向 / 非枚举 accessor 搬运（cancelBubble 联动）/ UIEvent view
  TypeError / 合法 view 不抛 / 基类 new Event() 不回归——4 断言组。

## 验证

- `make test` 65 套件全绿（v8 + quickjs 双矩阵，含 quickjs clippy）
- `cargo fmt --all -- --check` 无 diff；v8 clippy + quickjs clippy（engine/webview/wpt-runner）零警告

## 教训

1. **「工厂返对象」构造器对 `class extends` 是结构性破坏**——super() 拿不到 this，子类体必炸；
   需改真 [[Construct]]（new.target 派发 + this 填充）。JS shim 里所有可被 extends 的构造器
   都须过此检查。
2. **for-in 拷贝漏 defineProperty 定义的非枚举 accessor**——对象搬运一律
   `getOwnPropertyNames + getOwnPropertyDescriptor`。
3. **native 叠加路径的原型链是两个世界的接缝**：V8 FunctionTemplate 的模板原型链在 shim
   覆盖全局构造器后指向孤儿模板——重接 prototype 是最小修法（不动 native 本体装配）。
4. A/B 门再次证明价值：cancelBubble 8P→0P 的回归就是双路径对照抓的，同轮修复后才 land。

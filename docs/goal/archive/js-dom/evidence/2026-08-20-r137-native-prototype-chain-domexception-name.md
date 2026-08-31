# R137 — native 叠加路径原型链重接 + DOMException 构造器 name（Document-createElementNS native 596F→0F）

**日期**: 2026-08-20
**里程碑**: M4（WPT dom 上游基线建立与扩展）
**Driving 用例**: `dom/nodes/Document-createElementNS.html`（596 subtest，WPT 上游）
**运行入口**: `make testharness-dom-native FILTER=Document-createElementNS`（native）/ `make testharness-dom FILTER=...`（polyfill）

## 背景

R136 后 native nodes fail 集 968 vs polyfill 188——差值大头 = Document-createElementNS
**native 596F**（polyfill 0F）。本轮归因该块并收口到双路径 100%。

## 根因（两层，探针实证链）

### ① 原型链断层（instanceof / 常量族失败的根因）

- native 只注册 `HTMLElement` 构造器（`html_element.rs`），**不注册 Node/Element**。
- native `HTMLElement.prototype` 是 FunctionTemplate 产物，原型直链
  `Object.prototype`（探针：`HEchain=?>Object.prototype`）。
- shim 侧 `_zwBuiltNodeChain=false`（native 已注册）跳过自建链 →
  `Element.prototype`（R3019 挂 parentNode/childNodes/remove/cloneNode）与
  `Node.prototype`（R80 常量族 + R136 getRootNode）都是**裸对象直链
  Object.prototype**，与 native 链无连接。
- native 路径 created 元素（`createElement`/`createElementNS` 产物经
  `_wrapHandle` proxy 的 getPrototypeOf trap 返 `HTMLSpanElement.prototype` 等 →
  native `HTMLElement.prototype`）链上：`instanceof Node/Element` 恒 false、
  `el.ELEMENT_NODE` undefined（探针实证）。

### ② DOMException 构造器 name 空串（assert 簇 "did not throw" 假失败的根因）

- WPT testharness `assert_throws_dom(type, ctor, fn)` 按
  `funcOrConstructor.name === "DOMException"` **分派构造器形态**：
  - 匹配 → 调 `fn` 断言抛 DOMException；
  - 不匹配 → 把 **ctor 当被测函数**直接调用（`DOMException()` 不抛 →
    "did not throw" 假失败）。
- native FunctionTemplate 构造器默认 `name` 为**空串**（探针：
  `DEname=""`，而 shim 路径 `Event.name="Event"` 正常）→ 330 个
  assert_throws_dom 用例全走错分支。
- 修复尝试① `f.set(scope, name_key, ...)` **无效**——V8 对
  FunctionTemplate 产物的 `name` 属性是 non-writable（探针：
  `descWritable=w=false`），对象 set 被静默拦截。
- 修复② `v8::Function::set_name(key)`（C++ 侧 `v8__Function__SetName`）——生效
  （探针：`DEstr=function DOMException() { [native code] }`）。

## 修复（两处）

1. **part03.js**（native 模式 `_zwBuiltNodeChain=false` 分支，幂等 setPrototypeOf 三连）：
   - `HTMLElement.prototype` → `Element.prototype`（仅当当前 proto 是
     Object.prototype，防重复重接；R128 Attr.prototype 同款模式）；
   - `Element.prototype` → `Node.prototype`（shim Element ctor 走
     `if (!globalThis.Element)` 兜底创建，其 prototype 是裸对象）；
   - Node 常量族补挂（`_zwBuiltNodeChain=false` 跳过了常量挂载分支）：
     Node.prototype + Node ctor 双面 defineProperty（own 已有不动——R130
     字面量表同款）。
   - R136 的 getRootNode own 补挂保留（防 Element.prototype 后续被替换的保险层）。
2. **dom_exception.rs** `build_and_register`：`f.set_name(key)` 设构造器名
   "DOMException"。

## A/B 验证

- **Document-createElementNS**：native 596F→**0F（596P 100%）**；polyfill
  596P 不变（零回归）。
- **dom/nodes 全量**：native 7684→**8262P（+578）** fail 968→**188**——
  **与 polyfill fail 集逐行 diff 完全一致**（双路径 fail 集首次完全重合，
  native 对等里程碑）；polyfill 8463P/188F 与 R136 一致。
- **跨域**：events polyfill 423P/28F、traversal 1589P/15F、collections 49P
  与 R136 逐项一致；native events 412P/39F 为**基线既存**（stash A/B 重建
  旧二进制同值——非本轮改动面，历史 native 面缺口记 R138 候选）。
- **单测**：engine `native_dom_exception_ctor_name_r137`（构造器 name +
  异常 instanceof/constructor identity 双断言），首跑即过。

## 教训

1. **V8 FunctionTemplate 构造器的 name 属性 non-writable**——对象 `set` 静默
   拦截不报错，改名必须走 `Function::set_name`（C++ 侧）；同族构造器
   （Event 名字正常是因 shim 覆盖，native HTMLElement.name 仍空——后续
   native ctor 注册应统一 set_name）。
2. **testharness 的 assert 系列按 `ctor.name` 分派形态**——构造器名不对会
   产生「did not throw」类的**假失败**，归因时先探 `ctor.name` 再查被测函数。
3. **native 叠加路径的原型链是两个世界的接缝**（R109 教训重演）：native
   注册的构造器原型与 shim 原型链各自独立，凡 native 注册 ctor 而 shim 依赖
   链上成员（常量/方法）的，须显式 setPrototypeOf 接缝——本例三连补接后
   双路径 fail 集完全重合。
4. **探针驱动的归因链**：fail message 里 `function () { [native code] }`
   是分派错误的**指纹**（被调函数本是用户闭包却显示 native code = ctor 被
   当 fn 调用）。

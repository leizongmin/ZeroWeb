# R138 — native 叠加路径事件面三缺口（native events 39F→28F，fail 集与 polyfill 完全重合）

**日期**: 2026-08-20
**里程碑**: M4（WPT dom 上游基线建立与扩展）
**Driving 用例**: `dom/events/Event-stopPropagation-cancel-bubbling.html`（1F）+ `Event-timestamp-high-resolution.html`（2F）+ `Event-dispatch-click.html`（2F）
**运行入口**: `make testharness-dom-native FILTER=events` / `make testharness-dom FILTER=events`

## 背景

R137 后 native nodes fail 集与 polyfill 重合，但 native events 412P/39F vs
polyfill 423P/28F 仍有 11 差（R137 stash A/B 证基线既存）。本轮归因收口。

## 根因（三缺口，探针实证）

### ① 事件方法族不可达（stopPropagation "not a function"）

- shim `_makeEvent` 把 stopPropagation/preventDefault/stopImmediatePropagation
  作为 **own 属性**挂工厂普通对象上——polyfill 路径事件（`new Event()` 经 shim
  构造器 → `_makeEvent`）有这些方法。
- native 叠加路径的 `MouseEvent`/`KeyboardEvent` 是 **native V8
  FunctionTemplate 构造器**（R109 只把 prototype 重接到 shim UIEvent.prototype），
  `new MouseEvent()` 实例化的对象**没有** `_makeEvent` 的 own 方法，原型链上
  `Event.prototype` 也没有（方法全在实例 own）→ listener 内
  `event.stopPropagation()` 报 "not a function"（探针：
  `stop:ERR:event.stopPropagation is not a function`）。

### ② timeStamp origin 独立（high-resolution 断言恒 false）

- native `Event.timeStamp`（dom_bindings event.rs）用**自有 origin**（R22 的
  `OnceLock<Instant>` 首次构造 Event 时初始化）。
- `performance.now()`（JS 侧）的 origin = `register_dom_callbacks` 时刻的另一
  `Instant`。
- WPT 断言 `ev.timeStamp >= before = performance.now()`（取自创建前）——spec
  要求**同 time origin**；native timeStamp 从更晚的独立 origin 起算，数值恒
  小于 performance.now()（探针：expected >= 28.87 but got 0.000038）。

### ③ srcElement / returnValue 缺位（post-click event state）

- native 构造器 `set_event_init` 在实例 own 设 `srcElement = null`（data
  属性），**遮蔽**原型 accessor getter；shim `_dispatchWithBubble` 只设
  `event.target`（shim 工厂事件的 srcElement 是 accessor 读 target 自动跟随，
  native 实例的 own data 属性不跟随）→ dispatch 后 `clickEvent.srcElement`
  仍 null。
- native 实例无 `returnValue`（shim own accessor 的原型版缺失）。

## 修复（三处）

1. **part05.js**：事件方法族（composedPath/preventDefault/stopPropagation/
   stopImmediatePropagation）幂等 defineProperty 上 `Event.prototype`（模块
   顶层，R23 常量块前；**首版放构造器体内首构造前不生效**——构造器体只在
   `new Event()` 时执行，移出）；补 `returnValue`（R28 getter/setter 原型版）
   + `srcElement`（R32 getter 原型版）accessor。shim 工厂产物 own 方法/accessor
   遮蔽原型版——语义零变化；native 实例经 R109 重接链可达。
2. **part03.js** `_dispatchWithBubble`：设 `event.target` 时同步 own-set
   `event.srcElement = target`（own-set 覆盖 native data 属性，两形态统一）。
3. **callbacks.rs + event.rs**：perf origin 提为线程本地共享
   `SHARED_PERF_ORIGIN`（`shared_perf_origin()` 懒初始化）——`performance.now()`
   回调与 native `Event.timeStamp`（`perf_time_origin` 改读共享）三方同源。

## A/B 验证

- **native events**：39F→**28F**（412→423P）——**fail 集与 polyfill 完全
  重合**（逐行 diff 空）；polyfill 422-423P/28F 不变（Timeout 波动 12↔10 为
  既知 flake 面）。
- **跨域**：nodes 双路径 8464P/188F 完全一致（零回归）；traversal
  1589P/15F、collections 49P/0F 不变。
- **单测**：engine `test_event_proto_methods_native_overlay_r138`（Event.prototype
  方法族 typeof + capture stopPropagation 止 bubble + srcElement===target +
  returnValue/eventPhase/currentTarget 派发后态——10 断言段），首跑即过
  （首版断言 `typeof Event.prototype.returnValue` 期望 string 实为
  boolean——accessor defineProperty 后 typeof 走 getter 返 boolean，按实际纠正）。
- `make test` 66 套件全绿；fmt 无 diff；clippy 双矩阵零警告。

## 教训

1. **shim 的 own-方法模式对 native 实例系统性失明**——工厂把方法挂实例 own
   （`_makeEvent` 模式）时，凡有 native 兄弟构造器（FunctionTemplate 实例化
   不经工厂）的方法族都要**原型版双保险**（own 已有则遮蔽，语义零变化）。
2. **构造器体内补挂 = 首构造前不生效**——原型补挂代码必须在模块顶层立即
   执行（或首次构造前的确定点），构造器体内的补挂对「先定义后构造」的
   消费时序有窗口。
3. **spec「same time origin」是多钟一致性约束**——Event.timeStamp 与
   performance.now() 不止各自单调，还须**共享 origin**；独立 origin 即使
   各自正确，跨钟比较断言必炸。共享线程本地 OnceCell 是同线程多消费者的
   最小接缝。
4. **own data 属性遮蔽原型 accessor**——native 构造器把可 alias 的属性
   （srcElement）设为实例 data 属性时，后续 dispatch 的 own-set 覆盖是唯一
   统一路径（原型 getter 对 data 属性不生效）。

# M4 R33 — Window.event（HTML current event legacy global）

**日期**: 2026-08-14
**轮次**: R33
**里程碑**: M4（WPT dom 上游基线 + 按聚类驱动修复）
**前置**: R32（Event.srcElement legacy IE 别名）
**状态**: ✅ 已 land（双路径对等，零回归）

---

## 背景

`Window.event` 是 HTML spec `current event`（legacy IE 全局）。语义：
- Window own `event` 属性，初值 **undefined**（dispatch 前）
- dispatch 期 = 正在派发的 event（innermost-first；嵌套 dispatch 后恢复外层）
- dispatch 后回 **undefined**

WPT `dom/events/event-global.html` 是专门的 window.event 测试（12 subtest），此前双路径 0-pass。`Event-stopPropagation-cancel-bubbling.html`（capture listener 用裸 `event.stopPropagation()`）也依赖此全局。

诊断 probe：capture/bubble 两阶段 dispatch 已工作（order 正确），唯一缺口是 dispatch 期裸 `event` 全局 = `Window.event` 缺失。

## 实现

### polyfill（part01.js + part03.js）

**part01.js（初始化）**：globalThis window 属性段（`globalThis.self/top/parent`）后加：
```js
Object.defineProperty(globalThis, 'event', {
  value: undefined, writable: true, configurable: true, enumerable: true
});
```
- writable:true：dispatch 期可写
- enumerable:true：`assert_own_property(window,'event')` + for-in 可见（WPT event-global 第一个 subtest）

**part03.js `_dispatchWithBubble`（dispatch 期 set/restore）**：
```js
var prevEvent = globalThis.event;
globalThis.event = event;
try { /* capture/target/bubble 三阶段 */ }
finally {
  event._composedPath = null;
  globalThis.event = prevEvent;   // restore 外层（嵌套 dispatch 正确）；顶层后回 undefined
  event._propagationStopped = false;
}
```
- `prevEvent` 局部变量保 dispatch 栈：嵌套 dispatch（redispatch）时内层 finally 恢复外层 event（spec innermost-first）
- 多个 `return !event._defaultPrevented`（capture/target stopPropagation 早退）均经 finally（JS 语义保证 restore 配对）

### native（不改）

`event_target.rs dispatch_event_impl` 是 V8 侧 dispatch，但**用例侧 `document` 始终是 polyfill**（master.md 未解决问题 #9），dispatch 走 polyfill `_dispatchWithBubble`。native dispatch 设 globalThis.event 不会被当前 driving 用例观测到（死路径），且 native 函数多 return 点需全覆盖 restore（易漏致 stale event）。按「轻量修复优先」，本轮不碰 native——native Window.event 留作 default-on 前对齐项。

## 验证

| 门禁 | 命令 | 结果 |
|------|------|------|
| R33 polyfill 单测 | `cargo test -p zero-engine --features v8 --lib test_window_event_current_event_global_r33` | ✅ 1 passed（含嵌套 dispatch restore 外层 event） |
| engine v8 全量 | `cargo test -p zero-engine --features v8 --lib` | ✅ 2110 passed（R32 基线 2109 +1，零回归） |
| engine quickjs 全量 | `cargo test -p zero-engine --no-default-features --features quickjs --lib` | ✅ 1411 passed（零回归） |
| clippy v8 | `cargo clippy -p zero-engine --features v8 --all-targets -- -D warnings` | ✅ 零警告 |
| clippy quickjs | `cargo clippy -p zero-engine -p zero-wpt-runner --no-default-features --features quickjs --all-targets -- -D warnings` | ✅ 零警告 |
| fmt | `cargo fmt --all -- --check` | ✅ 无 diff |
| WPT polyfill event-global | `make testharness-dom FILTER=event-global` | ✅ 0P→**5P**/7F |
| WPT native event-global | `make testharness-dom-native FILTER=event-global` | ✅ 0P→**5P**/7F（双路径对等） |
| dom/events polyfill 全量 | `make testharness-dom FILTER=dom/events` | 174P/153F/6timeout（R29 基线 159P → +15，52.13%→53.23%） |
| dom/events native 全量 | `make testharness-dom-native FILTER=dom/events` | 154P/174F/6timeout（R29 基线 146P → +8，45.57%→46.95%） |

**event-global.html 双路径 5P**：event exists on window initially undefined / only defined during dispatch / undefined if target in shadow tree (dispatched outside) / set to current event during dispatch / set to current event (event passed to dispatch)。

**剩余 7F**（深结构，本轮不碰）：shadow tree 内派发 window.event（需 Shadow DOM 基础设施，slot retarget）+ window.onerror restore（window event handler 基础设施）+ dispatch (2)（redispatch 期 event 重设）。

## 决策记录

- **为何 polyfill 用 data 属性 init + dispatch set/restore，而非 prototype getter**：dispatch 期 window.event 是**可变**全局（每次 dispatch 不同 event + 嵌套 restore），prototype getter 无法表达「当前正在派发的 event」的栈语义；data 属性 + try/finally save/restore 正确建模 dispatch 栈。
- **为何不改 native dispatch_event_impl**：用例侧 dispatch 走 polyfill（未解决问题 #9），native 设 globalThis.event 是死路径；native 函数多 return 点全覆盖 restore 易漏致 stale event；按轻量修复优先 + 双引擎行为等价（driving 用例双路径均走 polyfill）原则，native 留作 default-on 前对齐项。
- **Event-stopPropagation-cancel-bubbling 仍 Fail**：根因非 Window.event，而是 target 阶段（AT_TARGET）stopPropagation 不止同元素后续 listener（`_dispatchToListeners` 'all' 模式需每次 listener 后检查 `_propagationStopped`）——独立 bug，记入下轮候选，非本切片。

## 净影响

- DC-3（WPT dom 基线）：dom/events polyfill 52.13%→53.23%（+1.10pp）/ native 45.57%→46.95%（+1.38pp），双路径对等差 6.28pp（基线 6.56pp）
- DC-4（A/B 对照）：polyfill vs native 双路径行为等价（window.event dispatch 期 set + 嵌套 restore + 后回 undefined）
- 解锁 legacy `event` 全局（listener 内裸 `event.stopPropagation()` 等 IE 风格写法可用）

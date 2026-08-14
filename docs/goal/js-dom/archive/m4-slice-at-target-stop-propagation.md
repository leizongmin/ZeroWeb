# M4 R34 — AT_TARGET stopPropagation 止同元素 listener + 双 flag 兼容

**日期**: 2026-08-14
**轮次**: R34
**里程碑**: M4（WPT dom 上游基线 + 按聚类驱动修复）
**前置**: R33（Window.event current event global）
**状态**: ✅ 已 land（双路径对等，零回归）

---

## 背景

R33 实现 Window.event 后发现 `Event-stopPropagation-cancel-bubbling.html` 仍 Fail。诊断 probe 确认根因**非** Window.event，而是两层独立 bug：

1. **AT_TARGET stopPropagation 不止同元素后续 listener**：WPT 用例 element 无祖先（detached），capture listener 与 bubble listener 注册在同一元素，dispatch 只有 target 阶段。`_dispatchToListeners`（part03.js）'all' 模式下 non-capture 循环未检查 stop propagation flag，capture listener 调 stopPropagation 后同元素 non-capture listener 仍触发。
2. **双 stopPropagation flag 不一致**：polyfill Event（part05 `_makeEvent`）的 stopPropagation 设 `_propagationStopped`；但 ZW_NATIVE_DOM=1 叠加路径下 `new MouseEvent` 走 **native Event 构造器**（dom_bindings Event.prototype），其 stopPropagation（`native_stop_propagation_invoke`）设 `__zw_stop`。而 dispatch 仍走 polyfill `_dispatchWithBubble`（用例侧 document=shim，未解问题 #9），polyfill dispatch 只认 `_propagationStopped`，不认 `__zw_stop` → native Event 的 stopPropagation 在叠加路径下失效。

诊断方法：probe 用 `assert_equals(log, 'XX', 'NATIVE_LOG=...')` 故意 fail 暴露 native 路径执行日志，定位到 `cap-stop-ok|bub`（capture 调了 stopPropagation 但 bubble 仍触发）。

## 实现

### `_dispatchToListeners`（part03.js）—— AT_TARGET 止息 + 双 flag

新增两个局部 helper（兼容 polyfill `_propagationStopped`/`_immediateStopped` 与 native `__zw_stop`/`__zw_stop_immediate`）：
```js
var stopped = function() { return event._propagationStopped || event.__zw_stop === true; };
var immediateStopped = function() { return event._immediateStopped || event.__zw_stop_immediate === true; };
```
- non-capture 循环入口加 `if (stopped()) { ... return; }`：AT_TARGET 时 capture listener stopPropagation 后止同元素 non-capture listener
- 所有 `_immediateStopped` 检查改用 `immediateStopped()`

### `_dispatchWithBubble`（part03.js）—— 冒泡止息双 flag + native flag 重置

- 新增局部 `bubbleStopped` helper（同款双 flag 兼容），3 处 capture/target/bubble 阶段后的 `if (event._propagationStopped)` 改用 `if (bubbleStopped())`
- finally 块加 native flag 重置（`__zw_stop`/`__zw_stop_immediate`），与 `_propagationStopped` 同语义（dispatch 内设的清，支持同 event 重派发 fresh；叠加路径下 native dispatch_event_impl 未跑，polyfill dispatch 负责重置）

### 顺带修：并行 canvas 流 clippy 红灯

main HEAD 的 canvas 流 commit `c1e0bcc8`（getImageData/putImageData）在 `crates/canvas/src/context/context_impl.rs:1285-1286` 引入 2 个冗余 `as i32` 转换（`x.max(0) as i32` / `(x+iw).min(canvas_w) as i32`，操作数已是 i32），触发 clippy `unnecessary_cast` → `-D warnings` 全 workspace clippy 红灯，阻塞 js-dom 流 land。canvas 流推 main 前未本地跑 clippy（run-rules §门禁教训）。机械删除 2 个冗余 cast（零逻辑变化），恢复 main 全绿。历史切片 R10/R15 同款「顺带修并行流 clippy 红灯」。

## 验证

| 门禁 | 命令 | 结果 |
|------|------|------|
| R34 polyfill 单测 | `cargo test -p zero-engine --features v8 --lib test_at_target_stop_propagation_halts_same_element_r34` | ✅ 1 passed（4 场景） |
| 既有 event 测试 | `cargo test -p zero-engine --features v8 --lib test_event` | ✅ 15 passed（零回归） |
| engine v8 全量 | `cargo test -p zero-engine --features v8 --lib` | ✅ 2111 passed（R33 基线 2110 +1） |
| engine quickjs 全量 | `cargo test -p zero-engine --no-default-features --features quickjs --lib` | ✅ 1411 passed（零回归） |
| clippy v8（含 canvas 修复） | `cargo clippy -p zero-engine --features v8 --all-targets -- -D warnings` | ✅ 零警告 |
| clippy quickjs | `cargo clippy -p zero-engine -p zero-wpt-runner --no-default-features --features quickjs --all-targets -- -D warnings` | ✅ 零警告 |
| fmt | `cargo fmt --all -- --check` | ✅ 无 diff |
| WPT polyfill | `make testharness-dom FILTER=Event-stopPropagation-cancel-bubbling` | ✅ 0P→**全 Pass** |
| WPT native | `make testharness-dom-native FILTER=Event-stopPropagation-cancel-bubbling` | ✅ 0P→**全 Pass** |
| dom/events polyfill 全量 | `make testharness-dom FILTER=dom/events` | 175P/152F/6timeout（R33 174P → +1，53.23%→53.52%） |
| dom/events native 全量 | `make testharness-dom-native FILTER=dom/events` | 155P/173F/6timeout（R33 154P → +1，46.95%→47.25%） |

## 决策记录

- **双 flag 兼容而非统一**：polyfill Event 与 native Event 是两个独立的 Event 实现（polyfill=Proxy shim / native=V8 对象），各自 stopPropagation 设各自 flag。叠加路径（ZW_NATIVE_DOM=1，native Event 对象 + polyfill dispatch）下须两套 flag 互认。统一 flag 需改 native Event 构造器或 polyfill dispatch 之一，改动面大；双 flag 兼容（dispatch 同时认两个）改动最小且正确，符合「轻量修复优先」。根治（统一 flag）随 M1 L2 polyfill-live 合一 / default-on 自然解决。
- **AT_TARGET 止息语义**：WPT `Event-stopPropagation-cancel-bubbling` 断言 `t.unreached_func` 不调——即 capture listener 内 stopPropagation 必须止同元素 non-capture listener。spec `concept-event-dispatch`：AT_TARGET 时 capture-listener 先于 non-capture-listener，capture 内 stop propagation flag 后 non-capture 不再触发。
- **为何修 canvas clippy 红灯（跨工作面）**：main HEAD 红灯阻塞所有流的 `-D warnings` 门禁（CI 必 catch）。canvas 流推 main 前未本地跑 clippy（违反 run-rules §门禁）。2 行机械修正（删冗余 cast，零逻辑变化），恢复 main 全绿是全流义务。已在 commit message 注明归因。

## 净影响

- DC-3（WPT dom 基线）：dom/events polyfill 53.23%→53.52%（+0.29pp）/ native 46.95%→47.25%（+0.30pp），双路径对等差 6.13pp
- DC-4（A/B 对照）：polyfill vs native 双路径行为等价（AT_TARGET stopPropagation 止同元素 + 双 flag 兼容叠加路径）
- main 全绿：消除 canvas 流 c1e0bcc8 引入的 clippy unnecessary_cast 红灯
- 解锁 legacy `event.stopPropagation()` 在叠加路径（native Event + polyfill dispatch）下的正确性（default-on 前对齐）

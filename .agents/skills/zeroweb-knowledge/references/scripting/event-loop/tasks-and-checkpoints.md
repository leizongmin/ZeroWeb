# 脚本顺序、microtask、定时器与观察者

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

查脚本段与 Promise、timer、observer、rAF 的交错；`page_scripts.rs` 负责宿主 execute/apply，`timer_bridge.rs` 负责完成投递。动态外链与 module 见 [脚本加载](../modules.md)。

### User Task Navigation

`apps/renderer/src/tick_per_task_tests.rs` 钉调用次数与 pending 保留；`page_scripts/bounded_wait_tests.rs` 钉等待上界；`crates/webview/src/tests/async_drain_bound.rs` 钉排水期限。这些入口未在本轮执行。

## Gotchas
- timer 后台完成只排 resolver，真正回调与 microtask checkpoint 在 JS 线程；宿主轮询拿到完成通知不等于 Promise 续体或 DOM mutation 已 apply。 (`crates/engine/src/timer_bridge.rs`, `crates/engine/src/async_resolver.rs`, `apps/renderer/src/page_scripts.rs`)
- renderer observer per-task 只在 `ZW_RENDERER_TICK_PER_TASK=="1"` 开启；WPT timer per-task 则默认开、`ZW_TESTHARNESS_TIMER_PER_TASK=="0"` 才关闭，不能把 runner 默认推广到 renderer。 (`apps/renderer/src/page_scripts.rs`, `tests/wpt-runner/src/testharness.rs`)
- `__zw_observers_tick_once(from)` 每次安排一个活跃 observer 并返回下一游标，耗尽为 -1；两 observer 的宿主测试包含第三次终止探测，不能把 execute 次数直接当通知条数。 (`crates/engine/src/js_dom_shim/part01.js`, `apps/renderer/src/tick_per_task_tests.rs`)
- per-task observer 宿主循环最多 64 轮，每轮 execute 后 apply；rAF 独立优先 fire-and-forget，相关 mutation 可能到下一次 apply 才落定，不是同步 observer 返回即完成所有帧回调。 (`apps/renderer/src/page_scripts.rs`)
- observer tick 不可先清 pending mutation，异步 timer turn 可能已写队列尚未 drain；快照更新与 observer 写入应合并落 host，否则无声丢掉前一 turn。 (`apps/renderer/src/page_scripts.rs`, `apps/renderer/src/tick_per_task_tests.rs`)
- `drain_pending_dom_mutations` 只在 async-ready 标志出现时发送空 checkpoint；200ms 等待上限不是 JS 执行取消，超时后命令仍可能继续产生 mutation。 (`apps/renderer/src/page_scripts.rs`, `apps/renderer/src/page_scripts/bounded_wait_tests.rs`)
- async drain 的 deadline 要在每轮处理后检查；只在无完成项的空档检查，会被连续 resolver 链保持队列非空拖成 livelock。 (`crates/webview/src/webview.rs`, `crates/webview/src/tests/async_drain_bound.rs`, `git:254e51e3547a6b903625dcf5896bedab4c6afdb0`)
- FileReader 与消息投递已用 setTimeout task 承载，task 之间让 Promise 链完整 drain；无 host 的 shim 回落 microtask 是粒度退化，不能宣称同样拥有真实 task 间隔。 (`crates/engine/src/js_dom_shim/part02.js`, `crates/engine/src/js_dom_shim/part05.js`, `git:b39c119222db50d9ead3c53cd7825f2500aa5555`)

## Architecture
- readyState 过渡采用赋值和 readystatechange 派发的一段原子脚本，interactive 先于 DCL、complete 先于 load；shim 无宿主状态时缺省 complete，孤立脚本测试不能证明加载期三态。 (`apps/renderer/src/page_scripts.rs`, `crates/engine/src/js_dom_shim/part06.js`, `tests/wpt-runner/src/testharness.rs`)
- WPT rAF 帧驱动另由 `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=="1"` 开启，缺省维持同步 stub；计时与动画兼容结论必须注明此分支，不把 runner 全量通过当作生产帧时序。 (`tests/wpt-runner/src/testharness.rs`, `crates/engine/src/js_dom_shim/part01.js`)

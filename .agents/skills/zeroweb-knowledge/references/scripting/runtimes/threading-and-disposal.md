# 线程亲和、任务队列、中断与销毁

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

查宿主等待卡住、导航清队、异步完成与 Drop。`ThreadedRuntimeCore` 是 Dedicated Worker 等使用的通用运行时，不等于 renderer 的双通道页面 worker。

## Gotchas
- `ThreadedRuntimeCore::terminate` 先置终止标志、发送退出命令并中断引擎，再限时五秒 join；超限 detach 只保证宿主不无限等，不保证脚本线程已经结束。 (`crates/script-sandbox/src/threaded_runtime.rs`)
- renderer 的 `shutdown` / Drop 发送 Shutdown 后直接 join，不能套用通用运行时的五秒 detach 保证。 (`apps/renderer/src/js_worker.rs`, `crates/script-sandbox/src/threaded_runtime.rs`)
- renderer 快照等待超时可以放行宿主，但不会取消排队快照；随后优先通道的生命周期 Execute 必须先跨快照屏障，否则 onload 在空 DOM 上完成一次性安装后永久漏派。 (`apps/renderer/src/js_worker.rs`, `git:48f4d1c896eca9076a3f578d1930e6e6395bacdf`)
- 屏障要按最后一个更早快照截取普通队列前缀，并把已有 deferred 命令放在新前缀之前；只抽快照或分桶会让快照越过更早普通 Execute，破坏跨轮 FIFO。 (`apps/renderer/src/js_worker.rs`, `git:137779ea27771edf9e24e475afefb89a8ffc128b`)
- seq 的分配与入队可被不同发送方交错，当前屏障并不提供所有发送方的全局严格排序；它恢复的是已扫描队列前缀与目标优先命令之间的约束。 (`apps/renderer/src/js_worker.rs`)
- `ResetDocumentState` 不套 Execute 的快照屏障，复位臂按 reset seq 决定命令存活；屏障暂存、deferred 与两个通道都必须按同一判据清旧保新。 (`apps/renderer/src/js_worker.rs`)

## Architecture
- 异步宿主只向 resolver 投递结果，Promise 结算回 JS 线程执行；timer/fetch 等后台线程不能直接访问 V8/QuickJS 对象。 (`crates/engine/src/async_resolver.rs`, `crates/engine/src/timer_bridge.rs`, `apps/browser/src/tab_js_worker.rs`)
- 导航复位必须在 worker arm 再清 mutation 队列：主线程清队后、复位执行前，旧 JS 仍可能写入；仅宿主侧清队不能阻止跨文档污染。 (`apps/renderer/src/js_worker.rs`, `apps/renderer/src/tick_per_task_tests.rs`)
- worker 内部恢复命令使用局部续派队列而非回送输入通道，避免等待者再次消费自身暂存输出及新命令插队。 (`apps/renderer/src/js_worker.rs`, `apps/renderer/src/page_scripts.rs`)

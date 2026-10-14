# 脚本执行器与线程运行时

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

执行器负责 Context、回调、中断与值输出；页面线程负责快照与命令次序。排查先判断是通用 Sandbox、Dedicated Worker 还是 renderer/tab 页面 worker，不用一个运行时的等待上界解释另一个。

## Architecture
- renderer 与 tab worker 均在自己的 JS 线程注册桥和刷新快照，WebView 可配置外部执行器；调用 `Sandbox` trait 并不表示执行发生在当前宿主线程。 (`apps/renderer/src/js_worker.rs`, `apps/browser/src/tab_js_worker.rs`, `crates/webview/src/webview.rs`)
- 引擎配置与线程关闭是不同所有权层：Sandbox reset 不负责清页面 TLS，ThreadedRuntimeCore 的限时关闭也不替 renderer 的直接 join 生效。 (`crates/script-sandbox/src/lib.rs`, `crates/script-sandbox/src/threaded_runtime.rs`, `apps/renderer/src/js_worker.rs`)

## Child Knowledge Nodes
- [V8、QuickJS 与沙箱宿主回调](engines-and-sandbox.md) — 持久 Context、同名替换、旧函数捕获和引擎超时。
- [线程亲和、任务队列、中断与销毁](threading-and-disposal.md) — resolver 投递、快照屏障、导航清队和退出保证。

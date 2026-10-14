# Dedicated Worker 与消息传递

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

区分 Rust `DedicatedWorker` / `QuickJSDedicatedWorker`、页面 Worker shim 和 renderer/tab 页面 JS 执行线程。Service Worker 的注册、生命周期与 fetch 唯一归 [Service Worker](../web-data/service-workers/index.md)。

### User Task Navigation

`crates/webview/src/tests/worker_integration.rs` 验证页面 shim 的 data/blob Worker 消息；`script-sandbox/src/worker.rs` / `quickjs_worker.rs` 的测试验证独立运行时。不要把其中一组通过当成另一组的隔离性证据。

## Gotchas
- 页面 `Worker(url)` 当前用同沙箱 `new Function` 与影子 self 执行，不会自动调用 Rust DedicatedWorker 建独立线程/Context；页面 Worker 可见不代表实现了独立 global 隔离。 (`crates/engine/src/js_dom_shim/part05.js`, `crates/script-sandbox/src/worker.rs`, `crates/webview/src/tests/worker_integration.rs`)
- 页面 Worker 的双向 message 仍走 structuredClone 加 queueMicrotask，transfer 参数未完整消费；不能把 window/MessagePort 已迁移 task 与转移语义的结论推广到 Worker。 (`crates/engine/src/js_dom_shim/part05.js`)
- 外链 Worker 需要 `__zw_fetch_script`；未注册时可构造对象却不执行脚本。data/blob 脚本走页内解码/Blob store，不证明宿主网络取回路径存在。 (`crates/engine/src/js_dom_shim/part05.js`, `crates/webview/src/webview.rs`, `tests/wpt-runner/src/testharness.rs`)
- blob Worker 先精确命中页内 store，支持 opaque `blob:null/...`，可解析 origin 则校验同源；它与通用 fetch blob 处理有刻意分支，不能统一按网络 URL 解析拒绝 opaque blob。 (`crates/engine/src/js_dom_shim/part05.js`)
- Worker Blob 脚本先递归物化全部字节再 UTF-8 解码，逐字节 fromCharCode 会把非 ASCII TypedArray/嵌套 Blob 内容错误当 Latin-1。 (`crates/engine/src/js_dom_shim/part05.js`)
- 当前 shim 用正则预取并内联 importScripts 到同 Function 域，以共享 testharness 全局定义；这不是任意表达式 importScripts 的完整 worker 加载器，导入失败仍可被静默跳过。 (`crates/engine/src/js_dom_shim/part05.js`, `tests/wpt-runner/src/testharness.rs`)
- nested Worker、Request、fetch 的相对 URL 基准应取当前 worker script URL，而非主页面地址；shim 显式传 parent/base，转发时丢它会让嵌套资源从错目录取回。 (`crates/engine/src/js_dom_shim/part05.js`)
- QuickJS DedicatedWorker interrupt handler 只检查 terminate flag，没有 QuickJSSandbox 的 timeout deadline；不能从沙箱支持 timeout_ms 推断 worker 每段执行也受该时限保护。 (`crates/script-sandbox/src/quickjs_worker.rs`, `crates/script-sandbox/src/quickjs_runtime.rs`)

## Architecture
- Rust DedicatedWorker 通过 ThreadedRuntimeCore 持有命令/事件队列、终止标志与引擎中断器，宿主轮询消息，不跨线程持有页面 Document；退出保证详见 [线程与销毁](runtimes/threading-and-disposal.md)。 (`crates/script-sandbox/src/worker.rs`, `crates/script-sandbox/src/quickjs_worker.rs`, `crates/script-sandbox/src/threaded_runtime.rs`)

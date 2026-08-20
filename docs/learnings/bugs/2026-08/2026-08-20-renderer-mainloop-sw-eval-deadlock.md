---
date: 2026-08-20
modules: zero-renderer, zero-browser, zero-protocol
---

# SW 求值下放 renderer 后与同步 automation 请求三方互等死锁

## 问题

把 Service Worker 脚本求值从 browser 主进程下放到 renderer 进程（service-workers M1-2）后，两个多进程端到端测试（`multiprocess_navigator_registration_uses_browser_owner` 等）在**第一个** `execute_script` 上 20 秒超时：页面 JS 的 `navigator.serviceWorker.register()` Promise 永远不 settle，browser 发出的 `ServiceWorkerHostCommand(Evaluate)` renderer 侧无任何日志。

## 根因

三方互等死锁，全部在同一 renderer 进程内闭环：

1. **页面 JS**（js_worker 线程）`await register()` → `ServiceWorkerIpcClient` 同步阻塞等 `ServiceWorkerResponse`。
2. **renderer 主循环**被 automation 请求（`ExecuteScript` 等待 js_worker 执行结果）同步占住——而那个脚本正是发起 register 的脚本。
3. **SW 托管命令**按最初实现挂在主循环 `dispatch_message` 里处理——排在被占住的主循环队列末尾，永远轮不到。

browser 侧同时也在等 renderer 的求值事件才回 `ServiceWorkerResponse`，环闭合。

基线（求值在 browser 进程内）没有死锁的关键差异：SW **响应**由 renderer 的 IPC reader 线程直接路由（`route_browser_ipc_inbound` → `ServiceWorkerResponseRouter`），不经过主循环；而新加的**命令**处理最初依赖主循环。

## 解决

- SW runtime 托管挪到 renderer 内**独立线程**（`apps/renderer/src/service_worker_host.rs`）：命令由 IPC reader 线程直接投递（复用 SW 响应同款旁路），事件由托管线程直接经 `SharedWriter` 写回 browser——完全不依赖主循环。
- 排查中发现的第二个 bug：托管线程 `let _ = receiver.recv_timeout(POLL)` 把**等待期间到达的命令直接丢弃**（recv_timeout 返回 `Ok(msg)` 而非信号量语义），必须 `if let Ok(params) = ... { handle(params) }`。

## 如何避免

- renderer 内任何"页面 JS 正在等结果"的新 IPC 消息类型，处理路径必须挂在 reader 线程路由或独立线程上，不得只挂主循环 `dispatch_message`——主循环会被同步 automation/脚本执行长期占住。
- `mpsc::Receiver::recv_timeout` 的 `Ok` 变体携带消息本体；`let _ =` 会吞消息。等待 + drain 混合循环里，`Ok` 必须交给处理逻辑。

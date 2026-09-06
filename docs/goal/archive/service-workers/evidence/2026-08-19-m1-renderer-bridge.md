# M1-4c Renderer Service Worker IPC Bridge

**日期**：2026-08-19
**状态**：M1-4c complete
**前置**：[M1-4b browser owner](2026-08-19-m1-browser-owner.md)

## 0. Response router

renderer 新增独立 `ServiceWorkerResponseRouter` 与 `ServiceWorkerIpcClient`：

- request ID 使用 `1 << 62` 起始的独立空间；
- pending request 最多 64；
- 单次同步 callback 等待上限 20 秒；
- stdin reader/router 线程先分流 SW response，再分流 IndexedDB response；
- browser IPC 断开时 fail 所有 pending callback；
- 非 SW 消息保持原样交给 renderer runtime；
- response 必须匹配原 `IpcMessage.id`。

JS worker 阻塞等待 callback response 时，stdin router 仍在独立线程消费 browser response，不依赖
renderer runtime 主循环重入。

## 1. JS worker callbacks

production `RendererJsWorker` 注入三个 callback：

- `__zw_sw_register(script, scope, documentURL)`；
- `__zw_sw_snapshot(registrationID)`；
- `__zw_sw_unregister(registrationID)`。

callback 只做 typed IPC 与 shim JSON wire 转换。URL、安全上下文、origin 授权、script fetch、
runtime/lifecycle 和 registration state 仍由 browser owner 决定。

standalone `RendererJsWorker::spawn()` 不注入 client，继续明确 reject
`Service Worker host bridge unavailable`，不恢复 timer 生命周期模拟。

## 2. Renderer 生命周期

- normal `ServiceWorkerManager` 位于 browser process，不随 renderer disconnect 销毁；
- navigation start/disconnect 清旧 request correlation，防旧 response 命中新 document request ID；
- normal registration 在 renderer disconnect 后仍可由同 origin、已知 ID 读取 snapshot；
- fresh renderer 对旧 registration 的主动发现尚未接入；当前 `getRegistration(s)` 仍只投影页面已知对象。

browser-backed registration discovery 单列 M1-4d，M1-4c 不宣称该能力完成。

## 3. 验证

- router 单测 3 项：typed snapshot correlation、无关消息透传、outbound send 失败清 pending；
- owner 新增 1 项：normal registration 跨 renderer disconnect 存活；
- fresh `zero-renderer` + browser ProcessBackend E2E：
  - loopback committed page；
  - live renderer `navigator.serviceWorker.register('/sw.js')`；
  - browser-owned localhost script fetch；
  - evaluate/install/activate 与 `ready.active.state === 'activated'`；
  - `registration.unregister() === true`；
- fresh V8 与 QuickJS renderer 均通过相同多进程 E2E；
- QuickJS browser + renderer all-targets clippy 通过；
- `make test` 通过：fresh peers、workspace V8、94 项 adapter GPU、QuickJS WebView 565/565、
  QuickJS WPT runner 110/110、QuickJS renderer；
- `make bench-gate`：16/16 microbenches；welcome/medium/morning total p95 分别为
  14.95/431.41/132.31 ms；retained form p95 0.051 ms、jank 0；绝对预算通过；
- `cargo fmt --all -- --check` 通过。

## 4. 未完成边界

- browser-backed `getRegistration()` / `getRegistrations()`；
- renderer restart 后页面恢复 registration JS projection；
- 下一导航 controller；
- update/updatefound、skipWaiting 页面语义；
- WPT Tier A runner 与 M2 fetch interception。

# M1-4b Browser Service Worker Owner

**日期**：2026-08-19
**状态**：M1-4b complete
**前置**：[M1-4a IPC contract](2026-08-19-m1-ipc-contract.md)

## 0. Owner

`BrowserServiceWorkerOwner` 在 browser process 内持有：

- normal profile 的单一 `ServiceWorkerManager`；
- 每个 private tab 的独立 manager；
- browser-owned script fetch receiver；
- `(profile, registration ID)` 到原 IPC request/tab 的 evaluate correlation。

renderer 只提交 URL、scope、document URL 或 registration ID。script source、runtime、slot 和
registration state 不跨 IPC。

## 1. Authority 与网络

- 只有匹配的 `NavigationCommitted` 才建立 browser authoritative document URL；
- 新 navigation start、renderer disconnect 会撤销旧 authority；
- renderer 声明的 document URL 必须与 committed URL 相等；
- snapshot/unregister/activate-waiting 必须匹配 committed document origin；
- normal profile 跨 tab 共享 manager，但 registration ID 对其他 origin 表现为 NotFound；
- private tab 使用独立 manager，关闭 tab 或退出 private profile 时销毁；
- script fetch 复用 `TabFetchProxy` 的 normal/private loader、cache partition、navigation epoch
  与 security context；
- fetch 异步执行，不阻塞 browser UI poll；
- 非 2xx、redirect、跨源 final URL、非 UTF-8、超过 16 MiB 均 fail closed。

browser 只在 script evaluate success 后返回 `Registered`。网络、脚本、容量和 manager 状态错误
映射为 typed `ServiceWorkerError`，response 复用原 `IpcMessage.id`。

## 2. 生命周期

- renderer crash 只清旧 request correlation，不销毁 profile manager；
- normal registration 可跨 renderer 重建继续存在；
- private manager 可跨同一 tab 的 renderer 重建继续存在；
- tab 真正关闭时销毁 private manager；
- browser `poll()` 在 IPC 前后推进 fetch/evaluate/lifecycle，不由 renderer 推进状态。

## 3. 验证

- owner 单测 5 项：fetch/evaluate + correlation、document authority mismatch、navigation stale
  response 撤销、private namespace 隔离、跨 origin ID 隐藏；
- ProcessBackend authority 单测 2 项：commit 后建立/start 后撤销、mismatched commit 不授权；
- TabFetchProxy 真实 localhost HTTP 测试 1 项：browser loader 抓取并返回脚本正文；
- `cargo test -p zero-browser --no-default-features --features quickjs`：
  lib 7/7，bin 363 passed / 1 ignored，总计 370 passed；
- `cargo clippy -p zero-browser --no-default-features --features quickjs --all-targets -- -D warnings`
  通过；
- default V8 与 QuickJS `zero-browser` all-targets clippy 均通过；
- `make test` 通过：fresh renderer/compositor/image-decoder、workspace V8、94 项 adapter GPU、
  QuickJS WebView 565/565、QuickJS WPT runner 110/110、QuickJS renderer；
- renderer exhaustive IPC match 已显式覆盖 SW request/response，fresh peer build 通过；
- `make bench-gate`：16/16 microbenches、三页绝对预算与 retained-form 门禁通过；当前主机
  与共享 baseline 硬件不同，相对指标按规则不比较；
- `cargo fmt --all -- --check` 通过。

## 4. 未完成边界

- renderer `js_worker` 尚未把 `__zw_sw_register/snapshot/unregister` callback 接到 IPC；
- 尚未用 fresh `zero-browser` + `zero-renderer` binaries 做真实多进程 register；
- production 页面尚不能触发本 owner，M1-4c 完成前不宣称端到端可用；
- 下一导航 controller、update/updatefound、skipWaiting 页面语义仍未完成；
- WPT Tier A runner 与 M2 fetch interception 未开始。

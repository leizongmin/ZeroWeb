# M3-4 Page-to-Worker Message

**日期**：2026-08-20
**状态**：complete

## 实现

- `ServiceWorker.postMessage()` 对 payload 执行 `structuredClone()`，再序列化 JSON-compatible
  structured data；函数、symbol、循环引用与 transferables fail closed。
- protocol 追加 `PostMessage` operation，判别值为 8，既有 0–7 不变；payload 上限为 1 MiB。
- browser owner 用 committed origin 授权 registration ID，renderer 不能向其他 origin 的 worker
  投递消息。
- manager 只允许 installed/waiting、activating 或 activated version 接收消息，并通过 typed
  runtime command 投递。
- Service Worker global 派发 `MessageEvent`，支持 `addEventListener('message')` 与
  `onmessage`；`data` 保留对象/数组结构。
- message handler 抛错投影为独立 `MessageFailed`，不复用 script evaluation failure，也不改变
  active worker lifecycle。

## 回归

- script-sandbox：结构化对象进入 `MessageEvent.data`，handler 写入 persistent global。
- page-runtime：active worker 收到 typed message command，并返回 matching event ID。
- protocol：PostMessage round-trip 与 append-only operation discriminant。
- WebView V8/QuickJS：`registration.active.postMessage({kind, items})` 成功触发 worker message
  event；完整包 V8 619/619、QuickJS 572/572。
- fresh renderer：browser-owned active worker 接受当前 committed page 的 PostMessage IPC。
- Service Worker core baseline：12/12 case、36/36 subtest Pass，连续两轮结果确定。
- `make test`：fresh peers、workspace、94/94 adapter GPU、QuickJS WPT runner 113/113 和
  renderer 全过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。

## 边界

- 本阶段不支持 transferables、MessagePort/MessageChannel。
- worker→page `Client.postMessage()`、container `message` event 与消息队列留待下一切片。
- 现有上游 message WPT 均绑定 iframe、MessageChannel 或反向回传，未调整 WPT 分母。

## 性能

`make bench-gate` 报告 `benchmark_20260820_024729.json`：

- 16/16 crate、94 个微基准完成，报告未标记 suspect；
- page total p95：15.67 / 413.89 / 102.28 ms；
- retained form p95：0.0456 ms，jank 0；
- 当前主机与固定基线 CPU 不同，relative gate 不可比较；absolute page-total 与 retained-form
  budgets 通过。

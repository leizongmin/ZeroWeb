# M3-5 Worker-to-Page Message

**日期**：2026-08-20
**状态**：complete

## 实现

- 页面投递消息时，runtime `MessageEvent.source` 是 browser-owned `Client`，包含 stable client
  identity、committed URL、`type=window` 与 `frameType=top-level`。
- worker `event.source.postMessage()` 将 JSON-compatible structured payload 作为 typed outbound
  event 返回 manager，不直接调用页面引擎。
- manager 按 `(worker version, client identity)` 保存不可消费 message log；renderer/WebView
  持有独立 cursor，多个读取者不会互相丢消息。
- production client identity 使用 `renderer ID:navigation epoch`；embedded WebView 使用
  `instance ID:document generation`。same-tab reload 与新 Document 不重放旧队列。
- protocol 追加 `ClientMessages` operation 9 和 result 7；operation 0–8、result 0–6 判别值不变。
- 页面按 task 轮询并向 `navigator.serviceWorker` 派发 `MessageEvent`；`event.source` 与发出请求的
  `ServiceWorker` JS identity 一致，`target/currentTarget` 为 container。

## 资源上限

- 单条双向 payload 最大 1 MiB。
- 单次 worker event 最多产生 64 条 outbound message。
- 每个 worker version 最多跟踪 256 个 client，每个 client 最多保留 1024 个 event batch；
  pending event 计入容量预留，达限后在 runtime command 入队前返回 capacity error，不启动页面轮询。
- 每个 host 已接受的 page message 最终对应一个 completion batch；handler failure、无回复与 runtime
  断连均以空 batch 推进 cursor，连续 `postMessage()` 等待各自 completion，不会提前停止轮询。
- version redundant/unregister 时清除对应 client logs。

## 回归

- script-sandbox：worker 收到结构化 page payload，经 `Client.postMessage()` 回传对象；
  `Client.id/url` 与宿主输入一致。
- page-runtime：active worker outbound message 写入指定 client log，cursor suffix 精确返回。
- page-runtime：pending event 参与容量检查；runtime 断连时结清 reservation 并写入空 completion batch。
- protocol：ClientMessages request/result round-trip 与 append-only discriminants。
- WebView V8/QuickJS：完整 page→worker→page 往返；container MessageEvent 的 data/source/
  target identity 正确；连续消息按序完成，空回复停止轮询；完整包 V8 619/619、QuickJS 572/572。
- fresh renderer：browser owner 使用 committed URL 与 navigation epoch 隔离 client，往返结果
  返回同一 renderer Document。
- Service Worker core baseline：12/12 case、36/36 subtest Pass，连续两轮结果确定。
- `make test`：fresh peers、workspace、94/94 adapter GPU、QuickJS WPT runner 113/113 和
  renderer 全过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。

## 边界

- 本阶段不支持 transferables、MessagePort/MessageChannel 与多 browsing-context client 枚举。
- 上游 reverse-message WPT 仍依赖 iframe/MessageChannel 或多 client，未调整 WPT 分母。

## 性能

`make bench-gate` 报告 `benchmark_20260820_043845.json`：

- 16/16 crate、94 个微基准完成，报告未标记 suspect；
- page total p95：17.62 / 470.35 / 114.40 ms；
- retained form p95：0.0493 ms，jank 0；
- 当前主机与固定基线 CPU 不同，relative gate 不可比较；absolute page-total 与 retained-form
  budgets 通过。

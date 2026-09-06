# M3-10 Service Worker Import Response Policy

**日期**：2026-08-20
**状态**：complete

## 实现

- classic `importScripts()` 按 no-cors script fetch 语义接受跨源 HTTPS 响应，不再错误要求
  `Access-Control-Allow-Origin`；status、final URL scheme、secure-context downgrade、
  JavaScript MIME、UTF-8 与 size 校验保持 fail closed。
- embedded WebView 新增结构化 `ServiceWorkerScriptFetcher`，保留 status、headers、final URL
  和 redirect count；普通页面/Worker 的 `ScriptSourceFetcher` 契约不变。
- WPT adapter 对固定 revision 的静态 worker、跨源 version handler 和 query-driven MIME
  handler 生成结构化响应。5 个新增资产按 bytes + Git blob SHA 固定，支持 restore、
  verify-only、篡改与缺失恢复回归。
- worker global 具备 `WorkerGlobalScope` / `ServiceWorkerGlobalScope` prototype brand；
  页面 `ServiceWorker` / `ServiceWorkerRegistration` 具备 WebIDL `Symbol.toStringTag`。
- import fetch failure 抛真实 `NetworkError` `DOMException`，legacy `code` /
  `DOMException.NETWORK_ERR` 为 19。
- worker remote-test protocol 的单事件消息上限从 64 调整为 1,024，同时增加 16 MiB
  aggregate batch 上限并保留单消息 1 MiB 上限。固定 MIME WPT 的 21 worker tests 会产生
  65 条协议消息。
- WPT runner 仅在 harness `phase=4`、`pending=0` 且结果数等于注册 test 数时结束，不再由
  可陈旧的 completion callback 提前返回。
- core baseline verifier 除 shape 与两轮 determinism 外，任何非 Pass subtest 都会
  fail closed。

## WPT

- `import-scripts-cross-origin.https.html`：1/1 Pass。
- `import-scripts-mime-types.https.html`：23/23 Pass，包括无 MIME、4 个非法 MIME、
  16 个标准/legacy JavaScript MIME、setup 与 worker result wrapper。
- core baseline：16/16 case、62/62 subtest Pass；V8 与 QuickJS 各连续两轮 deterministic。
- disposition：294 source / 331 URL 确定性重建为 16 core / 49 defer /
  187 gated / 42 skip。
- import-response assets：5/5 restore、verify-only、篡改与缺失 fail-closed 回归通过。
  manifest SHA-256：`d0c49194f874f016115cef33dfaa2d8193cfa24ab504e124f2a627d743323af6`。

## 回归

- Service Worker runtime V8/QuickJS 各 19/19：global brand、NetworkError DOMException、
  65-message batch 与 1,025-message rejection 通过。
- WebView V8/QuickJS 各 18/18：结构化跨源无 ACAO 响应、非法 MIME 拒绝及既有
  register/lifecycle/update/import graph 通过。
- browser owner：跨源无 ACAO 接受、非法 MIME 与 HTTPS→HTTP downgrade 拒绝通过。
- `make test`：V8 WebView 624/624、QuickJS WebView 577/577、QuickJS WPT runner 113/113、
  adapter GPU 94/94、CPU/GPU consistency、fresh renderer/compositor 与 QuickJS Clippy 全过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。

## 性能

`make bench-gate` 报告 `benchmark_20260820_135143.json`：

- 16/16 crate 完成，报告未标记 suspect；
- startup 95.77 ms，peak RSS 155.51 MiB；
- page total p95：18.25 / 569.32 / 124.72 ms；
- retained form p95 0.0547 ms，jank 0；
- 当前主机与固定 baseline CPU 不同，relative gate 不可比较；absolute page-total 与
  retained-form budgets 通过。

## 边界

- `import-scripts-redirect` 的 redirect/stash/update body 状态和
  `import-scripts-resource-map` 的 request-time version/resource map 尚未进入 fixture adapter。
- install/activate/message event 中首次调用 `importScripts()` 的长期 browser fetch context、
  module Service Worker graph 与 M2 FetchEvent/Cache pipeline 仍待后续。

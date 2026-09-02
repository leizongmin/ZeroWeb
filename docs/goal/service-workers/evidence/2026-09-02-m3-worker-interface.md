# M3 Service Worker Interface Requirements

**日期**：2026-09-02
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：complete；promoted to core baseline

## 变更

- `service-workers/service-worker/interface-requirements-sw.https.html` 从
  `defer-advanced` 提升到 core runner。
- 固化独立 interface wave asset manifest：
  [2026-09-02-m3-worker-interface-assets.tsv](2026-09-02-m3-worker-interface-assets.tsv)。
- `FetchEvent` constructor 现在按 WebIDL required dictionary member 语义拒绝缺失或
  非 `Request` 的 `FetchEventInit.request`，覆盖 `undefined`、`{}` 与 `{request: null}`
  三个负例。
- 继续确认 `XMLHttpRequest` 与 `URL.createObjectURL` 不暴露在
  `ServiceWorkerGlobalScope`，且基础 event flags 与 `clientId` 默认值符合该 WPT。

## 验证

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox fetch_event_constructor_requires_request_member -- --nocapture`：
  1 passed
- `make testharness-service-workers-core FILTER=interface-requirements-sw --always-make`：
  4 Pass

## 结论

core Service Worker baseline 从 42 case / 169 subtest 提升到 43 case / 173 subtest。
该切片不改变 CacheStorage 分母，但收紧 SW worker-harness 依赖的 `FetchEvent`
constructor 合规性与全局接口暴露面。

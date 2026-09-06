# M3 Service Worker Global Self Identity

**日期**：2026-09-02
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：complete；promoted to core baseline

## 变更

- `service-workers/service-worker/global-serviceworker.https.any.js` 从
  `defer-worker-global-lifecycle` 提升到 core runner。
- Service Worker runtime 现在在 worker global 暴露只读 `self.serviceWorker`，并让
  `registration.installing` / `registration.waiting` / `registration.active` 在 install /
  activate 事件派发窗口中指向同一个当前 worker 对象。
- `serviceWorker.postMessage()` 在 worker global 内支持启动期自消息，异步派发
  `MessageEvent` 且 `event.source === serviceWorker`。
- 固化 global-serviceworker wave asset manifest：
  [2026-09-02-m3-global-serviceworker-assets.tsv](2026-09-02-m3-global-serviceworker-assets.tsv)。

## 验证

- 初始红线：`make testharness-service-workers-core FILTER=global-serviceworker TIME_LIMIT=300`
  失败 4 个 worker subtest，缺少 `self.serviceWorker` 与 registration lifecycle slots。
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-script-sandbox service_worker_global_ -- --nocapture`：
  3 passed
- `make test-wpt-service-workers-global-serviceworker-wave-assets`：PASS
- `make testharness-service-workers-core FILTER=global-serviceworker TIME_LIMIT=300`：
  1 case / 5 subtests / 5 Pass
- `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json TIME_LIMIT=900`：
  47 cases / 188 subtests / 188 Pass，double-run deterministic

## 结论

core Service Worker baseline 从 46 case / 183 subtest 提升到 47 case / 188 subtest。
该切片补齐 Service Worker global 自身份对象、生命周期事件期 registration slot 投影与启动期
self-message，不改变 CacheStorage window/SW 分母。

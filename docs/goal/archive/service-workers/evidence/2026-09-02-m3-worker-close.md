# M3 Service Worker Global Close Absence

**日期**：2026-09-02
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：complete；promoted to core baseline

## 变更

- `service-workers/service-worker/ServiceWorkerGlobalScope/close.https.html` 从
  `defer-advanced` 提升到 core runner。
- 固化独立 close wave asset manifest：
  [2026-09-02-m3-worker-close-assets.tsv](2026-09-02-m3-worker-close-assets.tsv)。
- 当前 Service Worker global 未暴露 `close()`，符合该 WPT 对
  `ServiceWorkerGlobalScope` 的 negative surface 要求。

## 验证

- `make testharness-service-workers-core FILTER=ServiceWorkerGlobalScope/close --always-make`：
  2 Pass

## 结论

core Service Worker baseline 从 41 case / 167 subtest 提升到 42 case / 169 subtest。
该切片不改变 fetch/CacheStorage 分母，只收敛 SW global 的基础接口暴露面。

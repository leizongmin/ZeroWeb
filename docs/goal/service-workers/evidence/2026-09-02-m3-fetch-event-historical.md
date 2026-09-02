# M3 FetchEvent Historical Interface

**日期**：2026-09-02
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：complete；promoted to core baseline

## 变更

- `service-workers/service-worker/historical.https.any.js` 从 `defer-fetch-interface`
  提升到 core runner。
- 固化独立 FetchEvent historical wave asset manifest：
  [2026-09-02-m3-fetch-event-historical-assets.tsv](2026-09-02-m3-fetch-event-historical-assets.tsv)。
- 该用例确认历史接口 `FetchEvent.prototype.targetClientId` 不暴露，避免已删除 API
  重新进入 Service Worker global。

## 验证

- `make testharness-service-workers-core FILTER=historical --always-make`：2 Pass
- `make audit-wpt-service-workers-fetch-event-historical-wave`：3 assets verified
- `make test-wpt-service-workers-fetch-event-historical-wave-assets`：PASS
- `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json`：
  44 cases / 175 subtests / 175 Pass，double-run deterministic

## 结论

core Service Worker baseline 从 43 case / 173 subtest 提升到 44 case / 175 subtest。
该切片不改变 CacheStorage 分母，只收敛 `FetchEvent` 的历史接口负面暴露面。

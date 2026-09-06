# M3 Service Worker Immutable Prototype

**日期**：2026-09-02
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：complete；promoted to core baseline

## 变更

- `service-workers/service-worker/immutable-prototype-serviceworker.https.html` 从
  `defer-worker-global-prototype` 提升到 core runner。
- Service Worker runtime 现在按 WebIDL immutable prototype exotic object 语义保护
  worker global prototype chain：`self`、`ServiceWorkerGlobalScope.prototype`、
  `WorkerGlobalScope.prototype`、其上层 global 原型与 `Object.prototype` 对
  `Object.setPrototypeOf()` 变更抛出 `TypeError`，`Reflect.setPrototypeOf()` 返回
  `false`。
- 保持普通对象 `Object.setPrototypeOf()` 行为不变，避免把全局保护扩散成通用对象冻结。
- 固化 immutable-prototype wave asset manifest：
  [2026-09-02-m3-immutable-prototype-assets.tsv](2026-09-02-m3-immutable-prototype-assets.tsv)。

## 验证

- 初始红线：`make testharness-service-workers-core FILTER=immutable-prototype TIME_LIMIT=300`
  失败，worker 回传 prototype chain 为 `mutable, mutable, mutable, mutable, immutable`。
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-script-sandbox service_worker_global_prototype_chain_is_immutable -- --nocapture`：
  1 passed
- `make test-wpt-service-workers-immutable-prototype-wave-assets`：PASS
- `make testharness-service-workers-core FILTER=immutable-prototype TIME_LIMIT=300`：
  1 case / 1 subtest / 1 Pass
- `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json TIME_LIMIT=900`：
  48 cases / 189 subtests / 189 Pass，double-run deterministic

## 结论

core Service Worker baseline 从 47 case / 188 subtest 提升到 48 case / 189 subtest。
该切片补齐 Service Worker global prototype chain 的不可变原型语义，不改变 CacheStorage
window/SW 分母。

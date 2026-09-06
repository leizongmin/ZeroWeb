# M3 Service Worker WorkerGlobalScope.isSecureContext

**日期**：2026-09-02
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：complete；promoted to core baseline

## 变更

- `service-workers/service-worker/ServiceWorkerGlobalScope/isSecureContext.https.html`
  从 `defer-worker-harness` 提升到 core runner。
- Service Worker bootstrap 在 `WorkerGlobalScope.prototype` 暴露
  `isSecureContext` getter。当前 runner 与 production SW 脚本入口均在 secure HTTPS
  origin 下执行，因此值固定为 `true`。
- 新增独立 secure-context asset manifest：
  [2026-09-02-m3-worker-secure-context-assets.tsv](2026-09-02-m3-worker-secure-context-assets.tsv)。

## 验证

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox service_worker_global_is_secure_context -- --nocapture`：
  1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- ./target/release/zero-wpt-runner testharness-service-workers --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root isSecureContext --json`：
  2 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-service-workers-core-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --output docs/goal/archive/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json`：
  40 cases / 166 subtests / 166 Pass，double-run deterministic

## 结论

core Service Worker baseline 从 39 case / 164 subtest 提升到 40 case / 166 subtest。
该切片只补 SW global secure-context 表面，不改变 M2 fetch/CacheStorage 分母。

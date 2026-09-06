# M2 CacheStorage Top-Level Credentials

- Date: 2026-09-02
- WPT case: `service-workers/cache-storage/credentials.https.html`
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Manifest SHA: `0fe4a0a0ac0de1097a3ba35689df5396eaa613ec`
- Scope: Service Worker CacheStorage runner promotion; no product code change.

## Result

The top-level upstream credentials page is now part of the pinned Service Worker
CacheStorage wave. It reuses the existing `resources/credentials-worker.js` and
`resources/credentials-iframe.html` fixtures and covers credential-bearing URLs
through iframe XHR, Service Worker fetch interception, Cache key storage,
`Cache.match()`, `Cache.matchAll()`, `CacheStorage.match()`, and worker-to-client
`postMessage()`.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- cargo run -p zero-wpt-runner -- testharness-service-workers-cache-storage service-workers/cache-storage/credentials.https.html --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --json`: 1 case / 1 subtest / 1 Pass
- `python3 tests/wpt-runner/scripts/audit-service-worker-disposition.py --write`: core=51 / defer=34 / gated=167 / skip=42
- `make audit-wpt-service-workers-cache-storage-wave`: 37 assets matched pinned manifest
- `make baseline-wpt-service-workers-cache-storage OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md TIME_LIMIT=420`: 14 cases / 160 subtests / 160 Pass, double-run deterministic

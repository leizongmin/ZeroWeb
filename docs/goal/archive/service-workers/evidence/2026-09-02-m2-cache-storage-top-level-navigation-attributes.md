# M2 CacheStorage Top-Level Navigation Attributes

- Date: 2026-09-02
- WPT case: `service-workers/cache-storage/cache-keys-attributes-for-service-worker.https.html`
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Manifest SHA: `3c96348e0e033cf0b0a14b1f025b70d787febb35`
- Scope: Service Worker CacheStorage runner promotion; no product code change.

## Result

The top-level upstream page is now part of the pinned Service Worker CacheStorage
wave. It reuses the existing `resources/cache-keys-attributes-for-service-worker.js`
worker fixture and covers browser-created navigation `Request.isReloadNavigation`
and `Request.isHistoryNavigation` preservation through `Cache.put(event.request)`
and `Cache.keys()`.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- cargo run -p zero-wpt-runner -- testharness-service-workers-cache-storage service-workers/cache-storage/cache-keys-attributes-for-service-worker.https.html --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --json`: 1 case / 2 subtests / 2 Pass
- `python3 tests/wpt-runner/scripts/audit-service-worker-disposition.py --write`: core=51 / defer=34 / gated=167 / skip=42
- `make audit-wpt-service-workers-cache-storage-wave`: 37 assets matched pinned manifest
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo test -p zero-wpt-runner service_worker_cache_storage_manifest_has_expected_unique_cases -- --nocapture`: 1 passed
- `make baseline-wpt-service-workers-cache-storage OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md TIME_LIMIT=420`: 14 cases / 160 subtests / 160 Pass, double-run deterministic
- `./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo clippy --workspace --all-targets -- -D warnings`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include CARGO_BUILD_JOBS=1 cargo test --workspace --jobs 1`: passed

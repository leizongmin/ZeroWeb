# M2 CacheStorage Worker Sharing WPT Expansion

Date: 2026-08-22

## Scope

This slice adds the upstream CacheStorage window WPT:

- `service-workers/cache-storage/common.https.window.js`
- `service-workers/cache-storage/resources/common-worker.js`

The assets are pinned at WPT revision
`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`.

The new case expands the pinned window corpus from 9 cases / 136 subtests to
10 cases / 137 subtests. It exercises Window and Dedicated Worker visibility
over the same CacheStorage owner: the worker performs three `Cache.put()`
operations and the window reads the entries back with `Cache.match()`.

## Fixed Gap

The runner now supports CacheStorage `.https.window.js` script cases in
addition to `.https.any.js` window variants. Dedicated Worker script fetches in
the WPT runner resolve relative URLs against the page URL, and the JS DOM Worker
shim exposes the existing `CacheStorage`, `Cache`, and `caches` globals to the
worker context so worker writes use the same host bridge as window reads.

## Current Baseline

- Cases: 10
- Subtests: 137
- Status: 137 Pass / 0 Fail / 0 Timeout
- Deterministic double run: true
- Baseline JSON: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json`
- Baseline summary: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`
- Asset manifest: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-assets.tsv`

## Asset Hashes

| Path | Bytes | Git Blob SHA |
|---|---:|---|
| `service-workers/cache-storage/common.https.window.js` | 1609 | `eba312c273de149ae9007d69bb4796b147e03841` |
| `service-workers/cache-storage/resources/common-worker.js` | 481 | `d0e8544b56c2677a9c60d47a9e8b587d63bc6d6c` |

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage common.https.window.js --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 1 subtest / 1 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_cache_api_dedicated_worker_uses_window_cache_storage_bridge -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`: 19 assets matched pinned manifest
- `cargo test -p zero-wpt-runner cache_storage_window_manifest_has_ten_unique_cases`: 1 passed
- `cargo test -p zero-wpt-runner cache_storage_runner_reports_every_case_when_harness_is_missing`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo build --release -p zero-wpt-runner`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`: 10 cases / 137 subtests / 137 Pass, deterministic double run

## Remaining

- Expand to dynamic-server and cross-origin CacheStorage WPT cases.
- Implement full `basic` / `cors` / `opaque` / `opaqueredirect` filtered
  response creation paths.
- Continue Service Worker fetch/cache-specific WPT expansion separately under
  `docs/goal/service-workers.md`.

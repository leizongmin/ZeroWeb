# M2 Cache Abort Top-Level Any WPT Expansion

- Date: 2026-09-02
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Scope: promote the upstream top-level `cache-abort.https.any.js` window variant into the pinned CacheStorage window runner.

## Imported WPT Asset

| Path | Role | Bytes | Git Blob SHA |
|---|---|---:|---|
| `service-workers/cache-storage/cache-abort.https.any.js` | testharness case | 3102 | `99f29b0a08bae82f4be0c0dee98ce5b31a941a48` |

The case reuses already pinned support assets:

- `service-workers/cache-storage/resources/test-helpers.js`
- `common/utils.js`
- `fetch/api/resources/infinite-slow-response.py`
- `fetch/api/resources/stash-take.py`
- `fetch/api/resources/stash-put.py`

## Runtime Impact

No product-code change was required. The existing `cache-abort` fixture and CacheStorage abort handling already satisfy the top-level `.any.js` window entry:

- `Cache.put()` rejects already-aborted, same-task aborted, and headers-received aborted requests with `AbortError`.
- `Cache.add()` rejects the same abort states with `AbortError`.
- `Cache.addAll()` rejects the same abort states with `AbortError`.

## Baseline

- Cases: 38
- Subtests: 448
- Status: 448 Pass / 0 Fail / 0 Timeout
- `cache-abort.https.any.js` contribution: 1 case / 9 subtests / 9 Pass
- Deterministic double run: true
- Baseline JSON: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json`
- Baseline summary: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`
- Asset manifest: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-assets.tsv`

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 240 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh`: 67 assets restored
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`: 67 assets matched pinned manifest
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_window_manifest -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- cargo run -p zero-wpt-runner -- testharness-cache-storage cache-abort.https.any.js --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 1 case / 9 subtests / 9 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`: 38 cases / 448 subtests / 448 Pass, deterministic double run

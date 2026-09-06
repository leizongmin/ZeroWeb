# M2 Storage Buckets CacheStorage WPT Expansion

- Date: 2026-08-22
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Scope: CacheStorage window-environment WPT expansion for bucket-local `caches`.

## Imported WPT Assets

| Path | Role | Bytes | Git Blob SHA |
|---|---|---:|---|
| `service-workers/cache-storage/cache-storage-buckets.https.any.js` | testharness case | 2240 | `fd59ba464db0305de210fc2935d739b2469ec4ae` |
| `storage/buckets/resources/util.js` | support script | 1459 | `5fff4894442a214f6035956128fe95f4a955791a` |

These assets raise the pinned CacheStorage window-runner manifest from 47 to 49
assets and the executable case set from 22 to 23 cases.

## Runtime Fix

The page CacheStorage shim now exposes a minimal `navigator.storageBuckets`
surface for the pinned upstream bucket-local CacheStorage test:

- `navigator.storageBuckets.open(name)` creates an in-memory `StorageBucket`.
- `StorageBucket.caches` delegates to the existing page CacheStorage host bridge
  with a bucket-name prefix encoded from UTF-16 code units.
- `StorageBucket.caches.keys()` strips only that bucket prefix and hides global
  or other-bucket cache names.
- `navigator.storageBuckets.delete(name)` removes prefixed caches and makes old
  `bucket.caches` operations reject with `UnknownError`.

This is intentionally a narrow CacheStorage-support slice. It does not add a
persistent Storage Buckets data model.

## Baseline

- Cases: 23
- Subtests: 293
- Status: 293 Pass / 0 Fail / 0 Timeout
- `cache-storage-buckets` contribution: 1 case / 2 subtests / 2 Pass
- Deterministic double run: true
- Baseline JSON: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json`
- Baseline summary: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`
- Asset manifest: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-assets.tsv`

## Verification

- `./target/test-guard --time-limit 120 -- cargo fmt --all -- --check`: passed
- `./target/test-guard --time-limit 180 -- cargo test -p zero-engine test_cache_api_storage_buckets_namespace_and_delete -- --nocapture`: 1 passed
- `./target/test-guard --time-limit 180 -- cargo test -p zero-wpt-runner cache_storage_window_manifest_has_expected_unique_cases -- --nocapture`: 1 passed
- `./target/test-guard --time-limit 180 -- bash tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`: 49 assets matched pinned manifest
- `./target/test-guard --time-limit 240 -- cargo run -p zero-wpt-runner -- testharness-cache-storage cache-storage-buckets.https.any.js --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 2 subtests / 2 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`: 23 cases / 293 subtests / 293 Pass, deterministic double run

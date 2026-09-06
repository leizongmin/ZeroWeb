# M2 CacheStorage Buckets Service Worker Baseline

- Date: 2026-09-02
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Scope: promote `service-workers/cache-storage/cache-storage-buckets.https.any.js`
  into the pinned Service Worker CacheStorage runner.

## Result

The Service Worker CacheStorage runner now covers 24 cases / 308 subtests, all
passing and deterministic across consecutive runs. This promotion adds
worker-global coverage for bucket-local `CacheStorage` namespaces:

- `navigator.storageBuckets.open()` creates independent bucket objects.
- `bucket.caches.open()` maps cache names into an internal bucket-prefixed
  registration CacheStorage namespace.
- `navigator.storageBuckets.delete()` removes only cache entries for the deleted
  bucket.
- Existing `bucket.caches` and opened `Cache` objects reject after bucket
  deletion with `UnknownError`.

The promotion also fixed classic `importScripts()` helper visibility for WPT
support files: top-level function declarations from imported classic scripts are
now projected onto `globalThis`, matching the existing projection for top-level
`const` / `let` / `class` bindings in the Service Worker runtime harness.

## Verification

- `cargo test -p zero-script-sandbox imported_classic_script_exposes_top_level_lexical_binding`
- `cargo test -p zero-script-sandbox storage_bucket_caches_use_prefixed_registration_cache_namespace`
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-service-workers-cache-storage cache-storage-buckets --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --json`: 1 case / 3 subtests / 3 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-service-workers-cache-storage-baseline.py --runner target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --output docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.json --summary docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md`: 24 cases / 308 subtests / 308 Pass, double-run deterministic

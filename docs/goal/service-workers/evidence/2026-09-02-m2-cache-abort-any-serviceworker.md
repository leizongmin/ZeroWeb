# M2 CacheStorage Abort `.any.js` Service Worker Baseline

- Date: 2026-09-02
- WPT case: `service-workers/cache-storage/cache-abort.https.any.js`
- Result: 1 case / 10 subtests / 10 Pass
- Updated runner baseline: 25 cases / 318 subtests / 318 Pass / deterministic true
- Asset manifest: 52 assets

## Root Cause

The Service Worker CacheStorage runner could execute the existing
`serviceworker/cache-abort.https.html` wrapper because that wrapper imports
`script-tests/cache-abort.js`; the Service Worker script fetcher already
prepended the local dynamic fetch/stash fixture for that imported script.

The top-level `.any.js` Service Worker global variant runs the same abort tests
directly after `service_worker_any_js_source()` wraps the file with the worker
harness. That path did not prepend the `cache-abort` dynamic fixture, so fetches
to upstream wptserve Python resources such as `stash-take.py` fell through to
the static fixture loader and returned Python source text. The WPT then failed
while parsing `response.json()`.

## Fix

`service_worker_any_js_source()` now injects the same `CACHE_ABORT_FETCH_FIXTURE`
used by HTML wrapper imports when the wrapped `.any.js` path contains
`cache-abort`. This keeps the fix in the WPT runner harness layer and does not
change product CacheStorage behavior.

The fixed runner path is covered by
`service_worker_fixture_fetcher_wraps_cache_abort_any_js_with_dynamic_fetch_fixture`.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-service-workers-cache-storage service-workers/cache-storage/cache-abort.https.any.js --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --json`
  - 10 Pass
- `make audit-wpt-service-workers-cache-storage-wave`
  - 52 assets verified
- `cargo test -p zero-wpt-runner service_worker_cache_storage_manifest_has_expected_unique_cases`
  - 1 passed
- `cargo test -p zero-wpt-runner service_worker_fixture_fetcher_wraps_cache_abort_any_js_with_dynamic_fetch_fixture`
  - 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-service-workers-cache-storage-baseline.py --runner target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --output docs/goal/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.json --summary docs/goal/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md`
  - 25 cases / 318 subtests / 318 Pass / deterministic true

# M2 CacheStorage Window WPT Expansion

Date: 2026-08-22

## Scope

This slice expands the pinned upstream CacheStorage window WPT baseline from
4 cases / 35 subtests to 6 cases / 62 subtests:

- `service-workers/cache-storage/cache-matchAll.https.any.js`
- `service-workers/cache-storage/cache-storage-match.https.any.js`

Both files use the existing `resources/test-helpers.js` support closure at WPT
revision `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`.

The expansion exposed that upstream `simple_entries` prepopulates a cache with
`Response.error()`. CacheStorage now allows error filtered responses to be
stored and read back with `type == "error"`. This is separate from
`FetchEvent.respondWith()`, which still rejects status 0 responses through the
fetch response validator.

## Current Baseline

- Cases: 6
- Subtests: 62
- Status: 62 Pass / 0 Fail / 0 Timeout
- Deterministic double run: true
- Baseline JSON: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json`
- Baseline summary: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`
- Asset manifest: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-assets.tsv`

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-storage cache_put -- --nocapture`: 11 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_cache_api_page_shim_puts_error_response -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime cache_storage_handler_preserves_error_response_type -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-protocol service_worker_host_fetch_command_and_event_round_trip -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox cache_put_sends_error_response_to_host_storage -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-webview page_cache_api_rejects_uncacheable_put_and_atomic_add_all -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- make baseline-wpt-cache-storage OUTPUT=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json SUMMARY=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`: 6 cases / 62 subtests / 62 Pass, deterministic double run

## Remaining

- Expand to dynamic-server and cross-origin CacheStorage WPT cases.
- Implement true `basic` / `cors` / `opaque` / `opaqueredirect` filtered
  response creation paths.
- Add per-origin persistent CacheStorage.

# M2 CacheStorage Window WPT Expansion

Date: 2026-08-22

## Scope

This slice expands the pinned upstream CacheStorage window WPT baseline from
4 cases / 35 subtests to 8 cases / 114 subtests:

- `service-workers/cache-storage/cache-matchAll.https.any.js`
- `service-workers/cache-storage/cache-storage-match.https.any.js`
- `service-workers/cache-storage/cache-match.https.any.js`
- `service-workers/cache-storage/cache-put.https.any.js`

The first two files use the existing `resources/test-helpers.js` support
closure. `cache-match.https.any.js` also pulls in `/common/get-host-info.sub.js`
and exercises fetched `simple.txt`, `blank.html`, and `vary.py` fixtures at WPT
revision `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`.

The added `cache-put.https.any.js` case also uses `fetch-status.py` and covers
`Cache.put()` validation, body consumption state, opaque filtered responses with
internal 206 / `Vary: *`, `Response.redirect()`, and Blob/FormData response
bodies.

The expansion exposed that upstream `simple_entries` prepopulates a cache with
`Response.error()`. CacheStorage now allows error filtered responses to be
stored and read back with `type == "error"`. This is separate from
`FetchEvent.respondWith()`, which still rejects status 0 responses through the
fetch response validator.

The added `cache-match.https.any.js` case exposed five integration gaps:
cached `Response.url` was lost, the local WPT fetch handler did not emulate
root-relative support scripts, cross-host WPT fixture fetches, `pipe=header(...)`
or `vary.py`, `Response.blob()` did not reflect current `Content-Type`, and
`new Request(URL, { mode: "no-cors" })` did not produce an opaque fetch response
because URL objects were treated as request-like objects without a `.url` field.

## Current Baseline

- Cases: 8
- Subtests: 114
- Status: 114 Pass / 0 Fail / 0 Timeout
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
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-storage cache_vary_ignored_for_opaque_response`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime cache_storage_handler_preserves_response_url`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_fetch_ --features v8`: 7 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage cache-match.https.any.js --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 25 subtests / 25 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- cargo test -p zero-storage cache_ -- --nocapture`: 77 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 240 -- cargo test -p zero-engine test_response_body_used_redirect_and_blob_formdata_cache_put_support -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 240 -- cargo test -p zero-engine test_cache_api_page_shim_put_response_validation_and_opaque_internal_response -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage cache-put.https.any.js --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 27 subtests / 27 Pass
- `bash tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`: 16 assets matched pinned manifest
- `cargo test -p zero-wpt-runner cache_storage_window_manifest -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo build --release --bin zero-wpt-runner`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`: 8 cases / 114 subtests / 114 Pass, deterministic double run
- `cargo fmt --all -- --check`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`: passed
- `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`: passed

## Remaining

- Expand to dynamic-server and cross-origin CacheStorage WPT cases.
- Implement full `basic` / `cors` / `opaque` / `opaqueredirect` filtered
  response creation paths.
- Add per-origin persistent CacheStorage.

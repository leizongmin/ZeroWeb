# M2 Cache.add WPT Expansion

Date: 2026-08-22

## Scope

This slice adds the upstream CacheStorage window WPT:

- `service-workers/cache-storage/cache-add.https.any.js`

The asset is pinned at WPT revision
`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83` and uses the existing
`resources/test-helpers.js`, `/common/get-host-info.sub.js`, and
`resources/simple.txt` support closure.

The new case expands the pinned window corpus from 8 cases / 114 subtests to
9 cases / 136 subtests. It specifically exercises `Cache.add()` and
`Cache.addAll()` request conversion, request body consumption, duplicate
request rejection, and response `Vary` handling during batch writes.

## Fixed Gaps

- A body-less GET `Request.text()` no longer sets `bodyUsed`, matching Fetch
  null-body behavior and allowing `Cache.add()` to reuse the request.
- `Cache.addAll()` now rejects `undefined` entries with `TypeError` before any
  fetch.
- `Cache.addAll()` keeps early duplicate rejection for identical URL, method,
  and request headers, then performs a post-fetch duplicate check using the
  fetched response `Vary` header before writing any entry.

## Current Baseline

- Cases: 9
- Subtests: 136
- Status: 136 Pass / 0 Fail / 0 Timeout
- Deterministic double run: true
- Baseline JSON: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json`
- Baseline summary: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`
- Asset manifest: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-assets.tsv`

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage cache-add.https.any.js --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 22 subtests / 22 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_request_null_body_text_does_not_mark_body_used -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_cache_api_page_shim_add_all_validates_entries_and_vary_duplicates -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-engine test_cache_api_page_shim -- --nocapture`: 10 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_window_manifest -- --nocapture`: 1 passed
- `bash tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`: 17 assets matched pinned manifest
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo build --release --bin zero-wpt-runner`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`: 9 cases / 136 subtests / 136 Pass, deterministic double run
- `cargo fmt --all -- --check`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo clippy -p zero-engine -p zero-wpt-runner --all-targets -- -D warnings`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`: passed
- `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`: passed

## Remaining

- Expand to dynamic-server and cross-origin CacheStorage WPT cases.
- Implement full `basic` / `cors` / `opaque` / `opaqueredirect` filtered
  response creation paths.

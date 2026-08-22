# M2 Cache.put Error Response Type

Date: 2026-08-22

## Scope

This slice adds the first `Response.type` cacheability guard shared by page
Cache API and Service Worker CacheStorage:

- `zero-storage::CacheResponse` now carries a `response_type` string with
  `"default"` as the compatibility default.
- `Cache::put()` rejects `response_type == "error"` as a TypeError-shaped
  cacheability failure.
- Page `Response.error()` can construct an error filtered response without
  going through the normal 200..599 constructor path.
- Page Cache API serializes response `type` into the host request and rejects
  `Cache.put(..., Response.error())` before calling the host `put` operation.
- Page-runtime CacheStorage host accepts the `type` JSON field and maps the
  storage error into a TypeError.
- Service Worker runtime serializes response `type`, rejects
  `Cache.put(..., Response.error())` as a rejected Promise before host write,
  and keeps FetchEvent response settlement constrained to normal 200..599
  responses.
- Service Worker IPC keeps `ServiceWorkerFetchResponseWire.response_type`
  backward-compatible through serde default `"default"` and validates
  CacheStorage responses separately from FetchEvent/fetch responses.

Out of scope for this slice:

- Full `basic`/`cors`/`opaque`/`opaqueredirect` filtered-response creation and
  readback semantics.
- Broader upstream `cache-storage` WPT import beyond the existing first window
  subset.
- Cache API persistence.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo check -p zero-storage -p zero-page-runtime -p zero-script-sandbox -p zero-protocol -p zero-renderer -p zero-browser -p zero-webview --all-targets`: passed
- `cargo fmt --all -- --check`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-storage cache_api::tests::test_cache_put_rejects_uncacheable_requests_and_responses -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-page-runtime cache_storage_handler_ -- --nocapture`: 11 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-engine test_cache_api_page_shim -- --nocapture`: 8 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox cache_put_rejects_error_response_before_host_write -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-script-sandbox cache -- --nocapture`: 5 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-protocol service_worker_protocol -- --nocapture`: 19 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-webview cache_storage -- --nocapture`: 7 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-renderer service_worker_host -- --nocapture`: 12 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-browser service_worker_owner -- --nocapture`: 52 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy -p zero-storage -p zero-page-runtime -p zero-engine -p zero-script-sandbox -p zero-protocol -p zero-renderer -p zero-browser -p zero-webview --all-targets -- -D warnings`: passed
- `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`: passed

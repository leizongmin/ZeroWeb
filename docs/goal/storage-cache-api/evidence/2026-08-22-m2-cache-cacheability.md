# M2 Cache.put/addAll Cacheability

Date: 2026-08-22

## Scope

This slice tightens the shared Cache API write path used by page Cache API and Service Worker runtime CacheStorage:

- `Cache.put()` rejects non-GET requests.
- `Cache.put()` rejects non-HTTP(S) request URLs.
- `Cache.put()` rejects `206 Partial Content` responses.
- `Cache.put()` rejects responses whose `Vary` header contains `*`.
- `Cache.addAll()` validates all fetched responses before writing any entry, so a rejected batch does not partially populate the cache.

Out of scope for this slice:

- `Response.type` / opaque filtered response handling.
- Broader upstream `cache-storage` WPT import beyond the existing first window subset.
- Cache API persistence.

## Verification

- `cargo test -p zero-storage cache_api::tests:: -- --nocapture`: 56 passed
- `cargo test -p zero-storage cache -- --nocapture`: 75 passed
- `cargo test -p zero-page-runtime cache_storage_handler_ -- --nocapture`: 11 passed
- `cargo test -p zero-engine test_cache_api_page_shim -- --nocapture`: 7 passed
- `cargo test -p zero-webview cache_storage -- --nocapture`: 7 passed
- `cargo fmt --all -- --check`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo clippy -p zero-storage -p zero-page-runtime -p zero-engine -p zero-webview --all-targets -- -D warnings`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`: passed
- `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`: passed

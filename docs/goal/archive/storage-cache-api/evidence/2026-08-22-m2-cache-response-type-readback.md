# M2 Cache Response Type Readback

Date: 2026-08-22

## Scope

This slice closes the readback side of the `Response.type` metadata path for
page Cache API responses:

- CacheStorage host match/matchAll responses now use a Cache-specific
  `__zwcr:` wire payload that includes `response_type`.
- The page Cache API parser accepts both the new `__zwcr:` payload and legacy
  `__zwfr:` fixtures, then restores `Response.type` on the returned JS
  `Response`.
- `Response.clone()` preserves non-error response types, keeping cached
  response metadata stable through common body-consumption patterns.
- The host validates response type strings against the known Fetch response
  type set before storing page-provided wire values.

Out of scope for this slice:

- Constructing true `basic`/`cors`/`opaque`/`opaqueredirect` filtered responses
  from fetch/CORS policy.
- Importing the next upstream `cache-storage` WPT batch.
- Cache API persistence.

## Verification

- `cargo fmt --all -- --check`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime cache_storage_handler_ -- --nocapture`: 12 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_cache_api_page_shim_host_roundtrip -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-webview page_cache_api_match_preserves_cached_response_type -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo clippy -p zero-page-runtime -p zero-engine -p zero-webview --all-targets -- -D warnings`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`: passed
- `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`: passed

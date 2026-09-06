# M2 Service Worker Cache Response Type Guard

Date: 2026-08-22

## Scope

This slice carries Cache API response type metadata through the Service Worker
fetch/cache stack and rejects error filtered responses before they reach the
browser-owned CacheStorage:

- `ServiceWorkerFetchResponse` and `ServiceWorkerFetchResponseWire` now include
  `response_type`, defaulting to `"default"` on the IPC wire for compatibility.
- Runtime `Response` objects expose `type`, `clone()` preserves it, and
  `Response.error()` creates an error filtered response without using the
  normal 200..599 constructor path.
- Runtime Cache API serializes `type` for CacheStorage writes and rejects
  `Cache.put(..., Response.error())` as a rejected Promise before emitting a
  host `Put` request.
- Renderer/browser/WebView Service Worker adapters preserve `response_type`
  across explicit conversion boundaries.
- Protocol validation keeps FetchEvent/worker fetch responses constrained to
  200..599, while CacheStorage responses may carry non-error `status == 0` and
  reject `response_type == "error"`.

Out of scope for this slice:

- Importing the Service Worker fetch/cache WPT baseline.
- Full filtered-response creation for `basic`/`cors`/`opaque`/`opaqueredirect`.
- Completion/archival of `service-workers.md`.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo check -p zero-storage -p zero-page-runtime -p zero-script-sandbox -p zero-protocol -p zero-renderer -p zero-browser -p zero-webview --all-targets`: passed
- `cargo fmt --all -- --check`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox cache_put_rejects_error_response_before_host_write -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-script-sandbox cache -- --nocapture`: 5 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-protocol service_worker_protocol -- --nocapture`: 19 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-renderer service_worker_host -- --nocapture`: 12 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-browser service_worker_owner -- --nocapture`: 52 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy -p zero-storage -p zero-page-runtime -p zero-engine -p zero-script-sandbox -p zero-protocol -p zero-renderer -p zero-browser -p zero-webview --all-targets -- -D warnings`: passed
- `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`: passed

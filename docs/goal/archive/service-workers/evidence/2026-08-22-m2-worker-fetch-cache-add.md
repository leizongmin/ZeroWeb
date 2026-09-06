---
date: 2026-08-22
modules: service-workers,script-sandbox,page-runtime,protocol,browser,renderer,webview
---

# M2 Service Worker Worker Fetch and Cache.add

## Scope

This slice adds the worker-global ordinary `fetch()` path needed by Service
Worker runtime cache population:

- `zero-script-sandbox::ServiceWorkerRuntime` exposes `globalThis.fetch()` and
  emits typed `ServiceWorkerEvent::FetchRequested` events.
- `ServiceWorkerRuntime::complete_fetch()` resolves the blocked worker fetch
  with a browser-owned `ServiceWorkerFetchResponse`.
- `Cache.prototype.add()` and `Cache.prototype.addAll()` in the Service Worker
  runtime fetch GET requests and store successful responses through the existing
  typed `Cache.put()` bridge.
- `zero-page-runtime::ServiceWorkerManager` forwards worker fetch requests as
  `ServiceWorkerManagerEvent::WorkerFetchRequested` and exposes
  `complete_worker_fetch()`.
- Renderer/browser IPC gained `FetchRequested` and `CompleteFetch` messages so
  production worker runtimes can fetch through the browser process.
- Browser process fetches worker-global requests through `TabFetchProxy` using
  normal request/response handling, not Service Worker script MIME or redirect
  policy.
- Embedded WebView handles worker-global fetch with its configured
  `fetch_handler` when present, otherwise through the shared resource loader /
  HTTP client fallback.

The browser process remains the owner of Service Worker registration state and
network policy. The renderer/runtime only passes pure-value requests and
responses.

## Boundary

This does not complete the Service Worker goal:

- The Service Worker fetch/cache WPT baseline is still pending.
- `Cache.add()` currently enforces GET and successful `response.ok`, but the full
  Cache API cacheability matrix and `Vary` behavior remain owned by the sibling
  `storage-cache-api` goal.
- Response bodies crossing the current SW runtime bridge remain UTF-8 strings;
  binary response body support is still outside this slice.

## Verification

Targeted checks run for this slice:

```sh
cargo fmt --all
cargo check -p zero-script-sandbox --no-default-features --features quickjs
cargo check -p zero-protocol
cargo check -p zero-page-runtime --no-default-features --features quickjs
cargo check -p zero-browser --no-default-features --features quickjs
cargo check -p zero-webview --no-default-features --features quickjs
cargo test -p zero-page-runtime install_worker_fetch_request_completes_through_manager --no-default-features --features quickjs -- --nocapture
cargo test -p zero-script-sandbox worker_global_fetch_and_cache_add_roundtrip --no-default-features --features quickjs -- --nocapture
cargo test -p zero-protocol service_worker_host_fetch_command_and_event_round_trip -- --nocapture
cargo test -p zero-renderer worker_global_fetch_round_trips_through_renderer_host --no-default-features --features quickjs -- --nocapture
cargo test -p zero-browser ipc_worker_fetch_event_creates_plan_and_completes_renderer_runtime --no-default-features --features quickjs -- --nocapture
cargo test -p zero-webview worker_global_fetch_powers_cache_add_in_service_worker_runtime --no-default-features --features quickjs -- --nocapture
make test
cargo clippy --workspace --all-targets -- -D warnings
```

Result:

- `zero-script-sandbox` worker fetch/cache.add runtime test: 1 passed
- `zero-page-runtime` manager worker fetch completion test: 1 passed
- `zero-protocol` Service Worker protocol round-trip test: 1 passed
- `zero-renderer` Service Worker host IPC fetch test: 1 passed
- `zero-browser` Service Worker owner fetch plan test: 1 passed
- `zero-webview` in-process Service Worker cache.add test: 1 passed
- `make test`: passed
- default-feature workspace clippy: passed

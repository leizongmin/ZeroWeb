---
date: 2026-08-22
modules: service-workers,script-sandbox,page-runtime,protocol,browser,renderer,storage
---

# M2 Service Worker CacheQueryOptions

## Scope

This slice extends the Service Worker runtime Cache API bridge with query
options:

- Worker-global `Cache.match(input, options)`, `Cache.matchAll(input, options)`,
  `Cache.keys(input, options)`, and `CacheStorage.match(input, options)` now
  serialize `ignoreSearch`, `ignoreMethod`, and `ignoreVary` into typed
  host requests.
- `zero-script-sandbox::ServiceWorkerCacheStorageRequest` carries
  `ServiceWorkerCacheQueryOptions` for `Match`, `MatchAll`, and `Keys`.
- `zero-protocol::ServiceWorkerCacheStorageRequestWire` carries matching
  `ServiceWorkerCacheQueryOptionsWire` values across renderer/browser IPC.
- Renderer host and browser owner conversions preserve options without giving
  the renderer ownership of cache state.
- `ServiceWorkerManager` applies `ignoreSearch` and `ignoreMethod` when
  resolving browser-owned registration CacheStorage requests.

## Boundary

This does not complete the Service Worker goal:

- Full Service Worker fetch/cache WPT promotion and pass-rate evidence remain
  pending.
- `ignoreVary` is currently parsed and transported, but Vary matching still
  depends on the sibling Cache API goal adding request-header snapshot semantics.

## Verification

Targeted checks run for this slice:

```sh
cargo fmt --all
cargo test -p zero-protocol service_worker -- --nocapture
cargo test -p zero-script-sandbox cache_query_options_roundtrip_from_worker_script --no-default-features --features quickjs -- --nocapture
cargo test -p zero-renderer cache_query_options_round_trip_through_renderer_host --no-default-features --features quickjs -- --nocapture
cargo test -p zero-browser ipc_cache_query_options --no-default-features --features quickjs -- --nocapture
cargo test -p zero-page-runtime cache --no-default-features --features quickjs -- --nocapture
```

Result:

- `zero-protocol` Service Worker protocol tests: 19 passed
- `zero-script-sandbox` worker query-options runtime test: 1 passed
- `zero-renderer` Service Worker host query-options IPC test: 1 passed
- `zero-browser` Service Worker owner query-options cache test: 1 passed
- `zero-page-runtime` cache and Service Worker cache tests: 9 passed

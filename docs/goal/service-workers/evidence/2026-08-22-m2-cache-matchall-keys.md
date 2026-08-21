---
date: 2026-08-22
modules: service-workers,script-sandbox,page-runtime,protocol,browser,renderer
---

# M2 Service Worker Cache.matchAll and Cache.keys

## Scope

This slice extends the Service Worker runtime Cache API bridge from
`caches.open()` / `Cache.put()` / `Cache.match()` to list-style operations:

- `Cache.matchAll(input?)` sends a typed cache operation and resolves to a
  bounded `Response[]`.
- `Cache.keys(input?)` sends a typed cache operation and resolves to a bounded
  `Request[]`.
- Protocol, renderer host, browser owner, and in-process manager conversions
  carry the new operations as pure values:
  `ServiceWorkerCacheStorageRequest::{MatchAll, Keys}` and
  `ServiceWorkerCacheStorageResult::{MatchAll, Keys}`.
- Browser-owned registration `CacheStorage` remains the single owner of Service
  Worker cache state; renderer/runtime code only transfers validated request and
  response values.
- Optional request filters preserve method-sensitive behavior on both
  `matchAll()` and `keys()`.

## Boundary

This does not complete the Service Worker goal:

- Service Worker fetch/cache WPT promotion and pass-rate evidence are still
  pending.
- CacheQueryOptions/Vary/add/addAll/full persistence semantics remain under the
  sibling `storage-cache-api` goal.

## Verification

Targeted checks run for this slice:

```sh
cargo fmt --all
cargo test -p zero-script-sandbox caches --no-default-features --features quickjs -- --nocapture
cargo test -p zero-page-runtime cache --no-default-features --features quickjs -- --nocapture
cargo test -p zero-protocol service_worker -- --nocapture
cargo test -p zero-renderer cache_storage_open_put_match_all_keys_round_trips_through_renderer_host --no-default-features --features quickjs -- --nocapture
cargo test -p zero-browser service_worker_owner --no-default-features --features quickjs -- --nocapture
```

Result:

- `zero-script-sandbox` cache tests: 2 passed
- `zero-page-runtime` cache and Service Worker cache tests: 8 passed
- `zero-protocol` Service Worker tests: 19 passed
- `zero-renderer` focused Service Worker host test: 1 passed
- `zero-browser` Service Worker owner/process tests: 50 passed

Submission gates:

- `cargo fmt --all -- --check`: passed
- `git diff --check`: passed
- `cargo clippy --workspace --all-targets --no-default-features --features quickjs -- -D warnings`: passed
- `make test`: passed
- forbidden vendor-marker content scan: no matches

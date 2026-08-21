---
date: 2026-08-22
modules: service-workers,script-sandbox,page-runtime,protocol,browser,renderer
---

# M2 CacheStorage Write Bridge

## Scope

Service Worker runtime CacheStorage support now covers the minimal write path
needed by fetch handlers:

- `caches.open(name)` returns a `Cache` handle after a browser-owned open/create
  operation.
- `Cache.put(request, response)` serializes pure-value Request/Response data and
  stores it in the active registration's browser-owned `CacheStorage`.
- `Cache.match(request)` and `caches.match(request)` share the same typed
  browser-owned operation bridge.

The IPC shape was generalized from a match-only request/response pair to typed
CacheStorage operations:

- `ServiceWorkerCacheStorageRequest::{Open, Match, Put}`
- `ServiceWorkerCacheStorageResult::{Done, Match}`
- `ServiceWorkerHostEvent::CacheStorageRequested`
- `ServiceWorkerHostCommand::CompleteCacheStorage`

The browser process remains the single owner of Service Worker registration
cache state; renderer/runtime code only sends pure-value operations.

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p zero-protocol`
- `cargo check -p zero-script-sandbox`
- `cargo check -p zero-page-runtime`
- `cargo check -p zero-renderer --no-default-features --features quickjs`
- `cargo check -p zero-browser --no-default-features --features quickjs`
- `cargo test -p zero-script-sandbox caches -- --nocapture`
- `cargo test -p zero-page-runtime cache -- --nocapture`
- `cargo test -p zero-protocol service_worker -- --nocapture`
- `cargo test -p zero-renderer service_worker --no-default-features --features quickjs -- --nocapture`
- `cargo test -p zero-browser service_worker_owner --no-default-features --features quickjs -- --nocapture`
- `cargo clippy --workspace --all-targets --no-default-features --features quickjs -- -D warnings`

## Remaining

This closes the SW runtime CacheStorage write bridge slice, but not the overall
Service Worker goal. The remaining M2/DC-4 work is to import and baseline the
current executable Service Worker fetch/cache WPT coverage, with skip/gated
classification where the current runner cannot host the upstream scenario.

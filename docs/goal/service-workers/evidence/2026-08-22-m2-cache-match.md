# M2-3 Service Worker CacheStorage.match Evidence

Date: 2026-08-22

## Scope

This slice wires the minimal Cache API read path needed by cache-first Service
Worker fetch handlers.

Implemented behavior:

- Service Worker globals now expose `CacheStorage` and `globalThis.caches`.
- `caches.match(input)` accepts a `Request` or request URL, serializes a
  bounded pure-value request, and resolves to a `Response` or `undefined`.
- The runtime emits typed `CacheMatchRequested` events and waits for a matching
  `CompleteCacheMatch` response using the same bounded host-callback pattern as
  `importScripts()`, `registration.update()`, and `clients.*`.
- `ServiceWorkerManager` keeps CacheStorage browser-owned by matching against
  the active registration's `cache_storage`.
- Browser/renderer Service Worker host IPC carries `CacheMatchRequested` and
  `CompleteCacheMatch` without exposing cache state to renderer-owned durable
  storage.
- Cached text responses can flow into `event.respondWith(caches.match(event.request))`
  and return through the existing fetch-interception response path.

## Boundary

This does not complete the umbrella goal:

- Only `caches.match()` is exposed inside the Service Worker runtime.
- `caches.open()`, `Cache.put()`, deletion, keys, quota, persistence, Vary, and
  full Fetch/Response binary body semantics remain out of this slice.
- WPT promotion for Service Worker fetch/cache cases still needs a selected
  upstream fixture and pass-rate report.

## Validation

Targeted checks run during the slice:

```sh
cargo test -p zero-script-sandbox service_worker --no-fail-fast
cargo test -p zero-page-runtime service_worker_manager --no-fail-fast
cargo test -p zero-protocol service_worker --no-fail-fast
cargo test -p zero-browser service_worker_owner --no-fail-fast
cargo test -p zero-renderer service_worker_host --no-fail-fast
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
make test
```

Result:

- `zero-script-sandbox`: 35 passed
- `zero-page-runtime`: 40 passed
- `zero-protocol`: 19 passed
- `zero-browser` Service Worker owner/process tests: 48 passed
- `zero-renderer` Service Worker host tests: 9 passed
- `cargo fmt --all -- --check`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- `make test`: passed

New focused tests:

- `service_worker::tests::caches_match_resolves_into_fetch_response`
- `service_worker_manager::tests::fetch_handler_can_respond_with_cache_storage_match`
- `service_worker_protocol::service_worker_host_fetch_command_and_event_round_trip`
- `service_worker_host::tests::cache_match_round_trips_through_renderer_host`
- `service_worker_owner::tests::ipc_cache_match_uses_browser_owned_registration_cache`

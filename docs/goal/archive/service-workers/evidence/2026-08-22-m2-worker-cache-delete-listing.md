---
date: 2026-08-22
modules: service-workers,script-sandbox,protocol,renderer,browser,page-runtime,storage
---

# M2 Service Worker Cache Delete And Listing

## Scope

This slice completes the missing worker-global Cache API delete/listing bridge
for the browser-owned Service Worker registration CacheStorage.

- `ServiceWorkerRuntime` now exposes `Cache.delete()` plus
  `CacheStorage.delete()`, `CacheStorage.has()`, and `CacheStorage.keys()`.
- The runtime JSON bridge now maps those operations to typed
  `ServiceWorkerCacheStorageRequest` variants and returns boolean or cache-name
  list payloads through `ServiceWorkerCacheStorageResult`.
- Renderer/browser IPC gained matching wire variants and validation for
  request payloads and host completions.
- `ServiceWorkerManager` executes the operations against the active
  registration-local `zero-storage::CacheStorage`.
- Successful entry deletes and named cache deletes are treated as CacheStorage
  mutations, so normal-profile Service Worker persistence is marked dirty.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox cache_delete_and_storage_listing_roundtrip_from_worker_script -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 12 --time-limit 600 -- cargo test -p zero-renderer service_worker_host -- --nocapture`: 13 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime fetch_handler_can_delete_cache_entries_and_named_cache_storage -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-browser ipc_cache_delete_and_storage_listing_use_browser_owned_registration_cache -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-protocol service_worker_host_fetch_command_and_event_round_trip -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-script-sandbox cache -- --nocapture`: 6 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-page-runtime cache_storage -- --nocapture`: 20 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-browser service_worker_owner -- --nocapture`: 54 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-protocol service_worker_protocol -- --nocapture`: 19 passed
- `cargo fmt --all -- --check`: passed
- `git diff --check`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 12 --time-limit 1200 -- cargo clippy -p zero-script-sandbox -p zero-protocol -p zero-renderer -p zero-page-runtime -p zero-browser --all-targets -- -D warnings`: passed

## Remaining

- Broader Service Worker cache-storage WPT coverage remains open.
- Full `basic` / `cors` / `opaque` / `opaqueredirect` filtered response
  creation coverage remains open under `storage-cache-api`.

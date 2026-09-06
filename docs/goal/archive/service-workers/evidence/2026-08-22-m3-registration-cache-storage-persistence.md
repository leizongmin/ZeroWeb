---
date: 2026-08-22
modules: service-workers,page-runtime,storage,browser
---

# M3 Service Worker CacheStorage Persistence

## Scope

This slice persists the CacheStorage owned by an active Service Worker
registration.

- `zero-storage` now exposes serializable `CacheStorageSnapshot` DTOs plus
  `CacheStorage::snapshot()` and `CacheStorage::from_snapshot()`.
- `CacheStorage::from_snapshot()` restores through `Cache::put()` validation
  instead of exposing or bypassing Cache internals.
- `ServiceWorkerPersistentRegistration` now carries a defaulted
  `cache_storage` snapshot, so older persistence JSON remains readable.
- Restored active registrations validate and rehydrate registration-local
  CacheStorage before runtime startup.
- Normal-profile `caches.open()` and `Cache.put()` mutations mark the browser
  owner persistence state dirty, so the existing Service Worker persistence
  writer records the updated registration snapshot.
- Private-profile Service Worker CacheStorage remains memory-only.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime persistent_registration_round_trips_cache_storage -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-browser persistent_owner_restores_registration_cache_storage -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-storage cache_storage -- --nocapture`: 27 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime cache_storage -- --nocapture`: 19 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-browser service_worker_owner -- --nocapture`: 53 passed
- `cargo fmt --all -- --check`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo clippy -p zero-storage -p zero-page-runtime -p zero-browser --all-targets -- -D warnings`: passed
- `git diff --check`: passed

## Remaining

- Broader Service Worker fetch/cache WPT coverage remains open.
- Full `basic` / `cors` / `opaque` / `opaqueredirect` filtered response
  creation coverage remains open under `storage-cache-api`.

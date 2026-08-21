---
date: 2026-08-22
modules: storage-cache-api,storage,page-runtime,engine,webview
---

# M2 Cache.matchAll and Cache.keys

## Scope

This slice extends the page Cache API bridge beyond single-entry
`Cache.match()`:

- `zero-storage::Cache` now exposes `match_all(&CacheRequest)` for all matching
  responses and `request_keys()` for full `Request`-like keys while preserving
  the existing `keys() -> Vec<&str>` Rust helper.
- The page runtime CacheStorage host accepts `match_all` and `cache_keys`
  operations against the shared per-origin `StorageManager`.
- `Cache.prototype.matchAll(request?)` returns `Response[]` through the
  existing Response wire format.
- `Cache.prototype.keys(request?)` returns `Request[]` with URL and method
  preserved.
- WebView and V8 shim coverage verify that `Cache.keys()` returns `Request`
  objects and that request filtering keeps method-sensitive matches.

## Boundary

This does not complete the Cache API goal:

- `CacheQueryOptions` (`ignoreSearch`, `ignoreMethod`, `ignoreVary`) are still
  not wired through the page host.
- `add/addAll` still need the real fetch pipeline and response cacheability
  checks.
- per-origin disk persistence and WPT `cache-storage` baseline remain pending.

## Verification

Targeted checks run for this slice:

```sh
cargo fmt --all
cargo test -p zero-storage cache_api -- --nocapture
cargo test -p zero-page-runtime cache --no-default-features --features quickjs -- --nocapture
cargo test -p zero-webview cache_storage --no-default-features --features quickjs -- --nocapture
cargo test -p zero-engine test_cache_api_page_shim_host_roundtrip -- --nocapture
```

Result:

- `zero-storage`: 52 passed
- `zero-page-runtime` cache tests: 8 passed
- `zero-webview` cache storage tests: 2 passed
- V8 page shim bridge test: 1 passed

Submission gates:

- `cargo fmt --all -- --check`: passed
- `git diff --check`: passed
- `cargo clippy --workspace --all-targets --no-default-features --features quickjs -- -D warnings`: passed
- `make test`: passed
- forbidden vendor-marker content scan: no matches

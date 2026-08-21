---
date: 2026-08-22
modules: storage-cache-api,storage,page-runtime,engine,webview
---

# M2 CacheQueryOptions ignoreSearch and ignoreMethod

## Scope

This slice wires Cache API query options through the page-facing Cache API
surface:

- `zero-storage::CacheQueryOptions` is now applied by option-aware
  `Cache.match_request_with_options()`, `Cache.match_all_with_options()`,
  `Cache.delete_with_options()`, `Cache.request_keys_with_options()`, and
  `CacheStorage.match_request_with_options()`.
- URL matching strips fragments for normal matching and also strips query
  parameters when `ignoreSearch` is set.
- Method matching remains strict by default and is bypassed only when
  `ignoreMethod` is set.
- The page CacheStorage host accepts `options` for `Cache.match()`,
  `Cache.matchAll()`, `Cache.delete()`, `Cache.keys()`, and
  `CacheStorage.match()`.
- The page JS shim serializes `ignoreSearch`, `ignoreMethod`, and `ignoreVary`
  from the user option dictionary, and preserves `CacheStorage.match()`
  `cacheName` targeting.
- WebView coverage verifies the user-visible page API behavior against the
  shared per-origin `StorageManager`.

## Boundary

This does not complete the Cache API goal:

- `ignoreVary` is parsed and carried in the wire shape, but correct Vary
  semantics still need request-header snapshots at storage time.
- `add()` / `addAll()` still need fetch integration and cacheability checks.
- WPT `cache-storage` baseline and per-origin disk persistence remain pending.

## Verification

Targeted checks run for this slice:

```sh
cargo fmt --all
cargo test -p zero-storage cache_query_options -- --nocapture
cargo test -p zero-page-runtime cache_storage_handler_applies_query_options --no-default-features --features quickjs -- --nocapture
cargo test -p zero-engine test_cache_api_page_shim_query_options_wire -- --nocapture
cargo test -p zero-webview page_cache_api_query_options_match_delete_and_keys -- --nocapture
```

Result:

- `zero-storage` CacheQueryOptions tests: 3 passed
- `zero-page-runtime` host query-options test: 1 passed
- V8 page shim query-options wire test: 1 passed
- `zero-webview` page Cache API query-options e2e test: 1 passed

---
date: 2026-08-22
modules: storage-cache-api,engine,webview
---

# M2 Cache.add and Cache.addAll Page Fetch Path

## Scope

This slice wires the page-facing `Cache.add()` and `Cache.addAll()` methods to
the existing page `fetch()` path and Cache `put()` bridge:

- `Cache.add(request)` now constructs or reuses a `Request`, rejects non-GET
  requests, fetches a clone of the request, requires a successful
  `Response.ok`, and stores the fetched response through `Cache.put()`.
- `Cache.addAll(requests)` now maps the iterable-like input through
  `Cache.add()` and resolves after all stores complete.
- The V8 page shim bridge test verifies that `add()` and `addAll()` call
  `__zw_fetch` only for GET requests and write the fetched response through
  `__zw_cache_storage`.
- The WebView e2e test runs real page script through `load_html()` and
  `run_page_scripts_strict()`, using `WebViewConfig.fetch_handler` to confirm
  fetched responses are stored and can be read back with `Cache.match()`.

## Boundary

This does not complete the Cache API goal:

- Response cacheability remains partial: this slice enforces `Response.ok`,
  but Vary matching and the full WPT cacheability matrix remain pending.
- WPT `cache-storage` window baseline is still not imported.
- per-origin disk persistence remains pending.
- Service Worker runtime `Cache.add()` / `Cache.addAll()` are not implemented
  in this slice because the worker-global `fetch()` surface is still separate
  from the page fetch bridge.

## Verification

Targeted checks run for this slice:

```sh
cargo fmt --all
cargo test -p zero-engine test_cache_api_page_shim_add_and_add_all_wire -- --nocapture
cargo test -p zero-webview page_cache_api_add_and_add_all_fetch_then_store -- --nocapture
```

Result:

- V8 page shim `Cache.add()` / `Cache.addAll()` wire test: 1 passed
- `zero-webview` page Cache API add/addAll e2e test: 1 passed

Submission gates:

- `cargo fmt --all -- --check`: passed
- `cargo clippy --workspace --all-targets --no-default-features --features quickjs -- -D warnings`: passed
- `make test`: passed

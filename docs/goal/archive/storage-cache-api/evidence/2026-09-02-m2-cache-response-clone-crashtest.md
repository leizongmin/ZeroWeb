# M2 CacheStorage Response Clone Crashtest

Date: 2026-09-02

## Scope

- Added WPT: `service-workers/cache-storage/crashtests/cache-response-clone.https.html`
- Default WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Asset manifest: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-assets.tsv`

## Behavior Covered

The upstream crashtest opens a CacheStorage cache, stores the current document with
`cache.add("")`, reads it back through `cache.match("")`, takes the cached
`Response.body` stream, calls `Response.clone()`, then reads from the original
stream. The test passes when the module script completes and removes
`test-wait` from the document element.

This fixes a runner gap rather than adding new Cache API product code:

- no-harness pages now wait for WPT's `test-wait` completion marker before
  recording a crash-style pass
- no-harness page script errors are reported as failures, avoiding a previous
  false-positive path where a compile/runtime error with zero registered tests
  could pass
- module script lowering now supports top-level `await` by generating an async
  module body IIFE

## Verification

- `./target/test-guard --time-limit 90 --total-mem 4 --per-proc-mem 3 -- cargo test -p zero-script-sandbox test_top_level_await_wraps_module_in_async_iife -- --nocapture`: 1 passed
- `./target/test-guard --time-limit 90 --total-mem 4 --per-proc-mem 3 -- cargo test -p zero-webview test_module_top_level_await_completes_r3083 -- --nocapture`: 1 passed
- `./target/test-guard --time-limit 90 --total-mem 4 --per-proc-mem 3 -- cargo test -p zero-wpt-runner no_harness -- --nocapture`: 3 passed
- `./target/test-guard --time-limit 120 --total-mem 4 --per-proc-mem 3 -- cargo run -p zero-wpt-runner -- testharness-cache-storage cache-response-clone --wpt-data /tmp/zw-wpt-cache-storage --json`: 1 case / 1 subtest / 1 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- make baseline-wpt-cache-storage OUTPUT=docs/goal/storage-cache-api/evidence/2026-09-02-m2-cache-response-clone-baseline.json SUMMARY=docs/goal/storage-cache-api/evidence/2026-09-02-m2-cache-response-clone-baseline.md`: 39 cases / 449 subtests / 449 Pass, deterministic double run

## Deferred

`service-workers/cache-storage/cross-partition.https.tentative.html` remains
deferred. It requires dispatcher/popup/SharedWorker and partitioned-storage
semantics, so it is not a small CacheStorage window-runner slice.

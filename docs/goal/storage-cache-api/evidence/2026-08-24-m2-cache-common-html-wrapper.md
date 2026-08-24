# M2 CacheStorage Common HTML Wrapper WPT Expansion

- Date: 2026-08-24
- Goal: `docs/goal/storage-cache-api.md`
- Scope: Add the upstream `common.https.html` CacheStorage wrapper to the
  pinned window baseline.

## Imported WPT

| Path | Kind | Bytes | Git blob SHA | Source revision |
|---|---|---:|---|---|
| `service-workers/cache-storage/common.https.html` | testharness case | 1842 | `c06e1f13a67f145c5d0bfb3b0555236db515d3f5` | `24197a11e8c5bd29a5cb7bdf18135a82be8a8546` |

## Coverage

This HTML wrapper verifies that a Window can read Cache entries written by a
Dedicated Worker through the same CacheStorage owner. It complements the
already-pinned `common.https.window.js` variant and keeps the window goal on a
real upstream WPT case.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- bash tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`: 61 assets matched pinned manifest
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_window_manifest_has_expected_unique_cases -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage common.https.html --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 1 case / 1 subtest / 1 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- make baseline-wpt-cache-storage OUTPUT=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json SUMMARY=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`: 34 cases / 432 subtests / 432 Pass, deterministic double run

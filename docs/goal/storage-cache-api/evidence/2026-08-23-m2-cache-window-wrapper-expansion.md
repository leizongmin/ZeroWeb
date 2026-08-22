# M2 CacheStorage Window Wrapper WPT Expansion

- Date: 2026-08-23
- Default WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Wrapper source revision: `24197a11e8c5bd29a5cb7bdf18135a82be8a8546`
- Scope: CacheStorage WPT coverage for window wrapper HTML cases.

## Imported WPT Assets

| Path | Role | Bytes | Git Blob SHA |
|---|---|---:|---|
| `service-workers/cache-storage/window/cache-storage.https.html` | testharness case | 388 | `b2f5c385983a7b4a0fe56f2a2756e3a5fb345e24` |
| `service-workers/cache-storage/window/cache-storage-keys.https.html` | testharness case | 398 | `669ab9105e24219ce16f50b59555e85c9ce36a18` |
| `service-workers/cache-storage/window/cache-delete.https.html` | testharness case | 386 | `077e6a3fdc1b2a02dae333007ae9603e62217d43` |
| `service-workers/cache-storage/window/cache-keys.https.html` | testharness case | 380 | `8398c33e146c5983c10922457b7f8591b49f5ed8` |
| `service-workers/cache-storage/window/cache-matchAll.https.html` | testharness case | 377 | `1288b2c034335759364ad6ddc8f44d22ab2eeeae` |
| `service-workers/cache-storage/window/cache-storage-match.https.html` | testharness case | 406 | `f16c45d001e7f2eac0ef6cf2a17c85338b47b179` |
| `service-workers/cache-storage/window/cache-match.https.html` | testharness case | 436 | `f28efad0b76b341171ad230e18d243e229153687` |
| `service-workers/cache-storage/window/cache-put.https.html` | testharness case | 939 | `9641c3470518636f1d2afbbc692f721f20790739` |
| `service-workers/cache-storage/window/cache-add.https.html` | testharness case | 447 | `b5a64c6c369b5028b4aba1396738dc892bc62d3b` |

These wrappers reuse the script-test assets already pinned by the Dedicated
Worker wrapper slice. `window/sandboxed-iframes.https.html` remains out of this
slice because it validates sandbox/origin iframe behavior and needs a separate
iframe-origin fixture review.

## Baseline

- Cases: 32
- Subtests: 429
- Status: 429 Pass / 0 Fail / 0 Timeout
- Window wrapper contribution in this slice: 9 cases / 136 subtests / 136 Pass
- Current `window/` wrapper total, including `window/cache-abort.https.html`: 10 cases / 145 subtests / 145 Pass
- Deterministic double run: true
- Baseline JSON: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json`
- Baseline summary: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`
- Asset manifest: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-assets.tsv`

## Verification

- `bash -n tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh`: passed
- `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh`: 58 assets restored
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`: 58 assets matched pinned manifest
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_window_manifest_has_expected_unique_cases -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo run -p zero-wpt-runner -- testharness-cache-storage window/ --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 10 cases / 145 subtests / 145 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`: 32 cases / 429 subtests / 429 Pass, deterministic double run

# M2 Cache Abort WPT Expansion

- Date: 2026-08-22
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Scope: CacheStorage `cache-abort` coverage for window and Dedicated Worker wrapper cases.

## Imported WPT Assets

| Path | Role | Bytes | Git Blob SHA |
|---|---|---:|---|
| `service-workers/cache-storage/window/cache-abort.https.html` | testharness case | 415 | `405d34d665c27cee93a03f7013f087cfc0a61af7` |
| `service-workers/cache-storage/worker/cache-abort.https.html` | testharness case | 357 | `68bbade07d3862c883bf0f95b02a5cb6181d8e79` |
| `service-workers/cache-storage/script-tests/cache-abort.js` | shared script | 3085 | `3c7aa5cd2ffc9ef975e45456b10a60d1c4281693` |
| `common/utils.js` | support script | 2447 | `62e742bee7f67cf3bd92a217a0a92b23fddf3017` |
| `fetch/api/resources/infinite-slow-response.py` | dynamic fetch fixture source | 986 | `a26cd8064c88531e6877d2561cd16df964eb7f6e` |
| `fetch/api/resources/stash-take.py` | dynamic fetch fixture source | 302 | `e6db80dd86df1e5a80e177b78bd40a59234885b0` |
| `fetch/api/resources/stash-put.py` | dynamic fetch fixture source | 745 | `dbc7ceebb882ebb77a0202b3c9a828e2c70dc3bc` |

These assets raise the pinned CacheStorage window-runner manifest from 40 to 47
assets and the executable case set from 20 to 22 cases.

## Runtime Fix

- Page `fetch()` now defers resolving a synchronous host `__zw_fetch` result when
  an `AbortSignal` is present, so a same-task `controller.abort()` can reject
  with `AbortError` before the sync test fixture response settles.
- Dedicated Worker script execution now binds bare `fetch` to `self.fetch`, so
  worker test scripts use the worker-global fetch wrapper rather than bypassing
  the injected test fixture.
- The WPT runner injects a case-local `cache-abort` fixture only for
  `cache-abort` cases. It models the three dynamic WPT resources used by the
  upstream test:
  `fetch/api/resources/infinite-slow-response.py`,
  `fetch/api/resources/stash-take.py`, and
  `fetch/api/resources/stash-put.py`.

## Fixture Limitation

ZeroWeb's current WPT data fetch path is synchronous, while upstream
`infinite-slow-response.py` intentionally keeps a response open until the test
aborts it. A Rust-side blocking resource fixture caused the WebView synchronous
fetch bridge to hang. This slice keeps the dynamic behavior inside the page JS
fixture, scoped to `cache-abort`, so normal runner fetch behavior remains
unchanged.

## Baseline

- Cases: 22
- Subtests: 291
- Status: 291 Pass / 0 Fail / 0 Timeout
- `cache-abort` contribution: 2 cases / 18 subtests / 18 Pass
- Deterministic double run: true
- Baseline JSON: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json`
- Baseline summary: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`
- Asset manifest: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-assets.tsv`

## Verification

- `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh`: 47 assets restored
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`: 47 assets matched pinned manifest
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage window/cache-abort.https.html --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 1 case / 9 subtests / 9 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage worker/cache-abort.https.html --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 1 case / 9 subtests / 9 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage -- --nocapture`: 2 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_fetch_abort_signal_r3044 -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`: 22 cases / 291 subtests / 291 Pass, deterministic double run

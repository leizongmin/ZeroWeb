# M2 CacheStorage Dedicated Worker Wrapper WPT Expansion

- Date: 2026-08-22
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Scope: CacheStorage WPT coverage for Dedicated Worker wrapper HTML cases.

## Imported WPT Assets

| Path | Role | Bytes | Git Blob SHA |
|---|---|---:|---|
| `service-workers/cache-storage/worker/cache-storage.https.html` | testharness case | 355 | `0899609e4aeb46463baf719cbeaea50a3523940e` |
| `service-workers/cache-storage/worker/cache-storage-keys.https.html` | testharness case | 365 | `71d995bc9400b173bef486b178f43d29b8038076` |
| `service-workers/cache-storage/worker/cache-delete.https.html` | testharness case | 353 | `3d63a2f7f7d3120a5af4b43dc528223b1d9f85b5` |
| `service-workers/cache-storage/worker/cache-keys.https.html` | testharness case | 347 | `6bafe21d30bb55d4e0dfdc8f3633bcf359fc7d6c` |
| `service-workers/cache-storage/worker/cache-matchAll.https.html` | testharness case | 359 | `c7e893a23cd63c30d9c0108d92effc55e2c0bfe6` |
| `service-workers/cache-storage/worker/cache-storage-match.https.html` | testharness case | 373 | `cd93410d234886ce63d97e6c13461b90a9d4d369` |
| `service-workers/cache-storage/worker/cache-match.https.html` | testharness case | 350 | `479a29d1eec5f49416a075aae9306bcd5b5cc3e6` |
| `service-workers/cache-storage/worker/cache-put.https.html` | testharness case | 344 | `20aeb2351efb70ceedbd7c5c46dc7d500def71f8` |
| `service-workers/cache-storage/worker/cache-add.https.html` | testharness case | 361 | `2658e1e50f9ebfe8ac5971a16af9e26b02d140a8` |
| `service-workers/cache-storage/script-tests/cache-storage.js` | worker script | 8270 | `0de2dbb05e96dd6756f94af769ab2417f3902a95` |
| `service-workers/cache-storage/script-tests/cache-storage-keys.js` | worker script | 1154 | `ef06ccdff5c8df29c2aefb5d11c4d3dfcf493bf6` |
| `service-workers/cache-storage/script-tests/cache-delete.js` | worker script | 5981 | `e5238c455620fc9741a7b149bd060cf10a410874` |
| `service-workers/cache-storage/script-tests/cache-keys.js` | worker script | 7573 | `94b34d1ebf0d91681059875a2e30d83b31e9ca4b` |
| `service-workers/cache-storage/script-tests/cache-matchAll.js` | worker script | 9174 | `438ddebbdc0c716702de01c7cf457678b1276ec5` |
| `service-workers/cache-storage/script-tests/cache-storage-match.js` | worker script | 9074 | `54be7e7b5d78b35314b9fafecd1166203eec9b4d` |
| `service-workers/cache-storage/script-tests/cache-match.js` | worker script | 17226 | `22c0689d36d60cb036376482ffa4a9ca334fc14b` |
| `service-workers/cache-storage/script-tests/cache-put.js` | worker script | 14860 | `f4251105cab0e0e397c65304f784f9af62d17af7` |
| `service-workers/cache-storage/script-tests/cache-add.js` | worker script | 13938 | `62b44c7880168b5762785ed9f75c2dfa167f9401` |

`worker/cache-abort.https.html` remains out of this slice because it depends on
AbortController plus dynamic slow-response/stash fixtures; it should be handled
with a fetch abort fixture slice.

## Runtime Fix

The Dedicated Worker shim now exposes the minimum worker global surface needed
by WPT worker wrappers:

- `self` is a real alias for the worker global context.
- `WorkerGlobalScope` / `DedicatedWorkerGlobalScope` markers allow
  `testharness.js` to select its Dedicated Worker environment.
- Worker `self.addEventListener("message", ...)` uses the same EventTarget path
  as the page-side `Worker` object.
- `Request` and `fetch` in worker scripts resolve relative URLs against the
  worker script URL, while `location` exposes `protocol`, `host`, `pathname`,
  `search`, `hash`, and `origin`.
- Globals written by imported helper scripts, such as
  `self.create_temporary_cache`, are visible to the subsequently executed
  worker test script as bare identifiers.

## Baseline

- Cases: 20
- Subtests: 273
- Status: 273 Pass / 0 Fail / 0 Timeout
- Worker wrapper contribution: 9 cases / 135 subtests / 135 Pass
- Deterministic double run: true
- Baseline JSON: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json`
- Baseline summary: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`
- Asset manifest: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-assets.tsv`

## Verification

- `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh`: 40 assets restored
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_window_manifest -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_cache_api_dedicated_worker_uses_window_cache_storage_bridge -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_dedicated_worker_imported_self_property_is_bare_global -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage worker/cache-storage.https.html --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 10 subtests / 10 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage worker/cache-storage-match.https.html --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 11 subtests / 11 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage worker/cache-put.https.html --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 26 subtests / 26 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage worker/cache-match.https.html --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 25 subtests / 25 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage worker/cache-add.https.html --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 22 subtests / 22 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo run -p zero-wpt-runner -- testharness-cache-storage worker/ --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 9 cases / 135 subtests / 135 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`: 20 cases / 273 subtests / 273 Pass, deterministic double run

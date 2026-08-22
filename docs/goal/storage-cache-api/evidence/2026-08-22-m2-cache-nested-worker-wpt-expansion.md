# M2 CacheStorage Nested Worker WPT Expansion

- Date: 2026-08-22
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Scope: CacheStorage window-environment WPT expansion for nested Dedicated Worker access.

## Imported WPT Assets

| Path | Role | Bytes | Git Blob SHA |
|---|---|---:|---|
| `service-workers/cache-storage/cache-api-nested-worker.https.html` | testharness case | 641 | `769200f66aa377840ba9c827d374c9cfaff7ec60` |
| `service-workers/cache-storage/cache-api-nested-worker1.js` | worker script | 110 | `eb5c5cd90bb4f352f97071a3201d163039a10cb3` |
| `service-workers/cache-storage/cache-api-nested-worker2.js` | worker script | 79 | `0af8c3eebfeffb8eab2b33e1b78083f3fa9f1031` |

## Runtime Fix

The CacheStorage runner now executes `.https.html` cases as HTML instead of wrapping them as `.any.js` window scripts. The Dedicated Worker shim now carries each worker's resolved script URL into `self.location.href`, and worker-created nested workers resolve relative script URLs against that parent worker script URL. The same worker context continues to expose the page-owned `CacheStorage` / `Cache` / `caches` bridge.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_dedicated_worker_nested_worker_resolves_against_parent_script_url -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-cache-storage cache-api-nested-worker.https.html --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --json`: 1 subtest / 1 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`: 22 assets matched pinned manifest
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_window_manifest -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo build --release --bin zero-wpt-runner`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`: 11 cases / 138 subtests / 138 Pass / 0 Fail, deterministic double run
- `cargo fmt --all -- --check`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`: passed
- `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`: passed

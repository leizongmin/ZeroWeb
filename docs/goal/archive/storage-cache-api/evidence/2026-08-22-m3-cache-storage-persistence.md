# M3 CacheStorage Persistence

Date: 2026-08-22

## Scope

This slice adds per-origin CacheStorage persistence to `zero-storage` and wires
the page/WebView host path to use it.

- CacheStorage entries serialize request URL/method/headers and response URL,
  status, status text, type, headers, and byte body.
- Origin files are named by SHA-256 hash under a CacheStorage directory with a
  `.cache` extension.
- Writes use a temporary file, `sync_all`, atomic rename, and directory sync on
  Unix. Startup removes orphan `.tmp` files and restores `.bak` files left by
  interrupted Windows replacements.
- `StorageManager` now exposes persistence-aware CacheStorage mutation helpers
  that write the candidate state before replacing live state.
- Browser normal profiles use the existing IndexedDB directory plus a sibling
  CacheStorage directory. `ZERO_PRIVATE` continues to keep storage in memory.
- Embedded `IndexedDbOwner::persistent(path)` preserves the existing IndexedDB
  root layout and adds CacheStorage at `path/CacheStorage`.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo check -p zero-storage -p zero-page-runtime -p zero-webview --all-targets`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-storage cache_storage_persistence -- --nocapture`: 3 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime cache_storage_handler_ -- --nocapture`: 15 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-webview cache_storage -- --nocapture`: 9 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-webview indexed_db_owner -- --nocapture`: 4 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo clippy -p zero-storage -p zero-page-runtime -p zero-webview -p zero-browser --all-targets -- -D warnings`: passed
- `cargo fmt --all -- --check`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`: passed
- `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- cargo test --workspace --jobs 1`: passed
- `make baseline-wpt-cache-storage OUTPUT=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json SUMMARY=docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`: 8 cases / 114 subtests / 114 Pass / 0 Fail, deterministic double run

## Remaining

- Service Worker registration-local CacheStorage persistence was completed by
  the follow-up slice recorded in
  `docs/goal/archive/service-workers/evidence/2026-08-22-m3-registration-cache-storage-persistence.md`.
- Full `basic` / `cors` / `opaque` / `opaqueredirect` filtered response
  creation coverage remains open.
- Dynamic-server and cross-origin CacheStorage WPT expansion remains open.

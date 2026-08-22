# CacheStorage Window Manifest Source Revisions

- Date: 2026-08-23
- Scope: `docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-assets.tsv`
- Default WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Additional source revision: `24197a11e8c5bd29a5cb7bdf18135a82be8a8546`

## Change

The CacheStorage window asset manifest now records a per-asset `source_revision`
column. The restore script uses that column when downloading missing assets and
keeps the original default revision as the compatibility fallback for older
7-column manifests.

## Reason

The current 32-case baseline intentionally combines the initial fixed
CacheStorage `.any.js` corpus with later window/worker wrapper assets. The
wrapper assets and shared script-tests match the local WPT checkout at
`24197a11e8c5bd29a5cb7bdf18135a82be8a8546`, while the original `.any.js`
cases and harness assets are pinned to
`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`. A single global revision in the
restore script was therefore not enough to reconstruct every gitignored WPT
asset from a clean checkout.

## Validation

- `bash -n tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh`: passed
- `./target/test-guard --time-limit 120 -- bash tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only`: 49 assets matched the manifest before the 2026-08-23 window wrapper expansion
- `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --time-limit 120 -- bash tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh`: restored a deleted gitignored `worker/cache-add.https.html` asset and verified its manifest bytes/blob
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner cache_storage_window_manifest_has_expected_unique_cases -- --nocapture`: passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py --runner ./target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root --output docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.json --summary docs/goal/storage-cache-api/evidence/2026-08-22-cache-storage-window-baseline.md`: 32 cases / 429 subtests / 429 Pass, deterministic double run after the window wrapper expansion

## Follow-up

Future CacheStorage WPT imports should either use one revision for the whole
corpus or add the exact source revision for each new asset in the manifest.

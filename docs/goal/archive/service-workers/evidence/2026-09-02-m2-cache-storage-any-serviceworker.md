# M2 CacheStorage `.any.js` Service Worker Variant

- Date: 2026-09-02
- WPT case: `service-workers/cache-storage/cache-storage.https.any.js`
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Manifest SHA: `b7d5af7b532036351ab842de451002ca36f7b4af`
- Scope: Service Worker CacheStorage runner promotion plus WPT `.any.js` META support loading.

## Result

The upstream `.any.js` CacheStorage case now runs as a real Service Worker global
variant in the pinned Service Worker CacheStorage wave. The runner now honors
`// META: script=` declarations when wrapping Service Worker `.any.js` sources,
so worker-side helpers such as `resources/test-helpers.js` are loaded before the
case body.

This promotes direct Service Worker `.any.js` coverage for `CacheStorage.open()`,
delete dooming, empty cache names, required-argument TypeError behavior,
`CacheStorage.has()` / `delete()`, and DOMString cache-name preservation.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- cargo run -p zero-wpt-runner -- testharness-service-workers-cache-storage service-workers/cache-storage/cache-storage.https.any.js --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --json`: 1 case / 11 subtests / 11 Pass
- `make audit-wpt-service-workers-cache-storage-wave`: 38 assets matched pinned manifest
- `python3 tests/wpt-runner/scripts/audit-service-worker-disposition.py --write`: core=52 / defer=34 / gated=166 / skip=42
- `make baseline-wpt-service-workers-cache-storage OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md TIME_LIMIT=420`: 15 cases / 171 subtests / 171 Pass, double-run deterministic

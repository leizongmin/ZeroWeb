# CacheStorage Window WPT Baseline

- Date: 2026-08-22
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Cases: 4
- Subtests: 35
- Deterministic double run: true

## Status Counts

| Status | Count |
|---|---:|
| Fail | 2 |
| Pass | 33 |

## Notes

This is the first pinned window-environment CacheStorage baseline. Failures are preserved as baseline data for follow-up semantic work; the script only requires the case set and status mapping to be stable between consecutive runs.

## Non-Pass Subtests

| Case | Subtest | Status | Message |
|---|---|---|---|
| `service-workers/cache-storage/cache-storage.https.any.js` | CacheStorage.delete dooms, but does not delete immediately | Fail | assert_equals: expected -1 but got 1 |
| `service-workers/cache-storage/cache-storage.https.any.js` | CacheStorage names are DOMStrings not USVStrings | Fail | promise_test: Unhandled rejection with value: object "TypeError: TypeError: invalid CacheStorage request: unexpected end of hex escape at line 1 column 36" |

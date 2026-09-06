# M3 Service Worker Onactivate Script Error

- Date: 2026-09-03
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/onactivate-script-error.https.html`
- Support:
  - `service-workers/service-worker/resources/onactivate-throw-error-worker.js`
  - `service-workers/service-worker/resources/onactivate-throw-error-with-empty-onerror-worker.js`
  - `service-workers/service-worker/resources/onactivate-throw-error-from-nested-event-worker.js`
  - `service-workers/service-worker/resources/onactivate-throw-error-then-cancel-worker.js`
  - `service-workers/service-worker/resources/onactivate-throw-error-then-prevent-default-worker.js`

## Result

`onactivate-script-error.https.html` is promoted to the Service Worker core
runner. The case covers synchronous exceptions from activate listeners and
nested dispatched events being reported through the worker-global `error`
event without failing activation.

## Verification

- `make test-wpt-service-workers-onactivate-script-error-wave-assets`:
  6 assets / regression PASS
- `cargo test -p zero-wpt-runner service_worker_core_manifest_has_expected_unique_cases -- --nocapture`:
  1 passed
- `make testharness-service-workers-core FILTER=service-worker/onactivate-script-error.https.html TIME_LIMIT=300`:
  1 case / 5 subtests / 5 Pass
- `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-09-03-m3-onactivate-script-error-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-09-03-m3-onactivate-script-error-baseline.md TIME_LIMIT=1200`:
  62 cases / 228 subtests / 228 Pass, deterministic

## Asset Contract

- [2026-09-03-m3-onactivate-script-error-assets.tsv](2026-09-03-m3-onactivate-script-error-assets.tsv)
- [2026-09-03-m3-onactivate-script-error-baseline.md](2026-09-03-m3-onactivate-script-error-baseline.md)
- [2026-09-03-m3-onactivate-script-error-baseline.json](2026-09-03-m3-onactivate-script-error-baseline.json)

## Conclusion

The Service Worker core baseline expands from 61 cases / 223 subtests to
62 cases / 228 subtests. This fixes the WPT disposition lane from `defer` to
`core` for activate listener exception handling.

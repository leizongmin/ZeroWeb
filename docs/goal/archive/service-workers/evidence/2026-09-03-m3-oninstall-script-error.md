# M3 Service Worker Oninstall Script Error

- Date: 2026-09-03
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/oninstall-script-error.https.html`
- Support:
  - `service-workers/service-worker/resources/oninstall-throw-error-worker.js`
  - `service-workers/service-worker/resources/oninstall-throw-error-with-empty-onerror-worker.js`
  - `service-workers/service-worker/resources/oninstall-throw-error-from-nested-event-worker.js`
  - `service-workers/service-worker/resources/oninstall-waituntil-throw-error-worker.js`
  - `service-workers/service-worker/resources/oninstall-throw-error-then-cancel-worker.js`
  - `service-workers/service-worker/resources/oninstall-throw-error-then-prevent-default-worker.js`

## Result

`oninstall-script-error.https.html` is promoted to the Service Worker core
runner. The case covers synchronous exceptions from install listeners and
nested dispatched events being reported through the worker-global `error`
event without failing installation, while rejected `event.waitUntil()`
promises still make install fail.

## Verification

- `./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox install_event_ -- --nocapture`:
  4 passed
- `make test-wpt-service-workers-oninstall-script-error-wave-assets`:
  7 assets / regression PASS
- `make testharness-service-workers-core FILTER=service-worker/oninstall-script-error.https.html TIME_LIMIT=300`:
  1 case / 6 subtests / 6 Pass
- `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-09-03-m3-oninstall-script-error-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-09-03-m3-oninstall-script-error-baseline.md TIME_LIMIT=1200`:
  61 cases / 223 subtests / 223 Pass, deterministic

## Asset Contract

- [2026-09-03-m3-oninstall-script-error-assets.tsv](2026-09-03-m3-oninstall-script-error-assets.tsv)
- [2026-09-03-m3-oninstall-script-error-baseline.md](2026-09-03-m3-oninstall-script-error-baseline.md)
- [2026-09-03-m3-oninstall-script-error-baseline.json](2026-09-03-m3-oninstall-script-error-baseline.json)

## Conclusion

The Service Worker core baseline expands from 60 cases / 217 subtests to
61 cases / 223 subtests. This fixes the WPT disposition lane from `defer` to
`core` for install listener exception handling while preserving
`waitUntil()` rejection as the lifecycle failure signal.

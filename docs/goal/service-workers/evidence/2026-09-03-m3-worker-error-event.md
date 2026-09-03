# M3 Service Worker ErrorEvent

- Date: 2026-09-03
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/ServiceWorkerGlobalScope/service-worker-error-event.https.html`
- Support:
  - `service-workers/service-worker/ServiceWorkerGlobalScope/resources/error-worker.js`

## Result

`service-worker-error-event.https.html` is promoted to the Service Worker core
runner. The case covers exceptions thrown by page-to-worker `message` event
handlers, dispatch of a worker-global `ErrorEvent`, preservation of the thrown
value in `event.error`, script filename/line/column reporting, and the original
`WindowClient` source being usable from the error handler.

The runtime now records the evaluating classic worker script URL/source on
event listeners and dispatches worker-global `error` events when message
callbacks throw. Primitive thrown values use listener source metadata for
filename and throw-site position when the engine does not provide a stack.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox service_worker::tests::page_message_error_listener_observes_thrown_error_event -- --nocapture`:
  1 passed
- `python3 tests/wpt-runner/scripts/audit-service-worker-disposition.py`:
  `core=66 defer=33 fetch=3 gated=150 skip=42`
- `make audit-wpt-service-workers-next-wave`:
  9 assets matched pinned manifest
- `make testharness-service-workers-core FILTER=service-worker-error-event TIME_LIMIT=300`:
  1 Pass
- `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-09-03-m3-worker-error-event-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-03-m3-worker-error-event-baseline.md TIME_LIMIT=1200`:
  53 cases / 201 subtests / 201 Pass, double-run deterministic

## Asset Contract

- [2026-08-19-m1-next-wave-assets.tsv](2026-08-19-m1-next-wave-assets.tsv)

## Conclusion

The Service Worker core baseline expands from 52 cases / 200 subtests to
53 cases / 201 subtests. This fixes the WPT disposition lane from `defer` to
`core` for worker-global `ErrorEvent` dispatch from message handler failures.

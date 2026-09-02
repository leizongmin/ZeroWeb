# M3 Service Worker MessageEvent Ports

- Date: 2026-09-02
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/ServiceWorkerGlobalScope/message-event-ports.https.html`
- Support: `service-workers/service-worker/ServiceWorkerGlobalScope/message-event-ports-worker.js`

## Result

`message-event-ports.https.html` is promoted to the Service Worker core runner.
The case registers a real Service Worker, posts a message with transferred
ports, and verifies that the worker-side `MessageEvent.ports` getter returns
the same array object across repeated reads.

The promotion is a coverage expansion over the existing message event runtime:
no runtime behavior changed in this batch.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-service-workers --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root message-event-ports.https.html --json`:
  1 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- make baseline-wpt-service-workers-core`:
  50 cases / 194 subtests / 194 Pass / deterministic true

## Asset Contract

- [2026-09-02-m3-message-event-ports-assets.tsv](2026-09-02-m3-message-event-ports-assets.tsv)

## Conclusion

The Service Worker core baseline expands from 49 cases / 193 subtests to
50 cases / 194 subtests. This fixes the WPT disposition lane from `gated` to
`core` for the worker-global message ports identity check.

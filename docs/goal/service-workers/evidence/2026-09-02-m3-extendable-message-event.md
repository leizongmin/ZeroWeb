# M3 Service Worker ExtendableMessageEvent

- Date: 2026-09-02
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/ServiceWorkerGlobalScope/extendable-message-event.https.html`
- Support:
  - `service-workers/service-worker/ServiceWorkerGlobalScope/resources/extendable-message-event-ping-worker.js`
  - `service-workers/service-worker/ServiceWorkerGlobalScope/resources/extendable-message-event-pong-worker.js`
  - `service-workers/service-worker/ServiceWorkerGlobalScope/resources/extendable-message-event-utils.js`

## Result

`extendable-message-event.https.html` is promoted to the Service Worker core
runner. The case covers page-to-worker `ExtendableMessageEvent` construction,
nested `WindowClient` source projection, worker loopback messages, and
worker-to-worker `ServiceWorker.postMessage()` delivery between active and
waiting versions of the same registration.

The runtime now routes Service Worker peer messages through the manager-owned
registration slots, syncs active/waiting peer projections before dispatch, and
preserves page client frame/focus metadata when constructing worker-side
message event sources.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-service-workers --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root ServiceWorkerGlobalScope/extendable-message-event.https.html --json`:
  4 Pass

## Asset Contract

- [2026-09-02-m3-extendable-message-event-assets.tsv](2026-09-02-m3-extendable-message-event-assets.tsv)

## Conclusion

The Service Worker core baseline expands from 50 cases / 194 subtests to
51 cases / 198 subtests. This fixes the WPT disposition lane from `gated` to
`core` for worker-global extendable message dispatch across page, nested
client, active worker, and waiting worker sources.

# M3 Service Worker PostMessage Worker Ports

- Date: 2026-09-02
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/ServiceWorkerGlobalScope/postmessage.https.html`
- Support:
  - `service-workers/service-worker/ServiceWorkerGlobalScope/resources/postmessage-loopback-worker.js`
  - `service-workers/service-worker/ServiceWorkerGlobalScope/resources/postmessage-ping-worker.js`
  - `service-workers/service-worker/ServiceWorkerGlobalScope/resources/postmessage-pong-worker.js`

## Result

`postmessage.https.html` is promoted to the Service Worker fetch/message runner.
The case covers `ServiceWorker.postMessage()` with transferred `MessagePort`
endpoints in two paths: loopback delivery through `registration.active`, and
active-to-waiting worker delivery followed by a reply over the transferred port
back to the original page port.

The runtime now preserves local self-posted transferred ports, tags
worker-owned transferred endpoints with their owning registration, and lets the
manager route worker-owned `MessagePort.postMessage()` replies back into the
owning worker runtime before page-owned ports are delivered through the normal
client message queue.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo test -p zero-script-sandbox worker_message_dispatch_transfers_message_ports -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo test -p zero-page-runtime worker_to_worker_transferred_port_can_reply_to_page_client -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-service-workers-fetch --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root ServiceWorkerGlobalScope/postmessage.https.html --json`: 2 Pass

## Asset Contract

- `docs/goal/archive/service-workers/evidence/2026-08-22-m2-fetch-request-end-to-end-assets.tsv` now includes the case and three worker scripts for this WPT.

## Conclusion

The Service Worker fetch/message baseline expands from 27 cases / 71 subtests
to 28 cases / 73 subtests. This fixes the WPT disposition lane from `gated` to
`fetch/message` for worker-global postMessage delivery over transferred
MessagePort endpoints.

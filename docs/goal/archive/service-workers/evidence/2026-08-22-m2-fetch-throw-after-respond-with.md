---
date: 2026-08-22
modules: service-workers,script-sandbox
---

# M2 Service Worker Fetch Throw After respondWith Guard

## Scope

This slice fixes the Service Worker runtime FetchEvent settlement path when a
fetch handler synchronously throws after it has already called
`event.respondWith(response)`.

Before this change, the runtime catch branch treated that later throw as an
immediate failed fetch even though the response promise had already been
committed through `respondWith()`. The corrected behavior keeps the committed
response promise authoritative. A synchronous exception only immediately fails
the fetch when `respondWith()` was never called.

The follow-up iframe navigation slice makes the upstream WPT pass end to end.
Controlled iframe navigation fetches now use an async completion path so the
page can keep pumping MessagePort tasks while the worker `respondWith()` promise
is pending. During that pending fetch window, unaddressed worker-to-client
messages are routed to the single pending fetch client, which lets the WPT page
receive the worker's `SYNC` message and send its `ACK` before the iframe load
settles.

## WPT Baseline

The upstream candidate
`service-workers/service-worker/fetch-event-throws-after-respond-with.https.html`
is imported at WPT revision `24197a11e8c5bd29a5cb7bdf18135a82be8a8546`.

Imported assets:

- `service-workers/service-worker/fetch-event-throws-after-respond-with.https.html`
  - bytes: `1392`
  - blob SHA: `d98fb22ff423271dd84460200ebdb60573ed6371`
- `service-workers/service-worker/resources/respond-then-throw-worker.js`
  - bytes: `999`
  - blob SHA: `adb48de69e72351ab0aca1df5be3757da0a93796`

The case is now part of `SERVICE_WORKER_FETCH_CASES`. It raises the fetch-wave
baseline to 15 cases / 34 subtests / 34 Pass.

## Verification

Targeted checks run for this slice:

```sh
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox fetch_event_throw_after_respond_with_keeps_committed_response -- --nocapture
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 240 -- cargo test -p zero-webview controlled_iframe_fetch_waits_for_message_port_backed_response -- --nocapture
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- cargo run -p zero-wpt-runner -- testharness-service-workers-fetch --wpt-data "$HOME/github/others/wpt" fetch-event-throws-after-respond-with.https.html
```

Result:

- `zero-script-sandbox` throw-after-`respondWith()` runtime test: 1 passed
- `zero-webview` controlled iframe MessagePort-backed response test: 1 passed
- focused WPT `fetch-event-throws-after-respond-with.https.html`: 1 Pass

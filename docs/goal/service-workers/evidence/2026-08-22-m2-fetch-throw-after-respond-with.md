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

## WPT Probe

The upstream candidate
`service-workers/service-worker/fetch-event-throws-after-respond-with.https.html`
was probed at WPT revision `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`.

Candidate assets checked:

- `service-workers/service-worker/fetch-event-throws-after-respond-with.https.html`
  - bytes: `1392`
  - blob SHA: `d98fb22ff423271dd84460200ebdb60573ed6371`
- `service-workers/service-worker/resources/respond-then-throw-worker.js`
  - bytes: `999`
  - blob SHA: `adb48de69e72351ab0aca1df5be3757da0a93796`

The runtime-level semantic defect is covered by a new unit test, but the full
WPT case is not added to `SERVICE_WORKER_FETCH_CASES` yet. The current WPT run
still fails before the final assertion because the controlled iframe document
load path exposes a null `contentDocument.body`:

```text
promise_test: Unhandled rejection with value: object "TypeError: Cannot read properties of null (reading 'body')"
```

That remaining issue appears to be an iframe document materialization/load
timing gap, not the FetchEvent throw-after-`respondWith()` settlement bug fixed
in this slice.

## Verification

Targeted checks run for this slice:

```sh
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox fetch_event_throw_after_respond_with_keeps_committed_response -- --nocapture
```

Result:

- `zero-script-sandbox` throw-after-`respondWith()` runtime test: 1 passed

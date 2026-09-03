# M3 Service Worker MessageError Event

- Date: 2026-09-03
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/ServiceWorkerGlobalScope/error-message-event.https.html`
- Support:
  - `service-workers/service-worker/ServiceWorkerGlobalScope/error-message-event-worker.js`

## Result

`error-message-event.https.html` is promoted to the Service Worker core runner.
The case covers a page posting a canvas capture `MediaStreamTrack` to a Service
Worker, the worker receiving `messageerror` instead of a normal `message`, and
the worker replying to the originating `WindowClient`.

The page shim now refreshes window named element access after DOM callbacks are
registered, exposes a minimal `HTMLCanvasElement.captureStream()` surface for
canvas video tracks, and marks those synthetic tracks as not deserializable in
Service Worker globals. The Service Worker runtime recognizes that marker and
dispatches an `ExtendableMessageEvent` named `messageerror` with the originating
window client as `event.source`.

## Verification

- `./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_canvas_capture_stream_track_marks_service_worker_messageerror -- --nocapture`:
  1 passed
- `./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_service_worker_post_message_routes_canvas_track_to_messageerror -- --nocapture`:
  1 passed
- `./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox page_message_error_marker_dispatches_messageerror_event -- --nocapture`:
  1 passed
- `make testharness-service-workers-core FILTER=error-message-event TIME_LIMIT=300`:
  1 Pass

## Asset Contract

- [2026-08-19-m1-tier-a-assets.tsv](2026-08-19-m1-tier-a-assets.tsv)

## Conclusion

The Service Worker core baseline expands from 53 cases / 201 subtests to
54 cases / 202 subtests. This fixes the WPT disposition lane from `gated` to
`core` for worker-global `messageerror` dispatch from page messages that cannot
be deserialized in the worker context.

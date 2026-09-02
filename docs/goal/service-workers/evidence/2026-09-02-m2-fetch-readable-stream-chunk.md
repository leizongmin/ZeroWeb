# M2 Fetch ReadableStream Chunk WPT

- Date: 2026-09-02
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/fetch-event-respond-with-readable-stream-chunk.https.html`
- Support: `service-workers/service-worker/resources/fetch-event-respond-with-readable-stream-chunk-worker.js`

## Result

`fetch-event-respond-with-readable-stream-chunk.https.html` is promoted to the
Service Worker fetch/message runner. The case registers a real Service Worker,
loads a controlled iframe, and verifies that `respondWith(new Response(stream))`
serializes a `ReadableStream` whose pull source emits empty and non-empty
`Uint8Array` chunks into the controlled iframe fetch body.

The promoted case keeps the existing non-streaming body transfer contract: the
page consumes the response with `response.text()`. It does not require
`response.body` forwarding, page-side reader cancellation, or abort propagation.

## Probe Notes

Adjacent cases were probed but not promoted:

- `fetch-event-respond-with-body-loaded-in-chunk.https.html`: later promoted by
  adding a deterministic `trickle.py` network fixture for the loaded body path.
- `fetch-event-respond-with-response-body-with-invalid-chunk.https.html`: later
  promoted after the Service Worker runtime started transferring
  non-`Uint8Array` stream chunks as page-side body errors.

`fetch-event-respond-with-partial-stream.https.html` was left gated because it
explicitly requires incremental `response.body.getReader()` delivery before the
worker closes the stream, which belongs with the broader streaming/cancel
follow-up.

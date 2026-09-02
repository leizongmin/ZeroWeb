# M2 Fetch Body Loaded In Chunk WPT

- Date: 2026-09-02
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/fetch-event-respond-with-body-loaded-in-chunk.https.html`
- Support: `service-workers/service-worker/resources/fetch-event-respond-with-body-loaded-in-chunk-worker.js`
- Dynamic fixture: `fetch/api/resources/trickle.py`

## Result

`fetch-event-respond-with-body-loaded-in-chunk.https.html` is promoted to the
Service Worker fetch/message runner. The case registers a real Service Worker,
loads a controlled iframe, performs a worker-side network `fetch()` for a body
that arrives in chunks, and verifies that `respondWith(new Response(body))`
returns the full chunk-loaded body to the controlled iframe fetch.

The WPT runner now provides a deterministic local equivalent for
`fetch/api/resources/trickle.py`, returning `TEST_TRICKLE\n` repeated by the
requested `count` query parameter. This keeps the fixture local to the pinned WPT
asset corpus while preserving the observable body needed by the WPT.

The runner also treats missing `service-workers/service-worker/resources/*.html`
scope documents as empty 404 HTML responses instead of synthetic fetch failures.
That preserves the document/client creation shape used by this WPT's nonexistent
scope iframe URL without widening general missing-file behavior.

## Probe Notes

`fetch-event-respond-with-partial-stream.https.html` remains gated. It requires
incremental page-side `response.body.getReader()` delivery before the worker
closes the stream, plus cancel/abort propagation, and belongs with the broader
streaming response body follow-up.

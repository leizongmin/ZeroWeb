# M2 Fetch Invalid Stream Chunk WPT

- Date: 2026-09-02
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/fetch-event-respond-with-response-body-with-invalid-chunk.https.html`
- Support:
  - `service-workers/service-worker/resources/fetch-event-respond-with-response-body-with-invalid-chunk-worker.js`
  - `service-workers/service-worker/resources/fetch-event-respond-with-response-body-with-invalid-chunk-iframe.html`

## Result

`fetch-event-respond-with-response-body-with-invalid-chunk.https.html` is
promoted to the Service Worker fetch/message runner. The case registers a real
Service Worker, loads a controlled iframe, and verifies that
`respondWith(new Response(stream))` transfers a response body stream with a
non-`Uint8Array` chunk as an errored page-side body. The page `fetch()` resolves
to a `Response`, while `response.body.getReader().read()` rejects with the
iframe realm `TypeError`.

The runtime now keeps `BodyInit` string coercion for direct `Response("text")`
bodies, but enforces the Fetch body stream chunk boundary while serializing a
Service Worker `ReadableStream` response body.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox fetch_event_readable_stream_invalid_chunk_errors_body -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner service_worker_fetch_manifest_has_request_end_to_end_case -- --nocapture`: 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 120 -- make audit-wpt-service-workers-fetch-wave`: 80 assets matched pinned manifest
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make testharness-service-workers-fetch FILTER=fetch-event-respond-with-response-body-with-invalid-chunk.https.html`: 1 case / 1 subtest / 1 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/service-workers/evidence/2026-09-02-m2-fetch-invalid-stream-chunk-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-02-m2-fetch-invalid-stream-chunk-baseline.md TIME_LIMIT=900`: 30 cases / 75 subtests / 75 Pass, double-run deterministic
- `BINDGEN_EXTRA_CLANG_ARGS='-isystem /usr/lib/gcc/x86_64-linux-gnu/13/include' ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`: passed

## Follow-Up

`fetch-event-respond-with-partial-stream.https.html` remains gated because it
requires incremental `response.body.getReader()` delivery before the Service
Worker closes the body stream, plus cancel/abort propagation across the
runtime/page boundary.

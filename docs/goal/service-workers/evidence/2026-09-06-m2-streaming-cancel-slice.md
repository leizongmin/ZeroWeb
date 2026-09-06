# M2-44 Streaming/Cancel Slice

- Date: 2026-09-06
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Scope: promote `service-workers/service-worker/fetch-event-respond-with-readable-stream.https.html`
  (full case, 10 subtests) into the pinned Service Worker fetch/message runner.

## What landed

1. **Body-cancel back-propagation (cross-runtime)**: `respondWith()` stream bodies are
   registered per event id in the runtime. A new typed `CancelFetchBody` command
   (runtime ↔ manager ↔ host trait) invokes the stream's cancel algorithm, making the
   underlying source `cancel()` callback observable. The in-process WebView path bridges
   page `response.body.cancel()` / settle-time `AbortController.abort()` through an
   internal `X-Zero-Sw-Fetch-Id` header + `__zw_sw_fetch_body_cancel` host callback.
   The browser IPC host keeps a default no-op (production bodies are not streamed yet;
   documented, not hidden).
2. **Worker `ReadableStream.cancel()`**: the worker runtime stream class previously had
   no `cancel` method at all; added per Streams spec (locked → reject; otherwise close +
   `source.cancel(reason)`).
3. **Pump watchdog for open streams**: `respondWith()` serialization now abandons a
   stream that makes no progress after 64 task-pump iterations, delivering accumulated
   bytes. Implemented as a pump-count watchdog rather than `setTimeout` because the
   runtime timer queue is FIFO without delay ordering (a delay-ordered timer jumped the
   queue and broke `fetch-error`'s error-before-abandon ordering; regression caught and
   fixed in the same slice).
4. **Settlement semantics aligned with spec**: when `respondWith()` was called,
   `FetchSettled` no longer waits for `ExtendableEvent.waitUntil()` lifetime — per
   https://w3c.github.io/ServiceWorker/#fetch-event-respondwith the response is used as
   soon as the respondWith promise fulfills; `waitUntil()` only extends the event
   lifetime. Without this, the observe-cancel subtests deadlocked: the worker's
   `waitUntil` awaited a query fetch that the page could only issue after the first
   fetch settled.

## Result

`testharness-service-workers-fetch`: 30 cases / 75 subtests → **31 cases / 85 subtests,
all Pass, deterministic across consecutive runs.** No regression in core (249/249) or
CacheStorage (318/318) runners.

## Known flake (pre-existing, unrelated)

`skip-waiting-using-registration.https.html` intermittently fails
(~15–25%: "Controller state should be activating expected activating but got activated")
on local full-core runs; reproduced on clean HEAD before this slice. Same-family timing
race in the controller-change state projection; left as a dedicated follow-up slice.

## Verification

- `make testharness-service-workers-fetch`: 85 Pass / 0 Fail (double run)
- `make testharness-service-workers-core`: 249 Pass (flaky case excluded above)
- `make testharness-service-workers-cache-storage`: 318 Pass
- `cargo test -p zero-script-sandbox` (V8): 259 passed incl. new
  `cancel_fetch_body_invokes_stream_source_cancel`
- `cargo test -p zero-script-sandbox --no-default-features --features quickjs`: 180 passed
- `cargo clippy -p zero-script-sandbox -p zero-page-runtime -p zero-webview --all-targets -- -D warnings`: clean
- `cargo fmt` clean

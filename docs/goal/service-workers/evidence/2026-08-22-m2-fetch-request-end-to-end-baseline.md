# Service Worker Fetch WPT Baseline

- Date: 2026-08-22
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Cases: 2
- Subtests: 5
- Pass: 5
- Fail: 0
- Timeout: 0
- Unsupported: 0
- Deterministic: true

## Scope

This pinned Service Worker M2 fetch/interception baseline covers two cases. `request-end-to-end.https.html` registers a real service worker, loads a controlled iframe, dispatches a FetchEvent, and validates the Request projection returned via `respondWith(new Response(...))`. `fetch-event-async-respond-with.https.html` fixes the FetchEvent `respondWith()` timing boundary: calls from the dispatch microtask checkpoint are accepted, while later task calls throw `InvalidStateError`.

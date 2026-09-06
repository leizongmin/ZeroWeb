# Service Worker Fetch WPT Baseline

- Date: 2026-08-22
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Cases: 5
- Subtests: 8
- Pass: 8
- Fail: 0
- Timeout: 0
- Unsupported: 0
- Deterministic: true

## Scope

This pinned Service Worker M2 fetch/interception baseline covers five cases. `request-end-to-end.https.html` registers a real service worker, loads a controlled iframe, dispatches a FetchEvent, and validates the Request projection returned via `respondWith(new Response(...))`. `fetch-event-async-respond-with.https.html` fixes the FetchEvent `respondWith()` timing boundary: calls from the dispatch microtask checkpoint are accepted, while later task calls throw `InvalidStateError`. `fetch-event-network-error.https.html` covers rejected `respondWith()`, `preventDefault()` without `respondWith()`, consumed response body network errors, and pass-through after a thrown fetch handler. `fetch-event-respond-with-argument.https.html` covers Response, Promise<Response>, and invalid non-Response arguments producing a network error. `iso-latin1-header.https.html` covers synthetic `respondWith()` response headers with ISO-8859-1 values flowing through the controlled iframe's XMLHttpRequest path.

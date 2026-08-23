# Service Worker Fetch WPT Baseline

- Date: 2026-08-23
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Additional WPT revision: `24197a11e8c5bd29a5cb7bdf18135a82be8a8546` for `fetch-event-within-sw.https.html` and its new support assets
- Cases: 7
- Subtests: 12
- Pass: 12
- Fail: 0
- Timeout: 0
- Unsupported: 0
- Deterministic: true

## Scope

This pinned Service Worker M2 fetch/interception baseline covers seven cases. `request-end-to-end.https.html` registers a real service worker, loads a controlled iframe, dispatches a FetchEvent, and validates the Request projection returned via `respondWith(new Response(...))`. `fetch-event-add-async.https.html` verifies that adding a fetch listener from a later Service Worker task is accepted. `fetch-event-async-respond-with.https.html` fixes the FetchEvent `respondWith()` timing boundary: calls from the dispatch microtask checkpoint are accepted, while later task calls throw `InvalidStateError`. `fetch-event-within-sw.https.html` covers controlled-window `fetch()` and `Cache.add()` interception while worker-global `fetch()` / `Cache.add()` requests do not self-intercept. `fetch-event-network-error.https.html` covers rejected `respondWith()`, `preventDefault()` without `respondWith()`, consumed response body network errors, and pass-through after a thrown fetch handler. `fetch-event-respond-with-argument.https.html` covers Response, Promise<Response>, and invalid non-Response arguments producing a network error. `iso-latin1-header.https.html` covers synthetic `respondWith()` response headers with ISO-8859-1 values flowing through the controlled iframe's XMLHttpRequest path.

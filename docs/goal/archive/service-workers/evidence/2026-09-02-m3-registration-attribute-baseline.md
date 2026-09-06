# Service Worker Core WPT Baseline

- Date: 2026-09-02
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Cases: 52
- Subtests: 200
- Pass: 200
- Fail: 0
- Timeout: 0
- Unsupported: 0
- Deterministic: true

## Scope

This pinned Service Worker core baseline covers registration, update, install/activate lifecycle, worker-global APIs, messaging, module script loading, skipWaiting, and updateViaCache cases. `ServiceWorkerGlobalScope/registration-attribute.https.html` extends the baseline with worker-global `registration.scope`, `registration.installing/waiting/active` slot visibility, registration and worker `EventTarget` methods, `updatefound`, and `statechange` ordering across initial and replacement workers.

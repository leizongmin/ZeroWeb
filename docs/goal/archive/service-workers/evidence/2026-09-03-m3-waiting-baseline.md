# Service Worker Core WPT Baseline

- Date: 2026-09-03
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Cases: 59
- Subtests: 216
- Pass: 216
- Fail: 0
- Timeout: 0
- Unsupported: 0
- Deterministic: true

## Scope

This pinned Service Worker core baseline covers registration, update, install/activate lifecycle, worker-global APIs, messaging, module script loading, skipWaiting, and updateViaCache cases. `ServiceWorkerGlobalScope/registration-attribute.https.html` extends the baseline with worker-global `registration.scope`, `registration.installing/waiting/active` slot visibility, registration and worker `EventTarget` methods, `updatefound`, and `statechange` ordering across initial and replacement workers. `ServiceWorkerGlobalScope/service-worker-error-event.https.html` extends it with worker-global `ErrorEvent` dispatch for message handler failures. `ServiceWorkerGlobalScope/error-message-event.https.html` extends it with worker-global `messageerror` dispatch for unserializable page messages. `controller-on-load.https.html` extends it with newly loaded controlled iframe controller projection and iframe-realm registration worker identity. `getregistration.https.html` extends it with same-origin document URL validation, fragment-insensitive lookup, and unregistered controlled iframe discovery. `registration-iframe.https.html` extends it with iframe-global scriptURL/scope parsing and immediate installing worker slot visibility. `installing.https.html` extends it with the top-level registration installing slot visibility and SameObject identity across `getRegistration()`. `waiting.https.html` extends it with top-level and iframe registration waiting slot visibility, controller nullability, and waiting/active SameObject identity.

# M3 Service Worker Controller On Load

- Date: 2026-09-03
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/controller-on-load.https.html`
- Support:
  - `service-workers/service-worker/resources/blank.html`
  - `service-workers/service-worker/resources/empty-worker.js`
  - `service-workers/service-worker/resources/test-helpers.sub.js`
  - `service-workers/service-worker/resources/testharness-helpers.js`

## Result

`controller-on-load.https.html` is promoted to the Service Worker core runner.
The case covers a newly loaded controlled iframe exposing
`navigator.serviceWorker.controller` during load, and verifies that
`navigator.serviceWorker.getRegistration()` from the iframe global resolves to
the same iframe-realm `ServiceWorker` wrapper as `controller`.

The iframe Service Worker container now resolves default `getRegistration()`
lookups against the iframe document URL rather than the parent page URL. Iframe
registration wrappers also project `installing`, `waiting`, and `active`
through iframe-realm `ServiceWorker` wrappers, so object identity is consistent
inside the child window while remaining distinct from the parent realm.

## Verification

- `./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-webview iframe_get_registration_uses_iframe_url_and_realm_workers -- --nocapture`:
  1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-service-workers --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root controller-on-load.https.html --json`:
  1 Pass

## Asset Contract

- [2026-09-03-m3-controller-on-load-assets.tsv](2026-09-03-m3-controller-on-load-assets.tsv)

## Conclusion

The Service Worker core baseline expands from 54 cases / 202 subtests to
55 cases / 203 subtests. This fixes the WPT disposition lane from `defer` to
`core` for controller projection on newly loaded controlled iframes.

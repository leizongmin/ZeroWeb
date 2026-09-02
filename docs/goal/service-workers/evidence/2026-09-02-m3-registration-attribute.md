# M3 Service Worker Registration Attribute

- Date: 2026-09-02
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/ServiceWorkerGlobalScope/registration-attribute.https.html`
- Support:
  - `service-workers/service-worker/ServiceWorkerGlobalScope/resources/registration-attribute-worker.js`
  - `service-workers/service-worker/ServiceWorkerGlobalScope/resources/registration-attribute-newer-worker.js`

## Result

`registration-attribute.https.html` is promoted to the Service Worker core
runner. The case covers the worker-global `registration` attribute across the
initial worker and a replacement worker: `registration.scope`,
`registration.{installing,waiting,active}` slot visibility, registration and
worker `EventTarget` methods, `onupdatefound`, and `statechange` ordering.

The runtime now seeds each worker evaluation with the registration scope and the
incumbent slot peers available before the new installing candidate is exposed.
Lifecycle dispatch syncs peers before install/activate events, and state changes
flow through the worker-side `ServiceWorker` event target so page-visible slot
updates do not mask lifecycle `statechange` events.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- cargo run -p zero-wpt-runner -- testharness-service-workers registration-attribute --wpt-data /tmp/zw-wpt-cache-storage --json`:
  2 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- ./target/release/zero-wpt-runner testharness-service-workers --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root skip-waiting-using-registration.https.html --json`:
  12 consecutive runs, 2 Pass each
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-09-02-m3-registration-attribute-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-02-m3-registration-attribute-baseline.md TIME_LIMIT=1200`:
  52 cases / 200 subtests / 200 Pass, double-run deterministic

## Asset Contract

- [2026-09-02-m3-registration-attribute-assets.tsv](2026-09-02-m3-registration-attribute-assets.tsv)
- [2026-09-02-m3-registration-attribute-baseline.md](2026-09-02-m3-registration-attribute-baseline.md)

## Conclusion

The Service Worker core baseline expands from 51 cases / 198 subtests to
52 cases / 200 subtests. This fixes the WPT disposition lane from `gated` to
`core` for worker-global registration attribute and lifecycle slot observation.

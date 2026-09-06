# M3 Service Worker Controller On Disconnect

- Date: 2026-09-03
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/controller-on-disconnect.https.html`
- Support:
  - `service-workers/service-worker/resources/blank.html`
  - `service-workers/service-worker/resources/empty-worker.js`
  - `service-workers/service-worker/resources/test-helpers.sub.js`
  - `service-workers/service-worker/resources/testharness-helpers.js`

## Result

`controller-on-disconnect.https.html` is promoted to the Service Worker core
runner. The case covers a controlled iframe exposing a `ServiceWorker`
controller while connected, then clearing `navigator.serviceWorker.controller`
after the iframe is removed from the document.

## Verification

- `make testharness-service-workers-core FILTER=service-worker/controller-on-disconnect.https.html TIME_LIMIT=300`:
  1 case / 1 subtest / 1 Pass
- `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-09-03-m3-controller-on-disconnect-baseline.json SUMMARY=docs/goal/archive/service-workers/evidence/2026-09-03-m3-controller-on-disconnect-baseline.md TIME_LIMIT=1200`:
  60 cases / 217 subtests / 217 Pass, deterministic

## Asset Contract

- [2026-09-03-m3-controller-on-disconnect-assets.tsv](2026-09-03-m3-controller-on-disconnect-assets.tsv)
- [2026-09-03-m3-controller-on-disconnect-baseline.md](2026-09-03-m3-controller-on-disconnect-baseline.md)
- [2026-09-03-m3-controller-on-disconnect-baseline.json](2026-09-03-m3-controller-on-disconnect-baseline.json)

## Conclusion

The Service Worker core baseline expands from 59 cases / 216 subtests to
60 cases / 217 subtests. This fixes the WPT disposition lane from `defer` to
`core` for clearing controller projection on detached iframe globals.

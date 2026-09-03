# M3 Service Worker ExtendableEvent waitUntil

- Date: 2026-09-03
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/extendable-event-waituntil.https.html`
- Support:
  - `service-workers/service-worker/resources/extendable-event-waituntil.js`

## Result

`extendable-event-waituntil.https.html` is promoted to the Service Worker
core runner. The case covers install/activate `ExtendableEvent.waitUntil()`
lifetime promise ordering, multiple waitUntil promises, install rejection, and
activate waitUntil rejection continuing to activated state.

## Verification

- `make test-wpt-service-workers-extendable-event-waituntil-wave-assets`:
  2 assets / regression PASS
- `make testharness-service-workers-core FILTER=service-worker/extendable-event-waituntil.https.html TIME_LIMIT=300`:
  1 case / 6 subtests / 6 Pass
- `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-09-03-m3-extendable-event-waituntil-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-03-m3-extendable-event-waituntil-baseline.md TIME_LIMIT=1200`:
  63 cases / 234 subtests / 234 Pass, deterministic

## Asset Contract

- [2026-09-03-m3-extendable-event-waituntil-assets.tsv](2026-09-03-m3-extendable-event-waituntil-assets.tsv)
- [2026-09-03-m3-extendable-event-waituntil-baseline.md](2026-09-03-m3-extendable-event-waituntil-baseline.md)
- [2026-09-03-m3-extendable-event-waituntil-baseline.json](2026-09-03-m3-extendable-event-waituntil-baseline.json)

## Conclusion

The Service Worker core baseline expands from 62 cases / 228 subtests to
63 cases / 234 subtests. This fixes the WPT disposition lane from `defer` to
`core` for `ExtendableEvent.waitUntil()` lifecycle ordering.

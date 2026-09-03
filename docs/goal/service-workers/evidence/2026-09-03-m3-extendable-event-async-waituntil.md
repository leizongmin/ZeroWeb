# M3 Service Worker ExtendableEvent async waitUntil

- Date: 2026-09-03
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/extendable-event-async-waituntil.https.html`
- Support:
  - `service-workers/service-worker/resources/extendable-event-async-waituntil.js`

## Result

`extendable-event-async-waituntil.https.html` is promoted to the Service
Worker core runner. The case covers `ExtendableEvent.waitUntil()` eligibility
across dispatch-time microtasks, later tasks with an existing lifetime
extension, expired extension windows, script-constructed events, and FetchEvent
`respondWith()` lifetime extension boundaries.

## Runtime Notes

The Service Worker runtime now tracks `waitUntil()` lifetime state per
`ExtendableEvent`, closes the event dispatch flag at the microtask checkpoint,
and records promise rejection with an explicit flag so `reject(undefined)` still
fails lifecycle settlement. FetchEvent `respondWith()` participates in the same
lifetime accounting while keeping response serialization independent from
`waitUntil()` settlement.

## Verification

- `make test-wpt-service-workers-extendable-event-async-waituntil-wave-assets`:
  2 assets / regression PASS
- `make testharness-service-workers-core FILTER=service-worker/extendable-event-waituntil.https.html TIME_LIMIT=300`:
  1 case / 6 subtests / 6 Pass
- `make testharness-service-workers-core FILTER=service-worker/extendable-event-async-waituntil.https.html TIME_LIMIT=300`:
  1 case / 14 subtests / 14 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- python3 tests/wpt-runner/scripts/run-service-workers-core-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --output docs/goal/service-workers/evidence/2026-09-03-m3-extendable-event-async-waituntil-baseline.json --summary docs/goal/service-workers/evidence/2026-09-03-m3-extendable-event-async-waituntil-baseline.md`:
  64 cases / 248 subtests / 248 Pass, deterministic
- `python3 tests/wpt-runner/scripts/audit-service-worker-disposition.py`:
  PASS, `core=77 defer=23 fetch=3 gated=149 skip=42`

## Asset Contract

- [2026-09-03-m3-extendable-event-async-waituntil-assets.tsv](2026-09-03-m3-extendable-event-async-waituntil-assets.tsv)
- [2026-09-03-m3-extendable-event-async-waituntil-baseline.md](2026-09-03-m3-extendable-event-async-waituntil-baseline.md)
- [2026-09-03-m3-extendable-event-async-waituntil-baseline.json](2026-09-03-m3-extendable-event-async-waituntil-baseline.json)

## Conclusion

The Service Worker core baseline expands from 63 cases / 234 subtests to
64 cases / 248 subtests. This fixes the WPT disposition lane from `defer` to
`core` for async `ExtendableEvent.waitUntil()` task/microtask eligibility.

# M2 CacheStorage `.any.js` Service Worker Batch

- Date: 2026-09-02
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Scope: promote eight top-level CacheStorage `.any.js` cases into the pinned
  Service Worker CacheStorage runner.

## Promoted Cases

| Case | Subtests | Manifest SHA |
|---|---:|---|
| `service-workers/cache-storage/cache-add.https.any.js` | 23 | `c4846eaae40afa38d9bf000169a4917d45a6df81` |
| `service-workers/cache-storage/cache-delete.https.any.js` | 9 | `3eae2b6a08b7954a6a5e644b8248f6ebab562a83` |
| `service-workers/cache-storage/cache-keys.https.any.js` | 17 | `232fb760d4080a02051ba5cf5e8d2964771fda9a` |
| `service-workers/cache-storage/cache-match.https.any.js` | 26 | `9ca45903cbb101810488ced4b31bb9f34957b1ef` |
| `service-workers/cache-storage/cache-matchAll.https.any.js` | 17 | `93c55178918c333df25104e69ca74af5dbf04be9` |
| `service-workers/cache-storage/cache-put.https.any.js` | 28 | `dbf2650a75a5ff051f222ccb736b14f65d98f1b9` |
| `service-workers/cache-storage/cache-storage-keys.https.any.js` | 2 | `f19522be1b437d619b356b5b0290679a8f391866` |
| `service-workers/cache-storage/cache-storage-match.https.any.js` | 12 | `0c31b726294b14b1b849ecbeb369d23a59d126d8` |

## Result

The Service Worker CacheStorage runner now includes nine top-level `.any.js`
Service Worker global variants in total. This batch adds direct worker-global
coverage for `Cache.add()`, `Cache.addAll()`, `Cache.delete()`, `Cache.keys()`,
`Cache.match()`, `Cache.matchAll()`, `Cache.put()`, `CacheStorage.keys()`, and
`CacheStorage.match()` without relying only on the `serviceworker/` HTML wrapper
variants.

The pinned baseline moved from 15 cases / 171 subtests to 23 cases / 305
subtests, all passing and deterministic across consecutive runs. The WPT
disposition contract moved the eight promoted sources from `gated` to `core`,
for lane totals of core=60 / defer=34 / gated=158 / skip=42.

`service-workers/cache-storage/cache-abort.https.any.js` remains gated for now.
Its top-level `.any.js` Service Worker variant reaches the stash-backed abort
subtests, but the local dynamic `stash-take.py` response path currently returns
a non-JSON protocol payload. The existing
`serviceworker/cache-abort.https.html` wrapper continues to cover abort semantics
and remains passing in the same baseline.

## Verification

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo run -p zero-wpt-runner -- testharness-service-workers-cache-storage --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --json`: 23 cases / 305 subtests / 305 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- python3 tests/wpt-runner/scripts/run-service-workers-cache-storage-baseline.py --runner target/debug/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --output docs/goal/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.json --summary docs/goal/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md`: 23 cases / 305 subtests / 305 Pass, double-run deterministic

---
date: 2026-09-03
modules: service-workers, wpt-runner
---

# Service Worker Installing Core Promotion

## Scope

Promote upstream WPT
`service-workers/service-worker/installing.https.html` from
`defer-single-iframe` to the Service Worker core runner.

This slice covers:

- A newly registered top-level Service Worker registration exposing a non-null
  `registration.installing` worker before activation.
- The installing worker reporting the registered script URL.
- `getRegistration(scope)` returning the SameObject installing worker for the
  same underlying service worker.

## Result

The existing runtime and page projection already satisfy the test. No runtime
source change was required; this slice only assets the WPT and promotes it into
the pinned core baseline.

## Evidence

Single-case WPT:

```text
BINDGEN_EXTRA_CLANG_ARGS='-isystem /usr/lib/gcc/x86_64-linux-gnu/13/include' \
make testharness-service-workers-core FILTER=installing TIME_LIMIT=300
```

Result: 1 case / 2 subtests / 2 Pass.

## Asset Contract

- [2026-09-03-m3-installing-assets.tsv](2026-09-03-m3-installing-assets.tsv)

## Conclusion

The Service Worker core baseline expands from 57 cases / 212 subtests to
58 cases / 214 subtests. This fixes the WPT disposition lane from `defer` to
`core` for registration installing slot visibility and SameObject identity.

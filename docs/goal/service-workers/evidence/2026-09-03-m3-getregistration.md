---
date: 2026-09-03
modules: service-workers, webview, js-dom-shim, wpt-runner
---

# Service Worker getRegistration Core Promotion

## Scope

Promote upstream WPT `service-workers/service-worker/getregistration.https.html`
from `defer-single-iframe` to the Service Worker core runner.

This slice covers:

- `navigator.serviceWorker.getRegistration()` resolving `undefined` when no
  registration matches.
- Registered scope lookup and fragment-insensitive document URL matching.
- Cross-origin document URL rejection with `SecurityError`.
- Lookup after `unregister()` from both top-level and controlled iframe
  contexts.

## Fix

The first probe passed 5/6 subtests. The remaining failure was:

```text
getRegistration with a cross origin URL
expected SecurityError, got resolved promise
```

Root cause: the JS shim resolved the supplied document URL but delegated it to
the host without first applying the Service Worker container same-origin
precondition. The host lookup naturally returned no registration for the foreign
origin, but the WPT requires API-level rejection.

The fix adds same-origin validation to both the top-level
`ServiceWorkerContainer.getRegistration()` and the iframe wrapper before calling
`__zw_sw_get_registration`.

## Evidence

Single-case WPT:

```text
BINDGEN_EXTRA_CLANG_ARGS='-isystem /usr/lib/gcc/x86_64-linux-gnu/13/include' \
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- \
  cargo run -p zero-wpt-runner -- testharness-service-workers \
  --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root \
  getregistration --json
```

Result: 1 case / 6 subtests / 6 Pass.

WebView regression:

```text
BINDGEN_EXTRA_CLANG_ARGS='-isystem /usr/lib/gcc/x86_64-linux-gnu/13/include' \
./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 \
  --time-limit 300 -- \
  cargo test -p zero-webview get_registration_rejects_cross_origin_document_url -- --nocapture
```

Result: 1 passed.

## Follow-up

`registration-iframe.https.html` was probed in the same area and passed 2/3
subtests. It remains deferred because the normal-case registration Promise can
resolve after `registration.installing` has already become `null`, which needs a
separate slot visibility/timing fix.

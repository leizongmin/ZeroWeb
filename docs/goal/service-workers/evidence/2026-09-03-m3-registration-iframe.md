---
date: 2026-09-03
modules: service-workers, webview, js-dom-shim, wpt-runner
---

# Service Worker Registration Iframe Core Promotion

## Scope

Promote upstream WPT
`service-workers/service-worker/registration-iframe.https.html` from
`defer-single-iframe` to the Service Worker core runner.

This slice covers:

- `iframe.contentWindow.navigator.serviceWorker.register()` resolving script URL
  and scope against the iframe document URL.
- The resolved registration exposing an immediate `installing` worker in the
  iframe realm.
- Rejection behavior for iframe-relative script/scope inputs that resolve
  outside the allowed scope path.

## Fix

The first probe passed 2/3 subtests. The remaining failure was:

```text
register method should use the "relevant global object" to parse its scriptURL and scope - normal case
wait_for_state needs a ServiceWorker object to be passed
```

Root cause: the iframe `register()` path called the host registration bridge,
then rediscovered the registration through the parent container. That discovery
path can observe the host lifecycle after the installing slot has already
advanced, so the registration Promise could resolve with
`registration.installing === null`.

The fix lets the parent Service Worker container materialize a manual
installing registration snapshot for a newly-created host registration ID, then
wraps that registration in the iframe realm before resolving. Existing
registrations still follow the previous discovery path.

## Evidence

Single-case WPT:

```text
BINDGEN_EXTRA_CLANG_ARGS='-isystem /usr/lib/gcc/x86_64-linux-gnu/13/include' \
make testharness-service-workers-core FILTER=registration-iframe TIME_LIMIT=300
```

Result: 1 case / 3 subtests / 3 Pass.

WebView regression:

```text
BINDGEN_EXTRA_CLANG_ARGS='-isystem /usr/lib/gcc/x86_64-linux-gnu/13/include' \
./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 \
  --time-limit 300 -- \
  cargo test -p zero-webview iframe_register_resolves_with_installing_worker -- --nocapture
```

Result: 1 passed.

## Asset Contract

- [2026-09-03-m3-registration-iframe-assets.tsv](2026-09-03-m3-registration-iframe-assets.tsv)

## Conclusion

The Service Worker core baseline expands from 56 cases / 209 subtests to
57 cases / 212 subtests. This fixes the WPT disposition lane from `defer` to
`core` for iframe registration URL resolution and immediate installing worker
slot visibility.

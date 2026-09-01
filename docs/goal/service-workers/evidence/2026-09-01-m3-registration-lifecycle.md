---
date: 2026-09-01
modules: service-workers,wpt-runner,webview
---

# M3 Registration Lifecycle WPT Expansion

## Summary

Added a focused Service Worker lifecycle wave covering:

- `service-workers/service-worker/registration-events.https.html`
- `service-workers/service-worker/registration-end-to-end.https.html`

Both cases execute through the real Service Worker runtime path and passed as
single-case WPT runs after rebuilding `zero-wpt-runner`.

## Validation

- `make audit-wpt-service-workers-message-lifecycle-wave`: passed, 7 assets
  matched pinned WPT revision `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`.
- `make test-wpt-service-workers-message-lifecycle-wave-assets`: passed, asset
  tamper regression failed closed and restored.
- `cargo test -p zero-webview navigator_register_update_via_cache_noop_keeps_active_projection -- --nocapture`:
  passed, 1 test.
- `zero-wpt-runner testharness-service-workers ... registration-events.https.html`:
  passed, 1/1 subtest.
- `zero-wpt-runner testharness-service-workers ... registration-end-to-end.https.html`:
  passed, 1/1 subtest.

## Follow-up

The full Service Worker core runner was not promoted to a new all-green
baseline in this slice. While diagnosing the expanded corpus, the existing
`registration-updateviacache.https.html` case exposed a long-runner issue in
the full four-value matrix. Reduced `updateViaCache` matrices passed after the
page-side no-op projection fix, but the complete case still needs a separate
focused follow-up before the core expected subtest total can be raised safely.

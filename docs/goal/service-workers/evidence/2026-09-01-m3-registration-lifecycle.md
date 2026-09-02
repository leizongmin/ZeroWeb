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
single-case WPT runs after rebuilding `zero-wpt-runner`. The follow-up
`registration-updateviacache.https.html` hang has since been fixed, so these
cases are now promoted into the full Service Worker core baseline.

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
- `make testharness-service-workers-core`: passed after promotion,
  39 case / 164 subtest / 164 Pass.
- `make baseline-wpt-service-workers-core`: passed after updating the
  baseline shape to 39/164; two consecutive runs matched
  `(case, subtest, status)`.

## Result

The full Service Worker core runner now includes both lifecycle cases. The
baseline increased from 37 case / 162 subtest to 39 case / 164 subtest with
0 Fail, 0 Timeout, and 0 Unsupported.

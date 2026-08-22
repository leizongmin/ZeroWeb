# Service Worker CacheStorage WPT Baseline

- Date: 2026-08-23
- WPT revision: `24197a11e8c5bd29a5cb7bdf18135a82be8a8546`
- Cases: 1
- Subtests: 11
- Pass: 11
- Fail: 0
- Timeout: 0
- Unsupported: 0
- Deterministic: true

## Scope

This pinned Service Worker M2 CacheStorage baseline covers the `serviceworker/cache-storage.https.html` wrapper. It runs the upstream `script-tests/cache-storage.js` in a real Service Worker global and validates `caches.open()`, `CacheStorage.has/delete/keys()`, opened `Cache` identity, delete dooming semantics, empty cache names, required-argument TypeError behavior, worker `Cache.add()`, and DOMString cache-name preservation for unpaired surrogate code units.

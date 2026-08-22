# Service Worker CacheStorage WPT Baseline

- Date: 2026-08-23
- WPT revision: `24197a11e8c5bd29a5cb7bdf18135a82be8a8546`
- Cases: 8
- Subtests: 121
- Pass: 121
- Fail: 0
- Timeout: 0
- Unsupported: 0
- Deterministic: true

## Scope

This pinned Service Worker M2 CacheStorage baseline covers the eight serviceworker CacheStorage wrappers. They run the upstream `script-tests/cache-storage*.js`, `cache-delete.js`, `cache-keys.js`, `cache-match.js`, `cache-matchAll.js`, `cache-put.js`, and `cache-storage-match.js` in a real Service Worker global and validate `caches.open()`, `CacheStorage.has/delete/keys/match()`, opened `Cache` identity, delete dooming semantics, empty cache names, required-argument TypeError behavior, `Cache.match/delete/keys/matchAll/put()`, query option handling, Vary matching, worker `fetch()` response URL/type/body projection, redirect response round-trips, Blob bodies, and DOMString cache-name preservation for unpaired surrogate code units.

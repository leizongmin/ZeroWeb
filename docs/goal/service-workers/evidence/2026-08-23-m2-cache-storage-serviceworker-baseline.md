# Service Worker CacheStorage WPT Baseline

- Date: 2026-09-02
- WPT revisions: `24197a11e8c5bd29a5cb7bdf18135a82be8a8546`, `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Cases: 24
- Subtests: 308
- Pass: 308
- Fail: 0
- Timeout: 0
- Unsupported: 0
- Deterministic: true

## Scope

This pinned Service Worker M2 CacheStorage baseline covers the twelve serviceworker CacheStorage wrappers, ten top-level CacheStorage `.any.js` Service Worker global variants, and the `cache-keys-attributes-for-service-worker.https.html` and `credentials.https.html` pages. They run the upstream `script-tests/cache-storage*.js`, `cache-delete.js`, `cache-keys.js`, `cache-abort.js`, `cache-add.js`, `cache-match.js`, `cache-matchAll.js`, `cache-put.js`, `cache-storage-match.js`, the navigation-attribute service worker fixture, and the credentialed request cache-key fixture in a real Service Worker global and validate `caches.open()`, bucket-scoped CacheStorage, `CacheStorage.has/delete/keys/match()`, opened `Cache` identity, delete dooming semantics, empty cache names, required-argument TypeError behavior, `Cache.match/delete/keys/matchAll/put/add/addAll()`, query option handling, Vary matching, worker `fetch()` response URL/type/body projection, redirect response round-trips, Blob bodies, addAll failure atomicity, Vary-aware duplicate detection, AbortError rejection for aborted `Cache.put/add/addAll()` requests, DOMString cache-name preservation for unpaired surrogate code units, and `Request.isReloadNavigation`/`Request.isHistoryNavigation` preservation through `Cache.put()` and `Cache.keys()`, plus credentialed request URLs preserved through iframe XHR, worker fetch interception, Cache key storage, `Cache.match()`/`Cache.matchAll()`/`CacheStorage.match()`, and worker-to-controlled-iframe `Client.postMessage()`.

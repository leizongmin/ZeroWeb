# CacheStorage Window WPT Baseline

- Date: 2026-08-22
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Cases: 4
- Subtests: 35
- Deterministic double run: true

## Status Counts

| Status | Count |
|---|---:|
| Fail | 20 |
| Pass | 15 |

## Notes

This is the first pinned window-environment CacheStorage baseline. Failures are preserved as baseline data for follow-up semantic work; the script only requires the case set and status mapping to be stable between consecutive runs.

## Non-Pass Subtests

| Case | Subtest | Status | Message |
|---|---|---|---|
| `service-workers/cache-storage/cache-storage.https.any.js` | CacheStorage.delete dooms, but does not delete immediately | Fail | assert_equals: expected -1 but got 0 |
| `service-workers/cache-storage/cache-storage.https.any.js` | CacheStorage.open with no arguments | Fail | assert_unreached: Should have rejected: CacheStorage.open should throw TypeError if called with no arguments. Reached unreachable code |
| `service-workers/cache-storage/cache-storage.https.any.js` | CacheStorage names are DOMStrings not USVStrings | Fail | promise_test: Unhandled rejection with value: object "TypeError: TypeError: invalid CacheStorage request: unexpected end of hex escape at line 1 column 36" |
| `service-workers/cache-storage/cache-storage-keys.https.any.js` | CacheStorage keys | Fail | assert_array_equals: CacheStorage.keys should only return existing caches. expected property 1 to be "example" but got "A" (expected array ["", "example", "Another cache name", "A", "a", "ex ample"] got ["", "A", "Another cache name", "a", "ex ample", "example"]) |
| `service-workers/cache-storage/cache-delete.https.any.js` | Cache.delete with no arguments | Fail | assert_unreached: Should have rejected: Cache.delete should reject with a TypeError when called with no arguments. Reached unreachable code |
| `service-workers/cache-storage/cache-delete.https.any.js` | Cache.delete called with a HEAD request | Fail | assert_class_string: Cache.delete should leave non-matching response in the cache. expected "[object Response]" but got "[object Object]" |
| `service-workers/cache-storage/cache-delete.https.any.js` | Cache.delete supports ignoreVary | Fail | assert_false: Cache.delete should not delete if vary does not match unless ignoreVary is true expected false got true |
| `service-workers/cache-storage/cache-delete.https.any.js` | Cache.delete with ignoreSearch option (request with search parameters) | Fail | assert_class_string: undefined : object[0] expected "[object Response]" but got "[object Object]" |
| `service-workers/cache-storage/cache-delete.https.any.js` | Cache.delete with ignoreSearch option (when it is specified as false) | Fail | assert_class_string: undefined : object[0] expected "[object Response]" but got "[object Object]" |
| `service-workers/cache-storage/cache-keys.https.any.js` | Cache.keys with URL | Fail | assert_class_string: Cache.keys should match by URL. : object[0] expected "[object Request]" but got "[object Object]" |
| `service-workers/cache-storage/cache-keys.https.any.js` | Cache.keys with Request | Fail | assert_class_string: Cache.keys should match by Request. : object[0] expected "[object Request]" but got "[object Object]" |
| `service-workers/cache-storage/cache-keys.https.any.js` | Cache.keys with new Request | Fail | assert_class_string: Cache.keys should match by Request. : object[0] expected "[object Request]" but got "[object Object]" |
| `service-workers/cache-storage/cache-keys.https.any.js` | Cache.keys with ignoreSearch option (request with no search parameters) | Fail | assert_class_string: Cache.keys with ignoreSearch should ignore the search parameters of cached request. : object[0] expected "[object Request]" but got "[object Object]" |
| `service-workers/cache-storage/cache-keys.https.any.js` | Cache.keys with ignoreSearch option (request with search parameters) | Fail | assert_class_string: Cache.keys with ignoreSearch should ignore the search parameters of request. : object[0] expected "[object Request]" but got "[object Object]" |
| `service-workers/cache-storage/cache-keys.https.any.js` | Cache.keys supports ignoreMethod | Fail | assert_class_string: Cache.keys with ignoreMethod should ignore the method of request. : object[0] expected "[object Request]" but got "[object Object]" |
| `service-workers/cache-storage/cache-keys.https.any.js` | Cache.keys supports ignoreVary | Fail | assert_equals: Cache.keys should resolve with an empty array with a mismatched vary. expected 0 but got 1 |
| `service-workers/cache-storage/cache-keys.https.any.js` | Cache.keys with URL containing fragment | Fail | assert_class_string: Cache.keys should ignore URL fragment. : object[0] expected "[object Request]" but got "[object Object]" |
| `service-workers/cache-storage/cache-keys.https.any.js` | Cache.keys without parameters | Fail | assert_class_string: Cache.keys without parameters should match all entries. : object[0] expected "[object Request]" but got "[object Object]" |
| `service-workers/cache-storage/cache-keys.https.any.js` | Cache.keys with explicitly undefined request | Fail | assert_class_string: Cache.keys with undefined request should match all entries. : object[0] expected "[object Request]" but got "[object Object]" |
| `service-workers/cache-storage/cache-keys.https.any.js` | Cache.keys without parameters and VARY entries | Fail | assert_equals: Cache.keys without parameters should match all entries. expected 3 but got 1 |

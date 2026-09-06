# M2-2 Production Page Fetch Routing Evidence

Date: 2026-08-21

## Scope

This slice wires production renderer `FetchRequest` handling in the browser
process to the browser-owned Service Worker manager before the existing network
fallback.

Implemented behavior:

- `ProcessTabBackend::handle_fetch_request()` now attempts Service Worker
  dispatch for committed `http` / `https` documents.
- Dispatch requires the committed source document itself to be controlled by an
  active registration before an in-scope same-origin request can be intercepted.
- The owner correlates page fetch events by profile, registration ID, and
  manager event ID, then converts `respondWith(new Response(...))` into the
  normal renderer `FetchResponse` path.
- Fetches with no active same-origin scoped worker, no `respondWith()`, dispatch
  failure, non-UTF-8 body, `DNS-PREFETCH`, or image streaming metadata continue
  through the existing `TabFetchProxy` network path.
- Browser-owned internal headers are rebuilt from trusted request metadata:
  worker responses cannot spoof `X-Zero-Final-URL` or `X-Zero-Resource-Type`.

## Validation

Targeted command:

```sh
cargo test -p zero-browser service_worker_owner --no-fail-fast
```

Result:

- 47 passed
- 0 failed

Full gate:

```sh
cargo fmt --all -- --check
git diff --check
cargo clippy --workspace --all-targets -- -D warnings
make test
```

Result:

- Passed

## Remaining

This does not complete the umbrella goal:

- Cache API is not yet available inside Service Worker fetch handlers.
- The Service Worker WPT fetch interception lane still needs imported cases and
  a persisted pass-rate report.

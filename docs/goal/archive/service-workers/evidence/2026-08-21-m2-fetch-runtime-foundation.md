# M2-1 fetch runtime foundation evidence

Date: 2026-08-21

## Scope

This slice adds the minimal fetch-event execution path that later production
page fetch plumbing can call.

Implemented behavior:

- `zero-script-sandbox` exposes pure-value `ServiceWorkerFetchRequest` and
  `ServiceWorkerFetchResponse` types.
- Service Worker globals now include MVP `Headers`, `Request`, `Response`,
  `FetchEvent`, `onfetch`, and `__zwDispatchFetch()` support.
- `FetchEvent.respondWith()` must be called during dispatch and only once.
- No `respondWith()` settles as `response: None` so callers can pass through to
  the network path.
- Rejected or invalid `respondWith()` settles as `response: None` with a
  diagnostic message.
- Request/response URLs, methods, headers, bodies, status text, and pending
  fetch count are bounded at the runtime and manager trust boundaries.
- `ServiceWorkerManager::dispatch_fetch()` picks the longest matching active
  same-origin registration and records pending fetch correlation by
  `(registration_id, event_id)`.
- `zero-protocol` carries typed `DispatchFetch` commands and `FetchSettled`
  events between browser owner and renderer runtime host.
- Browser and renderer Service Worker hosts translate fetch command/event values
  without exposing script source over IPC.

## Boundary

This is a foundation slice, not the completed fetch interception milestone.

Still deferred:

- Production page `FetchRequest` routing in `apps/browser/src/process_backend.rs`.
- Turning `FetchSettled.response` into the browser fetch response path.
- Cache API access from the Service Worker fetch handler.
- WPT promotion for upstream fetch-interception cases that require the
  production fetch pipeline and cache integration.

## Verification

Targeted checks run during the slice:

- `cargo test -p zero-script-sandbox fetch_event_ -- --nocapture`
- `cargo test -p zero-page-runtime fetch_dispatch_ -- --nocapture`
- `cargo test -p zero-protocol service_worker_host_fetch_command_and_event_round_trip -- --nocapture`
- `cargo test -p zero-renderer service_worker_host -- --nocapture`
- `cargo check -p zero-page-runtime -p zero-protocol -p zero-browser -p zero-renderer`
- `cargo test -p zero-protocol service_worker_protocol -- --nocapture`
- `cargo test -p zero-renderer fetch_command_dispatches_and_returns_response_event -- --nocapture`
- `cargo fmt --all`
- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `make test`

New focused tests:

- `service_worker::tests::fetch_event_respond_with_serializes_response`
- `service_worker::tests::fetch_event_without_respond_with_passes_through`
- `service_worker::tests::fetch_event_rejects_duplicate_respond_with`
- `service_worker_manager::tests::fetch_dispatch_uses_longest_scope_active_worker`
- `service_worker_manager::tests::fetch_dispatch_passes_through_cross_origin_and_out_of_scope`
- `service_worker_protocol::service_worker_host_fetch_command_and_event_round_trip`
- `service_worker_host::tests::fetch_command_dispatches_and_returns_response_event`

# M3-32 committed top-level client evidence

Date: 2026-08-21

## Scope

This slice wires production navigation commits into the browser-owned Service
Worker window client registry.

Implemented behavior:

- `ProcessTabBackend::handle_navigation_committed()` now records the committed
  top-level Document as a Service Worker `window` client.
- The production client id is derived from `renderer_id:navigation_epoch`, using
  the same identity already used for renderer-originated Service Worker API
  requests.
- `stage_indexed_db_navigation()` still disconnects the tab before a replacement
  navigation, so the old top-level client is removed before the new committed
  Document is observed.
- Non-http(s) committed documents are ignored by the Service Worker client
  registry.

## Boundary

This does not implement real nested iframe or auxiliary popup creation events.
Those still need renderer/protocol signals for browsing-context creation and
destruction. M2 fetch interception remains gated on the js-dom fetch pipeline
and Cache API integration.

## Verification

- `cargo fmt --all -- --check`
- `cargo test -p zero-browser committed_navigation -- --nocapture`
- `cargo test -p zero-browser navigation_replacement -- --nocapture`
- `cargo test -p zero-browser service_worker_owner::tests -- --nocapture`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `make test`

New focused tests:

- `process_backend::service_worker_owner_tests::committed_navigation_observes_top_level_service_worker_client`
- `process_backend::service_worker_owner_tests::navigation_replacement_removes_stale_service_worker_client`

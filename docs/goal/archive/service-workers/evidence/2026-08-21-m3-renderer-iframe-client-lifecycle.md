# M3-34 renderer iframe client lifecycle evidence

Date: 2026-08-21

## Scope

This slice connects renderer-side iframe browsing-context lifecycle events to
the browser-owned Service Worker window client registry.

Implemented behavior:

- Added typed `ObserveWindowClient` and `RemoveWindowClient` Service Worker IPC
  requests. The protocol validates non-empty bounded client fields and accepts
  only `top-level`, `auxiliary`, or `nested` frame types.
- Renderer host callbacks expose `__zw_sw_observe_window_client()` and
  `__zw_sw_remove_window_client()` to the DOM shim, using the existing
  synchronous request router.
- Browser process normalizes renderer-local child client ids under the
  browser-owned committed document id (`renderer_id:navigation_epoch:<child>`),
  preserving empty ids so protocol validation still rejects them.
- Browser owner validates iframe client URLs against the committed document
  authority before recording a `nested` window client.
- Iframe `contentDocument` / `contentWindow` materialization observes a
  `nested` window client after same-origin URL resolution and document creation.
- DOM removal paths that destroy existing iframe subtrees now best-effort
  remove only previously observed iframe Service Worker clients.

## Boundary

This covers real iframe clients produced by the current DOM shim. Auxiliary
popup clients are still deferred because the current headless `window.open()`
path does not create a real browsing context.

M2 fetch interception remains gated on the js-dom fetch pipeline and Cache API
integration.

## Verification

- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo test -p zero-protocol service_worker`
- `cargo test -p zero-browser renderer_window_client_lifecycle_reaches_browser_owner`
- `cargo test -p zero-renderer window_client_callbacks_send_lifecycle_requests`
- `cargo test -p zero-engine test_iframe_content_document_r115`
- `make test`

New focused tests:

- `process_backend::service_worker_owner_tests::renderer_window_client_lifecycle_reaches_browser_owner`
- `ipc_service_worker::tests::window_client_callbacks_send_lifecycle_requests`
- `js_dom_bridge::tests::test_iframe_content_document_r115` now asserts iframe
  Service Worker observe/remove callback emission.

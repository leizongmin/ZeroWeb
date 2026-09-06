# M3-33 window client lifecycle evidence

Date: 2026-08-21

## Scope

This slice exposes browser-owner lifecycle entry points for Service Worker
window clients beyond the committed top-level Document.

Implemented behavior:

- `BrowserServiceWorkerOwner::observe_window_client()` records a same-profile
  `window` client with explicit `frameType` (`top-level`, `auxiliary`, or
  `nested`) after validating URL eligibility through the manager.
- `BrowserServiceWorkerOwner::remove_window_client()` removes exactly one
  known client from both the browser owner tab index and the backing
  `ServiceWorkerManager`.
- Removing a nested frame does not remove the top-level Document or auxiliary
  popup clients that share the same tab.
- Client message queues for the removed client are cleared by the manager,
  while messages for retained clients remain readable.

## Boundary

This does not add new renderer-to-browser IPC for DOM iframe or popup creation.
The production navigation commit path still observes top-level clients
directly. Future work should connect renderer browsing-context creation and
destruction events to these owner methods.

M2 fetch interception remains gated on the js-dom fetch pipeline and Cache API
integration.

## Verification

- `cargo test -p zero-browser window_client_lifecycle_removes_only_destroyed_client -- --nocapture`

New focused test:

- `service_worker_owner::tests::window_client_lifecycle_removes_only_destroyed_client`

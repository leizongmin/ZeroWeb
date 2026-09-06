# M3-30 Service Worker Client FrameType Projection

**日期**：2026-08-21
**状态**：complete

## 实现

- `ServiceWorkerManager` 新增 `observe_window_client_with_frame_type()`，让 host 可以显式投影
  window client 的 `frameType`，现有 `observe_window_client()` 保持 top-level 默认行为。
- manager 在 client 观测信任边界校验 Service Worker `FrameType` 枚举：
  `top-level` / `auxiliary` / `nested`。非法值 fail closed，不进入 client registry。
- IPC `ServiceWorkerClientInfoWire` 校验补齐标准 `auxiliary`，避免后续 popup/auxiliary
  window client 从 browser owner 传给 renderer host 时被协议层误拒。
- browser owner 新增内部 `observe_client_with_frame_type()`，为后续 iframe/popup 接线保留 typed
  入口；当前生产 `begin_request_for_client()` 仍按现有路径投影 top-level document。

## 边界

本切片只完成 `WindowClient.frameType` 元数据与 IPC 边界，不宣称同一 tab 下多个 browsing
context 的完整 client 生命周期。当前 `BrowserServiceWorkerOwner` 仍用 `TabId` 维护断连索引，
真实 iframe/nested client 接入需要把该索引扩展为一 tab 多 client。

## 验证

- `cargo fmt --all -- --check`：通过。
- `cargo test -p zero-page-runtime observed_window_client`：2/2 通过。
- `cargo test -p zero-protocol service_worker_protocol::service_worker_message_port_and_update_wires_round_trip`：1/1 通过。
- `cargo test -p zero-browser service_worker_owner::tests::ipc_clients_match_all_preserves_nested_frame_type`：1/1 通过。
- `cargo clippy -p zero-page-runtime -p zero-protocol -p zero-browser --all-targets -- -D warnings`：通过。
- `make test`：首次 QuickJS service worker runtime 用例出现一次 timeout；定向重跑通过，随后完整重跑通过。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `git diff --check`：通过。

## 下一步

- 把 browser owner 的 `clients_by_tab` 从单 client 改为一 tab 多 client，接入真实 iframe/popup
  lifecycle 后再把上游多 browsing-context `clients.matchAll()`/`clients.get()` 用例提升到 core。

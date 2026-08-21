# M3-31 Service Worker Same-Tab Client Index

**日期**：2026-08-21
**状态**：complete

## 实现

- `BrowserServiceWorkerOwner` 的 `clients_by_tab` 从每个 tab 单 client 扩展为每个 tab 多个
  `BrowserServiceWorkerClientReference`。
- `observe_client_with_frame_type()` 在同一 tab 下保留多个 window client，使 top-level
  document 与 nested iframe client 可同时进入 browser-owned Service Worker client
  registry。
- 同一 client ID 迁移到其他 tab 或 profile 时，会从旧 tab 索引移除旧引用；同一 tab 在
  normal/private profile 间切换时，会移除旧 profile 下的 client。
- `disconnect_tab()` 现在清理该 tab 的全部 Service Worker client，连同 manager 侧
  client messages、pending client messages 和 MessagePort 归属一起释放。
- `set_focused_tab()` 在多 client tab 中优先把 focus 投影给 `top-level` client，其次
  `auxiliary`，最后才回退到其他 client，避免 nested iframe 抢占 tab focus。

## 边界

本切片只补 browser owner 的 tab→clients 索引和断连清理语义。真实 iframe/popup 的创建、
导航替换和销毁事件尚未从 renderer/window lifecycle 自动接到
`observe_client_with_frame_type()`；仍是下一切片。

## 验证

- `cargo check -p zero-browser --tests`：通过。
- `cargo test -p zero-browser service_worker_owner::tests::same_tab_keeps_multiple_service_worker_clients_until_disconnect -- --nocapture`：1/1 通过。
- `cargo fmt --all -- --check`：通过。
- `git diff --check`：通过。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `make test`：通过。

## 下一步

- 将真实 iframe/popup lifecycle 事件接入 browser owner：
  - nested iframe 创建/导航提交时投影 `frameType = "nested"`；
  - popup/auxiliary window 创建时投影 `frameType = "auxiliary"`；
  - browsing context 销毁或导航替换时移除对应 client。

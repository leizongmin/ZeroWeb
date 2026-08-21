# M3-29 Service Worker Clients Focus Ordering

**日期**：2026-08-21
**状态**：complete

## 实现

- `ServiceWorkerManager` 的 committed client registry 记录 window client 的创建顺序与最近
  focus 顺序。
- `clients.matchAll()` 按 Service Worker 规范排序：曾聚焦的 window client 优先，并按最近
  focus 时间倒序；未聚焦 window client 之后按创建顺序；非 window client 保持在 window client
  之后。
- browser owner 新增 `set_focused_tab()`，生产 `ProcessBackend::poll(active_tab, ...)` 每轮把当前
  active tab 投影为 Service Worker client focus state；无 active tab 时清空当前 focus state，但保留
  历史 focus order。

## WPT

- 上游固定 revision `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83` 的
  `clients-matchall-order.https.html` 已复核：核心断言就是 focused window first + recent focus
  order + never-focused creation order。
- 该上游文件仍依赖多 iframe/window harness，当前 ZeroWeb service-worker runner 尚未把完整多
  browsing-context 测试接入 core；本轮用 manager/browser owner 定向测试固定对应排序不变量。

## 验证

- `cargo test -p zero-page-runtime match_all_orders_focused_windows_before_creation_order -- --nocapture`：1/1 通过。
- `cargo test -p zero-browser focused_tab_changes_clients_match_all_order -- --nocapture`：1/1 通过。

## 下一步

- 继续补齐多 browsing-context 的 frameType/nested client 投影，或在 M2 门控解除后推进 fetch
  interception。

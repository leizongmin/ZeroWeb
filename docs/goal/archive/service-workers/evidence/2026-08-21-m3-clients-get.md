# M3-28 Service Worker Clients.get

**日期**：2026-08-21
**状态**：complete

## 实现

- Service Worker global 新增 `clients.get(id)`，经 typed runtime event 查询 browser-owned
  committed client registry。
- `clients.get()` 返回同 origin client 的 `Client` 投影；未知 ID、已移除 client 或 cross-origin
  client resolve `undefined`，不向 worker 泄露其他 origin 的 client。
- browser owner / renderer host / protocol 均新增 append-only `ClientsGetRequested` 与
  `CompleteClientsGet` 通路；同步等待仍在独立 Service Worker host thread，不经过 renderer 主循环。
- embedded WebView 复用同一 manager 查询路径，Document 换代与 disconnect 的清理仍沿用既有
  client registry 逻辑。

## WPT

- 上游固定 revision `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83` 的
  `clients-get.https.html` 已复核：首个 subtest 覆盖 `clients.get()` 基础查询，但同文件后两项依赖
  `FetchEvent.resultingClientId` 与 fetch 事件拦截，仍属 M2 门控。
- 因 runner 以完整上游文件为验收单位，本轮未将该文件提升为 core 分母；core baseline 仍为
  34 case / 156 subtest。

## 验证

- `cargo test -p zero-script-sandbox service_worker::tests::clients_get -- --nocapture`：2/2 通过。
- `cargo test -p zero-page-runtime evaluation_clients_get_returns_same_origin_client_only -- --nocapture`：1/1 通过。
- `cargo test -p zero-protocol service_worker_message_port_and_update_wires_round_trip -- --nocapture`：1/1 通过。
- `cargo test -p zero-browser ipc_clients_get_uses_browser_owned_client_registry -- --nocapture`：1/1 通过。
- `cargo test -p zero-webview clients_get_during_evaluation_reaches_registering_page -- --nocapture`：1/1 通过。
- `make test`：workspace 主矩阵、QuickJS 矩阵与 GPU capability 分支通过。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `make testharness-service-workers-core FILTER=clients-matchall-on-evaluation`：
  既有 core client enumeration WPT smoke 保持 Pass。

## 下一步

- 继续推进多 client ordering/control 语义，或等待 js-dom S6 / storage-cache-api M1 后启动 M2
  fetch interception。

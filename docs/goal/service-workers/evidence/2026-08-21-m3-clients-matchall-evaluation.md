# M3-27 Service Worker Clients.matchAll During Evaluation

**日期**：2026-08-21
**状态**：complete

## 实现

- `ServiceWorkerManager` 保存有界的 committed window client 纯值记录，按 origin、
  controller、`includeUncontrolled` 与 client type 筛选。
- worker global `clients.matchAll()` 经 typed runtime event/host command 查询 browser-owned
  registry；同步等待链路保持在独立 Service Worker host thread，不经过 renderer 主循环。
- `Client.postMessage()` 携带 browser-owned target client ID；worker 顶层求值产生的主动消息
  在 `Evaluated` 前抽取，并按目标 client 分发到独立 cursor log。
- browser navigation/renderer disconnect 与 embedded WebView Document 换代会移除旧 client、
  消息和 MessagePort endpoint。
- Service Worker global 新增由 Rust `url` parser 支撑的 `URL` constructor，供固定 WPT 的
  `normalizeURL()` 使用。

## WPT

- 固定资产：
  [2026-08-21-m3-clients-matchall-evaluation-assets.tsv](2026-08-21-m3-clients-matchall-evaluation-assets.tsv)，
  5/5 asset 固定 revision、字节数和 Git blob SHA。
- `clients-matchall-on-evaluation.https.html`：V8/QuickJS 均 1/1 Pass。
- core baseline：
  [2026-08-21-m3-clients-matchall-evaluation-baseline.json](2026-08-21-m3-clients-matchall-evaluation-baseline.json)，
  34 case / 156 subtest，156 Pass，两轮 deterministic。
- disposition：34 core / 49 defer / 169 gated / 42 skip。

## 验证

- runtime、manager、protocol、browser owner 与 embedded WebView 定向回归通过。
- `RUST_TEST_THREADS=1 make test`、Clippy、GPU adapter 与 CPU/GPU consistency 通过。
- 定向性能报告 `benchmark_20260821_162057.json`：startup 104.28 ms，peak RSS
  155.71 MiB，page p95 18.20 / 596.86 / 119.70 ms，retained form p95 0.0667 ms；
  absolute budgets 通过，本机与共享 baseline CPU 不同，relative gate 不可比较。
- `make browser` 完成全部 release binary 并进入 GPU event loop；无显示环境仅因
  `WAYLAND_DISPLAY`、`WAYLAND_SOCKET` 与 `DISPLAY` 均未设置而退出。

## 下一步

- 实现 `clients.get()` 与多 client ordering/control 语义。
- js-dom S6 与 storage-cache-api M1 完成后启动 M2 fetch interception。

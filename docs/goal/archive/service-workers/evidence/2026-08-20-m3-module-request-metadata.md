# M3-20 Service Worker Script Request Metadata

**日期**：2026-08-20
**状态**：complete

## 实现

- browser-owned Service Worker main script request 携带：
  - `Service-Worker: script`
  - `Sec-Fetch-Mode: same-origin`
- classic imported script 使用 `Sec-Fetch-Mode: no-cors`，module dependency 使用
  `Sec-Fetch-Mode: cors`。
- `updateViaCache != all` 的 main script update 使用 `Cache-Control: no-cache`，
  已缓存 ETag 由共享 HTTP cache 转换为 `If-None-Match` 条件请求。
- embedded WebView 的真实网络 main-script 请求使用同一 header/cache policy。
- WPT 动态 fixture 按请求轮次注入实际应见的 metadata 和唯一版本字节。

## WPT

- 固定资产：
  [2026-08-20-m3-module-request-metadata-assets.tsv](2026-08-20-m3-module-request-metadata-assets.tsv)，
  6/6 asset 校验固定 revision、字节数和 Git blob SHA。
- `update-module-request-mode.https.html`：1/1 Pass。
- `update-no-cache-request-headers.https.html`：1/1 Pass。
- core baseline：
  [2026-08-20-m3-module-request-metadata-baseline.json](2026-08-20-m3-module-request-metadata-baseline.json)，
  26 case / 110 subtest，110 Pass，两轮 deterministic。
- disposition：26 core / 49 defer / 177 gated / 42 skip。

## 验证

- browser-owned fetch 本地 HTTP 回归验证 main script metadata 与 ETag revalidation。
- embedded WebView 本地 HTTP 回归验证相同 main-script headers；V8/QuickJS 均通过。
- workspace Clippy `-D warnings` 与 no-engine browser 编译通过。
- adapter GPU 94/94、CPU/GPU consistency 1/1。
- 定向性能报告 `benchmark_20260820_232351.json`：1/1 microbench，
  startup 90.15 ms，peak RSS 155.58 MiB，page p95 15.16 / 397.86 / 116.74 ms，
  retained form p95 0.0343 ms，absolute budgets 通过。

## 下一步

- 推进 `registration-updateviacache.https.html` 的 main/import HTTP cache 矩阵。
- 继续 MessagePort transfer 与多 client 枚举，或在依赖满足后启动 M2 fetch pipeline。

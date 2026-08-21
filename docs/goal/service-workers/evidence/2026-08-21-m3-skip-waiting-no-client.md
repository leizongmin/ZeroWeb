# M3-26 Service Worker skipWaiting Without Client

**日期**：2026-08-21
**状态**：complete

## 依赖复核

- js-dom S6 高层 API 去字符串仍未完成。
- storage-cache-api M1 仍待启动，页面 `caches` 尚未接线。
- 因此 M2 fetch interception + Cache pipeline 继续保持门控，本阶段推进独立的 M3
  worker-global 控制语义。

## WPT

- 固定资产：
  [2026-08-21-m3-skip-waiting-no-client-assets.tsv](2026-08-21-m3-skip-waiting-no-client-assets.tsv)，
  6/6 asset 固定 revision、字节数和 Git blob SHA。
- `skip-waiting-without-client.https.html`：2/2 Pass。
- worker global 单次调用及 8 次并发调用 `skipWaiting()` 均 resolve `undefined`；
  页面 `service_worker_test` wrapper 通过真实 worker-testharness 结果通道完成。
- core baseline：
  [2026-08-21-m3-skip-waiting-no-client-baseline.json](2026-08-21-m3-skip-waiting-no-client-baseline.json)，
  33 case / 155 subtest，155 Pass，两轮 deterministic。
- disposition：33 core / 49 defer / 170 gated / 42 skip。

## 验证

- `RUST_TEST_THREADS=1 make test` 通过；默认/QuickJS 工作区无失败，GPU adapter 94/94，
  CPU/GPU consistency 1/1。
- `make browser` 完成 `zero-browser`、`zero-renderer`、`zero-compositor` 与
  `zero-image-decoder` release build，并进入 GPU event loop；当前无显示环境仅因
  `WAYLAND_DISPLAY`、`WAYLAND_SOCKET` 与 `DISPLAY` 均未设置而退出。
- 本阶段只扩展固定 WPT corpus、审计脚本与证据，不修改生产执行路径。

## 下一步

- 建立 browser-owned client registry 和动态 query handshake。
- 推进 `clients.matchAll({includeUncontrolled: true})` 及 worker 主动向 client 发消息。

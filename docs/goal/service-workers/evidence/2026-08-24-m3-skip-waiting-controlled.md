# M3-37 Service Worker skipWaiting Controlled Client

**日期**：2026-08-24
**状态**：complete

## 实现

- `ServiceWorkerManager` 在 replacement 进入 activating 时把已受控 client 转移到新
  worker；activation failure 回滚到旧 active。
- 页面与 iframe 的 `navigator.serviceWorker.controller` 在 `controllerchange` 事件期使用
  controller snapshot，保证事件处理器可观测到 activating replacement，而不是 activation
  结束后的 live state。
- iframe `navigator.serviceWorker` 复用 `ServiceWorkerContainer` constructor/prototype，并在
  registration 变化后刷新 iframe wrapper，保持 parent/iframe registration 投影一致。
- worker-testharness 消息轮询覆盖 registration 的 `_worker`、`installing`、`waiting` 与
  `active` slot，并按 worker id 去重，避免 replacement active worker 的结果消息被漏收。
- update candidate 结算后同步 replacement `updateViaCache` 到 incumbent registration，保持
  `registration-updateviacache.https.html` 的 iframe registration 投影不回退。

## WPT

- 固定资产：
  [2026-08-21-m3-skip-waiting-no-client-assets.tsv](2026-08-21-m3-skip-waiting-no-client-assets.tsv)，
  9/9 asset 固定 revision、字节数和 Git blob SHA。
- `skip-waiting-using-registration.https.html`：2/2 Pass。
- `skip-waiting-without-using-registration.https.html` 回归：2/2 Pass。
- `registration-updateviacache.https.html` 回归：25/25 Pass。
- core baseline：
  [2026-08-24-m3-skip-waiting-controlled-baseline.json](2026-08-24-m3-skip-waiting-controlled-baseline.json)，
  37 case / 162 subtest，162 Pass，两轮 deterministic。
- disposition：37 core / 46 defer / 169 gated / 42 skip。

## 验证

- `make testharness-service-workers-core FILTER=skip-waiting-using-registration.https.html`
  通过，2/2 Pass。
- `make testharness-service-workers-core FILTER=skip-waiting-without-using-registration.https.html`
  通过，2/2 Pass。
- `zero-wpt-runner testharness-service-workers ... registration-updateviacache.https.html`
  通过，25/25 Pass。
- `make baseline-wpt-service-workers-core` 输出 37/162 deterministic baseline。

## 下一步

- 继续 M2 fetch/cache WPT 扩面。
- 继续 M3 多 browsing-context 控制语义与 popup/auxiliary 真实窗口接入。

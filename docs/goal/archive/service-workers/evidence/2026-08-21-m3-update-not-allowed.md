# M3-25 Service Worker Update Permissions and MessagePort Transfer

**日期**：2026-08-21
**状态**：complete

## 实现

- Service Worker runtime 的 lifecycle settlement 改为可轮询状态，install
  `waitUntil()` pending 时仍可派发 message event。
- page、browser owner、manager、renderer host 与 worker runtime 共用 typed
  MessagePort endpoint wire，支持 page→worker 和 worker→page 双向 transfer、端口寻址与
  到达早于 `onmessage` 时的消息排队。
- worker global 暴露 `registration.update()`；manager 按调用 worker 的真实状态裁决：
  installing worker 返回 `InvalidStateError`，active worker 可发起 browser-owned fetch
  或复用已有 installing replacement。

## WPT

- 固定资产：
  [2026-08-21-m3-update-not-allowed-assets.tsv](2026-08-21-m3-update-not-allowed-assets.tsv)，
  6/6 asset 固定 revision、字节数和 Git blob SHA，篡改/恢复测试通过。
- `update-not-allowed.https.html`：3/3 Pass。
- core baseline：
  [2026-08-21-m3-update-not-allowed-baseline.json](2026-08-21-m3-update-not-allowed-baseline.json)，
  32 case / 153 subtest，153 Pass，两轮 deterministic。
- disposition：32 core / 49 defer / 171 gated / 42 skip。

## 验证

- V8/QuickJS runtime 与 embedded WebView 覆盖 lifecycle/message 交错、双向 port transfer
  和三项 update 权限矩阵。
- protocol、manager、browser owner 与 renderer host typed wire 回归通过。
- workspace 串行 `make test`、Clippy、adapter GPU 94/94 与 CPU/GPU consistency 1/1
  通过。
- `make browser` 完成 `zero-browser`、`zero-renderer`、`zero-compositor` 与
  `zero-image-decoder` release build，并进入 GPU event loop；当前无显示环境仅因
  `WAYLAND_DISPLAY`、`WAYLAND_SOCKET` 与 `DISPLAY` 均未设置而退出。
- 定向性能报告 `benchmark_20260821_134946.json`：startup 89.92 ms，peak RSS
  156.09 MiB，page p95 15.06 / 403.57 / 121.80 ms，retained form p95 0.0421 ms；
  absolute budgets 通过，本机与共享 baseline CPU 不同，relative gate 不可比较。

## 下一步

- 复核 M2 fetch interception 与 Cache API 依赖。
- 继续多 client enumeration 与控制语义。

# M3-21 Service Worker updateViaCache Matrix

**日期**：2026-08-20
**状态**：complete

## 实现

- 相同 script URL、scope、type 和 `updateViaCache` 的重复 `register()` 在网络前 no-op。
- `updateViaCache` 变化继续执行 update job，成功后同步到现有 registration。
- main script 与 imported script 分别按 `all`、`imports`、`none` 应用缓存策略。
- 更新脚本求值失败时保留原 registration 的 `updateViaCache`。
- classic page script 在真实 global script scope 执行，跨 script 的 async function
  declaration 不再被 `try` block 隐藏；`currentScript` 仍在 Rust 控制流中可靠清理。
- 动态 iframe append 在连接文档后派发 `load`，iframe window 通过独立 registration
  wrapper 投影共享 browser-owned manager 状态。

## WPT

- 固定资产：
  [2026-08-20-m3-update-via-cache-matrix-assets.tsv](2026-08-20-m3-update-via-cache-matrix-assets.tsv)，
  8/8 asset 校验固定 revision、字节数和 Git blob SHA。
- `registration-updateviacache.https.html`：25/25 Pass：
  - 4 个注册后 update cache policy；
  - 16 个 `updateViaCache` 策略切换与 iframe 同步；
  - 4 个注销后属性保持；
  - 1 个失败 update 回滚。
- core baseline：
  [2026-08-20-m3-update-via-cache-matrix-baseline.json](2026-08-20-m3-update-via-cache-matrix-baseline.json)，
  27 case / 135 subtest，135 Pass，两轮 deterministic。
- disposition：27 core / 49 defer / 176 gated / 42 skip。

## 验证

- V8 WebView 648/648、`zero-page-runtime` 75/75。
- QuickJS WebView 601/601。
- workspace Clippy `-D warnings` 通过。
- adapter GPU 94/94、CPU/GPU consistency 1/1。
- 定向性能报告 `benchmark_20260821_000805.json`：2/2 microbench，
  startup 103.74 ms，peak RSS 155.78 MiB，page p95 18.45 / 528.99 / 119.08 ms，
  retained form p95 0.0468 ms，absolute budgets 通过。

## 下一步

- 推进剩余 dynamic update/import failure WPT。
- 继续 MessagePort transfer 与多 client 枚举，或在依赖满足后启动 M2 fetch pipeline。

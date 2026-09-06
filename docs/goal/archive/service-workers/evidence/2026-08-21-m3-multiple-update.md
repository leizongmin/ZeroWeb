# M3-24 Service Worker Multiple Update Coalescing

**日期**：2026-08-21
**状态**：complete

## 实现

- `ServiceWorkerManager::coalesced_update_candidate()` 以 registration key 为边界，
  仅在存在 active/waiting predecessor 时复用 installing candidate，避免把初次
  registration 安装误识别为 update job。
- Browser owner 与 embedded WebView 均在网络 fetch 前检查可合并 job；burst 后续
  `registration.update()` 直接返回同一 candidate，不重复 fetch、创建 runtime 或覆盖 slots。
- candidate 进入 waiting 后不再被视为 in-flight update，后续 update 可重新发起网络检查。

## WPT

- 固定资产：
  [2026-08-21-m3-multiple-update-assets.tsv](2026-08-21-m3-multiple-update-assets.tsv)，
  5/5 asset 校验固定 revision、字节数和 Git blob SHA，篡改/恢复测试通过。
- `multiple-update.https.html`：1/1 Pass，覆盖单次 update、10 路 burst 和 burst 后 update。
- core baseline：
  [2026-08-21-m3-multiple-update-baseline.json](2026-08-21-m3-multiple-update-baseline.json)，
  31 case / 150 subtest，150 Pass，两轮 deterministic。
- disposition：31 core / 49 defer / 172 gated / 42 skip。

## 验证

- Manager coalescing 与 browser owner 零额外 fetch/runtime 定向回归通过。
- V8/QuickJS WebView、workspace Clippy、完整串行 `make test` 通过。
- adapter GPU 94/94、CPU/GPU consistency 1/1。
- `make browser` 完成 browser/renderer/compositor/image-decoder release 构建并进入
  `--renderer=gpu`；当前无 Wayland/X11 display，窗口 event loop 无法创建。
- 定向性能报告 `benchmark_20260821_114326.json`：`zero-webview` 1/1 crate，
  startup 100.62 ms，peak RSS 155.89 MiB，page p95 15.43 / 412.21 / 112.14 ms，
  retained form p95 0.0360 ms；absolute budgets 通过，本机与共享 baseline CPU
  不同，relative gate 标记不可比较。

## 下一步

- 推进 `update-not-allowed.https.html` 的 installing/waiting update restrictions。
- 继续 MessagePort transfer 与多 client 枚举，或在依赖满足后启动 M2 fetch pipeline。

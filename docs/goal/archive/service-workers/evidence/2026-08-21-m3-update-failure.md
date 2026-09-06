# M3-23 Service Worker Update Failure Matrix

**日期**：2026-08-21
**状态**：complete

## 实现

- Browser owner 与 embedded WebView 对 main Service Worker script 执行一致的
  JavaScript MIME 校验；缺失或不支持的 MIME 在启动 candidate 前 fail closed。
- Browser IPC 将 invalid main-script MIME 分类为 `Security`，renderer/WebView 页面
  bridge 将其投影为 `SecurityError`；其他 network、redirect 与 script errors 保持
  `TypeError`。
- Dynamic WPT fixture 按 endpoint 和随机 key 隔离访问次数，精确模拟正常更新、
  invalid MIME、redirect、syntax error、install throw 及主脚本文件缩短。
- fetch/evaluation/install 失败均保留原 active registration；pending uninstall 后的
  update 拒绝，不创建残留 candidate。

## WPT

- 固定资产：
  [2026-08-21-m3-update-failure-assets.tsv](2026-08-21-m3-update-failure-assets.tsv)，
  12/12 asset 校验固定 revision、字节数和 Git blob SHA。
- `update.https.html`：7/7 Pass。
- core baseline：
  [2026-08-21-m3-update-failure-baseline.json](2026-08-21-m3-update-failure-baseline.json)，
  30 case / 149 subtest，149 Pass，两轮 deterministic。
- disposition：30 core / 49 defer / 173 gated / 42 skip。

## 验证

- Browser owner invalid-MIME rollback 与 WebView `SecurityError` 定向回归通过。
- V8/QuickJS WebView、workspace Clippy、完整 `make test` 通过。
- adapter GPU 94/94、CPU/GPU consistency 1/1。
- `make browser` 完成 browser/renderer/compositor/image-decoder release 构建并进入
  `--renderer=gpu` 启动；当前无 Wayland/X11 display，窗口 event loop 无法创建。
- 定向性能报告 `benchmark_20260821_023740.json`：`zero-webview` 1/1 crate，
  startup 97.74 ms，peak RSS 155.73 MiB，page p95 18.16 / 521.69 / 108.38 ms，
  retained form p95 0.0347 ms；absolute budgets 通过，本机与共享 baseline CPU
  不同，relative gate 标记不可比较。

## 下一步

- 推进 `multiple-update.https.html` 的并发 update job coalescing。
- 继续 MessagePort transfer 与多 client 枚举，或在依赖满足后启动 M2 fetch pipeline。

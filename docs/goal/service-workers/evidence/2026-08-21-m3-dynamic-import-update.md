# M3-22 Service Worker Dynamic Import Update

**日期**：2026-08-21
**状态**：complete

## 实现

- WPT runner 按 endpoint 和随机 key 隔离 dynamic update stash，覆盖主脚本文件切换、
  imported script 第二次 404、缓存复用及主脚本移除 `importScripts()`。
- 新 worker graph 加载失败时，candidate 进入 redundant，原 active registration 保留。
- 未受当前 document 控制的 replacement 安装完成后，经既有 `ActivateWaiting` typed
  operation 自动激活；browser renderer 与 embedded WebView 共用同一页面投影判断和
  browser-owned manager lifecycle。
- classic page script 的 `currentScript` 清理不再吞掉原始执行异常。

## WPT

- 固定资产：
  [2026-08-21-m3-dynamic-import-update-assets.tsv](2026-08-21-m3-dynamic-import-update-assets.tsv)，
  17/17 asset 校验固定 revision、字节数和 Git blob SHA，篡改/恢复测试通过。
- `update-import-scripts.https.html`：5/5 Pass。
- `update-missing-import-scripts.https.html`：2/2 Pass。
- core baseline：
  [2026-08-21-m3-dynamic-import-update-baseline.json](2026-08-21-m3-dynamic-import-update-baseline.json)，
  29 case / 142 subtest，142 Pass，两轮 deterministic。
- disposition：29 core / 49 defer / 174 gated / 42 skip。

## 验证

- V8 WebView 649/649；QuickJS WebView 602/602。
- `zero-page-runtime` 75/75；renderer Service Worker IPC 定向测试 3/3。
- workspace Clippy `-D warnings` 通过。
- `make test` 全绿，adapter GPU 94/94、CPU/GPU consistency 1/1。
- `make browser` 完成 browser/renderer/compositor/image-decoder release 构建并进入
  `--renderer=gpu` 启动；当前无 Wayland/X11 display，窗口 event loop 按预期无法创建。
- 定向性能报告 `benchmark_20260821_011104.json`：`zero-webview` 1/1 crate、
  7 个 benchmark group；startup 92.25 ms，peak RSS 154.80 MiB，
  page p95 15.78 / 449.92 / 114.72 ms，retained form p95 0.0394 ms；
  absolute budgets 通过，相对门禁因本机与 baseline CPU 不同标记不可比较。

## 下一步

- 推进 `update.https.html` 的 status、MIME、redirect、syntax/install failure matrix。
- 继续 MessagePort transfer 与多 client 枚举，或在依赖满足后启动 M2 fetch pipeline。

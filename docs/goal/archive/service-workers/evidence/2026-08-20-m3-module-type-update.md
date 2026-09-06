# M3-19 Service Worker Module Type Updates

**日期**：2026-08-20
**状态**：complete

## 实现

- 同 origin/scope 的重复 `register()` 通过共享 manager 执行完整 script graph 比较。
- main script URL、完整 imported graph 与 script type 共同决定是否创建新版本。
- 相同 URL、bytes 和 type 返回现有 registration；`updateViaCache` 仍更新到最新值。
- classic/module 类型变化创建新 worker，求值失败时保留现有 active version。
- module worker 调用 `importScripts()` 明确返回 `TypeError`。
- classic `importScripts()` 的顶层 lexical declaration 可供后续 worker script 访问。
- browser process owner 与 embedded WebView 复用同一 registration update 算法。
- 页面投影采用宿主 snapshot 的真实 state，unchanged registration 不再伪装成 installing。

## WPT

- 固定资产：
  [2026-08-20-m3-module-type-update-assets.tsv](2026-08-20-m3-module-type-update-assets.tsv)，
  8/8 asset 校验固定 revision、字节数和 Git blob SHA。
- `update-registration-with-type.https.html`：7/7 Pass，覆盖：
  - classic → module 与 module → classic；
  - main script bytes 相同时仅 type 变化仍触发更新；
  - main bytes 和 type 均相同时不产生 installing worker；
  - classic script 作为 module、module script 作为 classic 时求值失败。
- core baseline：
  [2026-08-20-m3-module-type-update-baseline.json](2026-08-20-m3-module-type-update-baseline.json)，
  24 case / 108 subtest，108 Pass，两轮 deterministic。
- disposition：24 core / 49 defer / 179 gated / 42 skip。

## 验证

- V8 `zero-script-sandbox` 187/187、`zero-page-runtime` 75/75。
- QuickJS `zero-script-sandbox` 108/108、WebView 599/599。
- browser owner、embedded WebView 与 core manifest 定向回归通过。
- workspace Clippy `-D warnings` 与 no-engine browser 编译通过。
- adapter GPU 94/94、CPU/GPU consistency 1/1。
- 定向性能报告 `benchmark_20260820_224910.json`：1/1 microbench，
  startup 90.15 ms，peak RSS 154.67 MiB，page p95 16.05 / 454.23 / 98.68 ms，
  retained form p95 0.0339 ms，absolute budgets 通过。
- `make browser` 已完成 browser/renderer/compositor/image-decoder release 构建；
  GUI 启动因当前会话无 `DISPLAY`/Wayland socket 而未进入窗口事件循环。

## 下一步

- 审计 module main-script request mode、update no-cache headers 与 `updateViaCache` HTTP cache 矩阵。
- 继续 MessagePort transfer 与多 client 枚举，或在依赖满足后启动 M2 fetch pipeline。

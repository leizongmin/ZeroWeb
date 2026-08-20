# M3-12 Service Worker Event-Time Import Context

**日期**：2026-08-20
**状态**：complete

## 实现

- WebView 将 startup evaluation、snapshot、state changes、client messages、controller 与公开
  runtime poll 的 manager drain 统一到同一 import-completion helper；install event 中首次
  `importScripts()` 不再因宿主丢弃 `ImportScriptsRequested` 而超时。
- event-time fetch 与 response validation 使用 registration 的 main script URL 作为持久
  worker context，不依赖当前页面 URL；页面导航后不会把新 Document 当作 worker fetch context。
- manager 固定 Service Worker script resource map updated flag：evaluation/install 阶段允许
  fetch 新 URL；安装完成后 activate/message 只能回放已有 URL，首次 URL 直接返回
  `NetworkError`，不进入 host fetch 层。
- production browser 保持单 owner 与异步 fetch：registration response 完成、
  `pending_evaluations` 清空后，event-time import 仍通过长期 renderer ownership 映射到原
  host tab，不新增 IPC wire。
- event wave 固定 updated-flag case、worker 与 query-driven Python handler，共 3 asset /
  3,830 bytes。manifest SHA-256：
  `88cefee242e20f03508ca1d9e5590db8a9a046a39b307f1ae44efc4b8a083cdb`。

## WPT

- `import-scripts-updated-flag.https.html`：5/5 Pass，覆盖初始化、evaluation-time replay、
  install-time replay、message-time late import `NetworkError` 与异步 unregister cleanup。
- core baseline：19/19 case、72/72 subtest Pass；V8 与 QuickJS 各连续两轮 deterministic。
- disposition：294 source / 331 URL 确定性重建为 19 core / 49 defer /
  184 gated / 42 skip。
- event assets：3/3 restore、verify-only、缺失/篡改 fail-closed 与恢复回归通过。

## 回归

- manager V8/QuickJS：install 首次 fetch 与 active message late-import rejection 通过。
- WebView V8/QuickJS：install fetch、activate resource-map replay、message resource-map
  replay 与 worker main script context 通过。
- production IPC：evaluation response 已完成后，event import 仍映射到 owned renderer tab。
- `make test` 全过；adapter GPU 94/94、CPU/GPU consistency、QuickJS WPT runner
  117/117 与 QuickJS Clippy 全过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。

## 性能

`make bench-gate` 报告 `benchmark_20260820_162907.json`：

- 16/16 microbench 通过；
- startup 95.49 ms，peak RSS 155.94 MiB；
- page total p95：15.88 / 427.46 / 118.25 ms；
- retained form p95 0.0417 ms，jank 0；
- 当前主机与固定 baseline CPU 不同，relative gate 不可比较；absolute page-total 与
  retained-form budgets 通过。

## 边界

- classic Service Worker startup/install/activate/message import resource-map 与 updated flag
  已闭合。
- module Service Worker graph、MessagePort transfer、多 client 枚举与 M2
  FetchEvent/Cache pipeline 仍待后续。

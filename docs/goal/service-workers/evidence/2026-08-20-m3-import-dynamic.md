# M3-11 Service Worker Dynamic Import Semantics

**日期**：2026-08-20
**状态**：complete

## 实现

- `ServiceWorkerManager` 将已导入的 canonical URL 与 UTF-8 source 作为 version-local
  script resource map；同一 version 的完整 import batch 全命中时直接回放，混合命中仍交给
  host fetch，避免改变部分 batch 契约。
- WPT adapter 以每个 WebView 独立状态实现固定 revision 的 `redirect.py`、
  `update-worker.py`、`import-scripts-version.py` 与 `import-scripts-get.py`：
  per-key visit count、第二次请求 redirect、redirect target 再计数、每次真实 fetch
  变化的 body 以及 query-driven source 均保持确定性。
- Service Worker global 新增 `URLSearchParams` 与只读 `WorkerLocation`；location 由 Rust
  `url` parser 从 main script URL 生成，覆盖 href/origin/protocol/host/hostname/port/
  pathname/search/hash，不在 JS 中重复实现 URL parser。
- `unregister()` 按 registration key 清理 installing/waiting/active 与 retained versions。
  这修复了稳定 `ServiceWorkerRegistration` identity 通过旧 active ID 注销时遗留 waiting
  candidate、阻止同 scope 后续注册自动激活的问题。
- dynamic wave 固定 2 case、4 worker、4 handler 与 `common/utils.js`，共 11 asset /
  11,109 bytes。manifest SHA-256：
  `a32feeeb7d3dacf3485c1696948b637be1a4b56b347cc267367352807bda1fdc`。

## WPT

- `import-scripts-redirect.https.html`：3/3 Pass，覆盖初次 redirect、redirect body 更新及
  第二次请求才 redirect。
- `import-scripts-resource-map.https.html`：2/2 Pass，覆盖重复同 URL 与多参数
  `importScripts()`。
- core baseline：18/18 case、67/67 subtest Pass；V8 与 QuickJS 各连续两轮 deterministic。
- disposition：294 source / 331 URL 确定性重建为 18 core / 49 defer /
  185 gated / 42 skip。
- dynamic assets：11/11 restore、verify-only、缺失/篡改 fail-closed 与恢复回归通过。

## 回归

- manager V8/QuickJS 各 21/21，包含 resource-map replay 与 registration-key unregister。
- Service Worker runtime V8/QuickJS 各 21/21，包含 URLSearchParams 与 WorkerLocation。
- WPT runner fixture V8/QuickJS 各 6/6。
- `make test` 全过；adapter GPU 94/94、CPU/GPU consistency、QuickJS WPT runner
  117/117 与 QuickJS Clippy 全过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。

## 性能

`make bench-gate` 报告 `benchmark_20260820_150752.json`：

- 16/16 microbench 通过；
- startup 103.20 ms，peak RSS 154.81 MiB；
- page total p95：19.55 / 533.86 / 95.98 ms；
- retained form p95 0.0357 ms，jank 0；
- 当前主机与固定 baseline CPU 不同，relative gate 不可比较；absolute page-total 与
  retained-form budgets 通过。

## 边界

- startup evaluation 期间的 classic import fetch 已由 browser owner/WebView adapter 覆盖；
  install/activate/message event 中首次调用 `importScripts()` 的长期 browser fetch context
  仍待后续。
- module Service Worker graph、MessagePort transfer、多 client 枚举与 M2
  FetchEvent/Cache pipeline 仍待后续。

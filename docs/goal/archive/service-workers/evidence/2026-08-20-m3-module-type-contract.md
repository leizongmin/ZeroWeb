# M3-13 Service Worker Module Type Contract

**日期**：2026-08-20
**状态**：complete

## 实现

- `RegistrationOptions.type` 校验 `classic|module` 并进入 WebView/browser host callback。
- script type 作为 typed value 贯穿 renderer request、browser fetch plan、manager、
  storage registration、snapshot、renderer host command 与 browser-owned persistence。
- update candidate 继承当前 registration type；旧 persistence 记录通过 serde default
  迁移为 classic。
- module graph loader 尚未接入时，WebView local host 与 renderer host 均明确返回
  typed script failure，不再把 module source 静默按 classic script 执行。
- IPC 既有 enum discriminant 保持 append-only；新增 script type 是结构字段和独立 wire enum。

## 验证

- protocol Service Worker 17/17，包含 module register/snapshot/host command round-trip。
- storage Service Worker 40/40；manager 23/23；renderer host 6/6。
- browser 验证 module type 到达 manager，旧 persistence 缺字段恢复为 classic。
- WebView V8/QuickJS 均验证 `{type:'module'}` 明确 TypeError，未发生 classic fallback。
- classic core baseline 保持 V8/QuickJS 19 case、72/72 Pass，两轮 deterministic。
- `make test`、adapter GPU 94/94、CPU/GPU consistency、QuickJS WPT runner 117/117 通过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。

## 性能

`make bench-gate` 报告 `benchmark_20260820_174503.json`：

- 16/16 microbench 通过；
- startup 95.79 ms，peak RSS 154.35 MiB；
- page total p95：17.99 / 499.26 / 122.65 ms；
- retained form p95 0.0443 ms，jank 0；
- 当前主机与固定 baseline CPU 不同，relative gate 不可比较；absolute budgets 通过。

## 下一步

- 在 `ServiceWorkerRuntime` 中复用 module compiler，递归请求并注册静态依赖图。
- browser owner 对 module dependency 使用 module fetch policy，并把完整 graph 纳入
  update bytecheck 与 persistence。
- driving WPT：`registration-scope-module-static-import.https.html`，随后扩展
  `update-bytecheck.https.html` 的 module 四项。

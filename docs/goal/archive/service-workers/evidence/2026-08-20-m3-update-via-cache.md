# M3-8 Service Worker updateViaCache

**日期**：2026-08-20
**状态**：complete

## 实现

- `register(scriptURL, { updateViaCache })` 接受 `imports`、`all` 和 `none`；缺省值为
  `imports`，其他值在页面 API 边界以 `TypeError` 拒绝。
- policy 以 typed enum 穿过 renderer/browser IPC、browser owner、manager registry、
  embedded WebView snapshot 和 production renderer projection；不依赖字符串透传。
- 初次 registration 的 top-level script 始终 bypass HTTP cache。后续
  `ServiceWorkerRegistration.update()` 中，`all` 允许 top-level script 使用 fresh cache，
  `imports` 和 `none` 使用 `Cache-Control: no-cache` 触发 browser-owned loader
  revalidation/bypass。
- policy 属于 scope-keyed registration metadata。changed update 继承当前 registration
  policy，snapshot 和稳定 JS `ServiceWorkerRegistration` identity 同步反映该值。
- normal profile persistence 保存 policy；旧 M3-7 schema 缺少字段时按 `imports` 迁移，
  private profile 仍保持纯内存。

## 回归

- HTTP loader：首次 bypass 获取 `version-1`；允许 cache 的 update 不产生第二次 network
  accept 并复用 fresh `version-1`；再次 bypass 获取 `version-2`。
- browser owner：初次 registration 无条件 bypass；`updateViaCache=all` 的 update 允许
  cache；manager snapshot 保持 `All`。
- persistence：`none` 写盘并在 owner restart 后恢复；旧 JSON 缺失字段恢复为 `imports`。
- WebView V8/QuickJS：`none` 投影到 registration；非法 enum reject `TypeError`。
- fresh renderer：production browser/renderer register 链投影 `updateViaCache === "all"`。
- core WPT：13/13 case、37/37 subtest Pass，连续两轮 deterministic。
- `make test`：V8 WebView 620/620、QuickJS 573/573、QuickJS WPT runner 113/113、
  adapter GPU 94/94、CPU/GPU consistency 和 fresh peers 全过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。

## 边界

- 本阶段只完成 top-level classic worker script 的 cache policy。`importScripts()` graph
  尚未抓取、执行、持久化或参与 byte comparison，因此 `imports` 与 `none` 对 imported
  scripts 的差异留待 M3-9。
- module worker graph、周期性 soft update 和 HTTP freshness 上限策略不在本阶段。

## 性能

`make bench-gate` 报告 `benchmark_20260820_074216.json`：

- 16/16 crate、94 个微基准完成，报告未标记 suspect；
- startup：108.56 ms，peak RSS：153.47 MiB；
- page total p95：15.75 / 420.52 / 143.12 ms；
- retained form p95：0.0340 ms，jank 0；
- 当前主机与固定基线 CPU 不同，relative gate 不可比较；absolute page-total 与 retained-form
  budgets 通过。

# M3-3 `clients.claim()`

**日期**：2026-08-20
**状态**：complete

## 实现

- Service Worker global 新增 `Clients`/`clients`，`clients.claim()` 返回 fulfilled Promise。
- activate lifecycle settlement 追加 typed `claim_clients` 信号；manager 只在 activate 成功后
  记录该 version 的 claim 状态，version redundant/unregister 时清除。
- StateChanges 结果附带 `claim_clients`，不改变既有 result 判别值；新增无参数 `Controller`
  operation，判别值为 7，既有 operation 0–6 不变。
- browser owner 的 Controller 查询只使用 committed Document authority；renderer 不能提供或伪造
  client URL。
- 当前 Document URL 命中 active scope 时，页面把 controller 设为该 active worker，并按 task
  派发 `controllerchange`；不命中 scope 时保持 uncontrolled。
- activated task 会重新采样 claim 元数据，避免 lifecycle states 与 activate settlement bit
  在不同 manager poll 中到达造成竞态。

## 回归

- script-sandbox：activate handler 的 `event.waitUntil(clients.claim())` 产生
  `LifecycleSettled { succeeded: true, claim_clients: true }`。
- page-runtime：成功 activated version 保存 claim bit。
- protocol：StateChanges claim bit round-trip；Controller operation append-only 判别值为 7。
- WebView V8/QuickJS：当前 matching Document 从 null controller 切到 active，EventTarget
  `controllerchange` target/currentTarget 与 worker identity 正确；定向稳定性 V8 5/5、QuickJS 3/3。
- fresh renderer：browser-owned committed Document Controller 查询与 active identity 一致。
- Service Worker core baseline：12/12 case、36/36 subtest Pass，连续两轮结果确定。
- `make test`：fresh peers、workspace、94/94 adapter GPU、QuickJS WebView 571/571、
  QuickJS WPT runner 113/113 和 renderer 全过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。

## 性能

`make bench-gate` 报告 `benchmark_20260820_021740.json`：

- 16/16 crate、94 个微基准完成，报告未标记 suspect；
- page total p95：15.13 / 413.00 / 116.50 ms；
- retained form p95：0.0424 ms，jank 0；
- 当前主机与固定基线 CPU 不同，relative gate 不可比较；absolute page-total 与 retained-form
  budgets 通过。

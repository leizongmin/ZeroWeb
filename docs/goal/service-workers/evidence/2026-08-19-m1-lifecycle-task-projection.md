# M1-5b Lifecycle Task Projection

**日期**：2026-08-19
**状态**：complete

## 实现

- `ServiceWorkerManager` 为每个 version 记录 `installed → activating → activated/redundant`
  transition log；日志按 renderer-owned cursor 查询，不使用全局消费队列。
- IPC 追加 `StateChanges` operation/result，既有 nested enum 判别值 0–5 不变，新变体为 6。
- Browser owner 按 committed origin 授权 transition 查询；WebView 使用同一 manager API。
- 页面 `ServiceWorker` 与 `ServiceWorkerRegistration` 继承 `EventTarget`，具备真实
  `Event` target/currentTarget。
- `updatefound` 在 register Promise reaction 后派发；每个 lifecycle transition 独占一个
  timer task。slot 先更新，再派发 `statechange`。
- 同一 version 的 worker identity 在 installing/waiting/active 间保持稳定；unregister 后投影
  `redundant` 并清空 slots。

## WPT 结果

固定 12 case / 36 subtest baseline 从 23 Pass / 12 Fail / 1 Timeout 提升到
30 Pass / 6 Fail / 0 Timeout：

- lifecycle state task：4 Fail + 1 Timeout 全部转绿；
- ServiceWorkerRegistration interface brand：2 Fail 全部转绿；
- Tier A：23/28 Pass；
- next-wave：4/4 Pass；
- static-wave：3/4 Pass；
- 连续两轮 `(case, subtest, status)` 一致。

剩余 6 Fail 均归 M1-5c：scope null/fragment/encoded separator、DOMException shape、
scriptURL fragment normalization。

## 回归

- manager transition log 全量与 cursor suffix；
- protocol round-trip 与 append-only discriminant；
- browser owner origin authorization、transition ordering 与 cursor；
- V8/QuickJS WebView interface brand、`updatefound`、三段 `statechange`、slot ordering、
  unregister `redundant`；
- `make baseline-wpt-service-workers-core` 两轮确定性验证；
- `make test`：fresh peers、workspace V8、94/94 adapter GPU、QuickJS WebView 567 项、
  QuickJS WPT runner 113/113、QuickJS renderer；
- `make bench-gate`：16/16；page total p95 15.17/439.00/117.49 ms；retained form
  p95 0.0461 ms、jank 0；绝对预算通过。

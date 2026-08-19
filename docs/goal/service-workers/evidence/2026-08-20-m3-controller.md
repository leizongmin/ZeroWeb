# M3-2 Document Controller

**日期**：2026-08-20
**状态**：complete

## 实现

- `navigator.serviceWorker.controller` 在每个新 Document 初始化时，按当前 URL 的最长 active
  scope 从 browser-owned manager 或 WebView adapter 查询。
- 当前页面完成首次注册和激活后仍保持 uncontrolled；scope 内后续 Document 才获得 controller。
- controller 与 `registration.active` 投影为同一个 `ServiceWorker` JS 对象。
- 已受控 Document 上的 replacement 经 `skipWaiting()` 激活后，controller 切换到新版本，并在
  后续 task 派发 `controllerchange`；旧 controller 已进入 redundant。
- `ServiceWorkerContainer` 接入 EventTarget，支持 listener 与 `oncontrollerchange` handler。
- register Promise reaction 先于 `updatefound` 和 lifecycle task。轮询调度移入 Promise 内部
  reaction，消除宿主 checkpoint 偶发先投影 installed 的竞态。

## 回归

- WebView V8/QuickJS：首次注册不控制当前页面；scope 内导航后 controller 为 v1；
  `skipWaiting()` replacement 后 controller 为 v2，旧版本 redundant，事件 target/currentTarget 正确。
- replacement Promise ordering 连续 10 轮稳定，完整 WebView 包 V8 617/617、QuickJS 570/570。
- fresh renderer 生产链：新 renderer 从 browser owner 恢复 controller，且与
  `getRegistration().active` identity 一致。
- Service Worker core baseline：12/12 case、36/36 subtest Pass，连续两轮结果确定。
- `make test`：fresh peers、workspace、94/94 adapter GPU、QuickJS WPT runner 113/113 和
  QuickJS renderer 全过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。

## 性能

`make bench-gate` 报告 `benchmark_20260820_012442.json`：

- 16/16 crate、94 个微基准完成，报告未标记 suspect；
- page total p95：20.26 / 540.49 / 106.00 ms；
- retained form p95：0.0507 ms，jank 0；
- 当前主机与固定基线 CPU 不同，relative gate 不可比较；absolute page-total 与 retained-form
  budgets 通过。

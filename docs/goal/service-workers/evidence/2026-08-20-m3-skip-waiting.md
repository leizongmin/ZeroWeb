# M3-1 `skipWaiting()` Activation

**日期**：2026-08-20
**状态**：complete

## 实现

- Service Worker global 的 `skipWaiting()` 记录 worker 级请求，并继续返回 fulfilled Promise。
- install lifecycle settlement 追加 typed `skip_waiting` 信号；V8 与 QuickJS 共用同一 bootstrap
  和 Rust event contract。
- manager 仅在 install 成功 settle 后消费信号；replacement 可直接从 waiting 进入 activating，
  不依赖宿主额外发 `ActivateWaiting` 命令。
- replacement 激活前保留旧 active；新版本 activated 后旧版本转 redundant。replacement
  失败时页面投影恢复旧 active。
- registration JS 对象按 scope 保持稳定，worker JS 对象按 version 更新。

## 回归

- script-sandbox：install handler 通过 `event.waitUntil(skipWaiting())` 产生
  `LifecycleSettled { succeeded: true, skip_waiting: true }`。
- page-runtime：已有 active 时，replacement 请求 `skipWaiting()` 后自动 activated，旧版本
  redundant。
- WebView：`register(v1) → ready → register(v2)` 不调用宿主 activation 命令；v2 成为 active，
  v1 变为 redundant，registration identity 不变。
- `make test`：workspace V8、94/94 adapter GPU、QuickJS WPT runner 113/113 及 renderer 全过。
- Service Worker core baseline：12/12 case、36/36 subtest Pass，连续两轮结果确定。

## 性能

`make bench-gate` 报告 `benchmark_20260820_004605.json`：

- 16/16 crate、94 个微基准完成，报告未标记 suspect；
- page total p95：14.99 / 423.40 / 121.03 ms；
- retained form p95：0.0336 ms，jank 0；
- 当前主机与固定基线 CPU 不同，relative gate 不可比较；absolute page-total 与 retained-form
  budgets 通过。
